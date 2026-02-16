use std::env;
use teloxide::prelude::*;

// Function to start the Telegram bot with basic authorization
pub async fn start_bot() {
    let bot = Bot::from_env();

    // Retrieve the admin ID from environment variables
    // We parse it to i64 to compare it with the chat ID
    let admin_id: i64 = env::var("ADMIN_ID")
        .expect("ADMIN_ID must be set")
        .parse()
        .expect("ADMIN_ID must be a valid integer");

    println!(
        "PocketSentinel is online. Only accepting messages from ID: {}",
        admin_id
    );

    // Start the REPL (Read-Eval-Print Loop)
    teloxide::repl(bot, move |bot: Bot, msg: Message| async move {
        // Verification: Only respond if the message comes from the admin
        if msg.chat.id.0 != admin_id {
            // Log unauthorized attempt
            println!("Unauthorized access attempt from ID: {}", msg.chat.id);
            return Ok(());
        }

        if let Some(text) = msg.text() {
            println!("Authorized command received: {}", text);

            // For now, we just acknowledge the message
            let response_text = format!("Authenticated: Processing '{}'...", text);
            bot.send_message(msg.chat.id, response_text).await?;
        }

        Ok(())
    })
    .await;
}
