use crate::ai::config::OpenAiConfig;
use crate::ai::traits::AiProviderTrait;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

/// A provider implementation for OpenAI.
///
/// This struct handles communication with the OpenAI API (GPT models).
pub struct OpenAiProvider {
    client: Client,
    config: OpenAiConfig,
}

impl OpenAiProvider {
    /// Creates a new `OpenAiProvider` with the given configuration.
    pub fn new(config: OpenAiConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }
}

#[async_trait]
impl AiProviderTrait for OpenAiProvider {
    async fn ask(&self, question: &str) -> Result<String, String> {
        let url = format!("{}/chat/completions", self.config.base_url);
        let body = json!({
            "model": self.config.model,
            "messages": [{"role": "user", "content": question}]
        });

        let res = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !res.status().is_success() {
            return Err(format!("API Error: {}", res.status()));
        }

        let json: Value = res
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No content in response".to_string())
    }

    async fn chat(&self, messages: &[crate::ai::models::ChatMessage]) -> Result<String, String> {
        let url = format!("{}/chat/completions", self.config.base_url);
        let body = json!({
            "model": self.config.model,
            "messages": messages
        });

        let res = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !res.status().is_success() {
            return Err(format!("API Error: {}", res.status()));
        }

        let json: Value = res
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;
        json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No content in response".to_string())
    }

    async fn list_models(&self) -> Result<Vec<String>, String> {
        Ok(vec![
            "gpt-4o".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-3.5-turbo".to_string(),
            "gpt-4o-mini".to_string(),
            "gpt-5-nano".to_string(),
        ])
    }

    async fn count_tokens(&self, text: &str) -> Result<usize, String> {
        // Try to get encoding for specific model, fallback to cl100k_base (gpt-4)
        let bpe = tiktoken_rs::get_bpe_from_model(&self.config.model)
            .or_else(|_| tiktoken_rs::cl100k_base())
            .map_err(|e| format!("Failed to load tokenizer: {}", e))?;

        Ok(bpe.encode_with_special_tokens(text).len())
    }

    fn get_info(&self) -> String {
        format!("OpenAI (Model: {})", self.config.model)
    }
}
