use std::sync::{Arc, Mutex};

use crate::metadata::{AstQuery, NodeKind, SymbolTableQuery, VisitNode, Visitable};
use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

use super::provider::DiagnosticProvider;

pub struct ParseSymbol {}

impl DiagnosticProvider for ParseSymbol {
    fn get_diagnostics(
        _ast_query: &Arc<Mutex<impl AstQuery>>,
        symbol_table_query: &Arc<Mutex<impl SymbolTableQuery>>,
    ) -> Vec<Diagnostic> {
        let symbol_table_query = symbol_table_query.lock().unwrap();
        let mut vec_diagnostic: Vec<Diagnostic> = vec![];

        for i in symbol_table_query.get_error() {
            vec_diagnostic.push(Diagnostic::new(
                i,
                Some(DiagnosticSeverity::ERROR),
                Some(tower_lsp::lsp_types::NumberOrString::String(
                    "symbol".to_string(),
                )),
                Some("symbol table".to_string()),
                "Symbol error.".to_string(),
                None,
                None,
            ))
        }

        for i in symbol_table_query.get_undefined() {
            vec_diagnostic.push(Diagnostic::new(
                i,
                Some(DiagnosticSeverity::WARNING),
                Some(tower_lsp::lsp_types::NumberOrString::String(
                    "symbol".to_string(),
                )),
                Some("symbol table".to_string()),
                "Symbol undefined.".to_string(),
                None,
                None,
            ))
        }

        return vec_diagnostic;
    }
}
