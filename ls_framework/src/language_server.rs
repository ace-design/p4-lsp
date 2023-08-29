use std::env;
use std::sync::RwLock;

use crate::language_def::{self, LanguageDefinition};
use crate::plugin_manager::PluginManager;
use crate::workspace::Workspace;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

extern crate simplelog;

use simplelog::*;

use std::fs::File;

pub struct Backend {
    client: Client,
    workspace: RwLock<Workspace>,
    plugin_manager: RwLock<PluginManager>,
}

impl Backend {
    pub fn init(client: Client, ts_language: tree_sitter::Language) -> Backend {
        Backend {
            client,
            workspace: Workspace::new(ts_language).into(),
            plugin_manager: PluginManager::new().into(),
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        let log_file_path = env::temp_dir().join(format!(
            "{}-lsf.log",
            LanguageDefinition::get().language.name.to_lowercase()
        ));

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

        std::panic::set_hook(Box::new(|info| {
            error!("{info}");
        }));

        info!("Initializing lsp");

        self.plugin_manager.write().unwrap().load_plugins();

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            range: Some(false),
                            legend: language_def::LanguageDefinition::get_semantic_token_legend(),
                            full: Some(SemanticTokensFullOptions::Delta { delta: Some(true) }),
                            ..Default::default()
                        },
                    ),
                ),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![String::from(".")]),
                    ..Default::default()
                }),
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
        let completion_list = {
            let workspace = self.workspace.read().unwrap();

            (*workspace)
                .get_completion(
                    params.text_document_position.text_document.uri,
                    params.text_document_position.position,
                    params.context,
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

        response
    }

    async fn did_change_configuration(&self, params: DidChangeConfigurationParams) {
        let mut workspace = self.workspace.write().unwrap();
        (*workspace).update_settings(params.settings);
    }
}
