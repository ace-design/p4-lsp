use crate::metadata::ast::{Ast, NodeKind, TypeDecType, Visitable};
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

        let root_table = table.parse_scope(ast);
        debug!("{:?}", root_table);
        let root_id = table.arena.new_node(root_table);

        table.root_id = Some(root_id);

        table
    }

    fn parse_scope(&self, ast: &Ast) -> ScopeSymbolTable {
        ScopeSymbolTable {
            types: self.parse_type_decs(ast),
        }
    }

    fn parse_type_decs(&self, ast: &Ast) -> Vec<Result<Symbol, SymbolError>> {
        let mut types: Vec<Result<Symbol, SymbolError>> = vec![];

        let root_id = ast.get_root_id();
        for node_id in ast.get_child_ids(root_id) {
            let node = ast.get_node(node_id);

            if let NodeKind::TypeDec(type_dec_type) = &node.kind {
                let name_node_id = ast.get_child_of_kind(node_id, NodeKind::Name).unwrap();
                let name = ast.get_node(name_node_id).content.clone();

                let symbol = match type_dec_type {
                    TypeDecType::TypeDef => Ok(Symbol {
                        name,
                        def_position: node.range,
                    }),
                    _ => Err(SymbolError::Unknown),
                };

                types.push(symbol);
            }
        }

        types
    }
}

#[derive(Debug, Default)]
struct ScopeSymbolTable {
    types: Vec<Result<Symbol, SymbolError>>,
}

#[derive(Debug, Default)]
struct Symbol {
    name: String,
    def_position: Range,
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
