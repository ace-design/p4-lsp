#![allow(dead_code)]

use indextree::{Arena, NodeId};
use tower_lsp::lsp_types::Range;

use super::translator::TreesitterTranslator;

pub enum BaseType {
    Bool,
    Error,
    MatchKind,
    String,
    Int,
    Bit,
    Varbit,
}

pub enum Type {
    Base(BaseType),
    Name,
    Specialized,
    Header,
    Tuple,
}

pub enum NodeKind {
    Root,
    ConstantDec,
    VariableDec,
    ParserDec,
    Type(Type),
    Expression,
    Name,
}

pub struct Node {
    pub kind: NodeKind,
    pub range: Range,
    pub content: String,
}

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
