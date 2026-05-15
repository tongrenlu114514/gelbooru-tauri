// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod models;
mod services;
mod db;

use std::sync::{Arc, Mutex};
use db::Database;
use commands::favorite_tags::DbState;

fn main() {
    // 初始化数据库
    let app_data_dir = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());

    let database = Arc::new(
        Database::new(&app_data_dir).expect("Failed to initialize database")
    );

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .manage(DbState(Mutex::new(Arc::clone(&database))))
        .setup(move |app| {
            use commands::indexing::setup_indexing_service;
            if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                setup_indexing_service(&app.handle().clone(), database.clone());
            })) {
                eprintln!("[main] IndexingService setup failed (graceful degrade): {:?}", e);
            }
            Ok(())
        })
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
            commands::download::restore_download_tasks,
            commands::download::clear_completed_tasks,
            commands::download::open_file,
            commands::gallery::get_local_images,
            commands::gallery::delete_image,
            commands::gallery::get_directory_tree,
            commands::gallery::get_directory_images,
            commands::gallery::get_local_image_base64,
            commands::favorite_tags::get_favorite_tags,
            commands::favorite_tags::add_parent_tag,
            commands::favorite_tags::add_child_tag,
            commands::favorite_tags::remove_favorite_tag,
            commands::favorite_tags::is_tag_favorited,
            commands::favorite_tags::get_child_tags,
            // indexing (Phase 11)
            commands::indexing::scan_gallery,
            commands::indexing::get_indexed_images,
            commands::indexing::generate_thumbnail,
            commands::indexing::get_thumbnail_path,
            commands::indexing::start_background_thumbnail_scan,
            commands::settings::get_settings,
            commands::settings::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}