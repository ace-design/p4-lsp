use indextree::{Arena, NodeId};

use super::tree::{Ast, Direction, Node, NodeKind, TypeDecType};
use crate::metadata::types::{BaseType, Type};
use crate::utils;

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
            tree,
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
        for child in tree.root_node().named_children(&mut cursor) {
            let new_child = if child.is_error() {
                Some(self.new_error_node(&child))
            } else {
                match child.kind() {
                    "constant_declaration" => self.parse_const_dec(&child),
                    "parser_declaration" => self.parse_parser(&child),
                    "type_declaration" => self.parse_type_dec(&child),
                    _ => None,
                }
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
            self.parse_type_ref(&type_node)
                .unwrap_or_else(|| self.new_error_node(&type_node)),
            &mut self.arena,
        );

        // Add name node
        let name_node = self.arena.new_node(Node::new(
            NodeKind::Name,
            &node.child_by_field_name("name").unwrap(),
            &self.source_code,
        ));
        node_id.append(name_node, &mut self.arena);

        // Add value node
        let value_node = node.child_by_field_name("value").unwrap();
        node_id.append(
            self.parse_value(&value_node)
                .unwrap_or_else(|| self.new_error_node(&type_node)),
            &mut self.arena,
        );

        Some(node_id)
    }

    fn parse_value(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        Some(
            self.arena
                .new_node(Node::new(NodeKind::Value, node, &self.source_code)),
        )
    }

    fn parse_type_dec(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let type_kind_node = node.child_by_field_name("type_kind")?;

        let type_type = match type_kind_node.kind() {
            "typedef_declaration" => TypeDecType::TypeDef,
            "header_type_declaration" => TypeDecType::HeaderType,
            "header_union_declaration" => TypeDecType::HeaderUnion,
            "struct_type_declaration" => TypeDecType::Struct,
            "enum_declaration" => TypeDecType::Enum,
            "parser_type_declaration" => TypeDecType::Parser,
            "control_type_declaration" => TypeDecType::Control,
            "package_type_declaration" => TypeDecType::Package,
            _ => return None,
        };

        let node_id = self.arena.new_node(Node::new(
            NodeKind::TypeDec(type_type.clone()),
            &node,
            &self.source_code,
        ));

        let name_node = self.arena.new_node(Node::new(
            NodeKind::Name,
            &type_kind_node.child_by_field_name("name")?,
            &self.source_code,
        ));
        node_id.append(name_node, &mut self.arena);

        match type_type {
            TypeDecType::TypeDef => {
                let type_node =
                    self.parse_type_ref(&type_kind_node.child_by_field_name("type")?)?;
                node_id.append(type_node, &mut self.arena);
            }
            // TODO: Implement other types
            _ => {}
        }

        Some(node_id)
    }

    fn parse_type_ref(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let child = node.named_child(0)?;
        let type_type: Type = match child.kind() {
            "base_type" => Type::Base(self.parse_base_type(&child)?),
            "type_name" => Type::Name,
            "specialized_type" => Type::Specialized,
            "header_stack_type" => Type::Header,
            "tuple_type" => Type::Tuple,
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

    fn parse_parser(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id = self
            .arena
            .new_node(Node::new(NodeKind::ParserDec, node, &self.source_code));

        let (name_node_id, parameters_node_id) = self
            .parse_parser_type_dec(&node.child_by_field_name("declaration")?)
            .unwrap();
        node_id.append(name_node_id, &mut self.arena);
        node_id.append(parameters_node_id, &mut self.arena);

        let body_syntax_node = &node.child_by_field_name("body")?;
        let body_node_id = self.arena.new_node(Node::new(
            NodeKind::Body,
            body_syntax_node,
            &self.source_code,
        ));
        node_id.append(body_node_id, &mut self.arena);

        let mut cursor = body_syntax_node.walk();
        for syntax_child in body_syntax_node.named_children(&mut cursor) {
            let child_node_id = match syntax_child.kind() {
                "constant_declaration" => self.parse_const_dec(&syntax_child),
                "variable_declaration" => self.parse_var_dec(&syntax_child),
                // TODO: Add instantiation, value_set_declaration and parser_state
                _ => None,
            };

            if let Some(id) = child_node_id {
                body_node_id.append(id, &mut self.arena);
            }
        }

        Some(node_id)
    }

    fn parse_parser_type_dec(&mut self, node: &tree_sitter::Node) -> Option<(NodeId, NodeId)> {
        let name_node_id = self.arena.new_node(Node::new(
            NodeKind::Name,
            &node.child_by_field_name("name")?,
            &self.source_code,
        ));

        let params_syntax_node = node.child_by_field_name("parameters").unwrap();
        let params_node_id = self
            .parse_params(&params_syntax_node)
            .unwrap_or_else(|| self.new_error_node(&params_syntax_node));

        Some((name_node_id, params_node_id))
    }

    fn parse_params(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let params_node_id =
            self.arena
                .new_node(Node::new(NodeKind::Params, &node, &self.source_code));

        let mut cursor = node.walk();
        for syntax_child in node.named_children(&mut cursor) {
            let new_node_id = if syntax_child.is_error() {
                self.new_error_node(&syntax_child)
            } else {
                let param_node_id = self.arena.new_node(Node::new(
                    NodeKind::Param,
                    &syntax_child,
                    &self.source_code,
                ));

                // Add name node
                let name_node_id = self.arena.new_node(Node::new(
                    NodeKind::Name,
                    &syntax_child.child_by_field_name("name")?,
                    &self.source_code,
                ));
                params_node_id.append(name_node_id, &mut self.arena);

                // Add type node
                let type_syntax_node = syntax_child.child_by_field_name("type").unwrap();
                params_node_id.append(
                    self.parse_type_ref(&type_syntax_node)
                        .unwrap_or_else(|| self.new_error_node(&type_syntax_node)),
                    &mut self.arena,
                );

                // Add direction node
                if let Some(value_syntax_node) = syntax_child.child_by_field_name("direction") {
                    param_node_id.append(
                        self.parse_direction(&value_syntax_node)
                            .unwrap_or_else(|| self.new_error_node(&value_syntax_node)),
                        &mut self.arena,
                    )
                };

                // Add value node
                if let Some(value_syntax_node) = syntax_child.child_by_field_name("value") {
                    param_node_id.append(
                        self.parse_value(&value_syntax_node)
                            .unwrap_or_else(|| self.new_error_node(&value_syntax_node)),
                        &mut self.arena,
                    );
                }

                param_node_id
            };

            params_node_id.append(new_node_id, &mut self.arena);
        }

        Some(params_node_id)
    }

    fn parse_direction(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let dir = match utils::get_node_text(node, &self.source_code).as_str() {
            "in" => Direction::In,
            "out" => Direction::Out,
            "inout" => Direction::InOut,
            _ => return None,
        };

        Some(
            self.arena
                .new_node(Node::new(NodeKind::Direction(dir), node, &self.source_code)),
        )
    }

    fn parse_var_dec(&mut self, node: &tree_sitter::Node) -> Option<NodeId> {
        let node_id =
            self.arena
                .new_node(Node::new(NodeKind::VariableDec, node, &self.source_code));

        // Add type node
        let type_node = node.child_by_field_name("type").unwrap();
        node_id.append(
            self.parse_type_ref(&type_node)
                .unwrap_or_else(|| self.new_error_node(&type_node)),
            &mut self.arena,
        );

        // Add name node
        let name_node = self.arena.new_node(Node::new(
            NodeKind::Name,
            &node.child_by_field_name("name").unwrap(),
            &self.source_code,
        ));
        node_id.append(name_node, &mut self.arena);

        // Add value node
        let value_node = node.child_by_field_name("value").unwrap();
        node_id.append(
            self.parse_value(&value_node)
                .unwrap_or_else(|| self.new_error_node(&value_node)),
            &mut self.arena,
        );

        Some(node_id)
    }
}

#[cfg(test)]
mod tests {
    use indextree::Arena;
    use tree_sitter::{Parser, Tree};
    use tree_sitter_p4::language;

    use super::super::tree::{Node, NodeKind, TypeDecType};
    use super::TreesitterTranslator;
    use super::{BaseType, Type};

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

        syntax_node = constant_syntax_node.child_by_field_name("value").unwrap();
        let value = arena.new_node(Node::new(NodeKind::Value, &syntax_node, source_code));
        constant_dec.append(value, &mut arena);

        print_arenas(&arena, &translated_ast.arena);
        assert!(translated_ast.arena.eq(&arena))
    }

    #[test]
    fn test_typedec_typedef() {
        let source_code = r#"
            typedef bit<9> egressSpec_t;
        "#;
        let syntax_tree = get_syntax_tree(source_code);
        let translated_ast =
            TreesitterTranslator::translate(source_code.to_string(), syntax_tree.clone());

        let mut arena: Arena<Node> = Arena::new();
        let mut syntax_node = syntax_tree.root_node();
        let root = arena.new_node(Node::new(NodeKind::Root, &syntax_node, source_code));

        syntax_node = syntax_node.named_child(0).unwrap();
        let typedec_syntax_node = syntax_node;
        let type_dec = arena.new_node(Node::new(
            NodeKind::TypeDec(TypeDecType::TypeDef),
            &syntax_node,
            source_code,
        ));
        root.append(type_dec, &mut arena);

        syntax_node = typedec_syntax_node
            .child(0)
            .unwrap()
            .child_by_field_name("name")
            .unwrap();
        let name_dec = arena.new_node(Node::new(NodeKind::Name, &syntax_node, source_code));
        type_dec.append(name_dec, &mut arena);

        syntax_node = typedec_syntax_node
            .child(0)
            .unwrap()
            .child_by_field_name("type")
            .unwrap();
        let type_type_dec = arena.new_node(Node::new(
            NodeKind::Type(Type::Base(BaseType::SizedBit(Some(9)))),
            &syntax_node,
            source_code,
        ));
        type_dec.append(type_type_dec, &mut arena);

        print_arenas(&arena, &translated_ast.arena);
        assert!(translated_ast.arena.eq(&arena))
    }
}
