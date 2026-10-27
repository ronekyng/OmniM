use super::{LlmProvider, ProviderConfig};
use crate::AmirError;
use async_trait::async_trait;
use serde_json::json;

pub struct OpenAiProvider {
    api_key: String,
    model: String,
    base_url: String,
    temperature: f64,
    max_tokens: u32,
    client: reqwest::Client,
}

impl OpenAiProvider {
    pub fn new(config: &ProviderConfig) -> Self {
        Self {
            api_key: config
                .api_key
                .clone()
                .unwrap_or_else(|| std::env::var("OPENAI_API_KEY").unwrap_or_default()),
            model: config.model.clone().unwrap_or_else(|| "gpt-4o-mini".into()),
            base_url: config
                .base_url
                .clone()
                .unwrap_or_else(|| "https://api.openai.com/v1".into()),
            temperature: config.temperature,
            max_tokens: config.max_tokens,
            client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    async fn chat_completion(&self, messages: Vec<serde_json::Value>) -> Result<String, AmirError> {
        let url = format!("{}/chat/completions", self.base_url);

        let body = json!({
            "model": self.model,
            "messages": messages,
            "temperature": self.temperature,
            "max_tokens": self.max_tokens,
        });

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| AmirError::LlmError(format!("OpenAI request failed: {}", e)))?;

        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|e| AmirError::LlmError(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            return Err(AmirError::LlmError(format!(
                "OpenAI returned {}: {}",
                status, text
            )));
        }

        let json: serde_json::Value =
            serde_json::from_str(&text)
                .map_err(|e| AmirError::ParseError(format!("Invalid JSON response: {}", e)))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| {
                AmirError::ParseError("No content in OpenAI response".into())
            })?;

        Ok(content.to_string())
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn structured_completion(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<serde_json::Value, AmirError> {
        let messages = vec![
            json!({
                "role": "system",
                "content": system_prompt
            }),
            json!({
                "role": "user",
                "content": user_prompt
            }),
        ];

        let content = self.chat_completion(messages).await?;

        // Try to extract JSON from the response (handles markdown code blocks)
        let json_str = extract_json(&content).unwrap_or(&content);

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
                "role": "system",
                "content": system_prompt
            }),
            json!({
                "role": "user",
                "content": user_prompt
            }),
        ];

        self.chat_completion(messages).await
    }

    fn provider_name(&self) -> &str {
        "openai"
    }
}

/// Extract JSON from a string that might be wrapped in markdown code blocks.
pub fn extract_json<'a>(text: &'a str) -> Option<&'a str> {
    // Try ```json ... ```
    if let Some(start) = text.find("```json") {
        let after = &text[start + 7..];
        if let Some(end) = after.find("```") {
            return Some(after[..end].trim());
        }
    }
    // Try ``` ... ```
    if let Some(start) = text.find("```") {
        let after = &text[start + 3..];
        if let Some(end) = after.find("```") {
            return Some(after[..end].trim());
        }
    }
    None
}
