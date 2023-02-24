use std::sync::{Arc, Mutex};

use tree_sitter::{Node, Tree};

pub struct Scope {
    variables: Vec<String>,
}

pub struct ScopeNode {
    pub scope: Scope,
    pub children: Vec<Arc<Mutex<ScopeNode>>>,
    pub parent: Arc<Mutex<Option<ScopeNode>>>,
}

impl ScopeNode {
    pub fn new(tree: Option<Tree>, content: &str) -> Option<ScopeNode> {
        if tree.is_none() {
            return None;
        }
        let tree = tree.unwrap();

        Some(parse_scope(None, tree.root_node(), content))
    }

    pub fn variables_in_scope(&self) -> Vec<String> {
        self.scope.variables.clone()
    }
}

fn parse_scope(
    parent_node: Option<ScopeNode>,
    current_syntax_node: Node,
    content: &str,
) -> ScopeNode {
    let cursor = &mut current_syntax_node.walk();
    let mut variables: Vec<String> = vec![];
    for child in current_syntax_node.named_children(cursor) {
        match child.kind() {
            "constant_declaration" => {
                let name_node = child.child_by_field_name("name").unwrap();
                let name_range = name_node.range();

                let name: String = content[name_range.start_byte..name_range.end_byte].to_string();

                variables.push(name);
            }
            _ => {}
        }
    }

    ScopeNode {
        scope: Scope { variables },
        children: vec![],
        parent: Arc::from(Mutex::from(parent_node)),
    }
}
