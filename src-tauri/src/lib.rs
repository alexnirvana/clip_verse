mod db;
mod utils;

use db::{delete_record, init_db, insert_text_record, list_text_records, stats, ClipboardRecord, DashboardStats};

#[tauri::command]
fn init_app() -> Result<String, String> {
    init_db().map_err(|e| e.to_string())?;
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
fn get_text_records(limit: Option<i64>, keyword: Option<String>) -> Result<Vec<ClipboardRecord>, String> {
    let safe_limit = limit.unwrap_or(100).clamp(1, 500);
    list_text_records(safe_limit, keyword.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
fn remove_record(record_id: i64) -> Result<(), String> {
    delete_record(record_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_dashboard_stats() -> Result<DashboardStats, String> {
    stats().map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(err) = init_db() {
        eprintln!("数据库初始化失败: {err}");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            init_app,
            add_text_record,
            get_text_records,
            remove_record,
            get_dashboard_stats
        ])
        .run(tauri::generate_context!())
        .expect("运行应用时发生错误");
}
