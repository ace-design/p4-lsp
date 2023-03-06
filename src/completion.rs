use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

pub struct Completion {
    pub items: Vec<CompletionItem>,
}

pub struct CompletionBuilder {
    items: Vec<CompletionItem>,
}

impl CompletionBuilder {
    pub fn new() -> CompletionBuilder {
        CompletionBuilder { items: vec![] }
    }

    pub fn add(
        mut self,
        new_items: Vec<String>,
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

    pub fn build(self) -> Completion {
        Completion { items: self.items }
    }
}
