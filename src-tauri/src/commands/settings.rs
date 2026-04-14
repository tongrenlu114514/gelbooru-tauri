use crate::commands::favorite_tags::DbState;
use tauri::State;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: String,
    pub sidebar_collapsed: bool,
    pub download_path: String,
    pub concurrent_downloads: i32,
    pub proxy_enabled: bool,
    pub proxy_host: String,
    pub proxy_port: i32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            sidebar_collapsed: false,
            download_path: String::new(),
            concurrent_downloads: 3,
            proxy_enabled: true,
            proxy_host: "127.0.0.1".to_string(),
            proxy_port: 7897,
        }
    }
}

#[tauri::command]
pub fn get_settings(db: State<DbState>) -> Result<AppSettings, String> {
    let database = db.0.lock().map_err(|e| e.to_string())?;
    let settings_map = database.get_all_settings()
        .map_err(|e| format!("Failed to get settings: {}", e))?;

    let mut settings = AppSettings::default();
    for (key, value) in settings_map {
        match key.as_str() {
            "theme" => settings.theme = value,
            "sidebar_collapsed" => settings.sidebar_collapsed = value == "true",
            "download_path" => settings.download_path = value,
            "concurrent_downloads" => {
                settings.concurrent_downloads = value.parse().unwrap_or(3);
            }
            "proxy_enabled" => settings.proxy_enabled = value == "true",
            "proxy_host" => settings.proxy_host = value,
            "proxy_port" => {
                settings.proxy_port = value.parse().unwrap_or(7897);
            }
            _ => {}
        }
    }

    Ok(settings)
}

#[tauri::command]
pub fn save_settings(db: State<DbState>, settings: AppSettings) -> Result<(), String> {
    let database = db.0.lock().map_err(|e| e.to_string())?;

    let settings_map: HashMap<&str, String> = HashMap::from([
        ("theme", settings.theme),
        ("sidebar_collapsed", settings.sidebar_collapsed.to_string()),
        ("download_path", settings.download_path),
        ("concurrent_downloads", settings.concurrent_downloads.to_string()),
        ("proxy_enabled", settings.proxy_enabled.to_string()),
        ("proxy_host", settings.proxy_host),
        ("proxy_port", settings.proxy_port.to_string()),
    ]);

    for (key, value) in settings_map {
        database.set_setting(key, &value)
            .map_err(|e| format!("Failed to save setting {}: {}", key, e))?;
    }

    Ok(())
}
