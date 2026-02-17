use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OpenAiConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GeminiConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

impl OpenAiConfig {
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
}

impl OllamaConfig {
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

    pub fn save(&self) -> Result<(), std::io::Error> {
        let content = serde_json::to_string_pretty(self)?;
        if let Some(parent) = Path::new("config/ai/ollama.json").parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write("config/ai/ollama.json", content)
    }
}

impl GeminiConfig {
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
}
