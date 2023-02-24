use std::sync::MutexGuard;

use tower_lsp::lsp_types::{DidChangeTextDocumentParams, Position};
use tree_sitter::{InputEdit, Parser, Tree};

use crate::utils;

use crate::scope_tree::ScopeNode;

pub struct File {
    pub content: String,
    pub tree: Option<Tree>,
    pub scopes: Option<ScopeNode>,
}

impl File {
    pub fn new(content: String, tree: Option<Tree>) -> File {
        File {
            content: content.clone(),
            tree: tree.clone(),
            scopes: ScopeNode::new(tree, &content),
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
    }

    pub fn get_variables_at_pos(&self, _position: Position) -> Vec<String> {
        let scopes = self.scopes.as_ref();

        if let Some(scopes) = scopes {
            scopes.variables_in_scope()
        } else {
            vec![]
        }
    }
}
