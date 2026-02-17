use crate::ai::config::{GeminiConfig, OllamaConfig, OpenAiConfig};
use crate::ai::providers::{
    gemini::GeminiProvider, ollama::OllamaProvider, openai::OpenAiProvider,
};
use crate::ai::traits::AiProviderTrait;
use std::env;

pub struct AiClient {
    provider: Box<dyn AiProviderTrait + Send + Sync>,
}

impl AiClient {
    pub fn new() -> Self {
        let provider_str = env::var("AI_PROVIDER").unwrap_or_else(|_| "ollama".to_string());

        let provider: Box<dyn AiProviderTrait + Send + Sync> =
            match provider_str.to_lowercase().as_str() {
                "openai" => Box::new(OpenAiProvider::new(OpenAiConfig::load())),
                "gemini" => Box::new(GeminiProvider::new(GeminiConfig::load())),
                _ => Box::new(OllamaProvider::new(OllamaConfig::load())),
            };

        AiClient { provider }
    }

    pub async fn ask(&self, question: &str) -> Result<String, String> {
        self.provider.ask(question).await
    }

    pub async fn ask_with_context(&self, question: &str, context: &str) -> Result<String, String> {
        let prompt = format!("Context:\n{}\n\nQuestion: {}", context, question);
        self.provider.ask(&prompt).await
    }

    pub async fn list_models(&self) -> Result<Vec<String>, String> {
        self.provider.list_models().await
    }

    pub fn get_provider_info(&self) -> String {
        self.provider.get_info()
    }
}
