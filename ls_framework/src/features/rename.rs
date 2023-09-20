use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::metadata::{AstQuery, Symbol, SymbolTableQuery, Visitable};
use tower_lsp::lsp_types::{Position, TextEdit, Url, WorkspaceEdit};

pub fn rename(
    ast_query: &Arc<Mutex<impl AstQuery>>,
    symbol_table_query: &Arc<Mutex<impl SymbolTableQuery>>,
    uri: Url,
    new_name: String,
    position: Position,
) -> Option<WorkspaceEdit> {
    let ast_query = ast_query.lock().unwrap();
    let root_visit = ast_query.visit_root();
    let node = root_visit.get_node_at_position(position)?;

    let symbol_table_query = symbol_table_query.lock().unwrap();
    let symbol = symbol_table_query.get_symbol(node.get().linked_symbol.clone()?)?;

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
