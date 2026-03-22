use scraper::{Html, Selector};
use crate::models::{GelbooruPost, GelbooruTag, GelbooruPostStatistics};
use regex::Regex;

const BASE_URL: &str = "https://gelbooru.com";
const PAGE_SIZE: u32 = 42;

const INCLUDE_TAGS: &[&str] = &["highres"];
const EXCLUDE_TAGS: &[&str] = &["-video", "-animated"];

pub struct GelbooruScraper;

impl GelbooruScraper {
    pub fn new() -> Self {
        Self
    }
    
    pub fn parse_page(&self, html: &str) -> (Vec<GelbooruPost>, Vec<GelbooruTag>) {
        let document = Html::parse_document(html);
        let posts = self.parse_post_list(&document);
        let tags = self.parse_tag_list(&document);
        (posts, tags)
    }
    
    pub fn parse_post(&self, html: &str) -> Option<(Vec<GelbooruTag>, GelbooruPostStatistics)> {
        let document = Html::parse_document(html);
        let tags = self.parse_tag_list(&document);
        let stats = self.parse_post_statistics(&document)?;
        Some((tags, stats))
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
                                span.text().collect::<String>()
                                    .trim()
                                    .parse::<u32>()
                                    .ok()
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
                let id = link.value().attr("id")
                    .and_then(|id| id.trim_start_matches('p').parse::<u32>().ok());
                
                let url = link.value().attr("href").unwrap_or("").to_string();
                
                let img_elem = img_selector.as_ref()
                    .and_then(|sel| link.select(sel).next());
                
                let title = img_elem
                    .and_then(|img| img.value().attr("title"))
                    .unwrap_or("")
                    .to_string();
                
                let thumbnail = img_selector.as_ref()
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
            "#image",                    // 主图片元素
            "img[alt='image']",          // 带 alt 属性的图片
            ".image-container img",      // 图片容器中的图片
        ];
        
        for selector_str in &sample_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                if let Some(img) = document.select(&selector).next() {
                    if let Some(src) = img.value().attr("src") {
                        if !src.is_empty() {
                            stats.sample = src.to_string();
                            println!("[DEBUG] Found sample via selector '{}': {}", selector_str, src);
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
                    let posted_str = text.split("Uploader:").next().unwrap_or("")
                        .trim_start_matches("Posted:")
                        .trim();
                    stats.posted = posted_str.to_string();
                }
                if text.starts_with("Source:") {
                    stats.source = li.select(&Selector::parse("a").ok()?)
                        .next()
                        .and_then(|a| a.value().attr("href"))
                        .unwrap_or("")
                        .to_string();
                }
                if text.starts_with("Score:") {
                    stats.score = li.select(&Selector::parse("span").ok()?)
                        .next()
                        .and_then(|span| span.text().collect::<String>().trim().parse().ok())
                        .unwrap_or(0);
                }
                // 如果还没有获取到原图 URL，尝试从 Original image 链接获取
                if text.starts_with("Original image") && stats.image.is_empty() {
                    let img_url = li.select(&Selector::parse("a").ok()?)
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
                    println!("[DEBUG] Found original image via Original image link: {}", stats.image);
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
