use std::{
    fs, path::Path, sync::Mutex, thread, time::Duration,
};

use arboard::Clipboard;
use image::{imageops::FilterType, ImageBuffer, Rgba};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter};




use crate::{
    db::{
        encrypted_images_dir, has_content_hash, images_raw_dir,
        images_thumbnail_dir, insert_file_record, insert_image_record, insert_text_record_with_hash,
        is_hash_deleted,
    },
    utils::time,
};

// 静态变量存储 AppHandle
static APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);

pub fn set_event_emitter(handle: AppHandle) {
    let mut guard = APP_HANDLE.lock().unwrap();
    *guard = Some(handle);
}

fn emit_new_record(content_type: &str) {
    let guard = APP_HANDLE.lock().unwrap();
    if let Some(handle) = guard.as_ref() {
        let _ = handle.emit("clipboard-new-record", content_type);
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn xor_encrypt(bytes: &[u8], key: &[u8]) -> Vec<u8> {
    if key.is_empty() {
        return bytes.to_vec();
    }

    bytes
        .iter()
        .enumerate()
        .map(|(idx, val)| val ^ key[idx % key.len()])
        .collect()
}

#[cfg(target_os = "windows")]
fn extract_file_icon(file_path: &str) -> Option<Vec<u8>> {
    use winapi::um::shellapi::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_SMALLICON};
    use winapi::shared::minwindef::DWORD;
    use winapi::um::wingdi::*;
    use winapi::um::winuser::*;
    use std::ptr::null_mut;

    let path: Vec<u16> = file_path.encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        let mut shfi: SHFILEINFOW = std::mem::zeroed();

        // 移除 SHGFI_USEFILEATTRIBUTES，使用实际文件图标
        let result = SHGetFileInfoW(
            path.as_ptr(),
            0,
            &mut shfi as *mut _ as *mut _,
            std::mem::size_of::<SHFILEINFOW>() as DWORD,
            SHGFI_ICON | SHGFI_SMALLICON,
        );

        if result == 0 || shfi.hIcon.is_null() {
            return None;
        }

        // 创建设备上下文
        let hdc = GetDC(null_mut());
        if hdc.is_null() {
            DestroyIcon(shfi.hIcon);
            return None;
        }

        // 创建兼容的内存DC
        let mem_dc = CreateCompatibleDC(hdc);
        if mem_dc.is_null() {
            ReleaseDC(null_mut(), hdc);
            DestroyIcon(shfi.hIcon);
            return None;
        }

        // 创建位图 - 使用 32x32 尺寸以获得更清晰的图标
        let bmp = CreateCompatibleBitmap(hdc, 32, 32);
        if bmp.is_null() {
            DeleteDC(mem_dc);
            ReleaseDC(null_mut(), hdc);
            DestroyIcon(shfi.hIcon);
            return None;
        }

        let old_bmp = SelectObject(mem_dc, bmp as *mut _);

        // 使用 0x0003 (DI_NORMAL) 标志绘制图标到 32x32
        DrawIconEx(
            mem_dc,
            0,
            0,
            shfi.hIcon,
            32,
            32,
            0,
            null_mut(),
            0x0003,
        );

        // 获取位图数据 - 32x32
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as DWORD,
                biWidth: 32,
                biHeight: -32,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [RGBQUAD {
                rgbBlue: 0,
                rgbGreen: 0,
                rgbRed: 0,
                rgbReserved: 0,
            }],
        };

        let mut buffer = vec![0u8; 32 * 32 * 4];
        GetDIBits(
            hdc,
            bmp,
            0,
            32,
            buffer.as_mut_ptr() as *mut _,
            &mut bmi,
            DIB_RGB_COLORS,
        );

        // 清理资源
        SelectObject(mem_dc, old_bmp);
        DeleteObject(bmp as *mut _);
        DeleteDC(mem_dc);
        ReleaseDC(null_mut(), hdc);
        DestroyIcon(shfi.hIcon);

        // 转换 BGRA 到 RGBA - 32x32
        let mut rgba_buffer = Vec::with_capacity(32 * 32 * 4);
        for chunk in buffer.chunks_exact(4) {
            rgba_buffer.push(chunk[2]); // R
            rgba_buffer.push(chunk[1]); // G
            rgba_buffer.push(chunk[0]); // B
            rgba_buffer.push(chunk[3]); // A
        }

        println!("成功提取文件图标: {}, 数据大小: {} 字节", file_path, rgba_buffer.len());
        Some(rgba_buffer)
    }
}

#[cfg(not(target_os = "windows"))]
fn extract_file_icon(_file_path: &str) -> Option<Vec<u8>> {
    // Linux/macOS 暂不支持
    None
}

#[cfg(target_os = "windows")]
fn get_clipboard_files() -> Option<Vec<String>> {
    use std::os::windows::ffi::OsStringExt;
    use clipboard_win::{Clipboard, formats, is_format_avail, raw};
    use winapi::um::shellapi::DragQueryFileW;
    use winapi::um::shellapi::HDROP;

    // 创建临时的剪贴板访问
    let _clipboard = match Clipboard::new() {
        Ok(c) => c,
        Err(err) => {
            eprintln!("无法打开剪贴板: {}", err);
            return None;
        }
    };

    // 检查是否有 CF_HDROP 格式
    let has_hdrop = is_format_avail(formats::CF_HDROP);

    if !has_hdrop {
        return None;
    }

    unsafe {
        let hdrop = match raw::get_clipboard_data(formats::CF_HDROP) {
            Ok(h) => h,
            Err(err) => {
                eprintln!("无法获取 CF_HDROP 数据: {}", err);
                return None;
            }
        };

        let hdrop_ptr = hdrop.as_ptr() as HDROP;

        if hdrop_ptr.is_null() {
            eprintln!("HDROP 指针为空");
            return None;
        }

        let count = DragQueryFileW(hdrop_ptr, 0xFFFFFFFF, std::ptr::null_mut(), 0);

        if count == 0 {
            return None;
        }

        let mut files = Vec::with_capacity(count as usize);
        for i in 0..count {
            let len = DragQueryFileW(hdrop_ptr, i, std::ptr::null_mut(), 0);
            if len == 0 {
                continue;
            }

            let mut buffer = vec![0u16; (len + 1) as usize];
            DragQueryFileW(hdrop_ptr, i, buffer.as_mut_ptr(), buffer.len() as u32);

            let path = std::ffi::OsString::from_wide(&buffer[..len as usize])
                .to_string_lossy()
                .to_string();

            files.push(path);
        }

        if files.is_empty() {
            None
        } else {
            Some(files)
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn get_clipboard_files() -> Option<Vec<String>> {
    // Linux/macOS 暂不支持文件列表检测
    None
}

pub fn start_clipboard_monitor() {
    let polling_ms = std::env::var("CLIP_VERSE_POLLING_MS")
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .filter(|val| *val >= 100)
        .unwrap_or(500);

    thread::spawn(move || {
        println!("剪贴板监控线程已启动，轮询间隔: {polling_ms}ms");

        let mut clipboard = match Clipboard::new() {
            Ok(instance) => instance,
            Err(err) => {
                eprintln!("无法连接系统剪贴板: {err}");
                return;
            }
        };

        loop {
            // 优先检测文件列表（Windows 系统）
            #[cfg(target_os = "windows")]
            {
                if let Some(files) = get_clipboard_files() {
                    let hash_input = files.join("|");
                    let hash = sha256_hex(hash_input.as_bytes());

                    // 检查是否在黑名单中（已被用户删除）
                    if is_hash_deleted(&hash) {
                        continue;
                    }

                    match has_content_hash(&hash) {
                        Ok(false) => {
                            println!("检测到新文件: {:?}", files);
                            // 为每个文件创建记录
                            for file_path in &files {
                                let metadata = match fs::metadata(file_path) {
                                    Ok(m) => m,
                                    Err(err) => {
                                        eprintln!("无法获取文件元数据 {}: {}", file_path, err);
                                        continue;
                                    }
                                };

                                let file_size = metadata.len() as i64;
                                let file_name = Path::new(file_path)
                                    .file_name()
                                    .and_then(|n| n.to_str())
                                    .map(|s| s.to_string());

                                // 提取文件图标
                                let icon_path: Option<String> = match extract_file_icon(file_path) {
                                    Some(icon_data) => {
                                        let now = time::now_timestamp_millis();
                                        let hash_short = &sha256_hex(&icon_data)[..16];
                                        let icon_file_name = format!("icon_{}_{}.png", now, hash_short);

                                        // 按日期创建图标目录
                                        let date_str = time::now_date_path();
                                        let icon_date_dir = images_thumbnail_dir().join(&date_str);
                                        if let Err(err) = fs::create_dir_all(&icon_date_dir) {
                                            eprintln!("创建图标目录失败: {err}");
                                            None
                                        } else {
                                            let icon_path = icon_date_dir.join(&icon_file_name);

                                            // 保存图标 - 使用 32x32 尺寸
                                            match ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(32, 32, icon_data) {
                                                Some(buffer) => {
                                                    match buffer.save(&icon_path) {
                                                        Ok(_) => Some(icon_path.to_string_lossy().to_string()),
                                                        Err(err) => {
                                                            eprintln!("保存图标失败: {err}");
                                                            None
                                                        }
                                                    }
                                                }
                                                None => None,
                                            }
                                        }
                                    }
                                    None => None,
                                };

                                if let Err(err) = insert_file_record(
                                    file_path,
                                    file_size,
                                    file_name.as_deref(),
                                    icon_path.as_deref(),
                                    &hash,
                                ) {
                                    eprintln!("保存文件记录失败: {err}");
                                } else {
                                    println!("检测到新的文件，已记录路径: {}", file_path);
                                }
                            }
                            emit_new_record("file");
                        }
                        Ok(true) => {}
                        Err(err) => eprintln!("查询文件哈希失败: {err}"),
                    }
                    continue;
                }
            }

            // 检测文本
            if let Ok(text) = clipboard.get_text() {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    let hash = sha256_hex(trimmed.as_bytes());

                    // 检查是否在黑名单中（已被用户删除）
                    if is_hash_deleted(&hash) {
                        continue;
                    }

                    match has_content_hash(&hash) {
                        Ok(false) => {
                            if let Err(err) = insert_text_record_with_hash(trimmed, &hash) {
                                eprintln!("保存文本剪贴板记录失败: {err}");
                            } else {
                                println!("检测到新的文本剪贴板内容，已保存");
                                emit_new_record("text");
                            }
                        }
                        Ok(true) => {}
                        Err(err) => eprintln!("查询文本哈希失败: {err}"),
                    }
                }
            } else if let Ok(image) = clipboard.get_image() {
                let raw_data = image.bytes.into_owned();
                let mut hashed = Vec::with_capacity(raw_data.len() + 16);
                hashed.extend_from_slice(&(image.width as u64).to_le_bytes());
                hashed.extend_from_slice(&(image.height as u64).to_le_bytes());
                hashed.extend_from_slice(&raw_data);

                let hash = sha256_hex(&hashed);

                // 检查是否在黑名单中（已被用户删除）
                if is_hash_deleted(&hash) {
                    continue;
                }

                match has_content_hash(&hash) {
                    Ok(true) => {}
                    Ok(false) => {
                        let now = time::now_timestamp_millis();
                        let file_name = format!("{now}_{}.png", &hash[..16]);

                        // 按日期创建目录结构: images/raw/YYYY/MM/
                        let date_str = time::now_date_path();
                        let raw_date_dir = images_raw_dir().join(&date_str);
                        if let Err(err) = fs::create_dir_all(&raw_date_dir) {
                            eprintln!("创建目录失败: {err}");
                            continue;
                        }
                        let raw_path = raw_date_dir.join(&file_name);

                        if let Some(buffer) = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_raw(
                            image.width as u32,
                            image.height as u32,
                            raw_data,
                        ) {
                            if let Err(err) = buffer.save(&raw_path) {
                                eprintln!("写入图片文件失败: {err}");
                            } else {
                                let mut thumbnail_rel: Option<String> = None;
                                let mut encrypted_rel: Option<String> = None;
                                let mut encrypted_flag = false;

                                // 按日期创建缩略图目录
                                let thumbnail_date_dir = images_thumbnail_dir().join(&date_str);
                                if let Err(err) = fs::create_dir_all(&thumbnail_date_dir) {
                                    eprintln!("创建缩略图目录失败: {err}");
                                }

                                if std::env::var("CLIP_VERSE_ENABLE_THUMBNAIL").ok().as_deref()
                                    == Some("1")
                                {
                                    let thumb = image::imageops::resize(
                                        &buffer,
                                        256,
                                        256,
                                        FilterType::Triangle,
                                    );
                                    let thumb_name = format!("thumb_{file_name}");
                                    let thumb_path = thumbnail_date_dir.join(&thumb_name);
                                    if let Err(err) = thumb.save(&thumb_path) {
                                        eprintln!("生成缩略图失败: {err}");
                                    } else {
                                        thumbnail_rel =
                                            Some(thumb_path.to_string_lossy().to_string());
                                    }
                                }

                                if std::env::var("CLIP_VERSE_ENABLE_ENCRYPT").ok().as_deref()
                                    == Some("1")
                                {
                                    match fs::read(&raw_path) {
                                        Ok(raw_file) => {
                                            let key = std::env::var("CLIP_VERSE_ENCRYPT_KEY")
                                                .unwrap_or_else(|_| {
                                                    "clip-verse-default-key".to_string()
                                                });
                                            let encrypted = xor_encrypt(&raw_file, key.as_bytes());
                                            let encrypted_name = format!("{file_name}.enc");

                                            // 按日期创建加密目录
                                            let encrypted_date_dir = encrypted_images_dir().join(&date_str);
                                            if let Err(err) = fs::create_dir_all(&encrypted_date_dir) {
                                                eprintln!("创建加密目录失败: {err}");
                                            }
                                            let encrypted_path = encrypted_date_dir.join(encrypted_name);

                                            if let Err(err) = fs::write(&encrypted_path, encrypted)
                                            {
                                                eprintln!("写入加密图片失败: {err}");
                                            } else {
                                                encrypted_flag = true;
                                                encrypted_rel = Some(
                                                    encrypted_path.to_string_lossy().to_string(),
                                                );
                                            }
                                        }
                                        Err(err) => eprintln!("读取原始图片用于加密失败: {err}"),
                                    }
                                }

                                if let Err(err) = insert_image_record(
                                    &raw_path.to_string_lossy(),
                                    thumbnail_rel.as_deref(),
                                    encrypted_rel.as_deref(),
                                    image.width as i64,
                                    image.height as i64,
                                    (image.width * image.height * 4) as i64,
                                    &hash,
                                    encrypted_flag,
                                ) {
                                    eprintln!("写入图片记录失败: {err}");
                                } else {
                                    println!(
                                        "检测到新的图片剪贴板内容，已保存: {}",
                                        raw_path.to_string_lossy()
                                    );
                                    emit_new_record("image");
                                }
                            }
                        } else {
                            eprintln!("无法解析剪贴板图片数据");
                        }
                    }
                    Err(err) => eprintln!("查询图片哈希失败: {err}"),
                }
            }

            thread::sleep(Duration::from_millis(polling_ms));
        }
    });
}
