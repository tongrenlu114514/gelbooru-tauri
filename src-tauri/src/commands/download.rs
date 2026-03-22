use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::{RwLock, mpsc, Semaphore};
use tauri::{AppHandle, Emitter, Manager};
use serde::{Serialize, Deserialize};

lazy_static::lazy_static! {
    static ref DOWNLOAD_MANAGER: Arc<DownloadManager> = Arc::new(DownloadManager::new());
    static ref TASK_ID_COUNTER: AtomicU32 = AtomicU32::new(1);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadTask {
    pub id: u32,
    pub post_id: u32,
    pub image_url: String,
    pub file_name: String,
    pub save_path: String,
    pub status: DownloadStatus,
    pub progress: f32,
    pub downloaded_size: u64,
    pub total_size: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
    Paused,
    Cancelled,
}

impl DownloadStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DownloadStatus::Pending => "pending",
            DownloadStatus::Downloading => "downloading",
            DownloadStatus::Completed => "completed",
            DownloadStatus::Failed => "failed",
            DownloadStatus::Paused => "paused",
            DownloadStatus::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgressEvent {
    pub id: u32,
    pub post_id: u32,
    pub status: String,
    pub progress: f32,
    pub downloaded_size: u64,
    pub total_size: u64,
    pub error: Option<String>,
}

pub struct DownloadManager {
    tasks: RwLock<HashMap<u32, DownloadTask>>,
    semaphore: Semaphore,
    cancel_tokens: RwLock<HashMap<u32, mpsc::Sender<()>>>,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            tasks: RwLock::new(HashMap::new()),
            semaphore: Semaphore::new(3), // 默认 3 个并发
            cancel_tokens: RwLock::new(HashMap::new()),
        }
    }

    pub async fn add_task(&self, task: DownloadTask) {
        let mut tasks = self.tasks.write().await;
        tasks.insert(task.id, task);
    }

    pub async fn get_task(&self, id: u32) -> Option<DownloadTask> {
        let tasks = self.tasks.read().await;
        tasks.get(&id).cloned()
    }

    pub async fn get_all_tasks(&self) -> Vec<DownloadTask> {
        let tasks = self.tasks.read().await;
        tasks.values().cloned().collect()
    }

    pub async fn update_task_status(&self, id: u32, status: DownloadStatus) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(&id) {
            task.status = status;
        }
    }

    pub async fn update_task_progress(&self, id: u32, progress: f32, downloaded: u64, total: u64) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(&id) {
            task.progress = progress;
            task.downloaded_size = downloaded;
            task.total_size = total;
        }
    }

    pub async fn set_task_error(&self, id: u32, error: String) {
        let mut tasks = self.tasks.write().await;
        if let Some(task) = tasks.get_mut(&id) {
            task.error = Some(error);
            task.status = DownloadStatus::Failed;
        }
    }

    pub async fn add_cancel_token(&self, id: u32, sender: mpsc::Sender<()>) {
        let mut tokens = self.cancel_tokens.write().await;
        tokens.insert(id, sender);
    }

    pub async fn remove_cancel_token(&self, id: u32) {
        let mut tokens = self.cancel_tokens.write().await;
        tokens.remove(&id);
    }

    pub async fn cancel_task(&self, id: u32) -> bool {
        let tokens = self.cancel_tokens.read().await;
        if let Some(sender) = tokens.get(&id) {
            let _ = sender.send(()).await;
            true
        } else {
            false
        }
    }

    pub async fn remove_task(&self, id: u32) {
        // 先取消任务
        self.cancel_task(id).await;
        
        let mut tasks = self.tasks.write().await;
        tasks.remove(&id);
        
        let mut tokens = self.cancel_tokens.write().await;
        tokens.remove(&id);
    }
}

#[tauri::command]
pub async fn add_download_task(
    app: AppHandle,
    post_id: u32,
    image_url: String,
    file_name: String,
    save_path: String,
) -> Result<DownloadTask, String> {
    let id = TASK_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    
    let task = DownloadTask {
        id,
        post_id,
        image_url,
        file_name,
        save_path: save_path.clone(),
        status: DownloadStatus::Pending,
        progress: 0.0,
        downloaded_size: 0,
        total_size: 0,
        error: None,
    };
    
    DOWNLOAD_MANAGER.add_task(task.clone()).await;
    
    // 发送任务添加事件
    let _ = app.emit("download-task-added", &task);
    
    Ok(task)
}

#[tauri::command]
pub async fn start_download(
    app: AppHandle,
    id: u32,
) -> Result<(), String> {
    let task = DOWNLOAD_MANAGER.get_task(id).await
        .ok_or("Task not found")?;
    
    if task.status != DownloadStatus::Pending && task.status != DownloadStatus::Paused {
        return Err("Task is not in pending or paused state".to_string());
    }
    
    let app_clone = app.clone();
    let task_clone = task.clone();
    
    tokio::spawn(async move {
        // 获取信号量许可（控制并发）
        let _permit = DOWNLOAD_MANAGER.semaphore.acquire().await.unwrap();
        
        // 创建取消令牌
        let (cancel_tx, mut cancel_rx) = mpsc::channel::<()>(1);
        DOWNLOAD_MANAGER.add_cancel_token(id, cancel_tx).await;
        
        // 更新状态为下载中
        DOWNLOAD_MANAGER.update_task_status(id, DownloadStatus::Downloading).await;
        emit_progress(&app_clone, id, task_clone.post_id, "downloading", 0.0, 0, 0, None);
        
        // 创建保存目录
        let save_path = PathBuf::from(&task_clone.save_path);
        if let Some(parent) = save_path.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    let err_msg = format!("Failed to create directory: {}", e);
                    DOWNLOAD_MANAGER.set_task_error(id, err_msg.clone()).await;
                    emit_progress(&app_clone, id, task_clone.post_id, "failed", 0.0, 0, 0, Some(err_msg));
                    DOWNLOAD_MANAGER.remove_cancel_token(id).await;
                    return;
                }
            }
        }
        
        // 开始下载
        let client = reqwest::Client::builder()
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .gzip(true)
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .unwrap();
        
        let response = match client
            .get(&task_clone.image_url)
            .header("Referer", "https://gelbooru.com/")
            .header("Accept", "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8")
            .send()
            .await 
        {
            Ok(r) => r,
            Err(e) => {
                let err_msg = format!("Request failed: {}", e);
                DOWNLOAD_MANAGER.set_task_error(id, err_msg.clone()).await;
                emit_progress(&app_clone, id, task_clone.post_id, "failed", 0.0, 0, 0, Some(err_msg));
                DOWNLOAD_MANAGER.remove_cancel_token(id).await;
                return;
            }
        };
        
        let total_size = response.content_length().unwrap_or(0);
        
        // 创建临时文件
        let temp_path = save_path.with_extension("tmp");
        let mut file = match fs::File::create(&temp_path) {
            Ok(f) => f,
            Err(e) => {
                let err_msg = format!("Failed to create file: {}", e);
                DOWNLOAD_MANAGER.set_task_error(id, err_msg.clone()).await;
                emit_progress(&app_clone, id, task_clone.post_id, "failed", 0.0, 0, 0, Some(err_msg));
                DOWNLOAD_MANAGER.remove_cancel_token(id).await;
                return;
            }
        };
        
        use std::io::Write;
        use tokio::io::AsyncReadExt;
        
        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();
        use futures::StreamExt;
        
        while let Some(chunk_result) = stream.next().await {
            // 检查是否取消
            if cancel_rx.try_recv().is_ok() {
                // 暂停状态 - 保留临时文件
                DOWNLOAD_MANAGER.update_task_status(id, DownloadStatus::Paused).await;
                emit_progress(&app_clone, id, task_clone.post_id, "paused", 
                    if total_size > 0 { (downloaded as f32 / total_size as f32) * 100.0 } else { 0.0 },
                    downloaded, total_size, None);
                DOWNLOAD_MANAGER.remove_cancel_token(id).await;
                return;
            }
            
            match chunk_result {
                Ok(chunk) => {
                    if let Err(e) = file.write_all(&chunk) {
                        let err_msg = format!("Write failed: {}", e);
                        DOWNLOAD_MANAGER.set_task_error(id, err_msg.clone()).await;
                        emit_progress(&app_clone, id, task_clone.post_id, "failed", 0.0, 0, 0, Some(err_msg));
                        DOWNLOAD_MANAGER.remove_cancel_token(id).await;
                        let _ = fs::remove_file(&temp_path);
                        return;
                    }
                    
                    downloaded += chunk.len() as u64;
                    let progress = if total_size > 0 {
                        (downloaded as f32 / total_size as f32) * 100.0
                    } else {
                        0.0
                    };
                    
                    DOWNLOAD_MANAGER.update_task_progress(id, progress, downloaded, total_size).await;
                    
                    // 每 100KB 发送一次进度更新
                    if downloaded % (100 * 1024) < chunk.len() as u64 || downloaded == total_size {
                        emit_progress(&app_clone, id, task_clone.post_id, "downloading", 
                            progress, downloaded, total_size, None);
                    }
                }
                Err(e) => {
                    let err_msg = format!("Stream error: {}", e);
                    DOWNLOAD_MANAGER.set_task_error(id, err_msg.clone()).await;
                    emit_progress(&app_clone, id, task_clone.post_id, "failed", 0.0, 0, 0, Some(err_msg));
                    DOWNLOAD_MANAGER.remove_cancel_token(id).await;
                    let _ = fs::remove_file(&temp_path);
                    return;
                }
            }
        }
        
        // 下载完成，重命名临时文件
        if let Err(e) = fs::rename(&temp_path, &save_path) {
            let err_msg = format!("Rename failed: {}", e);
            DOWNLOAD_MANAGER.set_task_error(id, err_msg.clone()).await;
            emit_progress(&app_clone, id, task_clone.post_id, "failed", 0.0, 0, 0, Some(err_msg));
            DOWNLOAD_MANAGER.remove_cancel_token(id).await;
            return;
        }
        
        // 更新状态为完成
        DOWNLOAD_MANAGER.update_task_status(id, DownloadStatus::Completed).await;
        DOWNLOAD_MANAGER.update_task_progress(id, 100.0, downloaded, total_size).await;
        emit_progress(&app_clone, id, task_clone.post_id, "completed", 100.0, downloaded, total_size, None);
        DOWNLOAD_MANAGER.remove_cancel_token(id).await;
    });
    
    Ok(())
}

#[tauri::command]
pub async fn pause_download(id: u32) -> Result<(), String> {
    let task = DOWNLOAD_MANAGER.get_task(id).await
        .ok_or("Task not found")?;
    
    if task.status != DownloadStatus::Downloading {
        return Err("Task is not downloading".to_string());
    }
    
    DOWNLOAD_MANAGER.cancel_task(id).await;
    Ok(())
}

#[tauri::command]
pub async fn resume_download(
    app: AppHandle,
    id: u32,
) -> Result<(), String> {
    start_download(app, id).await
}

#[tauri::command]
pub async fn cancel_download(id: u32) -> Result<(), String> {
    DOWNLOAD_MANAGER.cancel_task(id).await;
    DOWNLOAD_MANAGER.update_task_status(id, DownloadStatus::Cancelled).await;
    Ok(())
}

#[tauri::command]
pub async fn remove_download_task(id: u32) -> Result<(), String> {
    DOWNLOAD_MANAGER.remove_task(id).await;
    Ok(())
}

#[tauri::command]
pub async fn get_download_tasks() -> Result<Vec<DownloadTask>, String> {
    let tasks = DOWNLOAD_MANAGER.get_all_tasks().await;
    Ok(tasks)
}

#[tauri::command]
pub async fn clear_completed_tasks() -> Result<(), String> {
    let tasks = DOWNLOAD_MANAGER.get_all_tasks().await;
    for task in tasks {
        if task.status == DownloadStatus::Completed {
            DOWNLOAD_MANAGER.remove_task(task.id).await;
        }
    }
    Ok(())
}

fn emit_progress(
    app: &AppHandle,
    id: u32,
    post_id: u32,
    status: &str,
    progress: f32,
    downloaded_size: u64,
    total_size: u64,
    error: Option<String>,
) {
    let event = DownloadProgressEvent {
        id,
        post_id,
        status: status.to_string(),
        progress,
        downloaded_size,
        total_size,
        error,
    };
    let _ = app.emit("download-progress", &event);
}

#[tauri::command]
pub async fn open_file(path: String) -> Result<(), String> {
    let path = PathBuf::from(&path);
    
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }
    
    // 使用系统默认程序打开文件
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path.to_string_lossy()])
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| format!("Failed to open file: {}", e))?;
    }
    
    Ok(())
}