use crate::plugin_manager::notification::CustomParams;
use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};
use std::io::Write;
use std::process::{Command, Stdio};
use tower_lsp::lsp_types::Diagnostic;
use tower_lsp::lsp_types::*;

pub struct PluginManager {
    plugins: Vec<Plugin>,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum OnState {
    Save,
    Open,
    Change,
}

#[derive(Serialize, Deserialize, PartialEq, Clone)]
pub enum TypesNotification {
    Notification,
    Diagnostic,
    Nothing,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Plugin {
    name: String,
    path: String,
    on: Vec<OnState>,
    arguments: Vec<Argument>,
    state: bool,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct Argument {
    key: String,
    value: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct CustomResult {
    output_type: TypesNotification,
    data: String,
}

pub struct PluginsResult {
    pub diagnostic: Vec<Diagnostic>,
    pub notification: Vec<CustomParams>,
}
impl PluginsResult {
    pub fn new() -> PluginsResult {
        PluginsResult {
            diagnostic: Vec::new(),
            notification: Vec::new(),
        }
    }
}

impl PluginManager {
    pub fn new() -> PluginManager {
        PluginManager {
            plugins: Vec::new(),
        }
    }
    pub fn load_plugins(&mut self, uri: Option<Url>, json_str: &str) {
        if let Ok(mut plugins) = from_str::<Vec<Plugin>>(json_str) {
            if let Some(url) = uri {
                let key = String::from("workspace");
                for plugin in plugins.iter_mut() {
                    plugin.arguments.push(Argument {
                        key: key.clone(),
                        value: url
                            .to_file_path()
                            .unwrap()
                            .into_os_string()
                            .into_string()
                            .unwrap(),
                    })
                }
            }

            self.plugins = plugins;
        }
    }

    pub fn run_plugins(&mut self, file: Url, state: OnState) -> PluginsResult {
        let mut plugins_result: PluginsResult = PluginsResult::new();
        for plugin in self.plugins.clone().iter_mut() {
            let key = String::from("file");
            plugin.arguments.push(Argument {
                key: key.clone(),
                value: file
                    .to_file_path()
                    .unwrap()
                    .into_os_string()
                    .into_string()
                    .unwrap(),
            });
            if plugin.on.contains(&state) {
                let json_str = self.execute(plugin.clone()).unwrap();
                let results: CustomResult = from_str(json_str.as_str()).unwrap();

                match results.output_type {
                    TypesNotification::Diagnostic => {
                        let mut diag: Vec<Diagnostic> = from_str(results.data.as_str()).unwrap();
                        plugins_result.diagnostic.append(&mut diag);
                    }
                    TypesNotification::Notification => {
                        let notification: CustomParams = from_str(results.data.as_str()).unwrap();
                        plugins_result.notification.push(notification);
                    }
                    TypesNotification::Nothing => {}
                }
            }
        }
        plugins_result
    }

    fn execute(&mut self, plugin: Plugin) -> Option<String> {
        info!("Execute");

        // Replace "your_program" with the actual binary you want to execute
        let mut child = Command::new(plugin.path.clone())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();

        // Write data to the child process's stdin
        if let Some(mut stdin) = child.stdin.take() {
            let arguments = plugin.arguments;
            let json_str = to_string(&arguments).unwrap();
            info!("a-{}", json_str);
            stdin.write_all(json_str.as_bytes()).unwrap();
            info!("b");
        }

        // Wait for the child process to finish and capture its stdout
        let result = match child.wait_with_output() {
            Ok(output) => String::from_utf8(output.stdout).unwrap(),
            Err(e) => e.to_string(),
        };
        Some(result)
    }
}
