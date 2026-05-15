use crate::commands::favorite_tags::DbState;
use image::{imageops::FilterType, GenericImageView, open};
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tokio::sync::mpsc;

// Re-export is_image from gallery.rs so thumbnail generation stays consistent
fn is_image(path: &std::path::Path) -> bool {
    crate::commands::gallery::is_image(path)
}

// Thumbnail size types
#[derive(Debug, Clone, Copy)]
pub enum ThumbnailSize {
    Card,      // 320x320 — gallery card
    Filmstrip, // 60x72  — filmstrip
}

impl ThumbnailSize {
    pub fn dimensions(&self) -> (u32, u32) {
        match self {
            ThumbnailSize::Card => (320, 320),
            ThumbnailSize::Filmstrip => (60, 72),
        }
    }

    pub fn size_name(&self) -> &'static str {
        match self {
            ThumbnailSize::Card => "card",
            ThumbnailSize::Filmstrip => "filmstrip",
        }
    }
}

// Scan result
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub new_images: usize,
    pub updated_images: usize,
    pub total_images: usize,
}

// Indexed image record returned to frontend
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexedImage {
    pub id: i64,
    pub file_path: String,
    pub file_name: String,
    pub file_size: i64,
    pub width: i64,
    pub height: i64,
    pub thumbnail_path: Option<String>,
    pub last_scanned: i64,
}

#[derive(Serialize)]
pub struct IndexedImagesResult {
    pub images: Vec<IndexedImage>,
    pub total: usize,
    pub has_more: bool,
}

// Thumbnail job for background queue
#[derive(Debug, Clone)]
pub struct ThumbnailJob {
    pub image_id: i64,
    pub source_path: String,
    pub dest_path: String,
    pub max_width: u32,
    pub max_height: u32,
}

const THUMBNAIL_QUEUE_SIZE: usize = 100;

/// Get the thumbnail cache directory within app data
fn get_thumbnail_cache_dir(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {}", e))?;
    let cache_dir = app_data_dir.join("thumbnails");
    std::fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Failed to create cache dir: {}", e))?;
    Ok(cache_dir)
}

/// Compute a simple hash of the source path for thumbnail filename
fn compute_thumb_filename(source_path: &str, size: &ThumbnailSize) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    source_path.hash(&mut hasher);
    size.size_name().hash(&mut hasher);
    format!("{}_{}.jpg", hasher.finish(), size.size_name())
}

/// Get the thumbnail path for a given source image and size
pub fn get_thumbnail_path_for_size(
    app_handle: &AppHandle,
    source_path: &str,
    size: ThumbnailSize,
) -> Result<PathBuf, String> {
    let cache_dir = get_thumbnail_cache_dir(app_handle)?;
    let filename = compute_thumb_filename(source_path, &size);
    Ok(cache_dir.join(filename))
}

/// Generate a thumbnail using the image crate
/// Returns (generated_width, generated_height) on success
fn generate_thumbnail_impl(
    source_path: &std::path::Path,
    dest_path: &std::path::Path,
    max_width: u32,
    max_height: u32,
) -> Result<(u32, u32), String> {
    let img = open(source_path)
        .map_err(|e| format!("Failed to open image: {}", e))?;

    let (orig_w, orig_h) = img.dimensions();
    let ratio = (max_width as f64 / orig_w as f64).min(max_height as f64 / orig_h as f64);
    let new_w = (orig_w as f64 * ratio).round() as u32;
    let new_h = (orig_h as f64 * ratio).round() as u32;

    let resized = img.resize(new_w, new_h, FilterType::Lanczos3);
    resized
        .save_with_format(dest_path, image::ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to save thumbnail: {}", e))?;

    Ok((new_w, new_h))
}

/// Indexing service: manages background thumbnail generation queue
pub struct IndexingService {
    thumbnail_tx: mpsc::Sender<ThumbnailJob>,
}

impl IndexingService {
    pub fn new(db: Arc<crate::db::Database>, _app_handle: AppHandle) -> Self {
        let (tx, mut rx) = mpsc::channel::<ThumbnailJob>(THUMBNAIL_QUEUE_SIZE);
        let db_clone = Arc::clone(&db);

        tokio::spawn(async move {
            while let Some(job) = rx.recv().await {
                // Clone all paths BEFORE spawn_blocking so they survive the await
                let source = job.source_path.clone();
                let dest = job.dest_path.clone();
                let job_id = job.image_id;
                let max_w = job.max_width;
                let max_h = job.max_height;

                // Clone again for use after the await (strings moved into closure)
                let source_for_log = source.clone();
                let dest_for_db = dest.clone();

                let result = tokio::task::spawn_blocking(move || {
                    generate_thumbnail_impl(
                        std::path::Path::new(&source),
                        std::path::Path::new(&dest),
                        max_w,
                        max_h,
                    )
                })
                .await;

                match result {
                    Ok(Ok((w, h))) => {
                        // Save thumbnail entry to DB
                        if let Err(e) = db_clone.save_thumbnail_entry(job_id, &dest_for_db, w as i64, h as i64) {
                            eprintln!("[indexing] Failed to save thumbnail entry: {}", e);
                        }
                        // Update gallery_images.thumbnail_path with primary thumbnail (card size)
                        if max_w == 320 && max_h == 320 {
                            if let Err(e) = db_clone.update_gallery_thumbnail_path(job_id, &dest_for_db) {
                                eprintln!("[indexing] Failed to update thumbnail_path: {}", e);
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        eprintln!("[indexing] Thumbnail generation failed for {}: {}", source_for_log, e);
                    }
                    Err(e) => {
                        eprintln!("[indexing] Task join error: {}", e);
                    }
                }
            }
        });

        Self { thumbnail_tx: tx }
    }

    /// Queue a thumbnail for background generation (non-blocking, drops if queue full)
    pub fn queue_thumbnail(&self, job: ThumbnailJob) {
        let _ = self.thumbnail_tx.try_send(job);
    }
}

// ──────────────────────────────────────────────────────────────
// Tauri Commands
// ──────────────────────────────────────────────────────────────

/// Scan a directory recursively, indexing all images into SQLite.
/// Returns scan summary. Starts background thumbnail generation for new images.
#[tauri::command]
pub async fn scan_gallery(
    db: State<'_, DbState>,
    app_handle: AppHandle,
    root_path: String,
) -> Result<ScanResult, String> {
    use std::time::UNIX_EPOCH;

    let root = PathBuf::from(&root_path);
    if !root.exists() {
        return Err("Directory does not exist".to_string());
    }

    let db_inner = db.0.lock().map_err(|e| e.to_string())?;

    let mut new_count = 0;
    let mut updated_count = 0;
    let mut total_count = 0;

    let walker = walkdir::WalkDir::new(&root)
        .follow_links(false)
        .max_depth(usize::MAX)
        .into_iter()
        .filter_map(|e| e.ok());

    for entry in walker {
        let path = entry.path();
        if !path.is_file() || !is_image(path) {
            continue;
        }

        total_count += 1;
        let file_path = path.to_string_lossy().replace('\\', "/");
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // Get metadata
        let metadata = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(_) => continue,
        };
        let file_size = metadata.len() as i64;

        let mtime = metadata
            .modified()
            .unwrap_or(UNIX_EPOCH)
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Check existing entry
        let existing = db_inner.get_gallery_image_by_path(&file_path).ok().flatten();

        let (id, _is_new) = if let Some((existing_id, existing_mtime, _)) = existing {
            if existing_mtime < mtime {
                // Updated — re-index
                db_inner
                    .upsert_gallery_image(
                        &file_path,
                        &file_name,
                        file_size,
                        0,
                        0,
                        None,
                        None,
                        mtime,
                    )
                    .ok();
                updated_count += 1;
            }
            (existing_id, false)
        } else {
            // New image
            let id = db_inner
                .upsert_gallery_image(
                    &file_path,
                    &file_name,
                    file_size,
                    0,
                    0,
                    None,
                    None,
                    mtime,
                )
                .unwrap_or(0);
            new_count += 1;
            (id, true)
        };

        // Queue background thumbnail generation for all images
        if let Some(indexing_svc) = app_handle.try_state::<IndexingService>() {
            // Queue card thumbnail (320x320)
            if let Ok(thumb_path) =
                get_thumbnail_path_for_size(&app_handle, &file_path, ThumbnailSize::Card)
            {
                indexing_svc.queue_thumbnail(ThumbnailJob {
                    image_id: id,
                    source_path: file_path.clone(),
                    dest_path: thumb_path.to_string_lossy().to_string(),
                    max_width: 320,
                    max_height: 320,
                });
            }
            // Queue filmstrip thumbnail (60x72)
            if let Ok(thumb_path) =
                get_thumbnail_path_for_size(&app_handle, &file_path, ThumbnailSize::Filmstrip)
            {
                indexing_svc.queue_thumbnail(ThumbnailJob {
                    image_id: id,
                    source_path: file_path.clone(),
                    dest_path: thumb_path.to_string_lossy().to_string(),
                    max_width: 60,
                    max_height: 72,
                });
            }
        }
    }

    drop(db_inner);

    Ok(ScanResult {
        new_images: new_count,
        updated_images: updated_count,
        total_images: total_count,
    })
}

/// Get indexed images for a folder, with pagination.
#[tauri::command]
pub async fn get_indexed_images(
    db: State<'_, DbState>,
    folder_path: Option<String>,
    page: Option<usize>,
    page_size: Option<usize>,
) -> Result<IndexedImagesResult, String> {
    let page = page.unwrap_or(1);
    let page_size = page_size.unwrap_or(50);

    let db_inner = db.0.lock().map_err(|e| e.to_string())?;

    let folder = folder_path.unwrap_or_default();
    let offset = (page.saturating_sub(1)) * page_size;

    let (rows, total) = db_inner
        .get_indexed_images(&folder, offset, page_size)
        .map_err(|e| e.to_string())?;

    let images: Vec<IndexedImage> = rows
        .into_iter()
        .map(
            |(id, file_path, file_name, file_size, width, height, thumbnail_path, last_scanned)| {
                IndexedImage {
                    id,
                    file_path,
                    file_name,
                    file_size,
                    width,
                    height,
                    thumbnail_path,
                    last_scanned,
                }
            },
        )
        .collect();

    let has_more = offset + page_size < total;

    Ok(IndexedImagesResult {
        images,
        total,
        has_more,
    })
}

/// Generate a thumbnail on-demand for an uncached image.
/// Returns the thumbnail file path. Also queues background generation for the other size.
#[tauri::command]
pub async fn generate_thumbnail(
    db: State<'_, DbState>,
    app_handle: AppHandle,
    image_path: String,
    max_width: Option<u32>,
    max_height: Option<u32>,
) -> Result<String, String> {
    // Extract all data BEFORE dropping the lock — compiler needs this to verify
    // no MutexGuard crosses the await boundary
    let (image_id, dest_path, image_path_clone) = {
        let db_inner = db.0.lock().map_err(|e| e.to_string())?;
        let (id, _, existing_thumb) = db_inner
            .get_gallery_image_by_path(&image_path)
            .map_err(|e| e.to_string())?
            .unwrap_or((0, 0i64, None));

        let max_w = max_width.unwrap_or(320);
        let max_h = max_height.unwrap_or(320);
        let size_key = if max_w == 320 && max_h == 320 {
            ThumbnailSize::Card
        } else {
            ThumbnailSize::Filmstrip
        };

        if let Some(ref thumb_path) = existing_thumb {
            let p = std::path::Path::new(thumb_path);
            if p.exists() {
                return Ok(thumb_path.clone());
            }
        }

        let dest_path = get_thumbnail_path_for_size(&app_handle, &image_path, size_key)?;
        (id, dest_path, image_path.clone())
    };

    // On-demand generate
    let dest_path_str = dest_path.to_string_lossy().to_string();
    let result = tokio::task::spawn_blocking(move || {
        generate_thumbnail_impl(
            std::path::Path::new(&image_path_clone),
            std::path::Path::new(&dest_path_str),
            max_width.unwrap_or(320),
            max_height.unwrap_or(320),
        )
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;

    // Save to DB
    if image_id > 0 {
        let db_inner = db.0.lock().map_err(|e| e.to_string())?;
        let dest_str = dest_path.to_string_lossy();
        let _ = db_inner.save_thumbnail_entry(image_id, dest_str.as_ref(), result.0 as i64, result.1 as i64);
        let _ = db_inner.update_gallery_thumbnail_path(image_id, dest_str.as_ref());
    }

    // Queue the other size for background pre-generation
    if let Some(indexing_svc) = app_handle.try_state::<IndexingService>() {
        let other_size = if max_width.unwrap_or(320) == 320 && max_height.unwrap_or(320) == 320 {
            ThumbnailSize::Filmstrip
        } else {
            ThumbnailSize::Card
        };
        if let Ok(other_path) = get_thumbnail_path_for_size(&app_handle, &image_path, other_size) {
            indexing_svc.queue_thumbnail(ThumbnailJob {
                image_id,
                source_path: image_path,
                dest_path: other_path.to_string_lossy().to_string(),
                max_width: other_size.dimensions().0,
                max_height: other_size.dimensions().1,
            });
        }
    }

    Ok(dest_path.to_string_lossy().to_string())
}

/// Get thumbnail path for an image (generates on-demand if missing).
#[tauri::command]
pub async fn get_thumbnail_path(
    db: State<'_, DbState>,
    app_handle: AppHandle,
    image_path: String,
    size_type: String,
) -> Result<String, String> {
    let size = match size_type.as_str() {
        "filmstrip" => ThumbnailSize::Filmstrip,
        _ => ThumbnailSize::Card,
    };
    let (max_w, max_h) = size.dimensions();
    generate_thumbnail(db, app_handle, image_path, Some(max_w), Some(max_h)).await
}

/// Start background thumbnail scan for all indexed images missing thumbnails.
/// Queues work for the background service without blocking.
#[tauri::command]
pub async fn start_background_thumbnail_scan(
    db: State<'_, DbState>,
    app_handle: AppHandle,
) -> Result<(), String> {
    let db_inner = db.0.lock().map_err(|e| e.to_string())?;
    let rows = db_inner.get_images_without_thumbnails().map_err(|e| e.to_string())?;
    drop(db_inner);

    if let Some(indexing_svc) = app_handle.try_state::<IndexingService>() {
        for (image_id, file_path) in rows {
            // Queue card size
            if let Ok(dest_path) =
                get_thumbnail_path_for_size(&app_handle, &file_path, ThumbnailSize::Card)
            {
                indexing_svc.queue_thumbnail(ThumbnailJob {
                    image_id,
                    source_path: file_path.clone(),
                    dest_path: dest_path.to_string_lossy().to_string(),
                    max_width: 320,
                    max_height: 320,
                });
            }
            // Queue filmstrip size
            if let Ok(dest_path) =
                get_thumbnail_path_for_size(&app_handle, &file_path, ThumbnailSize::Filmstrip)
            {
                indexing_svc.queue_thumbnail(ThumbnailJob {
                    image_id,
                    source_path: file_path,
                    dest_path: dest_path.to_string_lossy().to_string(),
                    max_width: 60,
                    max_height: 72,
                });
            }
        }
    }

    Ok(())
}

// ──────────────────────────────────────────────────────────────
// App setup: register IndexingService on app startup
// ──────────────────────────────────────────────────────────────

/// Register the IndexingService with the Tauri app state.
/// Call this once during app startup (e.g., in main.rs setup).
pub fn setup_indexing_service(app_handle: &AppHandle, db: Arc<crate::db::Database>) {
    let service = IndexingService::new(db, app_handle.clone());
    app_handle.manage(service);
}