use std::sync::{Arc, Mutex};

use tower_lsp::lsp_types::{HoverContents, MarkedString, Position};

use crate::metadata::{AstQuery, SymbolTableQuery, Visitable};

pub fn get_hover_info(
    ast_query: &Arc<Mutex<impl AstQuery>>,
    symbol_table_query: &Arc<Mutex<impl SymbolTableQuery>>,
    position: Position,
) -> Option<HoverContents> {
    let ast_query = ast_query.lock().unwrap();
    let root_visit = ast_query.visit_root();
    let node = root_visit.get_node_at_position(position)?;

    let st_query = symbol_table_query.lock().unwrap();
    let symbol = st_query.get_symbol(node.get().linked_symbol.clone()?)?;

    let type_symbol = st_query.get_symbol(symbol.get_type_symbol()?)?;

    Some(HoverContents::Scalar(MarkedString::String(format!(
        "{}: {}",
        symbol.get_name(),
        type_symbol.get_name()
    ))))
}
