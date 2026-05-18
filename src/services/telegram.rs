use teloxide::prelude::*;

use crate::services::llm::query_local_llm;

pub async fn run_telegram_bot(token: String, llm_url: String) {
    let bot = Bot::new(token);
    println!("✅ Telegram bot started");

    teloxide::repl(bot, move |message: Message, bot: Bot| {
        let llm_url = llm_url.clone();
        async move {
            if let Some(text) = message.text() {
                let reply = query_local_llm(&llm_url, text).await;
                bot.send_message(message.chat.id, reply).await?;
            }
            respond(())
        }
    })
    .await;
}
