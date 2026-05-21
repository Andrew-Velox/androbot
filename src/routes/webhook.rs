use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use serde_json::Value;

use crate::{AppState, config::VERIFY_TOKEN};
use crate::services::{facebook::send_facebook_message, llm::query_llm};

// Meta verification struct
#[derive(Deserialize)]
pub struct VerifyQuery {
    #[serde(rename = "hub.mode")]
    mode: Option<String>,
    #[serde(rename = "hub.verify_token")]
    token: Option<String>,
    #[serde(rename = "hub.challenge")]
    challenge: Option<String>,
}

// 1. Meta Verification Endpoint
pub async fn verify_webhook(Query(params): Query<VerifyQuery>) -> impl IntoResponse {
    if let (Some(mode), Some(token), Some(challenge)) =
        (params.mode, params.token, params.challenge)
        && mode == "subscribe"
        && token == VERIFY_TOKEN
    {
        println!("✅ Webhook Verified by Meta!");
        return challenge;
    }
    "Forbidden".to_string()
}

// 2. Incoming Message Endpoint
pub async fn handle_incoming_message(
    State(_state): State<AppState>,
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
                let reply = query_llm(text).await;
                if let Err(err) = send_facebook_message(sender_id, &reply).await {
                    println!("Failed to send reply: {}", err);
                }
            }
        }
    }

    "EVENT_RECEIVED"
}
