use serde_json::Value;

use std::collections::HashMap;
use std::fs;
use std::process::Command;
use tower_lsp::lsp_types::{
    CompletionItem, Diagnostic, HoverContents, Location, Position, SemanticTokensResult,
    TextDocumentContentChangeEvent, Url, WorkspaceEdit,
};
use tree_sitter::Parser;
use tree_sitter_p4::language;
use walkdir::WalkDir;

use std::path::Path;

use crate::cache::Cache;
use crate::file_tree::{self as tree, Information};
use crate::{file::File, settings::Settings};
use indextree::{Arena, NodeId};
pub struct Workspace {
    settings: Settings,
    pub cache: Option<Cache>,
    parser: Parser,
}

impl Workspace {
    pub fn new() -> Workspace {
        let mut parser = Parser::new();
        parser.set_language(language()).unwrap();
        let cache = Some(Cache::new());
        let mut workspace = Workspace {
            settings: Settings::default(),
            cache: cache,
            parser,
        };
        //info!("Created Workspace");
        workspace
    }
    pub fn add_file(&mut self, url: Url, content: &str) {
        
        //info!("Adding Local File");
        /*let cachee: &mut Cache = self.cache.as_mut().unwrap();
        let (root,arena) = cachee.file_tree.get_prop().unwrap();
        for i in root.descendants(arena){
            let node  =arena.get(i).unwrap().get();
            if node.file != None{
                let ast = node.clone().file.clone().unwrap().clone().ast_manager.clone().lock().unwrap().ast.clone().get_debug_tree();
                let url = node.clone().file_information.unwrap().get_url().to_file_path().unwrap().into_os_string().into_string().unwrap();
               info!("File  AST {url}");
                //let content = read_file_string(url.as_str()).unwrap();
                //debug!("\n FileContents: {:?}",content);
                debug!("\nAst: {:?}",ast);
            }
          
        }*/
        
        //info!("Befor {}",self.cache.as_mut().unwrap().files.len());
        self.cache
            .as_mut()
            .unwrap()
            .add_file(url.clone(), content, tree::ControlState::InControl);
        for (url,exp) in self.cache.as_mut().unwrap().files.clone(){
            info!("after File {:?}",url.as_str());
        }
      
        info!("Adding Local File1 {}",url.as_str());
       // self.cache.as_mut().unwrap().add_node(url);

        //info!("Adding Local File1 {:?}",self.cache.as_mut().unwrap().file_tree);
        info!("File Tree AST");
        info!("Ast: {}",self.cache.as_ref().unwrap().file_tree);
    }

    pub fn init_files(&mut self){
        self.cache.as_mut().unwrap().add_init();
    }

    pub fn update_file(&mut self, url: Url, changes: Vec<TextDocumentContentChangeEvent>) {
        //info!("Updating Local File");
        let (mut file, mut information) = self.cache.as_mut().unwrap().get(url).unwrap(); //self.files.get_mut(&url).unwrap();

        file.update(changes, &mut self.parser, None);
    }

    pub fn get_definition_location(
        &mut self,
        url: Url,
        symbol_position: Position,
    ) -> Option<Location> {
        //info!("settingseee");
        let (mut file, mut information) = self.cache.as_mut().unwrap().get(url).unwrap(); //self.files.get_mut(&url).unwrap();

        file.get_definition_location(symbol_position)
    }

    pub fn rename_symbol(
        &mut self,
        url: Url,
        symbol_position: Position,
        new_name: String,
    ) -> Option<WorkspaceEdit> {
        //info!("fffsettings");
        let (mut file, mut information) = self.cache.as_mut().unwrap().get(url).unwrap(); //self.files.get_mut(&url).unwrap();

        file.rename_symbol(symbol_position, new_name)
    }

    pub fn get_semantic_tokens(&mut self, url: Url) -> Option<SemanticTokensResult> {
        //info!("setttttttings");
        let result = self.cache.as_mut().unwrap().get(url); //self.files.get_mut(&url).unwrap();
        match result {
            Some((mut file, mut information)) => file.get_semantic_tokens(),
            None => None,
        }
    }

    pub fn get_completion(&mut self, url: Url, position: Position) -> Option<Vec<CompletionItem>> {
        //info!("settfdfdings");
        let (mut file, mut information) = self.cache.as_mut().unwrap().get(url).unwrap(); //self.files.get_mut(&url).unwrap();

        file.get_completion_list(position)
    }

    pub fn get_hover_info(&mut self, url: Url, position: Position) -> Option<HoverContents> {
        //info!("setdfdftings");
        let result = self.cache.as_mut().unwrap().get(url); //self.files.get_mut(&url).unwrap();
        match result {
            Some((mut file, mut information)) => file.get_hover_info(position),
            None => None,
        }
    }

    pub fn get_quick_diagnostics(&mut self, url: Url) -> Vec<Diagnostic> {
        //info!("settinfdgs");
        let maybe_file = self.cache.as_mut().unwrap().get(url);
        if let Some((file, information)) = maybe_file {
            file.get_quick_diagnostics()
        } else {
            vec![]
        }
    }

    pub fn get_full_diagnostics(&mut self, url: Url) -> Vec<Diagnostic> {
        //info!("settings");
        let t = self.cache.as_mut();
        //info!("s");
        let tt = t.unwrap();
        //info!("s1,{:?}", url);
        let maybe_file = tt.get(url);
        //info!("settings1");
        if let Some((file, information)) = maybe_file {
            //info!("settings2");
            file.get_full_diagnostics()
        } else {
            //info!("settings3");
            vec![]
        }
    }

    pub fn update_settings(&mut self, settings: Value) {
        //info!("settings");
        self.settings = Settings::parse(settings);
        //info!("Settings: {:?}", self.settings);
    }
}