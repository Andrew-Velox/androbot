use axum::{
    routing::get,
    Router, extract::{Query, State}, response::IntoResponse,
};
use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};
use dotenvy::dotenv;
use std::env;

#[derive(Clone)]
struct AppState {
    verify_token: String,
    llm_url: String,
}

// Meta verification struct
#[derive(Deserialize)]
struct VerifyQuery {
    #[serde(rename = "hub.mode")]
    mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    token: Option<String>,
    #[serde(rename = "hub.challenge")]
    challenge: Option<String>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    println!("🚀 Starting Meta Webhook Server & Local LLM Bridge...");


    let verify_token = env::var("VERIFY_TOKEN")
        .unwrap_or_else(|_| "ANDROBOT_SECURE_TOKEN_123".to_string());
    if verify_token == "ANDROBOT_SECURE_TOKEN_123" {
        println!("⚠️  VERIFY_TOKEN not set; using insecure default.");
    }

    let llm_url = env::var("LLM_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8080/v1/chat/completions".to_string());
    let bind_addr = env::var("BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    let state = AppState {
        verify_token,
        llm_url,
    };

    // Build the axum router
    let app = Router::new()
        .route("/webhook", get(verify_webhook).post(handle_incoming_message))
        .with_state(state);

    // Bind to IPv4 by default; use BIND_ADDR env var to override
    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    println!("✅ Listening for Meta on {}", bind_addr);
    
    axum::serve(listener, app).await.unwrap();
}

// 1. Meta Verification Endpoint
async fn verify_webhook(
    State(state): State<AppState>,
    Query(params): Query<VerifyQuery>,
) -> impl IntoResponse {
    if let (Some(mode), Some(token), Some(challenge)) = (params.mode, params.token, params.challenge) {
        if mode == "subscribe" && token == state.verify_token {
            println!("✅ Webhook Verified by Meta!");
            return challenge;
        }
    }
    "Forbidden".to_string()
}

// 2. Incoming Message Endpoint (Updated to capture raw body strings)
async fn handle_incoming_message(
    State(state): State<AppState>,
    body: String,
) -> impl axum::response::IntoResponse {
    println!("🚨 META POST RECEIVED!");
    println!("Body content: {}", body);

    let payload: Value = match serde_json::from_str(&body) {
        Ok(parsed) => parsed,
        Err(err) => {
            println!("Failed to parse webhook JSON: {}", err);
            return "EVENT_RECEIVED";
        }
    };

    if payload.get("object").and_then(Value::as_str) != Some("page") {
        return "EVENT_RECEIVED";
    }

    let entries = payload.get("entry").and_then(Value::as_array).cloned().unwrap_or_default();
    for entry in entries {
        let messaging_events = entry.get("messaging").and_then(Value::as_array).cloned().unwrap_or_default();
        for event in messaging_events {
            let sender_id = event.get("sender").and_then(|s| s.get("id")).and_then(Value::as_str);
            let message = event.get("message");

            if message.and_then(|m| m.get("is_echo")).and_then(Value::as_bool) == Some(true) {
                continue;
            }

            let text = message.and_then(|m| m.get("text")).and_then(Value::as_str);

            if let (Some(sender_id), Some(text)) = (sender_id, text) {
                println!("Incoming text from {}: {}", sender_id, text);
                let reply = query_local_llm(&state.llm_url, text).await;
                if let Err(err) = send_facebook_message(sender_id, &reply).await {
                    println!("Failed to send reply: {}", err);
                }
            }
        }
    }

    "EVENT_RECEIVED"
}

// 3. Your exact LLM logic connecting to the mobile device
async fn query_local_llm(llm_url: &str, prompt: &str) -> String {
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

async fn send_facebook_message(recipient_id: &str, text: &str) -> Result<(), String> {
    let token = env::var("PAGE_ACCESS_TOKEN")
        .map_err(|_| "Missing PAGE_ACCESS_TOKEN env var".to_string())?;

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