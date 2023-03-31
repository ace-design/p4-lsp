#![allow(dead_code)]

use indextree::{Arena, NodeId};
use tower_lsp::lsp_types::Range;

use crate::metadata::types::Type;
use crate::utils;

use super::translator::TreesitterTranslator;

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    //...
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeDecType {
    TypeDef,
    HeaderType,
    HeaderUnion,
    Struct,
    Enum,
    Parser,
    Control,
    Package,
}

#[derive(Debug, PartialEq, Clone)]
pub enum NodeKind {
    Body,
    Root,
    ConstantDec,
    VariableDec,
    ParserDec,
    Type(Type),
    TypeDec(TypeDecType),
    Expression,
    Name,
    Params,
    Error(Option<String>),
}

const SCOPE_NODES: [NodeKind; 3] = [NodeKind::Root, NodeKind::ParserDec, NodeKind::Body];

impl NodeKind {
    pub fn is_scope_node(&self) -> bool {
        SCOPE_NODES.contains(&self)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub range: Range,
    pub content: String,
}

impl Node {
    pub fn new(kind: NodeKind, syntax_node: &tree_sitter::Node, source_code: &str) -> Node {
        Node {
            kind,
            range: utils::ts_range_to_lsp_range(syntax_node.range()),
            content: utils::get_node_text(syntax_node, source_code),
        }
    }

    pub fn get_error_msg(&self) -> Option<String> {
        match &self.kind {
            NodeKind::Error(Some(msg)) => Some(msg.clone()),
            _ => None,
        }
    }
}

pub trait Visitable {
    fn get_root_id(&self) -> NodeId;
    fn get_node(&self, node_id: NodeId) -> &Node;
    fn get_child_ids(&self, node_id: NodeId) -> Vec<NodeId>;
    fn get_child_of_kind(&self, node_id: NodeId, kind: NodeKind) -> Option<NodeId>;
    fn get_subscope_ids(&self, node_id: NodeId) -> Vec<NodeId>;
}

#[derive(Debug)]
pub struct Ast {
    pub arena: Arena<Node>,
    pub root_id: Option<NodeId>,
}

impl Visitable for Ast {
    fn get_root_id(&self) -> NodeId {
        self.root_id.unwrap()
    }

    fn get_node(&self, node_id: NodeId) -> &Node {
        self.arena.get(node_id).unwrap().get()
    }

    fn get_child_ids(&self, node_id: NodeId) -> Vec<NodeId> {
        node_id.children(&self.arena).collect()
    }

    fn get_subscope_ids(&self, node_id: NodeId) -> Vec<NodeId> {
        self.get_child_ids(node_id)
            .into_iter()
            .filter(|id| self.get_node(*id).kind.is_scope_node())
            .collect::<Vec<NodeId>>()
    }

    fn get_child_of_kind(&self, node_id: NodeId, node_kind: NodeKind) -> Option<NodeId> {
        node_id
            .children(&self.arena)
            .find(|id| self.arena.get(*id).unwrap().get().kind == node_kind)
    }
}

impl Ast {
    pub fn new(source_code: &str, syntax_tree: tree_sitter::Tree) -> Option<Ast> {
        Some(TreesitterTranslator::translate(
            source_code.to_string(),
            syntax_tree,
        ))
    }

    pub fn get_error_nodes(&self) -> Vec<Node> {
        let mut errors: Vec<Node> = vec![];
        for node in self.arena.iter() {
            let node = node.get();
            if let NodeKind::Error(_) = node.kind {
                errors.push(node.clone())
            };
        }
        errors
    }

    pub fn get_type(&self, node_id: NodeId) -> Option<Type> {
        self.get_child_ids(node_id).into_iter().find_map(|id| {
            if let NodeKind::Type(type_) = self.get_node(id).kind {
                Some(type_)
            } else {
                None
            }
        })
    }
}
