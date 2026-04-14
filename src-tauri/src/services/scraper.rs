use crate::models::{GelbooruPost, GelbooruPostStatistics, GelbooruTag};
use scraper::{Html, Selector};

const BASE_URL: &str = "https://gelbooru.com";
const PAGE_SIZE: u32 = 42;

const INCLUDE_TAGS: &[&str] = &["highres"];
const EXCLUDE_TAGS: &[&str] = &["-video", "-animated"];

pub struct GelbooruScraper;

impl GelbooruScraper {
    pub fn new() -> Self {
        Self
    }

    pub fn parse_page(&self, html: &str) -> (Vec<GelbooruPost>, Vec<GelbooruTag>, u32) {
        let document = Html::parse_document(html);
        let posts = self.parse_post_list(&document);
        let tags = self.parse_tag_list(&document);
        let total_pages = self.parse_pagination(&document);
        (posts, tags, total_pages)
    }

    pub fn parse_post(&self, html: &str) -> Option<(Vec<GelbooruTag>, GelbooruPostStatistics)> {
        let document = Html::parse_document(html);
        let tags = self.parse_tag_list(&document);
        let stats = self.parse_post_statistics(&document)?;
        Some((tags, stats))
    }

    fn parse_pagination(&self, document: &Html) -> u32 {
        let mut max_page = 1u32;

        // Gelbooru 分页结构分析：
        // 分页链接通常包含 pid 参数，pid = (page - 1) * 42

        // 方法1: 查找所有包含 pid 参数的链接，取最大值
        if let Ok(a_selector) = Selector::parse("a") {
            for a in document.select(&a_selector) {
                if let Some(href) = a.value().attr("href") {
                    // 检查链接是否包含 pid 参数
                    if href.contains("pid=") {
                        if let Some(pid_str) = href.split("pid=").nth(1) {
                            if let Ok(pid) = pid_str.split('&').next().unwrap_or("0").parse::<u32>()
                            {
                                let page = pid / PAGE_SIZE + 1;
                                if page > max_page {
                                    max_page = page;
                                    println!(
                                        "[DEBUG] Found page {} from pid {} in href: {}",
                                        page, pid, href
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        // 方法2: 查找特定的分页区域（Gelbooru 可能用不同的类名）
        let pagination_selectors = [".pagination", "#paginator", ".paginator", "nav.pagination"];
        for selector_str in &pagination_selectors {
            if let Ok(pagination_selector) = Selector::parse(selector_str) {
                if let Some(pagination) = document.select(&pagination_selector).next() {
                    println!("[DEBUG] Found pagination container: {}", selector_str);
                    if let Ok(a_selector) = Selector::parse("a") {
                        for a in pagination.select(&a_selector) {
                            if let Some(href) = a.value().attr("href") {
                                let text = a.text().collect::<String>();
                                println!("[DEBUG] Pagination link: {} -> {}", text, href);
                                if href.contains("pid=") {
                                    if let Some(pid_str) = href.split("pid=").nth(1) {
                                        if let Ok(pid) =
                                            pid_str.split('&').next().unwrap_or("0").parse::<u32>()
                                        {
                                            let page = pid / PAGE_SIZE + 1;
                                            if page > max_page {
                                                max_page = page;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // 方法3: 查找 "next" 或数字页码链接
        if let Ok(a_selector) = Selector::parse("a") {
            for a in document.select(&a_selector) {
                let text = a.text().collect::<String>().trim().to_lowercase();
                if let Some(href) = a.value().attr("href") {
                    // 检查 "next" 链接
                    if text == "next" || text == ">" || text == "›" || text == "→" {
                        if href.contains("pid=") {
                            if let Some(pid_str) = href.split("pid=").nth(1) {
                                if let Ok(pid) =
                                    pid_str.split('&').next().unwrap_or("0").parse::<u32>()
                                {
                                    // next 链接的 pid 是下一页的起始位置，所以总页数至少是下一页+1
                                    let page = pid / PAGE_SIZE + 2;
                                    if page > max_page {
                                        max_page = page;
                                        println!(
                                            "[DEBUG] Found next page {} from pid {}",
                                            page, pid
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        println!("[DEBUG] Final max_page: {}", max_page);
        max_page
    }

    fn parse_tag_list(&self, document: &Html) -> Vec<GelbooruTag> {
        let mut tags = Vec::new();

        let ul_selector = match Selector::parse("ul#tag-list") {
            Ok(s) => s,
            Err(_) => return tags,
        };

        let li_selector = match Selector::parse("li") {
            Ok(s) => s,
            Err(_) => return tags,
        };

        for ul in document.select(&ul_selector) {
            for li in ul.select(&li_selector) {
                let class = li.value().attr("class").unwrap_or("");
                if !class.starts_with("tag-type-") {
                    continue;
                }

                let tag_type = class.trim_start_matches("tag-type-").to_string();

                // Get tag text
                let a_selector = Selector::parse("a").ok();
                let text = if let Some(selector) = &a_selector {
                    li.select(selector)
                        .filter_map(|a| {
                            let text = a.text().collect::<String>();
                            let text = text.trim();
                            if text != "?" && text != "+" && text != "-" && !text.is_empty() {
                                Some(text.to_string())
                            } else {
                                None
                            }
                        })
                        .next()
                } else {
                    None
                };

                let text = match text {
                    Some(t) => t,
                    None => continue,
                };

                // Get count
                let span_selector = Selector::parse("span").ok();
                let count = if let Some(selector) = &span_selector {
                    li.select(selector)
                        .filter_map(|span| {
                            let style = span.value().attr("style").unwrap_or("");
                            if style == "color: #a0a0a0;" {
                                span.text().collect::<String>().trim().parse::<u32>().ok()
                            } else {
                                None
                            }
                        })
                        .next()
                } else {
                    None
                };

                tags.push(GelbooruTag::new(text, tag_type, count.unwrap_or(0)));
            }
        }

        tags
    }

    fn parse_post_list(&self, document: &Html) -> Vec<GelbooruPost> {
        let mut posts = Vec::new();

        let article_selector = match Selector::parse("article.thumbnail-preview") {
            Ok(s) => s,
            Err(_) => return posts,
        };

        let a_selector = Selector::parse("a").ok();
        let img_selector = Selector::parse("img").ok();

        for article in document.select(&article_selector) {
            let selector = match &a_selector {
                Some(s) => s,
                None => continue,
            };

            for link in article.select(selector) {
                let id = link
                    .value()
                    .attr("id")
                    .and_then(|id| id.trim_start_matches('p').parse::<u32>().ok());

                let url = link.value().attr("href").unwrap_or("").to_string();

                let img_elem = img_selector
                    .as_ref()
                    .and_then(|sel| link.select(sel).next());

                let title = img_elem
                    .and_then(|img| img.value().attr("title"))
                    .unwrap_or("")
                    .to_string();

                let thumbnail = img_selector
                    .as_ref()
                    .and_then(|sel| article.select(sel).next())
                    .and_then(|img| img.value().attr("src"))
                    .map(|s| s.to_string());

                if let Some(id) = id {
                    let mut post = GelbooruPost::new(id, url, title);
                    post.thumbnail = thumbnail;
                    posts.push(post);
                }
            }
        }

        posts
    }

    fn parse_post_statistics(&self, document: &Html) -> Option<GelbooruPostStatistics> {
        let mut stats = GelbooruPostStatistics::default();

        // 1. 获取预览图 URL (sample) - 从页面上的图片元素获取
        let sample_selectors = [
            "#image",               // 主图片元素
            "img[alt='image']",     // 带 alt 属性的图片
            ".image-container img", // 图片容器中的图片
        ];

        for selector_str in &sample_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(img) = document.select(&selector).next() {
                    if let Some(src) = img.value().attr("src") {
                        if !src.is_empty() {
                            stats.sample = src.to_string();
                            println!(
                                "[DEBUG] Found sample via selector '{}': {}",
                                selector_str, src
                            );
                            break;
                        }
                    }
                }
            }
        }

        // 2. 获取原图 URL (image) - 从 meta og:image 获取（用于下载）
        if let Ok(meta_selector) = Selector::parse("meta[property='og:image']") {
            if let Some(meta) = document.select(&meta_selector).next() {
                if let Some(content) = meta.value().attr("content") {
                    if !content.is_empty() {
                        stats.image = content.to_string();
                        println!("[DEBUG] Found original image via og:image: {}", content);
                    }
                }
            }
        }

        // 3. 解析其他统计信息
        let ul_selector = Selector::parse("ul#tag-list").ok()?;
        let li_selector = Selector::parse("li").ok()?;

        for ul in document.select(&ul_selector) {
            for li in ul.select(&li_selector) {
                if !li.value().attr("class").unwrap_or("").is_empty() {
                    continue;
                }

                let text = li.text().collect::<String>();
                let text = text.trim();

                if text.starts_with("Size:") {
                    stats.size = text.trim_start_matches("Size:").trim().to_string();
                }
                if text.starts_with("Rating:") {
                    stats.rating = text.trim_start_matches("Rating:").trim().to_string();
                }
                if text.starts_with("Posted:") {
                    let posted_str = text
                        .split("Uploader:")
                        .next()
                        .unwrap_or("")
                        .trim_start_matches("Posted:")
                        .trim();
                    stats.posted = posted_str.to_string();
                }
                if text.starts_with("Source:") {
                    stats.source = li
                        .select(&Selector::parse("a").ok()?)
                        .next()
                        .and_then(|a| a.value().attr("href"))
                        .unwrap_or("")
                        .to_string();
                }
                if text.starts_with("Score:") {
                    stats.score = li
                        .select(&Selector::parse("span").ok()?)
                        .next()
                        .and_then(|span| span.text().collect::<String>().trim().parse().ok())
                        .unwrap_or(0);
                }
                // 如果还没有获取到原图 URL，尝试从 Original image 链接获取
                if text.starts_with("Original image") && stats.image.is_empty() {
                    let img_url = li
                        .select(&Selector::parse("a").ok()?)
                        .next()
                        .and_then(|a| a.value().attr("href"))
                        .unwrap_or("")
                        .to_string();

                    if img_url.starts_with("//") {
                        stats.image = format!("https:{}", img_url);
                    } else if img_url.starts_with("/") {
                        stats.image = format!("https://img2.gelbooru.com{}", img_url);
                    } else {
                        stats.image = img_url;
                    }
                    println!(
                        "[DEBUG] Found original image via Original image link: {}",
                        stats.image
                    );
                }
            }
        }

        // 修正 URL 格式
        stats.sample = self.fix_image_url(&stats.sample);
        stats.image = self.fix_image_url(&stats.image);

        // 如果 image 为空，使用 sample 作为后备
        if stats.image.is_empty() {
            stats.image = stats.sample.clone();
        }

        println!("[DEBUG] Final sample URL: {}", stats.sample);
        println!("[DEBUG] Final image URL: {}", stats.image);

        Some(stats)
    }

    fn fix_image_url(&self, url: &str) -> String {
        if url.is_empty() {
            return String::new();
        }

        let mut fixed = url.to_string();

        // 确保 URL 是完整的
        if fixed.starts_with("//") {
            fixed = format!("https:{}", fixed);
        } else if fixed.starts_with("/") && !fixed.starts_with("//") {
            fixed = format!("https://img2.gelbooru.com{}", fixed);
        }

        // 修复 URL 中的双斜杠问题（在域名后面的路径部分）
        if let Some(pos) = fixed.find("://") {
            let prefix = &fixed[..pos + 3]; // "https://"
            let rest = &fixed[pos + 3..];
            let fixed_rest = rest.replace("//", "/");
            fixed = format!("{}{}", prefix, fixed_rest);
        }

        fixed
    }

    pub fn build_search_url(&self, tags: &[String], page: u32) -> String {
        let mut all_tags: Vec<String> = tags.to_vec();
        all_tags.extend(INCLUDE_TAGS.iter().map(|s| s.to_string()));
        all_tags.extend(EXCLUDE_TAGS.iter().map(|s| s.to_string()));

        let pid = (page.saturating_sub(1)) * PAGE_SIZE;

        format!(
            "{}/index.php?page=post&s=list&tags={}&pid={}",
            BASE_URL,
            urlencoding::encode(&all_tags.join(" ")),
            pid
        )
    }

    pub fn build_post_url(&self, id: u32) -> String {
        format!("{}/index.php?page=post&s=view&id={}", BASE_URL, id)
    }
}

impl Default for GelbooruScraper {
    fn default() -> Self {
        Self::new()
    }
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_page_extracts_post() {
        let html = r#"
            <html>
            <body>
            <article class="thumbnail-preview">
                <a id="p12345" href="/post/12345">
                    <img title="Beautiful Sakura" src="thumb.jpg">
                </a>
            </article>
            </body>
            </html>
        "#;

        let scraper = GelbooruScraper::new();
        let (posts, _, _) = scraper.parse_page(html);

        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0].id, 12345);
        assert_eq!(posts[0].url, "/post/12345");
        assert_eq!(posts[0].title, "Beautiful Sakura");
    }

    #[test]
    fn test_parse_page_extracts_multiple_posts() {
        let html = r#"
            <html>
            <body>
            <article class="thumbnail-preview">
                <a id="p100" href="/post/100">
                    <img title="Post 100" src="thumb1.jpg">
                </a>
            </article>
            <article class="thumbnail-preview">
                <a id="p200" href="/post/200">
                    <img title="Post 200" src="thumb2.jpg">
                </a>
            </article>
            <article class="thumbnail-preview">
                <a id="p300" href="/post/300">
                    <img title="Post 300" src="thumb3.jpg">
                </a>
            </article>
            </body>
            </html>
        "#;

        let scraper = GelbooruScraper::new();
        let (posts, _, _) = scraper.parse_page(html);

        assert_eq!(posts.len(), 3);
        assert_eq!(posts[0].id, 100);
        assert_eq!(posts[1].id, 200);
        assert_eq!(posts[2].id, 300);
    }

    #[test]
    fn test_parse_page_extracts_tags() {
        let html = r#"
            <html>
            <body>
            <ul id="tag-list">
                <li class="tag-type-artist">
                    <a>artist_name</a>
                    <span style="color: #a0a0a0;">100</span>
                </li>
                <li class="tag-type-character">
                    <a>saber</a>
                    <span style="color: #a0a0a0;">500</span>
                </li>
                <li class="tag-type-copyright">
                    <a>fate_series</a>
                    <span style="color: #a0a0a0;">200</span>
                </li>
            </ul>
            </body>
            </html>
        "#;

        let scraper = GelbooruScraper::new();
        let (_, tags, _) = scraper.parse_page(html);

        assert_eq!(tags.len(), 3);
        assert!(tags
            .iter()
            .any(|t| t.text == "artist_name" && t.tag_type == "artist"));
        assert!(tags
            .iter()
            .any(|t| t.text == "saber" && t.tag_type == "character"));
        assert!(tags
            .iter()
            .any(|t| t.text == "fate_series" && t.tag_type == "copyright"));
    }

    #[test]
    fn test_parse_page_handles_empty_html() {
        let html = "<html><body></body></html>";
        let scraper = GelbooruScraper::new();
        let (posts, tags, pages) = scraper.parse_page(html);

        assert!(posts.is_empty());
        assert!(tags.is_empty());
        assert_eq!(pages, 1); // default minimum
    }

    #[test]
    fn test_parse_page_extracts_thumbnail() {
        let html = r#"
            <html>
            <body>
            <article class="thumbnail-preview">
                <a id="p12345" href="/post/12345">
                    <img title="Test" src="https://img2.gelbooru.com/thumbnails/123/thumbnail.jpg">
                </a>
            </article>
            </body>
            </html>
        "#;

        let scraper = GelbooruScraper::new();
        let (posts, _, _) = scraper.parse_page(html);

        assert!(posts[0].thumbnail.is_some());
        assert_eq!(
            posts[0].thumbnail.as_ref().unwrap(),
            "https://img2.gelbooru.com/thumbnails/123/thumbnail.jpg"
        );
    }

    #[test]
    fn test_fix_image_url_protocol_relative() {
        let scraper = GelbooruScraper::new();
        assert_eq!(
            scraper.fix_image_url("//example.com/image.jpg"),
            "https://example.com/image.jpg"
        );
    }

    #[test]
    fn test_fix_image_url_root_relative() {
        let scraper = GelbooruScraper::new();
        assert_eq!(
            scraper.fix_image_url("/images/sample.png"),
            "https://img2.gelbooru.com/images/sample.png"
        );
    }

    #[test]
    fn test_fix_image_url_already_absolute() {
        let scraper = GelbooruScraper::new();
        assert_eq!(
            scraper.fix_image_url("https://img2.gelbooru.com/images/abc.jpg"),
            "https://img2.gelbooru.com/images/abc.jpg"
        );
    }

    #[test]
    fn test_fix_image_url_empty_string() {
        let scraper = GelbooruScraper::new();
        assert_eq!(scraper.fix_image_url(""), "");
    }

    #[test]
    fn test_fix_image_url_removes_double_slashes() {
        let scraper = GelbooruScraper::new();
        assert_eq!(
            scraper.fix_image_url("https://example.com//images//test.jpg"),
            "https://example.com/images/test.jpg"
        );
    }

    #[test]
    fn test_fix_image_url_preserves_https() {
        let scraper = GelbooruScraper::new();
        assert_eq!(
            scraper.fix_image_url("https://img2.gelbooru.com/samples/abc.jpg"),
            "https://img2.gelbooru.com/samples/abc.jpg"
        );
    }

    #[test]
    fn test_build_search_url() {
        let scraper = GelbooruScraper::new();
        let tags = vec!["saber".to_string(), "blue_eyes".to_string()];
        let url = scraper.build_search_url(&tags, 1);

        assert!(url.contains("tags=saber+blue_eyes"));
        assert!(url.contains("pid=0"));
        assert!(url.contains("highres")); // included tag
        assert!(url.contains("-video")); // excluded tag
        assert!(url.contains("https://gelbooru.com"));
    }

    #[test]
    fn test_build_search_url_pagination() {
        let scraper = GelbooruScraper::new();
        let tags = vec!["tag".to_string()];
        let url = scraper.build_search_url(&tags, 3);

        // page 3 -> pid = (3-1) * 42 = 84
        assert!(url.contains("pid=84"));
    }

    #[test]
    fn test_build_search_url_page_one() {
        let scraper = GelbooruScraper::new();
        let tags = vec!["1girl".to_string()];
        let url = scraper.build_search_url(&tags, 1);

        // page 1 -> pid = 0
        assert!(url.contains("pid=0"));
    }

    #[test]
    fn test_build_search_url_empty_tags() {
        let scraper = GelbooruScraper::new();
        let tags: Vec<String> = vec![];
        let url = scraper.build_search_url(&tags, 1);

        assert!(url.contains("tags="));
        assert!(url.contains("highres"));
        assert!(url.contains("-video"));
    }

    #[test]
    fn test_build_post_url() {
        let scraper = GelbooruScraper::new();
        let url = scraper.build_post_url(12345);
        assert_eq!(
            url,
            "https://gelbooru.com/index.php?page=post&s=view&id=12345"
        );
    }

    #[test]
    fn test_build_post_url_zero() {
        let scraper = GelbooruScraper::new();
        let url = scraper.build_post_url(0);
        assert_eq!(url, "https://gelbooru.com/index.php?page=post&s=view&id=0");
    }

    #[test]
    fn test_build_post_url_large_id() {
        let scraper = GelbooruScraper::new();
        let url = scraper.build_post_url(99999999);
        assert_eq!(
            url,
            "https://gelbooru.com/index.php?page=post&s=view&id=99999999"
        );
    }

    #[test]
    fn test_new_and_default() {
        let scraper = GelbooruScraper::new();
        let default = GelbooruScraper::default();
        // Both should work identically
        let html = "<html><body></body></html>";
        let (posts1, _, _) = scraper.parse_page(html);
        let (posts2, _, _) = default.parse_page(html);
        assert_eq!(posts1.len(), posts2.len());
    }

    #[test]
    fn test_parse_page_extracts_tag_counts() {
        let html = r#"
            <html>
            <body>
            <ul id="tag-list">
                <li class="tag-type-general">
                    <a>1girl</a>
                    <span style="color: #a0a0a0;">50000</span>
                </li>
                <li class="tag-type-meta">
                    <a>absres</a>
                    <span style="color: #a0a0a0;">25000</span>
                </li>
            </ul>
            </body>
            </html>
        "#;

        let scraper = GelbooruScraper::new();
        let (_, tags, _) = scraper.parse_page(html);

        assert_eq!(tags.len(), 2);
        // Find the 1girl tag and verify its count
        let general_tag = tags.iter().find(|t| t.text == "1girl").unwrap();
        assert_eq!(general_tag.count, 50000);
    }
}
