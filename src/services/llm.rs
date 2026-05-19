use reqwest::Client;
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::services::env_store::read_system_prompt;

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

pub async fn query_local_llm(llm_url: &str, prompt: &str) -> String {
    let client = Client::new();

    let system_prompt = read_system_prompt();
    let datetime = current_datetime_string();
    let datetime_prefix = format!("Current date and time: {}\n\n", datetime);

    let mut messages = vec![];
    if !system_prompt.trim().is_empty() {
        let full_system = format!("{}{}", datetime_prefix, system_prompt.trim());
        messages.push(json!({"role": "system", "content": full_system}));
    } else {
        messages.push(json!({"role": "system", "content": datetime_prefix.trim()}));
    }
    messages.push(json!({"role": "user", "content": prompt}));

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
            if let Ok(json) = response.json::<serde_json::Value>().await {
                if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
                    return content.to_string();
                }
            }
            "Error: I received a response, but couldn't parse the text.".to_string()
        }
        Err(e) => format!("Error: Failed to reach the local LLM server at {}. ({})", llm_url, e),
    }
}
