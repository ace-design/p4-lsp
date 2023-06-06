use crate::metadata::{AstQuery, SymbolTableQuery, Visitable};
use tower_lsp::lsp_types::{Position, Range};

pub fn get_definition_range(
    ast_query: &impl AstQuery,
    symbol_table_query: &impl SymbolTableQuery,
    position: Position,
) -> Option<Range> {
    let root_visit = ast_query.visit_root();
    let node = root_visit.get_node_at_position(position)?;

    let symbol = symbol_table_query.get_symbol_at_pos(node.get().content.clone(), position)?;

    Some(symbol.get_definition_range())
}
