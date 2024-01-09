use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::metadata::{
    AstQuery, Symbol, SymbolTableEdit, SymbolTableEditor, SymbolTableQuery, Visitable,
};

use crate::file_graph::FileGraph;
use tower_lsp::lsp_types::{Position, TextEdit, Url, WorkspaceEdit};

pub fn rename(
    ast_query: &Arc<Mutex<impl AstQuery>>,
    _symbol_table_query: &Arc<Mutex<impl SymbolTableQuery>>,
    _symbol_table_editor: &Arc<Mutex<impl SymbolTableEditor>>,
    _uri: Url,
    new_name: String,
    position: Position,
    graph: &FileGraph,
) -> Option<WorkspaceEdit> {
    let ast_query = ast_query.lock().unwrap();
    let root_visit = ast_query.visit_root();
    let node = root_visit.get_node_at_position(position)?;
    let previous_name = &node.get().content;

    let n = node.get().linked_symbol.clone()?;

    let y = graph.get_node(n.file_id).unwrap();
    let cur_url = y.file.uri.clone();
    let x = y.file.symbol_table_manager.lock().unwrap();

    debug!("previous name: {}", previous_name);
    let mut return_changes: Option<WorkspaceEdit> = None;
    let binding = x.get_all_symbols();
    binding.iter().for_each(|s| info!("T:{:?}", &s.get_name()));
    let symbol_exist = binding.iter().find(|s| &s.get_name() == previous_name);

    info!("Symbollldfddyy{:?} ", symbol_exist);
    if let Some(symbol) = symbol_exist {
        info!("Symbollldfddyy{:?} ", symbol);
        let mut editor = x;
        editor.new_edit(SymbolTableEdit::Rename {
            symbol_id: symbol.get_id(),
            new_name: new_name.clone(),
        });

        return_changes = Some(WorkspaceEdit::new(build_changes(
            cur_url, symbol, new_name, graph,
        )))
    }
    return_changes

    // TODO: Change this to utilize symbol id
}

fn build_changes(
    curr: Url,
    symbol: &Symbol,
    new_name: String,
    graph: &FileGraph,
) -> HashMap<Url, Vec<TextEdit>> {
    let mut edits: Vec<TextEdit> = Vec::new();
    let mut map: HashMap<Url, Vec<TextEdit>> = HashMap::new();

    edits.push(TextEdit::new(
        symbol.get_definition_range(),
        new_name.clone(),
    ));
    map.insert(curr, edits);
    for usage in symbol.get_usages() {
        info!("Usage: {:?}", usage);
        let url = graph
            .find_url_from_node_index(usage.file_id.unwrap())
            .unwrap();
        if let Some(existing_vector) = map.get_mut(&url) {
            existing_vector.push(TextEdit::new(usage.range, new_name.clone()));
        } else {
            let new_vector = vec![TextEdit::new(usage.range, new_name.clone())];
            map.insert(url, new_vector);
        }
    }

    map
}
