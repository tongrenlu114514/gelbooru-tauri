use super::{GelbooruPost, GelbooruTag};
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
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
    #[allow(dead_code)]
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

    #[allow(dead_code)]
    pub fn index_request(tags: Vec<String>, page_num: u32) -> Self {
        Self::new(tags, page_num)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn gelbooru_page_new_creates_page() {
        let tags = vec!["1girl".to_string(), "solo".to_string()];
        let page = GelbooruPage::new(tags.clone(), 1);

        assert_eq!(page.page, "post");
        assert_eq!(page.s, "list");
        assert_eq!(page.tags, tags);
        assert_eq!(page.page_num, 1);
        assert!(page.tag_list.is_empty());
        assert!(page.post_list.is_empty());
    }

    #[test]
    fn gelbooru_page_index_request() {
        let tags = vec!["blue_eyes".to_string()];
        let page = GelbooruPage::index_request(tags.clone(), 5);

        assert_eq!(page.tags, tags);
        assert_eq!(page.page_num, 5);
    }

    #[test]
    fn gelbooru_page_empty_tags() {
        let page = GelbooruPage::new(Vec::new(), 1);

        assert!(page.tags.is_empty());
    }

    #[test]
    fn gelbooru_page_with_posts() {
        let mut page = GelbooruPage::new(vec!["1girl".to_string()], 1);
        page.post_list.push(GelbooruPost::new(
            1,
            "url1".to_string(),
            "post1".to_string(),
        ));
        page.post_list.push(GelbooruPost::new(
            2,
            "url2".to_string(),
            "post2".to_string(),
        ));

        assert_eq!(page.post_list.len(), 2);
    }

    #[test]
    fn gelbooru_page_with_tag_list() {
        let mut page = GelbooruPage::new(vec!["sakura_kinomoto".to_string()], 1);
        page.tag_list.push(GelbooruTag::new(
            "artist_tag".to_string(),
            "artist".to_string(),
            100,
        ));

        assert_eq!(page.tag_list.len(), 1);
        assert_eq!(page.tag_list[0].text, "artist_tag");
    }

    #[rstest]
    #[case(vec![], 1)]
    #[case(vec!["1girl"], 1)]
    #[case(vec!["blue_eyes", "blonde_hair", "1girl"], 10)]
    #[case(vec!["rating:safe", "order:score"], 5)]
    fn gelbooru_page_various_page_numbers(#[case] tags: Vec<&str>, #[case] page_num: u32) {
        let tags_owned: Vec<String> = tags.into_iter().map(|s| s.to_string()).collect();
        let page = GelbooruPage::new(tags_owned.clone(), page_num);
        assert_eq!(page.page_num, page_num);
        assert_eq!(page.tags, tags_owned);
    }
}
