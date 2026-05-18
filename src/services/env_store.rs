use std::fs;

use crate::config::{DEFAULT_BIND_ADDR, DEFAULT_LLM_URL};

pub fn read_env_value(key: &str) -> Option<String> {
    let content = fs::read_to_string(".env").ok()?;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim() == key {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

pub fn write_env_file(
    page_access_token: &str,
    telegram_bot_token: Option<&str>,
    llm_url: &str,
    bind_addr: &str,
) -> std::io::Result<()> {
    let mut content = String::new();
    content.push_str(&format!("PAGE_ACCESS_TOKEN={}\n", page_access_token));
    if let Some(token) = telegram_bot_token {
        if !token.trim().is_empty() {
            content.push_str(&format!("TELEGRAM_BOT_TOKEN={}\n", token.trim()));
        }
    }
    let llm_url = if llm_url.is_empty() {
        DEFAULT_LLM_URL
    } else {
        llm_url
    };
    let bind_addr = if bind_addr.is_empty() {
        DEFAULT_BIND_ADDR
    } else {
        bind_addr
    };
    content.push_str(&format!("LLM_URL={}\n", llm_url));
    content.push_str(&format!("BIND_ADDR={}\n", bind_addr));

    fs::write(".env", content)
}
