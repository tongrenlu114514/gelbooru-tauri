use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GelbooruTag {
    pub text: String,
    pub tag_type: String,
    pub count: u32,
}

impl GelbooruTag {
    pub fn new(text: String, tag_type: String, count: u32) -> Self {
        Self { text, tag_type, count }
    }
}
