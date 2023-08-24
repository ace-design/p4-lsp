use chrono::Utc;
use regex::Regex;
use serde::*;
use std::env;
use std::fs;
use std::io::*;
use std::path::Path;
use std::process::{Command, Stdio};
use std::str;

fn get_command_output(command: &str, current_dir: &str, args: Vec<String>) -> (Vec<u8>, Vec<u8>) {
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

pub fn write_error(workspace: String, file: String, pert4: String, explanation: String) -> String {
    return format!(
        "<!DOCTYPE html>
<html lang='en'>
<head>
  <meta http-equiv='content-type' content='text/html; charset=utf-8'>
  <title>PETR4 STF TESTING FAIL</title>
  <style type='text/css'>
    html * {{ margin:0; }}
    body * {{ padding:10px 20px; }}
    body * * {{ padding:0; }}
    body {{ font:small sans-serif; background:#ddd; color:#000; }}
    h1 {{ font-weight:normal; margin-bottom:.4em; }}
    h1 span {{ font-size:60%; color:#eee; }}
    table {{ border-collapse: collapse; }}
    td, th {{ padding:3px 4px; }}
    th {{ width:12em; text-align:right; color:#eee; }}
    #title {{ background: #F4364C; }}
  </style>
</head>
<body>
  <div id='title'>
    <h1>PETR4 STF TESTING FAIL <span>{date}</span></h1>
    <table>
      <tr>
        <th>WORKSPACE :</th>
        <td>{workspace}</td>
      </tr>
      <tr>
        <th>FILE :</th>
        <td>{file}</td>
      </tr>
      <tr>
        <th>PETR4 PATH :</th>
        <td>{pert4}</td>
      </tr>
    </table>
  </div>
  <div>
    <p>
        {explanation}
    </p>
  </div>
</body>
</html>",
        date = Utc::now(),
        workspace = workspace,
        file = file,
        pert4 = pert4,
        explanation = string_to_html(explanation)
    );
}

pub fn fail(
    workspace: String,
    p4: String,
    pert4: String,
    message: &str,
    number_command: i32,
    type_command: &str,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
) -> String {
    let content = format!("Petr4 plugin fail :\n{}\n\nMore Information :\nNumber commande : {}\nThe type of commande : {}\nstdout : {}\n stderr : {}", message, number_command, type_command, String::from_utf8(stdout).unwrap(), String::from_utf8(stderr).unwrap());
    return write_error(workspace, p4, pert4, content);
}

pub fn testing(petr4: String, p4: String, workspace: String) -> (bool, String, String) {
    let path_petr4 = Path::new(&petr4);
    let path_p4 = Path::new(&p4);
    let mut number_commande = 0;

    // verify the stf file exists
    let path_stf: std::path::PathBuf = path_p4.clone().with_extension("stf");
    if !path_stf.exists() {
        return (true, "".to_string(), "".to_string());
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
            vec![
                "-t".to_string(),
                path_p4.as_os_str().to_str().unwrap().to_string(),
            ],
        );
        number_commande += 1;
        if !stderr.is_empty() {
            return (
                false,
                "petr4 testing : fail".to_string(),
                fail(
                    workspace,
                    p4,
                    petr4,
                    "The execution of the binary of Petr4 fail.",
                    number_commande,
                    "./bin/test.exe",
                    stdout,
                    stderr,
                ),
            );
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

                    let t = Regex::new(r"\x1b\[[0-9;]*[mK]").unwrap();

                    return (
                        false,
                        "petr4 testing : fail".to_string(),
                        write_error(
                            workspace,
                            p4,
                            petr4,
                            t.replace_all(
                                &fs::read_to_string(p4_testing.clone()).expect(&format!(
                                    "petr4 fail, but can't read the file of the output '{}'",
                                    p4_testing.as_os_str().to_str().unwrap()
                                )),
                                "",
                            )
                            .to_string(),
                        ),
                    );
                }
                break;
            }
        }
    }
    return (false, "petr4 testing : success".to_string(), "".to_string());
}

fn prepare_string(content: String) -> String {
    let data = content
        .replace("\\", "\\\\")
        .replace("\n", "\\n")
        .replace("\r", "\\r")
        .replace("\t", "\\t");
    return data.replace("\"", "\\\"");
}
fn string_to_html(content: String) -> String {
    return content.replace("\n", "<br>");
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Argument {
    key: String,
    value: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Notification {
    message: String,
    data: String,
}

pub fn main() {
    let os: &str = env::consts::OS;
    if os == "windows" {
        let json = serde_json::to_string(&Notification {
            message: "petr4 testing : fail.\nThe plugin don't work on windows.".to_string(),
            data: "".to_string(),
        })
        .unwrap();
        println!(
            "{{\"output_type\":\"Notification\", \"data\":\"{}\"}}",
            json //prepare_string(json)
        );
    } else {
        //example of input tat I will receive :
        // [{"key":"petr4","value":"/home/t/petr4/"},{"key":"workspace","value":"/home/t/p4-lsp"},{"key":"file","value":"/home/t/p4-lsp/examples/casts.p4"}]

        let mut input = String::new();
        stdin().read_line(&mut input).expect("Failed to read line");
        let object: Vec<Argument> = serde_json::from_str(&input).unwrap();

        let mut petr4: String = "".to_string(); //object.get("petr4").unwrap();
        let mut p4: String = "".to_string(); //object.get("file").unwrap();
        let mut workspace: String = "".to_string(); //object.get("workspace").unwrap();

        for arg in object {
            if arg.key == "petr4" {
                petr4 = arg.value;
            } else if arg.key == "file" {
                p4 = arg.value;
            } else if arg.key == "workspace" {
                workspace = arg.value;
            }
        }

        //println!("you entered : {} - {}", petr4, p4);
        if petr4 != "" && p4 != "" && workspace != "" {
            let (nothing, message, mut data) = testing(petr4, p4, workspace);

            if nothing {
                println!("{{\"output_type\":\"Nothing\", \"data\":\"\"}}");
            } else {
                let json = serde_json::to_string(&Notification { message, data }).unwrap();
                println!(
                    "{{\"output_type\":\"Notification\", \"data\":\"{}\"}}",
                    json //prepare_string(json)
                );
            }
        } else {
            let json = serde_json::to_string(&Notification {
                message:
                    "petr4 testing : fail.\\nYou didn't give me all the arguments that I need."
                        .to_string(),
                data: "".to_string(),
            })
            .unwrap();
            println!(
                "{{\"output_type\":\"Notification\", \"data\":\"{}\"}}",
                json //prepare_string(json)
            );
        }
    }
}
