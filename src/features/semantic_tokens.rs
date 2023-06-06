use tower_lsp::lsp_types::{
    SemanticToken, SemanticTokenType, SemanticTokens, SemanticTokensLegend, SemanticTokensResult,
};

pub fn get_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![SemanticTokenType::VARIABLE, SemanticTokenType::FUNCTION],
        token_modifiers: vec![],
    }
}

pub fn get_tokens() -> SemanticTokensResult {
    let tokens: Vec<SemanticToken> = vec![
        SemanticToken {
            delta_line: 0,
            delta_start: 0,
            length: 5,
            token_type: 0,
            token_modifiers_bitset: 0,
        },
        SemanticToken {
            delta_line: 0,
            delta_start: 5,
            length: 5,
            token_type: 1,
            token_modifiers_bitset: 0,
        },
    ];

    SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data: tokens,
    })
}

// fn new_token(node_type: u32, node: Node) -> SemanticToken {
//     SemanticToken {
//         delta_line: 0,
//         delta_start: 0,
//         length: 0,
//         token_type: 0,
//         token_modifiers_bitset: 0,
//     }
// }
