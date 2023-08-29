use serde_json::Value;

#[derive(Debug, Default)]
pub struct Settings {}

impl Settings {
    pub fn parse(value: Value) -> Settings {
        if let Value::Object(_map) = value {
            Settings {}
        } else {
            Settings {
                // ..Default::default()
            }
        }
    }
}
