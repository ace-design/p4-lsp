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

    pub fn add_list(
        mut self,
        list_items: Vec<String>,
        title: Option<String>,
    ) -> HoverContentBuilder {
        let mut text: String = if let Some(title) = title {
            format!("{title}:\n")
        } else {
            String::new()
        };

        for item in list_items {
            text.push_str("- ");
            text.push_str(item.as_str());
            text.push('\n');
        }
        text.push('\n');

        self.items.push(MarkedString::String(text));

        self
    }

    pub fn build(self) -> HoverContents {
        HoverContents::Array(self.items)
    }
}
