use crate::ai::config::{GeminiConfig, OllamaConfig, OpenAiConfig};
use crate::ai::providers::{
    gemini::GeminiProvider, ollama::OllamaProvider, openai::OpenAiProvider,
};
use crate::ai::traits::AiProviderTrait;
use std::env;

/// The main client for interacting with AI providers.
///
/// This client abstracts the underlying provider implementation (Ollama, OpenAI, Gemini)
/// allowing the rest of the application to interact with AI in a uniform way.
pub struct AiClient {
    /// The specific AI provider implementation.
    provider: tokio::sync::RwLock<Box<dyn AiProviderTrait + Send + Sync>>,
    /// Database pool for saving/loading configuration.
    pool: crate::db::DbPool,
}

impl AiClient {
    /// Creates a new `AiClient`.
    ///
    /// The provider is selected based on saved configuration or environment variable.
    pub async fn new(pool: crate::db::DbPool) -> Self {
        use crate::ai::config::GlobalConfig;

        let global_conf = GlobalConfig::load(&pool).await;

        let provider_str = if global_conf.provider == "ollama" {
            env::var("AI_PROVIDER").unwrap_or_else(|_| "ollama".to_string())
        } else {
            global_conf.provider
        };

        let provider: Box<dyn AiProviderTrait + Send + Sync> =
            match provider_str.to_lowercase().as_str() {
                "openai" => Box::new(OpenAiProvider::new(OpenAiConfig::load(&pool).await)),
                "gemini" => Box::new(GeminiProvider::new(GeminiConfig::load(&pool).await)),
                _ => Box::new(OllamaProvider::new(OllamaConfig::load(&pool).await)),
            };

        AiClient {
            provider: tokio::sync::RwLock::new(provider),
            pool,
        }
    }

    /// Switches the active AI provider.
    pub async fn set_provider(&self, name: &str) -> Result<String, String> {
        let new_provider: Box<dyn AiProviderTrait + Send + Sync> =
            match name.to_lowercase().as_str() {
                "openai" => Box::new(OpenAiProvider::new(OpenAiConfig::load(&self.pool).await)),
                "gemini" => Box::new(GeminiProvider::new(GeminiConfig::load(&self.pool).await)),
                "ollama" => Box::new(OllamaProvider::new(OllamaConfig::load(&self.pool).await)),
                _ => return Err(format!("Unknown provider: {}", name)),
            };

        let mut guard = self.provider.write().await;
        *guard = new_provider;

        // Persist
        use crate::ai::config::GlobalConfig;
        let config = GlobalConfig {
            provider: name.to_string(),
        };

        if let Err(e) = config.save(&self.pool).await {
            return Ok(format!(
                "Provider switched to {}, but failed to save config to DB: {}",
                name, e
            ));
        }

        Ok(format!("Provider switched to {}", name))
    }

    /// Asks the AI a question.
    pub async fn ask(&self, question: &str) -> Result<String, String> {
        let guard = self.provider.read().await;
        guard.ask(question).await
    }

    /// Asks the AI a question with additional context.
    pub async fn ask_with_context(&self, question: &str, context: &str) -> Result<String, String> {
        let prompt = format!("Context:\n{}\n\nQuestion: {}", context, question);
        let guard = self.provider.read().await;
        guard.ask(&prompt).await
    }

    /// Conversations with history.
    pub async fn chat(
        &self,
        messages: &[crate::ai::models::ChatMessage],
    ) -> Result<String, String> {
        let guard = self.provider.read().await;
        guard.chat(messages).await
    }

    /// Lists the available models for the current provider.
    pub async fn list_models(&self) -> Result<Vec<String>, String> {
        let guard = self.provider.read().await;
        guard.list_models().await
    }

    /// Returns the estimated token count for the given text.
    pub async fn count_tokens(&self, text: &str) -> Result<usize, String> {
        let guard = self.provider.read().await;
        guard.count_tokens(text).await
    }

    /// Returns information about the current AI provider and configuration.
    pub async fn get_provider_info(&self) -> String {
        let guard = self.provider.read().await;
        guard.get_info()
    }
}
