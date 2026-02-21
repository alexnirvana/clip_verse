use std::{fs, thread, time::Duration};

use arboard::Clipboard;
use image::{imageops::FilterType, ImageBuffer, Rgba};
use sha2::{Digest, Sha256};

use crate::{
    db::{
        encrypted_images_dir, has_content_hash, images_raw_dir, images_thumbnail_dir,
        insert_image_record, insert_text_record_with_hash,
    },
    utils::time,
};

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
            if let Ok(text) = clipboard.get_text() {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    let hash = sha256_hex(trimmed.as_bytes());
                    match has_content_hash(&hash) {
                        Ok(false) => {
                            if let Err(err) = insert_text_record_with_hash(trimmed, &hash) {
                                eprintln!("保存文本剪贴板记录失败: {err}");
                            } else {
                                println!("检测到新的文本剪贴板内容，已保存");
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
                match has_content_hash(&hash) {
                    Ok(true) => {}
                    Ok(false) => {
                        let now = time::now_timestamp_millis();
                        let file_name = format!("{now}_{}.png", &hash[..16]);
                        let raw_path = images_raw_dir().join(&file_name);

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
                                    let thumb_path = images_thumbnail_dir().join(&thumb_name);
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
                                            let encrypted_path =
                                                encrypted_images_dir().join(encrypted_name);
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
