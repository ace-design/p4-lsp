use extism_pdk::*;
use lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use regex::Regex;
use serde::{Deserialize, Serialize};

extern "C" {
    fn host_command(ptr: i64) -> i64;
}

#[derive(Serialize, Deserialize)]
struct CommandOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct TestOutput {
    stdout: String,
    stderr: String,
}

#[plugin_fn]
pub fn diagnostic(file_path: String) -> FnResult<Json<Vec<Diagnostic>>> {
    let command = format!("p4test {}", file_path);
    let memory = Memory::from_bytes(&command);
    let offset = unsafe { host_command(memory.offset as i64) };

    let out: CommandOutput = serde_json::from_str(get_string(offset as u64)?.as_str()).unwrap();

    let diags =
        parse_output(String::from_bytes(out.stderr).unwrap(), file_path).unwrap_or_default();
    Ok(Json(diags))
}

fn parse_output(message: String, file_path: String) -> Option<Vec<Diagnostic>> {
    // Parse and remove line number
    let line_nb_re = Regex::new(format!(r"{}\((\d+)\):?", file_path).as_str()).unwrap();
    let captures = line_nb_re.captures(&message)?;
    let line_nb = captures.get(1)?.as_str().parse::<u32>().ok()? - 1;
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

    Some(vec![Diagnostic::new(
        diag_range,
        Some(severity),
        Some(lsp_types::NumberOrString::String(kind.to_string())),
        Some("p4test".to_string()),
        diag_msg.trim().to_string(),
        None,
        None,
    )])
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

    Ok(memory.to_string()?)
}
