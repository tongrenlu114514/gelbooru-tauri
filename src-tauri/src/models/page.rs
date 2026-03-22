use serde::{Deserialize, Serialize};
use super::{GelbooruPost, GelbooruTag};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GelbooruPage {
    pub page: String,
    pub s: String,
    pub tags: Vec<String>,
    pub page_num: u32,
    pub tag_list: Vec<GelbooruTag>,
    pub post_list: Vec<GelbooruPost>,
}

impl GelbooruPage {
    pub fn new(tags: Vec<String>, page_num: u32) -> Self {
        Self {
            page: "post".to_string(),
            s: "list".to_string(),
            tags,
            page_num,
            tag_list: Vec::new(),
            post_list: Vec::new(),
        }
    }
    
    pub fn index_request(tags: Vec<String>, page_num: u32) -> Self {
        Self::new(tags, page_num)
    }
}
