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
            tree,
        }
    }

    pub fn translate(source_code: String, tree: tree_sitter::Tree) -> Ast {
        let translator = TreesitterTranslator::new(source_code, tree);

        Ast {
            arena: translator.arena,
            root_id: Some(translator.parse_root()),
        }
    }

    fn parse_root(&mut self) -> NodeId {
        let root = self.tree.root_node();

        let ast_root = self.arena.new_node(Node {
            kind: NodeKind::Root,
            range: utils::ts_range_to_lsp_range(root.range()),
            content: self.source_code,
        });

        let cursor = &mut root.walk();
        for child in root.children(cursor) {
            let new_child = match child.kind() {
                "constant_declaration" => self.parse_const_dec(child),
                _ => None,
            };

            if let Some(new_child) = new_child {
                ast_root.append(new_child, &mut self.arena);
            }
        }

        ast_root
    }

    fn parse_const_dec(&self, node: tree_sitter::Node) -> Option<NodeId> {
        let node_id = self.arena.new_node(Node {
            kind: NodeKind::ConstantDec,
            range: utils::ts_range_to_lsp_range(node.range()),
            content: utils::get_node_text(node, &self.source_code),
        });

        // Add type node
        node_id.append(
            self.parse_type(node.child_by_field_name("type").unwrap())
                .unwrap(),
            &mut self.arena,
        );

        // TODO: Add name node
        // TODO: Add value node

        Some(node_id)
    }

    fn parse_type(&self, node: tree_sitter::Node) -> Option<NodeId> {
        let type_type: Type = match node.kind() {
            "base_type" => Type::Base(BaseType::Int),
            "type_name" => {
                todo!()
            }
            "specialized_type" => {
                todo!()
            }
            "header_stack_type" => {
                todo!()
            }
            "tuple_type" => {
                todo!()
            }
            _ => panic!(),
        };

        Some(self.arena.new_node(Node {
            kind: NodeKind::Type(type_type),
            range: utils::ts_range_to_lsp_range(node.range()),
            content: utils::get_node_text(node, &self.source_code),
        }))
    }
}
