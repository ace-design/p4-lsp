use serde_json::Value;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Default)]
pub struct Settings {
    pub include_path: Option<PathBuf>,
    pub p4test_path: Option<PathBuf>,
}

impl Settings {
    pub fn parse(value: Value) -> Settings {
        if let Value::Object(map) = value {
            let include_path: Option<PathBuf> =
                if let Some(Value::String(path_str)) = map.get("include_path") {
                    Some(PathBuf::from(path_str))
                } else {
                    None
                };
            let p4test_path: Option<PathBuf> =
                if let Some(Value::String(path_str)) = map.get("p4test_path") {
                    Some(PathBuf::from(path_str))
                } else {
                    None
                };

            Settings {
                include_path,
                p4test_path,
            }
        } else {
            Settings {
                ..Default::default()
            }
        }
    }
    pub fn get_includes_folder() -> Option<PathBuf> {
        let output = if cfg!(target_os = "windows") {
            Command::new("where")
                .arg("p4c")
                .output()
                .expect("failed to execute process")
        } else {
            Command::new("which")
                .arg("p4c")
                .output()
                .expect("failed to execute process")
        };

        let text = "/usr/local/share/p4c/p4include"; //str::from_utf8(&output.stderr).expect("failed to convert output to string");
        let file_path = Path::new(text);
        Some(file_path.to_path_buf())
    }
}
