use serde::{Deserialize, Serialize};
use super::GelbooruTag;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GelbooruPostStatistics {
    pub size: String,
    pub rating: String,
    pub posted: String,
    pub source: String,
    pub score: i32,
    pub image: String,      // 原图 URL（用于下载）
    pub sample: String,     // 预览图 URL（用于显示）
}

impl Default for GelbooruPostStatistics {
    fn default() -> Self {
        Self {
            size: String::new(),
            rating: String::new(),
            posted: String::new(),
            source: String::new(),
            score: 0,
            image: String::new(),
            sample: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GelbooruPost {
    pub id: u32,
    pub url: String,
    pub title: String,
    pub tag_list: Vec<GelbooruTag>,
    pub statistics: GelbooruPostStatistics,
    pub thumbnail: Option<String>,
}

impl GelbooruPost {
    pub fn new(id: u32, url: String, title: String) -> Self {
        Self {
            id,
            url,
            title,
            tag_list: Vec::new(),
            statistics: GelbooruPostStatistics::default(),
            thumbnail: None,
        }
    }
}
