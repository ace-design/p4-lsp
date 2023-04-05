use tower_lsp::lsp_types::{
    SemanticToken, SemanticTokenType, SemanticTokens, SemanticTokensLegend, SemanticTokensResult,
};

use crate::metadata::Ast;

pub fn get_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![SemanticTokenType::VARIABLE],
        token_modifiers: vec![],
    }
}

pub fn get_tokens(ast: Ast) -> SemanticTokensResult {
    let tokens: Vec<SemanticToken> = vec![];

    SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data: tokens,
    })
}
