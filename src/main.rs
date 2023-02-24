use std::collections::HashMap;
use std::sync::Mutex;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use tree_sitter::{Node, Parser, Tree};
use tree_sitter_p4::language;

mod file;
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

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let files = self.state.files.lock().unwrap();

        let file_uri = params.text_document_position.text_document.uri;
        let file = files.get(&file_uri).unwrap();

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

        let mut variables_text = String::from("Variables in scope:\n");
        let variables = file.scopes.as_ref().unwrap().variables_in_scope();

        for var in variables {
            variables_text.push_str("- ");
            variables_text.push_str(var.as_str());
            variables_text.push_str("\n");
        }

        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(
                [node_hierarchy, variables_text].join("\n\n"),
            )),
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
