use reqwest::Client;
use serde_json::json;

pub async fn query_local_llm(llm_url: &str, prompt: &str) -> String {
    let client = Client::new();

    let payload = json!({
        "messages": [
            {"role": "system", "content": "You are a helpful and concise AI assistant running on a mobile device."},
            {"role": "user", "content": prompt}
        ],
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
