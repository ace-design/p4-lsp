use crate::metadata::{LinkObj, Usage};
use petgraph::dot::{Config, Dot};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use tower_lsp::lsp_types::{TextDocumentContentChangeEvent, Url};

use crate::file::File;

#[derive(Debug)]
pub enum Location {
    Local,
    External,
}

#[derive(Debug)]
pub struct Node {
    pub file_name: String,
    pub file_location: Location,
    pub file: File,
}

impl Node {
    fn new(file_name: String, file_location: Location, file: File) -> Self {
        Node {
            file_name: file_name.to_string(),
            file_location: file_location,
            file: file,
        }
    }
}

pub struct FileGraph {
    graph: DiGraph<Node, ()>,
    nodes: Vec<NodeIndex>,
}

impl FileGraph {
    pub fn new() -> Self {
        FileGraph {
            graph: DiGraph::new(),
            nodes: Vec::new(),
        }
    }
}

impl FileGraph {
    pub fn add_node(
        &mut self,
        file_name: String,
        file_location: Location,
        file: File,
    ) -> NodeIndex {
        let node = Node::new(file_name, file_location, file);
        let node_index = self.graph.add_node(node);
        self.nodes.push(node_index);
        node_index
    }
    pub fn find_node_with_url(&self, target_url: &str) -> Option<NodeIndex> {
        for node_index in self.graph.node_indices() {
            if let Some(node) = self.graph.node_weight(node_index) {
                if node.file_name == target_url {
                    return Some(node_index);
                }
            }
        }

        None
    }

    pub fn get_next_node_index(&self) -> NodeIndex {
        // Find the highest index node in the graph
        let max_index = self.graph.node_indices().max();

        // Generate the next available index
        match max_index {
            Some(max) => NodeIndex::new(max.index() + 1),
            None => NodeIndex::new(0), // If the graph is empty
        }
    }

    pub fn add_edge(&mut self, source_file_id: &NodeIndex, target_file_id: &NodeIndex) {
        if (self.nodes.contains(source_file_id) && self.nodes.contains(target_file_id)) {
            self.graph
                .add_edge(source_file_id.clone(), target_file_id.clone(), ());
        }
    }

    pub fn get_node(&self, node_index: NodeIndex) -> Option<&Node> {
        self.graph.node_weight(node_index).map(|n| n.clone())
    }

    pub fn get_mut_node(&mut self, node_index: NodeIndex) -> Option<&mut Node> {
        self.graph.node_weight_mut(node_index)
    }

    pub fn display_graph(&self) {
        println!(
            "{:?}",
            Dot::with_config(&self.graph, &[Config::EdgeNoLabel])
        );
    }

    pub fn get_all_undefined(&self, file_name: &str) -> Vec<Usage> {
        let mut undefined: Vec<Usage> = Vec::new();

        let nodes = self.nodes.iter().filter_map(|&node_index| {
            self.graph.node_weight(node_index).map(|n| {
                if n.file.check_if_import_exist(file_name) {
                    let mut node_undefined = n.file.get_undefined();
                    for node in node_undefined.iter_mut() {
                        node.file_id = Some(node_index);
                    }

                    undefined.extend(node_undefined);
                }
            })
        });

        undefined
    }

    pub fn update_file(
        &mut self,
        node_index: NodeIndex,
        parser: &mut tree_sitter::Parser,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) {
        let node = self.get_mut_node(node_index).unwrap();
        node.file.update(changes, parser, node_index);
    }

    pub fn update_nodes_symbols(&mut self) {
        //Either pass list or call fucntion
        let mut links: Vec<LinkObj> = Vec::new();

        for node_index in &self.nodes {
            
            let node = self.graph.node_weight(*node_index).unwrap();
            let undefined = self.get_all_undefined(node.file_name.as_str());
            
            
            let mut_node = self.graph.node_weight_mut(*node_index).unwrap();
            links.append(
                &mut mut_node
                    .file
                    .update_symbole_table(undefined)
                    .clone(),
            )
        }



        for link in links {
            let node =  self.graph.node_weight_mut(link.file_id).unwrap();
            node.file.update_nodes(link);
        }
    }
}
