use crate::metadata::{AstQuery, NodeKind, VisitNode, Visitable};
use crate::metadata::{BaseType, Type};
use std::sync::{Arc, Mutex};
use tower_lsp::lsp_types::{
    SemanticToken, SemanticTokenType, SemanticTokens, SemanticTokensLegend, SemanticTokensResult,
};
lazy_static::lazy_static! {
    static ref TOKENS_TYPES: Vec<SemanticTokenType> = vec![
        SemanticTokenType::VARIABLE,
        SemanticTokenType::STRING,
        SemanticTokenType::KEYWORD,
        SemanticTokenType::TYPE,
        SemanticTokenType::NUMBER,
        SemanticTokenType::FUNCTION,
        SemanticTokenType::MODIFIER,
        SemanticTokenType::DECORATOR,
    ];
}
pub fn get_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: TOKENS_TYPES.clone(),
        token_modifiers: vec![],
    }
}
pub struct ColorData {
    line: u32,
    start: u32,
    length: u32,
    node_type: u32,
}

pub fn get_tokens(ast_query: &Arc<Mutex<impl AstQuery>>) -> SemanticTokensResult {
    //Getting ast data
    let ast_query = ast_query.lock().unwrap();
    let root_visit = ast_query.visit_root();
    let mut array = get_visit_nodes(root_visit);
    //sort line

    array.sort_by_key(|token| token.line);

    let mut prev = 0;
    let mut tokens: Vec<SemanticToken> = vec![];

    let mut temp_array: Vec<SemanticToken> = Vec::new();
    for item in array {
        //Calculating the line diff between current and previous line
        let line = ((item.line - prev) as i32).abs();

        prev = item.line;
        //If line is greater than 0 then we save the temp array of tokens
        if line > 0 && !temp_array.is_empty() {
            let mut max_val = 0;
            temp_array.sort_by_key(|&token| token.delta_start); //sorting the start pos
                                                                //setting the first deltaline to conatin the diff value (line) and setting all other to 0
            for i in 0..temp_array.len() {
                if temp_array[i].delta_line > max_val {
                    max_val = temp_array[i].delta_line;
                }
                temp_array[i].delta_line = 0;
            }
            temp_array[0].delta_line = max_val;

            //ReCalculating the delta start relative to the previous start pos
            let mut prev_start = 0;
            temp_array = temp_array
                .iter()
                .map(|token| {
                    let temp_token = SemanticToken {
                        delta_line: token.delta_line,
                        delta_start: token.delta_start - prev_start,
                        length: token.length,
                        token_type: token.token_type,
                        token_modifiers_bitset: 0,
                    };

                    prev_start = token.delta_start;
                    temp_token
                })
                .collect();

            tokens.extend(temp_array); //concating two vectors
            temp_array = Vec::new();
        }

        temp_array.push(SemanticToken {
            delta_line: line as u32,
            delta_start: item.start,
            length: item.length,
            token_type: item.node_type,
            token_modifiers_bitset: 0,
        });
    }

    SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data: tokens,
    })
}

pub fn get_visit_nodes(visit_node: VisitNode) -> Vec<ColorData> {
    //Going throuhg all childrens
    let childrens = visit_node.get_children();
    let mut array: Vec<ColorData> = Vec::new();
    for child in &childrens {
        array.append(&mut get_visit_nodes(*child));
    }

    //Setting the node kidn with associated index from token type vector
    let node_visit = visit_node.get();
    let kind = &node_visit.kind;
    let temp_cmp = (TOKENS_TYPES.len() + 1) as u32;
    let mut node_type: u32 = temp_cmp;
    match kind {
        NodeKind::Type(type_node_visit) => match type_node_visit {
            Type::Base(base_types) => {
                match base_types {
                    BaseType::String => {
                        node_type = 1;
                    }
                    _ => {
                        node_type = 4;
                    }
                };
            }
            Type::Name => {
                node_type = 3;
            }
            Type::Specialized => {
                node_type = 6;
            }
            _ => {
                node_type = 7;
            }
        },
        NodeKind::Name => {
            node_type = 0;
        }
        NodeKind::Direction(_dir_node) => {
            node_type = 1;
        }
        NodeKind::KeyWord => {
            node_type = 5;
        }
        NodeKind::Expression => {
            node_type = 7;
        }
        NodeKind::ValueSymbol => {
            node_type = 4;
        }
        _ => {
            debug!("Error in Tokens {:?}", kind);
        }
    };
    //only pushing node types that we support currently
    if node_type != temp_cmp {
        let temp_length = ((node_visit.range.end.character - node_visit.range.start.character)
            as i32)
            .unsigned_abs();
        array.push(ColorData {
            length: temp_length,
            start: node_visit.range.start.character,
            line: node_visit.range.start.line,
            node_type,
        });
    }

    array
}
