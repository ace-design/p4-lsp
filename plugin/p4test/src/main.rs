use extism_pdk::*;
use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::io::*;

extern "C" {
    fn host_command(ptr: i64) -> i64;
}

#[derive(Serialize, Deserialize)]
struct CommandOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

pub fn diagnostic(file_path: String) -> String {
    let command = format!("p4test {}", file_path);
    let memory = Memory::from_bytes(&command);
    let offset = unsafe { host_command(memory.offset as i64) };

    let out: CommandOutput =
        serde_json::from_str(get_string(offset as u64).unwrap().as_str()).unwrap();

    serde_json::to_string(&parse_output(
        String::from_bytes(out.stderr).unwrap(),
        file_path,
    ))
    .unwrap()
}

fn parse_output(message: String, file_path: String) -> Vec<Diagnostic> {
    // Parse and remove line number
    let line_nb_re = Regex::new(format!(r"{}\((\d+)\):?", file_path).as_str()).unwrap();
    let captures = line_nb_re.captures(&message).unwrap();
    let line_nb = captures
        .get(1)
        .unwrap()
        .as_str()
        .parse::<u32>()
        .ok()
        .unwrap()
        - 1;
    let current_msg = line_nb_re.replace(&message, "");

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

    // Make and return diagnostic
    let lines: Vec<&str> = current_msg.trim().lines().collect();

    let diag_msg = lines[0].replace("error:", "").replace("warning:", "");
    let diag_range = get_range(line_nb, lines[2]);

    vec![Diagnostic::new(
        diag_range,
        Some(severity),
        Some(lsp_types::NumberOrString::String(kind.to_string())),
        Some("p4test".to_string()),
        diag_msg.trim().to_string(),
        None,
        None,
    )]
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

fn get_string(offset: u64) -> FnResult<String> {
    let length = unsafe { extism_pdk::bindings::extism_length(offset) };
    let memory = Memory {
        offset,
        length,
        free: false,
    };

    Ok(memory.to_string().unwrap())
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
        println!("{{\"output\":\"diagnostic\", \"data\":\"{}\"}}", json);
    } else {
        let json = serde_json::to_string(&Notification {
            message: "p4test testing : fail.\\nYou didn't give me all the arguments that I need."
                .to_string(),
            data: "".to_string(),
        })
        .unwrap();
        println!("{{\"output\":\"notification\", \"data\":\"{}\"}}", json);
    }
}
