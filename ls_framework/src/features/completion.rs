use std::sync::{Arc, Mutex};

use crate::{
    language_def::LanguageDefinition,
    metadata::{SymbolTableQuery, Symbols},
};
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, Position};

fn default_list(
    position: Position,
    query: &Arc<Mutex<impl SymbolTableQuery>>,
) -> Option<Vec<CompletionItem>> {
    let symbols: Symbols = query.lock().unwrap().get_symbols_at_pos(position);

    let mut items: Vec<CompletionItem> = Vec::new();

    for symbol_def in &LanguageDefinition::get().symbol_types {
        if let Some(list) = symbols.get_type(symbol_def.name.clone()) {
            items.append(
                &mut list
                    .iter()
                    .map(|item| CompletionItem {
                        label: item.get_name(),
                        kind: Some(symbol_def.completion_type.get()),
                        ..Default::default()
                    })
                    .collect(),
            );
        }
    }

    Some(items)
}

pub fn get_list(
    position: Position,
    source_code: &str,
    query: &Arc<Mutex<impl SymbolTableQuery>>,
) -> Option<Vec<CompletionItem>> {
    let name_field = query.lock().unwrap().get_name_field(position, source_code);

    match name_field {
        Some(fields) => {
            return Some(
                fields
                    .iter()
                    .map(|item| CompletionItem {
                        label: item.get_name(),
                        kind: Some(CompletionItemKind::FIELD),
                        ..Default::default()
                    })
                    .collect(),
            )
        }
        None => default_list(position, query),
    }
}
