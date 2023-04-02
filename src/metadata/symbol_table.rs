use crate::metadata::ast::{Ast, NodeKind, TypeDecType, Visitable};
use crate::metadata::types::Type;
use indextree::{Arena, NodeId};
use std::{error, fmt};
use tower_lsp::lsp_types::Range;

#[derive(Debug, Default)]
pub struct SymbolTable {
    arena: Arena<ScopeSymbolTable>,
    root_id: Option<NodeId>,
}

impl SymbolTable {
    pub fn new(ast: &Ast) -> SymbolTable {
        let mut table = SymbolTable::default();

        table.root_id = Some(table.parse_scope(ast.get_root_id(), ast));

        table
    }

    fn parse_scope(&mut self, scope_node_id: NodeId, ast: &Ast) -> NodeId {
        let table = ScopeSymbolTable::parse(scope_node_id, ast);

        debug!("{:?}", table);
        let node_id = self.arena.new_node(table);

        for subscope_id in ast.get_subscope_ids(scope_node_id) {
            let subtable = self.parse_scope(subscope_id, ast);
            node_id.append(subtable, &mut self.arena);
        }

        node_id
    }
}

#[derive(Debug, Default)]
struct ScopeSymbolTable {
    range: Range,
    types: Vec<Result<Symbol, SymbolError>>,
    constants: Vec<Result<Symbol, SymbolError>>,
    variables: Vec<Result<Symbol, SymbolError>>,
    functions: Vec<Result<Symbol, SymbolError>>,
}

impl ScopeSymbolTable {
    fn parse(scope_node_id: NodeId, ast: &Ast) -> ScopeSymbolTable {
        let mut table = ScopeSymbolTable {
            range: ast.get_node(scope_node_id).range,
            ..Default::default()
        };

        for node_id in ast.get_child_ids(scope_node_id) {
            let node = ast.get_node(node_id);
            let name_node_id = ast.get_child_of_kind(node_id, NodeKind::Name).unwrap();
            let name = ast.get_node(name_node_id).content.clone();

            let type_ = ast.get_type(node_id);

            match &node.kind {
                NodeKind::ConstantDec => {
                    table
                        .constants
                        .push(Ok(Symbol::new(name, node.range, type_)));
                }
                NodeKind::VariableDec => {
                    table
                        .variables
                        .push(Ok(Symbol::new(name, node.range, type_)));
                }
                NodeKind::TypeDec(type_dec_type) => {
                    let symbol = match type_dec_type {
                        TypeDecType::TypeDef => Ok(Symbol::new(name, node.range, type_)),
                        _ => Err(SymbolError::Unknown),
                    };

                    table.types.push(symbol);
                }
                _ => {}
            }
        }

        table
    }
}

#[derive(Debug)]
struct Symbol {
    name: String,
    def_position: Range,
    type_: Option<Type>,
    usages: Vec<Range>,
}

impl Symbol {
    pub fn new(name: String, def_position: Range, type_: Option<Type>) -> Symbol {
        Symbol {
            name,
            def_position,
            type_,
            usages: vec![],
        }
    }
}

#[derive(Debug)]
enum SymbolError {
    InvalidType,
    Unknown,
}

impl error::Error for SymbolError {}

impl fmt::Display for SymbolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let message = match self {
            SymbolError::InvalidType => "Invalid type.",
            SymbolError::Unknown => "Unknown error.",
        };

        write!(f, "{}", message)
    }
}
