use crate::llm::LlmProvider;
use crate::templates;
use crate::types::*;
use std::sync::Arc;

/// Reverse Parser — 模型/指令 → 自然语言描述
pub struct ReverseParser {
    provider: Arc<dyn LlmProvider>,
}

impl ReverseParser {
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self {
            provider,
        }
    }

    /// 生成指令序列的自然语言描述
    pub async fn commands_to_nl(
        &self,
        _commands: &[omnim_core::OmniCommand],
    ) -> Result<String, AmirError> {
        // 简单实现：从指令中提取摘要信息
        // 后续可改为 LLM 生成

        let count = _commands.len();
        if count == 0 {
            return Ok("No operations to perform.".to_string());
        }

        let summaries: Vec<String> = _commands
            .iter()
            .filter_map(|cmd| {
                let action = match omnim_core::Action::try_from(cmd.action) {
                    Ok(a) => a,
                    Err(_) => return None,
                };
                let action_name = match action {
                    omnim_core::Action::CreateObject => {
                        cmd.operations.first().and_then(|op| {
                            if let Some(omnim_core::operation::Op::CreateObject(co)) = &op.op {
                                Some(format!(
                                    "Create {} ({})",
                                    co.name,
                                    object_type_name(
                                        omnim_core::ObjectType::try_from(co.r#type).ok()
                                    )
                                ))
                            } else {
                                None
                            }
                        })
                    }
                    omnim_core::Action::ModifyObject => Some("Modify object".into()),
                    omnim_core::Action::DeleteObject => Some("Delete object".into()),
                    omnim_core::Action::MoveObject => Some("Move object".into()),
                    omnim_core::Action::SetProperty => Some("Set property".into()),
                    omnim_core::Action::AddContainment => Some("Add containment".into()),
                    omnim_core::Action::AddConnection => Some("Add connection".into()),
                    omnim_core::Action::CreateLifecycleRecord => {
                        Some("Create lifecycle record".into())
                    }
                    omnim_core::Action::DeleteLifecycleRecord => {
                        Some("Delete lifecycle record".into())
                    }

                    _ => Some(format!("{:?}", action)),
                };
                action_name
            })
            .collect();

        if summaries.is_empty() {
            Ok("No operations to perform.".to_string())
        } else {
            Ok(format!(
                "The following operations will be performed:\n- {}",
                summaries.join("\n- ")
            ))
        }
    }

    /// 描述对象概况
    pub async fn describe(
        &self,
        object_ids: Option<&[String]>,
        _format: &str,
    ) -> Result<String, AmirError> {
        match object_ids {
            Some(ids) if !ids.is_empty() => {
                Ok(format!("Describing {} object(s): {}", ids.len(), ids.join(", ")))
            }
            _ => Ok("No objects specified.".to_string()),
        }
    }

    /// 使用 LLM 生成丰富的对象描述
    pub async fn describe_with_llm(
        &self,
        object_json: &str,
    ) -> Result<String, AmirError> {
        let prompt = templates::object_description_prompt(object_json);
        self.provider
            .completion(templates::REVERSE_DESCRIPTION_SYSTEM, &prompt)
            .await
    }
}

fn object_type_name(t: Option<omnim_core::ObjectType>) -> &'static str {
    match t {
        Some(omnim_core::ObjectType::Wall) => "Wall",
        Some(omnim_core::ObjectType::Beam) => "Beam",
        Some(omnim_core::ObjectType::Column) => "Column",
        Some(omnim_core::ObjectType::Slab) => "Slab",
        Some(omnim_core::ObjectType::Door) => "Door",
        Some(omnim_core::ObjectType::Window) => "Window",
        Some(omnim_core::ObjectType::Road) => "Road",
        Some(omnim_core::ObjectType::PipePlant) => "Pipe",
        Some(omnim_core::ObjectType::Tank) => "Tank",
        Some(omnim_core::ObjectType::Pump) => "Pump",
        Some(omnim_core::ObjectType::Part) => "Part",
        Some(omnim_core::ObjectType::Assembly) => "Assembly",
        Some(omnim_core::ObjectType::Building) => "Building",
        Some(omnim_core::ObjectType::BuildingStorey) => "Storey",
        Some(omnim_core::ObjectType::Site) => "Site",
        Some(omnim_core::ObjectType::Space) => "Space",
        Some(omnim_core::ObjectType::Generic) => "Generic",
        Some(_) => "Object",
        None => "Object",
    }
}
