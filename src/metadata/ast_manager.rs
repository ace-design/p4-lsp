use crate::metadata::ast::VisitNode;

use tower_lsp::lsp_types::{Position,Url};
use super::Ast;

pub trait AstEditor {
    fn update(&mut self, content: &str, syntax_tree: tree_sitter::Tree);
}

pub trait AstQuery {
    fn visit_root(&self) -> VisitNode;
}

#[derive(Debug, Clone)]
pub struct AstManager {
    pub(crate) ast: Ast,
}

impl AstManager {
    pub fn new(uri:Url,source_code: &str, tree: tree_sitter::Tree) -> AstManager {
        let ast = Ast::new(source_code, tree).unwrap();

        let url = uri.to_string();
        debug!("\nAST : Url{url}:\n ");
        AstManager { ast }
    }

    pub fn get_ast(&self) -> &Ast {
        &self.ast
    }
}

impl AstQuery for AstManager {
    fn visit_root(&self) -> VisitNode {
        self.ast.visit_root()
    }
}

impl AstEditor for AstManager {
    fn update(&mut self, content: &str, syntax_tree: tree_sitter::Tree) {
        *self = AstManager::new(Url::parse("https://example.net").unwrap(),content, syntax_tree);
    }
}
