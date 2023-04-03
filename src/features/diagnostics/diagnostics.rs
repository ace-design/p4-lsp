use crate::File;
use tower_lsp::lsp_types::Diagnostic;

use super::external::P4Test;
use super::internal::Parse;

pub trait DiagnosticProvider {
    fn get_diagnostics(file: &File) -> Vec<Diagnostic>;
}

pub fn get_quick_diagnostics(file: &File) -> Vec<Diagnostic> {
    let diagnotics = Parse::get_diagnostics(file);

    diagnotics
}

pub fn get_full_diagnostics(file: &File) -> Vec<Diagnostic> {
    let mut diagnotics = P4Test::get_diagnostics(file);

    diagnotics.append(&mut Parse::get_diagnostics(file));

    diagnotics
}
