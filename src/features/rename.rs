use std::collections::HashMap;

use crate::metadata::{Ast, Symbol, SymbolTable, SymbolTableActions, Visitable};
use tower_lsp::lsp_types::{Position, TextEdit, Url, WorkspaceEdit};

pub fn rename(
    ast: &Ast,
    symbol_table: &mut SymbolTable,
    uri: Url,
    new_name: String,
    position: Position,
) -> Option<WorkspaceEdit> {
    let root_visit = ast.visit_root();
    let node = root_visit.get_node_at_position(position)?;
    let previous_name = &node.get().content;

    debug!("previous name: {}", previous_name);

    let symbol = symbol_table.get_symbol_at_pos_mut(previous_name.clone(), position)?;
    symbol.rename(new_name.clone());

    debug!("{:?}", symbol);

    Some(WorkspaceEdit::new(build_changes(uri, symbol, new_name)))
}

fn build_changes(uri: Url, symbol: &Symbol, new_name: String) -> HashMap<Url, Vec<TextEdit>> {
    let mut edits: Vec<TextEdit> = Vec::new();

    edits.push(TextEdit::new(
        symbol.get_definition_range(),
        new_name.clone(),
    ));

    for range in symbol.get_usages() {
        edits.push(TextEdit::new(*range, new_name.clone()));
    }

    HashMap::from([(uri, edits)])
}
