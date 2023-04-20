use crate::metadata::{Ast, SymbolTable, SymbolTableActions, Visitable};
use tower_lsp::lsp_types::{Position, Range};

pub fn get_definition_range(
    ast: &Ast,
    symbol_table: &SymbolTable,
    position: Position,
) -> Option<Range> {
    let root_visit = ast.visit_root();
    let node = root_visit.get_node_at_position(position)?;

    let symbol = symbol_table.get_symbol_at_pos(node.get().content.clone(), position)?;

    Some(symbol.get_definition_range())
}
