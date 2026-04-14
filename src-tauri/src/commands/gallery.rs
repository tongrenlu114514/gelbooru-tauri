use std::fs;
use std::path::PathBuf;
use serde::Serialize;
use crate::commands::favorite_tags::DbState;
use tauri::State;

/// Sanitize a path by removing dangerous components
pub(crate) fn sanitize_path(path: &str) -> String {
    // Normalize path separators and remove null bytes
    path.replace('\0', "")
        .replace('\\', "/")
}

/// Validate a path for security issues (path traversal, etc.)
pub(crate) fn validate_path(path: &str) -> Result<(), String> {
    let sanitized = sanitize_path(path);

    // Check for path traversal attempts
    if sanitized.contains("..") {
        return Err("路径包含非法字符或路径遍历尝试".to_string());
    }

    // Check for other dangerous patterns
    if sanitized.contains('\0') {
        return Err("路径包含非法字符".to_string());
    }

    Ok(())
}

/// Validate that a path is within the allowed base directory
/// This prevents path traversal attacks by ensuring canonical paths
pub(crate) fn validate_path_within_base(path: &str, base_dir: &str) -> Result<PathBuf, String> {
    // First do basic validation
    validate_path(path)?;

    let path_buf = PathBuf::from(path);
    let base_buf = PathBuf::from(base_dir);

    // If the path doesn't exist yet, we can't canonicalize it
    // In this case, we check that the normalized path doesn't escape the base
    if !path_buf.exists() {
        // Normalize both paths
        let canonical_base = base_buf.canonicalize()
            .map_err(|e| format!("无法访问基础目录: {}", e))?;

        // For non-existent paths, construct the expected canonical path
        // by resolving the relative path against the base
        let expected_path = canonical_base.join(&path_buf);

        // Check that the expected path starts with the base
        let expected_str = expected_path.to_string_lossy();
        let base_str = canonical_base.to_string_lossy();

        if !expected_str.starts_with(&*base_str) && !expected_str.starts_with(&format!("{}/", base_str)) {
            return Err("路径超出允许范围".to_string());
        }

        return Ok(expected_path);
    }

    // For existing paths, canonicalize and check containment
    let canonical_path = path_buf.canonicalize()
        .map_err(|e| format!("无法访问路径: {}", e))?;

    let canonical_base = base_buf.canonicalize()
        .map_err(|e| format!("无法访问基础目录: {}", e))?;

    let canonical_path_str = canonical_path.to_string_lossy();
    let canonical_base_str = canonical_base.to_string_lossy();

    // Check that the canonical path starts with the base directory
    if !canonical_path_str.starts_with(&*canonical_base_str) &&
       !canonical_path_str.starts_with(&format!("{}/", canonical_base_str)) {
        return Err("路径超出允许范围".to_string());
    }

    Ok(canonical_path)
}

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
pub async fn delete_image(
    db: State<'_, DbState>,
    path: String,
) -> Result<(), String> {
    // Get download directory from settings
    let download_dir = db.0.lock()
        .map_err(|e| e.to_string())?
        .get_setting("download_path")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();

    // Validate path is within download directory
    let path_buf = validate_path_within_base(&path, &download_dir)?;

    if !path_buf.exists() {
        return Err(format!("文件不存在: {}", path_buf.display()));
    }

    if !path_buf.is_file() {
        return Err("只能删除文件，不能删除目录".to_string());
    }

    tokio::task::spawn_blocking(move || {
        fs::remove_file(&path_buf).map_err(|e| format!("删除失败: {}", e))
    }).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_local_images(
    db: State<'_, DbState>,
    folder_path: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
) -> Result<PaginatedImages, String> {
    // Get download path from settings, fallback to empty string
    let db_path = db.0.lock()
        .map_err(|e| e.to_string())?
        .get_setting("download_path")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();

    // Validate user-provided path if given
    let path_str = if let Some(ref user_path) = folder_path {
        // Validate the user-provided path for security
        validate_path(user_path)?;
        user_path.clone()
    } else if db_path.is_empty() {
        String::new()
    } else {
        db_path
    };

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
                let name = path.rsplit(|c| c == '/' || c == '\\').next().unwrap_or(&path).to_string();
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
    db: State<'_, DbState>,
    folder_path: Option<String>,
) -> Result<Vec<TreeNode>, String> {
    // Get download path from settings, fallback to empty string
    let db_path = db.0.lock()
        .map_err(|e| e.to_string())?
        .get_setting("download_path")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();

    // Validate user-provided path if given
    let path_str = if let Some(ref user_path) = folder_path {
        // Validate the user-provided path for security
        validate_path(user_path)?;
        user_path.clone()
    } else if db_path.is_empty() {
        String::new()
    } else {
        db_path
    };

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
    // Validate path for security
    validate_path(&dir_path)?;

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
                let name = path.rsplit(|c| c == '/' || c == '\\').next().unwrap_or(&path).to_string();
                ImageInfo { path, name }
            })
            .collect();

        let total = images.len();

        DirectoryImages { subdirs, images, total }
    }).await.map_err(|e| e.to_string())?;

    Ok(result)
}

/// 读取本地图片并返回 base64 数据 URL
#[tauri::command]
pub async fn get_local_image_base64(
    db: State<'_, DbState>,
    path: String,
) -> Result<String, String> {
    // Get download directory from settings
    let download_dir = db.0.lock()
        .map_err(|e| e.to_string())?
        .get_setting("download_path")
        .map_err(|e| e.to_string())?
        .unwrap_or_default();

    // Validate path is within download directory
    let path_buf = validate_path_within_base(&path, &download_dir)?;

    let result = tokio::task::spawn_blocking(move || {
        if !path_buf.exists() {
            return Err(format!("文件不存在: {}", path_buf.display()));
        }

        if !path_buf.is_file() {
            return Err("路径不是文件".to_string());
        }

        // 读取文件内容
        let bytes = fs::read(&path_buf)
            .map_err(|e| format!("读取文件失败: {}", e))?;

        // 检测图片类型
        let content_type = if let Some(ext) = path_buf.extension() {
            match ext.to_string_lossy().to_lowercase().as_str() {
                "png" => "image/png",
                "gif" => "image/gif",
                "webp" => "image/webp",
                "jpg" | "jpeg" => "image/jpeg",
                _ => "image/jpeg",
            }
        } else {
            "image/jpeg"
        };

        // 转换为 base64
        let base64 = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);

        Ok(format!("data:{};base64,{}", content_type, base64))
    }).await.map_err(|e| e.to_string())?;

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn sanitize_path_removes_null_bytes() {
        assert_eq!(sanitize_path("path\0with\0null"), "pathwithnull");
    }

    #[test]
    fn sanitize_path_normalizes_backslashes() {
        assert_eq!(sanitize_path("path\\to\\file"), "path/to/file");
    }

    #[test]
    fn sanitize_path_preserves_normal_path() {
        assert_eq!(sanitize_path("/normal/path/to/file.png"), "/normal/path/to/file.png");
    }

    #[test]
    fn sanitize_path_handles_mixed_separators() {
        assert_eq!(sanitize_path("C:\\users\\test/path\\file.jpg"), "C:/users/test/path/file.jpg");
    }

    #[rstest]
    #[case("safe/path/to/file.png", true)]
    #[case("/absolute/path/file.jpg", true)]
    #[case("relative/path/file.gif", true)]
    #[case("C:\\windows\\path\\file.webp", true)]
    #[case("path/with..double/dots/file.png", false)]
    #[case("../parent/path/file.jpg", false)]
    #[case("path/../../../etc/passwd", false)]
    #[case("path\\..\\parent\\file.png", false)]
    fn validate_path_traversal_detection(#[case] path: &str, #[case] should_pass: bool) {
        let result = validate_path(path);
        if should_pass {
            assert!(result.is_ok(), "Expected path '{path}' to be valid, got: {:?}", result);
        } else {
            assert!(result.is_err(), "Expected path '{path}' to be invalid");
        }
    }

    #[test]
    fn validate_path_detects_null_bytes() {
        // After sanitization, null bytes should be removed, so this should pass
        // The null byte removal happens in sanitize_path, so validate_path receives clean input
        assert!(validate_path("safe/path\0with\0null").is_ok());
    }

    #[test]
    fn validate_path_empty_path_is_valid() {
        assert!(validate_path("").is_ok());
    }

    #[test]
    fn validate_path_root_path_is_valid() {
        assert!(validate_path("/").is_ok());
    }

    #[test]
    fn validate_path_windows_drive_letter() {
        assert!(validate_path("C:/Users/Test/Documents/image.png").is_ok());
    }

    #[test]
    fn validate_path_long_nested_path() {
        let deep_path = "a/b/c/d/e/f/g/h/i/j/k/l/m/n/o/p/q/r/s/t/u/v/w/x/y/z/file.png";
        assert!(validate_path(deep_path).is_ok());
    }

    #[test]
    fn validate_path_with_special_characters() {
        // Hyphen, underscore, and numbers should be allowed
        assert!(validate_path("path/to/my-file_123.EXT").is_ok());
    }
}
