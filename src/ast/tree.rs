#![allow(dead_code)]

use indextree::{Arena, NodeId};
use tower_lsp::lsp_types::Range;

use crate::utils;

use super::translator::TreesitterTranslator;

#[derive(Debug, PartialEq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    //...
}

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
pub enum Type {
    Base(BaseType),
    Name,
    Specialized,
    Header,
    Tuple,
}

#[derive(Debug, PartialEq)]
pub enum NodeKind {
    Root,
    ConstantDec,
    VariableDec,
    ParserDec,
    Type(Type),
    Expression,
    Name,
}

#[derive(Debug, PartialEq)]
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
}

#[derive(Debug)]
pub struct Ast {
    pub arena: Arena<Node>,
    pub root_id: Option<NodeId>,
}

impl Ast {
    pub fn new(syntax_tree: tree_sitter::Tree, source_code: &str) -> Option<Ast> {
        Some(TreesitterTranslator::translate(
            source_code.to_string(),
            syntax_tree,
        ))
    }
}
