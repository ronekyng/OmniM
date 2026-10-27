mod anthropic;
mod mock;
mod openai;

use crate::AmirError;
use async_trait::async_trait;

/// LLM 供应商抽象
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// 请求结构化输出（JSON mode / function calling）
    async fn structured_completion(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<serde_json::Value, AmirError>;

    /// 请求自由文本输出
    async fn completion(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, AmirError>;

    /// 供应商名称
    fn provider_name(&self) -> &str;
}

/// 供应商配置
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub temperature: f64,
    pub max_tokens: u32,
}

/// 带指数退避重试的 LLM 供应商包装器
pub struct RetryProvider {
    inner: Box<dyn LlmProvider>,
    max_retries: u32,
}

impl RetryProvider {
    pub fn new(inner: Box<dyn LlmProvider>, max_retries: u32) -> Self {
        Self { inner, max_retries }
    }
}

#[async_trait]
impl LlmProvider for RetryProvider {
    async fn structured_completion(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<serde_json::Value, AmirError> {
        let mut last_err = None;
        for attempt in 0..=self.max_retries {
            match self.inner.structured_completion(system_prompt, user_prompt).await {
                Ok(val) => return Ok(val),
                Err(e) => {
                    // 仅重试网络类错误，解析/配置错误直接透传
                    if !matches!(&e, AmirError::LlmError(_)) {
                        return Err(e);
                    }
                    if attempt < self.max_retries {
                        let delay = std::time::Duration::from_millis(500 * (1 << attempt));
                        tokio::time::sleep(delay).await;
                    }
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.unwrap_or_else(|| AmirError::LlmError("All retries exhausted".into())))
    }

    async fn completion(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, AmirError> {
        let mut last_err = None;
        for attempt in 0..=self.max_retries {
            match self.inner.completion(system_prompt, user_prompt).await {
                Ok(val) => return Ok(val),
                Err(e) => {
                    if !matches!(&e, AmirError::LlmError(_)) {
                        return Err(e);
                    }
                    if attempt < self.max_retries {
                        let delay = std::time::Duration::from_millis(500 * (1 << attempt));
                        tokio::time::sleep(delay).await;
                    }
                    last_err = Some(e);
                }
            }
        }
        Err(last_err.unwrap_or_else(|| AmirError::LlmError("All retries exhausted".into())))
    }

    fn provider_name(&self) -> &str {
        self.inner.provider_name()
    }
}

/// 根据配置创建 LLM 供应商实例（生产供应商自动启用重试）
pub fn create_provider(config: &ProviderConfig) -> Result<Box<dyn LlmProvider>, AmirError> {
    let inner: Box<dyn LlmProvider> = match config.provider.to_lowercase().as_str() {
        "openai" => Box::new(openai::OpenAiProvider::new(config)),
        "anthropic" => Box::new(anthropic::AnthropicProvider::new(config)),
        "mock" => Box::new(mock::MockProvider::new()),
        _ => return Err(AmirError::ProviderNotConfigured(format!(
            "Unknown provider '{}'. Supported: openai, anthropic, mock",
            config.provider
        ))),
    };
    // mock provider 不需要重试
    if config.provider == "mock" {
        Ok(inner)
    } else {
        Ok(Box::new(RetryProvider::new(inner, 3)))
    }
}
