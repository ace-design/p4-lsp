use super::super::DiagnosticProvider;
use crate::file::File;

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

pub struct Parse {}

impl DiagnosticProvider for Parse {
    fn get_diagnostics(file: &File) -> Vec<Diagnostic> {
        if let Some(metadata) = &file.metadata {
            let error_nodes = metadata.ast.get_error_nodes();

            error_nodes
                .into_iter()
                .map(|node| {
                    Diagnostic::new(
                        node.range,
                        Some(DiagnosticSeverity::ERROR),
                        Some(tower_lsp::lsp_types::NumberOrString::String(
                            "parsing".to_string(),
                        )),
                        Some("AST".to_string()),
                        "Parsing error.".to_string(),
                        None,
                        None,
                    )
                })
                .collect()
        } else {
            vec![]
        }
    }
}
