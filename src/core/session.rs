use crate::ai::client::AiClient;
use crate::ai::models::ChatMessage;
use crate::core::server_manager::ServerManager;
use crate::executor::ssh::SshExecutor;
use crate::models::CommandResponse;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct Session {
    pub server_alias: String,
    pub history: Vec<ChatMessage>,
}

#[derive(Clone)]
pub struct SessionManager {
    sessions: Arc<Mutex<HashMap<i64, Session>>>,
    ai_client: Arc<AiClient>,
    pool: crate::db::DbPool,
}

impl SessionManager {
    pub async fn new(pool: crate::db::DbPool) -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            ai_client: Arc::new(AiClient::new(pool.clone()).await),
            pool,
        }
    }

    pub async fn start_session(&self, chat_id: i64, alias: String) {
        let system_prompt = format!(
            include_str!("../../templates/prompts/server_assistant.html"),
            alias
        );

        let session = Session {
            server_alias: alias,
            history: vec![ChatMessage::new("system", &system_prompt)],
        };

        self.sessions.lock().unwrap().insert(chat_id, session);

        // Persist system message
        self.add_message(chat_id, "system", &system_prompt).await;
    }

    pub fn end_session(&self, chat_id: i64) -> Option<Session> {
        self.sessions.lock().unwrap().remove(&chat_id)
    }

    pub fn has_session(&self, chat_id: i64) -> bool {
        self.sessions.lock().unwrap().contains_key(&chat_id)
    }

    pub fn get_alias(&self, chat_id: i64) -> Option<String> {
        self.sessions
            .lock()
            .unwrap()
            .get(&chat_id)
            .map(|s| s.server_alias.clone())
    }

    pub async fn add_message(&self, chat_id: i64, role: &str, content: &str) {
        // Update memory
        if let Some(session) = self.sessions.lock().unwrap().get_mut(&chat_id) {
            session.history.push(ChatMessage::new(role, content));
        }

        // Update DB (best effort, log error)
        if let Err(e) =
            sqlx::query("INSERT INTO chat_history (chat_id, role, content) VALUES (?, ?, ?)")
                .bind(chat_id)
                .bind(role)
                .bind(content)
                .execute(&self.pool)
                .await
        {
            eprintln!("Failed to save chat message: {}", e);
        }
    }

    pub async fn process_user_input(&self, chat_id: i64, input: &str) -> CommandResponse {
        // Add user message
        self.add_message(chat_id, "user", input).await;

        // Get history
        // Get history and inject reminder into the last user message
        let history = {
            let guard = self.sessions.lock().unwrap();
            if let Some(session) = guard.get(&chat_id) {
                let mut history = session.history.clone();

                // Inject reminder directly into the last user message for maximum adherence
                if let Some(last_msg) = history.last_mut() {
                    if last_msg.role == "user" {
                        last_msg
                            .content
                            .push_str(include_str!("../../templates/prompts/ssh_reminder.txt"));
                    }
                }
                history
            } else {
                return CommandResponse::Text("No active session.".to_string());
            }
        };

        // Call AI
        match self.ai_client.chat(&history).await {
            Ok(response) => {
                // Add AI response to history
                self.add_message(chat_id, "assistant", &response).await;

                // We handle cases where the AI provides explanation before the command.
                if let Some(idx) = response.find("RUN:") {
                    let (message_part, cmd_part) = response.split_at(idx);
                    let cmd_raw = cmd_part.trim_start_matches("RUN:").trim();
                    // Strip HTML tags from the command (AI sometimes wraps in <code> etc.)
                    let cmd = cmd_raw
                        .replace("<code>", "")
                        .replace("</code>", "")
                        .replace("<b>", "")
                        .replace("</b>", "")
                        .replace("<i>", "")
                        .replace("</i>", "")
                        .trim()
                        .to_string();

                    // Only process checks if a command actually exists
                    if !cmd.is_empty() {
                        use base64::prelude::*;
                        let encoded_cmd = BASE64_STANDARD.encode(&cmd);

                        // Determine the message to show above the buttons
                        let title = if message_part.trim().is_empty() {
                            format!("AI suggests running: <code>{}</code>", cmd)
                        } else {
                            // Append the command to the message for clarity, or just use the message?
                            // Best to show both.
                            format!(
                                "{}\n\nRunning command: <code>{}</code>",
                                message_part.trim(),
                                cmd
                            )
                        };

                        CommandResponse::InteractiveList {
                            title,
                            options: vec!["✅ Run".to_string(), "❌ Skip".to_string()],
                            callback_prefix: format!("tool_run:{}:", encoded_cmd),
                        }
                    } else {
                        CommandResponse::Html(response)
                    }
                } else {
                    CommandResponse::Html(response)
                }
            }
            Err(e) => CommandResponse::Text(format!("AI Error: {}", e)),
        }
    }

    // Manual tool output injection
    pub async fn add_tool_output(&self, chat_id: i64, output: &str) {
        let content = format!("Command Output:\n{}", output);
        self.add_message(chat_id, "user", &content).await; // Treat tool output as user message for simplicity (context)
    }

    pub async fn execute_tool_command(&self, chat_id: i64, cmd: &str) -> CommandResponse {
        let alias = match self.get_alias(chat_id) {
            Some(a) => a,
            None => return CommandResponse::Text("No active session.".to_string()),
        };

        let manager = ServerManager::new(self.pool.clone());
        let output = match manager.get_server(&alias).await {
            Ok(Some(server)) => match SshExecutor::execute(&server, cmd) {
                Ok(out) => out,
                Err(e) => format!("Error: {}", e),
            },
            Ok(None) => "Server not found.".to_string(),
            Err(e) => format!("DB Error: {}", e),
        };

        self.add_tool_output(chat_id, &output).await;

        self.process_user_input(chat_id, "Command executed. Analyze results.")
            .await
    }

    pub async fn reload_ai_config(&self) {
        if let Err(e) = self.ai_client.reload_config().await {
            eprintln!("Failed to reload AI config: {}", e);
        }
    }
}
