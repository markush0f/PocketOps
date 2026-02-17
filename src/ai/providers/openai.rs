use crate::ai::config::OpenAiConfig;
use crate::ai::traits::AiProviderTrait;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

pub struct OpenAiProvider {
    client: Client,
    config: OpenAiConfig,
}

impl OpenAiProvider {
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

    async fn list_models(&self) -> Result<Vec<String>, String> {
        Ok(vec![
            "gpt-4o".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-3.5-turbo".to_string(),
        ])
    }
}
