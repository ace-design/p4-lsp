use super::super::DiagnosticProvider;
use crate::file::File;
use regex::Regex;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

pub struct P4Test {}

impl DiagnosticProvider for P4Test {
    fn get_diagnostics(file: &File) -> Vec<Diagnostic> {
        let output = get_p4test_output(&file.source_code);

        if let Some(output) = output {
            parse_output(output).unwrap_or_default()
        } else {
            vec![]
        }
    }
}

fn get_p4test_output(content: &str) -> Option<String> {
    let include_path = PathBuf::from("/home/alex/Documents/University/Master/p4c/p4include");
    let p4test_path = PathBuf::from("/home/alex/.local/bin/p4c_backend_p4test");

    let mut command = Command::new(p4test_path)
        .arg("/dev/stdin")
        .arg("-I")
        .arg(include_path.as_os_str())
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();

    let stdin = command.stdin.as_mut().unwrap();
    stdin.write_all(content.as_bytes()).unwrap();

    let result = command.wait_with_output().unwrap();

    let output = String::from_utf8(result.stderr).unwrap();
    if !output.is_empty() {
        Some(output)
    } else {
        None
    }
}

fn parse_output(message: String) -> Option<Vec<Diagnostic>> {
    // Parse and remove line number
    let line_nb_re = Regex::new(r"/dev/stdin\((\d+)\):?").unwrap();
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
        Some(tower_lsp::lsp_types::NumberOrString::String(
            kind.to_string(),
        )),
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

#[cfg(test)]
mod tests {
    use super::parse_output;
    use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};

    #[test]
    fn parse_output_warn() {
        let output = r#"
/dev/stdin(94): [--Wwarn=mismatch] warning: 8w512: value does not fit in 8 bits
z = 1 << 9;
    ^^^^^^
        "#
        .to_string();

        let diags = parse_output(output).unwrap();

        let expected_range = Range::new(Position::new(93, 4), Position::new(93, 10));
        assert_eq!(
            vec![Diagnostic::new(
                expected_range,
                Some(DiagnosticSeverity::WARNING),
                Some(tower_lsp::lsp_types::NumberOrString::String(
                    "mismatch".to_string(),
                )),
                Some("p4test".to_string()),
                "8w512: value does not fit in 8 bits".to_string(),
                None,
                None,
            )],
            diags
        )
    }
}
