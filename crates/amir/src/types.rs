use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================
// Intent types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Intent {
    CreateObject {
        object_type: String,
        name: Option<String>,
        parameters: HashMap<String, serde_json::Value>,
    },
    ModifyObject {
        target_ref: String,
        modifications: HashMap<String, serde_json::Value>,
    },
    DeleteObject {
        target_ref: String,
    },
    MoveObject {
        target_ref: String,
        new_parent_ref: String,
    },
    SetProperty {
        target_ref: String,
        property_name: String,
        property_value: serde_json::Value,
    },
    AddContainment {
        parent_ref: String,
        child_ref: String,
    },
    RemoveContainment {
        parent_ref: String,
        child_ref: String,
    },
    AddConnection {
        from_ref: String,
        to_ref: String,
        connection_type: Option<String>,
    },
    CreateLifecycleRecord {
        phase: String,
        record_type: String,
        name: Option<String>,
        description: Option<String>,
    },
    ModifyLifecycleRecord {
        record_ref: String,
        modifications: HashMap<String, serde_json::Value>,
    },
    DeleteLifecycleRecord {
        record_ref: String,
    },
    LinkRecordToObject {
        record_ref: String,
        object_ref: String,
    },
    UpdateSchedule {
        record_ref: String,
    },
    UpdateCost {
        record_ref: String,
    },
    UpdateProgress {
        record_ref: String,
    },
    CreateWorkOrder {
        asset_ref: String,
        title: Option<String>,
    },
    DefineComponent {
        name: String,
        category: String,
        description: Option<String>,
    },
    PublishComponent {
        component_ref: String,
    },
    CreateSeries {
        name: String,
        description: Option<String>,
        domain: Option<String>,
    },
    SelectFromSeries {
        series_ref: String,
        answers: Vec<String>,
    },
    CreateSketch {
        parent_ref: String,
        plane: String,
    },
    ExtrudeSketch {
        sketch_ref: String,
        depth: f64,
        direction: Option<String>,
    },
    RevolveSketch {
        sketch_ref: String,
        angle: f64,
    },
    AddFeature {
        parent_ref: String,
        feature_type: String,
        parameters: HashMap<String, serde_json::Value>,
    },
    CreateComponent {
        name: String,
    },
    BooleanOp {
        target_ref: String,
        tool_ref: String,
        operation: String,
    },
    ImportFile {
        format: String,
    },
    ExportFile {
        format: String,
    },
    QueryModel {
        query: String,
    },
    Undo,
    Redo,
    Unknown {
        original_text: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Domain {
    Building,
    Civil,
    Plant,
    Mechanical,
    General,
}

impl Domain {
    pub fn as_str(&self) -> &'static str {
        match self {
            Domain::Building => "building",
            Domain::Civil => "civil",
            Domain::Plant => "plant",
            Domain::Mechanical => "mechanical",
            Domain::General => "general",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "building" => Domain::Building,
            "civil" => Domain::Civil,
            "plant" => Domain::Plant,
            "mechanical" => Domain::Mechanical,
            _ => Domain::General,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub entity_type: EntityType,
    pub value: String,
    pub span: Option<(usize, usize)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityType {
    ObjectType,
    ObjectReference,
    PropertyName,
    PropertyValue,
    Unit,
    Dimension,
    SpatialReference,
    Phase,
    RecordType,
    Format,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedIntent {
    pub intent: Intent,
    pub confidence: f64,
    pub original_text: String,
    pub entities: Vec<Entity>,
    pub domain: Domain,
}

impl ParsedIntent {
    pub fn new(intent: Intent, text: &str, confidence: f64) -> Self {
        let domain = detect_domain(&intent);
        Self {
            intent,
            confidence,
            original_text: text.to_string(),
            entities: Vec::new(),
            domain,
        }
    }
}

fn detect_domain(intent: &Intent) -> Domain {
    match intent {
        Intent::CreateObject { object_type, .. } => match object_type.to_lowercase().as_str() {
            "wall" | "beam" | "column" | "slab" | "door" | "window" | "stair" | "roof"
            | "floor" | "building" | "storey" | "space" | "site" => Domain::Building,
            "road" | "bridge" | "tunnel" | "dam" | "culvert" | "retaining_wall" => {
                Domain::Civil
            }
            "tank" | "pump" | "valve" | "heat_exchanger" | "reactor" | "pipe" => {
                Domain::Plant
            }
            "part" | "assembly" | "gear" | "shaft" | "bearing" | "motor" | "fastener" => {
                Domain::Mechanical
            }
            _ => Domain::General,
        },
        _ => Domain::General,
    }
}

// ============================================================
// LLM 结构化输出 schema
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmIntentResponse {
    pub intent: String,
    pub domain: String,
    pub object_type: Option<String>,
    pub object_name: Option<String>,
    pub target_ref: Option<String>,
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub modifications: HashMap<String, serde_json::Value>,
    pub entities: Vec<LlmEntity>,
    pub confidence: Option<f64>,
    pub explanation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmEntity {
    pub entity_type: String,
    pub value: String,
}

// ============================================================
// AI Chat / Context types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationContext {
    pub session_id: String,
    pub messages: Vec<Message>,
    pub selected_object_ids: Vec<String>,
}

// ============================================================
// Errors
// ============================================================

#[derive(Debug, thiserror::Error)]
pub enum AmirError {
    #[error("LLM request failed: {0}")]
    LlmError(String),

    #[error("Failed to parse LLM response: {0}")]
    ParseError(String),

    #[error("Unsupported intent: {0}")]
    UnsupportedIntent(String),

    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    #[error("Reference not found: {0}")]
    ReferenceNotFound(String),

    #[error("Provider not configured: {0}")]
    ProviderNotConfigured(String),

    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("Planning error: {0}")]
    PlanningError(String),

    #[error("Conflict detected: {0}")]
    ConflictError(String),
}

// ============================================================
// Orchestrator types
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecomposedIntent {
    pub sub_intents: Vec<ParsedIntent>,
    pub domains: Vec<Domain>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictSeverity {
    Error,
    Warning,
    Info,
}

impl From<&ConflictSeverity> for &'static str {
    fn from(s: &ConflictSeverity) -> Self {
        match s {
            ConflictSeverity::Error => "ERROR",
            ConflictSeverity::Warning => "WARNING",
            ConflictSeverity::Info => "INFO",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    pub severity: ConflictSeverity,
    pub message: String,
    pub involved_intents: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanStepKind {
    Sequential,
    Parallel,
    Execute { intent_index: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub steps: Vec<PlanStepKind>,
    pub ordering_notes: Vec<String>,
}

// ============================================================
// General helper types
// ============================================================

/// Request: natural language → Omcom commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmirTranslateRequest {
    /// The natural language user input
    pub text: String,
    /// Object IDs currently selected in the UI (for context)
    pub selected_ids: Option<Vec<String>>,
}

/// Request: multi-turn conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmirChatRequest {
    /// Session ID for continuing a conversation (None = new session)
    pub session_id: Option<String>,
    /// The natural language user input
    pub text: String,
    /// Object IDs currently selected in the UI (for context)
    pub selected_ids: Option<Vec<String>>,
}

/// Response from a chat turn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmirChatResponse {
    /// Session ID for subsequent turns
    pub session_id: String,
    /// Natural language reply
    pub reply: String,
    /// Generated Omcom commands (as JSON values)
    pub commands: Vec<serde_json::Value>,
    /// Human-readable summary of the commands
    pub summary: String,
}

/// Response from a translate request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmirTranslateResponse {
    /// Whether the translation succeeded
    pub success: bool,
    /// Generated Omcom commands (as JSON values)
    pub commands: Vec<serde_json::Value>,
    /// Human-readable summary of the commands
    pub summary: String,
    /// Any errors encountered during translation
    pub errors: Vec<String>,
}

/// Request: describe model objects in natural language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescribeRequest {
    /// Object IDs to describe (None = describe whole project)
    pub object_ids: Option<Vec<String>>,
    /// Output format ("text" or "json")
    pub format: Option<String>,
}

/// Response from a describe request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DescribeResponse {
    /// Natural language description
    pub description: String,
}

/// Request: explain validation errors in natural language
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainRequest {
    /// Raw error messages from the Omcom validator
    pub errors: Vec<String>,
    /// Optional context (e.g. current operation name)
    pub context: Option<String>,
}

/// Response from an explain request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainResponse {
    /// Human-readable explanations
    pub explanations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_detect_building() {
        let intent = Intent::CreateObject { object_type: "wall".into(), name: None, parameters: HashMap::new() };
        let domain = detect_domain(&intent);
        assert_eq!(domain, Domain::Building);
    }

    #[test]
    fn test_domain_detect_civil() {
        let intent = Intent::CreateObject { object_type: "road".into(), name: None, parameters: HashMap::new() };
        let domain = detect_domain(&intent);
        assert_eq!(domain, Domain::Civil);
    }

    #[test]
    fn test_domain_detect_plant() {
        let intent = Intent::CreateObject { object_type: "pipe".into(), name: None, parameters: HashMap::new() };
        let domain = detect_domain(&intent);
        assert_eq!(domain, Domain::Plant);
    }

    #[test]
    fn test_domain_detect_mechanical() {
        let intent = Intent::CreateObject { object_type: "gear".into(), name: None, parameters: HashMap::new() };
        let domain = detect_domain(&intent);
        assert_eq!(domain, Domain::Mechanical);
    }

    #[test]
    fn test_domain_detect_general_for_delete() {
        let intent = Intent::DeleteObject { target_ref: "obj-1".into() };
        let domain = detect_domain(&intent);
        assert_eq!(domain, Domain::General);
    }

    #[test]
    fn test_domain_from_str() {
        assert_eq!(Domain::from_str("building"), Domain::Building);
        assert_eq!(Domain::from_str("CIVIL"), Domain::Civil);
        assert_eq!(Domain::from_str("Plant"), Domain::Plant);
        assert_eq!(Domain::from_str("mechanical"), Domain::Mechanical);
        assert_eq!(Domain::from_str("unknown"), Domain::General);
    }

    #[test]
    fn test_domain_as_str() {
        assert_eq!(Domain::Building.as_str(), "building");
        assert_eq!(Domain::General.as_str(), "general");
    }

    #[test]
    fn test_parsed_intent_new() {
        let intent = Intent::Undo;
        let parsed = ParsedIntent::new(intent.clone(), "undo that", 0.95);
        assert_eq!(parsed.intent, intent);
        assert_eq!(parsed.confidence, 0.95);
        assert_eq!(parsed.original_text, "undo that");
        assert!(parsed.entities.is_empty());
    }
}
