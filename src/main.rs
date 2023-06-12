use std::sync::RwLock;

use features::semantic_tokens;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use serde_json::Value;

#[macro_use]
extern crate simple_log;
use simple_log::LogConfigBuilder;

#[macro_use]
extern crate lazy_static;

mod features;
mod file;
mod metadata;
mod plugin_manager;
mod settings;
mod utils;
mod workspace;

use settings::Settings;
use workspace::Workspace;

struct Backend {
    client: Client,
    workspace: RwLock<Workspace>,
    settings: RwLock<Settings>,
}

impl Backend {
    async fn update_settings(&self, settings: Value) {
        self.client
            .log_message(MessageType::INFO, format!("{:?}", settings))
            .await;
        let mut options = self.settings.write().unwrap();
        *options = Settings::parse(settings);
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let config = LogConfigBuilder::builder()
            .path("/tmp/p4-lsp.log")
            .size(100)
            .roll_count(10)
            .time_format("%Y-%m-%d %H:%M:%S.%f")
            .level("debug")
            .output_file()
            .build();

        if simple_log::new(config).is_err() {
            self.client
                .log_message(MessageType::ERROR, "Log file couldn't be created.")
                .await;
        }

        info!("Initializing lsp");
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
                completion_provider: Some(CompletionOptions::default()),
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        will_save: Some(true),
                        will_save_wait_until: Some(true),
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

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        self.update_settings(params.settings).await;
        info!("Settings: {:?}", self.settings.read().unwrap());
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let diagnostics = {
            let workspace = self.workspace.read().unwrap();

            (*workspace).get_full_diagnostics(params.text_document.uri.clone())
        };

        self.client
            .publish_diagnostics(params.text_document.uri, diagnostics, None)
            .await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
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

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let doc = params.text_document;
        info!("Opening file: {}", doc.uri);

        let diagnostics = {
            let mut workspace = self.workspace.write().unwrap();
            (*workspace).add_file(doc.uri.clone(), &doc.text);

            (*workspace).get_full_diagnostics(doc.uri.clone())
        };

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
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        workspace: Workspace::new().into(),
        settings: Settings::default().into(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
