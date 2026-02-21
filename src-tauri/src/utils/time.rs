use chrono::{Datelike, FixedOffset, Utc};

/// 返回中国时区（UTC+8）当前时间。
pub fn now_shanghai() -> chrono::DateTime<FixedOffset> {
    let offset = FixedOffset::east_opt(8 * 3600).expect("UTC+8 时区必须有效");
    Utc::now().with_timezone(&offset)
}

/// 返回毫秒级时间戳（单位：毫秒）。
pub fn now_timestamp_millis() -> i64 {
    now_shanghai().timestamp_millis()
}

/// 返回 ISO8601 字符串，带 +08:00 偏移。
pub fn now_iso8601() -> String {
    now_shanghai().to_rfc3339()
}

/// 返回日期路径 YYYY/MM/DD，用于按日期组织文件。
pub fn now_date_path() -> String {
    let dt = now_shanghai();
    format!("{}/{:02}/{:02}", dt.year(), dt.month(), dt.day())
}
