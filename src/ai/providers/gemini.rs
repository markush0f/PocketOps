use crate::ai::config::GeminiConfig;
use crate::ai::traits::AiProviderTrait;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

/// A provider implementation for Google Gemini.
///
/// This struct handles communication with the Gemini API.
pub struct GeminiProvider {
    client: Client,
    config: GeminiConfig,
}

impl GeminiProvider {
    /// Creates a new `GeminiProvider` with the given configuration.
    pub fn new(config: GeminiConfig) -> Self {
        Self {
            client: Client::new(),
            config,
        }
    }
}

#[async_trait]
impl AiProviderTrait for GeminiProvider {
    async fn ask(&self, question: &str) -> Result<String, String> {
        let url = format!(
            "{}/{}:generateContent?key={}",
            self.config.base_url, self.config.model, self.config.api_key
        );
        let body = json!({
            "contents": [{
                "parts": [{"text": question}]
            }]
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
        json["candidates"][0]["content"]["parts"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| "No content in response".to_string())
    }

    async fn list_models(&self) -> Result<Vec<String>, String> {
        Ok(vec![
            "gemini-pro".to_string(),
            "gemini-1.5-pro".to_string(),
            "gemini-1.5-flash".to_string(),
        ])
    }

    fn get_info(&self) -> String {
        format!("Gemini (Model: {})", self.config.model)
    }
}
