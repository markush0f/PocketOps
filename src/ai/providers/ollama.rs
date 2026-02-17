use crate::ai::config::OllamaConfig;
use crate::ai::traits::AiProviderTrait;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

pub struct OllamaProvider {
    client: Client,
    config: OllamaConfig,
}

impl OllamaProvider {
    pub fn new(config: OllamaConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }
}

#[async_trait]
impl AiProviderTrait for OllamaProvider {
    async fn ask(&self, question: &str) -> Result<String, String> {
        let url = format!("{}/generate", self.config.base_url);
        let body = json!({
            "model": self.config.model,
            "prompt": question,
            "stream": false
        });

        let res = self
            .client
            .post(&url)
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
        json["response"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No response field".to_string())
    }
}
