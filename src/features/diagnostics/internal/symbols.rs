use super::super::DiagnosticProvider;
use crate::file::File;

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

pub struct Symbols {}

impl DiagnosticProvider for Symbols {
    fn get_diagnostics(file: &File) -> Vec<Diagnostic> {
        vec![]
    }
}
