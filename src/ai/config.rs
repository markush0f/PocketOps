use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use std::fs;
use std::path::Path;

/// Configuration settings for the OpenAI provider.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenAiConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

/// Configuration settings for the Ollama provider.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
}

/// Configuration settings for the Gemini provider.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeminiConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

/// Global settings to track current provider.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GlobalConfig {
    pub provider: String,
}

impl GlobalConfig {
    pub async fn load(pool: &Pool<Sqlite>) -> Self {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT value FROM ai_configs WHERE provider = 'global' AND key = 'provider'",
        )
        .fetch_optional(pool)
        .await
        .unwrap_or(None);

        if let Some((provider,)) = row {
            GlobalConfig { provider }
        } else {
            // Fallback to file/default
            Self::load_from_file()
        }
    }

    fn load_from_file() -> Self {
        let path = "config/ai/settings.json";
        if Path::new(path).exists() {
            let content = fs::read_to_string(path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or(GlobalConfig {
                provider: "ollama".to_string(),
            })
        } else {
            GlobalConfig {
                provider: "ollama".to_string(),
            }
        }
    }

    pub async fn save(&self, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT OR REPLACE INTO ai_configs (provider, key, value) VALUES ('global', 'provider', ?)")
            .bind(&self.provider)
            .execute(pool)
            .await?;
        Ok(())
    }
}

impl OpenAiConfig {
    pub async fn load(pool: &Pool<Sqlite>) -> Self {
        let key = sqlx::query_as::<_, (String,)>(
            "SELECT value FROM ai_configs WHERE provider = 'openai' AND key = 'api_key'",
        )
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
        .map(|r| r.0)
        .unwrap_or_default();

        // Try file if DB empty?
        if key.is_empty() {
            return Self::load_from_file();
        }

        let model = sqlx::query_as::<_, (String,)>(
            "SELECT value FROM ai_configs WHERE provider = 'openai' AND key = 'model'",
        )
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
        .map(|r| r.0)
        .unwrap_or_else(|| "gpt-4o".to_string());

        let base_url = sqlx::query_as::<_, (String,)>(
            "SELECT value FROM ai_configs WHERE provider = 'openai' AND key = 'base_url'",
        )
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
        .map(|r| r.0)
        .unwrap_or_else(|| "https://api.openai.com/v1".to_string());

        OpenAiConfig {
            api_key: key,
            model,
            base_url,
        }
    }

    fn load_from_file() -> Self {
        let path = "config/ai/openai.json";
        if Path::new(path).exists() {
            let content = fs::read_to_string(path).expect("Failed to read openai.json");
            serde_json::from_str(&content).unwrap_or(OpenAiConfig {
                api_key: "".to_string(),
                model: "gpt-4o".to_string(),
                base_url: "https://api.openai.com/v1".to_string(),
            })
        } else {
            OpenAiConfig {
                api_key: "".to_string(),
                model: "gpt-4o".to_string(),
                base_url: "https://api.openai.com/v1".to_string(),
            }
        }
    }

    pub async fn save(&self, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT OR REPLACE INTO ai_configs (provider, key, value) VALUES ('openai', 'api_key', ?)")
            .bind(&self.api_key).execute(pool).await?;
        sqlx::query("INSERT OR REPLACE INTO ai_configs (provider, key, value) VALUES ('openai', 'model', ?)")
            .bind(&self.model).execute(pool).await?;
        sqlx::query("INSERT OR REPLACE INTO ai_configs (provider, key, value) VALUES ('openai', 'base_url', ?)")
            .bind(&self.base_url).execute(pool).await?;
        Ok(())
    }
}

impl OllamaConfig {
    pub async fn load(pool: &Pool<Sqlite>) -> Self {
        let model = sqlx::query_as::<_, (String,)>(
            "SELECT value FROM ai_configs WHERE provider = 'ollama' AND key = 'model'",
        )
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
        .map(|r| r.0)
        .unwrap_or_else(|| "llama3".to_string());

        let base_url = sqlx::query_as::<_, (String,)>(
            "SELECT value FROM ai_configs WHERE provider = 'ollama' AND key = 'base_url'",
        )
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
        .map(|r| r.0)
        .unwrap_or_else(|| "http://localhost:11434/api".to_string());

        OllamaConfig { model, base_url }
    }

    // allow Sync load for default/fallback if needed, but primarily use async
    pub fn load_default() -> Self {
        OllamaConfig {
            base_url: "http://localhost:11434/api".to_string(),
            model: "llama3".to_string(),
        }
    }

    pub async fn save(&self, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT OR REPLACE INTO ai_configs (provider, key, value) VALUES ('ollama', 'model', ?)")
            .bind(&self.model).execute(pool).await?;
        sqlx::query("INSERT OR REPLACE INTO ai_configs (provider, key, value) VALUES ('ollama', 'base_url', ?)")
            .bind(&self.base_url).execute(pool).await?;
        Ok(())
    }
}

impl GeminiConfig {
    pub async fn load(pool: &Pool<Sqlite>) -> Self {
        let key = sqlx::query_as::<_, (String,)>(
            "SELECT value FROM ai_configs WHERE provider = 'gemini' AND key = 'api_key'",
        )
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
        .map(|r| r.0)
        .unwrap_or_default();

        if key.is_empty() {
            return Self::load_from_file();
        }

        let model = sqlx::query_as::<_, (String,)>(
            "SELECT value FROM ai_configs WHERE provider = 'gemini' AND key = 'model'",
        )
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
        .map(|r| r.0)
        .unwrap_or_else(|| "gemini-pro".to_string());

        let base_url = sqlx::query_as::<_, (String,)>(
            "SELECT value FROM ai_configs WHERE provider = 'gemini' AND key = 'base_url'",
        )
        .fetch_optional(pool)
        .await
        .unwrap_or(None)
        .map(|r| r.0)
        .unwrap_or_else(|| "https://generativelanguage.googleapis.com/v1beta/models".to_string());

        GeminiConfig {
            api_key: key,
            model,
            base_url,
        }
    }

    fn load_from_file() -> Self {
        let path = "config/ai/gemini.json";
        if Path::new(path).exists() {
            let content = fs::read_to_string(path).expect("Failed to read gemini.json");
            serde_json::from_str(&content).unwrap_or(GeminiConfig {
                api_key: "".to_string(),
                model: "gemini-pro".to_string(),
                base_url: "https://generativelanguage.googleapis.com/v1beta/models".to_string(),
            })
        } else {
            GeminiConfig {
                api_key: "".to_string(),
                model: "gemini-pro".to_string(),
                base_url: "https://generativelanguage.googleapis.com/v1beta/models".to_string(),
            }
        }
    }

    pub async fn save(&self, pool: &Pool<Sqlite>) -> Result<(), sqlx::Error> {
        sqlx::query("INSERT OR REPLACE INTO ai_configs (provider, key, value) VALUES ('gemini', 'api_key', ?)")
            .bind(&self.api_key).execute(pool).await?;
        sqlx::query("INSERT OR REPLACE INTO ai_configs (provider, key, value) VALUES ('gemini', 'model', ?)")
            .bind(&self.model).execute(pool).await?;
        sqlx::query("INSERT OR REPLACE INTO ai_configs (provider, key, value) VALUES ('gemini', 'base_url', ?)")
            .bind(&self.base_url).execute(pool).await?;
        Ok(())
    }
}
