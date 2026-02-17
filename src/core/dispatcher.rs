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
pub async fn dispatch(command: SystemCommand) -> String {
    let manager = ServerManager::new();
    let ai_client = AiClient::new();

    match command {
        SystemCommand::GetStatus => "System status: Operational".to_string(),

        SystemCommand::Help => {
            let mut help_msg = "Available commands:\n".to_string();
            for (cmd, desc) in SystemCommand::all_commands_info() {
                help_msg.push_str(&format!("  {} - {}\n", cmd, desc));
            }
            help_msg
        }

        SystemCommand::AddServer { alias, host, user } => {
            manager.add_server(alias.clone(), host, user, 22, None);
            format!(
                "Server '{}' added successfully (Key-based auth assumed).",
                alias
            )
        }

        SystemCommand::RemoveServer { alias } => {
            if manager.remove_server(&alias) {
                format!("Server '{}' removed.", alias)
            } else {
                format!("Server '{}' not found.", alias)
            }
        }

        SystemCommand::ListServers => {
            let servers = manager.list_servers();
            if servers.is_empty() {
                "No servers configured.".to_string()
            } else {
                let mut out = "Configured Servers:\n".to_string();
                for (alias, server) in servers {
                    out.push_str(&format!(
                        "- {}: {}@{}\n",
                        alias, server.ssh_user, server.hostname
                    ));
                }
                out
            }
        }

        SystemCommand::Exec { alias, cmd } => {
            println!("Dispatcher: Executing '{}' on '{}'", cmd, alias);
            if let Some(server) = manager.get_server(&alias) {
                println!("Dispatcher: Server found. Connecting...");
                match SshExecutor::execute(&server, &cmd) {
                    Ok(output) => {
                        println!("Dispatcher: Execution successful.");
                        format!("Output from {}:\n{}", alias, output)
                    }
                    Err(e) => {
                        println!("Dispatcher: Execution failed: {}", e);
                        format!("Error executing on {}: {}", alias, e)
                    }
                }
            } else {
                format!("Server '{}' not found. Use /add to configure it.", alias)
            }
        }

        SystemCommand::Ask { question } => {
            println!("Dispatcher: Asking AI: '{}'", question);
            match ai_client.ask(&question).await {
                Ok(answer) => answer,
                Err(e) => format!("AI Error: {}", e),
            }
        }

        SystemCommand::ConfigOllama { model, base_url } => {
            let mut config = crate::ai::config::OllamaConfig::load();
            config.model = model;
            if let Some(url) = base_url {
                config.base_url = url;
            }
            match config.save() {
                Ok(_) => format!(
                    "Ollama config updated. Model: {}, URL: {}",
                    config.model, config.base_url
                ),
                Err(e) => format!("Failed to save config: {}", e),
            }
        }

        SystemCommand::ListAiModels => match ai_client.list_models().await {
            Ok(models) => {
                let mut out = "Available Models:\n".to_string();
                for model in models {
                    out.push_str(&format!("- {}\n", model));
                }
                out
            }
            Err(e) => format!("Failed to list models: {}", e),
        },

        SystemCommand::AiInfo => {
            let info = ai_client.get_provider_info();
            format!("Current AI Provider: {}", info)
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
                            Ok(analysis) => format!(
                                "Discovery Report for {}:\n\n{}\n\nAI Analysis:\n{}",
                                alias, report_json, analysis
                            ),
                            Err(e) => format!(
                                "Discovery successful but AI analysis failed: {}\nReport:\n{}",
                                e, report_json
                            ),
                        }
                    }
                    Err(e) => format!("Discovery failed on {}: {}", alias, e),
                }
            } else {
                format!("Server '{}' not found. Use /add to configure it.", alias)
            }
        }

        SystemCommand::Unknown => "Unknown command. Type /help for assistance.".to_string(),
    }
}
