use crate::analysis::Analysis;
use crate::file::File;
use std::io::Write;
use std::process::Stdio;

use std::path::PathBuf;
use std::process::Command;

use tower_lsp::lsp_types::Diagnostic;

pub struct P4Test {}

impl Analysis for P4Test {
    fn get_diagnostics(file: &File) -> Vec<Diagnostic> {
        let output = get_p4test_output(&file.content);

        if let Some(output) = output {
            debug!("{}", output);
            parse_output(output)
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

    if !result.status.success() {
        Some(String::from_utf8(result.stderr).unwrap())
    } else {
        None
    }
}

fn parse_output(output: String) -> Vec<Diagnostic> {
    todo!()
}
