use std::collections::HashMap;

use crate::metadata::{AstQuery, Symbol, SymbolTableEditor, SymbolTableQuery, Visitable};
use tower_lsp::lsp_types::{Position, TextEdit, Url, WorkspaceEdit};

pub fn rename(
    metadata_manager: &mut (impl AstQuery + SymbolTableQuery + SymbolTableEditor),
    uri: Url,
    new_name: String,
    position: Position,
) -> Option<WorkspaceEdit> {
    let root_visit = metadata_manager.visit_root();
    let node = root_visit.get_node_at_position(position)?;
    let previous_name = &node.get().content;

    debug!("previous name: {}", previous_name);

    let symbol = metadata_manager
        .get_symbol_at_pos(previous_name.clone(), position)?
        .clone();
    metadata_manager.rename_symbol(symbol.get_id(), new_name.clone());

    debug!("{:?}", symbol);

    Some(WorkspaceEdit::new(build_changes(uri, &symbol, new_name)))
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
