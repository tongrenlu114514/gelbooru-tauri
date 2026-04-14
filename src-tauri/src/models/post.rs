use super::GelbooruTag;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GelbooruPostStatistics {
    pub size: String,
    pub rating: String,
    pub posted: String,
    pub source: String,
    pub score: i32,
    pub image: String,  // 原图 URL（用于下载）
    pub sample: String, // 预览图 URL（用于显示）
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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn gelbooru_post_new_creates_post() {
        let post = GelbooruPost::new(
            12345,
            "https://example.com/post".to_string(),
            "Test Post".to_string(),
        );

        assert_eq!(post.id, 12345);
        assert_eq!(post.url, "https://example.com/post");
        assert_eq!(post.title, "Test Post");
        assert!(post.tag_list.is_empty());
        assert!(post.thumbnail.is_none());
    }

    #[test]
    fn gelbooru_post_statistics_default() {
        let stats = GelbooruPostStatistics::default();

        assert_eq!(stats.size, "");
        assert_eq!(stats.rating, "");
        assert_eq!(stats.posted, "");
        assert_eq!(stats.source, "");
        assert_eq!(stats.score, 0);
        assert_eq!(stats.image, "");
        assert_eq!(stats.sample, "");
    }

    #[test]
    fn gelbooru_post_statistics_with_values() {
        let stats = GelbooruPostStatistics {
            size: "1920x1080".to_string(),
            rating: "s".to_string(),
            posted: "2024-01-15".to_string(),
            source: "original".to_string(),
            score: 1500,
            image: "https://example.com/image.jpg".to_string(),
            sample: "https://example.com/sample.jpg".to_string(),
        };

        assert_eq!(stats.size, "1920x1080");
        assert_eq!(stats.rating, "s");
        assert_eq!(stats.score, 1500);
    }

    #[test]
    fn gelbooru_post_with_thumbnail() {
        let mut post = GelbooruPost::new(1, "url".to_string(), "title".to_string());
        post.thumbnail = Some("https://example.com/thumb.jpg".to_string());

        assert!(post.thumbnail.is_some());
        assert_eq!(post.thumbnail.unwrap(), "https://example.com/thumb.jpg");
    }

    #[test]
    fn gelbooru_post_with_tags() {
        let mut post = GelbooruPost::new(1, "url".to_string(), "title".to_string());
        post.tag_list.push(GelbooruTag::new(
            "1girl".to_string(),
            "general".to_string(),
            100,
        ));
        post.tag_list.push(GelbooruTag::new(
            "solo".to_string(),
            "general".to_string(),
            200,
        ));

        assert_eq!(post.tag_list.len(), 2);
        assert_eq!(post.tag_list[0].text, "1girl");
        assert_eq!(post.tag_list[1].text, "solo");
    }

    #[rstest]
    #[case(1, "url1", "title1")]
    #[case(
        999999,
        "https://gelbooru.com/index.php?page=post&s=view&id=123",
        "Sample Post"
    )]
    #[case(0, "", "Minimal Post")]
    fn gelbooru_post_various_ids(#[case] id: u32, #[case] url: &str, #[case] title: &str) {
        let post = GelbooruPost::new(id, url.to_string(), title.to_string());
        assert_eq!(post.id, id);
        assert_eq!(post.url, url);
        assert_eq!(post.title, title);
    }
}
