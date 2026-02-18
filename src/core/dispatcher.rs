use crate::ai::client::AiClient;
use crate::core::server_manager::ServerManager;
use crate::executor::ssh::SshExecutor;
use crate::models::command::SystemCommand;
use crate::models::CommandResponse;

/// Dispatches a `SystemCommand` to the appropriate handler.
///
/// This function acts as the central controller for the application. It:
/// 1. Initializes necessary managers (`ServerManager`, `AiClient`).
/// 2. Matches the incoming command.
/// 3. Executes the corresponding logic (Server management, SSH execution, AI interaction).
/// 4. Returns a specialized `CommandResponse`.
///
/// # Arguments
///
/// * `command` - The parsed system command to execute.
/// * `pool` - The database connection pool.
///
/// # Returns
///
/// A `CommandResponse` ready to be handled by the interface layer (e.g., Telegram).
/// Dispatches a `SystemCommand` to the appropriate handler.
///
/// # Arguments
///
/// * `chat_id` - The ID of the user issuing the command.
/// * `command` - The command to execute.
/// * `pool` - The database connection pool.
/// * `session_manager` - The session manager.
///
/// # Returns
///
/// A `CommandResponse` enum containing the result.
pub async fn dispatch(
    chat_id: i64,
    command: SystemCommand,
    pool: crate::db::DbPool,
    session_manager: crate::core::session::SessionManager,
) -> CommandResponse {
    let manager = ServerManager::new(pool.clone());
    let ai_client = AiClient::new();

    // Log the command to audit_logs (best effort, ignore error)
    if let SystemCommand::Unknown = command {
        // Skip logging unknown commands as they might just be chat noise
    } else {
        let cmd_str = format!("{:?} (User: {})", command, chat_id);
        let _ = sqlx::query("INSERT INTO audit_logs (command) VALUES (?)")
            .bind(&cmd_str)
            .execute(&pool)
            .await;
    }

    match command {
        SystemCommand::Investigate { alias: _ } => CommandResponse::Text(
            "Use /ask <question> instead. Example: /ask investigate local".to_string(),
        ),

        SystemCommand::EndSession => {
            if session_manager.end_session(chat_id).is_some() {
                CommandResponse::Text("Session ended. Returning to normal mode.".to_string())
            } else {
                CommandResponse::Text("No active session to end.".to_string())
            }
        }

        SystemCommand::GetStatus => CommandResponse::Text("System status: Operational".to_string()),

        SystemCommand::Help => {
            let mut help_msg = "Available commands:\n".to_string();
            for (cmd, desc) in SystemCommand::all_commands_info() {
                help_msg.push_str(&format!("  {} - {}\n", cmd, desc));
            }
            CommandResponse::Text(help_msg)
        }

        SystemCommand::AddServer { alias, host, user } => {
            match manager
                .add_server(alias.clone(), host, user, 22, None)
                .await
            {
                Ok(_) => CommandResponse::Text(format!(
                    "Server '{}' added successfully (Key-based auth assumed).",
                    alias
                )),
                Err(e) => CommandResponse::Text(format!("Failed to add server: {}", e)),
            }
        }

        SystemCommand::RemoveServer { alias } => match manager.remove_server(&alias).await {
            Ok(removed) => {
                if removed {
                    CommandResponse::Text(format!("Server '{}' removed.", alias))
                } else {
                    CommandResponse::Text(format!("Server '{}' not found.", alias))
                }
            }
            Err(e) => CommandResponse::Text(format!("Failed to remove server: {}", e)),
        },

        SystemCommand::ListServers => match manager.list_servers().await {
            Ok(servers) => {
                if servers.is_empty() {
                    CommandResponse::Text("No servers configured.".to_string())
                } else {
                    let mut options = Vec::new();
                    for (alias, _) in servers {
                        options.push(alias);
                    }
                    CommandResponse::InteractiveList {
                        title: "Select a server to manage:".to_string(),
                        options,
                        callback_prefix: "menu_server:".to_string(),
                    }
                }
            }
            Err(e) => CommandResponse::Text(format!("Failed to list servers: {}", e)),
        },

        SystemCommand::Exec { alias, cmd } => {
            println!("Dispatcher: Executing '{}' on '{}'", cmd, alias);
            match manager.get_server(&alias).await {
                Ok(Some(server)) => {
                    println!("Dispatcher: Server found. Connecting...");
                    match SshExecutor::execute(&server, &cmd) {
                        Ok(output) => {
                            println!("Dispatcher: Execution successful.");

                            // Log output to audit log as well
                            let _ = sqlx::query("UPDATE audit_logs SET output = ? WHERE id = (SELECT MAX(id) FROM audit_logs)")
                                .bind(&output)
                                .execute(&pool)
                                .await;

                            CommandResponse::Text(format!("Output from {}:\n{}", alias, output))
                        }
                        Err(e) => {
                            println!("Dispatcher: Execution failed: {}", e);
                            CommandResponse::Text(format!("Error executing on {}: {}", alias, e))
                        }
                    }
                }
                Ok(None) => CommandResponse::Text(format!(
                    "Server '{}' not found. Use /add to configure it.",
                    alias
                )),
                Err(e) => CommandResponse::Text(format!("Database error: {}", e)),
            }
        }

        SystemCommand::Ask { question } => {
            // Check if we have an active session
            if session_manager.has_session(chat_id) {
                session_manager.process_user_input(chat_id, &question).await
            } else {
                // Try to infer server from question or defaults
                let servers = match manager.list_servers().await {
                    Ok(s) => s,
                    Err(e) => {
                        return CommandResponse::Text(format!("Failed to list servers: {}", e))
                    }
                };

                if servers.is_empty() {
                    return CommandResponse::Text(
                        "No servers configured. Use /add first.".to_string(),
                    );
                }

                // Find alias in question (case-insensitive for better UX?)
                // Just use simple contains for now.
                let target_alias = if servers.len() == 1 {
                    Some(servers[0].0.clone())
                } else {
                    // Sort aliases by length desc to match "prod-db" before "prod"
                    let mut aliases: Vec<String> = servers.iter().map(|(a, _)| a.clone()).collect();
                    aliases.sort_by(|a, b| b.len().cmp(&a.len()));

                    aliases.into_iter().find(|alias| question.contains(alias)) // simple case-sensitive match
                };

                if let Some(alias) = target_alias {
                    session_manager.start_session(chat_id, alias.clone());
                    // Add the user's first question to the session
                    session_manager.process_user_input(chat_id, &question).await
                } else {
                    CommandResponse::Html(
                        "Please specify which server you want to ask about (e.g., <code>/ask check local</code>) or start with <code>/servers</code>.<br>I cannot answer questions about a server without knowing which one you mean.".to_string()
                    )
                }
            }
        }

        SystemCommand::SetProvider { provider } => match provider {
            Some(name) => match ai_client.set_provider(&name).await {
                Ok(msg) => CommandResponse::Text(msg),
                Err(e) => CommandResponse::Text(format!("Failed to set provider: {}", e)),
            },
            None => CommandResponse::InteractiveList {
                title: "Select AI Provider:".to_string(),
                options: vec![
                    "ollama".to_string(),
                    "openai".to_string(),
                    "gemini".to_string(),
                ],
                callback_prefix: "set_provider:".to_string(),
            },
        },

        SystemCommand::ConfigOllama { model, base_url } => {
            let mut config = crate::ai::config::OllamaConfig::load();
            config.model = model;
            if let Some(url) = base_url {
                config.base_url = url;
            }
            match config.save() {
                Ok(_) => CommandResponse::Text(format!(
                    "Ollama config updated. Model: {}, URL: {}",
                    config.model, config.base_url
                )),
                Err(e) => CommandResponse::Text(format!("Failed to save config: {}", e)),
            }
        }

        SystemCommand::ListAiModels => match ai_client.list_models().await {
            Ok(models) => {
                if models.is_empty() {
                    CommandResponse::Text(
                        "No models found. Provider might not support listing models.".to_string(),
                    )
                } else {
                    CommandResponse::InteractiveList {
                        title: "Available AI Models. Click to select:".to_string(),
                        options: models,
                        callback_prefix: "set_model:".to_string(),
                    }
                }
            }
            Err(e) => CommandResponse::Text(format!("Failed to list models: {}", e)),
        },

        SystemCommand::AiInfo => {
            let info = ai_client.get_provider_info().await;
            CommandResponse::Text(format!("Current AI Provider: {}", info))
        }

        SystemCommand::Discover { alias } => {
            match manager.get_server(&alias).await {
                Ok(Some(server)) => {
                    println!("Dispatcher: Running discovery on '{}'", alias);
                    match crate::core::discovery::Discovery::run(&server) {
                        Ok(report) => {
                            let report_json =
                                serde_json::to_string_pretty(&report).unwrap_or_default();
                            println!("Dispatcher: Discovery successful. Analyzing with AI...");

                            // Save stats to DB
                            let _ = sqlx::query(
                                "INSERT INTO server_stats (server_id, cpu_load, memory_usage, disk_usage) VALUES (?, ?, ?, ?)"
                            )
                            .bind(&server.id)
                            .bind(&report.resources.cpu_usage)
                            .bind(&report.resources.memory_usage)
                            .bind(&report.resources.disk_usage)
                            .execute(&pool)
                            .await;

                            let question = "Analyze this server report and tell me what is the status of the server. Are there any issues? What should I check next? Be concise.";

                            match ai_client.ask_with_context(question, &report_json).await {
                                Ok(analysis) => CommandResponse::Text(format!(
                                    "Discovery Report for {}:\n\n{}\n\nAI Analysis:\n{}",
                                    alias, report_json, analysis
                                )),
                                Err(e) => CommandResponse::Text(format!(
                                    "Discovery successful but AI analysis failed: {}\nReport:\n{}",
                                    e, report_json
                                )),
                            }
                        }
                        Err(e) => {
                            CommandResponse::Text(format!("Discovery failed on {}: {}", alias, e))
                        }
                    }
                }
                Ok(None) => CommandResponse::Text(format!(
                    "Server '{}' not found. Use /add to configure it.",
                    alias
                )),
                Err(e) => CommandResponse::Text(format!("Database error: {}", e)),
            }
        }

        SystemCommand::CountTokens { text } => match ai_client.count_tokens(&text).await {
            Ok(count) => CommandResponse::Text(format!("Estimated token count: {}", count)),
            Err(e) => CommandResponse::Text(format!("Failed to count tokens: {}", e)),
        },

        SystemCommand::Explain => {
            let explanation = r#"
<b>PocketSentinel Architecture & Usage</b>

PocketSentinel is an AI-powered server management bot designed to help you monitor, configure, and troubleshoot Linux servers via Telegram.

<b>Core Components:</b>

1.  <b>Dispatcher</b>: The central brain. It receives your commands (e.g., <code>/exec</code>, <code>/ask</code>) and routes them to the appropriate module.
2.  <b>Server Manager</b>: Maintains your list of servers (SQLite DB). It handles adding/removing servers and retrieving connection details.
3.  <b>SSH Executor</b>: Securely connects to your servers using SSH (keys or passwords) to run commands and retrieve output.
4.  <b>AI Client</b>: The intelligence layer. It connects to providers like <b>Ollama</b> (local), <b>OpenAI</b>, or <b>Google Gemini</b> to analyze logs, answer questions (<code>/ask</code>), and generate reports (<code>/discover</code>).
5.  <b>Database</b>: Uses SQLite to store server configs, audit logs, and performance stats.

<b>Key Workflows:</b>

•   <b>Management</b>: Add your servers with <code>/add</code>. List them with <code>/servers</code>.
•   <b>Execution</b>: Run shell commands directly with <code>/exec &lt;alias&gt; &lt;cmd&gt;</code>.
•   <b>Discovery</b>: Use <code>/discover &lt;alias&gt;</code> to run a health check. The bot gathers OS info, resource usage, and running services, then sends this "context" to the AI for analysis.
•   <b>AI Assistance</b>: Use <code>/ask</code> for general questions or let the AI guide you based on previous command outputs.
•   <b>Interactive Investigation</b>: Use <code>/investigate &lt;alias&gt;</code> to start a conversation with the AI about a server, where it can execute commands.

<b>Configuration:</b>

Use <code>/config_ollama</code> (or edit JSON files in <code>config/ai/</code>) to switch models or providers.
            "#.trim().to_string();
            CommandResponse::Html(explanation)
        }

        SystemCommand::Unknown => {
            CommandResponse::Text("Unknown command. Type /help for assistance.".to_string())
        }
    }
}
