#![allow(unused)]

use tower_lsp::lsp_types::{HoverContents, MarkedString};

pub struct HoverContentBuilder {
    items: Vec<MarkedString>,
}

impl HoverContentBuilder {
    pub fn new() -> HoverContentBuilder {
        HoverContentBuilder { items: vec![] }
    }

    pub fn add_text(mut self, text: &str) -> HoverContentBuilder {
        self.items.push(MarkedString::String(format!("{text}\n")));

        self
    }

    pub fn build(self) -> HoverContents {
        HoverContents::Array(self.items)
    }
}
