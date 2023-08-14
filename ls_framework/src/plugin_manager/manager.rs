use crate::language_def::LanguageDefinition;

use super::host_functions::FUNCTIONS;
use extism::Plugin;
use std::{env, fs};
use tower_lsp::lsp_types::Diagnostic;

pub struct PluginManager {
    plugins: Vec<Plugin<'static>>,
}

impl PluginManager {
    pub fn new() -> PluginManager {
        PluginManager {
            plugins: Vec::new(),
        }
    }

    pub fn load_plugins(&mut self) {
        info!("Loading plugins");
        self.plugins = Vec::new();

        if let Some(mut home_path) = env::var_os("HOME") {
            home_path.push(format!(
                "/.config/{}-lsf/plugins",
                LanguageDefinition::get().language.name
            ));

            let paths = match fs::read_dir(&home_path) {
                Ok(paths) => paths,
                Err(_) => {
                    error!("Couldn't read from plugins path ({:?}).", home_path);
                    return;
                }
            };

            for dir_entry in paths.flatten() {
                info!("Loading plugin: {}", dir_entry.path().display());
                let file_content = fs::read(dir_entry.path()).unwrap();
                let functions = (*FUNCTIONS).clone();

                match Plugin::create(file_content, functions, true) {
                    Ok(plugin) => {
                        self.plugins.push(plugin);
                    }
                    Err(err) => {
                        error!(
                            "Failed loading plugin: {} Error: {}",
                            dir_entry.path().display(),
                            err
                        );
                    }
                }
            }
        }

        info!("Loaded {} plugin(s)", self.plugins.len());
    }

    pub fn run_diagnostic(&mut self, file_path: String) -> Vec<Diagnostic> {
        let mut diags = vec![];
        for plugin in &mut self.plugins {
            if plugin.has_function("diagnostic") {
                let result = plugin.call("diagnostic", file_path.clone());
                if let Ok(output) = result {
                    let out_str = String::from_utf8(output.to_vec()).expect("Invalid string");

                    info!("Plugin called: {}", out_str);
                    let mut deserialized = serde_json::from_str(&out_str).unwrap();
                    diags.append(&mut deserialized);
                }
            }
        }

        diags
    }
}
