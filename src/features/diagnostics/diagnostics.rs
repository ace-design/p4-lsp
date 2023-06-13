use tower_lsp::lsp_types::Diagnostic;

use super::external::P4Test;
use super::internal::Parse;
use crate::file::File;
use crate::metadata::{AstQuery, SymbolTableQuery};
use crate::settings::Settings;

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
        ast_query: &impl AstQuery,
        symbol_table_query: &impl SymbolTableQuery,
    ) -> Vec<Diagnostic>;
}

pub fn get_quick_diagnostics(
    ast_query: &impl AstQuery,
    symbol_table_query: &impl SymbolTableQuery,
) -> Vec<Diagnostic> {
    diags![Parse::get_diagnostics(ast_query, symbol_table_query)]
}

pub fn get_full_diagnostics(
    file: &File,
    ast_query: &impl AstQuery,
    symbol_table_query: &impl SymbolTableQuery,
    settings: &Settings,
) -> Vec<Diagnostic> {
    diags![
        P4Test::get_diagnostics(file, settings),
        Parse::get_diagnostics(ast_query, symbol_table_query)
    ]
}
