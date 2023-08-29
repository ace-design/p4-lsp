use std::sync::{Arc, Mutex};

use crate::{language_def::LanguageDefinition, metadata::SymbolTableQuery};
use tower_lsp::lsp_types::{
    CompletionContext, CompletionItem, CompletionItemKind, CompletionTriggerKind, Position,
};

fn default_list(
    position: Position,
    query: &Arc<Mutex<impl SymbolTableQuery>>,
) -> Option<Vec<CompletionItem>> {
    let mut items: Vec<CompletionItem> = Vec::new();

    for symbol in query.lock().unwrap().get_symbols_at_pos(position) {
        items.push(CompletionItem {
            label: symbol.get_name(),
            kind: get_symbol_completion_type(symbol.get_kind()),
            ..Default::default()
        })
    }

    Some(items)
}

fn get_symbol_completion_type(symbol_kind: String) -> Option<CompletionItemKind> {
    Some(
        LanguageDefinition::get()
            .symbol_types
            .iter()
            .find(|symbol_type| symbol_type.name == symbol_kind)?
            .completion_type
            .get(),
    )
}

pub fn get_list(
    position: Position,
    source_code: &str,
    query: &Arc<Mutex<impl SymbolTableQuery>>,
    context: Option<CompletionContext>,
) -> Option<Vec<CompletionItem>> {
    if let Some(context) = context {
        if context.trigger_kind == CompletionTriggerKind::TRIGGER_CHARACTER {
            // TODO
        }
    }

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
