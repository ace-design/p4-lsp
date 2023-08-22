use tower_lsp::lsp_types::*;
use tower_lsp::lsp_types::Diagnostic;
use serde::{Deserialize,Serialize};
use serde_json::{from_str,to_string};
use std::io::{self, Write};
use std::process::{Command, Stdio};
pub struct PluginManager {
    plugins: Vec<Plugin>,
}

#[derive(Serialize,Deserialize,PartialEq,Clone)] 
pub enum OnState {
    Save,
    Open,
    Change,
}

#[derive(Serialize,Deserialize,Clone)]
pub struct Plugin{
    name:String,
    path:String,
    on:Vec<OnState>,
    arguments:Vec<Argument>,
    state:bool,
}
#[derive(Serialize,Deserialize,Clone)]
pub struct Argument{
    key:String,
    value:String
}

impl PluginManager {
    pub fn new() -> PluginManager {
        PluginManager {
            plugins: Vec::new(),
        }
    }

    pub fn load_plugins(&mut self,uri:Option<Url> ,json_str:&str){
        let mut plugins: Vec<Plugin> = from_str(json_str).unwrap();
        if let Some(url) = uri{
            let key = String::from("workspace");
            for plugin in plugins.iter_mut(){
                plugin.arguments.push(Argument { key:key.clone(), value: url.to_string() })
            }
           
        }
        
        let json_str = to_string(&plugins).unwrap();
        self.plugins = plugins;
        info!("Deserialized Personn: {json_str}");
    }

    pub fn run_plugins(&mut self,file:Url,state:OnState){
         for plugin in self.plugins.clone().iter_mut(){
            let key = String::from("file");
            plugin.arguments.push(Argument { key:key.clone(), value: file.to_string() });
            if plugin.on.contains(&state) {
                self.execute(plugin.clone());
            }
        }
    }

    pub fn execute(&mut self,plugin:Plugin){
        info!("Excute");
        let mut cmd = Command::new(plugin.path.clone());
        let json_str = to_string(&plugin).unwrap();
    
        cmd.stdin(Stdio::piped());
        info!("1");
        let mut child = cmd.spawn().expect("Failed to start the process");

        if let Some(mut stdin) = child.stdin.take() {
            let data = json_str.as_str();
 info!("2");
            if stdin.write_all(data.as_bytes()).is_ok() {
                drop(stdin);
 info!("3");
                let status = child.wait().expect("Failed to wait for the process");
 info!("4");
                if status.success() {
                    info!("Process exited successfully");
                } else {
                    info!("Process exited with an error");
                }
            } else {
                info!("Failed to write data to stdin");
            }
        } else {
            info!("Failed to access stdin of the child process");
        }
    }

    pub fn run_diagnostic(&mut self, file_path: String) -> Vec<Diagnostic> {
        let mut diags = vec![];
        /*for plugin in &mut self.plugins {
            if plugin.has_function("diagnostic") {
                let result = plugin.call("diagnostic", file_path.clone());
                if let Ok(output) = result {
                    let out_str = String::from_utf8(output.to_vec()).expect("Invalid string");

                    info!("Plugin called: {}", out_str);
                    let mut deserialized = serde_json::from_str(&out_str).unwrap();
                    diags.append(&mut deserialized);
                }
            }
        }*/

        diags
    }
}
