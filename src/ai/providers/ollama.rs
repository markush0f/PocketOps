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

    async fn list_models(&self) -> Result<Vec<String>, String> {
        // Ollama API endpoint might change based on version, but usually /api/tags
        let base = self.config.base_url.replace("/api", ""); // standard construct usually includes /api
        let url = format!("{}/api/tags", base.trim_end_matches('/'));

        let res = self
            .client
            .get(&url)
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

        let models = json["models"]
            .as_array()
            .ok_or_else(|| "Invalid response format".to_string())?;

        let names = models
            .iter()
            .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
            .collect();

        Ok(names)
    }

    fn get_info(&self) -> String {
        format!(
            "Ollama (Model: {}, URL: {})",
            self.config.model, self.config.base_url
        )
    }
}
