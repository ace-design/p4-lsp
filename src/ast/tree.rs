#![allow(dead_code)]

use indextree::{Arena, NodeId};
use tower_lsp::lsp_types::Range;

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
pub enum BaseType {
    Bool,
    Error,
    MatchKind,
    String,
    Int,
    Bit,
    Varbit,
    SizedInt(Option<u32>),
    SizedVarbit(Option<u32>),
    SizedBit(Option<u32>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type {
    Base(BaseType),
    Name,
    Specialized,
    Header,
    Tuple,
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

#[derive(Debug)]
pub struct Ast {
    pub arena: Arena<Node>,
    pub root_id: Option<NodeId>,
}

trait Visitable {
    // add code here
}

impl Visitable for Ast {}

impl Ast {
    pub fn new(syntax_tree: tree_sitter::Tree, source_code: &str) -> Option<Ast> {
        Some(TreesitterTranslator::translate(
            source_code.to_string(),
            syntax_tree,
        ))
    }

    pub fn get_node(&self, node_id: NodeId) -> &Node {
        self.arena.get(node_id).unwrap().get()
    }

    pub fn get_child_of_kind(&self, node_id: NodeId, node_kind: NodeKind) -> Option<NodeId> {
        node_id
            .children(&self.arena)
            .find(|id| self.arena.get(*id).unwrap().get().kind == node_kind)
    }

    pub fn get_root_nodes(&self) -> Vec<NodeId> {
        self.root_id.unwrap().children(&self.arena).collect()
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
}
