use reqwest::Client;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::config::DEFAULT_LLM_URL;
use crate::services::env_store::{read_env_value, read_system_prompt};

const DEFAULT_API_BASE_URL: &str = "https://api.groq.com/openai/v1";
const DEFAULT_API_MODEL: &str = "llama-3.1-8b-instant";

fn current_datetime_string() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;
    let second = time_of_day % 60;

    // Compute year/month/day from days since Unix epoch
    let mut remaining = days_since_epoch;
    let mut year = 1970u64;
    loop {
        let days_in_year = if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) { 366 } else { 365 };
        if remaining < days_in_year { break; }
        remaining -= days_in_year;
        year += 1;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = [31u64, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let month_names = ["January","February","March","April","May","June",
                       "July","August","September","October","November","December"];
    let mut month = 0usize;
    for (i, &d) in month_days.iter().enumerate() {
        if remaining < d { month = i; break; }
        remaining -= d;
    }
    let day = remaining + 1;

    format!(
        "{} {:02}, {} {:02}:{:02}:{:02} UTC",
        month_names[month], day, year, hour, minute, second
    )
}

fn build_messages(prompt: &str) -> Vec<serde_json::Value> {
    let system_prompt = read_system_prompt();
    let datetime = current_datetime_string();

    let mut messages = vec![];
    if !system_prompt.trim().is_empty() {
        messages.push(json!({"role": "system", "content": system_prompt.trim()}));
    }

    // Prepend datetime to the user message for better grounding.
    let user_content = format!("[Current date and time: {}]\n{}", datetime, prompt);
    messages.push(json!({"role": "user", "content": user_content}));
    messages
}

async fn query_openai_compat(api_base_url: &str, api_key: &str, api_model: &str, messages: Vec<serde_json::Value>) -> String {
    let base = api_base_url.trim().trim_end_matches('/');
    if base.is_empty() {
        return "Error: LLM_API_BASE_URL is empty. Set it to your provider base URL (e.g. https://api.openai.com/v1).".to_string();
    }
    if api_model.trim().is_empty() {
        return "Error: LLM_API_MODEL is required when using an API key.".to_string();
    }

    let url = format!("{}/chat/completions", base);
    let payload = json!({
        "model": api_model.trim(),
        "messages": messages,
        "temperature": 0.7,
        "max_tokens": 150
    });

    let client = Client::new();
    let res = client
        .post(url)
        .bearer_auth(api_key.trim())
        .json(&payload)
        .send()
        .await;

    match res {
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
                    return content.to_string();
                }
                if let Some(err_msg) = json["error"]["message"].as_str() {
                    return format!("Error: {}", err_msg);
                }
            }

            format!(
                "Error: I received a response, but couldn't parse the text. (status: {}, body: {})",
                status,
                body
            )
        }
        Err(e) => format!("Error: Failed to reach the LLM API. ({})", e),
    }
}

async fn query_local_llm(llm_url: &str, messages: Vec<serde_json::Value>) -> String {
    let client = Client::new();
    let payload = json!({
        "messages": messages,
        "temperature": 0.7,
        "max_tokens": 150
    });

    let res = client.post(llm_url)
        .json(&payload)
        .send()
        .await;

    match res {
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
                    return content.to_string();
                }
                if let Some(err_msg) = json["error"]["message"].as_str() {
                    return format!("Error: {}", err_msg);
                }
            }

            format!(
                "Error: I received a response, but couldn't parse the text. (status: {}, body: {})",
                status,
                body
            )
        }
        Err(e) => format!("Error: Failed to reach the local LLM server at {}. ({})", llm_url, e),
    }
}

pub async fn query_llm(prompt: &str) -> String {
    let api_key = read_env_value("LLM_API_KEY").unwrap_or_default();
    let messages = build_messages(prompt);

    if !api_key.trim().is_empty() {
        let api_base_url = read_env_value("LLM_API_BASE_URL")
            .unwrap_or_else(|| DEFAULT_API_BASE_URL.to_string());
        let api_model = read_env_value("LLM_API_MODEL")
            .unwrap_or_else(|| DEFAULT_API_MODEL.to_string());
        let api_base_url = if api_base_url.trim().is_empty() {
            DEFAULT_API_BASE_URL.to_string()
        } else {
            api_base_url
        };
        let api_model = if api_model.trim().is_empty() {
            DEFAULT_API_MODEL.to_string()
        } else {
            api_model
        };
        let reply = query_openai_compat(&api_base_url, &api_key, &api_model, messages).await;
        println!("LLM reply: {}", reply);
        return reply;
    }

    let llm_url = read_env_value("LLM_URL").unwrap_or_else(|| DEFAULT_LLM_URL.to_string());
    let reply = query_local_llm(&llm_url, messages).await;
    println!("LLM reply: {}", reply);
    reply
}
