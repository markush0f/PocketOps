use async_trait::async_trait;

/// A trait defining the common interface for all AI providers.
///
/// Implementors of this trait handle the specific API communication logic
/// for different AI services (e.g., Ollama, OpenAI, Gemini).
#[async_trait]
pub trait AiProviderTrait: Send + Sync {
    /// Sends a prompt to the AI and returns the response.
    async fn ask(&self, question: &str) -> Result<String, String>;

    /// Sends a chat history to the AI and returns the next response.
    async fn chat(&self, messages: &[crate::ai::models::ChatMessage]) -> Result<String, String>;

    /// Lists the available models for this provider.
    async fn list_models(&self) -> Result<Vec<String>, String>;

    /// Returns the number of tokens in the given text using the provider's tokenizer.
    async fn count_tokens(&self, text: &str) -> Result<usize, String>;

    /// Returns a string describing the provider and its current configuration.
    fn get_info(&self) -> String;
}
