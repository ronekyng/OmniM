use crate::llm::LlmProvider;
use crate::templates;
use crate::types::*;
use std::sync::Arc;

/// Prompt Parser — 接收用户自然语言，调用 LLM 提取结构化意图
pub struct PromptParser {
    provider: Arc<dyn LlmProvider>,
}

impl PromptParser {
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self {
            provider,
        }
    }

    /// 解析自然语言输入，返回结构化的 LLM 响应
    pub async fn parse(
        &self,
        text: &str,
        selected_ids: &[String],
        project_context: Option<&str>,
    ) -> Result<LlmIntentResponse, AmirError> {
        // 构建用户 prompt（包含上下文信息）
        let user_prompt = self.build_user_prompt(text, selected_ids, project_context);

        let response_value = self
            .provider
            .structured_completion(
                templates::INTENT_CLASSIFICATION_SYSTEM,
                &user_prompt,
            )
            .await?;

        let response: LlmIntentResponse = serde_json::from_value(response_value)
            .map_err(|e| AmirError::ParseError(format!(
                "Failed to parse intent response: {}", e
            )))?;

        Ok(response)
    }

    /// 构建用户 prompt，注入项目上下文 + 选中对象等信息
    fn build_user_prompt(&self, text: &str, selected_ids: &[String], project_context: Option<&str>) -> String {
        let mut prompt = String::new();

        // 注入项目上下文（模型统计、当前状态等）
        if let Some(ctx) = project_context {
            if !ctx.is_empty() {
                prompt.push_str("Project context:\n");
                prompt.push_str(ctx);
                prompt.push_str("\n\n");
            }
        }

        // 注入选中对象信息
        if !selected_ids.is_empty() {
            prompt.push_str(&format!(
                "Currently selected objects: {}\n\n",
                selected_ids.join(", ")
            ));
        }

        prompt.push_str("User input: ");
        prompt.push_str(text);

        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::ProviderConfig;

    fn mock_provider() -> Arc<dyn LlmProvider> {
        let cfg = ProviderConfig {
            provider: "mock".into(),
            api_key: Some("key".into()),
            model: None,
            base_url: None,
            temperature: 0.1,
            max_tokens: 1024,
        };
        crate::llm::create_provider(&cfg).unwrap().into()
    }

    #[tokio::test]
    async fn test_parse_with_mock() {
        let parser = PromptParser::new(mock_provider());

        let result = parser.parse("User input: create wall", &[], None).await;
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());

        let response = result.unwrap();
        assert_eq!(response.intent, "create_object");
        assert_eq!(response.object_type, Some("wall".into()));
    }

    #[tokio::test]
    async fn test_parse_with_project_context() {
        let parser = PromptParser::new(mock_provider());

        let result = parser.parse("how many objects", &[], Some("Project 'Office' has 15 objects, 3 materials.")).await;
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        let response = result.unwrap();
        assert_eq!(response.intent, "query_model");
    }

    /// Integration test with real provider (requires env vars, skipped otherwise)
    #[tokio::test]
    async fn test_real_provider_integration() {
        let provider_key = match std::env::var("AMIR_API_KEY") {
            Ok(k) if !k.is_empty() && k != "mock-key" => k,
            _ => {
                eprintln!("Skipping real provider test: set AMIR_PROVIDER + AMIR_API_KEY env vars");
                return;
            }
        };
        let provider_name = std::env::var("AMIR_PROVIDER").unwrap_or_else(|_| "openai".into());

        let cfg = ProviderConfig {
            provider: provider_name,
            api_key: Some(provider_key),
            model: std::env::var("AMIR_MODEL").ok(),
            base_url: std::env::var("AMIR_BASE_URL").ok(),
            temperature: 0.1,
            max_tokens: 1024,
        };

        let provider = crate::llm::create_provider(&cfg).unwrap();
        let parser = PromptParser::new(provider.into());

        let result = parser.parse("create a wall 5 meters long", &[], None).await;
        assert!(result.is_ok(), "Real LLM parse failed: {:?}", result.err());
        let response = result.unwrap();
        assert_eq!(response.intent, "create_object");
        assert_eq!(response.object_type.as_deref(), Some("wall"));
    }

    #[tokio::test]
    async fn test_parse_with_mock_direct() {
        let parser = PromptParser::new(mock_provider());

        // Test that "create wall" in the prompt triggers the mock
        let result = parser.parse("create wall", &[], None).await;
        assert!(result.is_ok(), "Parse failed: {:?}", result.err());
        assert_eq!(result.unwrap().intent, "create_object");
    }
}
