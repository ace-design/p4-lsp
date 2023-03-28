use crate::ast::{Ast, NodeKind, TypeDecType};
use indextree::{Arena, NodeId};

#[derive(Debug)]
struct SymbolTableTree {
    arena: Arena<SymbolTable>,
    root_id: NodeId,
}

impl SymbolTableTree {
    pub fn new(ast: Ast) {}

    fn parse_type_decs(&self, ast: Ast) {
        for node_id in ast.get_root_nodes() {
            let node = ast.get_node(node_id);

            if let NodeKind::TypeDec(type_dec_type) = &node.kind {
                match type_dec_type {
                    TypeDecType::TypeDef => {}
                    _ => {}
                }
            }
        }
    }
}

#[derive(Debug, Default)]
struct SymbolTable {
    symbols: Vec<String>,
}

impl SymbolTable {
    pub fn new(ast: Ast) -> SymbolTable {
        SymbolTable::default()
    }
}
