use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Configuration settings for the OpenAI provider.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenAiConfig {
    /// The API key for authentication.
    pub api_key: String,
    /// The model name to use (e.g., "gpt-4").
    pub model: String,
    /// The base URL for the API (defaults to https://api.openai.com/v1).
    pub base_url: String,
}

/// Configuration settings for the Ollama provider.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OllamaConfig {
    /// The base URL for the Ollama server (e.g., http://localhost:11434).
    pub base_url: String,
    /// The model name to run (e.g., "llama3").
    pub model: String,
}

/// Configuration settings for the Gemini provider.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeminiConfig {
    /// The API key for authentication.
    pub api_key: String,
    /// The model name to use (e.g., "gemini-pro").
    pub model: String,
    /// The base URL for the API.
    pub base_url: String,
}

/// Global settings to track current provider.
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GlobalConfig {
    pub provider: String,
}

impl GlobalConfig {
    pub fn load() -> Self {
        let path = "config/ai/settings.json";
        if Path::new(path).exists() {
            let content = fs::read_to_string(path).expect("Failed to read settings.json");
            serde_json::from_str(&content).unwrap_or_else(|_| GlobalConfig {
                provider: "ollama".to_string(),
            })
        } else {
            GlobalConfig {
                provider: "ollama".to_string(),
            }
        }
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = Path::new("config/ai/settings.json").parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write("config/ai/settings.json", content)
    }
}

impl OpenAiConfig {
    /// Loads the configuration from `config/ai/openai.json`.
    pub fn load() -> Self {
        let path = "config/ai/openai.json";
        if Path::new(path).exists() {
            let content = fs::read_to_string(path).expect("Failed to read openai.json");
            serde_json::from_str(&content).expect("Failed to parse openai.json")
        } else {
            OpenAiConfig {
                api_key: "".to_string(),
                model: "gpt-4o".to_string(),
                base_url: "https://api.openai.com/v1".to_string(),
            }
        }
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = Path::new("config/ai/openai.json").parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write("config/ai/openai.json", content)
    }
}

impl OllamaConfig {
    /// Loads the configuration from `config/ai/ollama.json`.
    pub fn load() -> Self {
        let path = "config/ai/ollama.json";
        if Path::new(path).exists() {
            let content = fs::read_to_string(path).expect("Failed to read ollama.json");
            serde_json::from_str(&content).expect("Failed to parse ollama.json")
        } else {
            OllamaConfig {
                base_url: "http://localhost:11434/api".to_string(),
                model: "llama3".to_string(),
            }
        }
    }

    /// Saves the current configuration to `config/ai/ollama.json`.
    pub fn save(&self) -> Result<(), std::io::Error> {
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = Path::new("config/ai/ollama.json").parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write("config/ai/ollama.json", content)
    }
}

impl GeminiConfig {
    /// Loads the configuration from `config/ai/gemini.json`.
    pub fn load() -> Self {
        let path = "config/ai/gemini.json";
        if Path::new(path).exists() {
            let content = fs::read_to_string(path).expect("Failed to read gemini.json");
            serde_json::from_str(&content).expect("Failed to parse gemini.json")
        } else {
            GeminiConfig {
                api_key: "".to_string(),
                model: "gemini-pro".to_string(),
                base_url: "https://generativelanguage.googleapis.com/v1beta/models".to_string(),
            }
        }
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = Path::new("config/ai/gemini.json").parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write("config/ai/gemini.json", content)
    }
}
