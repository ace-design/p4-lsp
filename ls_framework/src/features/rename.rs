use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::metadata::{
    AstQuery, Symbol, SymbolTableEdit, SymbolTableEditor, SymbolTableQuery, Visitable,
};
use tower_lsp::lsp_types::{Position, TextEdit, Url, WorkspaceEdit};

pub fn rename(
    ast_query: &Arc<Mutex<impl AstQuery>>,
    symbol_table_query: &Arc<Mutex<impl SymbolTableQuery>>,
    symbol_table_editor: &Arc<Mutex<impl SymbolTableEditor>>,
    uri: Url,
    new_name: String,
    position: Position,
) -> Option<WorkspaceEdit> {
    let ast_query = ast_query.lock().unwrap();
    let root_visit = ast_query.visit_root();
    let node = root_visit.get_node_at_position(position)?;
    let previous_name = &node.get().content;

    debug!("previous name: {}", previous_name);

    let symbol = {
        let query = symbol_table_query.lock().unwrap();
        query
            .get_symbol_at_pos(previous_name.clone(), position)?
            .clone()
    };

    let mut editor = symbol_table_editor.lock().unwrap();
    editor.new_edit(SymbolTableEdit::Rename {
        symbol_id: symbol.get_id(),
        new_name: new_name.clone(),
    });

    // TODO: Change this to utilize symbol id

    Some(WorkspaceEdit::new(build_changes(uri, &symbol, new_name)))
}

fn build_changes(uri: Url, symbol: &Symbol, new_name: String) -> HashMap<Url, Vec<TextEdit>> {
    let mut edits: Vec<TextEdit> = Vec::new();

    edits.push(TextEdit::new(
        symbol.get_definition_range(),
        new_name.clone(),
    ));

    for usage in symbol.get_usages() {
        edits.push(TextEdit::new(usage.range.clone(), new_name.clone()));
    }

    HashMap::from([(uri, edits)])
}
