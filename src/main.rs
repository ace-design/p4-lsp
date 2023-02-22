use std::collections::HashMap;
use std::sync::Mutex;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use tree_sitter::{InputEdit, Node, Parser, Tree};
use tree_sitter_p4::language;

mod nodes;
use nodes::NODE_TYPES;

mod utils;

const LANGUAGE_IDS: [&str; 2] = ["p4", "P4"];

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

        let mut node: Node = tree
            .root_node()
            .named_descendant_for_point_range(point, point)
            .unwrap();

        let mut node_hierarchy = node.kind().to_string();
        while node.kind() != "source_file" {
            node = node.parent().unwrap();
            node_hierarchy = [node.kind().into(), node_hierarchy].join(" > ");
        }

        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(node_hierarchy)),
            range: None,
        }))
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let mut parser = self.state.parser.lock().unwrap();
        let mut files = self.state.files.lock().unwrap();

        let doc = params.text_document;
        if LANGUAGE_IDS.contains(&doc.language_id.as_str()) {
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

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let mut files = self.state.files.lock().unwrap();
        let mut parser = self.state.parser.lock().unwrap();

        let file_uri = params.text_document.uri;

        for change in params.content_changes {
            let mut old_tree: Option<&Tree> = None;
            let text: String;

            match change.range {
                Some(range) => {
                    let file = files.get_mut(&file_uri).unwrap();

                    let start_byte = utils::pos_to_byte(range.start, &file.content);
                    let old_end_byte = utils::pos_to_byte(range.end, &file.content);

                    let start_position = utils::pos_to_point(range.start);

                    let edit = InputEdit {
                        start_byte,
                        old_end_byte: utils::pos_to_byte(range.end, &file.content),
                        new_end_byte: start_byte + change.text.len(),
                        start_position,
                        old_end_position: utils::pos_to_point(range.end),
                        new_end_position: utils::calculate_end_point(start_position, &change.text),
                    };

                    file.content
                        .replace_range(start_byte..old_end_byte, &change.text);

                    text = file.content.clone();
                    let tree = files.get_mut(&file_uri).unwrap().tree.as_mut().unwrap();
                    tree.edit(&edit);
                    old_tree = Some(tree);
                }
                None => {
                    // If change.range is None, change.text represents the whole file
                    text = change.text.clone();
                }
            }

            files.get_mut(&file_uri).unwrap().tree = parser.parse(text, old_tree);
        }
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
