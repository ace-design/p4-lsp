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
}

pub struct Node {
    kind: NodeKind,
    range: Range,
    content: String,
}

pub struct Ast {
    arena: Arena<Node>,
    root_id: Option<NodeId>,
}

impl Ast {
    pub fn new(syntax_tree: tree_sitter::Tree, source_code: &str) -> Option<Ast> {
        let tree = TreesitterTranslator::translate(source_code.to_string(), syntax_tree);

        Some(tree)
    }
}
