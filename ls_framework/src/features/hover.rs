use std::sync::{Arc, Mutex};

use tower_lsp::lsp_types::{HoverContents, MarkedString, Position};

use crate::metadata::{AstQuery, SymbolTableQuery, Visitable};

use crate::file_graph::FileGraph;
use petgraph::graph::NodeIndex;
pub fn get_hover_info(
    ast_query: &Arc<Mutex<impl AstQuery>>,
    symbol_table_query: &Arc<Mutex<impl SymbolTableQuery>>,
    graph: &FileGraph,
    position: Position,

) -> Option<HoverContents> {
    let ast_query = ast_query.lock().unwrap();
    let root_visit = ast_query.visit_root();
    let node = root_visit.get_node_at_position(position)?;

    let n = node.get().linked_symbol.clone()?;
    
    info!("Symbol:1 ");
    let y = graph.get_node(n.file_id).unwrap();
    
    info!("Symbol:2");
    let st_query = y.file.symbol_table_manager.lock().unwrap();

    let symbol = st_query.get_symbol(n)?;
    info!("Symbol: {}",symbol);
    let mut type_name = String::from("");
    if let Some(symbol_type) = symbol.get_type_symbol(){
        let type_symbol = st_query.get_symbol(symbol_type).unwrap();
        
        info!("Symbofffl type: {}",type_symbol);
        type_name=type_symbol.get_name();
        info!("Symbol type: {}",type_symbol);
    }
    
    info!("Symbol name type: {}",type_name);
    Some(HoverContents::Scalar(MarkedString::String(format!(
        "{}: {}",
        symbol.get_name(),
        type_name
    ))))
}
