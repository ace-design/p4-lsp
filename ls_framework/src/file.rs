use crate::features::{completion, diagnostics, goto, hover, rename, semantic_tokens};
use crate::file_graph::FileGraph;
use crate::metadata::{
    AstEditor, AstManager, LinkObj, SymbolTableEditor, SymbolTableManager, Usage,
};
use crate::{language_def, utils};
use petgraph::graph::NodeIndex;
use std::sync::{Arc, Mutex};
use tower_lsp::lsp_types::Range;
use tower_lsp::lsp_types::{
    CompletionContext, CompletionItem, Diagnostic, HoverContents, Position, SemanticTokensResult,
    TextDocumentContentChangeEvent, Url, WorkspaceEdit,
};
use tree_sitter::{InputEdit, Parser, Tree};
#[derive(Debug, Clone)]
pub struct File {
    pub uri: Url,
    pub source_code: String,
    pub tree: Option<Tree>,
    pub symbol_table_manager: Arc<Mutex<SymbolTableManager>>,
    pub ast_manager: Arc<Mutex<AstManager>>,
}

impl File {
    pub fn new(uri: Url, source_code: &str, tree: &Option<Tree>, file_id: NodeIndex) -> File {
        let ast_manager = Arc::new(Mutex::new(AstManager::new(
            source_code,
            tree.to_owned().unwrap(),
        )));

        debug!("\nAST:\n{}", ast_manager.lock().unwrap());
        let symbol_table_manager = {
            let mut ast_manager = ast_manager.lock().unwrap();
            Arc::new(Mutex::new(SymbolTableManager::new(
                ast_manager.get_ast(),
                file_id,
            )))
        };

        debug!("\nAST:\n{}", ast_manager.lock().unwrap());
        debug!("\nSymbol Table:\n{}", symbol_table_manager.lock().unwrap());

        File {
            uri,
            source_code: source_code.to_string(),
            tree: tree.clone(),
            symbol_table_manager,
            ast_manager,
        }
    }

    pub fn update(
        &mut self,
        changes: Vec<TextDocumentContentChangeEvent>,
        parser: &mut Parser,
        node_index: NodeIndex,
    ) {
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

        let mut ast_manager: std::sync::MutexGuard<'_, AstManager> =
            self.ast_manager.lock().unwrap();
        let mut st_manager = self.symbol_table_manager.lock().unwrap();

        ast_manager.update(&self.source_code, self.tree.to_owned().unwrap());
        st_manager.update(ast_manager.get_ast(), node_index);

        debug!("\nAST:\n{}", ast_manager);
        debug!("\nSymbol Table:\n{}", st_manager);
    }

    pub fn get_quick_diagnostics(&self) -> Vec<Diagnostic> {
        diagnostics::get_quick_diagnostics(&self.ast_manager, &self.symbol_table_manager)
    }

    pub fn get_full_diagnostics(&self) -> Vec<Diagnostic> {
        diagnostics::get_full_diagnostics(&self.ast_manager, &self.symbol_table_manager)
    }

    pub fn get_completion_list(
        &self,
        position: Position,
        context: Option<CompletionContext>,
    ) -> Option<Vec<CompletionItem>> {
        completion::get_list(
            position,
            &self.source_code,
            &self.symbol_table_manager,
            context,
        )
    }

    pub fn get_hover_info(&self, position: Position, graph: &FileGraph) -> Option<HoverContents> {
        hover::get_hover_info(
            &self.ast_manager,
            &self.symbol_table_manager,
            graph,
            position,
        )
    }

    pub fn get_semantic_tokens(&self) -> Option<SemanticTokensResult> {
        self.tree.as_ref().map(|ts_tree| {
            semantic_tokens::get_tokens(
                &self.ast_manager,
                &self.symbol_table_manager,
                ts_tree,
                &self.source_code,
                &self.symbol_table_manager,
            )
        })
    }

    pub fn get_definition_location(
        &self,
        position: Position,
        graph: &FileGraph,
    ) -> Option<(Range, NodeIndex)> {
        goto::get_definition_range(
            &self.ast_manager,
            &self.symbol_table_manager,
            graph,
            position,
        )
    }

    pub fn rename_symbol(
        &self,
        position: Position,
        new_name: String,
        graph: &FileGraph,
    ) -> Option<WorkspaceEdit> {
        rename::rename(
            &self.ast_manager,
            &self.symbol_table_manager,
            &self.symbol_table_manager,
            self.uri.clone(),
            new_name,
            position,
            graph,
        )
    }

    pub fn get_undefined(&self) -> Vec<Usage> {
        return self
            .symbol_table_manager
            .lock()
            .unwrap()
            .symbol_table
            .undefined_list
            .clone();
    }

    pub fn check_if_import_exist(&self, file_name: String) -> bool {
        let imports = self.ast_manager.lock().unwrap().ast.get_imports();
        info!("imports {:?}", imports);
        info!("name {:?}", file_name);
        for import in imports.iter() {
            // info!("dimport {:?}",import);
            // info!("d{}",import.contains(&file_name));
            if file_name.contains(import) {
                info!("pass");
                return true;
            }
        }
        false
    }

    pub fn update_symbole_table(
        &mut self,
        undefined_list: Vec<Usage>,
        file_id_dest: NodeIndex,
    ) -> Vec<LinkObj> {
        let links: Vec<LinkObj> = self
            .symbol_table_manager
            .lock()
            .unwrap()
            .symbol_table
            .parse_undefined(undefined_list, file_id_dest);

        info!("Links {:?}", links);
        links
    }

    pub fn update_nodes(&mut self, link: LinkObj) {
        let mut binding = self.ast_manager.lock().unwrap();
        let arena = binding.ast.get_arena();
        for node in arena
            .iter_mut()
            .filter(|node| matches!(node.get().symbol, language_def::Symbol::Usage))
        {
            let node = node.get_mut();
            let symbol_name = &node.content;
            if symbol_name == &link.symbol {
                node.link(link.id, link.index, link.file_id_dest);
            }
        }
    }
}
