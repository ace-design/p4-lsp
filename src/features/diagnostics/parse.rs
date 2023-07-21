use std::sync::{Arc, Mutex};

use crate::metadata::{AstQuery, NodeKind, SymbolTableQuery, VisitNode, Visitable};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

use super::provider::DiagnosticProvider;

pub struct Parse {}

impl DiagnosticProvider for Parse {
    fn get_diagnostics(
        ast_query: &Arc<Mutex<impl AstQuery>>,
        _symbol_table_query: &Arc<Mutex<impl SymbolTableQuery>>,
    ) -> Vec<Diagnostic> {
        let ast_query = ast_query.lock().unwrap();
        let root = ast_query.visit_root();
        let mut errors: Vec<(VisitNode, Option<String>)> = vec![];
        for node in root.get_descendants() {
            if let NodeKind::Error(msg) = &node.get().kind {
                errors.push((node, msg.clone()))
            };
        }

        errors
            .into_iter()
            .map(|(node, msg)| {
                Diagnostic::new(
                    node.get().range,
                    Some(DiagnosticSeverity::ERROR),
                    Some(tower_lsp::lsp_types::NumberOrString::String(
                        "parsing".to_string(),
                    )),
                    Some("AST".to_string()),
                    if let Some(msg) = msg {
                        format!("Syntax error: {}", msg)
                    } else {
                        "Syntax error.".to_string()
                    },
                    None,
                    None,
                )
            })
            .collect()
    }
}
