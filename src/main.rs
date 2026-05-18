use axum::{
    routing::get,
    Router,
};
use dotenvy::dotenv;
use std::env;

mod config;
mod routes;
mod services;

use config::{DEFAULT_BIND_ADDR, DEFAULT_LLM_URL, SETUP_URL};
use services::telegram::run_telegram_bot;

#[derive(Clone)]
pub struct AppState {
    pub llm_url: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    println!("🚀 Starting Meta Webhook Server & Local LLM Bridge...");


    let llm_url = env::var("LLM_URL")
        .unwrap_or_else(|_| DEFAULT_LLM_URL.to_string());
    let bind_addr = env::var("BIND_ADDR")
        .unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_string());

    let state = AppState { llm_url };

    if let Ok(telegram_token) = env::var("TELEGRAM_BOT_TOKEN") {
        let llm_url = state.llm_url.clone();
        tokio::spawn(async move {
            run_telegram_bot(telegram_token, llm_url).await;
        });
    } else {
        println!("ℹ️ Telegram bot disabled (TELEGRAM_BOT_TOKEN not set).")
    }

    // Build the axum router
    let app = Router::new()
        .route("/webhook", get(routes::webhook::verify_webhook).post(routes::webhook::handle_incoming_message))
        .route("/setup", get(routes::setup::setup_page).post(routes::setup::save_setup))
        .with_state(state);

    // Bind to IPv4 by default; use BIND_ADDR env var to override
    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    println!("✅ Listening for Meta on {}", bind_addr);
    println!("🛠 Setup page: {}", SETUP_URL);
    
    axum::serve(listener, app).await.unwrap();
}