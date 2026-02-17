use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct OpenAiConfig {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OllamaConfig {
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Deserialize, Clone)]
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
            // Default fallback or panic? Let's return a default for now to avoid crashing if file missing
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
