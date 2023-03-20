use crate::File;
use tower_lsp::lsp_types::Diagnostic;

use crate::analysis::internal::Parse;

pub trait Analysis {
    fn get_diagnostics(file: &File) -> Vec<Diagnostic>;
}

pub fn get_quick_diagnostics(file: &File) -> Vec<Diagnostic> {
    let diagnotics = Parse::get_diagnostics(file);

    diagnotics
}

pub fn get_full_diagnostics(file: &File) -> Vec<Diagnostic> {
    let diagnotics = Parse::get_diagnostics(file);

    diagnotics
}
