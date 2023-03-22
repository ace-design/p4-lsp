use crate::analysis::Analysis;
use crate::file::File;

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

pub struct Parse {}

impl Analysis for Parse {
    fn get_diagnostics(file: &File) -> Vec<Diagnostic> {
        let error_nodes = file.ast.as_ref().unwrap().get_error_nodes();

        error_nodes
            .into_iter()
            .map(|node| {
                let err_msg = node.get_error_msg().unwrap_or("Parsing error.".into());
                Diagnostic::new(
                    node.range,
                    Some(DiagnosticSeverity::ERROR),
                    Some(tower_lsp::lsp_types::NumberOrString::String(
                        "parsing".to_string(),
                    )),
                    Some("AST".to_string()),
                    err_msg.trim().to_string(),
                    None,
                    None,
                )
            })
            .collect()
    }
}
