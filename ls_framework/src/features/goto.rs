use std::sync::{Arc, Mutex};

use crate::metadata::{AstQuery, SymbolTableQuery, Visitable};
use tower_lsp::lsp_types::{Position, Range};

pub fn get_definition_range(
    ast_query: &Arc<Mutex<impl AstQuery>>,
    symbol_table_query: &Arc<Mutex<impl SymbolTableQuery>>,
    position: Position,
) -> Option<Range> {
    let ast_query = ast_query.lock().unwrap();
    let root_visit = ast_query.visit_root();
    let node = root_visit.get_node_at_position(position)?;

    let symbol_table_query = symbol_table_query.lock().unwrap();
    let symbol = symbol_table_query.get_symbol(node.get().linked_symbol.clone()?)?;

    Some(symbol.get_definition_range())
}
