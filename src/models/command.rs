use serde::{Deserialize, Serialize};

// Represents all available actions the agent can perform
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum SystemCommand {
    GetStatus,
    Help,
    AddServer {
        alias: String,
        host: String,
        user: String,
    },
    RemoveServer {
        alias: String,
    },
    ListServers,
    Exec {
        alias: String,
        cmd: String,
    },
    Ask {
        question: String,
    },
    ConfigOllama {
        model: String,
        base_url: Option<String>,
    },
    ListAiModels,
    Unknown,
}

impl SystemCommand {
    // Helper function to convert a raw string into a structured SystemCommand
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

            ["/ask", question] => SystemCommand::Ask {
                question: question.to_string(),
            },

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
        ]
    }
}
