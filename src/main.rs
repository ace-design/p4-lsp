use std::sync::{Mutex, RwLock};

use dashmap::DashMap;
use features::{completion, hover, semantic_tokens};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use tree_sitter::{Node, Parser, Tree};
use tree_sitter_p4::language;

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

use file::File;
use settings::Settings;
use workspace::Workspace;

struct Backend {
    client: Client,
    workspace: Mutex<Workspace>,
    settings: RwLock<Settings>,
    parser: Mutex<Parser>,
    files: DashMap<Url, File>,
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
            // .output_console() Interferes with the communication between the LS and vscode
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
        let doc_uri = params.text_document.uri;
        let diagnotics = self.files.get(&doc_uri).unwrap().get_full_diagnostics();
        debug!("Save diags: {:?}", diagnotics);
        self.client
            .publish_diagnostics(doc_uri, diagnotics, None)
            .await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let file_uri = params.text_document_position.text_document.uri;
        let file = self.files.get(&file_uri).unwrap();

        if let Some(metadata) = &file.metadata {
            Ok(Some(CompletionResponse::Array(
                completion::get_list(params.text_document_position.position, metadata)
                    .unwrap_or_default(),
            )))
        } else {
            Ok(None)
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let file_uri = params.text_document_position_params.text_document.uri;
        let file = self.files.get(&file_uri).unwrap();

        if let Some(hover_info) = file.get_hover_info(params.text_document_position_params.position)
        {
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
        let tree = {
            let mut parser = self.parser.lock().unwrap();
            parser.parse(&doc.text, None)
        };

        self.files.insert(
            doc.uri.clone(),
            File::new(doc.uri.clone(), &doc.text, &tree),
        );

        let diagnotics = self.files.get(&doc.uri).unwrap().get_full_diagnostics();
        self.client
            .publish_diagnostics(doc.uri, diagnotics, None)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let mut file = self.files.get_mut(&uri).unwrap();

        {
            let parser = self.parser.lock().unwrap();
            file.update_old(params.content_changes, parser);
        }

        let diagnotics = file.get_quick_diagnostics();
        self.client.publish_diagnostics(uri, diagnotics, None).await;
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri;
        let file = self.files.get_mut(&uri).unwrap();

        let pos = params.text_document_position_params.position;
        let response = if let Some(location) = file.get_definition_location(pos) {
            Some(GotoDefinitionResponse::Scalar(location))
        } else {
            None
        };

        Ok(response)
    }

    async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri;
        let mut file = self.files.get_mut(&uri).unwrap();

        let response =
            Ok(file.rename_symbol(params.text_document_position.position, params.new_name));
        debug!("rename: {:?}", response);

        response
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri.clone();
        let file = self.files.get_mut(&uri).unwrap();

        Ok(file.get_semantic_tokens())
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let mut parser = Parser::new();
    parser.set_language(language()).unwrap();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        workspace: Workspace::new().into(),
        settings: Settings::default().into(),
        parser: parser.into(),
        files: DashMap::new(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
