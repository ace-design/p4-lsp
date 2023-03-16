use std::sync::MutexGuard;

use tower_lsp::lsp_types::{Diagnostic, DidChangeTextDocumentParams, Position};
use tree_sitter::{InputEdit, Parser, Tree};

use crate::ast::Ast;
use crate::utils;

use crate::scope_parser::ScopeTree;

pub struct File {
    pub content: String,
    pub tree: Option<Tree>,
    pub scopes: Option<ScopeTree>,
    pub ast: Option<Ast>,
}

impl File {
    pub fn new(content: &str, tree: &Option<Tree>) -> File {
        File {
            content: content.to_string(),
            tree: tree.clone(),
            scopes: ScopeTree::new(tree, content),
            ast: Ast::new(tree.to_owned().unwrap(), content),
        }
    }

    pub fn update(&mut self, params: DidChangeTextDocumentParams, mut parser: MutexGuard<Parser>) {
        for change in params.content_changes {
            let mut old_tree: Option<&Tree> = None;
            let text: String;

            if let Some(range) = change.range {
                let start_byte = utils::pos_to_byte(range.start, &self.content);
                let old_end_byte = utils::pos_to_byte(range.end, &self.content);

                let start_position = utils::pos_to_point(range.start);

                let edit = InputEdit {
                    start_byte,
                    old_end_byte: utils::pos_to_byte(range.end, &self.content),
                    new_end_byte: start_byte + change.text.len(),
                    start_position,
                    old_end_position: utils::pos_to_point(range.end),
                    new_end_position: utils::calculate_end_point(start_position, &change.text),
                };

                self.content
                    .replace_range(start_byte..old_end_byte, &change.text);

                text = self.content.clone();
                let tree = self.tree.as_mut().unwrap();
                tree.edit(&edit);
                old_tree = Some(tree);
            } else {
                // If change.range is None, change.text represents the whole file
                text = change.text.clone();
            }

            self.tree = parser.parse(text, old_tree);
        }

        self.scopes = ScopeTree::new(&self.tree, &self.content);
        self.ast = Ast::new(self.tree.to_owned().unwrap(), &self.content);
    }

    pub fn get_diagnotics(&self) -> Vec<Diagnostic> {
        let error_nodes = self.ast.as_ref().unwrap().get_error_nodes();

        error_nodes
            .into_iter()
            .map(|node| {
                let err_msg = node.get_error_msg().unwrap_or("Error".into());
                Diagnostic::new_simple(node.range, err_msg)
            })
            .collect()
    }

    pub fn get_variables_at_pos(&self, position: Position) -> (Vec<String>, Vec<String>) {
        let scopes = self.scopes.as_ref();

        if let Some(scopes) = scopes {
            let items = scopes.items_in_scope(utils::pos_to_byte(position, &self.content));

            (
                items.variables.get_names(position),
                items.constants.get_names(position),
            )
        } else {
            (vec![], vec![])
        }
    }
}
