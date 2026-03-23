use rusqlite::{Connection, OptionalExtension, Result as SqliteResult, Row};
use std::path::PathBuf;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

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
            
            CREATE TABLE IF NOT EXISTS favorite_tags (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                tag TEXT NOT NULL UNIQUE,
                tag_type TEXT NOT NULL DEFAULT 'general',
                parent_id INTEGER,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (parent_id) REFERENCES favorite_tags(id) ON DELETE CASCADE
            );
            
            CREATE INDEX IF NOT EXISTS idx_favorite_tags_parent ON favorite_tags(parent_id);
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
    
    pub fn add_favorite_tag_with_parent(&self, tag: &str, tag_type: &str, parent_id: i64) -> SqliteResult<i64> {
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
    
    fn get_child_tags_internal(&self, conn: &Connection, parent_id: i64) -> SqliteResult<Vec<FavoriteTag>> {
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
        self.conn.lock().unwrap()
            .query_row(
                "SELECT 1 FROM favorite_tags WHERE tag = ?1",
                rusqlite::params![tag],
                |_| Ok(true),
            )
            .unwrap_or(false)
    }
    
    pub fn get_favorite_tag_by_tag(&self, tag: &str) -> SqliteResult<Option<FavoriteTag>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, tag, tag_type, parent_id FROM favorite_tags WHERE tag = ?1"
        )?;
        
        let result = stmt
            .query_row(rusqlite::params![tag], Self::row_to_favorite_tag)
            .optional()?;
        
        Ok(result)
    }
}
