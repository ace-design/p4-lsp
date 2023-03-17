use serde_json::Value;
use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Settings {
    p4test_path: Option<PathBuf>,
}

impl Settings {
    pub fn parse(value: Value) -> Settings {
        if let Value::Object(map) = value {
            let p4test_path: Option<PathBuf> =
                if let Some(Value::String(path_str)) = map.get("p4test_path") {
                    Some(PathBuf::from(path_str))
                } else {
                    None
                };

            Settings { p4test_path }
        } else {
            Settings {
                ..Default::default()
            }
        }
    }
}
