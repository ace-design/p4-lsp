use std::sync::{Arc, Mutex};

use tree_sitter::{Node, Tree};

pub struct ScopeNode {
    pub scope: Scope,
    pub children: Vec<Arc<Mutex<ScopeNode>>>,
    pub parent: Option<Arc<Mutex<ScopeNode>>>,
}

impl ScopeNode {
    pub fn new(tree: Option<Tree>, content: &str) -> Option<ScopeNode> {
        if tree.is_none() {
            return None;
        }
        let tree = tree.unwrap();
        let cursor = &mut tree.root_node().walk();

        let mut variables: Vec<String> = vec![];

        for child in tree.root_node().named_children(cursor) {
            match child.kind() {
                "constant_declaration" => {
                    let name_node = child.child_by_field_name("name").unwrap();
                    let name_range = name_node.range();

                    let name: String =
                        content[name_range.start_byte..name_range.end_byte].to_string();

                    variables.push(name);
                }
                _ => {}
            }
        }

        Some(ScopeNode {
            scope: Scope { variables },
            children: vec![],
            parent: None,
        })
    }

    fn parse_scope(node: Node, content: &str) {}

    pub fn variables_in_scope(&self) -> Vec<String> {
        self.scope.variables.clone()
    }
}

pub struct Scope {
    variables: Vec<String>,
}
