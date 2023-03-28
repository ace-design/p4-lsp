use crate::analysis::Analysis;
use crate::file::File;

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity};

pub struct Parse {}

impl Analysis for Parse {
    fn get_diagnostics(file: &File) -> Vec<Diagnostic> {
        if let Some(metadata) = &file.metadata {
            let error_nodes = metadata.ast.get_error_nodes();

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
        } else {
            vec![]
        }
    }
}
