use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io::*;
use std::process::{Command, Stdio};

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
        (vec![], vec![])
    }
}

pub fn diagnostic(file_path: String) -> String {
    let (stdout, stderr) = get_command_output("p4test", "", vec![file_path.clone()]);

    let mut t: Vec<Diagnostic> = Vec::new();
    if !stderr.is_empty() {
        t = parse_output(String::from_utf8(stderr).unwrap(), file_path)
    }
    serde_json::to_string(&t).unwrap()
}

fn parse_output(message: String, file_path: String) -> Vec<Diagnostic> {
    // Parse and remove line number
    // Make and return diagnostic
    let mut vec_diagnostic: Vec<Diagnostic> = vec![];
    let lines: Vec<&str> = message.trim().lines().collect();
    for index in (0..(lines.len())).step_by(3) {
        let line = lines.get(index).unwrap();
        let line_nb_re = Regex::new(format!(r"{}\((\d+)\):?", file_path.clone()).as_str()).unwrap();
        let captures = line_nb_re.captures(&line).unwrap();
        let line_nb = captures
            .get(1)
            .unwrap()
            .as_str()
            .parse::<u32>()
            .ok()
            .unwrap()
            - 1;
        let current_msg = line_nb_re.replace(&line, "");

        let kind_re = Regex::new(r"\[--W(.*)=(.*)\]").unwrap();
        let captures = kind_re.captures(&current_msg);

        // Parse and remove severity and kind
        let (severity, kind) = if let Some(captures) = captures {
            let severity_capture = captures.get(1);
            let severity = if let Some(cap) = severity_capture {
                match cap.as_str() {
                    "error" => DiagnosticSeverity::ERROR,
                    "warn" => DiagnosticSeverity::WARNING,
                    _ => DiagnosticSeverity::ERROR,
                }
            } else {
                DiagnosticSeverity::ERROR
            };

            let kind_cap = captures.get(2);
            let kind = if let Some(cap) = kind_cap {
                cap.as_str()
            } else {
                ""
            };

            (severity, kind)
        } else {
            (DiagnosticSeverity::ERROR, "")
        };
        let current_msg = kind_re.replace(&current_msg, "");

        let diag_msg = current_msg.replace("error:", "").replace("warning:", "");
        let diag_range = get_range(line_nb, lines.get(index + 2).unwrap());
        vec_diagnostic.push(Diagnostic::new(
            diag_range,
            Some(severity),
            Some(lsp_types::NumberOrString::String(kind.to_string())),
            Some("p4test".to_string()),
            diag_msg.trim().to_string(),
            None,
            None,
        ));
    }

    vec_diagnostic
}

fn get_range(line_nb: u32, arrows: &str) -> Range {
    let mut start: u32 = 0;

    for char in arrows.chars() {
        if char == ' ' {
            start += 1;
        } else {
            break;
        }
    }

    Range::new(
        Position::new(line_nb, start),
        Position::new(line_nb, arrows.len() as u32),
    )
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
    let mut input = String::new();
    stdin().read_line(&mut input).expect("Failed to read line");
    let object: Vec<Argument> = serde_json::from_str(&input).unwrap();

    let mut p4: String = "".to_string();

    for arg in object {
        if arg.key == "file" {
            p4 = arg.value;
        }
    }

    if p4 != "" {
        let json = diagnostic(p4);
        println!("{{\"output_type\":\"Diagnostic\", \"data\":\"{}\"}}", json);
    } else {
        let json = serde_json::to_string(&Notification {
            message: "p4test testing : fail.\\nYou didn't give me all the arguments that I need."
                .to_string(),
            data: "".to_string(),
        })
        .unwrap();
        println!(
            "{{\"output_type\":\"Notification\", \"data\":\"{}\"}}",
            json
        );
    }
}
