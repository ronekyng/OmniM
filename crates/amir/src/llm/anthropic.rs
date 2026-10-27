use super::{LlmProvider, ProviderConfig};
use crate::AmirError;
use async_trait::async_trait;
use serde_json::json;

pub struct AnthropicProvider {
    api_key: String,
    model: String,
    base_url: String,
    temperature: f64,
    max_tokens: u32,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(config: &ProviderConfig) -> Self {
        Self {
            api_key: config
                .api_key
                .clone()
                .unwrap_or_else(|| std::env::var("ANTHROPIC_API_KEY").unwrap_or_default()),
            model: config
                .model
                .clone()
                .unwrap_or_else(|| "claude-sonnet-4-20250514".into()),
            base_url: config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.anthropic.com/v1".into()),
            temperature: config.temperature,
            max_tokens: config.max_tokens,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    async fn messages_api(&self, system: &str, messages: Vec<serde_json::Value>) -> Result<String, AmirError> {
        let url = format!("{}/messages", self.base_url);

        let body = json!({
            "model": self.model,
            "system": system,
            "messages": messages,
            "max_tokens": self.max_tokens,
            "temperature": self.temperature,
        });

        let resp = self
            .client
            .post(&url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AmirError::LlmError(format!("Anthropic request failed: {}", e)))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| AmirError::LlmError(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            return Err(AmirError::LlmError(format!(
                "Anthropic returned {}: {}",
                status, text
            )));
        }

        let json: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| AmirError::ParseError(format!("Invalid JSON response: {}", e)))?;

        let content = json["content"]
            .as_array()
            .and_then(|arr| {
                arr.iter().find(|c| c["type"] == "text")
            })
            .and_then(|c| c["text"].as_str())
            .ok_or_else(|| {
                AmirError::ParseError("No text content in Anthropic response".into())
            })?;

        Ok(content.to_string())
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    async fn structured_completion(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<serde_json::Value, AmirError> {
        let messages = vec![
            json!({
                "role": "user",
                "content": user_prompt
            }),
        ];

        let content = self.messages_api(system_prompt, messages).await?;

        // Try to extract JSON from the response
        let json_str = crate::llm::openai::extract_json(&content).unwrap_or(&content);

        let parsed: serde_json::Value = serde_json::from_str(json_str).map_err(|e| {
            AmirError::ParseError(format!(
                "Failed to parse structured response: {}. Raw: {}",
                e,
                &content[..content.len().min(200)]
            ))
        })?;

        Ok(parsed)
    }

    async fn completion(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, AmirError> {
        let messages = vec![
            json!({
                "role": "user",
                "content": user_prompt
            }),
        ];

        self.messages_api(system_prompt, messages).await
    }

    fn provider_name(&self) -> &str {
        "anthropic"
    }
}
