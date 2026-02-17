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
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            ai_client: Arc::new(AiClient::new()),
        }
    }

    pub fn start_session(&self, chat_id: i64, alias: String) {
        let system_prompt = format!(
            "You are a Linux server expert assistant interacting with server '{}'. \
            Your goal is to help the user troubleshoot or manage this server. \
            If you need to run a specific command to get information, reply strictly with: \
            RUN: <command> \
            Example: RUN: uptime \
            Do not provide explanation if you are asking to run a command, just the command. \
            If you have a conclusion or answer, just reply normally.",
            alias
        );

        let session = Session {
            server_alias: alias,
            history: vec![ChatMessage::new("system", system_prompt)],
        };

        self.sessions.lock().unwrap().insert(chat_id, session);
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

    pub fn add_message(&self, chat_id: i64, role: &str, content: &str) {
        if let Some(session) = self.sessions.lock().unwrap().get_mut(&chat_id) {
            session.history.push(ChatMessage::new(role, content));
        }
    }

    pub async fn process_user_input(&self, chat_id: i64, input: &str) -> CommandResponse {
        // Add user message
        self.add_message(chat_id, "user", input);

        // Get history
        let history = {
            let guard = self.sessions.lock().unwrap();
            if let Some(session) = guard.get(&chat_id) {
                session.history.clone()
            } else {
                return CommandResponse::Text("No active session.".to_string());
            }
        };

        // Call AI
        match self.ai_client.chat(&history).await {
            Ok(response) => {
                // Add AI response to history
                self.add_message(chat_id, "assistant", &response);

                // Check for tool call
                // Check for tool call
                if response.trim().starts_with("RUN:") {
                    let cmd = response.trim().trim_start_matches("RUN:").trim();

                    use base64::prelude::*;
                    let encoded_cmd = BASE64_STANDARD.encode(cmd);

                    CommandResponse::InteractiveList {
                        title: format!("AI suggests running: `{}`", cmd),
                        options: vec!["✅ Run".to_string(), "❌ Skip".to_string()],
                        callback_prefix: format!("tool_run:{}:", encoded_cmd),
                    }
                } else {
                    CommandResponse::Text(response)
                }
            }
            Err(e) => CommandResponse::Text(format!("AI Error: {}", e)),
        }
    }

    // Manual tool output injection
    pub fn add_tool_output(&self, chat_id: i64, output: &str) {
        let content = format!("Command Output:\n{}", output);
        self.add_message(chat_id, "user", &content); // Treat tool output as user message for simplicity (context)
    }
}
