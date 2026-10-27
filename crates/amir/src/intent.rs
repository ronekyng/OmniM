use crate::types::*;

/// 意图分类器 — 将 LLM 的结构化输出转为 ParsedIntent
pub struct IntentClassifier {
}

impl IntentClassifier {
    pub fn new() -> Self {
        Self {
        }
    }

    /// 将原始 LLM 响应分类为 ParsedIntent
    pub async fn classify(
        &self,
        llm_response: LlmIntentResponse,
    ) -> Result<ParsedIntent, AmirError> {
        let intent = self.map_intent(&llm_response)?;
        let confidence = llm_response.confidence.unwrap_or(0.5);

        let mut parsed = ParsedIntent::new(intent, "", confidence);
        parsed.entities = llm_response
            .entities
            .iter()
            .map(|e| Entity {
                entity_type: match e.entity_type.to_lowercase().as_str() {
                    "object_type" => EntityType::ObjectType,
                    "object_reference" | "object_ref" | "reference" => {
                        EntityType::ObjectReference
                    }
                    "property_name" => EntityType::PropertyName,
                    "property_value" => EntityType::PropertyValue,
                    "unit" => EntityType::Unit,
                    "dimension" => EntityType::Dimension,
                    "spatial_reference" => EntityType::SpatialReference,
                    "phase" => EntityType::Phase,
                    "record_type" => EntityType::RecordType,
                    "format" => EntityType::Format,
                    _ => EntityType::Unknown,
                },
                value: e.value.clone(),
                span: None,
            })
            .collect();
        parsed.domain = Domain::from_str(&llm_response.domain);

        Ok(parsed)
    }

    fn map_intent(&self, resp: &LlmIntentResponse) -> Result<Intent, AmirError> {
        match resp.intent.to_lowercase().as_str() {
            "create_object" => {
                let object_type = resp
                    .object_type
                    .clone()
                    .unwrap_or_else(|| "generic".into());
                let mut params = resp.parameters.clone();

                // 如果没有提供 name，自动生成
                if params.get("name").and_then(|v| v.as_str()).unwrap_or("").is_empty() {
                    if let Some(name) = &resp.object_name {
                        params.insert(
                            "name".into(),
                            serde_json::Value::String(name.clone()),
                        );
                    }
                }

                Ok(Intent::CreateObject {
                    object_type,
                    name: resp.object_name.clone(),
                    parameters: params,
                })
            }
            "modify_object" => Ok(Intent::ModifyObject {
                target_ref: resp
                    .target_ref
                    .clone()
                    .unwrap_or_else(|| "selected".into()),
                modifications: resp.parameters.clone(),
            }),
            "delete_object" => Ok(Intent::DeleteObject {
                target_ref: resp
                    .target_ref
                    .clone()
                    .unwrap_or_else(|| "selected".into()),
            }),
            "move_object" => Ok(Intent::MoveObject {
                target_ref: resp
                    .target_ref
                    .clone()
                    .unwrap_or_else(|| "selected".into()),
                new_parent_ref: resp
                    .parameters
                    .get("parent")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            }),
            "set_property" => Ok(Intent::SetProperty {
                target_ref: resp
                    .target_ref
                    .clone()
                    .unwrap_or_else(|| "selected".into()),
                property_name: resp
                    .parameters
                    .get("property_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                property_value: resp
                    .parameters
                    .get("property_value")
                    .cloned()
                    .unwrap_or(serde_json::Value::String(String::new())),
            }),
            "add_containment" => Ok(Intent::AddContainment {
                parent_ref: resp
                    .parameters
                    .get("parent")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                child_ref: resp
                    .parameters
                    .get("child")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            }),
            "remove_containment" => Ok(Intent::RemoveContainment {
                parent_ref: resp
                    .parameters
                    .get("parent")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                child_ref: resp
                    .parameters
                    .get("child")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            }),
            "add_connection" => Ok(Intent::AddConnection {
                from_ref: resp
                    .parameters
                    .get("from")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                to_ref: resp
                    .parameters
                    .get("to")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                connection_type: resp
                    .parameters
                    .get("connection_type")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            }),
            "create_lifecycle_record" => Ok(Intent::CreateLifecycleRecord {
                phase: resp
                    .parameters
                    .get("phase")
                    .and_then(|v| v.as_str())
                    .unwrap_or("planning")
                    .to_string(),
                record_type: resp
                    .parameters
                    .get("record_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("activity")
                    .to_string(),
                name: resp.object_name.clone(),
                description: resp
                    .parameters
                    .get("description")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
            }),
            "modify_lifecycle_record" => Ok(Intent::ModifyLifecycleRecord {
                record_ref: resp
                    .target_ref
                    .clone()
                    .unwrap_or_else(|| "selected".into()),
                modifications: resp.parameters.clone(),
            }),
            "delete_lifecycle_record" => Ok(Intent::DeleteLifecycleRecord {
                record_ref: resp
                    .target_ref
                    .clone()
                    .unwrap_or_else(|| "selected".into()),
            }),
            "link_record_to_object" => Ok(Intent::LinkRecordToObject {
                record_ref: resp
                    .target_ref
                    .clone()
                    .unwrap_or_else(|| "selected".into()),
                object_ref: resp
                    .parameters
                    .get("object")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            }),
            "update_schedule" => Ok(Intent::UpdateSchedule {
                record_ref: resp
                    .target_ref
                    .clone()
                    .unwrap_or_else(|| "selected".into()),
            }),
            "update_cost" => Ok(Intent::UpdateCost {
                record_ref: resp
                    .target_ref
                    .clone()
                    .unwrap_or_else(|| "selected".into()),
            }),
            "update_progress" => Ok(Intent::UpdateProgress {
                record_ref: resp
                    .target_ref
                    .clone()
                    .unwrap_or_else(|| "selected".into()),
            }),
            "create_work_order" => Ok(Intent::CreateWorkOrder {
                asset_ref: resp
                    .parameters
                    .get("asset")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                title: resp.object_name.clone(),
            }),
            "import_file" => Ok(Intent::ImportFile {
                format: resp
                    .parameters
                    .get("format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("ifc")
                    .to_string(),
            }),
            "export_file" => Ok(Intent::ExportFile {
                format: resp
                    .parameters
                    .get("format")
                    .and_then(|v| v.as_str())
                    .unwrap_or("ifc")
                    .to_string(),
            }),
            "create_sketch" => Ok(Intent::CreateSketch {
                parent_ref: resp
                    .target_ref
                    .clone()
                    .unwrap_or_else(|| "selected".into()),
                plane: resp
                    .parameters
                    .get("plane")
                    .and_then(|v| v.as_str())
                    .unwrap_or("xy")
                    .to_string(),
            }),
            "extrude_sketch" => {
                let depth = resp
                    .parameters
                    .get("depth")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(100.0);
                Ok(Intent::ExtrudeSketch {
                    sketch_ref: resp
                        .target_ref
                        .clone()
                        .unwrap_or_else(|| "selected".into()),
                    depth,
                    direction: resp
                        .parameters
                        .get("direction")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                })
            }
            "revolve_sketch" => {
                let angle = resp
                    .parameters
                    .get("angle")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(360.0);
                Ok(Intent::RevolveSketch {
                    sketch_ref: resp
                        .target_ref
                        .clone()
                        .unwrap_or_else(|| "selected".into()),
                    angle,
                })
            }
            "add_feature" => {
                let feature_type = resp
                    .object_type
                    .clone()
                    .unwrap_or_else(|| "extrude".into());
                Ok(Intent::AddFeature {
                    parent_ref: resp
                        .target_ref
                        .clone()
                        .unwrap_or_else(|| "selected".into()),
                    feature_type,
                    parameters: resp.parameters.clone(),
                })
            }
            "create_component" => {
                let name = resp
                    .object_name
                    .clone()
                    .unwrap_or_else(|| "New Component".into());
                Ok(Intent::CreateComponent { name })
            }
            "boolean_op" => {
                let operation = resp
                    .parameters
                    .get("operation")
                    .and_then(|v| v.as_str())
                    .unwrap_or("subtract")
                    .to_string();
                let tool_ref = resp
                    .parameters
                    .get("tool")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(Intent::BooleanOp {
                    target_ref: resp
                        .target_ref
                        .clone()
                        .unwrap_or_else(|| "selected".into()),
                    tool_ref,
                    operation,
                })
            }
            "query_model" => Ok(Intent::QueryModel {
                query: resp
                    .parameters
                    .get("query")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            }),
            "undo" => Ok(Intent::Undo),
            "redo" => Ok(Intent::Redo),
            _ => Ok(Intent::Unknown {
                original_text: resp
                    .parameters
                    .get("original_text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
            }),
        }
    }
}
