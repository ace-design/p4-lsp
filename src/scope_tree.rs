use std::sync::{Arc, Mutex};

use indextree::{Arena, NodeId};
use tower_lsp::lsp_types::Position;
use tree_sitter::{Range, Tree};

use crate::utils;

struct Scope {
    range: Range,
    variables: Vec<String>,
}

pub struct ScopeTree {
    arena: Arena<Scope>,
    root_id: Option<NodeId>,
}

impl ScopeTree {
    pub fn new(tree: Option<Tree>, content: &str) -> Option<ScopeTree> {
        if tree.is_none() {
            return None;
        }
        let tree = tree.unwrap();

        let arena = Arena::new();
        let mut scope_tree = ScopeTree {
            arena,
            root_id: None,
        };

        scope_tree.root_id = Some(scope_tree.parse_scopes(tree.root_node(), content));

        Some(scope_tree)
    }

    pub fn variables_in_scope(&self, position: Position) -> Vec<String> {
        self.scope_at_position(position).variables.clone()
    }

    fn scope_at_position(&self, _position: Position) -> &Scope {
        // TODO: Complete implementation
        self.arena.get(self.root_id.unwrap()).unwrap().get()
    }

    fn parse_scopes(&mut self, current_syntax_node: tree_sitter::Node, content: &str) -> NodeId {
        let body_node = match current_syntax_node.kind() {
            "source_file" => current_syntax_node,
            "parser_declaration" => current_syntax_node.child_by_field_name("body").unwrap(),
            _ => current_syntax_node,
        };

        let mut scope = Scope {
            variables: vec![],
            range: body_node.range(),
        };

        let mut children: Vec<NodeId> = vec![];

        let cursor = &mut current_syntax_node.walk();
        for child in body_node.named_children(cursor) {
            match child.kind() {
                "constant_declaration" | "variable_declaration" => {
                    let name_node = child.child_by_field_name("name").unwrap();
                    let name_range = name_node.range();

                    let name: String =
                        content[utils::ts_range_to_std_range(name_range)].to_string();

                    scope.variables.push(name);
                }
                // "parser_declaration" => children.push(
                //     self.parse_scopes(child, &content[utils::ts_range_to_std_range(child.range())]),
                // ),
                _ => {}
            }
        }

        let node_id = self.arena.new_node(scope);

        for child in children {
            node_id.append(child, &mut self.arena);
        }

        node_id
    }
}
