use crate::db::{Database, FavoriteTag};
use std::sync::Mutex;
use tauri::State;

pub struct DbState(pub Mutex<Database>);

#[tauri::command]
pub fn get_favorite_tags(db: State<DbState>) -> Result<Vec<(FavoriteTag, Vec<FavoriteTag>)>, String> {
    let db = db.0.lock().map_err(|e| e.to_string())?;
    db.get_all_favorite_tags().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_parent_tag(db: State<DbState>, tag: String, tag_type: String) -> Result<i64, String> {
    let db = db.0.lock().map_err(|e| e.to_string())?;
    db.add_favorite_tag(&tag, &tag_type).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_child_tag(db: State<DbState>, tag: String, tag_type: String, parent_id: i64) -> Result<i64, String> {
    let db = db.0.lock().map_err(|e| e.to_string())?;
    db.add_favorite_tag_with_parent(&tag, &tag_type, parent_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_favorite_tag(db: State<DbState>, id: i64) -> Result<(), String> {
    let db = db.0.lock().map_err(|e| e.to_string())?;
    db.remove_favorite_tag(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn is_tag_favorited(db: State<DbState>, tag: String) -> Result<bool, String> {
    let db = db.0.lock().map_err(|e| e.to_string())?;
    Ok(db.is_tag_favorited(&tag))
}

#[tauri::command]
pub fn get_child_tags(db: State<DbState>, parent_id: i64) -> Result<Vec<FavoriteTag>, String> {
    let db = db.0.lock().map_err(|e| e.to_string())?;
    db.get_child_tags(parent_id).map_err(|e| e.to_string())
}
