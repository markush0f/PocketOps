use crate::ai::client::AiClient;
use crate::ai::models::ChatMessage;
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
            "You are a Linux server expert assistant interacting with server '<b>{}</b>'. \
            You HAVE access to this server via the user. \
            If the user asks about system status (cpu, memory, disk, performance, etc.), you MUST ask to run a command to diagnose it. \
            Do NOT say you don't have access. Instead, reply with the command you need to run using the RUN: syntax. \
            \
            <b>Tool Syntax:</b> \
            To run a command, reply strictly with: <code>RUN: &lt;command&gt;</code> \
            Example: <code>RUN: uptime</code> \
            \
            <b>Output Format:</b> \
            USE HTML TAGS. <b>bold</b>, <i>italic</i>, <code>code</code>. \
            If you need more info, just ask to RUN the command.",
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
                        last_msg.content.push_str("\n\n[SYSTEM: You are connected to the server via SSH. You must RUN commands to answer status queries. Output `RUN: <command>` if needed. Format using HTML tags (e.g. <b>bold</b>). Do NOT use markdown.]");
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

                // Check for tool call
                // Check for tool call
                // Check for tool call (RUN: <cmd>)
                // We handle cases where the AI provides explanation before the command.
                if let Some(idx) = response.find("RUN:") {
                    let (message_part, cmd_part) = response.split_at(idx);
                    let cmd = cmd_part.trim_start_matches("RUN:").trim();

                    // Only process checks if a command actually exists
                    if !cmd.is_empty() {
                        use base64::prelude::*;
                        let encoded_cmd = BASE64_STANDARD.encode(cmd);

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
}
