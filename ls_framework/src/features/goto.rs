use std::sync::{Arc, Mutex};

use crate::metadata::{AstQuery, SymbolTableQuery, Visitable};
use tower_lsp::lsp_types::{Position, Range};

use crate::file_graph::FileGraph;
use petgraph::graph::NodeIndex;
pub fn get_definition_range(
    ast_query: &Arc<Mutex<impl AstQuery>>,
    symbol_table_query: &Arc<Mutex<impl SymbolTableQuery>>,
    graph: &FileGraph,
    position: Position,
) -> Option<(Range,NodeIndex)> {
    let ast_query = ast_query.lock().unwrap();
    let root_visit = ast_query.visit_root();
    let node = root_visit.get_node_at_position(position)?;
    
    let n = node.get().linked_symbol.clone()?;
    let file_id = n.file_id.clone();
    let y = graph.get_node(n.file_id).unwrap().clone();
    let x = y.file.symbol_table_manager.lock().unwrap();

    let symbol = x.get_symbol(n)?;
    Some((symbol.get_definition_range(),file_id))
}
