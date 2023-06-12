use super::host_functions::FUNCTIONS;
use extism::{Context, Plugin};
use std::fs;

struct Manager<'a> {
    context: Context,
    plugins: Vec<Plugin<'a>>,
}

impl<'a> Manager<'a> {
    pub fn new() -> Manager<'a> {
        Manager {
            context: Context::new(),
            plugins: Vec::new(),
        }
    }

    pub fn load_plugins(&'a mut self) {
        self.plugins = Vec::new();

        let paths = fs::read_dir(".").unwrap();

        for path in paths {
            let file_content = fs::read(path.unwrap().path()).unwrap();
            let functions = (*FUNCTIONS).clone();

            let plugin = Plugin::new(&self.context, file_content, functions, true).unwrap();

            self.plugins.push(plugin);
        }
    }
}
