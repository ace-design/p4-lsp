use crate::metadata::ast::{Ast, NodeKind, TypeDecType};
use indextree::{Arena, NodeId};

#[derive(Debug, Default)]
pub struct SymbolTable {
    arena: Arena<ScopeSymbolTable>,
    root_id: Option<NodeId>,
}

impl SymbolTable {
    pub fn new(ast: &Ast) -> SymbolTable {
        SymbolTable::default()
    }

    fn parse_type_decs(&self, ast: Ast) {
        for node_id in ast.get_root_nodes() {
            let node = ast.get_node(node_id);

            if let NodeKind::TypeDec(type_dec_type) = &node.kind {
                let name_node_id = ast.get_child_of_kind(node_id, NodeKind::Name).unwrap();
                let name_node = ast.get_node(name_node_id);

                let symbol = match type_dec_type {
                    TypeDecType::TypeDef => Symbol {},
                    _ => Symbol {},
                };
            }
        }
    }
}

#[derive(Debug, Default)]
struct ScopeSymbolTable {
    types: Vec<Result<Symbol, String>>,
}

#[derive(Debug, Default)]
struct Symbol {}

impl ScopeSymbolTable {
    pub fn new(ast: Ast) -> ScopeSymbolTable {
        ScopeSymbolTable::default()
    }
}
