use reqwest::Client;
use serde_json::json;

use crate::services::env_store::read_system_prompt;

pub async fn query_local_llm(llm_url: &str, prompt: &str) -> String {
    let client = Client::new();

    let system_prompt = read_system_prompt();
    let mut messages = vec![];
    if !system_prompt.trim().is_empty() {
        messages.push(json!({"role": "system", "content": system_prompt.trim()}));
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
