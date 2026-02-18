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
}

impl AiClient {
    /// Creates a new `AiClient`.
    ///
    /// The provider is selected based on the `AI_PROVIDER` environment variable.
    /// Defaults to `ollama` if not specified.
    pub fn new() -> Self {
        let provider_str = env::var("AI_PROVIDER").unwrap_or_else(|_| "ollama".to_string());

        let provider: Box<dyn AiProviderTrait + Send + Sync> =
            match provider_str.to_lowercase().as_str() {
                "openai" => Box::new(OpenAiProvider::new(OpenAiConfig::load())),
                "gemini" => Box::new(GeminiProvider::new(GeminiConfig::load())),
                _ => Box::new(OllamaProvider::new(OllamaConfig::load())),
            };

        AiClient {
            provider: tokio::sync::RwLock::new(provider),
        }
    }

    /// Switches the active AI provider.
    pub async fn set_provider(&self, name: &str) -> Result<String, String> {
        let new_provider: Box<dyn AiProviderTrait + Send + Sync> =
            match name.to_lowercase().as_str() {
                "openai" => Box::new(OpenAiProvider::new(OpenAiConfig::load())),
                "gemini" => Box::new(GeminiProvider::new(GeminiConfig::load())),
                "ollama" => Box::new(OllamaProvider::new(OllamaConfig::load())),
                _ => return Err(format!("Unknown provider: {}", name)),
            };

        let mut guard = self.provider.write().await;
        *guard = new_provider;
        Ok(format!("Provider switched to {}", name))
    }

    /// Asks the AI a question.
    ///
    /// # Arguments
    ///
    /// * `question` - The question/prompt to send to the AI.
    ///
    /// # Returns
    ///
    /// A `Result` containing the AI's answer as a `String` or an error message.
    pub async fn ask(&self, question: &str) -> Result<String, String> {
        let guard = self.provider.read().await;
        guard.ask(question).await
    }

    /// Asks the AI a question with additional context.
    ///
    /// This is useful for providing logs, command outputs, or previous conversation history.
    ///
    /// # Arguments
    ///
    /// * `question` - The question/prompt to send to the AI.
    /// * `context` - Additional information to prepend to the prompt.
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
    ///
    /// Example: "Ollama (Model: llama3, URL: ...)"
    pub async fn get_provider_info(&self) -> String {
        let guard = self.provider.read().await;
        guard.get_info()
    }
}
