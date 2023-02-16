use std::collections::HashMap;
use std::fs;
use std::sync::Mutex;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use tree_sitter::{InputEdit, Parser, Point, Tree};

mod utils;

const LANGUAGE_ID: &str = "p4";

struct File {
    content: String,
    tree: Option<Tree>,
}

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
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        self.client
            .log_message(MessageType::INFO, "server stopped!")
            .await;
        Ok(())
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![
            CompletionItem::new_simple("Hello".to_string(), "Some detail".to_string()),
            CompletionItem::new_simple("Bye".to_string(), "More detail".to_string()),
        ])))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let files = self.state.files.lock().unwrap();

        let file_uri = params.text_document_position_params.text_document.uri;
        let file = files.get(&file_uri).unwrap();

        let tree: &Tree = file.tree.as_ref().unwrap();

        let point = utils::pos_to_point(params.text_document_position_params.position);
        let info: String = tree
            .root_node()
            .named_descendant_for_point_range(point, point)
            .unwrap()
            .kind()
            .to_string();

        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(info)),
            range: None,
        }))
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let mut parser = self.state.parser.lock().unwrap();
        let mut files = self.state.files.lock().unwrap();

        let doc = params.text_document;
        if doc.language_id == LANGUAGE_ID {
            let tree = parser.parse(&doc.text, None);

            files.insert(
                doc.uri,
                File {
                    content: doc.text,
                    tree,
                },
            );
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) -> () {
        let mut files = self.state.files.lock().unwrap();
        let mut parser = self.state.parser.lock().unwrap();

        let file_uri = params.text_document.uri;
        let file_path = file_uri.to_file_path().unwrap();
        let new_file_content = fs::read_to_string(file_path).unwrap();

        for change in params.content_changes {
            let mut old_tree: Option<&Tree> = None;
            let text: &str;

            match change.range {
                Some(range) => {
                    let file = files.get(&file_uri).unwrap();

                    let start_byte = utils::pos_to_byte(range.start, &file.content);
                    let start_position = utils::pos_to_point(range.start);

                    let edit = InputEdit {
                        start_byte,
                        old_end_byte: utils::pos_to_byte(range.end, &file.content),
                        new_end_byte: start_byte + &change.text.len(),
                        start_position,
                        old_end_position: utils::pos_to_point(range.end),
                        new_end_position: new_end_point(start_position, &change.text),
                    };

                    text = &new_file_content;

                    let tree = files.get_mut(&file_uri).unwrap().tree.as_mut().unwrap();
                    tree.edit(&edit);
                    old_tree = Some(tree);
                }
                None => {
                    // If change.range is None, change.text represents the whole file
                    text = &change.text;
                }
            }

            files.get_mut(&file_uri).unwrap().tree = parser.parse(text, old_tree);
        }

        ()
    }
}

fn new_end_point(start: Point, new_content: &str) -> Point {
    let new_lines: Vec<&str> = new_content.lines().collect();

    let column = if new_lines.len() == 1 {
        start.column + new_content.len()
    } else {
        new_lines.last().unwrap().len()
    };

    Point {
        column,
        row: start.row + new_lines.len() - 1,
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let mut parser = Parser::new();
    parser.set_language(tree_sitter_p4::language()).unwrap();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        state: ServerState {
            parser: Mutex::new(parser.into()),
            files: Mutex::new(HashMap::new()),
        },
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
