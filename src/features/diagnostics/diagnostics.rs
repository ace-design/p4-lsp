use std::sync::{Arc, Mutex};

use tower_lsp::lsp_types::Diagnostic;

use super::parse::Parse;
use crate::metadata::{AstQuery, SymbolTableQuery};

macro_rules! diags {
    ($($diag:expr),*) => {
        {
            let mut diags = Vec::new();

            $(
                diags.append(&mut $diag);
            )*

            diags
        }
    };
}

pub trait DiagnosticProvider {
    fn get_diagnostics(
        ast_query: &Arc<Mutex<impl AstQuery>>,
        symbol_table_query: &Arc<Mutex<impl SymbolTableQuery>>,
    ) -> Vec<Diagnostic>;
}

pub fn get_quick_diagnostics(
    ast_query: &Arc<Mutex<impl AstQuery>>,
    symbol_table_query: &Arc<Mutex<impl SymbolTableQuery>>,
) -> Vec<Diagnostic> {
    diags![Parse::get_diagnostics(ast_query, symbol_table_query)]
}

pub fn get_full_diagnostics(
    ast_query: &Arc<Mutex<impl AstQuery>>,
    symbol_table_query: &Arc<Mutex<impl SymbolTableQuery>>,
) -> Vec<Diagnostic> {
    diags![Parse::get_diagnostics(ast_query, symbol_table_query)]
}
