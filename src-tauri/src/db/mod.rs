use rusqlite::{Connection, Result as SqliteResult};
use std::path::PathBuf;
use std::sync::Mutex;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_data_dir: &str) -> SqliteResult<Self> {
        let db_path = PathBuf::from(app_data_dir).join("gelbooru.db");
        
        // Ensure directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        
        let conn = Connection::open(&db_path)?;
        
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
            "#,
        )?;
        
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
    
    pub fn add_download(&self, post_id: u32, file_name: &str, file_path: &str, image_url: &str) -> SqliteResult<i64> {
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
        self.conn.lock().unwrap()
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
        self.conn.lock().unwrap()
            .query_row(
                "SELECT 1 FROM favorites WHERE post_id = ?1",
                rusqlite::params![post_id],
                |_| Ok(true),
            )
            .unwrap_or(false)
    }
}
