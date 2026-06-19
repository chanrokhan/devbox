use aes_gcm::{
    Aes128Gcm, Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, OsRng},
};
use base64::{Engine, engine::general_purpose};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use md5::Md5;
use rand::{Rng, RngCore, distributions::Alphanumeric, thread_rng};
use serde::Serialize;
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};
use uuid::Uuid;

#[derive(Serialize)]
struct ClockSnapshot {
    local: String,
    utc: String,
    unix_seconds: i64,
    unix_millis: i64,
}

#[derive(Serialize)]
struct JsonResult {
    output: String,
    status: String,
    parsed: Option<serde_json::Value>,
}

#[tauri::command]
fn clock_snapshot() -> ClockSnapshot {
    let now = Local::now();

    ClockSnapshot {
        local: now.format("%Y-%m-%d %H:%M:%S %:z").to_string(),
        utc: now
            .with_timezone(&Utc)
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
        unix_seconds: now.timestamp(),
        unix_millis: now.timestamp_millis(),
    }
}

#[tauri::command]
fn time_from_timestamp(input: String) -> String {
    let trimmed = input.trim();
    let parsed = match trimmed.parse::<i64>() {
        Ok(value) => value,
        Err(_) => return "请输入有效整数时间戳".to_string(),
    };

    let millis = if trimmed.len() >= 13 {
        parsed
    } else {
        parsed.saturating_mul(1000)
    };

    match Utc.timestamp_millis_opt(millis).single() {
        Some(utc) => {
            let local: DateTime<Local> = DateTime::from(utc);
            format!(
                "Local: {}\nUTC:   {}\nUnix seconds: {}\nUnix millis:  {}",
                local.format("%Y-%m-%d %H:%M:%S %:z"),
                utc.format("%Y-%m-%d %H:%M:%S UTC"),
                utc.timestamp(),
                utc.timestamp_millis()
            )
        }
        None => "时间戳超出可表示范围".to_string(),
    }
}

#[tauri::command]
fn timestamp_from_local(input: String) -> String {
    let naive = match NaiveDateTime::parse_from_str(input.trim(), "%Y-%m-%d %H:%M:%S") {
        Ok(value) => value,
        Err(_) => return "无法解析日期，请使用 YYYY-MM-DD HH:mm:ss".to_string(),
    };

    match Local.from_local_datetime(&naive).single() {
        Some(date) => format!(
            "Unix seconds: {}\nUnix millis:  {}\nUTC:          {}",
            date.timestamp(),
            date.timestamp_millis(),
            date.with_timezone(&Utc).format("%Y-%m-%d %H:%M:%S UTC")
        ),
        None => "本地时间不明确或不存在，请换一个时间点".to_string(),
    }
}

#[tauri::command]
fn process_json(input: String, mode: String) -> JsonResult {
    let parsed = parse_json_value(&input);
    let status = json_status_from_result(&parsed);

    let output = match (&parsed, mode.as_str()) {
        (Ok(value), "minify") => serde_json::to_string(value).unwrap_or_else(|err| err.to_string()),
        (Ok(value), _) => serde_json::to_string_pretty(value).unwrap_or_else(|err| err.to_string()),
        (Err(err), _) => format!("JSON 解析失败: {err}"),
    };

    JsonResult {
        output,
        status,
        parsed: parsed.ok(),
    }
}

#[tauri::command]
fn codec_transform(input: String, mode: String) -> String {
    match mode.as_str() {
        "base64_encode" => base64_encode(&input),
        "base64_decode" => base64_decode(&input),
        "url_encode" => urlencoding::encode(&input).to_string(),
        "url_decode" => url_decode(&input),
        "hex_encode" => hex::encode(input.as_bytes()),
        "hex_decode" => hex_decode(&input),
        _ => "未知编码转换类型".to_string(),
    }
}

#[tauri::command]
fn hash_text(input: String, algorithm: String) -> String {
    match algorithm.as_str() {
        "md5" => hash_digest::<Md5>(&input),
        "sha1" => hash_digest::<Sha1>(&input),
        "sha256" => hash_digest::<Sha256>(&input),
        "sha512" => hash_digest::<Sha512>(&input),
        _ => "未知摘要算法".to_string(),
    }
}

#[tauri::command]
fn aes_gcm_transform(input: String, key_hex: String, nonce_hex: String, mode: String) -> String {
    match mode.as_str() {
        "encrypt" => aes_encrypt(&input, &key_hex, &nonce_hex),
        "decrypt" => aes_decrypt(&input, &key_hex, &nonce_hex),
        _ => "未知 AES 操作".to_string(),
    }
}

#[tauri::command]
fn generate_random(kind: String, length: String) -> String {
    let size = parse_len(&length);

    match kind.as_str() {
        "password" => random_password(size),
        "hex" => random_hex(size),
        "base64" => base64_encode_bytes(size),
        "uuid" => Uuid::new_v4().to_string(),
        _ => "未知生成类型".to_string(),
    }
}

#[tauri::command]
fn random_hex_bytes(length: String) -> String {
    random_hex(parse_len(&length))
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            clock_snapshot,
            time_from_timestamp,
            timestamp_from_local,
            process_json,
            codec_transform,
            hash_text,
            aes_gcm_transform,
            generate_random,
            random_hex_bytes,
        ])
        .run(tauri::generate_context!())
        .expect("error while running DevToolbox");
}

fn parse_json_value(input: &str) -> serde_json::Result<serde_json::Value> {
    serde_json::from_str::<serde_json::Value>(input)
}

fn json_status_from_result(result: &serde_json::Result<serde_json::Value>) -> String {
    match result {
        Ok(value) => {
            let kind = match value {
                serde_json::Value::Null => "null",
                serde_json::Value::Bool(_) => "boolean",
                serde_json::Value::Number(_) => "number",
                serde_json::Value::String(_) => "string",
                serde_json::Value::Array(_) => "array",
                serde_json::Value::Object(_) => "object",
            };
            format!("JSON 有效，根节点类型：{kind}")
        }
        Err(err) => format!("JSON 无效: {err}"),
    }
}

fn base64_encode(input: &str) -> String {
    general_purpose::STANDARD.encode(input.as_bytes())
}

fn base64_encode_bytes(size: usize) -> String {
    let mut bytes = vec![0_u8; size];
    OsRng.fill_bytes(&mut bytes);
    general_purpose::STANDARD.encode(bytes)
}

fn base64_decode(input: &str) -> String {
    match general_purpose::STANDARD.decode(input.trim()) {
        Ok(bytes) => String::from_utf8(bytes)
            .unwrap_or_else(|err| format!("Base64 已解码，但不是 UTF-8 文本: {err}")),
        Err(err) => format!("Base64 解码失败: {err}"),
    }
}

fn url_decode(input: &str) -> String {
    match urlencoding::decode(input) {
        Ok(value) => value.to_string(),
        Err(err) => format!("URL 解码失败: {err}"),
    }
}

fn hex_decode(input: &str) -> String {
    match hex::decode(input.trim()) {
        Ok(bytes) => String::from_utf8(bytes)
            .unwrap_or_else(|err| format!("Hex 已解码，但不是 UTF-8 文本: {err}")),
        Err(err) => format!("Hex 解码失败: {err}"),
    }
}

fn hash_digest<D>(input: &str) -> String
where
    D: Digest + Default,
{
    let mut hasher = D::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

fn aes_encrypt(input: &str, key_hex: &str, nonce_hex: &str) -> String {
    let key = match hex::decode(key_hex.trim()) {
        Ok(value) => value,
        Err(err) => return format!("Key Hex 无效: {err}"),
    };
    let nonce = match decode_nonce(nonce_hex) {
        Ok(value) => value,
        Err(err) => return err,
    };

    let encrypted = match key.len() {
        16 => Aes128Gcm::new_from_slice(&key)
            .map_err(|err| err.to_string())
            .and_then(|cipher| {
                cipher
                    .encrypt(Nonce::from_slice(&nonce), input.as_bytes())
                    .map_err(|err| err.to_string())
            }),
        32 => Aes256Gcm::new_from_slice(&key)
            .map_err(|err| err.to_string())
            .and_then(|cipher| {
                cipher
                    .encrypt(Nonce::from_slice(&nonce), input.as_bytes())
                    .map_err(|err| err.to_string())
            }),
        _ => return "AES-GCM Key 必须是 16 或 32 字节 Hex".to_string(),
    };

    encrypted
        .map(|bytes| general_purpose::STANDARD.encode(bytes))
        .unwrap_or_else(|err| format!("AES 加密失败: {err}"))
}

fn aes_decrypt(input: &str, key_hex: &str, nonce_hex: &str) -> String {
    let key = match hex::decode(key_hex.trim()) {
        Ok(value) => value,
        Err(err) => return format!("Key Hex 无效: {err}"),
    };
    let nonce = match decode_nonce(nonce_hex) {
        Ok(value) => value,
        Err(err) => return err,
    };
    let ciphertext = match general_purpose::STANDARD.decode(input.trim()) {
        Ok(value) => value,
        Err(err) => return format!("密文 Base64 无效: {err}"),
    };

    let decrypted = match key.len() {
        16 => Aes128Gcm::new_from_slice(&key)
            .map_err(|err| err.to_string())
            .and_then(|cipher| {
                cipher
                    .decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref())
                    .map_err(|err| err.to_string())
            }),
        32 => Aes256Gcm::new_from_slice(&key)
            .map_err(|err| err.to_string())
            .and_then(|cipher| {
                cipher
                    .decrypt(Nonce::from_slice(&nonce), ciphertext.as_ref())
                    .map_err(|err| err.to_string())
            }),
        _ => return "AES-GCM Key 必须是 16 或 32 字节 Hex".to_string(),
    };

    decrypted
        .map(|bytes| {
            String::from_utf8(bytes)
                .unwrap_or_else(|err| format!("解密成功，但不是 UTF-8 文本: {err}"))
        })
        .unwrap_or_else(|err| format!("AES 解密失败: {err}"))
}

fn decode_nonce(nonce_hex: &str) -> Result<Vec<u8>, String> {
    match hex::decode(nonce_hex.trim()) {
        Ok(value) if value.len() == 12 => Ok(value),
        Ok(_) => Err("AES-GCM Nonce 必须是 12 字节 Hex".to_string()),
        Err(err) => Err(format!("Nonce Hex 无效: {err}")),
    }
}

fn random_password(size: usize) -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .map(char::from)
        .collect()
}

fn random_hex(size: usize) -> String {
    let mut bytes = vec![0_u8; size];
    OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

fn parse_len(input: &str) -> usize {
    input.trim().parse::<usize>().unwrap_or(32).clamp(1, 4096)
}
