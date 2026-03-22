use std::fs;
use std::path::PathBuf;
use serde::Serialize;

#[derive(Serialize)]
pub struct ImageInfo {
    path: String,
    name: String,
}

#[derive(Serialize)]
pub struct PaginatedImages {
    images: Vec<ImageInfo>,
    total: usize,
    has_more: bool,
}

#[tauri::command]
pub async fn delete_image(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);
    
    if !path.exists() {
        return Err(format!("文件不存在: {}", path.display()));
    }
    
    tokio::task::spawn_blocking(move || {
        fs::remove_file(&path).map_err(|e| format!("删除失败: {}", e))
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_local_images(
    folder_path: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
) -> Result<PaginatedImages, String> {
    let default_path = "D:/project/gelbooru/imgs/";
    let path_str = folder_path.unwrap_or_else(|| default_path.to_string());
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(50);
    
    // Use spawn_blocking to avoid blocking the async runtime
    let result = tokio::task::spawn_blocking(move || {
        let path = PathBuf::from(&path_str);
        
        if !path.exists() {
            return PaginatedImages {
                images: Vec::new(),
                total: 0,
                has_more: false,
            };
        }
        
        // Collect images with their metadata in one pass
        let mut images_with_time: Vec<(String, std::time::SystemTime)> = Vec::new();
        
        fn collect_images(dir: &PathBuf, images: &mut Vec<(String, std::time::SystemTime)>) {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        collect_images(&path, images);
                    } else if let Some(ext) = path.extension() {
                        let ext = ext.to_string_lossy().to_lowercase();
                        if ["jpg", "jpeg", "png", "gif", "webp"].contains(&ext.as_str()) {
                            // Get metadata once during collection
                            let mtime = fs::metadata(&path)
                                .ok()
                                .and_then(|m| m.modified().ok())
                                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                            images.push((path.to_string_lossy().to_string(), mtime));
                        }
                    }
                }
            }
        }
        
        collect_images(&path, &mut images_with_time);
        
        // Sort once by modification time (newest first)
        images_with_time.sort_by(|a, b| b.1.cmp(&a.1));
        
        let total = images_with_time.len();
        let skip = (page.saturating_sub(1)) * page_size;
        
        // Paginate
        let paginated: Vec<ImageInfo> = images_with_time
            .into_iter()
            .skip(skip)
            .take(page_size)
            .map(|(path, _)| {
                let name = path.split(|c| c == '/' || c == '\\').last().unwrap_or(&path).to_string();
                ImageInfo { path, name }
            })
            .collect();
        
        PaginatedImages {
            images: paginated,
            total,
            has_more: skip + page_size < total,
        }
    }).await.map_err(|e| e.to_string())?;
    
    Ok(result)
}
