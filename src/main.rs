use axum::{
    routing::get,
    Router,
};
use dotenvy::dotenv;
use std::{env, sync::Arc};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

mod config;
mod routes;
mod services;

use config::{DEFAULT_BIND_ADDR, SETUP_URL};
use services::telegram::{run_telegram_bot, validate_telegram_token};

#[derive(Clone)]
pub struct AppState {
    pub telegram_task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let bind_addr = env::var("BIND_ADDR")
        .unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_string());

    let telegram_task: Arc<Mutex<Option<JoinHandle<()>>>> = Arc::new(Mutex::new(None));

    let tg_status = if let Ok(token) = env::var("TELEGRAM_BOT_TOKEN") {
        if validate_telegram_token(&token).await {
            let handle = tokio::spawn(async move {
                run_telegram_bot(token).await;
            });
            *telegram_task.lock().await = Some(handle);
            "✅ running"
        } else {
            "⚠️  invalid token"
        }
    } else {
        "—  not configured"
    };

    let state = AppState { telegram_task };

    let app = Router::new()
        .route("/webhook", get(routes::webhook::verify_webhook).post(routes::webhook::handle_incoming_message))
        .route("/setup", get(routes::setup::setup_page).post(routes::setup::save_setup))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    println!("🚀 Androbot running");
    println!("   Server:   {}", bind_addr);
    println!("   Setup:    {}", SETUP_URL);
    println!("   Telegram: {}", tg_status);

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.ok();
            println!("Shutting down...");
        })
        .await
        .unwrap();
}