// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod models;
mod services;
mod db;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            commands::gelbooru::search_posts,
            commands::gelbooru::get_post_detail,
            commands::gelbooru::get_image_base64,
            commands::download::start_download,
            commands::download::pause_download,
            commands::download::resume_download,
            commands::download::get_download_progress,
            commands::gallery::get_local_images,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
