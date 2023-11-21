use std::collections::HashMap;

use serde_json::Value;
use tower_lsp::lsp_types::{
    CompletionContext, CompletionItem, Diagnostic, HoverContents, Location, Position,
    SemanticTokensResult, TextDocumentContentChangeEvent, Url, WorkspaceEdit,
};
extern crate walkdir;

use crate::{file::File, settings::Settings};
use std::fs;
use walkdir::WalkDir;
use std::path::Path;
use crate::file_graph::FileGraph;

use crate::file_graph;

pub struct Workspace {
    settings: Settings,
    graph: FileGraph,
    parser: tree_sitter::Parser,
}

impl Workspace {
    pub fn new(ts_language: tree_sitter::Language) -> Workspace {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(ts_language).unwrap();

        Workspace {
            settings: Settings::default(),
            graph: FileGraph::new(),
            parser,
        }
    }

    pub fn read_local_files(&mut self, workspace_addr: &String) -> Result<(), std::io::Error> {
        info!("Reading local");
        let file_extension = "p4";
        let mut file_map: HashMap<Url, &str> = HashMap::new();
        info!("{}",workspace_addr.clone());
       // Parse the URL
        let url = Url::parse(&String::from(r"C:\Users\Noel LAPTOP\Downloads\tutorials-master\tutorials-master\exercises\basic")).expect("Failed to parse URL");

        // Extract the path part of the URL
        let url_path = url.path();

        // Convert the URL path to a Path object
        let path = Path::new(url_path);
        let walker = WalkDir::new(path).into_iter();
       
        info!("ddddd");
        for entry in walker {
            info!("Runnifddffdfdfdng");
            if let Ok(entry) = entry {
                info!("Running");
                let path = entry.path();
                
                let path_as_string = path.to_string_lossy().to_string();
                info!("{}",path_as_string);

                if path.extension().unwrap_or_default() == file_extension {
                    //info!("Stuff {:?}", path.canonicalize().unwrap().display());
                    let temp = Url::from_file_path(path.canonicalize().unwrap().as_path());
                    let file_contents = fs::read_to_string(entry.path())?;
                    info!("{}",temp.clone().unwrap().to_string());
                    self.add_file(temp.unwrap(), &file_contents.as_str())
                }
            } else {
                //info!("Error");
            }
        }
        Ok(())
    }

    pub fn read_external_files(&mut self) -> Result<(), std::io::Error> {
        info!("Reading external");
        let file_extension = "p4";
        let mut file_map: HashMap<Url, &str> = HashMap::new();
        let walker = WalkDir::new(String::from(r"C:\include")).into_iter();
        info!("walker");
        for entry in walker {
            info!("rnty");
            if let Ok(entry) = entry {
                info!("Running");
                let path = entry.path();

                if path.extension().unwrap_or_default() == file_extension {
                    info!("Stuff {:?}", path.canonicalize().unwrap().display());
                    let temp = Url::from_file_path(path.canonicalize().unwrap().as_path());
                    let file_contents = fs::read_to_string(entry.path())?;
                    info!("{}",temp.clone().unwrap().to_string());
                    self.add_file(temp.unwrap(), &file_contents.as_str())
                }
            } else {
                //info!("Error");
            }
        }
        Ok(())
    }
    pub fn add_file(&mut self, url: Url, content: &str) {
        info!("Added File1");
        let tree = self.parser.parse(content, None);
        let index = self.graph.get_next_node_index();
        info!("info :{:?}",index);
        info!("infto :{:?}",tree.clone().unwrap());
        
        self.graph.add_node(
            url.to_string().clone(),
            file_graph::Location::Local,
            File::new(url, content, &tree, index),
        );
        info!("NodeId {}", self.graph.nodes[0].index());
        self.graph.display_graph();
    }

    pub fn update_file(&mut self, url: Url, changes: Vec<TextDocumentContentChangeEvent>) {
        let file_id = self.graph.find_node_with_url(url.as_str());
        self.graph
            .update_file(file_id.unwrap(), &mut self.parser, changes)
    }

    pub fn get_definition_location(&self, url: Url, symbol_position: Position) -> Option<Location> {
        let file_id1 = self.graph.find_node_with_url(url.as_str());
        let file = &self.graph.get_node(file_id1.unwrap()).unwrap().file;
        info!("File Id {:?}",file_id1);

        let (range,file_id) = file.get_definition_location(symbol_position,&self.graph).unwrap();
        let uri = &self.graph.get_node(file_id).unwrap().file.uri;
        info!("uri:{:?}",uri);
        Some(Location::new(uri.clone(), range))
    }

    pub fn rename_symbol(
        &mut self,
        url: Url,
        symbol_position: Position,
        new_name: String,
    ) -> Option<WorkspaceEdit> {
        let file_id = self.graph.find_node_with_url(url.as_str()).unwrap();
        let file = &self.graph.get_node(file_id).unwrap().file;

        file.rename_symbol(symbol_position, new_name,&self.graph)
    }

    pub fn get_semantic_tokens(&self, url: Url) -> Option<SemanticTokensResult> {
        let file_id = self.graph.find_node_with_url(url.as_str()).unwrap();
        let file = &self.graph.get_node(file_id).unwrap().file;

        file.get_semantic_tokens()
    }

    pub fn get_completion(
        &self,
        url: Url,
        position: Position,
        context: Option<CompletionContext>,
    ) -> Option<Vec<CompletionItem>> {
        let file_id = self.graph.find_node_with_url(url.as_str()).unwrap();
        let file = &self.graph.get_node(file_id).unwrap().file;

        file.get_completion_list(position, context)
    }

    pub fn get_hover_info(&self, url: Url, position: Position) -> Option<HoverContents> {
        let file_id = self.graph.find_node_with_url(url.as_str()).unwrap();
        let file = &self.graph.get_node(file_id).unwrap().file;

        file.get_hover_info(position,&self.graph)
    }

    pub fn get_quick_diagnostics(&self, url: Url) -> Vec<Diagnostic> {
        /* let maybe_file = self.files.get(&url);

        if let Some(file) = maybe_file {
            file.get_quick_diagnostics()
        } else {
            vec![]
        }*/
        vec![]
    }

    pub fn get_full_diagnostics(&self, url: Url) -> Vec<Diagnostic> {
        /*let maybe_file = self.files.get(&url);

        if let Some(file) = maybe_file {
            file.get_full_diagnostics()
        } else {
            vec![]
        }*/
        vec![]
    }

    pub fn update_settings(&mut self, settings: Value) {
        self.settings = Settings::parse(settings);
        info!("Settings: {:?}", self.settings);
    }
}
