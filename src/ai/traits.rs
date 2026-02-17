use async_trait::async_trait;

/// A trait defining the common interface for all AI providers.
///
/// Implementors of this trait handle the specific API communication logic
/// for different AI services (e.g., Ollama, OpenAI, Gemini).
#[async_trait]
pub trait AiProviderTrait: Send + Sync {
    /// Sends a prompt to the AI and returns the response.
    async fn ask(&self, question: &str) -> Result<String, String>;

    /// Lists the available models for this provider.
    async fn list_models(&self) -> Result<Vec<String>, String>;

    /// Returns a string describing the provider and its current configuration.
    fn get_info(&self) -> String;
}
