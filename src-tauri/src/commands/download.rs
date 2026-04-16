use crate::commands::favorite_tags::DbState;
use crate::commands::gallery::validate_path_within_base;
use crate::commands::gelbooru::HTTP_CLIENT;
use crate::db::DownloadTaskRecord;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::{mpsc, RwLock, Semaphore};

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

// Convert database record to DownloadTask
fn record_to_task(record: DownloadTaskRecord) -> DownloadTask {
    let status = match record.status.as_str() {
        "pending" => DownloadStatus::Pending,
        "downloading" => DownloadStatus::Downloading,
        "completed" => DownloadStatus::Completed,
        "failed" => DownloadStatus::Failed,
        "paused" => DownloadStatus::Paused,
        "cancelled" => DownloadStatus::Cancelled,
        _ => DownloadStatus::Pending,
    };

    DownloadTask {
        id: record.id as u32,
        post_id: record.post_id as u32,
        image_url: record.image_url,
        file_name: record.file_name,
        save_path: record.file_path,
        status,
        progress: record.progress as f32,
        downloaded_size: record.downloaded_size as u64,
        total_size: record.total_size as u64,
        error: record.error_message,
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

    pub async fn restore_tasks(&self, tasks: Vec<DownloadTask>) {
        let mut task_map = self.tasks.write().await;
        for task in tasks {
            task_map.insert(task.id, task);
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
    db: State<'_, DbState>,
    post_id: u32,
    image_url: String,
    file_name: String,
    save_path: String,
) -> Result<DownloadTask, String> {
    let id = TASK_ID_COUNTER.fetch_add(1, Ordering::SeqCst);

    let task = DownloadTask {
        id,
        post_id,
        image_url: image_url.clone(),
        file_name: file_name.clone(),
        save_path: save_path.clone(),
        status: DownloadStatus::Pending,
        progress: 0.0,
        downloaded_size: 0,
        total_size: 0,
        error: None,
    };

    // Persist to database
    if let Ok(database) = db.0.lock() {
        let record = DownloadTaskRecord {
            id: id as i64,
            post_id: post_id as i32,
            file_name,
            file_path: save_path.clone(),
            image_url,
            status: "pending".to_string(),
            progress: 0.0,
            downloaded_size: 0,
            total_size: 0,
            error_message: None,
        };
        if let Err(e) = database.save_download_task(&record) {
            eprintln!("Failed to persist download task: {}", e);
        }
    }

    DOWNLOAD_MANAGER.add_task(task.clone()).await;

    // 发送任务添加事件
    let _ = app.emit("download-task-added", &task);

    Ok(task)
}

#[tauri::command]
pub async fn start_download(
    app: AppHandle,
    _db: State<'_, DbState>,
    id: u32,
) -> Result<(), String> {
    let task = DOWNLOAD_MANAGER
        .get_task(id)
        .await
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
        DOWNLOAD_MANAGER
            .update_task_status(id, DownloadStatus::Downloading)
            .await;
        emit_progress(
            &app_clone,
            id,
            task_clone.post_id,
            "downloading",
            0.0,
            0,
            0,
            None,
        );
        persist_progress_async(&app_clone, id as i64, "downloading", 0.0, 0, 0).await;

        // 创建保存目录
        let save_path = PathBuf::from(&task_clone.save_path);
        if let Some(parent) = save_path.parent() {
            if !parent.exists() {
                if let Err(e) = fs::create_dir_all(parent) {
                    let err_msg = format!("Failed to create directory: {}", e);
                    DOWNLOAD_MANAGER.set_task_error(id, err_msg.clone()).await;
                    emit_progress(
                        &app_clone,
                        id,
                        task_clone.post_id,
                        "failed",
                        0.0,
                        0,
                        0,
                        Some(err_msg.clone()),
                    );
                    persist_error_async(&app_clone, id as i64, &err_msg).await;
                    DOWNLOAD_MANAGER.remove_cancel_token(id).await;
                    return;
                }
            }
        }

        // 开始下载
        let http_client = HTTP_CLIENT.read().await;
        let response = match http_client
            .download_image(&task_clone.image_url, "https://gelbooru.com/")
            .await
        {
            Ok(r) => r,
            Err(e) => {
                let err_msg = format!("Request failed: {}", e);
                DOWNLOAD_MANAGER.set_task_error(id, err_msg.clone()).await;
                emit_progress(
                    &app_clone,
                    id,
                    task_clone.post_id,
                    "failed",
                    0.0,
                    0,
                    0,
                    Some(err_msg.clone()),
                );
                persist_error_async(&app_clone, id as i64, &err_msg).await;
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
                emit_progress(
                    &app_clone,
                    id,
                    task_clone.post_id,
                    "failed",
                    0.0,
                    0,
                    0,
                    Some(err_msg.clone()),
                );
                persist_error_async(&app_clone, id as i64, &err_msg).await;
                DOWNLOAD_MANAGER.remove_cancel_token(id).await;
                return;
            }
        };

        use std::io::Write;

        let mut downloaded: u64 = 0;
        let mut stream = response.bytes_stream();
        use futures::StreamExt;

        while let Some(chunk_result) = stream.next().await {
            // 检查是否取消
            if cancel_rx.try_recv().is_ok() {
                // 暂停状态 - 保留临时文件
                DOWNLOAD_MANAGER
                    .update_task_status(id, DownloadStatus::Paused)
                    .await;
                emit_progress(
                    &app_clone,
                    id,
                    task_clone.post_id,
                    "paused",
                    if total_size > 0 {
                        (downloaded as f32 / total_size as f32) * 100.0
                    } else {
                        0.0
                    },
                    downloaded,
                    total_size,
                    None,
                );
                DOWNLOAD_MANAGER.remove_cancel_token(id).await;
                return;
            }

            match chunk_result {
                Ok(chunk) => {
                    if let Err(e) = file.write_all(&chunk) {
                        let err_msg = format!("Write failed: {}", e);
                        DOWNLOAD_MANAGER.set_task_error(id, err_msg.clone()).await;
                        emit_progress(
                            &app_clone,
                            id,
                            task_clone.post_id,
                            "failed",
                            0.0,
                            0,
                            0,
                            Some(err_msg.clone()),
                        );
                        persist_error_async(&app_clone, id as i64, &err_msg).await;
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

                    DOWNLOAD_MANAGER
                        .update_task_progress(id, progress, downloaded, total_size)
                        .await;

                    // 每 100KB 发送一次进度更新
                    if downloaded % (100 * 1024) < chunk.len() as u64 || downloaded == total_size {
                        emit_progress(
                            &app_clone,
                            id,
                            task_clone.post_id,
                            "downloading",
                            progress,
                            downloaded,
                            total_size,
                            None,
                        );
                        persist_progress_async(
                            &app_clone,
                            id as i64,
                            "downloading",
                            progress,
                            downloaded,
                            total_size,
                        )
                        .await;
                    }
                }
                Err(e) => {
                    let err_msg = format!("Stream error: {}", e);
                    DOWNLOAD_MANAGER.set_task_error(id, err_msg.clone()).await;
                    emit_progress(
                        &app_clone,
                        id,
                        task_clone.post_id,
                        "failed",
                        0.0,
                        0,
                        0,
                        Some(err_msg.clone()),
                    );
                    persist_error_async(&app_clone, id as i64, &err_msg).await;
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
            emit_progress(
                &app_clone,
                id,
                task_clone.post_id,
                "failed",
                0.0,
                0,
                0,
                Some(err_msg.clone()),
            );
            persist_error_async(&app_clone, id as i64, &err_msg).await;
            DOWNLOAD_MANAGER.remove_cancel_token(id).await;
            return;
        }

        // 更新状态为完成
        DOWNLOAD_MANAGER
            .update_task_status(id, DownloadStatus::Completed)
            .await;
        DOWNLOAD_MANAGER
            .update_task_progress(id, 100.0, downloaded, total_size)
            .await;
        emit_progress(
            &app_clone,
            id,
            task_clone.post_id,
            "completed",
            100.0,
            downloaded,
            total_size,
            None,
        );
        persist_progress_async(
            &app_clone,
            id as i64,
            "completed",
            100.0,
            downloaded,
            total_size,
        )
        .await;
        DOWNLOAD_MANAGER.remove_cancel_token(id).await;
    });

    Ok(())
}

#[tauri::command]
pub async fn pause_download(id: u32) -> Result<(), String> {
    let task = DOWNLOAD_MANAGER
        .get_task(id)
        .await
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
    db: State<'_, DbState>,
    id: u32,
) -> Result<(), String> {
    start_download(app, db, id).await
}

#[tauri::command]
pub async fn cancel_download(id: u32) -> Result<(), String> {
    DOWNLOAD_MANAGER.cancel_task(id).await;
    DOWNLOAD_MANAGER
        .update_task_status(id, DownloadStatus::Cancelled)
        .await;
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
pub async fn restore_download_tasks(db: State<'_, DbState>) -> Result<Vec<DownloadTask>, String> {
    // Get all tasks from database (drop lock before await)
    let records = {
        let database = db.0.lock().map_err(|e| e.to_string())?;
        database
            .get_all_download_tasks()
            .map_err(|e| format!("Failed to load download tasks: {}", e))?
    };

    if records.is_empty() {
        return Ok(Vec::new());
    }

    // Find the max ID and update the counter
    let max_id = records.iter().map(|r| r.id).max().unwrap_or(0) as u32;
    TASK_ID_COUNTER.store(max_id + 1, Ordering::SeqCst);

    // Convert records to tasks
    let tasks: Vec<DownloadTask> = records.into_iter().map(record_to_task).collect();

    // Restore tasks to manager
    DOWNLOAD_MANAGER.restore_tasks(tasks.clone()).await;

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

#[allow(clippy::too_many_arguments)]
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

// Async versions that use AppHandle (for use in spawned tasks)
async fn persist_progress_async(
    app: &AppHandle,
    id: i64,
    status: &str,
    progress: f32,
    downloaded_size: u64,
    total_size: u64,
) {
    let db = app.state::<DbState>();
    let database = db.0.lock().unwrap();
    let _ = database.update_download_task_progress(
        id,
        status,
        progress as f64,
        downloaded_size as i64,
        total_size as i64,
    );
}

async fn persist_error_async(app: &AppHandle, id: i64, error: &str) {
    let db = app.state::<DbState>();
    let database = db.0.lock().unwrap();
    let _ = database.update_download_task_error(id, error);
}

/// Describes the outcome of a single download attempt for test mocking.
#[cfg(test)]
#[derive(Debug, Clone)]
pub enum DownloadAttempt {
    /// Transport/network error (maps to reqwest Err).
    TransportError(String),
    /// HTTP response with the given status code.
    Response(u16),
}

/// Retry decision returned by the mock fetch closure.
#[cfg(test)]
pub type RetryFetchResult = Result<DownloadAttempt, String>;

#[cfg(test)]
const MAX_RETRIES: u32 = 3;
#[cfg(test)]
const BASE_DELAY_MS: u64 = 1000;

/// Wraps a download with exponential-backoff retry (up to MAX_RETRIES).
/// fetch: closure returning DownloadAttempt
/// cancel_rx: cancellation channel
#[cfg(test)]
pub async fn download_with_retry<F, Fut>(
    mut fetch: F,
    mut cancel_rx: mpsc::Receiver<()>,
) -> Result<DownloadAttempt, String>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = RetryFetchResult>,
{
    let mut attempt: u32 = 1;

    loop {
        // Race the download attempt against cancellation.
        let outcome = tokio::select! {
            result = fetch() => result,
            _ = cancel_rx.recv() => return Err("cancelled".to_string()),
        };

        // If sender was dropped (channel closed), treat as cancellation.
        if cancel_rx.is_closed() {
            return Err("cancelled".to_string());
        }

        match outcome {
            // Transport error — retry if attempts remain.
            Ok(DownloadAttempt::TransportError(msg)) => {
                if attempt >= MAX_RETRIES {
                    // Race return against cancellation — user may cancel even at the end.
                    tokio::select! {
                        _ = cancel_rx.recv() => return Err("cancelled".to_string()),
                        () = tokio::time::sleep(tokio::time::Duration::ZERO) => {
                            if cancel_rx.is_closed() {
                                return Err("cancelled".to_string());
                            }
                            return Err(format!("Request failed after {} attempt(s): {}", attempt, msg));
                        }
                    }
                }
                let delay = tokio::time::Duration::from_millis(
                    BASE_DELAY_MS * (2_u64.saturating_pow(attempt.saturating_sub(1))),
                );
                tokio::select! {
                    _ = tokio::time::sleep(delay) => {
                        attempt += 1;
                    }
                    _ = cancel_rx.recv() => return Err("cancelled".to_string()),
                }
                // Check if sender was dropped while sleeping (recv returns None, not Err).
                if cancel_rx.is_closed() {
                    return Err("cancelled".to_string());
                }
            }
            // HTTP response — check status.
            Ok(DownloadAttempt::Response(status_code)) => {
                let status = reqwest::StatusCode::from_u16(status_code)
                    .unwrap_or(reqwest::StatusCode::INTERNAL_SERVER_ERROR);
                if status.is_server_error() {
                    // 5xx — retryable.
                    if attempt >= MAX_RETRIES {
                        // Race return against cancellation.
                        tokio::select! {
                            _ = cancel_rx.recv() => return Err("cancelled".to_string()),
                            () = tokio::time::sleep(tokio::time::Duration::ZERO) => {
                                if cancel_rx.is_closed() {
                                    return Err("cancelled".to_string());
                                }
                                return Err(format!(
                                    "Server error {} after {} retries",
                                    status,
                                    MAX_RETRIES
                                ));
                            }
                        }
                    }
                    let delay = tokio::time::Duration::from_millis(
                        BASE_DELAY_MS * (2_u64.saturating_pow(attempt.saturating_sub(1))),
                    );
                    tokio::select! {
                        _ = tokio::time::sleep(delay) => {
                            attempt += 1;
                        }
                        _ = cancel_rx.recv() => return Err("cancelled".to_string()),
                    }
                    // Check if sender was dropped while sleeping.
                    if cancel_rx.is_closed() {
                        return Err("cancelled".to_string());
                    }
                } else {
                    // 2xx or 4xx — do not retry.
                    return Ok(DownloadAttempt::Response(status_code));
                }
            }
            // Logic error from test mock.
            Err(e) => return Err(e),
        }
    }
}

#[cfg(test)]
mod download_with_retry_tests {
    use super::*;

    /// Helper: makes a reqwest Response that implements bytes_stream().
    fn mock_attempt(status: u16) -> DownloadAttempt {
        DownloadAttempt::Response(status)
    }

    // -------------------------------------------------------------------------
    // Test 1: succeeds on first attempt (no retry needed)
    // -------------------------------------------------------------------------
    #[tokio::test]
    async fn succeeds_on_first_attempt() {
        let (_tx, rx) = mpsc::channel(1);
        let result = download_with_retry(|| async { Ok(mock_attempt(200)) }, rx).await;
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), DownloadAttempt::Response(200)));
    }

    // -------------------------------------------------------------------------
    // Test 2: retries on transport error, succeeds after retry
    // -------------------------------------------------------------------------
    #[tokio::test]
    async fn retries_on_transport_error_succeeds_on_retry() {
        tokio::time::pause();
        let (tx, rx) = mpsc::channel(1);

        let attempt_count = std::sync::atomic::AtomicU32::new(0);
        let attempt_count_clone = &attempt_count;

        let result = download_with_retry(
            || {
                let cnt = attempt_count_clone;
                let tx = tx.clone();
                async move {
                    cnt.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    // Fail first time, succeed second time.
                    if cnt.load(std::sync::atomic::Ordering::SeqCst) == 1 {
                        Ok(DownloadAttempt::TransportError("timeout".to_string()))
                    } else {
                        drop(tx); // Close channel to prevent hanging.
                        Ok(mock_attempt(200))
                    }
                }
            },
            rx,
        )
        .await;

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), DownloadAttempt::Response(200)));
        assert_eq!(attempt_count.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    // -------------------------------------------------------------------------
    // Test 3: does NOT retry on 4xx — returns immediately
    // -------------------------------------------------------------------------
    #[tokio::test]
    async fn does_not_retry_on_4xx() {
        tokio::time::pause();
        let attempt_count = std::sync::atomic::AtomicU32::new(0);
        let attempt_count_clone = &attempt_count;
        let (tx, rx) = mpsc::channel(1);

        let result = download_with_retry(
            || {
                let cnt = attempt_count_clone;
                async move {
                    cnt.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Ok(mock_attempt(404))
                }
            },
            rx,
        )
        .await;

        // Should return immediately without retry.
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), DownloadAttempt::Response(404)));
        assert_eq!(attempt_count.load(std::sync::atomic::Ordering::SeqCst), 1);
        drop(tx);
    }

    // -------------------------------------------------------------------------
    // Test 4: retries on 5xx, succeeds after retry
    // -------------------------------------------------------------------------
    #[tokio::test]
    async fn retries_on_5xx_succeeds_on_retry() {
        tokio::time::pause();
        let (tx, rx) = mpsc::channel(1);
        let attempt_count = std::sync::atomic::AtomicU32::new(0);
        let attempt_count_clone = &attempt_count;

        let result = download_with_retry(
            || {
                let cnt = attempt_count_clone;
                let tx = tx.clone();
                async move {
                    cnt.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if cnt.load(std::sync::atomic::Ordering::SeqCst) == 1 {
                        Ok(mock_attempt(502))
                    } else {
                        drop(tx);
                        Ok(mock_attempt(200))
                    }
                }
            },
            rx,
        )
        .await;

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), DownloadAttempt::Response(200)));
        assert_eq!(attempt_count.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    // -------------------------------------------------------------------------
    // Test 5: respects cancellation during retry sleep — exits without waiting
    // -------------------------------------------------------------------------
    #[tokio::test]
    async fn respects_cancellation_during_retry_sleep() {
        tokio::time::pause();

        // Use mpsc so we can .send() synchronously (not async like oneshot).
        let (cancel_tx, cancel_rx) = mpsc::channel::<()>(1);

        let attempt_count = std::sync::atomic::AtomicU32::new(0);
        let attempt_count_clone = &attempt_count;

        // Spawn a task that drops the sender after a virtual delay.
        // This simulates the user cancelling during the backoff sleep.
        tokio::spawn(async move {
            // We need the sender to be dropped DURING the download's first sleep (1000ms).
            // Calculate deadline = now + 800ms. At t=800, the download is still in
            // its first sleep (which started at t=0 and lasts until t=1000).
            let deadline = tokio::time::Instant::now() + tokio::time::Duration::from_millis(800);
            tokio::time::sleep_until(deadline).await;
            drop(cancel_tx);
        });

        let result = download_with_retry(
            || {
                let cnt = attempt_count_clone;
                async move {
                    cnt.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    // Always fail — cancellation should interrupt the backoff sleep.
                    Ok(DownloadAttempt::TransportError("timeout".to_string()))
                }
            },
            cancel_rx,
        )
        .await;

        // Should be cancelled, not error after waiting for full delay.
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "cancelled");
        // Only 1 attempt made — the second would happen after the sleep,
        // but cancellation arrived during the sleep and stopped the loop.
        assert_eq!(attempt_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    // -------------------------------------------------------------------------
    // Test 6: after MAX_RETRIES exhausted on retryable error, returns error
    // -------------------------------------------------------------------------
    #[tokio::test]
    async fn returns_error_after_max_retries_exhausted() {
        tokio::time::pause();
        let (tx, rx) = mpsc::channel(1);

        let result = download_with_retry(
            || async { Ok(DownloadAttempt::TransportError("persistent failure".to_string())) },
            rx,
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("persistent failure"));
        assert!(err.contains("3 attempt")); // 3 total attempts (1 initial + 2 retries).
        drop(tx);
    }

    // -------------------------------------------------------------------------
    // Test 7: backoff delays are 1s, 2s, 4s for attempts 1, 2, 3 respectively
    // -------------------------------------------------------------------------
    #[tokio::test]
    async fn backoff_delays_are_1s_2s_4s() {
        tokio::time::pause();
        let (tx, rx) = mpsc::channel(1);

        let attempt_count = std::sync::atomic::AtomicU32::new(0);
        let tx_clone = tx.clone();

        let start = tokio::time::Instant::now();

        let result = download_with_retry(
            || {
                let cnt = &attempt_count;
                let tx = tx_clone.clone();
                async move {
                    let n = cnt.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if n < 2 {
                        // Attempts 1 and 2 fail (n=0, n=1), attempt 3 (n=2) succeeds.
                        Ok(DownloadAttempt::TransportError("fail".to_string()))
                    } else {
                        drop(tx);
                        Ok(mock_attempt(200))
                    }
                }
            },
            rx,
        )
        .await;

        let elapsed = start.elapsed();

        assert!(result.is_ok());
        // After 3 attempts (1 initial + 2 retries), delays should be 1s + 2s = 3s total.
        assert!(
            elapsed >= tokio::time::Duration::from_secs(3),
            "Expected at least 3s elapsed (1s + 2s delays), got {:?}",
            elapsed
        );
        assert!(
            elapsed < tokio::time::Duration::from_secs(4),
            "Should not have waited for the 3rd delay (4s) since 3rd attempt succeeded"
        );
        drop(tx);
    }

    // -------------------------------------------------------------------------
    // Test 8: does not retry on 5xx after max retries — returns server error
    // -------------------------------------------------------------------------
    #[tokio::test]
    async fn returns_server_error_after_max_retries_on_5xx() {
        tokio::time::pause();
        let (tx, rx) = mpsc::channel(1);

        let result = download_with_retry(
            || async { Ok(mock_attempt(503)) },
            rx,
        )
        .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("503"));
        assert!(err.contains("retries"));
        drop(tx);
    }
}

#[tauri::command]
pub async fn open_file(db: State<'_, DbState>, path: String) -> Result<(), String> {
    // Get download directory from settings
    let download_dir =
        db.0.lock()
            .map_err(|e| e.to_string())?
            .get_setting("download_path")
            .map_err(|e| e.to_string())?
            .unwrap_or_default();

    // Validate path is within download directory
    let path = validate_path_within_base(&path, &download_dir)?;

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
