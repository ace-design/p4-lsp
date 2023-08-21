#![allow(dead_code)]

use std::fmt;

use indextree::{Arena, NodeId};
use serde::Deserialize;
use tower_lsp::lsp_types::{Position, Range};

use crate::{
    language_def::{self, Symbol},
    lsp_mappings::HighlightType,
    metadata::symbol_table::SymbolId,
    utils,
};

use super::rules_translator::RulesTranslator;

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    //...
}

#[derive(Debug, PartialEq, Clone, Deserialize)]
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

#[derive(Debug, PartialEq, Clone, Deserialize)]
pub enum Direction {
    In,
    Out,
    InOut,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub enum NodeKind {
    Node(String),
    Error(Option<String>),
}

impl NodeKind {
    pub fn is_scope_node(&self) -> bool {
        language_def::LanguageDefinition::get_scope_nodes().contains(self)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub range: Range,
    pub content: String,
    pub symbol: Symbol,
    pub semantic_token_type: Option<HighlightType>,
    pub linked_symbol: Option<SymbolId>,
}

impl Node {
    pub fn new(
        kind: NodeKind,
        syntax_node: &tree_sitter::Node,
        source_code: &str,
        symbol: Symbol,
        semantic_token_type: Option<HighlightType>,
    ) -> Node {
        Node {
            kind,
            range: utils::ts_range_to_lsp_range(syntax_node.range()),
            content: utils::get_node_text(syntax_node, source_code),
            symbol,
            semantic_token_type,
            linked_symbol: None,
        }
    }

    pub fn link(&mut self, symbol_table_id: NodeId, index: usize) {
        self.linked_symbol = Some(SymbolId::new(symbol_table_id, index));
    }
}

pub trait Visitable {
    fn get(&self) -> &Node;
    fn get_id(&self) -> NodeId;
    fn get_children(&self) -> Vec<VisitNode>;
    fn get_descendants(&self) -> Vec<VisitNode>;
    fn get_child_of_kind(&self, kind: NodeKind) -> Option<VisitNode>;
    fn get_subscopes(&self) -> Vec<VisitNode>;
    fn get_node_at_position(&self, position: Position) -> Option<VisitNode>;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct VisitNode<'a> {
    arena: &'a Arena<Node>,
    id: NodeId,
}

impl<'a> VisitNode<'a> {
    pub fn new(arena: &'a Arena<Node>, node_id: NodeId) -> VisitNode<'a> {
        VisitNode { arena, id: node_id }
    }
}

impl Visitable for VisitNode<'_> {
    fn get(&self) -> &Node {
        self.arena.get(self.id).unwrap().get()
    }

    fn get_children(&self) -> Vec<VisitNode> {
        self.id
            .children(self.arena)
            .map(|id| VisitNode::new(self.arena, id))
            .collect::<Vec<VisitNode>>()
    }

    fn get_descendants(&self) -> Vec<VisitNode> {
        self.id
            .descendants(self.arena)
            .map(|id| VisitNode::new(self.arena, id))
            .collect::<Vec<VisitNode>>()
    }

    fn get_child_of_kind(&self, kind: NodeKind) -> Option<VisitNode> {
        let id = self
            .id
            .children(self.arena)
            .find(|id| self.arena.get(*id).unwrap().get().kind == kind)?;

        Some(VisitNode::new(self.arena, id))
    }

    fn get_subscopes(&self) -> Vec<VisitNode> {
        self.get_children()
            .into_iter()
            .filter(|child| child.get().kind.is_scope_node())
            .collect::<Vec<VisitNode>>()
    }

    fn get_node_at_position(&self, position: Position) -> Option<VisitNode> {
        let mut child_id = self.id;

        loop {
            let children = child_id.children(self.arena);
            if let Some(new_child) = children.into_iter().find(|id| {
                let range = self.arena.get(*id).unwrap().get().range;
                position >= range.start && position <= range.end
            }) {
                child_id = new_child;
            } else {
                return Some(VisitNode::new(self.arena, child_id));
            }
        }
    }

    fn get_id(&self) -> NodeId {
        self.id
    }
}

pub trait Translator {
    fn translate(source_code: String, syntax_tree: tree_sitter::Tree) -> Ast;
}

#[derive(Debug, Clone)]
pub struct Ast {
    arena: Arena<Node>,
    root_id: NodeId,
}

impl fmt::Display for Ast {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.get_debug_tree())
    }
}

impl Ast {
    pub fn initialize(arena: Arena<Node>, root_id: NodeId) -> Ast {
        Ast { arena, root_id }
    }

    pub fn new(source_code: &str, syntax_tree: tree_sitter::Tree) -> Option<Ast> {
        Some(RulesTranslator::translate(
            source_code.to_string(),
            syntax_tree,
        ))
    }

    pub fn visit_root(&self) -> VisitNode {
        VisitNode::new(&self.arena, self.root_id)
    }

    pub fn get_debug_tree(&self) -> String {
        let mut result = String::new();
        self._get_debug_tree(self.root_id, "", true, &mut result);
        result
    }

    pub fn get_arena(&mut self) -> &mut Arena<Node> {
        &mut self.arena
    }

    fn _get_debug_tree(&self, node_id: NodeId, indent: &str, last: bool, result: &mut String) {
        let node = self.arena.get(node_id).unwrap().get();
        let line = format!(
            "{}{} {}\n",
            indent,
            if last { "+- " } else { "|- " },
            match node.kind.clone() {
                NodeKind::Node(name) =>
                    if node.linked_symbol.is_some() {
                        format!("{}*", name)
                    } else {
                        name
                    },
                NodeKind::Error(msg) =>
                    format!("Error: {}", msg.unwrap_or(String::from("Unknown"))),
            }
        );

        result.push_str(&line);
        let indent = if last {
            indent.to_string() + "   "
        } else {
            indent.to_string() + "|  "
        };

        // Sorting since nodes aren't parsed in order currently
        let mut children: Vec<NodeId> = node_id.children(&self.arena).collect();
        children.sort_by(|a, b| {
            self.arena
                .get(*a)
                .unwrap()
                .get()
                .range
                .start
                .cmp(&self.arena.get(*b).unwrap().get().range.start)
        });

        for (i, child) in children.into_iter().enumerate() {
            self._get_debug_tree(
                child,
                &indent,
                i == node_id.children(&self.arena).collect::<Vec<_>>().len() - 1,
                result,
            );
        }
    }
}
