use crate::ai::config::OllamaConfig; // Added for callback_handler
use crate::core::dispatcher;
use crate::models::command::SystemCommand;
use crate::models::CommandResponse; // Ensure this is imported
use std::env;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};

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

    let handler = dptree::entry()
        .branch(Update::filter_message().endpoint(message_handler))
        .branch(Update::filter_callback_query().endpoint(callback_handler));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![pool, admin_id])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn message_handler(
    bot: Bot,
    msg: Message,
    pool: crate::db::DbPool,
    admin_id: i64,
) -> ResponseResult<()> {
    // Security check
    if msg.chat.id.0 != admin_id {
        return Ok(());
    }

    if let Some(text) = msg.text() {
        let command = SystemCommand::from_str(text);
        let response = dispatcher::dispatch(command, pool).await;

        match response {
            CommandResponse::Text(text) => {
                send_long_message(&bot, msg.chat.id, text, None).await?;
            }
            CommandResponse::Html(html) => {
                if let Err(_) =
                    send_long_message(&bot, msg.chat.id, html.clone(), Some(ParseMode::Html)).await
                {
                    let _ = bot
                        .send_message(
                            msg.chat.id,
                            "Error sending HTML message. sending plain text.",
                        )
                        .await;
                    send_long_message(&bot, msg.chat.id, html, None).await?;
                }
            }
            CommandResponse::InteractiveList {
                title,
                options,
                callback_prefix,
            } => {
                let buttons: Vec<Vec<InlineKeyboardButton>> = options
                    .chunks(1) // 1 button per row
                    .map(|chunk| {
                        chunk
                            .iter()
                            .map(|opt| {
                                InlineKeyboardButton::callback(
                                    opt.clone(),
                                    format!("{}{}", callback_prefix, opt),
                                )
                            })
                            .collect()
                    })
                    .collect();

                let keyboard = InlineKeyboardMarkup::new(buttons);
                bot.send_message(msg.chat.id, title)
                    .reply_markup(keyboard)
                    .await?;
            }
        }
    }
    Ok(())
}

async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
    pool: crate::db::DbPool,
) -> ResponseResult<()> {
    if let Some(data) = q.data {
        if let Some(model) = data.strip_prefix("set_model:") {
            // Update Ollama config
            let mut config = OllamaConfig::load();
            config.model = model.to_string();
            let result_msg = match config.save() {
                Ok(_) => format!("Model changed to: {}", model),
                Err(e) => format!("Failed to change model: {}", e),
            };

            bot.answer_callback_query(q.id).await?;
            if let Some(msg) = q.message {
                let chat_id = msg.chat().id;
                bot.send_message(chat_id, result_msg).await?;
            }
        } else if let Some(alias) = data.strip_prefix("menu_server:") {
            // Show server actions
            let buttons = vec![
                vec![InlineKeyboardButton::callback(
                    "🔍 Discover",
                    format!("act_discover:{}", alias),
                )],
                vec![InlineKeyboardButton::callback(
                    "🗑️ Remove",
                    format!("act_remove:{}", alias),
                )],
            ];
            let keyboard = InlineKeyboardMarkup::new(buttons);

            bot.answer_callback_query(q.id).await?;
            if let Some(msg) = q.message {
                let chat_id = msg.chat().id;
                bot.send_message(chat_id, format!("Actions for server: {}", alias))
                    .reply_markup(keyboard)
                    .await?;
            }
        } else if let Some(alias) = data.strip_prefix("act_discover:") {
            // Trigger discovery
            // Extract chat_id first because we need it for async calls
            let chat_id = if let Some(msg) = &q.message {
                Some(msg.chat().id)
            } else {
                None
            };

            if let Some(cid) = chat_id {
                bot.answer_callback_query(q.id)
                    .text("Starting discovery...")
                    .await?;
                bot.send_message(cid, format!("Starting discovery on {}...", alias))
                    .await?;

                let command = SystemCommand::Discover {
                    alias: alias.to_string(),
                };
                let response = dispatcher::dispatch(command, pool.clone()).await;

                match response {
                    CommandResponse::Text(text) => {
                        send_long_message(&bot, cid, text, None).await?;
                    }
                    CommandResponse::Html(html) => {
                        send_long_message(&bot, cid, html, Some(ParseMode::Html)).await?;
                    }
                    _ => {}
                }
            } else {
                bot.answer_callback_query(q.id).await?;
            }
        } else if let Some(alias) = data.strip_prefix("act_remove:") {
            let chat_id = if let Some(msg) = &q.message {
                Some(msg.chat().id)
            } else {
                None
            };

            if let Some(cid) = chat_id {
                bot.answer_callback_query(q.id)
                    .text("Removing server...")
                    .await?;
                let command = SystemCommand::RemoveServer {
                    alias: alias.to_string(),
                };
                let response = dispatcher::dispatch(command, pool.clone()).await;

                match response {
                    CommandResponse::Text(text) => {
                        bot.send_message(cid, text).await?;
                    }
                    _ => {}
                }
            } else {
                bot.answer_callback_query(q.id).await?;
            }
        }
    }
    Ok(())
}

async fn send_long_message(
    bot: &Bot,
    chat_id: ChatId,
    text: String,
    parse_mode: Option<ParseMode>,
) -> ResponseResult<()> {
    const MAX_LEN: usize = 4000;
    if text.len() <= MAX_LEN {
        let mut req = bot.send_message(chat_id, text);
        if let Some(pm) = parse_mode {
            req = req.parse_mode(pm);
        }
        req.await?;
    } else {
        let mut start = 0;
        while start < text.len() {
            let end = std::cmp::min(start + MAX_LEN, text.len());
            let chunk = &text[start..end];
            let mut req = bot.send_message(chat_id, chunk);
            if let Some(pm) = parse_mode {
                req = req.parse_mode(pm);
            }
            req.await?;
            start = end;
        }
    }
    Ok(())
}
