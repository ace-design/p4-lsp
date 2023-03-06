use std::collections::HashMap;
use std::sync::Mutex;

use completion::CompletionBuilder;
use hover::HoverContentBuilder;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use tree_sitter::{Node, Parser, Tree};
use tree_sitter_p4::language;

#[macro_use]
extern crate simple_log;
use simple_log::LogConfigBuilder;

mod completion;
mod file;
mod hover;
mod nodes;
mod scope_tree;
mod utils;

use file::File;

const LANGUAGE_IDS: [&str; 2] = ["p4", "P4"];

struct ServerState {
    parser: Mutex<Parser>,
    files: Mutex<HashMap<Url, File>>,
}

struct Backend {
    client: Client,
    state: ServerState,
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
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
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

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let files = self.state.files.lock().unwrap();

        let file_uri = params.text_document_position.text_document.uri;
        let file = files.get(&file_uri).unwrap();

        let (var_names, const_names) =
            file.get_variables_at_pos(params.text_document_position.position);

        let completion_list = CompletionBuilder::new()
            .add(var_names, CompletionItemKind::VARIABLE)
            .add(const_names, CompletionItemKind::CONSTANT)
            .build();

        Ok(Some(CompletionResponse::Array(completion_list)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let files = self.state.files.lock().unwrap();

        let file_uri = params.text_document_position_params.text_document.uri;
        let file = files.get(&file_uri).unwrap();

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

        let (var_names, const_names) =
            file.get_variables_at_pos(params.text_document_position_params.position);

        let hover_content = HoverContentBuilder::new()
            .add_text(node_hierarchy)
            .add_list(var_names, Some("Variables in scope".to_string()))
            .add_list(const_names, Some("Constants in scope".to_string()))
            .build();

        Ok(Some(Hover {
            contents: hover_content,
            range: None,
        }))
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let mut parser = self.state.parser.lock().unwrap();
        let mut files = self.state.files.lock().unwrap();

        let doc = params.text_document;
        if LANGUAGE_IDS.contains(&doc.language_id.as_str()) {
            let tree = parser.parse(&doc.text, None);

            files.insert(doc.uri, File::new(doc.text, tree));
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut files = self.state.files.lock().unwrap();
        let parser = self.state.parser.lock().unwrap();

        let file = files.get_mut(&params.text_document.uri).unwrap();

        file.update(params, parser);
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
        state: ServerState {
            parser: Mutex::new(parser),
            files: Mutex::new(HashMap::new()),
        },
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
