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

mod features;
mod file;
mod metadata;
mod settings;
mod utils;

use file::File;
use settings::Settings;

const LANGUAGE_ID: &str = "p4";

struct Backend {
    client: Client,
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
            .time_format("%Y-%m-%d %H:%M:%S.%f") //E.g:%H:%M:%S.%f
            .level("debug")
            .output_file()
            .output_console()
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

        Ok(Some(CompletionResponse::Array(
            completion::get_list(params.text_document_position.position, &file).unwrap_or_default(),
        )))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let file_uri = params.text_document_position_params.text_document.uri;
        let file = self.files.get(&file_uri).unwrap();

        let tree: &Tree = file.tree.as_ref().unwrap();

        let point = utils::pos_to_point(params.text_document_position_params.position);

        let mut node: Node = tree
            .root_node()
            .named_descendant_for_point_range(point, point)
            .unwrap();

        let mut node_hierarchy = node.kind().to_string();
        while node.kind() != "source_file" {
            node = node.parent().unwrap();
            node_hierarchy = [node.kind().into(), node_hierarchy].join(" > ");
        }

        let hover_content = hover::HoverContentBuilder::new()
            .add_text(&node_hierarchy)
            .build();

        Ok(Some(Hover {
            contents: hover_content,
            range: None,
        }))
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let doc = params.text_document;
        if LANGUAGE_ID == &doc.language_id.as_str().to_lowercase() {
            let tree = {
                let mut parser = self.parser.lock().unwrap();
                parser.parse(&doc.text, None)
            };

            self.files
                .insert(doc.uri.clone(), File::new(&doc.text, &tree));

            let diagnotics = self.files.get(&doc.uri).unwrap().get_full_diagnostics();
            self.client
                .publish_diagnostics(doc.uri, diagnotics, None)
                .await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let mut file = self.files.get_mut(&uri).unwrap();

        {
            let parser = self.parser.lock().unwrap();
            file.update(params, parser);
        }

        let diagnotics = file.get_quick_diagnostics();
        self.client.publish_diagnostics(uri, diagnotics, None).await;
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
        settings: Settings::default().into(),
        parser: Mutex::new(parser),
        files: DashMap::new(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
