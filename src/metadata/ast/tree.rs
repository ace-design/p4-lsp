#![allow(dead_code)]

use std::fmt;

use indextree::{Arena, NodeId};
use serde::Deserialize;
use tower_lsp::lsp_types::{Position, Range};

use crate::metadata::types::Type;
use crate::utils;

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
    Body,
    Root,
    ConstantDec,
    VariableDec,
    ParserDec,
    ParserState,
    ControlDec,
    Type(Type),
    TypeList(Type),
    Direction(Direction),
    TypeDec(TypeDecType),
    Expression,
    Name,
    Param,
    Params,
    Field,
    Fields,
    Option,
    Options,
    Error(Option<String>),
    Value,
    ValueSymbol,
    ControlAction,
    Block,
    Assignment,
    Conditional,
    BodyIf,
    BodyElse,
    Return,
    ValueSet,
    TypeArgList,
    TransitionStatement,
    Row,
    Instantiation,
    Obj,
    Function,
    FunctionPrototype,
    FunctionName,
    Switch,
    SwitchCase,
    ParserBlock,
    DirectApplication,
    ControlTable,
    Table,
    Keys,
    Key,
    Actions,
    Action,
    Entries,
    Entrie,
    TableKw,
    PreprocInclude,
    PreprocDefine,
    PreprocUndef,
    ErrorCst,
    Methods,
    Method,
    MethodPrototype,
    Annotations,
    Annotation,
    Token,
    KvList,
    Kv,
    KeyWord,
    ParamType,
    NameStatement,
    StatementDouble,
    StatementExpr,
    StatementDot,
    Extern,
    MatchKind,
    Args,
    Arg,
    EmptyStatement,
    ExitStatement,
}

const SCOPE_NODES: [NodeKind; 17] = [
    NodeKind::Root,
    NodeKind::ParserDec,
    NodeKind::TransitionStatement,
    NodeKind::ControlDec,
    NodeKind::Table,
    NodeKind::Switch,
    NodeKind::Obj,
    NodeKind::Function,
    NodeKind::Block,
    NodeKind::Extern,
    NodeKind::Body,
    NodeKind::ParserState,
    NodeKind::ControlTable,
    NodeKind::Block,
    NodeKind::Instantiation,
    NodeKind::ControlAction,
    NodeKind::SwitchCase,
];

impl NodeKind {
    pub fn is_scope_node(&self) -> bool {
        SCOPE_NODES.contains(self)
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
}

pub trait Visitable {
    fn get(&self) -> &Node;
    fn get_children(&self) -> Vec<VisitNode>;
    fn get_descendants(&self) -> Vec<VisitNode>;
    fn get_child_of_kind(&self, kind: NodeKind) -> Option<VisitNode>;
    fn get_subscopes(&self) -> Vec<VisitNode>;
    fn get_type_node(&self) -> Option<VisitNode>;
    fn get_value_node(&self) -> Option<VisitNode>;
    fn get_value_symbol_node(&self) -> Option<VisitNode>;
    fn get_type(&self) -> Option<Type>;
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

    fn get_type_node(&self) -> Option<VisitNode> {
        self.get_children().into_iter().find_map(|child| {
            let node = child.get();
            if matches!(node.kind, NodeKind::Type(_)) {
                Some(VisitNode::new(self.arena, child.id))
            } else {
                None
            }
        })
    }

    fn get_value_node(&self) -> Option<VisitNode> {
        self.get_children().into_iter().find_map(|child| {
            let node = child.get();
            if matches!(node.kind, NodeKind::Value) {
                Some(VisitNode::new(self.arena, child.id))
            } else {
                None
            }
        })
    }

    fn get_value_symbol_node(&self) -> Option<VisitNode> {
        self.get_children().into_iter().find_map(|child| {
            let node = child.get();
            if matches!(node.kind, NodeKind::ValueSymbol) {
                return Some(VisitNode::new(self.arena, child.id));
            } else {
                None
            }
        })
    }

    fn get_type(&self) -> Option<Type> {
        if let NodeKind::Type(type_) = self.get().kind {
            Some(type_)
        } else {
            None
        }
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

    pub fn get_arena(&self) -> Arena<Node> {
        self.arena.clone()
    }

    fn _get_debug_tree(&self, node_id: NodeId, indent: &str, last: bool, result: &mut String) {
        let node = self.arena.get(node_id).unwrap().get();
        let line = format!(
            "{}{} {:?}\n",
            indent,
            if last { "+- " } else { "|- " },
            node.kind
        );

        result.push_str(&line);
        let indent = if last {
            indent.to_string() + "   "
        } else {
            indent.to_string() + "|  "
        };

        for (i, child) in node_id.children(&self.arena).enumerate() {
            self._get_debug_tree(
                child,
                &indent,
                i == node_id.children(&self.arena).collect::<Vec<_>>().len() - 1,
                result,
            );
        }
    }
}
