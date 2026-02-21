mod db;
mod monitor;
mod utils;

use db::{
    cleanup_expired_records_on_startup, db_path, delete_record, ensure_settings_config,
    get_auto_start_enabled, get_expiration_days, get_file_metadata, get_record_expiration_enabled, images_raw_dir,
    init_db, insert_text_record, list_all_records, list_text_records, set_auto_start_enabled,
    set_expiration_days, set_favorite, set_record_expiration_enabled, settings_config_path, stats, ClipboardRecord,
    DashboardStats,
};
use monitor::{set_event_emitter, start_clipboard_monitor};
use tauri_plugin_autostart::ManagerExt;

#[tauri::command]
fn init_app() -> Result<String, String> {
    init_db().map_err(|e| e.to_string())?;
    cleanup_expired_records_on_startup().map_err(|e| e.to_string())?;
    Ok("初始化完成".to_string())
}

#[tauri::command]
fn add_text_record(content: String) -> Result<i64, String> {
    if content.trim().is_empty() {
        return Err("文本内容不能为空".to_string());
    }
    insert_text_record(&content).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_text_records(
    limit: Option<i64>,
    keyword: Option<String>,
) -> Result<Vec<ClipboardRecord>, String> {
    let safe_limit = limit.unwrap_or(100).clamp(1, 500);
    list_text_records(safe_limit, keyword.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_all_records(
    limit: Option<i64>,
    keyword: Option<String>,
) -> Result<Vec<ClipboardRecord>, String> {
    let safe_limit = limit.unwrap_or(100).clamp(1, 500);
    list_all_records(safe_limit, keyword.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_record(record_id: i64) -> Result<(), String> {
    delete_record(record_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_dashboard_stats() -> Result<DashboardStats, String> {
    stats().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_file_info(record_id: i64) -> Result<(String, i64, Option<String>), String> {
    get_file_metadata(record_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_favorite(record_id: i64, is_favorite: bool) -> Result<(), String> {
    set_favorite(record_id, is_favorite).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
struct StorageSettings {
    database_path: String,
    image_save_path: String,
    settings_json_path: String,
}

#[tauri::command]
fn get_storage_settings() -> Result<StorageSettings, String> {
    ensure_settings_config().map_err(|e| e.to_string())?;
    Ok(StorageSettings {
        database_path: db_path().to_string_lossy().to_string(),
        image_save_path: images_raw_dir().to_string_lossy().to_string(),
        settings_json_path: settings_config_path().to_string_lossy().to_string(),
    })
}

#[derive(serde::Serialize)]
struct AutoStartSettings {
    auto_start_enabled: bool,
}

#[tauri::command]
fn get_auto_start_settings() -> Result<AutoStartSettings, String> {
    let enabled = get_auto_start_enabled().map_err(|e| e.to_string())?;
    Ok(AutoStartSettings {
        auto_start_enabled: enabled,
    })
}

#[tauri::command]
fn set_auto_start_settings(app: tauri::AppHandle, auto_start_enabled: bool) -> Result<(), String> {
    if auto_start_enabled {
        app.autolaunch().enable().map_err(|e| e.to_string())?;
    } else {
        app.autolaunch().disable().map_err(|e| e.to_string())?;
    }

    set_auto_start_enabled(auto_start_enabled).map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
struct RecordExpirationSettings {
    expiration_enabled: bool,
    expiration_days: i64,
}

#[tauri::command]
fn get_record_expiration_settings() -> Result<RecordExpirationSettings, String> {
    let enabled = get_record_expiration_enabled().map_err(|e| e.to_string())?;
    let days = get_expiration_days().map_err(|e| e.to_string())?;
    Ok(RecordExpirationSettings {
        expiration_enabled: enabled,
        expiration_days: days,
    })
}

#[tauri::command]
fn set_record_expiration_settings(expiration_enabled: bool, expiration_days: Option<i64>) -> Result<(), String> {
    set_record_expiration_enabled(expiration_enabled).map_err(|e| e.to_string())?;
    if let Some(days) = expiration_days {
        set_expiration_days(days).map_err(|e| e.to_string())?;
    }
    if expiration_enabled {
        cleanup_expired_records_on_startup().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(err) = init_db() {
        eprintln!("数据库初始化失败: {err}");
    }

    start_clipboard_monitor();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            // 设置事件发送器
            set_event_emitter(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            init_app,
            add_text_record,
            get_text_records,
            get_all_records,
            remove_record,
            get_dashboard_stats,
            get_file_info,
            get_storage_settings,
            get_auto_start_settings,
            set_auto_start_settings,
            get_record_expiration_settings,
            set_record_expiration_settings,
            toggle_favorite
        ])
        .run(tauri::generate_context!())
        .expect("运行应用时发生错误");
}
