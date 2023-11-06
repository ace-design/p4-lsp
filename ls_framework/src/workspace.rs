use std::collections::HashMap;

use serde_json::Value;
use tower_lsp::lsp_types::{
    CompletionContext, CompletionItem, Diagnostic, HoverContents, Location, Position,
    SemanticTokensResult, TextDocumentContentChangeEvent, Url, WorkspaceEdit,
};

use crate::{file::File, settings::Settings};

use crate::file_graph::FileGraph;

use crate::file_graph;

pub struct Workspace {
    settings: Settings,
    graph: FileGraph,
    parser: tree_sitter::Parser,
}

impl Workspace {
    pub fn new(ts_language: tree_sitter::Language) -> Workspace {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(ts_language).unwrap();

        Workspace {
            settings: Settings::default(),
            graph: FileGraph::new(),
            parser,
        }
    }

    pub fn add_file(&mut self, url: Url, content: &str) {
        let tree = self.parser.parse(content, None);
        let index = self.graph.get_next_node_index();

        self.graph.add_node(
            url.to_string().clone(),
            file_graph::Location::Local,
            File::new(url, content, &tree, index),
        );
    }

    pub fn update_file(&mut self, url: Url, changes: Vec<TextDocumentContentChangeEvent>) {
        let file_id = self.graph.find_node_with_url(url.as_str());
        self.graph
            .update_file(file_id.unwrap(), &mut self.parser, changes)
    }

    pub fn get_definition_location(&self, url: Url, symbol_position: Position) -> Option<Location> {
        let file_id = self.graph.find_node_with_url(url.as_str());
        let file = &self.graph.get_node(file_id.unwrap()).unwrap().file;

        file.get_definition_location(symbol_position)
    }

    pub fn rename_symbol(
        &mut self,
        url: Url,
        symbol_position: Position,
        new_name: String,
    ) -> Option<WorkspaceEdit> {
        let file_id = self.graph.find_node_with_url(url.as_str()).unwrap();
        let file = &self.graph.get_mut_node(file_id).unwrap().file;

        file.rename_symbol(symbol_position, new_name)
    }

    pub fn get_semantic_tokens(&self, url: Url) -> Option<SemanticTokensResult> {
        let file_id = self.graph.find_node_with_url(url.as_str()).unwrap();
        let file = &self.graph.get_node(file_id).unwrap().file;

        file.get_semantic_tokens()
    }

    pub fn get_completion(
        &self,
        url: Url,
        position: Position,
        context: Option<CompletionContext>,
    ) -> Option<Vec<CompletionItem>> {
        let file_id = self.graph.find_node_with_url(url.as_str()).unwrap();
        let file = &self.graph.get_node(file_id).unwrap().file;

        file.get_completion_list(position, context)
    }

    pub fn get_hover_info(&self, url: Url, position: Position) -> Option<HoverContents> {
        let file_id = self.graph.find_node_with_url(url.as_str()).unwrap();
        let file = &self.graph.get_node(file_id).unwrap().file;

        file.get_hover_info(position)
    }

    pub fn get_quick_diagnostics(&self, url: Url) -> Vec<Diagnostic> {
        /* let maybe_file = self.files.get(&url);

        if let Some(file) = maybe_file {
            file.get_quick_diagnostics()
        } else {
            vec![]
        }*/
        vec![]
    }

    pub fn get_full_diagnostics(&self, url: Url) -> Vec<Diagnostic> {
        /*let maybe_file = self.files.get(&url);

        if let Some(file) = maybe_file {
            file.get_full_diagnostics()
        } else {
            vec![]
        }*/
        vec![]
    }

    pub fn update_settings(&mut self, settings: Value) {
        self.settings = Settings::parse(settings);
        info!("Settings: {:?}", self.settings);
    }
}
