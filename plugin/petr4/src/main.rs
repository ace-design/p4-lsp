use lsp_types::*;
use serde::*;
use serde_json::*;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::*;
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;

fn get_command_output(
    command: &str,
    current_dir: &str,
    args: &mut Vec<String>,
) -> (Vec<u8>, Vec<u8>) {
    let mut partial_command = Command::new(command);

    if !args.is_empty() {
        partial_command.args(args);
    }

    if current_dir != "" {
        partial_command.current_dir(current_dir);
    }

    let command_result = partial_command
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn();

    if let Ok(command) = command_result {
        let result = command.wait_with_output().unwrap();

        (result.stdout, result.stderr)
    } else {
        (vec![], "error".as_bytes().to_vec())
    }
}

pub fn fail(number_command: i32, type_command: &str, stdout: Vec<u8>, stderr: Vec<u8>) -> String {
    let content = format!("Petr4 plugin fail.\nNumber commande : {}\nThe type of commande : {}\nstdout : {}\n stderr : {}", number_command, type_command, String::from_utf8(stdout).unwrap(), String::from_utf8(stderr).unwrap());
    return content;
}

pub fn pass() -> String {
    return "success".to_string();
}

pub fn windows() -> String {
    return "the plugin don't work for windows yet.".to_string();
}

pub fn testing(petr4: String, p4: String) -> String {
    let path_petr4 = Path::new(&petr4);
    let path_p4 = Path::new(&p4);
    let mut number_commande = 0;

    // verify the stf file exists
    let path_stf: std::path::PathBuf = path_p4.clone().with_extension("stf");
    if !path_stf.exists() {
        return "".to_string();
    }

    // verify the binary of the petr4 exists
    let path_test_binary = path_petr4.clone().join("_build/default/bin/test.exe");
    if path_test_binary.exists() {
        // execute the commande
        let (stdout, stderr) = get_command_output(
            path_petr4
                .clone()
                .clone()
                .join("_build/default/bin/test.exe")
                .as_os_str()
                .to_str()
                .unwrap(),
            "",
            &mut vec![
                "-t".to_string(),
                path_p4.as_os_str().to_str().unwrap().to_string(),
            ],
        );
        number_commande += 1;
        if !stderr.is_empty() {
            return fail(number_commande, "./bin/test.exe", stdout, stderr);
        }

        // get the output
        let parts = str::from_utf8(&stdout).unwrap().split("\n");
        for part in parts {
            if part.contains("file testing") && part.contains("p4_lsp stf tests") {
                if part.contains("[FAIL]") {
                    let mut index = "0";

                    let mut p4_testing_name = "p4_lsp stf tests.".to_string();
                    let ext = format!("{}.output", index.clone());
                    let mut t = format!("{}{}", p4_testing_name.clone(), ext.clone());
                    let mut p4_testing = path_petr4
                        .clone()
                        .join(format!("_build/default/_build/_tests/Stf-tests/{}", t));
                    while !p4_testing.exists() {
                        p4_testing_name = format!("{}0", p4_testing_name.clone());
                        t = format!("{}{}", p4_testing_name.clone(), ext.clone());
                        p4_testing = p4_testing.with_file_name(t);
                    }

                    return fs::read_to_string(p4_testing.clone()).expect(&format!(
                        "petr4 fail, but can't read the file of the output '{}'",
                        p4_testing.as_os_str().to_str().unwrap()
                    ));
                }
                break;
            }
        }
    }
    return pass();
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Argument {
    key: String,
    value: String,
}

pub fn main() {
    let os: &str = env::consts::OS;
    if os == "windows" {
        println!("{{\"result\":\"\"}}")
    } else {
        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read line");
        let object: Vec<Argument> = serde_json::from_str(&input).unwrap();

        let mut petr4: String = "".to_string(); //object.get("petr4").unwrap();
        let mut p4: String = "".to_string(); //object.get("file").unwrap();

        for arg in object {
            if arg.key == "petr4" {
                petr4 = arg.value;
            } else if arg.key == "file" {
                p4 = arg.value;
            }
        }

        println!("you entered : {} - {}", petr4, p4);
        if petr4 != "" && p4 != "" {
            println!("{{\"result\":\"{}\"}}", testing(petr4, p4));
        }
    }
}
