use crate::ai::client::AiClient;
use crate::core::server_manager::ServerManager;
use crate::executor::ssh::SshExecutor;
use crate::models::command::SystemCommand;

/// Dispatches a `SystemCommand` to the appropriate handler.
///
/// This function acts as the central controller for the application. It:
/// 1. Initializes necessary managers (`ServerManager`, `AiClient`).
/// 2. Matches the incoming command.
/// 3. Executes the corresponding logic (Server management, SSH execution, AI interaction).
/// 4. Returns a user-friendly string response.
///
/// # Arguments
///
/// * `command` - The parsed system command to execute.
///
/// # Returns
///
/// A `String` containing the result of the command execution, ready to be sent back to the user/UI.
/// Returns a tuple `(ResponseText, IsHtml)`.
pub async fn dispatch(command: SystemCommand) -> (String, bool) {
    let manager = ServerManager::new();
    let ai_client = AiClient::new();

    match command {
        SystemCommand::GetStatus => ("System status: Operational".to_string(), false),

        SystemCommand::Help => {
            let mut help_msg = "Available commands:\n".to_string();
            for (cmd, desc) in SystemCommand::all_commands_info() {
                help_msg.push_str(&format!("  {} - {}\n", cmd, desc));
            }
            (help_msg, false)
        }

        SystemCommand::AddServer { alias, host, user } => {
            manager.add_server(alias.clone(), host, user, 22, None);
            (
                format!(
                    "Server '{}' added successfully (Key-based auth assumed).",
                    alias
                ),
                false,
            )
        }

        SystemCommand::RemoveServer { alias } => {
            if manager.remove_server(&alias) {
                (format!("Server '{}' removed.", alias), false)
            } else {
                (format!("Server '{}' not found.", alias), false)
            }
        }

        SystemCommand::ListServers => {
            let servers = manager.list_servers();
            if servers.is_empty() {
                ("No servers configured.".to_string(), false)
            } else {
                let mut out = "Configured Servers:\n".to_string();
                for (alias, server) in servers {
                    out.push_str(&format!(
                        "- {}: {}@{}\n",
                        alias, server.ssh_user, server.hostname
                    ));
                }
                (out, false)
            }
        }

        SystemCommand::Exec { alias, cmd } => {
            println!("Dispatcher: Executing '{}' on '{}'", cmd, alias);
            if let Some(server) = manager.get_server(&alias) {
                println!("Dispatcher: Server found. Connecting...");
                match SshExecutor::execute(&server, &cmd) {
                    Ok(output) => {
                        println!("Dispatcher: Execution successful.");
                        (format!("Output from {}:\n{}", alias, output), false)
                    }
                    Err(e) => {
                        println!("Dispatcher: Execution failed: {}", e);
                        (format!("Error executing on {}: {}", alias, e), false)
                    }
                }
            } else {
                (
                    format!("Server '{}' not found. Use /add to configure it.", alias),
                    false,
                )
            }
        }

        SystemCommand::Ask { question } => {
            println!("Dispatcher: Asking AI: '{}'", question);
            match ai_client.ask(&question).await {
                Ok(answer) => (answer, false),
                Err(e) => (format!("AI Error: {}", e), false),
            }
        }

        SystemCommand::ConfigOllama { model, base_url } => {
            let mut config = crate::ai::config::OllamaConfig::load();
            config.model = model;
            if let Some(url) = base_url {
                config.base_url = url;
            }
            match config.save() {
                Ok(_) => (
                    format!(
                        "Ollama config updated. Model: {}, URL: {}",
                        config.model, config.base_url
                    ),
                    false,
                ),
                Err(e) => (format!("Failed to save config: {}", e), false),
            }
        }

        SystemCommand::ListAiModels => match ai_client.list_models().await {
            Ok(models) => {
                let mut out = "Available Models:\n".to_string();
                for model in models {
                    out.push_str(&format!("- {}\n", model));
                }
                (out, false)
            }
            Err(e) => (format!("Failed to list models: {}", e), false),
        },

        SystemCommand::AiInfo => {
            let info = ai_client.get_provider_info();
            (format!("Current AI Provider: {}", info), false)
        }

        SystemCommand::Discover { alias } => {
            if let Some(server) = manager.get_server(&alias) {
                println!("Dispatcher: Running discovery on '{}'", alias);
                match crate::core::discovery::Discovery::run(&server) {
                    Ok(report) => {
                        let report_json = serde_json::to_string_pretty(&report).unwrap_or_default();
                        println!("Dispatcher: Discovery successful. Analyzing with AI...");

                        let question = "Analyze this server report and tell me what is the status of the server. Are there any issues? What should I check next? Be concise.";

                        match ai_client.ask_with_context(question, &report_json).await {
                            Ok(analysis) => (
                                format!(
                                    "Discovery Report for {}:\n\n{}\n\nAI Analysis:\n{}",
                                    alias, report_json, analysis
                                ),
                                false,
                            ),
                            Err(e) => (
                                format!(
                                    "Discovery successful but AI analysis failed: {}\nReport:\n{}",
                                    e, report_json
                                ),
                                false,
                            ),
                        }
                    }
                    Err(e) => (format!("Discovery failed on {}: {}", alias, e), false),
                }
            } else {
                (
                    format!("Server '{}' not found. Use /add to configure it.", alias),
                    false,
                )
            }
        }

        SystemCommand::CountTokens { text } => match ai_client.count_tokens(&text).await {
            Ok(count) => (format!("Estimated token count: {}", count), false),
            Err(e) => (format!("Failed to count tokens: {}", e), false),
        },

        SystemCommand::Explain => {
            let explanation = r#"
<b>PocketSentinel Architecture & Usage</b>

PocketSentinel is an AI-powered server management bot designed to help you monitor, configure, and troubleshoot Linux servers via Telegram.

<b>Core Components:</b>

1.  <b>Dispatcher</b>: The central brain. It receives your commands (e.g., <code>/exec</code>, <code>/ask</code>) and routes them to the appropriate module.
2.  <b>Server Manager</b>: Maintains your list of servers (<code>servers.json</code>). It handles adding/removing servers and retrieving connection details.
3.  <b>SSH Executor</b>: Securely connects to your servers using SSH (keys or passwords) to run commands and retrieve output.
4.  <b>AI Client</b>: The intelligence layer. It connects to providers like <b>Ollama</b> (local), <b>OpenAI</b>, or <b>Google Gemini</b> to analyze logs, answer questions (<code>/ask</code>), and generate reports (<code>/discover</code>).

<b>Key Workflows:</b>

•   <b>Management</b>: Add your servers with <code>/add</code>. List them with <code>/servers</code>.
•   <b>Execution</b>: Run shell commands directly with <code>/exec &lt;alias&gt; &lt;cmd&gt;</code>.
•   <b>Discovery</b>: Use <code>/discover &lt;alias&gt;</code> to run a health check. The bot gathers OS info, resource usage, and running services, then sends this "context" to the AI for analysis.
•   <b>AI Assistance</b>: Use <code>/ask</code> for general questions or let the AI guide you based on previous command outputs.

<b>Configuration:</b>

Use <code>/config_ollama</code> (or edit JSON files in <code>config/ai/</code>) to switch models or providers.
            "#.trim().to_string();
            (explanation, true)
        }

        SystemCommand::Unknown => (
            "Unknown command. Type /help for assistance.".to_string(),
            false,
        ),
    }
}
