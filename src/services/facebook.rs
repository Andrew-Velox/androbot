use reqwest::Client;
use serde_json::json;

use crate::services::env_store::read_env_value;

pub async fn send_facebook_message(recipient_id: &str, text: &str) -> Result<(), String> {
    let token = read_env_value("PAGE_ACCESS_TOKEN")
        .ok_or_else(|| "Missing PAGE_ACCESS_TOKEN".to_string())?;

    let client = Client::new();
    let url = format!(
        "https://graph.facebook.com/v25.0/me/messages?access_token={}",
        token
    );

    let payload = json!({
        "recipient": {"id": recipient_id},
        "message": {"text": text}
    });

    let response = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|err| err.to_string())?;

    if response.status().is_success() {
        Ok(())
    } else {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        Err(format!("Graph API error ({}): {}", status, body))
    }
}
