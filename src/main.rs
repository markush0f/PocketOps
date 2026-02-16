use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    println!("Hello, world!");
    pretty_env_logger::init();
    log::info!("Starting bot...");

    // Get token from environment variable
    let bot = Bot::from_env();

    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
         bot.send_message(msg.chat.id, format!("Echo: {}", msg.text().unwrap_or("")))
            .await?;
        Ok(())
    }).await;
}
