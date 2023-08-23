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

#[derive(Serialize,Deserialize,Clone)]
pub struct CustomResult{
    message:String,
    data:String
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
                plugin.arguments.push(Argument { key:key.clone(), value: url.to_file_path().unwrap().into_os_string().into_string().unwrap() })
            }
           
        }
        
        let json_str = to_string(&plugins).unwrap();
        self.plugins = plugins;
        info!("Deserialized Personn: {json_str}");
    }

    pub fn run_plugins(&mut self,file:Url,state:OnState) ->Option<(String,String)>{
        let mut message = String::from("");
        let mut data = String::from("");
         for plugin in self.plugins.clone().iter_mut(){
            let key = String::from("file");
            plugin.arguments.push(Argument { key:key.clone(), value: file.to_file_path().unwrap().into_os_string().into_string().unwrap() });
            if plugin.on.contains(&state) {
                let json_str = self.execute(plugin.clone()).unwrap();
                let results: CustomResult = from_str(json_str.as_str()).unwrap();
                if(!results.message.is_empty()){
                    message = results.message;
                }
                if(!results.data.is_empty()){
                    data = results.data;
                }
            }
        }
        Some((message,data))
    }

    pub fn execute(&mut self,plugin:Plugin) -> Option<String>{
        info!("Execute");


            // Replace "your_program" with the actual binary you want to execute
            let mut child = Command::new(plugin.path.clone())
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn().unwrap();
            

            // Write data to the child process's stdin
            if let Some(mut stdin) = child.stdin.take() {
                let arguments = plugin.arguments;
                let json_str = to_string(&arguments).unwrap();
                info!("a-{}",json_str);
                stdin.write_all(json_str.as_bytes()).unwrap();
                info!("b");
            }
        
            // Wait for the child process to finish and capture its stdout
            let result = match child.wait_with_output() {
                Ok(output) => {
                    String::from_utf8(output.stderr).unwrap()
                }
                Err(e) => {
                    e.to_string()
                }
            };
            return Some(result);
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
