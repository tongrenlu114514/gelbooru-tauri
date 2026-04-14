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

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn gelbooru_tag_new_creates_tag() {
        let tag = GelbooruTag::new("sakura_kinomoto".to_string(), "artist".to_string(), 1234);

        assert_eq!(tag.text, "sakura_kinomoto");
        assert_eq!(tag.tag_type, "artist");
        assert_eq!(tag.count, 1234);
    }

    #[test]
    fn gelbooru_tag_default_values() {
        let tag = GelbooruTag {
            text: "test".to_string(),
            tag_type: "character".to_string(),
            count: 100,
        };

        assert_eq!(tag.text, "test");
        assert_eq!(tag.tag_type, "character");
        assert_eq!(tag.count, 100);
    }

    #[rstest]
    #[case("blue_eyes", "character", 5000)]
    #[case("solo", "general", 100000)]
    #[case("1girl", "general", 50000)]
    #[case("copyright", "copyright", 10)]
    fn gelbooru_tag_various_types(
        #[case] text: &str,
        #[case] tag_type: &str,
        #[case] count: u32,
    ) {
        let tag = GelbooruTag::new(text.to_string(), tag_type.to_string(), count);
        assert_eq!(tag.text, text);
        assert_eq!(tag.tag_type, tag_type);
        assert_eq!(tag.count, count);
    }
}
