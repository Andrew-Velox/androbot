use reqwest::Client;
use serde_json::json;
use teloxide::{prelude::*, utils::command::BotCommands};

#[tokio::main]
async fn main() {
    println!("🚀 Starting Telegram AI Bridge...");
    
    // Initialize the bot from the TELEGRAM_BOT_TOKEN environment variable
    let bot = Bot::from_env();
    
    println!("✅ Bot is online and listening for messages!");
    // Start the long-polling loop
    teloxide::repl(bot, handle_message).await;
}

async fn handle_message(bot: Bot, msg: Message) -> ResponseResult<()> {
    // We only process text messages for now
    if let Some(text) = msg.text() {
        println!("Received message: {}", text);
        
        // Show the "typing..." action in the Telegram chat
        bot.send_chat_action(msg.chat.id, teloxide::types::ChatAction::Typing).await?;

        // Forward the text to the local Qwen model
        let ai_response = query_local_llm(text).await;

        // Send the AI's response back to the user
        bot.send_message(msg.chat.id, ai_response).await?;
    }
    Ok(())
}

async fn query_local_llm(prompt: &str) -> String {
    let client = Client::new();
    
    // Construct the exact JSON payload expected by llama-server
    let payload = json!({
        "messages": [
            {"role": "system", "content": "You are a helpful and concise AI assistant running on a mobile device."},
            {"role": "user", "content": prompt}
        ],
        "temperature": 0.7,
        "max_tokens": 150
    });

    // Hit the local endpoint
    let res = client.post("http://127.0.0.1:8080/v1/chat/completions")
        .json(&payload)
        .send()
        .await;

    // Parse the JSON response just like we saw in the curl output
    match res {
        Ok(response) => {
            if let Ok(json) = response.json::<serde_json::Value>().await {
                if let Some(content) = json["choices"][0]["message"]["content"].as_str() {
                    return content.to_string();
                }
            }
            "Error: I received a response, but couldn't parse the text.".to_string()
        }
        Err(e) => format!("Error: Failed to reach the local LLM server. Is it running? ({})", e),
    }
}