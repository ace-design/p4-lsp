use indextree::{Arena, NodeId};
use tree_sitter::{Range, Tree};

use super::items::NamedItems;

struct Scope {
    range: Range,
    variables: NamedItems,
}

pub struct ScopeTree {
    arena: Arena<Scope>,
    root_id: Option<NodeId>,
}

impl ScopeTree {
    pub fn new(tree: &Option<Tree>, content: &str) -> Option<ScopeTree> {
        let arena = Arena::new();
        let mut scope_tree = ScopeTree {
            arena,
            root_id: None,
        };

        scope_tree.root_id = Some(scope_tree.parse_scopes(tree.as_ref()?.root_node(), content, 0));

        Some(scope_tree)
    }

    pub fn variables_in_scope(&self, byte_pos: usize) -> NamedItems {
        let mut variables = NamedItems::new();

        let mut id = self.root_id.unwrap();

        let mut subscope_exists = true;
        while subscope_exists {
            subscope_exists = false;

            variables.add_subscope(self.arena.get(id).unwrap().get().variables.clone());

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
        let body_node = get_body_node(current_syntax_node);

        let mut scope = Scope {
            variables: NamedItems::new(),
            range: body_node.range(),
        };

        if current_syntax_node.kind() == "parser_declaration" {
            if let Some(variables) =
                get_parser_declaration_params(current_syntax_node, content, offset)
            {
                scope.variables.add_subscope(variables);
            }
        }

        let mut children: Vec<NodeId> = vec![];

        let cursor = &mut current_syntax_node.walk();
        for child in body_node.named_children(cursor) {
            match child.kind() {
                "constant_declaration" => {
                    let name_node = child.child_by_field_name("name").unwrap();
                    let name_range = name_node.range();

                    let name: String = content
                        [(name_range.start_byte - offset)..(name_range.end_byte - offset)]
                        .to_string();

                    scope.variables.add_constant(name, name_range);
                }
                "variable_declaration" => {
                    let name_node = child.child_by_field_name("name").unwrap();
                    let name_range = name_node.range();

                    let name: String = content
                        [(name_range.start_byte - offset)..(name_range.end_byte - offset)]
                        .to_string();

                    scope.variables.add_variable(name, name_range);
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

fn get_body_node(node: tree_sitter::Node) -> tree_sitter::Node {
    match node.kind() {
        "source_file" => node,
        "parser_declaration" => node.child_by_field_name("body").unwrap(),
        _ => node,
    }
}

fn get_parser_declaration_params(
    node: tree_sitter::Node,
    content: &str,
    offset: usize,
) -> Option<NamedItems> {
    let mut variables = NamedItems::new();

    let parameters = node
        .child_by_field_name("declaration")?
        .child_by_field_name("parameters")?;

    let mut cursor = parameters.walk();
    for param in parameters.named_children(&mut cursor) {
        let name_node = param.child_by_field_name("name").unwrap();
        let name_range = name_node.range();

        let name: String =
            content[(name_range.start_byte - offset)..(name_range.end_byte - offset)].to_string();
        variables.add_variable(name, name_range);
    }

    Some(variables)
}
