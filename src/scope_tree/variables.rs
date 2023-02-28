use tree_sitter::Range;

#[derive(Debug, Clone)]
pub struct NamedDataItem {
    name: String,
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

    pub fn add(&mut self, mut variables: NamedDataItems) {
        self.variables.append(&mut variables.variables);
        self.constants.append(&mut variables.constants);
    }

    pub fn add_constant(&mut self, name: String, usages: Vec<Range>) {
        self.constants.push(NamedDataItem { name, usages });
    }

    pub fn add_variable(&mut self, name: String, usages: Vec<Range>) {
        self.variables.push(NamedDataItem { name, usages });
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
