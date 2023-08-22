use serde_json::Value;
use std::path::PathBuf;

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
}
