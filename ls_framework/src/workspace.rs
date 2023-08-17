use std::collections::HashMap;

use serde_json::Value;
use tower_lsp::lsp_types::{
    CompletionContext, CompletionItem, Diagnostic, HoverContents, Location, Position,
    SemanticTokensResult, TextDocumentContentChangeEvent, Url, WorkspaceEdit,
};

use crate::{file::File, settings::Settings};

pub struct Workspace {
    settings: Settings,
    files: HashMap<Url, File>,
    parser: tree_sitter::Parser,
}

impl Workspace {
    pub fn new(ts_language: tree_sitter::Language) -> Workspace {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(ts_language).unwrap();

        Workspace {
            settings: Settings::default(),
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

    pub fn get_completion(
        &self,
        url: Url,
        position: Position,
        context: Option<CompletionContext>,
    ) -> Option<Vec<CompletionItem>> {
        let file = self.files.get(&url)?;

        file.get_completion_list(position, context)
    }

    pub fn get_hover_info(&self, url: Url, position: Position) -> Option<HoverContents> {
        let file = self.files.get(&url)?;

        file.get_hover_info(position)
    }

    pub fn get_quick_diagnostics(&self, url: Url) -> Vec<Diagnostic> {
        let maybe_file = self.files.get(&url);

        if let Some(file) = maybe_file {
            file.get_quick_diagnostics()
        } else {
            vec![]
        }
    }

    pub fn get_full_diagnostics(&self, url: Url) -> Vec<Diagnostic> {
        let maybe_file = self.files.get(&url);

        if let Some(file) = maybe_file {
            file.get_full_diagnostics()
        } else {
            vec![]
        }
    }

    pub fn update_settings(&mut self, settings: Value) {
        self.settings = Settings::parse(settings);
        info!("Settings: {:?}", self.settings);
    }
}
