use crate::{
    language_def::{self, LanguageDefinition},
    lsp_mappings::HighlightType,
    metadata::{AstQuery, SymbolTableQuery, Visitable},
    utils,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use tower_lsp::lsp_types::{SemanticToken, SemanticTokens, SemanticTokensResult};
use tree_sitter::Node;

pub struct ColorData {
    line: u32,
    start: u32,
    length: u32,
    node_type: u32,
}

pub fn get_tokens(
    ast_query: &Arc<Mutex<impl AstQuery>>,
    st_query: &Arc<Mutex<impl SymbolTableQuery>>,
    ts_tree: &tree_sitter::Tree,
    source_code: &str,
) -> SemanticTokensResult {
    //Getting ast data
    let mut array = get_keyword_color_data(&ts_tree.root_node(), source_code);
    array.append(&mut get_symbols_color_data(st_query));
    array.append(&mut get_ast_color_data(ast_query));
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
            for item in &mut temp_array {
                if item.delta_line > max_val {
                    max_val = item.delta_line;
                }
                item.delta_line = 0;
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

fn get_semantic_token_map() -> HashMap<String, usize> {
    let mut semantic_token_types_map = HashMap::new();

    for (i, token_type) in language_def::LanguageDefinition::get_semantic_token_types()
        .iter()
        .enumerate()
    {
        semantic_token_types_map.insert(token_type.as_str().to_string(), i);
    }

    semantic_token_types_map
}

pub fn get_keyword_color_data(root_node: &tree_sitter::Node, source_code: &str) -> Vec<ColorData> {
    let keywords = crate::language_def::LanguageDefinition::get_keywords();

    let mut cursor = root_node.walk();
    let mut to_visit = root_node.children(&mut cursor).collect::<Vec<Node>>();

    let mut color_data = vec![];

    while let Some(node) = to_visit.pop() {
        if !node.is_named() && keywords.contains(&utils::get_node_text(&node, source_code)) {
            color_data.push(ColorData {
                length: (node.range().end_byte - node.range().start_byte) as u32,
                start: node.range().start_point.column as u32,
                line: node.range().start_point.row as u32,
                node_type: 0,
            });
        } else {
            to_visit.append(&mut node.children(&mut cursor).collect::<Vec<Node>>());
        }
    }

    color_data
}

pub fn get_symbols_color_data(st_query: &Arc<Mutex<impl SymbolTableQuery>>) -> Vec<ColorData> {
    let semantic_token_types_map = get_semantic_token_map();

    let symbols = st_query.lock().unwrap().get_all_symbols();

    let mut color_data = vec![];
    for symbol in symbols {
        let highlight_type = get_symbol_highlight_type(symbol.get_kind());
        let node_type = *semantic_token_types_map
            .get(highlight_type.get().as_str())
            .unwrap() as u32;

        let def_range = symbol.get_definition_range();
        color_data.push(ColorData {
            line: def_range.start.line,
            start: def_range.start.character,
            length: def_range.end.character - def_range.start.character,
            node_type,
        });

        for range in symbol.get_usages() {
            color_data.push(ColorData {
                line: range.start.line,
                start: range.start.character,
                length: range.end.character - range.start.character,
                node_type,
            });
        }
    }

    color_data
}

fn get_symbol_highlight_type(symbol_kind: String) -> HighlightType {
    LanguageDefinition::get()
        .symbol_types
        .iter()
        .find(|symbol_type| symbol_type.name == symbol_kind)
        .unwrap()
        .highlight_type
        .clone()
}

pub fn get_ast_color_data(ast_query: &Arc<Mutex<impl AstQuery>>) -> Vec<ColorData> {
    let semantic_token_types_map = get_semantic_token_map();

    let ast_query = ast_query.lock().unwrap();

    let mut color_data = vec![];
    for visit_node in ast_query.visit_root().get_descendants() {
        let node = visit_node.get();

        if let Some(semantic_token_type) = &node.semantic_token_type {
            color_data.push(ColorData {
                line: node.range.start.line,
                start: node.range.start.character,
                length: node.range.end.character - node.range.start.character,
                node_type: *semantic_token_types_map
                    .get(semantic_token_type.get().as_str())
                    .unwrap() as u32,
            });
        }
    }

    color_data
}
