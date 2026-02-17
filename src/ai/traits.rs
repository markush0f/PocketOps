use async_trait::async_trait;

#[async_trait]
#[async_trait]
pub trait AiProviderTrait: Send + Sync {
    async fn ask(&self, question: &str) -> Result<String, String>;
    async fn list_models(&self) -> Result<Vec<String>, String>;
}
