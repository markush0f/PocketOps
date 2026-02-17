use crate::core::dispatcher;
use crate::models::command::SystemCommand;
use std::env;
use teloxide::prelude::*;

pub async fn start_bot() {
    let bot = Bot::from_env();

    let admin_id: i64 = env::var("ADMIN_ID")
        .expect("ADMIN_ID must be set")
        .parse()
        .expect("ADMIN_ID must be a valid integer");

    println!(
        "PocketSentinel is online. Only accepting messages from ID: {}",
        admin_id
    );

    teloxide::repl(bot, move |bot: Bot, msg: Message| async move {
        // Security check
        if msg.chat.id.0 != admin_id {
            return Ok(());
        }

        if let Some(text) = msg.text() {
            // 1. Translate text to a structured command
            let command = SystemCommand::from_str(text);

            // 2. Dispatch the command to get a result
            let response = dispatcher::dispatch(command).await;

            // 3. Send the result back to Telegram
            bot.send_message(msg.chat.id, response).await?;
        }
        Ok(())
    })
    .await;
}
