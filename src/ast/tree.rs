use indextree::{Arena, NodeId};
use tree_sitter::Range;

struct NamedItem {
    name: String,
    name_def: Range,
    usages: Vec<Range>,
}

struct ControlBlock {}

enum Node {
    Variable(NamedItem),
    Variable_Dec,
    Constant(NamedItem),
    Constant_Dec,
    Parser_Dec(ControlBlock),
}

pub struct Ast {
    arena: Arena<Node>,
    root_id: Option<NodeId>,
}

impl Ast {
    pub fn new(syntax_tree: tree_sitter::Tree, content: &str) -> Option<Ast> {
        let mut tree = Ast {
            arena: Arena::new(),
            root_id: None,
        };

        tree.root_id = tree.parse_syntax_tree(syntax_tree, content);

        Some(tree)
    }

    fn parse_syntax_tree(&self, syntax_tree: tree_sitter::Tree, content: &str) -> Option<NodeId> {
        todo!()
    }
}
