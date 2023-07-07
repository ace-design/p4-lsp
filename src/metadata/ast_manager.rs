use crate::metadata::ast::VisitNode;

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
    pub fn new(source_code: &str, tree: tree_sitter::Tree) -> AstManager {
        let ast = Ast::new(source_code, tree).unwrap();
        debug!("\nAST:\n{ast}");
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
        *self = AstManager::new(content, syntax_tree);
    }
}
