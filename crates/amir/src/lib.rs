pub mod context;
pub mod feedback;
pub mod generator;
pub mod intent;
pub mod llm;
pub mod orchestrator;
pub mod prompt_parser;
pub mod reverse;
pub mod templates;
pub mod types;

use context::SessionManager;
use generator::OmcomGenerator;
use intent::IntentClassifier;
use llm::{create_provider, ProviderConfig};
use orchestrator::Orchestrator;
use prompt_parser::PromptParser;
use reverse::ReverseParser;
use std::sync::Arc;

pub use types::*;

/// AMIR 引擎 — 一切从这里开始
pub struct AmirEngine {
    prompt_parser: PromptParser,
    intent_classifier: IntentClassifier,
    orchestrator: Orchestrator,
    generator: OmcomGenerator,
    reverse_parser: ReverseParser,
    session_manager: SessionManager,
}

impl AmirEngine {
    /// 创建一个新的 AMIR 引擎实例
    pub fn new(config: AmirConfig) -> Result<Self, AmirError> {
        let provider = create_provider(&config.to_provider_config())?;
        let provider = Arc::from(provider);

        Ok(Self {
            prompt_parser: PromptParser::new(Arc::clone(&provider)),
            intent_classifier: IntentClassifier::new(),
            orchestrator: Orchestrator::new(),
            generator: OmcomGenerator::new(),
            reverse_parser: ReverseParser::new(provider),
            session_manager: SessionManager::new(),
        })
    }

    /// 正向翻译：自然语言 → Omcom 指令
    pub async fn translate(
        &self,
        text: &str,
        selected_ids: &[String],
    ) -> Result<AmirTranslateResponse, AmirError> {
        self.translate_with_context(text, selected_ids, None).await
    }

    /// 正向翻译（带项目上下文）
    pub async fn translate_with_context(
        &self,
        text: &str,
        selected_ids: &[String],
        project_context: Option<&str>,
    ) -> Result<AmirTranslateResponse, AmirError> {
        let parsed = self.prompt_parser.parse(text, selected_ids, project_context).await?;
        let intents = self.intent_classifier.classify(parsed).await?;

        let (ordered_intents, conflicts, _plan) = self.orchestrator.orchestrate(std::slice::from_ref(&intents))?;

        let mut commands = Vec::new();
        for intent in &ordered_intents {
            let cmds = self.generator.generate(intent)?;
            commands.extend(cmds);
        }

        let errors: Vec<String> = conflicts
            .iter()
            .map(|c| format!("[{}] {}", Into::<&str>::into(&c.severity), c.message))
            .collect();

        let summary = self
            .reverse_parser
            .commands_to_nl(&commands)
            .await?;

        let cmd_values: Vec<serde_json::Value> = commands
            .iter()
            .map(|c| serde_json::to_value(c).unwrap_or_default())
            .collect();

        Ok(AmirTranslateResponse {
            success: errors.is_empty(),
            commands: cmd_values,
            summary,
            errors,
        })
    }

    /// 多轮对话
    pub async fn chat(
        &self,
        session_id: Option<String>,
        text: &str,
        selected_ids: &[String],
    ) -> Result<AmirChatResponse, AmirError> {
        self.chat_with_context(session_id, text, selected_ids, None).await
    }

    /// 多轮对话（带项目上下文）
    pub async fn chat_with_context(
        &self,
        session_id: Option<String>,
        text: &str,
        selected_ids: &[String],
        project_context: Option<&str>,
    ) -> Result<AmirChatResponse, AmirError> {
        let session_id = self.session_manager.get_or_create(session_id);
        self.session_manager.add_user_message(&session_id, text);

        let context_text = self.session_manager.build_context(&session_id);

        let full_context = match project_context {
            Some(ctx) if !ctx.is_empty() => {
                Some(format!("{}\n\nPrevious conversation:\n{}", ctx, context_text))
            }
            _ => Some(format!("Previous conversation:\n{}", context_text)),
        };

        let parsed = self.prompt_parser.parse(text, selected_ids, full_context.as_deref()).await?;
        let intents = self.intent_classifier.classify(parsed).await?;

        let (ordered_intents, _conflicts, _plan) = self.orchestrator.orchestrate(std::slice::from_ref(&intents))?;

        let mut commands = Vec::new();
        for intent in &ordered_intents {
            let cmds = self.generator.generate(intent)?;
            commands.extend(cmds);
        }

        let summary = self
            .reverse_parser
            .commands_to_nl(&commands)
            .await?;

        let cmd_values: Vec<serde_json::Value> = commands
            .iter()
            .map(|c| serde_json::to_value(c).unwrap_or_default())
            .collect();

        self.session_manager.add_assistant_message(&session_id, &summary);

        Ok(AmirChatResponse {
            session_id,
            reply: summary.clone(),
            commands: cmd_values,
            summary,
        })
    }

    /// 反向描述：模型 → 自然语言
    pub async fn describe(
        &self,
        object_ids: Option<&[String]>,
        format: &str,
    ) -> Result<DescribeResponse, AmirError> {
        let description = self.reverse_parser.describe(object_ids, format).await?;
        Ok(DescribeResponse { description })
    }

    /// 验证错误 → 自然语言
    pub fn explain_errors(&self, errors: &[String], context: Option<&str>) -> ExplainResponse {
        let explanations: Vec<String> = errors
            .iter()
            .map(|e| feedback::translate_error(e, context.unwrap_or("")))
            .collect();
        ExplainResponse { explanations }
    }
}

/// AMIR 配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AmirConfig {
    /// LLM 供应商： "openai" 或 "anthropic"
    pub provider: String,
    /// API key (可从环境变量读取)
    pub api_key: Option<String>,
    /// 模型名称（可选，使用供应商默认值）
    pub model: Option<String>,
    /// 自定义 base_url（可选，用于代理或自部署兼容 API）
    pub base_url: Option<String>,
    /// 温度参数
    pub temperature: Option<f64>,
    /// 最大 token 数
    pub max_tokens: Option<u32>,
}

impl AmirConfig {
    /// 从环境变量创建配置
    pub fn from_env() -> Result<Self, AmirError> {
        let provider = std::env::var("AMIR_PROVIDER")
            .unwrap_or_else(|_| "mock".to_string());

        Ok(Self {
            provider,
            api_key: std::env::var("AMIR_API_KEY").ok(),
            model: std::env::var("AMIR_MODEL").ok(),
            base_url: std::env::var("AMIR_BASE_URL").ok(),
            temperature: std::env::var("AMIR_TEMPERATURE")
                .ok()
                .and_then(|v| v.parse().ok()),
            max_tokens: std::env::var("AMIR_MAX_TOKENS")
                .ok()
                .and_then(|v| v.parse().ok()),
        })
    }

    pub fn to_provider_config(&self) -> ProviderConfig {
        ProviderConfig {
            provider: self.provider.clone(),
            api_key: self.api_key.clone(),
            model: self.model.clone(),
            base_url: self.base_url.clone(),
            temperature: self.temperature.unwrap_or(0.1),
            max_tokens: self.max_tokens.unwrap_or(1024),
        }
    }
}

impl Default for AmirConfig {
    fn default() -> Self {
        Self {
            provider: "mock".to_string(),
            api_key: None,
            model: None,
            base_url: None,
            temperature: Some(0.1),
            max_tokens: Some(1024),
        }
    }
}
