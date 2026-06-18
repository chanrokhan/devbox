use aes_gcm::{
    Aes128Gcm, Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, OsRng},
};
use base64::{Engine, engine::general_purpose};
use chrono::{DateTime, Local, NaiveDateTime, TimeZone, Utc};
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use md5::Md5;
use rand::{Rng, RngCore, distributions::Alphanumeric, thread_rng};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};
use uuid::Uuid;

const STYLE: &str = include_str!("../assets/main.css");

fn main() {
    let window = WindowBuilder::new()
        .with_title("DevToolbox")
        .with_inner_size(dioxus::desktop::LogicalSize::new(1180.0, 760.0))
        .with_min_inner_size(dioxus::desktop::LogicalSize::new(980.0, 640.0));

    LaunchBuilder::desktop()
        .with_cfg(Config::new().with_window(window))
        .launch(App);
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tool {
    Time,
    Json,
    Codec,
    Crypto,
    Random,
}

impl Tool {
    fn title(self) -> &'static str {
        match self {
            Self::Time => "时间转换",
            Self::Json => "JSON 工具",
            Self::Codec => "编码转换",
            Self::Crypto => "加密摘要",
            Self::Random => "随机生成",
        }
    }

    fn subtitle(self) -> &'static str {
        match self {
            Self::Time => "Unix 时间戳、UTC、本地时间互转",
            Self::Json => "格式化、压缩、校验 JSON 内容",
            Self::Codec => "Base64、URL、Hex 编解码",
            Self::Crypto => "哈希摘要与 AES-GCM 加解密",
            Self::Random => "Key、密码、UUID 与随机字节",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::Time => "◷",
            Self::Json => "{}",
            Self::Codec => "⇄",
            Self::Crypto => "⌁",
            Self::Random => "✦",
        }
    }

    fn all() -> [Tool; 5] {
        [
            Self::Time,
            Self::Json,
            Self::Codec,
            Self::Crypto,
            Self::Random,
        ]
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum JsonView {
    Source,
    Tree,
}

#[component]
fn App() -> Element {
    let mut selected = use_signal(|| Tool::Time);

    rsx! {
        document::Style { "{STYLE}" }
        main { class: "app-shell",
            aside { class: "sidebar",
                div { class: "brand",
                    div { class: "brand-title",
                        strong { "DevToolbox" }
                        span { "开发工具集" }
                    }
                }

                nav { class: "tool-nav",
                    for tool in Tool::all() {
                        button {
                            class: if selected() == tool { "nav-item active" } else { "nav-item" },
                            onclick: move |_| selected.set(tool),
                            span { class: "nav-icon", "{tool.icon()}" }
                            span { class: "nav-copy",
                                strong { "{tool.title()}" }
                                small { "{tool.subtitle()}" }
                            }
                        }
                    }
                }
            }

            section { class: "workspace",
                header { class: "topbar",
                    div {
                        h1 { "{selected().title()}" }
                        p { "{selected().subtitle()}" }
                    }
                    div { class: "badge", "本地处理" }
                }

                div { class: "tool-surface",
                    match selected() {
                        Tool::Time => rsx! { TimeTool {} },
                        Tool::Json => rsx! { JsonTool {} },
                        Tool::Codec => rsx! { CodecTool {} },
                        Tool::Crypto => rsx! { CryptoTool {} },
                        Tool::Random => rsx! { RandomTool {} },
                    }
                }
            }
        }
    }
}

#[component]
fn TimeTool() -> Element {
    let now = Local::now();
    let mut timestamp = use_signal(|| now.timestamp().to_string());
    let mut input_datetime = use_signal(|| now.format("%Y-%m-%d %H:%M:%S").to_string());
    let mut output = use_signal(|| time_from_timestamp(&timestamp()));

    rsx! {
        div { class: "grid two",
            section { class: "panel",
                div { class: "panel-head",
                    h2 { "时间戳转日期" }
                    span { "秒 / 毫秒自动识别" }
                }
                textarea {
                    class: "field mono short",
                    value: "{timestamp}",
                    oninput: move |event| {
                        timestamp.set(event.value());
                    }
                }
                div { class: "button-row",
                    button {
                        onclick: move |_| output.set(time_from_timestamp(&timestamp())),
                        "转换"
                    }
                    button {
                        onclick: move |_| {
                            let value = Local::now().timestamp().to_string();
                            timestamp.set(value.clone());
                            output.set(time_from_timestamp(&value));
                        },
                        "当前秒"
                    }
                    button {
                        onclick: move |_| {
                            let value = Local::now().timestamp_millis().to_string();
                            timestamp.set(value.clone());
                            output.set(time_from_timestamp(&value));
                        },
                        "当前毫秒"
                    }
                }
                OutputBlock { value: output() }
            }

            section { class: "panel",
                div { class: "panel-head",
                    h2 { "日期转时间戳" }
                    span { "格式：YYYY-MM-DD HH:mm:ss" }
                }
                textarea {
                    class: "field mono short",
                    value: "{input_datetime}",
                    oninput: move |event| input_datetime.set(event.value())
                }
                div { class: "button-row",
                    button {
                        onclick: move |_| output.set(timestamp_from_local(&input_datetime())),
                        "转换"
                    }
                    button {
                        onclick: move |_| {
                            let value = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                            input_datetime.set(value.clone());
                            output.set(timestamp_from_local(&value));
                        },
                        "填入当前时间"
                    }
                }
                div { class: "reference-list",
                    Metric { label: "本地时间", value: Local::now().format("%Y-%m-%d %H:%M:%S %:z").to_string() }
                    Metric { label: "UTC 时间", value: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string() }
                    Metric { label: "Unix 秒", value: Local::now().timestamp().to_string() }
                    Metric { label: "Unix 毫秒", value: Local::now().timestamp_millis().to_string() }
                }
            }
        }
    }
}

#[component]
fn JsonTool() -> Element {
    let mut input = use_signal(|| {
        r#"{"name":"DevToolbox","features":["json","base64","aes"],"config":{"theme":"native","offline":true},"local":true}"#.to_string()
    });
    let mut output = use_signal(|| pretty_json(&input()));
    let mut status = use_signal(|| "等待处理".to_string());
    let mut parsed = use_signal(|| parse_json_value(&input()).ok());
    let mut view = use_signal(|| JsonView::Source);

    rsx! {
        div { class: "grid two fill",
            section { class: "panel tall",
                div { class: "panel-head",
                    h2 { "输入" }
                    span { "粘贴 JSON 文本" }
                }
                textarea {
                    class: "field mono grow",
                    value: "{input}",
                    oninput: move |event| input.set(event.value())
                }
                div { class: "button-row",
                    button {
                        onclick: move |_| {
                            let result = pretty_json(&input());
                            status.set(json_status(&input()));
                            parsed.set(parse_json_value(&input()).ok());
                            output.set(result);
                            view.set(JsonView::Source);
                        },
                        "格式化"
                    }
                    button {
                        onclick: move |_| {
                            let result = minify_json(&input());
                            status.set(json_status(&input()));
                            parsed.set(parse_json_value(&input()).ok());
                            output.set(result);
                            view.set(JsonView::Source);
                        },
                        "压缩"
                    }
                    button {
                        onclick: move |_| {
                            status.set(json_status(&input()));
                            parsed.set(parse_json_value(&input()).ok());
                            if let Ok(value) = parse_json_value(&input()) {
                                output.set(serde_json::to_string_pretty(&value).unwrap_or_default());
                            } else {
                                output.set(json_status(&input()));
                            }
                        },
                        "校验"
                    }
                }
                div { class: "status-pill", "{status}" }
            }

            section { class: "panel tall",
                div { class: "panel-head",
                    h2 { "输出" }
                    span { "高亮源码 / 可折叠树" }
                }
                div { class: "segmented",
                    button {
                        class: if view() == JsonView::Source { "segment active" } else { "segment" },
                        onclick: move |_| view.set(JsonView::Source),
                        "高亮源码"
                    }
                    button {
                        class: if view() == JsonView::Tree { "segment active" } else { "segment" },
                        onclick: move |_| view.set(JsonView::Tree),
                        "树结构"
                    }
                }
                if view() == JsonView::Source {
                    JsonHighlighter { source: output() }
                } else {
                    div { class: "json-tree mono",
                        if let Some(value) = parsed() {
                            JsonTree { value, label: None }
                        } else {
                            div { class: "json-empty", "{status}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn CodecTool() -> Element {
    let mut input = use_signal(|| "Hello, DevToolbox".to_string());
    let mut output = use_signal(String::new);

    rsx! {
        div { class: "grid two fill",
            section { class: "panel tall",
                div { class: "panel-head",
                    h2 { "原文 / 编码内容" }
                    span { "UTF-8 文本" }
                }
                textarea {
                    class: "field mono grow",
                    value: "{input}",
                    oninput: move |event| input.set(event.value())
                }
                div { class: "button-grid",
                    button { onclick: move |_| output.set(base64_encode(&input())), "Base64 编码" }
                    button { onclick: move |_| output.set(base64_decode(&input())), "Base64 解码" }
                    button { onclick: move |_| output.set(urlencoding::encode(&input()).to_string()), "URL 编码" }
                    button { onclick: move |_| output.set(url_decode(&input())), "URL 解码" }
                    button { onclick: move |_| output.set(hex::encode(input().as_bytes())), "Hex 编码" }
                    button { onclick: move |_| output.set(hex_decode(&input())), "Hex 解码" }
                }
            }

            section { class: "panel tall",
                div { class: "panel-head",
                    h2 { "结果" }
                    span { "编码转换输出" }
                }
                textarea {
                    class: "field mono grow",
                    value: "{output}",
                    oninput: move |event| output.set(event.value())
                }
            }
        }
    }
}

#[component]
fn CryptoTool() -> Element {
    let mut input = use_signal(|| "payload".to_string());
    let mut key = use_signal(|| random_hex(32));
    let mut nonce = use_signal(|| random_hex(12));
    let mut output = use_signal(String::new);

    rsx! {
        div { class: "grid two fill",
            section { class: "panel tall",
                div { class: "panel-head",
                    h2 { "输入" }
                    span { "摘要或 AES-GCM 明文/密文" }
                }
                textarea {
                    class: "field mono grow",
                    value: "{input}",
                    oninput: move |event| input.set(event.value())
                }
                div { class: "button-grid",
                    button { onclick: move |_| output.set(hash_digest::<Md5>(&input())), "MD5" }
                    button { onclick: move |_| output.set(hash_digest::<Sha1>(&input())), "SHA-1" }
                    button { onclick: move |_| output.set(hash_digest::<Sha256>(&input())), "SHA-256" }
                    button { onclick: move |_| output.set(hash_digest::<Sha512>(&input())), "SHA-512" }
                }
                div { class: "key-grid",
                    label {
                        span { "AES Key Hex（16 或 32 字节）" }
                        input {
                            class: "field mono single",
                            value: "{key}",
                            oninput: move |event| key.set(event.value())
                        }
                    }
                    label {
                        span { "Nonce Hex（12 字节）" }
                        input {
                            class: "field mono single",
                            value: "{nonce}",
                            oninput: move |event| nonce.set(event.value())
                        }
                    }
                }
                div { class: "button-row",
                    button { onclick: move |_| key.set(random_hex(32)), "生成 AES-256 Key" }
                    button { onclick: move |_| nonce.set(random_hex(12)), "生成 Nonce" }
                    button { onclick: move |_| output.set(aes_encrypt(&input(), &key(), &nonce())), "AES-GCM 加密" }
                    button { onclick: move |_| output.set(aes_decrypt(&input(), &key(), &nonce())), "AES-GCM 解密" }
                }
            }

            section { class: "panel tall",
                div { class: "panel-head",
                    h2 { "结果" }
                    span { "摘要为 Hex；AES 密文为 Base64" }
                }
                textarea {
                    class: "field mono grow",
                    value: "{output}",
                    oninput: move |event| output.set(event.value())
                }
            }
        }
    }
}

#[component]
fn RandomTool() -> Element {
    let mut length = use_signal(|| "32".to_string());
    let mut output = use_signal(|| random_password(32));

    rsx! {
        div { class: "grid two",
            section { class: "panel",
                div { class: "panel-head",
                    h2 { "生成器" }
                    span { "适合 token、secret、测试数据" }
                }
                label { class: "inline-field",
                    span { "长度" }
                    input {
                        class: "field mono single small-input",
                        value: "{length}",
                        oninput: move |event| length.set(event.value())
                    }
                }
                div { class: "button-grid",
                    button {
                        onclick: move |_| output.set(random_password(parse_len(&length()))),
                        "随机密码"
                    }
                    button { onclick: move |_| output.set(random_hex(parse_len(&length()))), "Hex Key" }
                    button { onclick: move |_| output.set(base64_encode_bytes(parse_len(&length()))), "Base64 Key" }
                    button { onclick: move |_| output.set(Uuid::new_v4().to_string()), "UUID v4" }
                }
                div { class: "reference-list",
                    Metric { label: "AES-128 Key", value: random_hex(16) }
                    Metric { label: "AES-256 Key", value: random_hex(32) }
                    Metric { label: "GCM Nonce", value: random_hex(12) }
                }
            }
            section { class: "panel",
                div { class: "panel-head",
                    h2 { "结果" }
                    span { "一键复制生成值" }
                }
                textarea {
                    class: "field mono medium",
                    value: "{output}",
                    oninput: move |event| output.set(event.value())
                }
            }
        }
    }
}

#[component]
fn Metric(label: String, value: String) -> Element {
    rsx! {
        div { class: "metric",
            span { "{label}" }
            code { "{value}" }
        }
    }
}

#[component]
fn OutputBlock(value: String) -> Element {
    rsx! {
        div { class: "output-block mono", "{value}" }
    }
}

#[component]
fn JsonHighlighter(source: String) -> Element {
    let tokens = tokenize_json(&source);

    rsx! {
        pre { class: "json-source mono",
            for token in tokens {
                span { class: "{token.class_name}", "{token.text}" }
            }
        }
    }
}

#[component]
fn JsonTree(value: serde_json::Value, label: Option<String>) -> Element {
    match value {
        serde_json::Value::Object(map) => {
            let count = map.len();
            rsx! {
                details { class: "tree-node", open: true,
                    summary {
                        JsonLabel { label }
                        span { class: "json-punctuation", "{{" }
                        span { class: "json-meta", "{count} keys" }
                        span { class: "json-punctuation", "}}" }
                    }
                    div { class: "tree-children",
                        for (key, child) in map {
                            JsonTree { value: child, label: Some(key) }
                        }
                    }
                }
            }
        }
        serde_json::Value::Array(items) => {
            let count = items.len();
            rsx! {
                details { class: "tree-node", open: true,
                    summary {
                        JsonLabel { label }
                        span { class: "json-punctuation", "[" }
                        span { class: "json-meta", "{count} items" }
                        span { class: "json-punctuation", "]" }
                    }
                    div { class: "tree-children",
                        for (index, child) in items.into_iter().enumerate() {
                            JsonTree { value: child, label: Some(format!("[{index}]")) }
                        }
                    }
                }
            }
        }
        primitive => {
            let (class_name, text) = json_primitive_text(&primitive);
            rsx! {
                div { class: "tree-leaf",
                    JsonLabel { label }
                    span { class: "{class_name}", "{text}" }
                }
            }
        }
    }
}

#[component]
fn JsonLabel(label: Option<String>) -> Element {
    rsx! {
        if let Some(label) = label {
            span { class: "json-key", "\"{label}\"" }
            span { class: "json-punctuation", ": " }
        }
    }
}

fn time_from_timestamp(input: &str) -> String {
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

fn timestamp_from_local(input: &str) -> String {
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

fn pretty_json(input: &str) -> String {
    match parse_json_value(input) {
        Ok(value) => serde_json::to_string_pretty(&value).unwrap_or_else(|err| err.to_string()),
        Err(err) => format!("JSON 解析失败: {err}"),
    }
}

fn minify_json(input: &str) -> String {
    match parse_json_value(input) {
        Ok(value) => serde_json::to_string(&value).unwrap_or_else(|err| err.to_string()),
        Err(err) => format!("JSON 解析失败: {err}"),
    }
}

fn json_status(input: &str) -> String {
    match parse_json_value(input) {
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

fn parse_json_value(input: &str) -> serde_json::Result<serde_json::Value> {
    serde_json::from_str::<serde_json::Value>(input)
}

#[derive(Clone)]
struct JsonToken {
    class_name: &'static str,
    text: String,
}

fn tokenize_json(source: &str) -> Vec<JsonToken> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut index = 0;

    while index < chars.len() {
        let current = chars[index];
        if current == '"' {
            let start = index;
            index += 1;
            while index < chars.len() {
                if chars[index] == '\\' {
                    index += 2;
                    continue;
                }
                if chars[index] == '"' {
                    index += 1;
                    break;
                }
                index += 1;
            }
            let text: String = chars[start..index].iter().collect();
            let mut next = index;
            while next < chars.len() && chars[next].is_whitespace() {
                next += 1;
            }
            let class_name = if next < chars.len() && chars[next] == ':' {
                "json-key"
            } else {
                "json-string"
            };
            tokens.push(JsonToken { class_name, text });
        } else if current.is_ascii_digit() || current == '-' {
            let start = index;
            index += 1;
            while index < chars.len()
                && (chars[index].is_ascii_digit()
                    || chars[index] == '.'
                    || chars[index] == 'e'
                    || chars[index] == 'E'
                    || chars[index] == '+'
                    || chars[index] == '-')
            {
                index += 1;
            }
            tokens.push(JsonToken {
                class_name: "json-number",
                text: chars[start..index].iter().collect(),
            });
        } else if chars_start_with(&chars, index, "true") {
            tokens.push(JsonToken {
                class_name: "json-bool",
                text: "true".to_string(),
            });
            index += 4;
        } else if chars_start_with(&chars, index, "false") {
            tokens.push(JsonToken {
                class_name: "json-bool",
                text: "false".to_string(),
            });
            index += 5;
        } else if chars_start_with(&chars, index, "null") {
            tokens.push(JsonToken {
                class_name: "json-null",
                text: "null".to_string(),
            });
            index += 4;
        } else if "{}[]:,".contains(current) {
            tokens.push(JsonToken {
                class_name: "json-punctuation",
                text: current.to_string(),
            });
            index += 1;
        } else {
            tokens.push(JsonToken {
                class_name: "json-space",
                text: current.to_string(),
            });
            index += 1;
        }
    }

    tokens
}

fn chars_start_with(chars: &[char], index: usize, word: &str) -> bool {
    word.chars()
        .enumerate()
        .all(|(offset, expected)| chars.get(index + offset) == Some(&expected))
}

fn json_primitive_text(value: &serde_json::Value) -> (&'static str, String) {
    match value {
        serde_json::Value::String(value) => (
            "json-string",
            serde_json::to_string(value).unwrap_or_else(|_| "\"\"".to_string()),
        ),
        serde_json::Value::Number(value) => ("json-number", value.to_string()),
        serde_json::Value::Bool(value) => ("json-bool", value.to_string()),
        serde_json::Value::Null => ("json-null", "null".to_string()),
        serde_json::Value::Array(_) | serde_json::Value::Object(_) => {
            ("json-punctuation", String::new())
        }
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
