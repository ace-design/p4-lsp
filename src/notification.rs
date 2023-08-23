use tower_lsp::lsp_types::notification::Notification;

use serde::{Deserialize, Serialize};

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomParams {
    pub message: String,
    pub data: String,
}

impl CustomParams {
    pub fn new(
        message: String,
        data: String,
    ) -> CustomParams {
        CustomParams {
            message,data
        }
    }
}


#[derive(Debug)]
pub enum CustomNotification {}

impl Notification for CustomNotification{
     type Params = CustomParams;
    const METHOD: &'static str = "custom";
}