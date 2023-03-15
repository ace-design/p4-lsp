use indextree::{Arena, NodeId};

use crate::utils;

use super::tree::{Ast, BaseType, Node, NodeKind, Type};

pub struct TreesitterTranslator {
    arena: Arena<Node>,
    source_code: String,
    tree: tree_sitter::Tree,
}

impl TreesitterTranslator {
    fn new(source_code: String, tree: tree_sitter::Tree) -> TreesitterTranslator {
        TreesitterTranslator {
            arena: Arena::new(),
            source_code,
            tree: tree.into(),
        }
    }

    pub fn translate(source_code: String, tree: tree_sitter::Tree) -> Ast {
        let mut translator = TreesitterTranslator::new(source_code, tree);
        let root_id = translator.parse_root();
        Ast {
            arena: translator.arena,
            root_id: Some(root_id),
        }
    }

    fn new_error_node(&mut self, node: &tree_sitter::Node) -> NodeId {
        self.arena
            .new_node(Node::new(NodeKind::Error, node, &self.source_code))
    }

    fn parse_root(&mut self) -> NodeId {
        let root_syntax_node = self.tree.root_node();
        let ast_root = self.arena.new_node(Node::new(
            NodeKind::Root,
            &root_syntax_node,
            &self.source_code,
        ));

        // TODO: REMOVE CLONE
        let tree = self.tree.clone();
        let mut cursor = tree.walk();
        for child in tree.root_node().children(&mut cursor) {
            let new_child = match child.kind() {
                "constant_declaration" => self.parse_const_dec(&child),
                _ => None,
            };

            if let Some(new_child) = new_child {
                ast_root.append(new_child, &mut self.arena);
            }
        }

        ast_root
    }

    fn parse_const_dec(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::ConstantDec, node, &self.source_code));

        // Add type node
        let type_node = node.child_by_field_name("type").unwrap();
        node_id.append(
            self.parse_type(&type_node)
                .unwrap_or_else(|| self.new_error_node(&type_node)),
            &mut self.arena,
        );

        // Add name node
        node_id.append(
            self.parse_name(&node.child_by_field_name("name").unwrap())
                .unwrap_or_else(|| self.new_error_node(node)),
            &mut self.arena,
        );
        // TODO: Add value node

        Some(node_id)
    }

    fn parse_name(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        Some(
            self.arena
                .new_node(Node::new(NodeKind::Name, node, &self.source_code)),
        )
    }

    fn parse_type(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let child = node.named_child(0)?;
        let type_type: Type = match child.kind() {
            "base_type" => Type::Base(self.parse_base_type(&child)?),
            "type_name" => {
                return None;
            }
            "specialized_type" => {
                return None;
            }
            "header_stack_type" => {
                return None;
            }
            "tuple_type" => {
                return None;
            }
            _ => return None,
        };

        Some(self.arena.new_node(Node::new(
            NodeKind::Type(type_type),
            node,
            &self.source_code,
        )))
    }

    fn parse_base_type(&self, node: &tree_sitter::Node) -> Option<BaseType> {
        let node_text = utils::get_node_text(node, &self.source_code);
        let text = node_text.as_str().trim();

        match text {
            "bool" => Some(BaseType::Bool),
            "int" => Some(BaseType::Int),
            "bit" => Some(BaseType::Bit),
            "string" => Some(BaseType::String),
            "varbit" => Some(BaseType::Varbit),
            "error" => Some(BaseType::Error),
            "match_kind" => Some(BaseType::MatchKind),
            _ => {
                let child = node.named_child(0).unwrap();
                let size = if child.kind() == "integer" {
                    Some(
                        utils::get_node_text(&child, &self.source_code)
                            .parse::<u32>()
                            .unwrap(),
                    )
                } else {
                    None
                };

                if text.starts_with("int") {
                    Some(BaseType::SizedInt(size))
                } else if text.starts_with("bit") {
                    Some(BaseType::SizedBit(size))
                } else if text.starts_with("varbit") {
                    Some(BaseType::SizedVarbit(size))
                } else {
                    None
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use indextree::Arena;
    use tree_sitter::{Parser, Tree};
    use tree_sitter_p4::language;

    use crate::ast::tree::{BaseType, Node, NodeKind, Type};

    use super::TreesitterTranslator;

    fn get_syntax_tree(source_code: &str) -> Tree {
        let mut parser = Parser::new();
        parser.set_language(language()).unwrap();
        parser.parse(source_code, None).unwrap()
    }

    fn print_arenas(expected: &Arena<Node>, actual: &Arena<Node>) {
        println!("Expected:");
        for node in expected.iter() {
            println!("{:?}", node.get());
        }
        println!();
        println!("Actual:");
        for node in actual.iter() {
            println!("{:?}", node.get());
        }
    }

    #[test]
    fn test_const_declaration() {
        let source_code = r#"
            const bit<16> TYPE_IPV4 = 10;
        "#;
        let syntax_tree = get_syntax_tree(source_code);
        let translated_ast =
            TreesitterTranslator::translate(source_code.to_string(), syntax_tree.clone());

        let mut arena: Arena<Node> = Arena::new();
        let mut syntax_node = syntax_tree.root_node();
        let root = arena.new_node(Node::new(NodeKind::Root, &syntax_node, source_code));

        syntax_node = syntax_node.named_child(0).unwrap();
        let constant_syntax_node = syntax_node;
        let constant_dec =
            arena.new_node(Node::new(NodeKind::ConstantDec, &syntax_node, source_code));
        root.append(constant_dec, &mut arena);

        syntax_node = constant_syntax_node.child_by_field_name("type").unwrap();
        let type_dec = arena.new_node(Node::new(
            NodeKind::Type(Type::Base(BaseType::SizedBit(Some(16)))),
            &syntax_node,
            source_code,
        ));

        constant_dec.append(type_dec, &mut arena);

        syntax_node = constant_syntax_node.child_by_field_name("name").unwrap();
        let name_dec = arena.new_node(Node::new(NodeKind::Name, &syntax_node, source_code));

        constant_dec.append(name_dec, &mut arena);

        print_arenas(&arena, &translated_ast.arena);
        assert!(translated_ast.arena.eq(&arena))
    }

    #[test]
    fn test_const_declaration_error() {
        let source_code = r#"
            const bi<16> TYPE_IPV4 = 10;
        "#;
        let syntax_tree = get_syntax_tree(source_code);
        let translated_ast =
            TreesitterTranslator::translate(source_code.to_string(), syntax_tree.clone());

        let mut arena: Arena<Node> = Arena::new();
        let mut syntax_node = syntax_tree.root_node();
        let root = arena.new_node(Node::new(NodeKind::Root, &syntax_node, source_code));

        syntax_node = syntax_node.named_child(0).unwrap();
        let constant_syntax_node = syntax_node;
        let constant_dec =
            arena.new_node(Node::new(NodeKind::ConstantDec, &syntax_node, source_code));
        root.append(constant_dec, &mut arena);

        syntax_node = constant_syntax_node.child_by_field_name("type").unwrap();
        let type_dec = arena.new_node(Node::new(NodeKind::Error, &syntax_node, source_code));

        constant_dec.append(type_dec, &mut arena);

        syntax_node = constant_syntax_node.child_by_field_name("name").unwrap();
        let name_dec = arena.new_node(Node::new(NodeKind::Name, &syntax_node, source_code));

        constant_dec.append(name_dec, &mut arena);

        print_arenas(&arena, &translated_ast.arena);
        assert!(translated_ast.arena.eq(&arena))
    }
}
