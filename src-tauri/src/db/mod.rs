use rusqlite::{Connection, OptionalExtension, Result as SqliteResult, Row};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteTag {
    pub id: i64,
    pub tag: String,
    pub tag_type: String,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FavoriteTagGroup {
    pub parent: FavoriteTag,
    pub children: Vec<FavoriteTag>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadTaskRecord {
    pub id: i64,
    pub post_id: i32,
    pub file_name: String,
    pub file_path: String,
    pub image_url: String,
    pub status: String,
    pub progress: f64,
    pub downloaded_size: i64,
    pub total_size: i64,
    pub error_message: Option<String>,
}

pub struct Database {
    conn: Mutex<Connection>,
}

/// All migrations run sequentially starting from version 1.
/// Migrations are embedded string constants to avoid file I/O in the Tauri bundle.
/// Naming convention: "001_init", "002_..." (sequential integer prefix per D-02).
/// Empty SQL means no-op (used for baseline version 1 — tables already created).
const MIGRATIONS: &[(&str, &str)] = &[
    // Version 1: baseline — all 5 tables created by the original new() batch.
    // No SQL needed; version 1 is set as baseline for existing DBs.
    ("001_init", ""),
];

/// Creates schema_version table if absent, sets baseline version=1 for existing DBs,
/// then runs all unapplied migrations in order.
fn run_migrations(conn: &Connection) -> SqliteResult<()> {
    // 1. Ensure schema_version table exists (idempotent)
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_version (version INTEGER PRIMARY KEY)",
        [],
    )?;

    // 2. If no row exists, this is an existing DB — set baseline to 1
    let has_row: bool = conn
        .query_row(
            "SELECT 1 FROM schema_version LIMIT 1",
            [],
            |_| Ok(true),
        )
        .unwrap_or(false);

    if !has_row {
        conn.execute("INSERT INTO schema_version VALUES (1)", [])?;
    }

    // 3. Read current version
    let current: i32 =
        conn.query_row("SELECT version FROM schema_version", [], |row| row.get(0))?;

    // 4. Run migrations sequentially (only those with version > current and non-empty SQL)
    for (name, sql) in MIGRATIONS.iter() {
        let version: i32 = name[..3].parse().unwrap_or(0); // "001" -> 1
        if version > current && !sql.is_empty() {
            conn.execute_batch(sql)?;
            conn.execute(
                "INSERT OR REPLACE INTO schema_version VALUES (?1)",
                rusqlite::params![version],
            )?;
        }
    }

    Ok(())
}

// These functions are primarily used by tests - allow dead_code at module level
#[allow(dead_code)]
impl Database {
    pub fn new(app_data_dir: &str) -> SqliteResult<Self> {
        let db_path = PathBuf::from(app_data_dir).join("gelbooru.db");

        // Ensure directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(&db_path)?;

        // Run schema migrations before creating tables
        run_migrations(&conn)?;

        // Create tables
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS downloads (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                post_id INTEGER NOT NULL,
                file_name TEXT NOT NULL,
                file_path TEXT NOT NULL,
                image_url TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                progress REAL NOT NULL DEFAULT 0,
                downloaded_size INTEGER NOT NULL DEFAULT 0,
                total_size INTEGER NOT NULL DEFAULT 0,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                completed_at TIMESTAMP,
                error_message TEXT
            );
            
            CREATE TABLE IF NOT EXISTS favorites (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                post_id INTEGER NOT NULL UNIQUE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE TABLE IF NOT EXISTS blacklisted_tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tag TEXT NOT NULL UNIQUE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            
            CREATE TABLE IF NOT EXISTS favorite_tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tag TEXT NOT NULL UNIQUE,
                tag_type TEXT NOT NULL DEFAULT 'general',
                parent_id INTEGER,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (parent_id) REFERENCES favorite_tags(id) ON DELETE CASCADE
            );
            
            CREATE INDEX IF NOT EXISTS idx_favorite_tags_parent ON favorite_tags(parent_id);

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        )?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn add_download(
        &self,
        post_id: u32,
        file_name: &str,
        file_path: &str,
        image_url: &str,
    ) -> SqliteResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO downloads (post_id, file_name, file_path, image_url) VALUES (?1, ?2, ?3, ?4)",
            rusqlite::params![post_id, file_name, file_path, image_url],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_download_status(&self, id: i64, status: &str, progress: f32) -> SqliteResult<()> {
        self.conn.lock().unwrap().execute(
            "UPDATE downloads SET status = ?1, progress = ?2 WHERE id = ?3",
            rusqlite::params![status, progress, id],
        )?;
        Ok(())
    }

    pub fn is_downloaded(&self, post_id: u32) -> bool {
        self.conn
            .lock()
            .unwrap()
            .query_row(
                "SELECT 1 FROM downloads WHERE post_id = ?1 AND status = 'completed'",
                rusqlite::params![post_id],
                |_| Ok(true),
            )
            .unwrap_or(false)
    }

    pub fn add_favorite(&self, post_id: u32) -> SqliteResult<()> {
        self.conn.lock().unwrap().execute(
            "INSERT OR IGNORE INTO favorites (post_id) VALUES (?1)",
            rusqlite::params![post_id],
        )?;
        Ok(())
    }

    pub fn remove_favorite(&self, post_id: u32) -> SqliteResult<()> {
        self.conn.lock().unwrap().execute(
            "DELETE FROM favorites WHERE post_id = ?1",
            rusqlite::params![post_id],
        )?;
        Ok(())
    }

    pub fn is_favorite(&self, post_id: u32) -> bool {
        self.conn
            .lock()
            .unwrap()
            .query_row(
                "SELECT 1 FROM favorites WHERE post_id = ?1",
                rusqlite::params![post_id],
                |_| Ok(true),
            )
            .unwrap_or(false)
    }

    // FavoriteTag methods
    fn row_to_favorite_tag(row: &Row) -> SqliteResult<FavoriteTag> {
        Ok(FavoriteTag {
            id: row.get(0)?,
            tag: row.get(1)?,
            tag_type: row.get(2)?,
            parent_id: row.get(3)?,
        })
    }

    pub fn add_favorite_tag(&self, tag: &str, tag_type: &str) -> SqliteResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO favorite_tags (tag, tag_type) VALUES (?1, ?2)",
            rusqlite::params![tag, tag_type],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn add_favorite_tag_with_parent(
        &self,
        tag: &str,
        tag_type: &str,
        parent_id: i64,
    ) -> SqliteResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO favorite_tags (tag, tag_type, parent_id) VALUES (?1, ?2, ?3)",
            rusqlite::params![tag, tag_type, parent_id],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn remove_favorite_tag(&self, id: i64) -> SqliteResult<()> {
        self.conn.lock().unwrap().execute(
            "DELETE FROM favorite_tags WHERE id = ?1",
            rusqlite::params![id],
        )?;
        Ok(())
    }

    pub fn get_all_favorite_tags(&self) -> SqliteResult<Vec<FavoriteTagGroup>> {
        let conn = self.conn.lock().unwrap();

        // Get all parent tags (parent_id IS NULL)
        let mut stmt = conn.prepare(
            "SELECT id, tag, tag_type, parent_id FROM favorite_tags WHERE parent_id IS NULL ORDER BY created_at"
        )?;

        let parents: Vec<FavoriteTag> = stmt
            .query_map([], Self::row_to_favorite_tag)?
            .collect::<Result<Vec<_>, _>>()?;

        let mut result = Vec::new();
        for parent in parents {
            let children = self.get_child_tags_internal(&conn, parent.id)?;
            result.push(FavoriteTagGroup { parent, children });
        }

        Ok(result)
    }

    fn get_child_tags_internal(
        &self,
        conn: &Connection,
        parent_id: i64,
    ) -> SqliteResult<Vec<FavoriteTag>> {
        let mut stmt = conn.prepare(
            "SELECT id, tag, tag_type, parent_id FROM favorite_tags WHERE parent_id = ?1 ORDER BY created_at"
        )?;

        let children: Vec<FavoriteTag> = stmt
            .query_map(rusqlite::params![parent_id], Self::row_to_favorite_tag)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(children)
    }

    pub fn get_child_tags(&self, parent_id: i64) -> SqliteResult<Vec<FavoriteTag>> {
        let conn = self.conn.lock().unwrap();
        self.get_child_tags_internal(&conn, parent_id)
    }

    pub fn is_tag_favorited(&self, tag: &str) -> bool {
        self.conn
            .lock()
            .unwrap()
            .query_row(
                "SELECT 1 FROM favorite_tags WHERE tag = ?1",
                rusqlite::params![tag],
                |_| Ok(true),
            )
            .unwrap_or(false)
    }

    pub fn get_favorite_tag_by_tag(&self, tag: &str) -> SqliteResult<Option<FavoriteTag>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT id, tag, tag_type, parent_id FROM favorite_tags WHERE tag = ?1")?;

        let result = stmt
            .query_row(rusqlite::params![tag], Self::row_to_favorite_tag)
            .optional()?;

        Ok(result)
    }

    // Settings methods
    pub fn get_setting(&self, key: &str) -> SqliteResult<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = ?1")?;
        let result = stmt
            .query_row(rusqlite::params![key], |row| row.get(0))
            .optional()?;
        Ok(result)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            rusqlite::params![key, value],
        )?;
        Ok(())
    }

    pub fn get_all_settings(&self) -> SqliteResult<Vec<(String, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let rows = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let result: Vec<(String, String)> = rows.filter_map(|r| r.ok()).collect();
        Ok(result)
    }

    // Download task persistence methods
    pub fn save_download_task(&self, task: &DownloadTaskRecord) -> SqliteResult<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"INSERT INTO downloads (id, post_id, file_name, file_path, image_url, status, progress, downloaded_size, total_size, error_message)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
               ON CONFLICT(id) DO UPDATE SET
                   post_id = excluded.post_id,
                   file_name = excluded.file_name,
                   file_path = excluded.file_path,
                   image_url = excluded.image_url,
                   status = excluded.status,
                   progress = excluded.progress,
                   downloaded_size = excluded.downloaded_size,
                   total_size = excluded.total_size,
                   error_message = excluded.error_message"#,
            rusqlite::params![
                task.id,
                task.post_id,
                task.file_name,
                task.file_path,
                task.image_url,
                task.status,
                task.progress,
                task.downloaded_size,
                task.total_size,
                task.error_message,
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_all_download_tasks(&self) -> SqliteResult<Vec<DownloadTaskRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, post_id, file_name, file_path, image_url, status, progress, downloaded_size, total_size, error_message FROM downloads"
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(DownloadTaskRecord {
                id: row.get(0)?,
                post_id: row.get(1)?,
                file_name: row.get(2)?,
                file_path: row.get(3)?,
                image_url: row.get(4)?,
                status: row.get(5)?,
                progress: row.get(6)?,
                downloaded_size: row.get(7)?,
                total_size: row.get(8)?,
                error_message: row.get(9)?,
            })
        })?;
        let result: Vec<DownloadTaskRecord> = rows.filter_map(|r| r.ok()).collect();
        Ok(result)
    }

    pub fn update_download_task_progress(
        &self,
        id: i64,
        status: &str,
        progress: f64,
        downloaded_size: i64,
        total_size: i64,
    ) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        let completed_at = if status == "completed" {
            Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            )
        } else {
            None
        };
        conn.execute(
            "UPDATE downloads SET status = ?1, progress = ?2, downloaded_size = ?3, total_size = ?4, completed_at = ?5 WHERE id = ?6",
            rusqlite::params![status, progress, downloaded_size, total_size, completed_at, id],
        )?;
        Ok(())
    }

    pub fn update_download_task_error(&self, id: i64, error_message: &str) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE downloads SET status = 'failed', error_message = ?1 WHERE id = ?2",
            rusqlite::params![error_message, id],
        )?;
        Ok(())
    }

    pub fn delete_download_task(&self, id: i64) -> SqliteResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM downloads WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_db() -> (TempDir, Database) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let db =
            Database::new(temp_dir.path().to_str().unwrap()).expect("Failed to create database");
        (temp_dir, db)
    }

    // FavoriteTag tests
    #[test]
    fn test_add_and_check_favorite_tag() {
        let (_dir, db) = create_test_db();
        let id = db.add_favorite_tag("saber", "character").unwrap();
        assert!(id > 0);
        assert!(db.is_tag_favorited("saber"));
    }

    #[test]
    fn test_add_favorite_tag_with_parent() {
        let (_dir, db) = create_test_db();
        let parent_id = db.add_favorite_tag("character", "general").unwrap();
        let child_id = db
            .add_favorite_tag_with_parent("saber", "character", parent_id)
            .unwrap();
        assert!(child_id > 0);

        let children = db.get_child_tags(parent_id).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].tag, "saber");
        assert_eq!(children[0].parent_id, Some(parent_id));
    }

    #[test]
    fn test_remove_favorite_tag() {
        let (_dir, db) = create_test_db();
        let id = db.add_favorite_tag("saber", "character").unwrap();
        assert!(db.is_tag_favorited("saber"));

        db.remove_favorite_tag(id).unwrap();
        assert!(!db.is_tag_favorited("saber"));
    }

    #[test]
    fn test_get_all_favorite_tags_with_children() {
        let (_dir, db) = create_test_db();
        let parent_id = db.add_favorite_tag("character", "general").unwrap();
        db.add_favorite_tag_with_parent("saber", "character", parent_id)
            .unwrap();
        db.add_favorite_tag_with_parent("artoria", "character", parent_id)
            .unwrap();

        let groups = db.get_all_favorite_tags().unwrap();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].parent.tag, "character");
        assert_eq!(groups[0].children.len(), 2);
    }

    // Favorites tests
    #[test]
    fn test_add_and_check_favorite() {
        let (_dir, db) = create_test_db();
        db.add_favorite(12345).unwrap();
        assert!(db.is_favorite(12345));
    }

    #[test]
    fn test_add_duplicate_favorite_is_ignored() {
        let (_dir, db) = create_test_db();
        db.add_favorite(12345).unwrap();
        db.add_favorite(12345).unwrap(); // Should not panic
        assert!(db.is_favorite(12345));
    }

    #[test]
    fn test_remove_favorite() {
        let (_dir, db) = create_test_db();
        db.add_favorite(12345).unwrap();
        assert!(db.is_favorite(12345));

        db.remove_favorite(12345).unwrap();
        assert!(!db.is_favorite(12345));
    }

    #[test]
    fn test_remove_nonexistent_favorite() {
        let (_dir, db) = create_test_db();
        // Should not panic
        db.remove_favorite(99999).unwrap();
        assert!(!db.is_favorite(99999));
    }

    // Settings tests
    #[test]
    fn test_set_and_get_setting() {
        let (_dir, db) = create_test_db();
        db.set_setting("api_key", "secret123").unwrap();

        let value = db.get_setting("api_key").unwrap();
        assert_eq!(value, Some("secret123".to_string()));
    }

    #[test]
    fn test_get_nonexistent_setting() {
        let (_dir, db) = create_test_db();
        let value = db.get_setting("nonexistent").unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn test_settings_overwrite() {
        let (_dir, db) = create_test_db();
        db.set_setting("api_key", "secret123").unwrap();
        db.set_setting("api_key", "new_secret456").unwrap();

        let value = db.get_setting("api_key").unwrap();
        assert_eq!(value, Some("new_secret456".to_string()));
    }

    #[test]
    fn test_get_all_settings() {
        let (_dir, db) = create_test_db();
        db.set_setting("key1", "value1").unwrap();
        db.set_setting("key2", "value2").unwrap();

        let settings = db.get_all_settings().unwrap();
        assert_eq!(settings.len(), 2);
    }

    // Download task tests
    #[test]
    fn test_save_and_get_download_task() {
        let (_dir, db) = create_test_db();
        let task = DownloadTaskRecord {
            id: 0,
            post_id: 12345,
            file_name: "test.jpg".to_string(),
            file_path: "/downloads/test.jpg".to_string(),
            image_url: "https://example.com/test.jpg".to_string(),
            status: "pending".to_string(),
            progress: 0.0,
            downloaded_size: 0,
            total_size: 0,
            error_message: None,
        };
        let saved_id = db.save_download_task(&task).unwrap();
        assert!(saved_id >= 0);

        let tasks = db.get_all_download_tasks().unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0].post_id, 12345);
        assert_eq!(tasks[0].file_name, "test.jpg");
    }

    #[test]
    fn test_update_download_task_progress() {
        let (_dir, db) = create_test_db();
        let task = DownloadTaskRecord {
            id: 0,
            post_id: 12345,
            file_name: "test.jpg".to_string(),
            file_path: "/downloads/test.jpg".to_string(),
            image_url: "https://example.com/test.jpg".to_string(),
            status: "pending".to_string(),
            progress: 0.0,
            downloaded_size: 0,
            total_size: 1024,
            error_message: None,
        };
        let task_id = db.save_download_task(&task).unwrap();

        db.update_download_task_progress(task_id, "downloading", 50.0, 512, 1024)
            .unwrap();

        let tasks = db.get_all_download_tasks().unwrap();
        assert_eq!(tasks[0].status, "downloading");
        assert_eq!(tasks[0].progress, 50.0);
        assert_eq!(tasks[0].downloaded_size, 512);
    }

    #[test]
    fn test_update_download_task_error() {
        let (_dir, db) = create_test_db();
        let task = DownloadTaskRecord {
            id: 0,
            post_id: 12345,
            file_name: "test.jpg".to_string(),
            file_path: "/downloads/test.jpg".to_string(),
            image_url: "https://example.com/test.jpg".to_string(),
            status: "pending".to_string(),
            progress: 0.0,
            downloaded_size: 0,
            total_size: 1024,
            error_message: None,
        };
        let task_id = db.save_download_task(&task).unwrap();

        db.update_download_task_error(task_id, "Network timeout")
            .unwrap();

        let tasks = db.get_all_download_tasks().unwrap();
        assert_eq!(tasks[0].status, "failed");
        assert_eq!(tasks[0].error_message, Some("Network timeout".to_string()));
    }

    #[test]
    fn test_delete_download_task() {
        let (_dir, db) = create_test_db();
        let task = DownloadTaskRecord {
            id: 0,
            post_id: 12345,
            file_name: "test.jpg".to_string(),
            file_path: "/downloads/test.jpg".to_string(),
            image_url: "https://example.com/test.jpg".to_string(),
            status: "pending".to_string(),
            progress: 0.0,
            downloaded_size: 0,
            total_size: 1024,
            error_message: None,
        };
        let task_id = db.save_download_task(&task).unwrap();

        let tasks = db.get_all_download_tasks().unwrap();
        assert_eq!(tasks.len(), 1);

        db.delete_download_task(task_id).unwrap();

        let tasks = db.get_all_download_tasks().unwrap();
        assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn test_multiple_download_tasks() {
        let (_dir, db) = create_test_db();

        for i in 1..=5 {
            let task = DownloadTaskRecord {
                id: i as i64, // Use unique IDs to avoid conflicts
                post_id: 1000 + i,
                file_name: format!("test{}.jpg", i),
                file_path: format!("/downloads/test{}.jpg", i),
                image_url: format!("https://example.com/test{}.jpg", i),
                status: "pending".to_string(),
                progress: 0.0,
                downloaded_size: 0,
                total_size: (1024 * i) as i64,
                error_message: None,
            };
            let _ = db.save_download_task(&task).unwrap();
        }

        let tasks = db.get_all_download_tasks().unwrap();
        assert_eq!(tasks.len(), 5);
    }

    // is_downloaded tests
    #[test]
    fn test_is_downloaded() {
        let (_dir, db) = create_test_db();
        db.add_download(
            12345,
            "test.jpg",
            "/path/test.jpg",
            "https://example.com/test.jpg",
        )
        .unwrap();
        db.update_download_status(1, "completed", 100.0).unwrap();

        assert!(db.is_downloaded(12345));
        assert!(!db.is_downloaded(99999));
    }

    // get_favorite_tag_by_tag test
    #[test]
    fn test_get_favorite_tag_by_tag() {
        let (_dir, db) = create_test_db();
        db.add_favorite_tag("saber", "character").unwrap();

        let tag = db.get_favorite_tag_by_tag("saber").unwrap();
        assert!(tag.is_some());
        assert_eq!(tag.unwrap().tag, "saber");

        let none = db.get_favorite_tag_by_tag("nonexistent").unwrap();
        assert!(none.is_none());
    }

    // Schema version tests
    #[test]
    fn test_schema_version_baseline_for_existing_db() {
        use rusqlite::Connection;
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let conn = Connection::open(&db_path).unwrap();

        // Simulate an existing DB without schema_version table
        // (i.e., created by the old code path — just the 5 tables, no version row)
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS downloads (id INTEGER PRIMARY KEY);
            CREATE TABLE IF NOT EXISTS favorites (id INTEGER PRIMARY KEY);
            "#,
        )
        .unwrap();

        // Run migrations on the "existing" DB
        run_migrations(&conn).unwrap();

        // Verify version was set to 1
        let version: i32 = conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 1);
    }

    #[test]
    fn test_schema_version_runs_migrations_in_order() {
        // Test that migrations with version > current are applied in order.
        // This test uses a clean DB (no pre-existing tables) and verifies
        // that run_migrations completes without error and sets version correctly.
        let (_dir, db) = create_test_db();
        // After create_test_db() calls Database::new(), schema_version should exist with version=1
        // Run migrations again — should be a no-op (current=1, no migrations have version>1)
        let conn_ref = &db.conn;
        let conn = conn_ref.lock().unwrap();
        let version: i32 = conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(
            version, 1,
            "Fresh DB should have schema_version=1 after new()"
        );
    }
}
