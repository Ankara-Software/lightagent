use std::env;
use reqwest::blocking::Client;
use serde_json::{json, Value};

pub struct LlmClient {
    client: Client,
}

impl LlmClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub fn get_next_action(&self, system_prompt: &str, history: &str) -> Result<Value, String> {
        let anthropic_key = env::var("ANTHROPIC_API_KEY").ok();
        let openai_key = env::var("OPENAI_API_KEY").ok();

        if let Some(key) = anthropic_key {
            self.call_anthropic(&key, system_prompt, history)
        } else if let Some(key) = openai_key {
            self.call_openai(&key, system_prompt, history)
        } else {
            // Default to local Ollama
            self.call_ollama(system_prompt, history)
        }
    }

    fn call_ollama(&self, system_prompt: &str, history: &str) -> Result<Value, String> {
        let ollama_url = env::var("OLLAMA_URL").unwrap_or_else(|_| "http://localhost:11434".to_string());
        let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "qwen2.5-coder".to_string());
        let url = format!("{}/api/chat", ollama_url);

        let payload = json!({
            "model": model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": history }
            ],
            "stream": false,
            "format": "json"
        });

        let response = self.client.post(&url)
            .json(&payload)
            .send()
            .map_err(|e| format!("Ollama connection error: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Ollama returned HTTP error: {}", response.status()));
        }

        let body: Value = response.json().map_err(|e| format!("Ollama JSON parse error: {}", e))?;
        let content_str = body.get("message")
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or("Invalid response format from Ollama")?;

        serde_json::from_str(content_str)
            .map_err(|e| format!("Failed to parse LLM response as JSON action: {}\nRaw: {}", e, content_str))
    }

    fn call_openai(&self, key: &str, system_prompt: &str, history: &str) -> Result<Value, String> {
        let url = "https://api.openai.com/v1/chat/completions";
        let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());

        let payload = json!({
            "model": model,
            "messages": [
                { "role": "system", "content": system_prompt },
                { "role": "user", "content": history }
            ],
            "response_format": { "type": "json_object" }
        });

        let response = self.client.post(url)
            .bearer_auth(key)
            .json(&payload)
            .send()
            .map_err(|e| format!("OpenAI connection error: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("OpenAI returned HTTP error: {}", response.status()));
        }

        let body: Value = response.json().map_err(|e| format!("OpenAI JSON parse error: {}", e))?;
        let content_str = body.get("choices")
            .and_then(|c| c.as_array())
            .and_then(|a| a.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|msg| msg.get("content"))
            .and_then(|c| c.as_str())
            .ok_or("Invalid response format from OpenAI")?;

        serde_json::from_str(content_str)
            .map_err(|e| format!("Failed to parse LLM response as JSON action: {}\nRaw: {}", e, content_str))
    }

    fn call_anthropic(&self, key: &str, system_prompt: &str, history: &str) -> Result<Value, String> {
        let url = "https://api.anthropic.com/v1/messages";
        let model = env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| "claude-3-5-sonnet-latest".to_string());

        let payload = json!({
            "model": model,
            "max_tokens": 4000,
            "system": system_prompt,
            "messages": [
                { "role": "user", "content": history }
            ]
        });

        let response = self.client.post(url)
            .header("x-api-key", key)
            .header("anthropic-version", "2023-06-01")
            .json(&payload)
            .send()
            .map_err(|e| format!("Anthropic connection error: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Anthropic returned HTTP error: {}", response.status()));
        }

        let body: Value = response.json().map_err(|e| format!("Anthropic JSON parse error: {}", e))?;
        let content_str = body.get("content")
            .and_then(|c| c.as_array())
            .and_then(|a| a.first())
            .and_then(|item| item.get("text"))
            .and_then(|t| t.as_str())
            .ok_or("Invalid response format from Anthropic")?;

        // Find JSON block if Anthropic added markdown formatting
        let json_start = content_str.find('{').ok_or("No JSON object found in LLM response")?;
        let json_end = content_str.rfind('}').ok_or("No JSON object found in LLM response")? + 1;
        let clean_json = &content_str[json_start..json_end];

        serde_json::from_str(clean_json)
            .map_err(|e| format!("Failed to parse LLM response as JSON action: {}\nRaw: {}", e, clean_json))
    }
}
