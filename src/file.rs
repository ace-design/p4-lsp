use std::sync::MutexGuard;

use tower_lsp::lsp_types::{
    Diagnostic, DidChangeTextDocumentParams, Location, Position, SemanticTokensResult, Url,
    WorkspaceEdit,
};
use tree_sitter::{InputEdit, Parser, Tree};

use crate::features::{diagnostics, goto, rename, semantic_tokens};
use crate::metadata::{Metadata, SymbolTableActions, Symbols};
use crate::utils;

use crate::metadata::Field;

pub struct File {
    pub uri: Url,
    pub source_code: String,
    pub tree: Option<Tree>,
    pub metadata: Option<Metadata>,
}

impl File {
    pub fn new(uri: Url, source_code: &str, tree: &Option<Tree>) -> File {
        File {
            uri,
            source_code: source_code.to_string(),
            tree: tree.clone(),
            metadata: Metadata::new(source_code, tree.as_ref().unwrap().clone()),
        }
    }

    pub fn update(&mut self, params: DidChangeTextDocumentParams, mut parser: MutexGuard<Parser>) {
        for change in params.content_changes {
            let mut old_tree: Option<&Tree> = None;
            let text: String;

            if let Some(range) = change.range {
                let start_byte = utils::pos_to_byte(range.start, &self.source_code);
                let old_end_byte = utils::pos_to_byte(range.end, &self.source_code);

                let start_position = utils::pos_to_point(range.start);

                let edit = InputEdit {
                    start_byte,
                    old_end_byte: utils::pos_to_byte(range.end, &self.source_code),
                    new_end_byte: start_byte + change.text.len(),
                    start_position,
                    old_end_position: utils::pos_to_point(range.end),
                    new_end_position: utils::calculate_end_point(start_position, &change.text),
                };

                self.source_code
                    .replace_range(start_byte..old_end_byte, &change.text);

                text = self.source_code.clone();
                let tree = self.tree.as_mut().unwrap();
                tree.edit(&edit);
                old_tree = Some(tree);
            } else {
                // If change.range is None, change.text represents the whole file
                text = change.text.clone();
            }

            self.tree = parser.parse(text, old_tree);
        }

        self.metadata = Metadata::new(&self.source_code, self.tree.to_owned().unwrap());
    }

    pub fn get_quick_diagnostics(&self) -> Vec<Diagnostic> {
        diagnostics::get_quick_diagnostics(self)
    }

    pub fn get_full_diagnostics(&self) -> Vec<Diagnostic> {
        diagnostics::get_full_diagnostics(self)
    }

    pub fn get_semantic_tokens(&self) -> Option<SemanticTokensResult> {
        Some(semantic_tokens::get_tokens(&self.metadata.as_ref()?.ast))
    }

    pub fn get_definition_location(&self, position: Position) -> Option<Location> {
        let range = goto::get_definition_range(
            &self.metadata.as_ref()?.ast,
            &self.metadata.as_ref()?.symbol_table,
            position,
        )?;
        Some(Location::new(self.uri.clone(), range))
    }

    pub fn rename_symbol(&mut self, position: Position, new_name: String) -> Option<WorkspaceEdit> {
        let metadata = &mut self.metadata.as_mut()?;

        rename::rename(
            &metadata.ast,
            &mut metadata.symbol_table,
            self.uri.clone(),
            new_name,
            position,
        )
    }

    pub fn get_symbols_at_pos(&self, position: Position) -> Symbols {
        return self
            .metadata
            .as_ref()
            .unwrap()
            .symbol_table
            .get_symbols_in_scope(position);
    }

    pub fn get_name_field(&self, position: Position) -> Option<Vec<Field>> {
        self.metadata
            .as_ref()?
            .symbol_table
            .get_variable_in_pos(position, &self.source_code)
    }
}
