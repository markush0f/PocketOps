use reqwest::Client;
use serde_json::{json, Value};
use std::env;

#[derive(Clone)]
pub enum AiProvider {
    OpenAI,
    Ollama,
    Gemini,
}

#[derive(Clone)]
pub struct AiClient {
    client: Client,
    provider: AiProvider,
    api_key: String,
    base_url: String,
    model: String,
}

use crate::ai::config::{GeminiConfig, OllamaConfig, OpenAiConfig};

impl AiClient {
    pub fn new() -> Self {
        let provider_str = env::var("AI_PROVIDER").unwrap_or_else(|_| "ollama".to_string());

        // Default values
        let mut api_key = String::new();
        let mut base_url = String::new();
        let mut model = String::new();

        let provider = match provider_str.to_lowercase().as_str() {
            "openai" => {
                let config = OpenAiConfig::load();
                api_key = config.api_key;
                base_url = config.base_url;
                model = config.model;
                AiProvider::OpenAI
            }
            "gemini" => {
                let config = GeminiConfig::load();
                api_key = config.api_key;
                base_url = config.base_url;
                model = config.model;
                AiProvider::Gemini
            }
            _ => {
                let config = OllamaConfig::load();
                base_url = config.base_url;
                model = config.model;
                AiProvider::Ollama
            }
        };

        AiClient {
            client: Client::new(),
            provider,
            api_key,
            base_url,
            model,
        }
    }

    pub async fn ask(&self, question: &str) -> Result<String, String> {
        match self.provider {
            AiProvider::OpenAI => self.ask_openai(question).await,
            AiProvider::Ollama => self.ask_ollama(question).await,
            AiProvider::Gemini => self.ask_gemini(question).await,
        }
    }

    async fn ask_openai(&self, question: &str) -> Result<String, String> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = json!({
            "model": self.model,
            "messages": [{"role": "user", "content": question}]
        });

        let res = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
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

    async fn ask_ollama(&self, question: &str) -> Result<String, String> {
        let url = format!("{}/generate", self.base_url);
        let body = json!({
            "model": self.model,
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

    async fn ask_gemini(&self, question: &str) -> Result<String, String> {
        let url = format!(
            "{}/{}:generateContent?key={}",
            self.base_url, self.model, self.api_key
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
}
