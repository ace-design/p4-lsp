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
    let previous_name = &node.get().content;
    let n = node.get().linked_symbol.clone()?;

    let y = graph.get_node(n.file_id).unwrap().clone();
    let cur_url = y.file.uri.clone();
    let x = y.file.symbol_table_manager.lock().unwrap();
    
    let binding = x.get_all_symbols();
    binding.iter().for_each(|s| info!("T:{:?}",&s.get_name()));
    let symbol_exist = binding.iter().find(|s| &s.get_name() == previous_name);
    
    info!("Symbollldfddyy{:?} ",symbol_exist);


    let mut type_name = String::from("");
    

    let mut symbol_name = String::from("");
    if let Some(symbol) = symbol_exist{
        info!("Symbollldfddyydddd{:?} ",symbol);
        symbol_name = symbol.get_name();
        if let Some(symbol_type) = symbol.get_type_symbol(){
            let type_symbol = x.get_symbol(symbol_type).unwrap();
            
            info!("Symbofffl type: {}",type_symbol);
            type_name=type_symbol.get_name();
            info!("Symbol type: {}",type_symbol);
        }
    }
  
    
    info!("Symbol name type: {}",type_name);
    Some(HoverContents::Scalar(MarkedString::String(format!(
        "{}: {}",
        symbol_name,
        type_name
    ))))
}
