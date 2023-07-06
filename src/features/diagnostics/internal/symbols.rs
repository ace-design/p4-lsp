use super::super::DiagnosticProvider;
use crate::file::File;

use tower_lsp::lsp_types::Diagnostic;

pub struct Symbols {}

impl DiagnosticProvider for Symbols {
    fn get_diagnostics(_file: &File) -> Vec<Diagnostic> {
        vec![]
    }
}
