use crate::models::{GelbooruPost, GelbooruTag, GelbooruPage};
use crate::services::{HttpClient, GelbooruScraper};
use std::sync::Arc;
use tokio::sync::RwLock;

lazy_static::lazy_static! {
    static ref HTTP_CLIENT: Arc<RwLock<HttpClient>> = Arc::new(RwLock::new(HttpClient::new().expect("Failed to create HTTP client")));
    static ref SCRAPER: GelbooruScraper = GelbooruScraper::new();
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub post_list: Vec<GelbooruPost>,
    pub tag_list: Vec<GelbooruTag>,
}

#[tauri::command]
pub async fn search_posts(tags: Vec<String>, page: u32) -> Result<SearchResult, String> {
    let client = HTTP_CLIENT.read().await;
    let url = SCRAPER.build_search_url(&tags, page);
    
    println!("[DEBUG] Fetching URL: {}", url);
    
    let html = client.get(&url).await
        .map_err(|e| {
            println!("[ERROR] HTTP request failed: {}", e);
            format!("HTTP request failed: {}", e)
        })?;
    
    println!("[DEBUG] Response length: {} bytes", html.len());
    
    let (post_list, tag_list) = SCRAPER.parse_page(&html);
    
    println!("[DEBUG] Parsed {} posts, {} tags", post_list.len(), tag_list.len());
    
    Ok(SearchResult { post_list, tag_list })
}

#[tauri::command]
pub async fn get_post_detail(id: u32) -> Result<GelbooruPost, String> {
    let client = HTTP_CLIENT.read().await;
    let url = SCRAPER.build_post_url(id);
    
    println!("[DEBUG] Getting post detail: {}", url);
    
    let html = client.get(&url).await
        .map_err(|e| format!("HTTP request failed: {}", e))?;
    
    let (tag_list, statistics) = SCRAPER.parse_post(&html)
        .ok_or_else(|| "Failed to parse post".to_string())?;
    
    println!("[DEBUG] Post {} image URL: {}", id, statistics.image);
    
    Ok(GelbooruPost {
        id,
        url: url.clone(),
        title: String::new(),
        tag_list,
        statistics,
        thumbnail: None,
    })
}

#[tauri::command]
pub async fn get_image_base64(url: String) -> Result<String, String> {
    let client = HTTP_CLIENT.read().await;
    
    println!("[DEBUG] Fetching image: {}", url);
    
    let bytes = client.get_image_with_referer(&url, "https://gelbooru.com/")
        .await
        .map_err(|e| format!("Failed to fetch image: {}", e))?;
    
    // 转换为 base64
    let base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
    
    // 检测图片类型
    let content_type = if url.ends_with(".png") {
        "image/png"
    } else if url.ends_with(".gif") {
        "image/gif"
    } else if url.ends_with(".webp") {
        "image/webp"
    } else {
        "image/jpeg"
    };
    
    Ok(format!("data:{};base64,{}", content_type, base64))
}
