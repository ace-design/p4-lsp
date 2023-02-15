use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use tree_sitter::{Parser, Tree};

struct File {
    path: PathBuf,
    content: String,
    tree: Option<Tree>,
}

struct State {
    parser: Parser,
    files: HashMap<Url, File>,
}

struct Backend {
    client: Client,
    state: Mutex<State>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let mut state = self.state.lock().unwrap();

        let path = params.root_uri.unwrap().to_file_path().unwrap();
        let paths = fs::read_dir(path).unwrap();

        for dir_entry in paths {
            if dir_entry
                .as_ref()
                .unwrap()
                .path()
                .extension()
                .unwrap()
                .to_ascii_lowercase()
                == "p4"
            {
                let file_path = dir_entry.unwrap().path();
                let file_content = fs::read_to_string(file_path.clone()).unwrap();
                let tree = state.parser.parse(file_content.clone(), None);

                state.files.insert(
                    Url::parse(file_path.to_str().unwrap()).unwrap(),
                    File {
                        path: file_path.into(),
                        content: file_content,
                        tree,
                    },
                );
            }
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
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
        let state = self.state.lock().unwrap();
        let file_uri = params.text_document_position_params.text_document.uri;

        let file = state.files.get(&file_uri).unwrap();
        let info: String = file.tree.as_ref().unwrap().root_node().kind().to_string();

        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(info)),
            range: None,
        }))
    }

    async fn did_change(&self, _: DidChangeTextDocumentParams) -> () {
        self.client
            .log_message(MessageType::INFO, "document changed!")
            .await;
        ()
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
        state: Mutex::new(State {
            parser,
            files: HashMap::new(),
        }),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
