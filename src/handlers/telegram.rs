use crate::core::dispatcher;
use crate::models::command::SystemCommand;
use std::env;
use teloxide::prelude::*;

pub async fn start_bot(pool: crate::db::DbPool) {
    let bot = Bot::from_env();

    let admin_id: i64 = env::var("ADMIN_ID")
        .expect("ADMIN_ID must be set")
        .parse()
        .expect("ADMIN_ID must be a valid integer");

    println!(
        "PocketSentinel is online. Only accepting messages from ID: {}",
        admin_id
    );

    let dispatcher_pool = pool.clone();
    teloxide::repl(bot, move |bot: Bot, msg: Message| {
        let pool = dispatcher_pool.clone();
        async move {
            // Security check
            if msg.chat.id.0 != admin_id {
                return Ok(());
            }

            if let Some(text) = msg.text() {
                // Translate text to a structured command
                let command = SystemCommand::from_str(text);

                // Dispatch the command to get a result
                let (response, is_html) = dispatcher::dispatch(command, pool).await;

                println!("Response to send (len: {}): {:?}", response.len(), response);

                use teloxide::types::ParseMode;
                // Send the result back to Telegram
                // Telegram has a message limit (approx 4096 chars). We'll split it safely.
                const MAX_LEN: usize = 4000;

                let parse_mode = if is_html { Some(ParseMode::Html) } else { None };

                if response.len() <= MAX_LEN {
                    let mut req = bot.send_message(msg.chat.id, response);
                    if let Some(pm) = parse_mode {
                        req = req.parse_mode(pm);
                    }

                    if let Err(e) = req.await {
                        eprintln!("Failed to send message: {}", e);
                        if is_html {
                            let _ = bot
                                .send_message(msg.chat.id, "Error sending HTML message.")
                                .await;
                        }
                    }
                } else {
                    let mut start = 0;
                    while start < response.len() {
                        let end = std::cmp::min(start + MAX_LEN, response.len());
                        let chunk = &response[start..end];

                        let mut req = bot.send_message(msg.chat.id, chunk);
                        if let Some(pm) = parse_mode {
                            req = req.parse_mode(pm);
                        }

                        if let Err(e) = req.await {
                            eprintln!("Failed to send chunk: {}", e);
                        }

                        start = end;
                    }
                }
            }
            Ok(())
        }
    })
    .await;
}
