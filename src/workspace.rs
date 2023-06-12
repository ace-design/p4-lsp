use std::collections::HashMap;

use tower_lsp::lsp_types::{
    CompletionItem, HoverContents, Location, Position, SemanticTokensResult,
    TextDocumentContentChangeEvent, Url, WorkspaceEdit,
};
use tree_sitter::Parser;
use tree_sitter_p4::language;

use crate::file::File;

pub struct Workspace {
    files: HashMap<Url, File>,
    parser: Parser,
}

impl Workspace {
    pub fn new() -> Workspace {
        let mut parser = Parser::new();
        parser.set_language(language()).unwrap();

        Workspace {
            files: HashMap::new(),
            parser,
        }
    }

    pub fn add_file(&mut self, url: Url, content: &str) {
        let tree = self.parser.parse(content, None);

        self.files
            .insert(url.clone(), File::new(url, content, &tree));
    }

    pub fn update_file(&mut self, url: Url, changes: Vec<TextDocumentContentChangeEvent>) {
        let file = self.files.get_mut(&url).unwrap();

        file.update(changes, &mut self.parser);
    }

    pub fn get_definition_location(&self, url: Url, symbol_position: Position) -> Option<Location> {
        let file = self.files.get(&url)?;

        file.get_definition_location(symbol_position)
    }

    pub fn rename_symbol(
        &mut self,
        url: Url,
        symbol_position: Position,
        new_name: String,
    ) -> Option<WorkspaceEdit> {
        let file = self.files.get_mut(&url).unwrap();

        file.rename_symbol(symbol_position, new_name)
    }

    pub fn get_semantic_tokens(&self, url: Url) -> Option<SemanticTokensResult> {
        let file = self.files.get(&url)?;

        file.get_semantic_tokens()
    }

    pub fn get_completion(&self, url: Url, position: Position) -> Option<Vec<CompletionItem>> {
        let file = self.files.get(&url)?;

        file.get_completion_list(position)
    }

    pub fn get_hover_info(&self, url: Url, position: Position) -> Option<HoverContents> {
        let file = self.files.get(&url)?;

        file.get_hover_info(position)
    }
}
