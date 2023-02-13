use std::fs;
use std::path::PathBuf;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use tree_sitter::{Parser, Tree};

struct File {
    path: PathBuf,
    content: String,
    tree: Option<Tree>,
}

struct Backend {
    client: Client,
    parser: Parser,
    files: Option<Vec<File>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let path = params.root_uri.unwrap().to_file_path().unwrap();

        let paths = fs::read_dir(path).unwrap();

        let p4_files = paths
            .into_iter()
            .filter(|file| file.as_ref().unwrap().path().extension().unwrap() == "p4")
            .map(|file| {
                let file_path = file.unwrap().path();
                let file_content = fs::read_to_string(file_path.clone()).unwrap();

                File {
                    path: file_path,
                    content: file_content,
                    tree: None,
                }
            })
            .collect::<Vec<File>>();

        self.client
            .log_message(MessageType::INFO, p4_files.len())
            .await;

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

    async fn hover(&self, _: HoverParams) -> Result<Option<Hover>> {
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String("You're hovering!".to_string())),
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
        parser,
        files: None,
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
