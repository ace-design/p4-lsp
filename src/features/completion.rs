use crate::file::File;
use crate::metadata::{Symbols, Field};
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind, Position};

pub struct CompletionBuilder {
    items: Vec<CompletionItem>,
}

impl CompletionBuilder {
    pub fn new() -> CompletionBuilder {
        CompletionBuilder { items: vec![] }
    }

    pub fn add(
        mut self,
        new_items: &[String],
        completion_type: CompletionItemKind,
    ) -> CompletionBuilder {
        self.items.append(
            &mut new_items
                .iter()
                .map(|var| CompletionItem {
                    label: var.to_string(),
                    kind: Some(completion_type),
                    ..Default::default()
                })
                .collect(),
        );

        self
    }

    pub fn build(self) -> Vec<CompletionItem> {
        self.items
    }
}

pub fn get_list(position: Position, file: &File, trigger_character: Option<String>) -> Option<Vec<CompletionItem>> {
    fn default(position: Position, file: &File) -> Option<Vec<CompletionItem>> {
        let symbols = file.get_symbols_at_pos(position);

        Some(
            CompletionBuilder::new()
                .add(
                    &symbols
                        .types
                        .iter()
                        .map(|s| s.get_name())
                        .collect::<Vec<_>>(),
                    CompletionItemKind::TYPE_PARAMETER,
                )
                .add(
                    &symbols
                        .constants
                        .iter()
                        .map(|s| s.get_name())
                        .collect::<Vec<_>>(),
                    CompletionItemKind::CONSTANT,
                )
                .add(
                    &symbols
                        .variables
                        .iter()
                        .map(|s| s.get_name())
                        .collect::<Vec<_>>(),
                    CompletionItemKind::VARIABLE,
                )
                .add(
                    &symbols
                        .functions
                        .iter()
                        .map(|s| s.get_name())
                        .collect::<Vec<_>>(),
                    CompletionItemKind::FUNCTION,
                )
                .build(),
        )
    }

    match file.get_name_field(position){
        Some(fields) => {
            return Some(
                CompletionBuilder::new()
                    .add(&fields
                    .iter()
                    .map(|s| s.get_name())
                    .collect::<Vec<_>>()
                    ,CompletionItemKind::FIELD,
                ).build(),
            )
        }
        None => {
            return default(position, file);
        }
    }
}

