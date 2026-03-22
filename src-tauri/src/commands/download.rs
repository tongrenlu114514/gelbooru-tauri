use crate::services::HttpClient;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::path::PathBuf;
use std::fs;

lazy_static::lazy_static! {
    static ref DOWNLOAD_CLIENT: Arc<RwLock<HttpClient>> = Arc::new(RwLock::new(HttpClient::new().expect("Failed to create HTTP client")));
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DownloadProgress {
    pub id: u32,
    pub post_id: u32,
    pub status: String,
    pub progress: f32,
    pub downloaded: u64,
    pub total: u64,
}

#[tauri::command]
pub async fn start_download(
    post_id: u32,
    image_url: String,
    file_name: String,
    download_path: String,
) -> Result<String, String> {
    let client = DOWNLOAD_CLIENT.read().await;
    
    // Create download directory if not exists
    let path = PathBuf::from(&download_path);
    if !path.exists() {
        fs::create_dir_all(&path)
            .map_err(|e| format!("Failed to create directory: {}", e))?;
    }
    
    let file_path = path.join(&file_name);
    
    // Download the image
    let bytes = client.get_bytes(&image_url).await
        .map_err(|e| format!("Download failed: {}", e))?;
    
    // Save to file
    fs::write(&file_path, &bytes)
        .map_err(|e| format!("Failed to write file: {}", e))?;
    
    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn pause_download(id: u32) -> Result<(), String> {
    // TODO: Implement pause logic with download manager
    Ok(())
}

#[tauri::command]
pub async fn resume_download(id: u32) -> Result<(), String> {
    // TODO: Implement resume logic with download manager
    Ok(())
}

#[tauri::command]
pub async fn get_download_progress() -> Result<Vec<DownloadProgress>, String> {
    // TODO: Return actual progress from download manager
    Ok(Vec::new())
}
