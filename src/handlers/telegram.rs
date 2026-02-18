use crate::ai::config::OllamaConfig;
use crate::core::dispatcher;
use crate::core::session::SessionManager;
use crate::models::command::SystemCommand;
use crate::models::CommandResponse;
use base64::prelude::*;
use std::env;
use teloxide::prelude::*;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardMarkup, ParseMode};

pub async fn start_bot(pool: crate::db::DbPool, session_manager: SessionManager) {
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
        .dependencies(dptree::deps![pool, admin_id, session_manager])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

/// Handles incoming messages from the user.
async fn message_handler(
    bot: Bot,
    msg: Message,
    pool: crate::db::DbPool,
    session_manager: SessionManager,
    admin_id: i64,
) -> ResponseResult<()> {
    // Security check
    if msg.chat.id.0 != admin_id {
        return Ok(());
    }

    if let Some(command) = extract_command(&msg) {
        let response = dispatcher::dispatch(
            msg.chat.id.0,
            command,
            pool.clone(),
            session_manager.clone(),
        )
        .await;

        handle_command_response(&bot, msg.chat.id, response).await?;
    }
    Ok(())
}

/// Extracts a `SystemCommand` from a message, handling replies for configuration.
fn extract_command(msg: &Message) -> Option<SystemCommand> {
    let text = msg.text()?;

    // Check if this is a reply to an API configuration request
    if let Some(reply) = msg.reply_to_message() {
        if let Some(reply_text) = reply.text() {
            if reply_text.contains("API Key for") {
                if let Some(start) = reply_text.find("for ") {
                    let provider_part = &reply_text[start + 4..];
                    let provider = provider_part.trim_end_matches('.').trim();
                    // Clean up any remaining HTML tags just in case
                    let provider = provider.replace("<b>", "").replace("</b>", "");

                    let encoded_key = BASE64_STANDARD.encode(text.trim());

                    return Some(SystemCommand::SetApiKey {
                        provider,
                        key: encoded_key,
                    });
                }
            }
        }
    }

    Some(SystemCommand::from_str(text))
}

/// Handles callback queries (button clicks).
async fn callback_handler(
    bot: Bot,
    q: CallbackQuery,
    pool: crate::db::DbPool,
    session_manager: SessionManager,
) -> ResponseResult<()> {
    let data = match q.data.clone() {
        Some(d) => d,
        None => return Ok(()),
    };

    if let Some(model) = data.strip_prefix("set_model:") {
        handle_set_model(bot, q, pool, model).await
    } else if let Some(alias) = data.strip_prefix("menu_server:") {
        handle_menu_server(bot, q, alias).await
    } else if let Some(provider) = data.strip_prefix("set_provider:") {
        handle_set_provider(bot, q, pool, session_manager, provider).await
    } else if let Some(provider) = data.strip_prefix("config_key_provider:") {
        handle_config_key_provider(bot, q, session_manager, provider).await
    } else if let Some(alias) = data.strip_prefix("act_discover:") {
        handle_action_discover(bot, q, pool, session_manager, alias).await
    } else if let Some(alias) = data.strip_prefix("act_remove:") {
        handle_action_remove(bot, q, pool, session_manager, alias).await
    } else if let Some(rest) = data.strip_prefix("tool_run:") {
        handle_tool_run(bot, q, session_manager, rest).await
    } else {
        Ok(())
    }
}

// --- Specific Callback Handlers ---

async fn handle_set_model(
    bot: Bot,
    q: CallbackQuery,
    pool: crate::db::DbPool,
    model: &str,
) -> ResponseResult<()> {
    let mut config = OllamaConfig::load(&pool).await;
    config.model = model.to_string();
    let result_msg = match config.save(&pool).await {
        Ok(_) => format!("Model changed to: {}", model),
        Err(e) => format!("Failed to change model: {}", e),
    };

    bot.answer_callback_query(q.id).await?;
    if let Some(msg) = q.message {
        bot.send_message(msg.chat().id, result_msg).await?;
    }
    Ok(())
}

async fn handle_menu_server(bot: Bot, q: CallbackQuery, alias: &str) -> ResponseResult<()> {
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
        bot.send_message(msg.chat().id, format!("Actions for server: {}", alias))
            .reply_markup(keyboard)
            .await?;
    }
    Ok(())
}

async fn handle_set_provider(
    bot: Bot,
    q: CallbackQuery,
    pool: crate::db::DbPool,
    session_manager: SessionManager,
    provider: &str,
) -> ResponseResult<()> {
    let chat_id = match q.message {
        Some(ref msg) => msg.chat().id,
        None => return Ok(()),
    };

    let command = SystemCommand::SetProvider {
        provider: Some(provider.to_string()),
    };
    let response = dispatcher::dispatch(chat_id.0, command, pool, session_manager).await;

    if let CommandResponse::Text(text) = response {
        bot.answer_callback_query(q.id).text(text.clone()).await?;
        bot.send_message(chat_id, text).await?;
    }
    Ok(())
}

async fn handle_config_key_provider(
    bot: Bot,
    q: CallbackQuery,
    _session_manager: SessionManager,
    provider: &str,
) -> ResponseResult<()> {
    let chat_id = match q.message {
        Some(ref msg) => msg.chat().id,
        None => return Ok(()),
    };

    bot.answer_callback_query(q.id).await?;
    bot.send_message(
        chat_id,
        format!(
            "Please reply to this message with your API Key for <b>{}</b>.",
            provider
        ),
    )
    .parse_mode(ParseMode::Html)
    .reply_markup(teloxide::types::ForceReply::new().selective())
    .await?;

    Ok(())
}

async fn handle_action_discover(
    bot: Bot,
    q: CallbackQuery,
    pool: crate::db::DbPool,
    session_manager: SessionManager,
    alias: &str,
) -> ResponseResult<()> {
    let chat_id = match q.message {
        Some(ref msg) => msg.chat().id,
        None => return Ok(()),
    };

    bot.answer_callback_query(q.id)
        .text("Starting discovery...")
        .await?;
    bot.send_message(chat_id, format!("Starting discovery on {}...", alias))
        .await?;

    let command = SystemCommand::Discover {
        alias: alias.to_string(),
    };
    let response = dispatcher::dispatch(chat_id.0, command, pool, session_manager).await;
    handle_command_response(&bot, chat_id, response).await
}

async fn handle_action_remove(
    bot: Bot,
    q: CallbackQuery,
    pool: crate::db::DbPool,
    session_manager: SessionManager,
    alias: &str,
) -> ResponseResult<()> {
    let chat_id = match q.message {
        Some(ref msg) => msg.chat().id,
        None => return Ok(()),
    };

    bot.answer_callback_query(q.id)
        .text("Removing server...")
        .await?;
    let command = SystemCommand::RemoveServer {
        alias: alias.to_string(),
    };
    let response = dispatcher::dispatch(chat_id.0, command, pool, session_manager).await;
    handle_command_response(&bot, chat_id, response).await
}

async fn handle_tool_run(
    bot: Bot,
    q: CallbackQuery,
    session_manager: SessionManager,
    rest: &str,
) -> ResponseResult<()> {
    let (encoded, action) = match rest.split_once(':') {
        Some(pair) => pair,
        None => return Ok(()),
    };

    // Case 1: Cancel / Skip
    if action != "✅ Run" && action != "Confirm" && action != "Execute" {
        bot.answer_callback_query(q.id).text("Cancelled").await?;
        if let Some(msg) = q.message {
            bot.send_message(msg.chat().id, "Command execution skipped.")
                .await?;
            session_manager
                .add_message(msg.chat().id.0, "user", "I skipped the command execution.")
                .await;
        }
        return Ok(());
    }

    // Case 2: Run - Decode Command
    let cmd_vec = match BASE64_STANDARD.decode(encoded) {
        Ok(v) => v,
        Err(_) => {
            bot.answer_callback_query(q.id).text("Decode error").await?;
            return Ok(());
        }
    };

    let cmd = match String::from_utf8(cmd_vec) {
        Ok(s) => s,
        Err(_) => {
            bot.answer_callback_query(q.id)
                .text("Invalid command encoding")
                .await?;
            return Ok(());
        }
    };

    // Execute Command
    bot.answer_callback_query(q.id)
        .text(format!("Running: {}", cmd))
        .await?;

    if let Some(msg) = q.message {
        let chat_id = msg.chat().id;
        bot.send_message(chat_id, format!("⏳ Executing: `{}`", cmd))
            .await?;

        let response = session_manager.execute_tool_command(chat_id.0, &cmd).await;
        handle_command_response(&bot, chat_id, response).await?;
    }

    Ok(())
}

// --- Response Helpers ---

async fn handle_command_response(
    bot: &Bot,
    chat_id: ChatId,
    response: CommandResponse,
) -> ResponseResult<()> {
    match response {
        CommandResponse::Text(text) => {
            send_long_message(bot, chat_id, text, None).await?;
        }
        CommandResponse::Html(html) => {
            if let Err(_) =
                send_long_message(bot, chat_id, html.clone(), Some(ParseMode::Html)).await
            {
                let _ = bot
                    .send_message(chat_id, "Error sending HTML message. sending plain text.")
                    .await;
                send_long_message(bot, chat_id, html, None).await?;
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
            bot.send_message(chat_id, title)
                .parse_mode(ParseMode::Html)
                .reply_markup(keyboard)
                .await?;
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
