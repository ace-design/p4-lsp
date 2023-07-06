use std::env;
use std::sync::RwLock;

use features::semantic_tokens;
use plugin_manager::PluginManager;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[macro_use]
extern crate log;
extern crate simplelog;

use simplelog::*;

use std::fs::File;

#[macro_use]
extern crate lazy_static;

mod features;
mod file;
mod metadata;
mod plugin_manager;
mod settings;
mod utils;
mod workspace;

use workspace::Workspace;

struct Backend {
    client: Client,
    workspace: RwLock<Workspace>,
    plugin_manager: RwLock<PluginManager>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let log_file_path = env::temp_dir().join("p4-lsp.log");

        if let Ok(log_file) = File::create(log_file_path) {
            let result = WriteLogger::init(
                LevelFilter::Debug,
                ConfigBuilder::new()
                    .add_filter_ignore(String::from("cranelift"))
                    .add_filter_ignore(String::from("wasmtime"))
                    .add_filter_ignore(String::from("extism"))
                    .build(),
                log_file,
            );

            if result.is_err() {
                self.client
                    .log_message(MessageType::ERROR, "Log file couldn't be created.")
                    .await;
            }
        }

        info!("Initializing lsp");

        self.plugin_manager.write().unwrap().load_plugins();

        let mut completion_temp = CompletionOptions::default();
        completion_temp.trigger_characters = Some(vec![".".to_string()]);
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            range: Some(false),
                            legend: semantic_tokens::get_legend(),
                            full: Some(SemanticTokensFullOptions::Delta { delta: Some(true) }),
                            ..Default::default()
                        },
                    ),
                ),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(completion_temp),
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        will_save: Some(false),
                        will_save_wait_until: Some(false),
                        save: Some(TextDocumentSyncSaveOptions::Supported(true)),
                    },
                )),
                definition_provider: Some(OneOf::Left(true)),
                rename_provider: Some(OneOf::Right(RenameOptions {
                    prepare_provider: Some(false),
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                })),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("Lsp initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Lsp stopped");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let doc = params.text_document;
        info!("Opening file: {}", doc.uri);

        let mut diagnostics = {
            let mut workspace = self.workspace.write().unwrap();
            (*workspace).add_file(doc.uri.clone(), &doc.text);

            (*workspace).get_full_diagnostics(doc.uri.clone())
        };

        diagnostics.append(
            &mut self
                .plugin_manager
                .write()
                .unwrap()
                .run_diagnostic(doc.uri.path().into()),
        );

        self.client
            .publish_diagnostics(doc.uri, diagnostics, None)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let diagnostics = {
            let mut workspace = self.workspace.write().unwrap();
            (*workspace).update_file(params.text_document.uri.clone(), params.content_changes);

            (*workspace).get_quick_diagnostics(params.text_document.uri.clone())
        };

        self.client
            .publish_diagnostics(params.text_document.uri, diagnostics, None)
            .await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let mut diagnostics = {
            let workspace = self.workspace.read().unwrap();

            (*workspace).get_full_diagnostics(params.text_document.uri.clone())
        };

        diagnostics.append(
            &mut self
                .plugin_manager
                .write()
                .unwrap()
                .run_diagnostic(params.text_document.uri.path().into()),
        );

        self.client
            .publish_diagnostics(params.text_document.uri, diagnostics, None)
            .await;
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;

        let maybe_location = {
            let workspace = self.workspace.read().unwrap();

            (*workspace).get_definition_location(uri, params.text_document_position_params.position)
        };

        if let Some(location) = maybe_location {
            Ok(Some(GotoDefinitionResponse::Scalar(location)))
        } else {
            Ok(None)
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let maybe_hover_info = {
            let workspace = self.workspace.read().unwrap();

            (*workspace).get_hover_info(
                params.text_document_position_params.text_document.uri,
                params.text_document_position_params.position,
            )
        };

        if let Some(hover_info) = maybe_hover_info {
            Ok(Some(Hover {
                contents: hover_info,
                range: None,
            }))
        } else {
            Ok(None)
        }
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let response = {
            let workspace = self.workspace.read().unwrap();

            Ok((*workspace).get_semantic_tokens(params.text_document.uri))
        };

        response
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        debug!("COMPLETION");
        let completion_list = {
            let workspace = self.workspace.read().unwrap();

            (*workspace)
                .get_completion(
                    params.text_document_position.text_document.uri,
                    params.text_document_position.position,
                )
                .unwrap_or_default()
        };

        Ok(Some(CompletionResponse::Array(completion_list)))
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let response = {
            let mut workspace = self.workspace.write().unwrap();

            Ok((*workspace).rename_symbol(
                params.text_document_position.text_document.uri,
                params.text_document_position.position,
                params.new_name,
            ))
        };

        debug!("rename: {:?}", response);

        response
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        let mut workspace = self.workspace.write().unwrap();
        (*workspace).update_settings(params.settings);
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        workspace: Workspace::new().into(),
        plugin_manager: PluginManager::new().into(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
