use tree_sitter::Range;

#[derive(Debug, Clone)]
pub struct NamedDataItem {
    name: String,
    name_def: Range,
    usages: Vec<Range>,
}

#[derive(Debug, Clone)]
pub struct NamedDataItems {
    pub variables: Vec<NamedDataItem>,
    pub constants: Vec<NamedDataItem>,
}

impl NamedDataItems {
    pub fn new() -> NamedDataItems {
        NamedDataItems {
            variables: vec![],
            constants: vec![],
        }
    }

    pub fn add_subscope(&mut self, mut items: NamedDataItems) {
        for new_var in items.variables.iter().chain(&items.constants) {
            self.variables.retain(|var| var.name != new_var.name);
            self.constants.retain(|var| var.name != new_var.name);
        }

        self.variables.append(&mut items.variables);
        self.constants.append(&mut items.constants);
    }

    pub fn add_constant(&mut self, name: String, name_def: Range) {
        self.constants.push(NamedDataItem {
            name,
            name_def,
            usages: vec![],
        });
    }

    pub fn add_variable(&mut self, name: String, name_def: Range) {
        self.variables.push(NamedDataItem {
            name,
            name_def,
            usages: vec![],
        });
    }

    pub fn get_names(&self) -> (Vec<String>, Vec<String>) {
        let variable_names = self
            .variables
            .iter()
            .map(|item| item.name.clone())
            .collect();
        let constant_names = self
            .constants
            .iter()
            .map(|item| item.name.clone())
            .collect();

        (variable_names, constant_names)
    }
}
