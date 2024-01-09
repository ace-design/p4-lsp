use std::sync::{Arc, Mutex};

use tower_lsp::lsp_types::{HoverContents, MarkedString, Position};

use crate::metadata::{AstQuery, SymbolTableQuery, Visitable};

use crate::file_graph::FileGraph;

pub fn get_hover_info(
    ast_query: &Arc<Mutex<impl AstQuery>>,
    symbol_table_query: &Arc<Mutex<impl SymbolTableQuery>>,
    graph: &FileGraph,
    position: Position,
) -> Option<HoverContents> {
    info!("Hover");
    let ast_query = ast_query.lock().unwrap();
    let root_visit = ast_query.visit_root();
    let node = root_visit.get_node_at_position(position)?;
    let temp = {
        let st_query = symbol_table_query.lock().unwrap();
        let symbol = st_query.get_symbol(node.get().linked_symbol.clone()?)?;
        symbol.get_type_symbol()
    };

    if let Some(temp1) = temp {
        let st_query = symbol_table_query.lock().unwrap();
        let symbol = st_query.get_symbol(node.get().linked_symbol.clone()?)?;
        info!(
            "Sylinkedmbol:{:?}",
            node.get().linked_symbol.clone().unwrap()
        );

        info!("Symbol:{}", symbol);

        info!("Temp Hover");
        let type_symbol = st_query.get_symbol(temp1)?;
        Some(HoverContents::Scalar(MarkedString::String(format!(
            "{}: {}",
            symbol.get_name(),
            type_symbol.get_name()
        ))))
    } else {
        info!("Added Hover");
        let previous_name = &node.get().content;
        let linked_symbol = node.get().linked_symbol.clone()?;

        info!("A");

        info!("prev:{:?}", previous_name);
        let linked_node = graph.get_node(linked_symbol.file_id).unwrap();
        info!("A1");

        info!("preve:{:?}", linked_node);
        let linked_manager = linked_node.file.symbol_table_manager.lock().unwrap();
        info!("A2");
        let binding = linked_manager.get_all_symbols();
        info!("A3");

        info!("list {:?}", binding);
        let symbol_exist = binding.iter().find(|s| &s.get_name() == previous_name);
        info!("A4");
        info!("{:?}", symbol_exist);
        let symbol = symbol_exist?;
        info!("A5");
        info!("{:?}", symbol);
        info!("{:?}", symbol.get_type_symbol().unwrap());
        let type_symbol = linked_manager.get_symbol(symbol.get_type_symbol()?)?;

        Some(HoverContents::Scalar(MarkedString::String(format!(
            "{}: {}",
            symbol.get_name(),
            type_symbol.get_name()
        ))))
    }
}
