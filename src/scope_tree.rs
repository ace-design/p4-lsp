use indextree::{Arena, NodeId};
use tree_sitter::{Range, Tree};

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
        let arena = Arena::new();
        let mut scope_tree = ScopeTree {
            arena,
            root_id: None,
        };

        scope_tree.root_id = Some(scope_tree.parse_scopes(tree?.root_node(), content, 0));

        Some(scope_tree)
    }

    pub fn variables_in_scope(&self, byte_pos: usize) -> Vec<String> {
        let mut variables: Vec<String> = vec![];

        let mut id = self.root_id.unwrap();

        let mut subscope_exists = true;
        while subscope_exists {
            subscope_exists = false;

            variables.append(&mut self.arena.get(id).unwrap().get().variables.clone());

            let children = id.children(&self.arena);
            for child in children {
                let range = self.arena.get(child).unwrap().get().range;
                if range.start_byte <= byte_pos && byte_pos <= range.end_byte {
                    id = child;
                    subscope_exists = true;
                    break;
                }
            }
        }

        variables
    }

    fn parse_scopes(
        &mut self,
        current_syntax_node: tree_sitter::Node,
        content: &str,
        offset: usize,
    ) -> NodeId {
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

                    let name: String = content
                        [(name_range.start_byte - offset)..(name_range.end_byte - offset)]
                        .to_string();

                    scope.variables.push(name);
                }
                "parser_declaration" => children.push(self.parse_scopes(
                    child,
                    &content[child.range().start_byte..child.range().end_byte],
                    child.range().start_byte,
                )),
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
