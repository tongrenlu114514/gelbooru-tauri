use std::fs;
use std::path::PathBuf;
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct ImageInfo {
    path: String,
    name: String,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TreeNode {
    pub key: String,
    pub label: String,
    pub path: String,
    pub is_leaf: bool,
    pub image_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<TreeNode>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubDirInfo {
    pub path: String,
    pub name: String,
    pub image_count: usize,
    pub thumbnail: Option<String>,
}

#[derive(Serialize)]
pub struct DirectoryImages {
    pub subdirs: Vec<SubDirInfo>,
    pub images: Vec<ImageInfo>,
    pub total: usize,
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

#[tauri::command]
pub async fn get_directory_tree(
    folder_path: Option<String>,
) -> Result<Vec<TreeNode>, String> {
    let default_path = "D:/project/gelbooru/imgs/";
    let path_str = folder_path.unwrap_or_else(|| default_path.to_string());
    
    let result = tokio::task::spawn_blocking(move || {
        let path = PathBuf::from(&path_str);
        
        if !path.exists() {
            return Vec::new();
        }
        
        fn build_tree(dir: &PathBuf, base_path: &str) -> Option<TreeNode> {
            if !dir.exists() || !dir.is_dir() {
                return None;
            }
            
            let mut children: Vec<TreeNode> = Vec::new();
            let mut image_count = 0usize;
            let mut first_image: Option<String> = None;
            
            if let Ok(entries) = fs::read_dir(dir) {
                let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
                entries.sort_by_key(|e| e.file_name());
                
                for entry in entries {
                    let entry_path = entry.path();
                    
                    if entry_path.is_dir() {
                        if let Some(child_node) = build_tree(&entry_path, base_path) {
                            image_count += child_node.image_count;
                            if first_image.is_none() {
                                first_image = child_node.thumbnail.clone();
                            }
                            children.push(child_node);
                        }
                    } else if let Some(ext) = entry_path.extension() {
                        let ext = ext.to_string_lossy().to_lowercase();
                        if ["jpg", "jpeg", "png", "gif", "webp"].contains(&ext.as_str()) {
                            image_count += 1;
                            if first_image.is_none() {
                                first_image = Some(entry_path.to_string_lossy().to_string());
                            }
                        }
                    }
                }
            }
            
            if image_count == 0 {
                return None;
            }
            
            let dir_name = dir.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| dir.to_string_lossy().to_string());
            
            let relative_path = dir.to_string_lossy().to_string();
            
            Some(TreeNode {
                key: relative_path.clone(),
                label: dir_name,
                path: relative_path,
                is_leaf: children.is_empty(),
                image_count,
                children: if children.is_empty() { None } else { Some(children) },
                thumbnail: first_image,
            })
        }
        
        let mut result = Vec::new();
        if let Ok(entries) = fs::read_dir(&path) {
            let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            entries.sort_by_key(|e| e.file_name());
            
            for entry in entries {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    if let Some(node) = build_tree(&entry_path, &path_str) {
                        result.push(node);
                    }
                }
            }
        }
        
        result
    }).await.map_err(|e| e.to_string())?;
    
    Ok(result)
}

#[tauri::command]
pub async fn get_directory_images(
    dir_path: String,
) -> Result<DirectoryImages, String> {
    let result = tokio::task::spawn_blocking(move || {
        let path = PathBuf::from(&dir_path);
        
        if !path.exists() || !path.is_dir() {
            return DirectoryImages {
                subdirs: Vec::new(),
                images: Vec::new(),
                total: 0,
            };
        }
        
        let mut subdirs: Vec<SubDirInfo> = Vec::new();
        let mut images_with_time: Vec<(String, std::time::SystemTime)> = Vec::new();
        
        if let Ok(entries) = fs::read_dir(&path) {
            let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
            entries.sort_by_key(|e| e.file_name());
            
            for entry in entries {
                let entry_path = entry.path();
                
                if entry_path.is_dir() {
                    // Count images in subdirectory and find first image as thumbnail
                    let mut sub_image_count = 0usize;
                    let mut first_image: Option<String> = None;
                    
                    fn count_images_in_dir(dir: &PathBuf, count: &mut usize, first: &mut Option<String>) {
                        if let Ok(entries) = fs::read_dir(dir) {
                            for entry in entries.filter_map(|e| e.ok()) {
                                let p = entry.path();
                                if p.is_dir() {
                                    count_images_in_dir(&p, count, first);
                                } else if let Some(ext) = p.extension() {
                                    let ext = ext.to_string_lossy().to_lowercase();
                                    if ["jpg", "jpeg", "png", "gif", "webp"].contains(&ext.as_str()) {
                                        *count += 1;
                                        if first.is_none() {
                                            *first = Some(p.to_string_lossy().to_string());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    
                    count_images_in_dir(&entry_path, &mut sub_image_count, &mut first_image);
                    
                    if sub_image_count > 0 {
                        let dir_name = entry_path.file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_else(|| "未命名".to_string());
                        
                        subdirs.push(SubDirInfo {
                            path: entry_path.to_string_lossy().to_string(),
                            name: dir_name,
                            image_count: sub_image_count,
                            thumbnail: first_image,
                        });
                    }
                } else if entry_path.is_file() {
                    if let Some(ext) = entry_path.extension() {
                        let ext = ext.to_string_lossy().to_lowercase();
                        if ["jpg", "jpeg", "png", "gif", "webp"].contains(&ext.as_str()) {
                            let mtime = fs::metadata(&entry_path)
                                .ok()
                                .and_then(|m| m.modified().ok())
                                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                            images_with_time.push((entry_path.to_string_lossy().to_string(), mtime));
                        }
                    }
                }
            }
        }
        
        images_with_time.sort_by(|a, b| b.1.cmp(&a.1));
        
        let images: Vec<ImageInfo> = images_with_time
            .into_iter()
            .map(|(path, _)| {
                let name = path.split(|c| c == '/' || c == '\\').last().unwrap_or(&path).to_string();
                ImageInfo { path, name }
            })
            .collect();
        
        let total = images.len();
        
        DirectoryImages { subdirs, images, total }
    }).await.map_err(|e| e.to_string())?;
    
    Ok(result)
}
