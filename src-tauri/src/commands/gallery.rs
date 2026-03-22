use std::fs;
use std::path::PathBuf;

#[tauri::command]
pub fn get_local_images(folder_path: Option<String>) -> Result<Vec<String>, String> {
    let default_path = "D:/project/gelbooru/imgs/";
    let path = folder_path.as_deref().unwrap_or(default_path);
    
    let mut images = Vec::new();
    let path = PathBuf::from(path);
    
    if !path.exists() {
        return Ok(images);
    }
    
    fn collect_images(dir: &PathBuf, images: &mut Vec<String>) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    collect_images(&path, images);
                } else if let Some(ext) = path.extension() {
                    let ext = ext.to_string_lossy().to_lowercase();
                    if ["jpg", "jpeg", "png", "gif", "webp"].contains(&ext.as_str()) {
                        images.push(path.to_string_lossy().to_string());
                    }
                }
            }
        }
    }
    
    collect_images(&path, &mut images);
    
    // Sort by modification time (newest first)
    images.sort_by(|a, b| {
        let meta_a = fs::metadata(a);
        let meta_b = fs::metadata(b);
        match (meta_a, meta_b) {
            (Ok(ma), Ok(mb)) => {
                let time_a = ma.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                let time_b = mb.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                time_b.cmp(&time_a)
            }
            _ => std::cmp::Ordering::Equal,
        }
    });
    
    Ok(images)
}
