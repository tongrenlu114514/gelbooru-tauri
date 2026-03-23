// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod models;
mod services;
mod db;

use std::sync::Mutex;
use db::Database;
use commands::favorite_tags::DbState;

fn main() {
    // 初始化数据库
    let app_data_dir = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());
    
    let database = Database::new(&app_data_dir)
        .expect("Failed to initialize database");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .manage(DbState(Mutex::new(database)))
        .invoke_handler(tauri::generate_handler![
            commands::gelbooru::search_posts,
            commands::gelbooru::get_post_detail,
            commands::gelbooru::get_image_base64,
            commands::gelbooru::set_proxy,
            commands::download::add_download_task,
            commands::download::start_download,
            commands::download::pause_download,
            commands::download::resume_download,
            commands::download::cancel_download,
            commands::download::remove_download_task,
            commands::download::get_download_tasks,
            commands::download::clear_completed_tasks,
            commands::download::open_file,
            commands::gallery::get_local_images,
            commands::gallery::delete_image,
            commands::gallery::get_directory_tree,
            commands::gallery::get_directory_images,
            commands::favorite_tags::get_favorite_tags,
            commands::favorite_tags::add_parent_tag,
            commands::favorite_tags::add_child_tag,
            commands::favorite_tags::remove_favorite_tag,
            commands::favorite_tags::is_tag_favorited,
            commands::favorite_tags::get_child_tags,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
