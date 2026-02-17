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
            // Translate text to a structured command
            let command = SystemCommand::from_str(text);

            // Dispatch the command to get a result
            let response = dispatcher::dispatch(command).await;

            println!("Response to send (len: {}): {:?}", response.len(), response);

            // Send the result back to Telegram
            // Telegram has a message limit (approx 4096 chars). We'll split it safely.
            const MAX_LEN: usize = 4000;
            if response.len() <= MAX_LEN {
                bot.send_message(msg.chat.id, response).await?;
            } else {
                let mut start = 0;
                while start < response.len() {
                    let end = std::cmp::min(start + MAX_LEN, response.len());
                    let chunk = &response[start..end];
                    bot.send_message(msg.chat.id, chunk).await?;
                    start = end;
                }
            }
        }
        Ok(())
    })
    .await;
}
