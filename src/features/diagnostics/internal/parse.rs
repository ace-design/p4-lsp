use std::sync::{Arc, Mutex};

use super::super::DiagnosticProvider;

use crate::metadata::{AstQuery, NodeKind, SymbolTableQuery, VisitNode, Visitable};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

pub struct Parse {}

impl DiagnosticProvider for Parse {
    fn get_diagnostics(
        ast_query: &Arc<Mutex<impl AstQuery>>,
        _symbol_table_query: &Arc<Mutex<impl SymbolTableQuery>>,
    ) -> Vec<Diagnostic> {
        let ast_query = ast_query.lock().unwrap();
        let root = ast_query.visit_root();
        let mut errors: Vec<VisitNode> = vec![];
        for node in root.get_descendants() {
            if let NodeKind::Error = node.get().kind {
                errors.push(node.clone())
            };
        }

        errors
            .into_iter()
            .map(|node| {
                Diagnostic::new(
                    node.get().range,
                    Some(DiagnosticSeverity::ERROR),
                    Some(tower_lsp::lsp_types::NumberOrString::String(
                        "parsing".to_string(),
                    )),
                    Some("AST".to_string()),
                    "Parsing error.".to_string(),
                    None,
                    None,
                )
            })
            .collect()
    }
}
