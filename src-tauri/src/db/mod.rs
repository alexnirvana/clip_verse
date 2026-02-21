use std::{fs, path::PathBuf};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use thiserror::Error;

use crate::utils::time;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("数据库错误: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("IO 错误: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Serialize)]
pub struct ClipboardRecord {
    pub id: i64,
    pub content_type: String,
    pub timestamp: i64,
    pub created_at: String,
    pub preview: String,
    pub content_size: i64,
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_records: i64,
}

fn data_root() -> PathBuf {
    if let Ok(custom_home) = std::env::var("HOME") {
        PathBuf::from(custom_home).join(".clip_verse")
    } else {
        PathBuf::from(".clip_verse")
    }
}

pub fn images_raw_dir() -> PathBuf {
    data_root().join("images").join("raw")
}

pub fn images_thumbnail_dir() -> PathBuf {
    data_root().join("images").join("thumbnails")
}

pub fn encrypted_images_dir() -> PathBuf {
    data_root().join("encrypted").join("images")
}

fn db_path() -> PathBuf {
    data_root().join("database").join("clipboard.db")
}

fn ensure_dirs() -> Result<(), DbError> {
    fs::create_dir_all(data_root().join("database"))?;
    fs::create_dir_all(images_raw_dir())?;
    fs::create_dir_all(images_thumbnail_dir())?;
    fs::create_dir_all(encrypted_images_dir())?;
    fs::create_dir_all(data_root().join("logs"))?;
    Ok(())
}

fn connection() -> Result<Connection, DbError> {
    ensure_dirs()?;
    Ok(Connection::open(db_path())?)
}

fn ensure_column(conn: &Connection, table: &str, column: &str, ddl: &str) -> Result<(), DbError> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let columns = stmt.query_map([], |row| row.get::<_, String>(1))?;

    let mut exists = false;
    for col in columns {
        if col? == column {
            exists = true;
            break;
        }
    }

    if !exists {
        conn.execute(ddl, [])?;
    }

    Ok(())
}

pub fn init_db() -> Result<(), DbError> {
    let conn = connection()?;

    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;

        CREATE TABLE IF NOT EXISTS clipboard_records (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content_type TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            preview TEXT,
            content_size INTEGER,
            is_encrypted BOOLEAN DEFAULT 0,
            is_favorite BOOLEAN DEFAULT 0,
            content_hash TEXT
        );

        CREATE TABLE IF NOT EXISTS text_contents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            record_id INTEGER NOT NULL,
            content TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (record_id) REFERENCES clipboard_records(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS image_contents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            record_id INTEGER NOT NULL,
            file_path TEXT NOT NULL,
            thumbnail_path TEXT,
            encrypted_path TEXT,
            width INTEGER NOT NULL,
            height INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            FOREIGN KEY (record_id) REFERENCES clipboard_records(id) ON DELETE CASCADE
        );

        CREATE INDEX IF NOT EXISTS idx_clipboard_records_timestamp
            ON clipboard_records(timestamp DESC);
        CREATE INDEX IF NOT EXISTS idx_clipboard_records_content_type
            ON clipboard_records(content_type);
        CREATE INDEX IF NOT EXISTS idx_clipboard_records_content_hash
            ON clipboard_records(content_hash);
        CREATE INDEX IF NOT EXISTS idx_text_contents_record_id
            ON text_contents(record_id);
        CREATE INDEX IF NOT EXISTS idx_image_contents_record_id
            ON image_contents(record_id);
        ",
    )?;

    ensure_column(
        &conn,
        "clipboard_records",
        "content_hash",
        "ALTER TABLE clipboard_records ADD COLUMN content_hash TEXT",
    )?;

    Ok(())
}

pub fn has_content_hash(content_hash: &str) -> Result<bool, DbError> {
    let conn = connection()?;
    let result = conn
        .query_row(
            "SELECT id FROM clipboard_records WHERE content_hash = ?1 LIMIT 1",
            params![content_hash],
            |row| row.get::<_, i64>(0),
        )
        .optional()?;

    Ok(result.is_some())
}

pub fn insert_text_record(content: &str) -> Result<i64, DbError> {
    insert_text_record_with_hash(content, "")
}

pub fn insert_text_record_with_hash(content: &str, content_hash: &str) -> Result<i64, DbError> {
    let conn = connection()?;
    let now_ts = time::now_timestamp_millis();
    let now_iso = time::now_iso8601();
    let preview = content.chars().take(80).collect::<String>();
    let content_size = content.len() as i64;

    conn.execute(
        "INSERT INTO clipboard_records (
            content_type, timestamp, created_at, updated_at, preview, content_size, is_encrypted, is_favorite, content_hash
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 0, ?7)",
        params!["text", now_ts, now_iso, now_iso, preview, content_size, content_hash],
    )?;

    let record_id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO text_contents (record_id, content, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![record_id, content, now_iso, now_iso],
    )?;

    Ok(record_id)
}

pub fn insert_image_record(
    file_path: &str,
    thumbnail_path: Option<&str>,
    encrypted_path: Option<&str>,
    width: i64,
    height: i64,
    content_size: i64,
    content_hash: &str,
    is_encrypted: bool,
) -> Result<i64, DbError> {
    let conn = connection()?;
    let now_ts = time::now_timestamp_millis();
    let now_iso = time::now_iso8601();
    let preview = format!("图片 {}x{}", width, height);

    conn.execute(
        "INSERT INTO clipboard_records (
            content_type, timestamp, created_at, updated_at, preview, content_size, is_encrypted, is_favorite, content_hash
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8)",
        params!["image", now_ts, now_iso, now_iso, preview, content_size, is_encrypted as i32, content_hash],
    )?;

    let record_id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO image_contents (record_id, file_path, thumbnail_path, encrypted_path, width, height, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![record_id, file_path, thumbnail_path, encrypted_path, width, height, now_iso, now_iso],
    )?;

    Ok(record_id)
}

pub fn list_text_records(
    limit: i64,
    keyword: Option<&str>,
) -> Result<Vec<ClipboardRecord>, DbError> {
    let conn = connection()?;
    let mut records = Vec::new();

    if let Some(search) = keyword {
        let like = format!("%{}%", search);
        let mut stmt = conn.prepare(
            "SELECT r.id, r.content_type, r.timestamp, r.created_at, COALESCE(r.preview, ''),
                    COALESCE(r.content_size, 0), t.content
             FROM clipboard_records r
             INNER JOIN text_contents t ON t.record_id = r.id
             WHERE r.content_type = 'text' AND t.content LIKE ?1
             ORDER BY r.timestamp DESC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![like, limit], |row| {
            Ok(ClipboardRecord {
                id: row.get(0)?,
                content_type: row.get(1)?,
                timestamp: row.get(2)?,
                created_at: row.get(3)?,
                preview: row.get(4)?,
                content_size: row.get(5)?,
                content: row.get(6)?,
            })
        })?;

        for row in rows {
            records.push(row?);
        }
    } else {
        let mut stmt = conn.prepare(
            "SELECT r.id, r.content_type, r.timestamp, r.created_at, COALESCE(r.preview, ''),
                    COALESCE(r.content_size, 0), t.content
             FROM clipboard_records r
             INNER JOIN text_contents t ON t.record_id = r.id
             WHERE r.content_type = 'text'
             ORDER BY r.timestamp DESC
             LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit], |row| {
            Ok(ClipboardRecord {
                id: row.get(0)?,
                content_type: row.get(1)?,
                timestamp: row.get(2)?,
                created_at: row.get(3)?,
                preview: row.get(4)?,
                content_size: row.get(5)?,
                content: row.get(6)?,
            })
        })?;

        for row in rows {
            records.push(row?);
        }
    }

    Ok(records)
}

pub fn delete_record(record_id: i64) -> Result<(), DbError> {
    let conn = connection()?;
    conn.execute(
        "DELETE FROM clipboard_records WHERE id = ?1",
        params![record_id],
    )?;
    Ok(())
}

pub fn stats() -> Result<DashboardStats, DbError> {
    let conn = connection()?;
    let total_records = conn.query_row("SELECT COUNT(*) FROM clipboard_records", [], |row| {
        row.get::<_, i64>(0)
    })?;

    Ok(DashboardStats { total_records })
}
