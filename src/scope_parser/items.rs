use tower_lsp::lsp_types::Position;
use tree_sitter::Range;

#[derive(Debug, Clone)]
pub struct NamedItem {
    name: String,
    name_def: Range,
    usages: Vec<Range>,
}

#[derive(Debug, Clone)]
pub struct NamedItemList {
    items: Vec<NamedItem>,
}

impl NamedItemList {
    pub fn new() -> NamedItemList {
        NamedItemList { items: vec![] }
    }

    pub fn add(&mut self, name: String, name_def: Range) {
        self.items.push(NamedItem {
            name,
            name_def,
            usages: vec![],
        });
    }

    pub fn add_list(&mut self, mut list: NamedItemList) {
        self.items.append(&mut list.items);
    }

    pub fn get_names(&self, position: Position) -> Vec<String> {
        self.items
            .iter()
            .filter(|item| (position.line as usize) > item.name_def.end_point.row)
            .map(|item| item.name.clone())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct NamedItems {
    pub variables: NamedItemList,
    pub constants: NamedItemList,
    pub functions: NamedItemList,
}

impl NamedItems {
    pub fn new() -> NamedItems {
        NamedItems {
            variables: NamedItemList::new(),
            constants: NamedItemList::new(),
            functions: NamedItemList::new(),
        }
    }

    pub fn add_subscope(&mut self, new_items: NamedItems) {
        self.variables.add_list(new_items.variables);
        self.constants.add_list(new_items.constants);
        self.functions.add_list(new_items.functions);
    }
}
