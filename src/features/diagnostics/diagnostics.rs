use crate::File;
use tower_lsp::lsp_types::Diagnostic;

use super::external::P4Test;
use super::internal::{Parse, Symbols};

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
    fn get_diagnostics(file: &File) -> Vec<Diagnostic>;
}

pub fn get_quick_diagnostics(file: &File) -> Vec<Diagnostic> {
    diags![Parse::get_diagnostics(file), Symbols::get_diagnostics(file)]
}

pub fn get_full_diagnostics(file: &File) -> Vec<Diagnostic> {
    diags![
        P4Test::get_diagnostics(file),
        Parse::get_diagnostics(file),
        Symbols::get_diagnostics(file)
    ]
}
