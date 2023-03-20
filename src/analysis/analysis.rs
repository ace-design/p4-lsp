use crate::File;
use tower_lsp::lsp_types::Diagnostic;

use crate::analysis::internal::Parse;

pub trait Analysis {
    fn get_diagnostics(file: &File) -> Vec<Diagnostic>;
}

pub fn get_ordered_diagnostics(file: &File) -> Vec<Diagnostic> {
    let diagnotics = Parse::get_diagnostics(file);

    diagnotics
}
