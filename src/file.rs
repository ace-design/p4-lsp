use std::sync::{Arc, Mutex};

use tower_lsp::lsp_types::{
    CompletionItem, Diagnostic, HoverContents, Location, Position, SemanticTokensResult,
    TextDocumentContentChangeEvent, Url, WorkspaceEdit,
};
use tree_sitter::{InputEdit, Parser, Tree};

use crate::file_tree::Node;
use crate::features::{completion, diagnostics, goto, hover, rename, semantic_tokens};
use crate::metadata::{AstEditor, AstManager, SymbolTableEditor, SymbolTableManager};
use crate::utils;
use std::fmt;
use std::collections::HashMap;

use indextree::{Arena, NodeId};
use crate::metadata::SymbolTable;

#[derive(Debug)]
pub struct File {
    pub uri: Url,
    pub source_code: String,
    pub tree: Option<Tree>,
    pub symbol_table_manager: Arc<Mutex<SymbolTableManager>>,
    pub ast_manager: Arc<Mutex<AstManager>>,
}

impl fmt::Display for File {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(&self.uri.to_string())
    }
}

impl File {
    pub fn new(uri: Url, source_code: &str, tree: &Option<Tree>,map:Option<HashMap<Url,NodeId>>,arena: &mut Arena<Node>) -> File {
        info!("Data {:?}",map);
        let t = AstManager::new(uri.clone(),
            source_code,
            tree.to_owned().unwrap(),
        );
        let tt = Mutex::new(t);
        

        let ast_manager = Arc::new(tt);
 

        let symbol_table_manager = {
           
            let ast_manager = ast_manager.lock().unwrap();
 
            let t = SymbolTableManager::new(ast_manager.get_ast(),map,uri.clone(),arena);
           
            let tt = Mutex::new(t);
      
            Arc::new(tt)
        };
      

        File {
            uri,
            source_code: source_code.to_string(),
            tree: tree.clone(),
            symbol_table_manager,
            ast_manager,
        }
    }

    pub fn update(&mut self, changes: Vec<TextDocumentContentChangeEvent>, parser: &mut Parser,map:Option<HashMap<Url,NodeId>>,arena: &mut Arena<Node>) {
        for change in changes {
            let mut old_tree: Option<&Tree> = None;
            let text: String;

            if let Some(range) = change.range {
                let start_byte = utils::pos_to_byte(range.start, &self.source_code);
                let old_end_byte = utils::pos_to_byte(range.end, &self.source_code);

                let start_position = utils::pos_to_point(range.start);

                let edit = InputEdit {
                    start_byte,
                    old_end_byte: utils::pos_to_byte(range.end, &self.source_code),
                    new_end_byte: start_byte + change.text.len(),
                    start_position,
                    old_end_position: utils::pos_to_point(range.end),
                    new_end_position: utils::calculate_end_point(start_position, &change.text),
                };

                self.source_code
                    .replace_range(start_byte..old_end_byte, &change.text);

                text = self.source_code.clone();
                let tree = self.tree.as_mut().unwrap();
                tree.edit(&edit);
                old_tree = Some(tree);
            } else {
                // If change.range is None, change.text represents the whole file
                text = change.text.clone();
            }

            self.tree = parser.parse(text, old_tree);
        }

        let mut ast_manager = self.ast_manager.lock().unwrap();
        let mut st_manager = self.symbol_table_manager.lock().unwrap();

        ast_manager.update(&self.source_code, self.tree.to_owned().unwrap());
        st_manager.update(ast_manager.get_ast(),map,self.uri.clone(),arena);
    }

    pub fn get_quick_diagnostics(&self) -> Vec<Diagnostic> {
        diagnostics::get_quick_diagnostics(&self.ast_manager, &self.symbol_table_manager)
    }

    pub fn get_full_diagnostics(&self) -> Vec<Diagnostic> {
        diagnostics::get_full_diagnostics(&self.ast_manager, &self.symbol_table_manager)
    }

    pub fn get_completion_list(&self, position: Position) -> Option<Vec<CompletionItem>> {
        completion::get_list(position, &self.source_code, &self.symbol_table_manager)
    }

    pub fn get_hover_info(&self, position: Position) -> Option<HoverContents> {
        let tree: &Tree = self.tree.as_ref()?;

        let point = utils::pos_to_point(position);

        let mut node: tree_sitter::Node = tree
            .root_node()
            .named_descendant_for_point_range(point, point)?;

        let mut node_hierarchy = node.kind().to_string();
        while node.kind() != "source_file" {
            node = node.parent()?;
            node_hierarchy = [node.kind().into(), node_hierarchy].join(" > ");
        }

        let hover_content = hover::HoverContentBuilder::new()
            .add_text(&node_hierarchy)
            .build();

        Some(hover_content)
    }

    pub fn get_semantic_tokens(&self) -> Option<SemanticTokensResult> {
        Some(semantic_tokens::get_tokens(&self.ast_manager))
    }

    pub fn get_definition_location(&self, position: Position) -> Option<Location> {
        let range =
            goto::get_definition_range(&self.ast_manager, &self.symbol_table_manager, position)?;
        Some(Location::new(self.uri.clone(), range))
    }

    pub fn rename_symbol(&self, position: Position, new_name: String) -> Option<WorkspaceEdit> {
        rename::rename(
            &self.ast_manager,
            &self.symbol_table_manager,
            &self.symbol_table_manager,
            self.uri.clone(),
            new_name,
            position,
        )
    }
}
