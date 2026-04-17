use crate::commands::favorite_tags::DbState;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::State;

// Error handling audit (Phase 4, REQ-4.2):
// All commands return Result<T, String> — confirmed consistent across:
//   gelbooru.rs, download.rs, gallery.rs, settings.rs, favorite_tags.rs
// Chinese error messages in gallery.rs (e.g. "删除失败") are intentional for UI.
// println!/eprintln! patterns preserved as-is per D-04.
//
/// Sanitize a path by removing dangerous components
pub(crate) fn sanitize_path(path: &str) -> String {
    // Normalize path separators and remove null bytes
    path.replace('\0', "").replace('\\', "/")
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
        let canonical_base = base_buf
            .canonicalize()
            .map_err(|e| format!("无法访问基础目录: {}", e))?;

        // For non-existent paths, construct the expected canonical path
        // by resolving the relative path against the base
        let expected_path = canonical_base.join(&path_buf);

        // Check that the expected path starts with the base
        let expected_str = expected_path.to_string_lossy();
        let base_str = canonical_base.to_string_lossy();

        if !expected_str.starts_with(&*base_str)
            && !expected_str.starts_with(&format!("{}/", base_str))
        {
            return Err("路径超出允许范围".to_string());
        }

        return Ok(expected_path);
    }

    // For existing paths, canonicalize and check containment
    let canonical_path = path_buf
        .canonicalize()
        .map_err(|e| format!("无法访问路径: {}", e))?;

    let canonical_base = base_buf
        .canonicalize()
        .map_err(|e| format!("无法访问基础目录: {}", e))?;

    let canonical_path_str = canonical_path.to_string_lossy();
    let canonical_base_str = canonical_base.to_string_lossy();

    // Check that the canonical path starts with the base directory
    if !canonical_path_str.starts_with(&*canonical_base_str)
        && !canonical_path_str.starts_with(&format!("{}/", canonical_base_str))
    {
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
pub async fn delete_image(db: State<'_, DbState>, path: String) -> Result<(), String> {
    // Get download directory from settings
    let download_dir =
        db.0.lock()
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
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn get_local_images(
    db: State<'_, DbState>,
    folder_path: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
) -> Result<PaginatedImages, String> {
    // Get download path from settings, fallback to empty string
    let db_path =
        db.0.lock()
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
                let name = path.rsplit(['/', '\\']).next().unwrap_or(&path).to_string();
                ImageInfo { path, name }
            })
            .collect();

        PaginatedImages {
            images: paginated,
            total,
            has_more: skip + page_size < total,
        }
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(result)
}

#[tauri::command]
pub async fn get_directory_tree(
    db: State<'_, DbState>,
    folder_path: Option<String>,
) -> Result<Vec<TreeNode>, String> {
    // Get download path from settings, fallback to empty string
    let db_path =
        db.0.lock()
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

    let result = build_tree_async(PathBuf::from(&path_str)).await;

    Ok(result)
}

#[tauri::command]
pub async fn get_directory_images(dir_path: String) -> Result<DirectoryImages, String> {
    // Validate path for security
    validate_path(&dir_path)?;

    let result = get_directory_images_async(PathBuf::from(&dir_path)).await;

    Ok(result)
}

/// 读取本地图片并返回 base64 数据 URL
#[tauri::command]
pub async fn get_local_image_base64(
    db: State<'_, DbState>,
    path: String,
) -> Result<String, String> {
    // Get download directory from settings
    let download_dir =
        db.0.lock()
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
        let bytes = fs::read(&path_buf).map_err(|e| format!("读取文件失败: {}", e))?;

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
    })
    .await
    .map_err(|e| e.to_string())?;

    result
}

const MAX_CONCURRENT_DIRS: usize = 10;

/// Returns true if the given path has an image extension (case-insensitive).
fn is_image(path: &std::path::Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| ["jpg", "jpeg", "png", "gif", "webp"].contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

/// Canonicalize a path, falling back to the original if canonicalization fails.
/// This normalizes Windows short-path names (e.g. `ADMINI~1`) to long names
/// so they match what `fs::read_dir` returns on this platform.
fn canonical_path(path: PathBuf) -> PathBuf {
    path.canonicalize().unwrap_or(path)
}

/// Scan a single directory asynchronously.
/// Semaphore limits concurrent directory handles to prevent FD exhaustion.
/// Uses spawn_blocking for directory enumeration (Windows: ReadDir is not Send).
/// Child subdirectories are traversed in parallel using std::thread::scope.
async fn scan_dir(
    dir: PathBuf,
    sem: Arc<tokio::sync::Semaphore>,
) -> Option<TreeNode> {
    // Acquire permit before spawn_blocking to prevent FD exhaustion deadlock
    let _permit = sem.acquire().await.expect("semaphore closed");

    // Move dir and a sem clone into spawn_blocking. The closure owns these values
    // (Arc is Send + Clone), satisfying the 'static bound. The permit is implicitly
    // dropped when this async fn returns, after spawn_blocking completes.
    let dir_clone = dir.clone();
    let sem_clone = sem.clone();
    tokio::task::spawn_blocking(move || {
        scan_dir_sync(dir_clone, sem_clone)
    })
    .await
    .ok()
    .flatten()
}

/// Synchronous recursive directory scan — called from spawn_blocking.
/// Uses std::thread::scope for parallel child subdirectory traversal.
fn scan_dir_sync(
    dir: PathBuf,
    sem: Arc<tokio::sync::Semaphore>,
) -> Option<TreeNode> {
    let mut subdirs: Vec<PathBuf> = Vec::new();
    let mut image_count: usize = 0;
    let mut first_image: Option<String> = None;

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                subdirs.push(path);
            } else if is_image(&path) {
                image_count += 1;
                if first_image.is_none() {
                    first_image = Some(path.to_string_lossy().to_string());
                }
            }
        }
    }

    let mut children: Vec<TreeNode> = Vec::new();

    // Parallel child subdirectory traversal using std::thread::scope
    std::thread::scope(|s| {
        let handles: Vec<_> = subdirs
            .into_iter()
            .map(|subdir| {
                let sem = Arc::clone(&sem);
                s.spawn(move || scan_dir_sync(subdir, sem))
            })
            .collect();

        for handle in handles {
            if let Ok(Some(child)) = handle.join() {
                image_count += child.image_count;
                if first_image.is_none() {
                    first_image = child.thumbnail.clone();
                }
                children.push(child);
            }
        }
    });

    if image_count == 0 {
        return None;
    }

    let dir_name = dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| dir.to_string_lossy().to_string());

    // A node is a leaf only if it has no child subdirectories with images.
    let is_leaf = children.is_empty();

    Some(TreeNode {
        key: dir.to_string_lossy().to_string(),
        label: dir_name,
        path: dir.to_string_lossy().to_string(),
        is_leaf,
        image_count,
        children: Some(children),
        thumbnail: first_image,
    })
}

/// Build directory tree asynchronously for a root path.
/// The root directory is always returned as result[0] with aggregated image counts.
/// Immediate subdirectories are scanned in parallel, each with bounded concurrency via Semaphore.
async fn build_tree_async(root: PathBuf) -> Vec<TreeNode> {
    // Canonicalize root so all path comparisons are consistent on Windows.
    let root = canonical_path(root);

    let sem = Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_DIRS));

    // Scan root directory to find immediate subdirectories
    let mut entries = match tokio::fs::read_dir(&root).await {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut root_own_images: usize = 0;
    let mut root_thumbnail: Option<String> = None;
    let mut subdirs: Vec<PathBuf> = Vec::new();

    while let Some(entry) = entries.next_entry().await.ok().flatten() {
        let ft = entry.file_type().await.map(|ft| ft.is_dir()).unwrap_or(false);
        if ft {
            subdirs.push(entry.path());
        } else {
            let path = entry.path();
            if is_image(&path) {
                root_own_images += 1;
                if root_thumbnail.is_none() {
                    root_thumbnail = Some(path.to_string_lossy().to_string());
                }
            }
        }
    }

    // Spawn parallel scans for each immediate subdirectory
    let futures: Vec<_> = subdirs
        .into_iter()
        .map(|subdir| {
            let sem = sem.clone();
            tokio::spawn(async move { scan_dir(subdir, sem).await })
        })
        .collect();

    let mut child_nodes: Vec<TreeNode> = Vec::new();
    let mut total_from_children: usize = 0;

    for future in futures {
        if let Ok(Some(node)) = future.await {
            total_from_children += node.image_count;
            if root_thumbnail.is_none() {
                root_thumbnail = node.thumbnail.clone();
            }
            child_nodes.push(node);
        }
    }

    child_nodes.sort_by_key(|n| n.label.clone());

    // Root node always included as result[0]
    let root_name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| root.to_string_lossy().to_string());

    let root_node = TreeNode {
        key: root.to_string_lossy().to_string(),
        label: root_name,
        path: root.to_string_lossy().to_string(),
        is_leaf: child_nodes.is_empty(),
        image_count: root_own_images + total_from_children,
        children: if child_nodes.is_empty() { None } else { Some(child_nodes) },
        thumbnail: root_thumbnail,
    };

    let result = vec![root_node];
    result
}

/// Recursively count images in a directory with semaphore-bounded concurrency.
/// Acquires the permit at START (before any awaits) to prevent deadlock in deep trees.
async fn count_dir_images(
    dir: std::path::PathBuf,
    sem: Arc<tokio::sync::Semaphore>,
) -> (usize, Option<String>) {
    // Clone sem first so we can acquire on the clone (avoid borrow of `sem` which moves
    // into spawn_blocking). Both Arcs point to the same underlying Semaphore — the permit
    // is released from the correct semaphore when _permit drops.
    let sem_clone = sem.clone();
    let _permit = sem_clone.acquire().await.expect("semaphore closed");

    tokio::task::spawn_blocking(move || {
        count_dir_recursive(dir, sem)
    })
    .await
    .unwrap_or((0, None))
}

/// Synchronous recursive helper — called from spawn_blocking.
/// Uses std::thread::scope for parallel child subdirectory traversal.
fn count_dir_recursive(
    dir: std::path::PathBuf,
    sem: Arc<tokio::sync::Semaphore>,
) -> (usize, Option<String>) {
    let mut subdirs: Vec<std::path::PathBuf> = Vec::new();
    let mut count = 0;
    let mut first: Option<String> = None;

    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                subdirs.push(path);
            } else if is_image(&path) {
                count += 1;
                if first.is_none() {
                    first = Some(path.to_string_lossy().to_string());
                }
            }
        }
    }

    // Parallel child subdirectory traversal using std::thread::scope
    std::thread::scope(|s| {
        let handles: Vec<_> = subdirs
            .into_iter()
            .map(|subdir| {
                let sem = Arc::clone(&sem);
                s.spawn(move || count_dir_recursive(subdir, sem))
            })
            .collect();

        for handle in handles {
            if let Ok((sub_count, sub_first)) = handle.join() {
                count += sub_count;
                if first.is_none() {
                    first = sub_first;
                }
            }
        }
    });

    (count, first)
}

/// Count images in a directory asynchronously (top-level wrapper with its own semaphore).
async fn count_images_async(dir: std::path::PathBuf) -> (usize, Option<String>) {
    let sem = Arc::new(tokio::sync::Semaphore::new(MAX_CONCURRENT_DIRS));
    count_dir_images(dir, sem).await
}

/// List subdirectories and images for a given directory path asynchronously.
async fn get_directory_images_async(dir_path: std::path::PathBuf) -> DirectoryImages {
    let mut subdirs: Vec<SubDirInfo> = Vec::new();
    let mut images_with_time: Vec<(String, std::time::SystemTime)> = Vec::new();

    let mut entries = match tokio::fs::read_dir(&dir_path).await {
        Ok(e) => e,
        Err(_) => {
            return DirectoryImages {
                subdirs,
                images: Vec::new(),
                total: 0,
            }
        }
    };

    while let Some(entry) = entries.next_entry().await.ok().flatten() {
        let path = entry.path();
        let metadata = match tokio::fs::metadata(&path).await.ok() {
            Some(m) => m,
            None => continue,
        };

        if metadata.is_dir() {
            let (count, first) = count_images_async(path.clone()).await;
            if count > 0 {
                let dir_name = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "未命名".to_string());
                subdirs.push(SubDirInfo {
                    path: path.to_string_lossy().to_string(),
                    name: dir_name,
                    image_count: count,
                    thumbnail: first,
                });
            }
        } else if is_image(&path) {
            let mtime = metadata
                .modified()
                .ok()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            images_with_time.push((path.to_string_lossy().to_string(), mtime));
        }
    }

    images_with_time.sort_by(|a, b| b.1.cmp(&a.1));
    let images: Vec<ImageInfo> = images_with_time
        .into_iter()
        .map(|(path, _)| {
            let name = path.rsplit(['/', '\\']).next().unwrap_or(&path).to_string();
            ImageInfo { path, name }
        })
        .collect();

    let total = images.len();
    DirectoryImages {
        subdirs,
        images,
        total,
    }
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
        assert_eq!(
            sanitize_path("/normal/path/to/file.png"),
            "/normal/path/to/file.png"
        );
    }

    #[test]
    fn sanitize_path_handles_mixed_separators() {
        assert_eq!(
            sanitize_path("C:\\users\\test/path\\file.jpg"),
            "C:/users/test/path/file.jpg"
        );
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
            assert!(
                result.is_ok(),
                "Expected path '{path}' to be valid, got: {:?}",
                result
            );
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

    // -------------------------------------------------------------------------
    // Async directory scan tests
    // -------------------------------------------------------------------------

    fn create_test_tree(depth: usize, dirs_per_level: usize, images_per_dir: usize) -> tempfile::TempDir {
        use std::fs;
        let temp = tempfile::TempDir::new().unwrap();
        fn populate(dir: &std::path::Path, d: usize, max_d: usize, dirs: usize, imgs: usize) {
            if d > max_d { return; }
            for i in 0..imgs {
                let _ = fs::write(dir.join(format!("img_{}.jpg", i)), b"fake");
            }
            for i in 0..dirs {
                let subdir = dir.join(format!("subdir_{}", i));
                let _ = fs::create_dir_all(&subdir);
                populate(&subdir, d + 1, max_d, dirs, imgs);
            }
        }
        populate(temp.path(), 0, depth, dirs_per_level, images_per_dir);
        temp
    }

    #[tokio::test]
    async fn build_tree_async_returns_empty_for_nonexistent_dir() {
        let result = super::build_tree_async(std::path::PathBuf::from("/nonexistent/path"));
        assert!(result.await.is_empty());
    }

    #[tokio::test]
    async fn build_tree_async_counts_all_images_in_nested_dirs() {
        // depth=2 creates levels 0, 1, 2
        // level 0: 3 images; level 1: 2*3=6 images; level 2: 4*3=12 images
        // Total: 3 + 6 + 12 = 21 images
        let temp = create_test_tree(2, 2, 3);
        let result = super::build_tree_async(temp.path().to_path_buf()).await;
        // Root's image_count includes all levels
        assert_eq!(result[0].image_count, 21);
    }

    #[tokio::test]
    async fn build_tree_async_sets_is_leaf_correctly() {
        // depth=1: root with 2 subdirs (each has 1 image)
        // Root is not a leaf (has children); its children are leaves
        let temp = create_test_tree(1, 2, 1);
        let result = super::build_tree_async(temp.path().to_path_buf()).await;
        // Root has children, so it is not a leaf
        assert!(!result[0].is_leaf, "Root node should not be a leaf (has subdirs)");
        if let Some(children) = &result[0].children {
            for child in children {
                assert!(child.is_leaf, "Leaf dirs with no subdirs should have is_leaf=true");
            }
        }
    }

    #[tokio::test]
    async fn build_tree_async_image_count_sums_children_and_own() {
        // depth=1 creates root with 2 subdirs (subdir_0, subdir_1), each empty
        // Patch 2 images to root, and add images to subdirs
        let temp = create_test_tree(1, 2, 0);
        {
            use std::fs;
            // Add 2 images to root
            let _ = fs::write(temp.path().join("root_1.jpg"), b"fake");
            let _ = fs::write(temp.path().join("root_2.jpg"), b"fake");
            // Add 3 images to subdir_0
            let sub0 = temp.path().join("subdir_0");
            let _ = fs::write(sub0.join("a.jpg"), b"fake");
            let _ = fs::write(sub0.join("b.jpg"), b"fake");
            let _ = fs::write(sub0.join("c.jpg"), b"fake");
            // Add 1 image to subdir_1
            let sub1 = temp.path().join("subdir_1");
            let _ = fs::write(sub1.join("d.jpg"), b"fake");
        }
        let result: Vec<TreeNode> = super::build_tree_async(temp.path().to_path_buf()).await;
        // Root's image_count = own (2) + subdir_0 (3) + subdir_1 (1) = 6
        assert_eq!(result[0].image_count, 6);
    }

    #[tokio::test]
    async fn build_tree_async_sets_first_image_as_thumbnail() {
        let temp = create_test_tree(1, 1, 2);
        let result = super::build_tree_async(temp.path().to_path_buf()).await;
        let thumbnail = &result[0].thumbnail;
        assert!(thumbnail.is_some(), "Should have a thumbnail (first image path)");
        let thumb = thumbnail.as_ref().unwrap();
        assert!(thumb.ends_with(".jpg"), "Thumbnail should be a .jpg path");
    }

    #[tokio::test]
    async fn build_tree_async_deep_tree_no_deadlock() {
        // depth=20, 1 subdir per level, 1 image per dir = 21 total (levels 0..20)
        // This should complete within 30 seconds or it's a deadlock
        let temp = create_test_tree(20, 1, 1);
        let start = std::time::Instant::now();
        let result: Vec<TreeNode> = super::build_tree_async(temp.path().to_path_buf()).await;
        let elapsed = start.elapsed();
        let total: usize = result.iter().map(|n| n.image_count).sum();
        assert_eq!(total, 21, "Should have counted 21 images in 20-level deep tree");
        assert!(elapsed.as_secs() < 30, "Deep tree scan took >30s — possible deadlock");
    }

    #[tokio::test]
    async fn get_directory_images_async_returns_correct_subdirs_and_images() {
        use std::fs;
        let temp = tempfile::TempDir::new().unwrap();
        // Create subdirs with images
        let sub_a = temp.path().join("subdir_a");
        fs::create_dir_all(&sub_a).unwrap();
        fs::write(sub_a.join("a1.jpg"), b"fake").unwrap();
        fs::write(sub_a.join("a2.png"), b"fake").unwrap();
        // Create subdir with no images
        let sub_b = temp.path().join("subdir_b");
        fs::create_dir_all(&sub_b).unwrap();
        // Create direct images in root
        fs::write(temp.path().join("direct.jpg"), b"fake").unwrap();
        fs::write(temp.path().join("direct.gif"), b"fake").unwrap();

        let result = super::get_directory_images_async(temp.path().to_path_buf()).await;
        assert_eq!(result.total, 2, "Root should have 2 direct images");
        assert_eq!(result.subdirs.len(), 1, "Only subdir_a has images, subdir_b is excluded");
        assert_eq!(result.subdirs[0].image_count, 2, "subdir_a should have 2 images");
    }

    // is_image helper test
    #[test]
    fn is_image_supports_supported_extensions() {
        use super::is_image;
        use std::path::Path;
        for ext in &["jpg", "jpeg", "png", "gif", "webp"] {
            let p = Path::new("test.").join(format!("file.{}", ext));
            assert!(is_image(&p), "Extension '{}' should be recognized as image", ext);
        }
    }

    #[test]
    fn is_image_rejects_unsupported_extensions() {
        use super::is_image;
        use std::path::Path;
        for ext in &["txt", "pdf", "html", "exe"] {
            let p = Path::new("test.").join(format!("file.{}", ext));
            assert!(!is_image(&p), "Extension '{}' should NOT be recognized as image", ext);
        }
    }
}
