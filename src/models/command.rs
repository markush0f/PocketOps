use serde::{Deserialize, Serialize};

/// Represents all available actions the agent or user can trigger within the system.
///
/// This enum maps directly to user commands (e.g., `/status`, `/help`) and internal
/// agent actions.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SystemCommand {
    /// Checks the bot's operational status.
    GetStatus,
    /// Displays the help message with available commands.
    Help,
    /// Adds a new server to the configuration.
    AddServer {
        alias: String,
        host: String,
        user: String,
    },
    /// Removes a server from the configuration.
    RemoveServer { alias: String },
    /// Lists all configured servers.
    ListServers,
    /// Executes a shell command on a specific server.
    Exec { alias: String, cmd: String },
    /// Asks the AI a question.
    Ask { question: String },
    /// Configures the Ollama provider settings.
    ConfigOllama {
        model: String,
        base_url: Option<String>,
    },
    /// Lists available AI models from the current provider.
    ListAiModels,
    /// Shows information about the current AI provider.
    AiInfo,
    /// Runs a discovery process on a server to gather system info.
    Discover { alias: String },
    /// Represents an unrecognized or invalid command.
    Unknown,
}

impl SystemCommand {
    /// Parses a raw string input into a `SystemCommand` variant.
    ///
    /// This function handles command parsing, including argument splitting
    /// and matching against known command patterns.
    ///
    /// # Arguments
    ///
    /// * `input` - The raw command string (e.g., "/add my-server 1.2.3.4 root").
    ///
    /// # Returns
    ///
    /// A `SystemCommand` corresponding to the input, or `SystemCommand::Unknown` if no match is found.
    pub fn from_str(input: &str) -> Self {
        let parts: Vec<&str> = input.split_whitespace().collect();
        match parts.as_slice() {
            ["/status"] => SystemCommand::GetStatus,
            ["/help"] => SystemCommand::Help,
            ["/servers"] => SystemCommand::ListServers,

            ["/add", alias, host, user] => SystemCommand::AddServer {
                alias: alias.to_string(),
                host: host.to_string(),
                user: user.to_string(),
            },

            ["/remove", alias] => SystemCommand::RemoveServer {
                alias: alias.to_string(),
            },

            ["/ask", ..] => {
                let question = parts[1..].join(" ");
                SystemCommand::Ask { question }
            }

            // /config_ollama <model> [base_url]
            ["/config_ollama", model] => SystemCommand::ConfigOllama {
                model: model.to_string(),
                base_url: None,
            },
            ["/config_ollama", model, base_url] => SystemCommand::ConfigOllama {
                model: model.to_string(),
                base_url: Some(base_url.to_string()),
            },

            ["/models"] | ["/ai_models"] => SystemCommand::ListAiModels,

            ["/current_model"] | ["/ai_info"] => SystemCommand::AiInfo,

            ["/discover", alias] => SystemCommand::Discover {
                alias: alias.to_string(),
            },

            ["/exec", alias, ..] => {
                let cmd = parts[2..].join(" ");
                SystemCommand::Exec {
                    alias: alias.to_string(),
                    cmd,
                }
            }

            _ => SystemCommand::Unknown,
        }
    }

    /// Returns a list of all available commands and their descriptions.
    ///
    /// Used for generating the help message.
    pub fn all_commands_info() -> Vec<(&'static str, &'static str)> {
        vec![
            ("/status", "Check bot status"),
            ("/help", "Show this help message"),
            ("/servers", "List configured servers"),
            ("/add <alias> <host> <user>", "Add a new server"),
            ("/remove <alias>", "Remove a server by alias"),
            ("/exec <alias> <cmd>", "Execute a shell command on a server"),
            ("/ask <question>", "Ask the AI a question"),
            ("/config_ollama <model> [url]", "Configure Ollama model/URL"),
            ("/models", "List available AI models"),
            ("/current_model", "Show current AI provider and model"),
            ("/discover <alias>", "Analyze a server's state"),
        ]
    }
}
