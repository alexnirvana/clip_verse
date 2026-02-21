use std::{
    collections::HashSet,
    fs,
    path::PathBuf,
    sync::{LazyLock, Mutex},
};

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use thiserror::Error;

use crate::utils::time;

// 全局黑名单：存储已删除记录的内容哈希
static DELETED_HASHES: LazyLock<Mutex<HashSet<String>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

pub fn add_deleted_hash(hash: &str) {
    let mut hashes = DELETED_HASHES.lock().unwrap();
    hashes.insert(hash.to_string());
}

pub fn is_hash_deleted(hash: &str) -> bool {
    let hashes = DELETED_HASHES.lock().unwrap();
    hashes.contains(hash)
}

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
    pub image_path: Option<String>,
    pub thumbnail_path: Option<String>,
    pub file_path: Option<String>,
    pub icon_path: Option<String>,
    pub is_favorite: bool,
}

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_records: i64,
}

pub const RECORD_EXPIRATION_DAYS: i64 = 200;

pub fn data_root() -> PathBuf {
    if let Ok(custom_home) = std::env::var("HOME") {
        PathBuf::from(custom_home).join(".clip_verse")
    } else {
        // Windows: 使用 APPDATA 或 USERPROFILE 环境变量
        if cfg!(windows) {
            if let Ok(appdata) = std::env::var("APPDATA") {
                PathBuf::from(appdata).join("clip_verse")
            } else if let Ok(userprofile) = std::env::var("USERPROFILE") {
                PathBuf::from(userprofile).join(".clip_verse")
            } else {
                PathBuf::from(".clip_verse")
            }
        } else {
            PathBuf::from(".clip_verse")
        }
    }
}

#[derive(Debug, Serialize, serde::Deserialize, Default)]
pub struct LocalSettingsConfig {
    #[serde(default)]
    pub auto_start_enabled: bool,
    #[serde(default)]
    pub record_expiration_enabled: bool,
    #[serde(default = "default_expiration_days")]
    pub expiration_days: i64,
}

fn default_expiration_days() -> i64 {
    200
}

pub fn settings_config_path() -> PathBuf {
    data_root().join("config").join("settings.json")
}

pub fn ensure_settings_config() -> Result<(), DbError> {
    let config_path = settings_config_path();
    if config_path.exists() {
        return Ok(());
    }

    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let default_config = LocalSettingsConfig::default();
    let json = serde_json::to_string_pretty(&default_config).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("序列化配置失败: {e}"))
    })?;
    fs::write(config_path, json)?;
    Ok(())
}

pub fn read_local_settings_config() -> Result<LocalSettingsConfig, DbError> {
    ensure_settings_config()?;
    let raw = fs::read_to_string(settings_config_path())?;
    let config = serde_json::from_str::<LocalSettingsConfig>(&raw).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("解析配置失败: {e}"))
    })?;
    Ok(config)
}

pub fn write_local_settings_config(config: &LocalSettingsConfig) -> Result<(), DbError> {
    let config_path = settings_config_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(config).map_err(|e| {
        std::io::Error::new(std::io::ErrorKind::Other, format!("序列化配置失败: {e}"))
    })?;
    fs::write(config_path, json)?;
    Ok(())
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

pub fn db_path() -> PathBuf {
    data_root().join("database").join("clipboard.db")
}

fn ensure_dirs() -> Result<(), DbError> {
    fs::create_dir_all(data_root().join("database"))?;
    fs::create_dir_all(images_raw_dir())?;
    fs::create_dir_all(images_thumbnail_dir())?;
    fs::create_dir_all(encrypted_images_dir())?;
    fs::create_dir_all(data_root().join("logs"))?;
    fs::create_dir_all(data_root().join("config"))?;
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

        CREATE TABLE IF NOT EXISTS file_contents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            record_id INTEGER NOT NULL,
            file_path TEXT NOT NULL,
            file_size INTEGER,
            file_name TEXT,
            icon_path TEXT,
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
        CREATE INDEX IF NOT EXISTS idx_file_contents_record_id
            ON file_contents(record_id);

        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );
        ",
    )?;

    ensure_column(
        &conn,
        "clipboard_records",
        "content_hash",
        "ALTER TABLE clipboard_records ADD COLUMN content_hash TEXT",
    )?;

    ensure_column(
        &conn,
        "file_contents",
        "icon_path",
        "ALTER TABLE file_contents ADD COLUMN icon_path TEXT",
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

pub fn insert_file_record(
    file_path: &str,
    file_size: i64,
    file_name: Option<&str>,
    icon_path: Option<&str>,
    content_hash: &str,
) -> Result<i64, DbError> {
    let conn = connection()?;
    let now_ts = time::now_timestamp_millis();
    let now_iso = time::now_iso8601();
    let preview = format!(
        "文件: {}",
        file_name.unwrap_or_else(|| {
            std::path::Path::new(file_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("未知文件")
        })
    );

    conn.execute(
        "INSERT INTO clipboard_records (
            content_type, timestamp, created_at, updated_at, preview, content_size, is_encrypted, is_favorite, content_hash
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, 0, ?7)",
        params!["file", now_ts, now_iso, now_iso, preview, file_size, content_hash],
    )?;

    let record_id = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO file_contents (record_id, file_path, file_size, file_name, icon_path, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![record_id, file_path, file_size, file_name, icon_path, now_iso, now_iso],
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
                    COALESCE(r.content_size, 0), t.content, COALESCE(r.is_favorite, 0)
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
                image_path: None,
                thumbnail_path: None,
                file_path: None,
                icon_path: None,
                is_favorite: row.get::<_, i64>(7)? != 0,
            })
        })?;

        for row in rows {
            records.push(row?);
        }
    } else {
        let mut stmt = conn.prepare(
            "SELECT r.id, r.content_type, r.timestamp, r.created_at, COALESCE(r.preview, ''),
                    COALESCE(r.content_size, 0), t.content, COALESCE(r.is_favorite, 0)
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
                image_path: None,
                thumbnail_path: None,
                file_path: None,
                icon_path: None,
                is_favorite: row.get::<_, i64>(7)? != 0,
            })
        })?;

        for row in rows {
            records.push(row?);
        }
    }

    Ok(records)
}

pub fn get_file_metadata(record_id: i64) -> Result<(String, i64, Option<String>), DbError> {
    let conn = connection()?;
    conn.query_row(
        "SELECT file_path, file_size, file_name FROM file_contents WHERE record_id = ?1",
        params![record_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )
    .map_err(|e| e.into())
}

pub fn delete_record(record_id: i64) -> Result<(), DbError> {
    let conn = connection()?;

    // 先获取要删除记录的 content_hash
    let hash: Option<String> = conn
        .query_row(
            "SELECT content_hash FROM clipboard_records WHERE id = ?1",
            params![record_id],
            |row| row.get(0),
        )
        .optional()?;

    // 获取图片路径（如果是图片记录）
    let (image_path, thumbnail_path, encrypted_path): (Option<String>, Option<String>, Option<String>) = conn
        .query_row(
            "SELECT file_path, thumbnail_path, encrypted_path FROM image_contents WHERE record_id = ?1",
            params![record_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()?
        .unwrap_or((None, None, None));

    // 获取文件图标路径（如果是文件记录）
    let icon_path: Option<String> = conn
        .query_row(
            "SELECT icon_path FROM file_contents WHERE record_id = ?1",
            params![record_id],
            |row| row.get(0),
        )
        .optional()?
        .flatten();

    // 删除记录（会自动级联删除 text_contents、image_contents 和 file_contents）
    conn.execute(
        "DELETE FROM clipboard_records WHERE id = ?1",
        params![record_id],
    )?;

    // 删除图片文件
    if let Some(path) = image_path {
        let _ = std::fs::remove_file(&path);
    }
    if let Some(path) = thumbnail_path {
        let _ = std::fs::remove_file(&path);
    }
    if let Some(path) = encrypted_path {
        let _ = std::fs::remove_file(&path);
    }

    // 删除文件图标（缩略图）
    if let Some(path) = icon_path {
        let _ = std::fs::remove_file(&path);
    }

    // 将哈希加入黑名单，防止重新记录
    if let Some(hash) = hash {
        add_deleted_hash(&hash);
    }

    Ok(())
}

pub fn set_favorite(record_id: i64, is_favorite: bool) -> Result<(), DbError> {
    let conn = connection()?;
    conn.execute(
        "UPDATE clipboard_records SET is_favorite = ?1, updated_at = ?2 WHERE id = ?3",
        params![
            if is_favorite { 1 } else { 0 },
            time::now_iso8601(),
            record_id
        ],
    )?;
    Ok(())
}

pub fn get_auto_start_enabled() -> Result<bool, DbError> {
    let config = read_local_settings_config()?;
    Ok(config.auto_start_enabled)
}

pub fn set_auto_start_enabled(enabled: bool) -> Result<(), DbError> {
    let mut config = read_local_settings_config()?;
    config.auto_start_enabled = enabled;
    write_local_settings_config(&config)
}

pub fn get_record_expiration_enabled() -> Result<bool, DbError> {
    let config = read_local_settings_config()?;
    Ok(config.record_expiration_enabled)
}

pub fn get_expiration_days() -> Result<i64, DbError> {
    let config = read_local_settings_config()?;
    Ok(config.expiration_days)
}

pub fn set_record_expiration_enabled(enabled: bool) -> Result<(), DbError> {
    let mut config = read_local_settings_config()?;
    config.record_expiration_enabled = enabled;
    write_local_settings_config(&config)
}

pub fn set_expiration_days(days: i64) -> Result<(), DbError> {
    let mut config = read_local_settings_config()?;
    config.expiration_days = days;
    write_local_settings_config(&config)
}

pub fn cleanup_expired_records_on_startup() -> Result<usize, DbError> {
    if !get_record_expiration_enabled()? {
        return Ok(0);
    }

    let now_ms = time::now_timestamp_millis();
    let days = get_expiration_days().unwrap_or(RECORD_EXPIRATION_DAYS);
    let cutoff_ms = now_ms - days * 24 * 60 * 60 * 1000;

    let expired_ids: Vec<i64> = {
        let conn = connection()?;
        let mut stmt = conn.prepare(
            "SELECT id FROM clipboard_records WHERE timestamp < ?1 ORDER BY timestamp ASC",
        )?;
        let rows = stmt.query_map(params![cutoff_ms], |row| row.get::<_, i64>(0))?;
        rows.collect::<Result<Vec<_>, _>>()?
    };

    for record_id in &expired_ids {
        delete_record(*record_id)?;
    }

    Ok(expired_ids.len())
}

pub fn stats() -> Result<DashboardStats, DbError> {
    let conn = connection()?;
    let total_records = conn.query_row("SELECT COUNT(*) FROM clipboard_records", [], |row| {
        row.get::<_, i64>(0)
    })?;

    Ok(DashboardStats { total_records })
}

// 获取所有记录（包括文本和图片）
pub fn list_all_records(
    limit: i64,
    keyword: Option<&str>,
) -> Result<Vec<ClipboardRecord>, DbError> {
    let conn = connection()?;
    let mut records = Vec::new();

    if let Some(search) = keyword {
        let like = format!("%{}%", search);
        let mut stmt = conn.prepare(
            "SELECT r.id, r.content_type, r.timestamp, r.created_at, COALESCE(r.preview, ''),
                    COALESCE(r.content_size, 0),
                    COALESCE(t.content, '') as content,
                    i.file_path as image_path,
                    i.thumbnail_path,
                    f.file_path as file_path,
                    f.icon_path,
                    COALESCE(r.is_favorite, 0)
             FROM clipboard_records r
             LEFT JOIN text_contents t ON t.record_id = r.id
             LEFT JOIN image_contents i ON i.record_id = r.id
             LEFT JOIN file_contents f ON f.record_id = r.id
             WHERE COALESCE(t.content, r.preview, f.file_name) LIKE ?1
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
                image_path: row.get(7)?,
                thumbnail_path: row.get(8)?,
                file_path: row.get(9)?,
                icon_path: row.get(10)?,
                is_favorite: row.get::<_, i64>(11)? != 0,
            })
        })?;

        for row in rows {
            records.push(row?);
        }
    } else {
        let mut stmt = conn.prepare(
            "SELECT r.id, r.content_type, r.timestamp, r.created_at, COALESCE(r.preview, ''),
                    COALESCE(r.content_size, 0),
                    COALESCE(t.content, '') as content,
                    i.file_path as image_path,
                    i.thumbnail_path,
                    f.file_path as file_path,
                    f.icon_path,
                    COALESCE(r.is_favorite, 0)
             FROM clipboard_records r
             LEFT JOIN text_contents t ON t.record_id = r.id
             LEFT JOIN image_contents i ON i.record_id = r.id
             LEFT JOIN file_contents f ON f.record_id = r.id
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
                image_path: row.get(7)?,
                thumbnail_path: row.get(8)?,
                file_path: row.get(9)?,
                icon_path: row.get(10)?,
                is_favorite: row.get::<_, i64>(11)? != 0,
            })
        })?;

        for row in rows {
            records.push(row?);
        }
    }

    Ok(records)
}
