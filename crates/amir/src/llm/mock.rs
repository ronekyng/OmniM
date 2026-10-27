use super::LlmProvider;
use crate::AmirError;
use async_trait::async_trait;
use std::collections::HashMap;

pub struct MockProvider {
    responses: HashMap<&'static str, String>,
}

impl MockProvider {
    pub fn new() -> Self {
        let mut responses = HashMap::new();

        responses.insert(
            "create a wall",
            r#"{"intent":"create_object","domain":"building","object_type":"wall","object_name":"Wall-1","parameters":{"length":5.0,"height":3.0,"thickness":0.2},"entities":[{"entity_type":"object_type","value":"wall"},{"entity_type":"dimension","value":"5m"},{"entity_type":"dimension","value":"3m"}],"confidence":0.95}"#.into(),
        );
        responses.insert(
            "create wall",
            r#"{"intent":"create_object","domain":"building","object_type":"wall","object_name":"Wall-1","parameters":{"length":5.0,"height":3.0,"thickness":0.2},"entities":[{"entity_type":"object_type","value":"wall"},{"entity_type":"dimension","value":"5m"},{"entity_type":"dimension","value":"3m"}],"confidence":0.95}"#.into(),
        );
        responses.insert(
            "create wall 5m",
            r#"{"intent":"create_object","domain":"building","object_type":"wall","object_name":"Wall-1","parameters":{"length":5.0,"height":3.0,"thickness":0.2},"entities":[{"entity_type":"object_type","value":"wall"},{"entity_type":"dimension","value":"5m"}],"confidence":0.95}"#.into(),
        );
        responses.insert(
            "create a beam",
            r#"{"intent":"create_object","domain":"building","object_type":"beam","object_name":"Beam-1","parameters":{"length":6.0,"width":0.3,"height":0.5},"entities":[{"entity_type":"object_type","value":"beam"}],"confidence":0.95}"#.into(),
        );
        responses.insert(
            "create beam",
            r#"{"intent":"create_object","domain":"building","object_type":"beam","object_name":"Beam-1","parameters":{"length":6.0,"width":0.3,"height":0.5},"entities":[{"entity_type":"object_type","value":"beam"}],"confidence":0.95}"#.into(),
        );
        responses.insert(
            "create a column",
            r#"{"intent":"create_object","domain":"building","object_type":"column","object_name":"Column-1","parameters":{"height":3.0,"width":0.4,"depth":0.4},"entities":[{"entity_type":"object_type","value":"column"}],"confidence":0.95}"#.into(),
        );
        responses.insert(
            "create column",
            r#"{"intent":"create_object","domain":"building","object_type":"column","object_name":"Column-1","parameters":{"height":3.0,"width":0.4,"depth":0.4},"entities":[{"entity_type":"object_type","value":"column"}],"confidence":0.95}"#.into(),
        );
        responses.insert(
            "create slab",
            r#"{"intent":"create_object","domain":"building","object_type":"slab","object_name":"Slab-1","parameters":{"length":10.0,"width":8.0,"thickness":0.2},"entities":[{"entity_type":"object_type","value":"slab"}],"confidence":0.95}"#.into(),
        );
        responses.insert(
            "create road",
            r#"{"intent":"create_object","domain":"civil","object_type":"road","object_name":"Road-1","parameters":{"length":100.0,"width":7.0,"thickness":0.3},"entities":[{"entity_type":"object_type","value":"road"}],"confidence":0.95}"#.into(),
        );
        responses.insert(
            "create pipe",
            r#"{"intent":"create_object","domain":"plant","object_type":"pipe","object_name":"Pipe-1","parameters":{"length":10.0,"radius":0.15},"entities":[{"entity_type":"object_type","value":"pipe"}],"confidence":0.95}"#.into(),
        );

        responses.insert(
            "delete",
            r#"{"intent":"delete_object","domain":"general","target_ref":"selected","entities":[{"entity_type":"object_reference","value":"selected"}],"confidence":0.9}"#.into(),
        );

        responses.insert(
            "modify",
            r#"{"intent":"modify_object","domain":"general","target_ref":"selected","parameters":{"name":"Modified-1"},"entities":[{"entity_type":"object_reference","value":"selected"}],"confidence":0.85}"#.into(),
        );

        responses.insert(
            "user input: create a wall",
            r#"{"intent":"create_object","domain":"building","object_type":"wall","object_name":"MainWall","parameters":{"length":5.0,"height":3.0,"thickness":0.2},"entities":[{"entity_type":"object_type","value":"wall"}],"confidence":0.95}"#.into(),
        );
        responses.insert(
            "user input: create a wall named mainwall",
            r#"{"intent":"create_object","domain":"building","object_type":"wall","object_name":"MainWall","parameters":{"length":5.0,"height":3.0},"entities":[{"entity_type":"object_type","value":"wall"}],"confidence":0.95}"#.into(),
        );

        responses.insert(
            "undo",
            r#"{"intent":"undo","domain":"general","entities":[],"confidence":0.95}"#.into(),
        );
        responses.insert(
            "redo",
            r#"{"intent":"redo","domain":"general","entities":[],"confidence":0.95}"#.into(),
        );

        responses.insert(
            "how many",
            r#"{"intent":"query_model","domain":"general","parameters":{"query":"count all objects"},"entities":[],"confidence":0.8}"#.into(),
        );

        Self { responses }
    }

    fn find_mock_response(&self, prompt: &str) -> Option<String> {
        let lower = prompt.to_lowercase();
        let normalized = lower
            .replace(" a ", " ")
            .replace(" an ", " ")
            .replace(" the ", " ");
        let normalized = normalized.split_whitespace().collect::<Vec<_>>().join(" ");

        for variant in &[lower.as_str(), normalized.as_str()] {
            for (key, value) in &self.responses {
                if variant.contains(key) {
                    return Some(value.clone());
                }
            }
        }

        for prefix in &["user input: ", "user: "] {
            if let Some(rest) = lower.strip_prefix(prefix) {
                let rest_normalized = rest
                    .replace(" a ", " ")
                    .replace(" an ", " ")
                    .replace(" the ", " ");
                let rest_normalized = rest_normalized.split_whitespace().collect::<Vec<_>>().join(" ");
                for variant in &[rest, rest_normalized.as_str()] {
                    for (key, value) in &self.responses {
                        if variant.contains(key) {
                            return Some(value.clone());
                        }
                    }
                }
            }
        }

        None
    }

    fn fallback_response(&self, user_prompt: &str) -> String {
        serde_json::json!({
            "intent": "unknown",
            "domain": "general",
            "original_text": user_prompt,
            "parameters": {},
            "modifications": {},
            "entities": [],
            "confidence": 0.0,
        })
        .to_string()
    }
}

#[async_trait]
impl LlmProvider for MockProvider {
    async fn structured_completion(
        &self,
        _system_prompt: &str,
        user_prompt: &str,
    ) -> Result<serde_json::Value, AmirError> {
        let response = self
            .find_mock_response(user_prompt)
            .unwrap_or_else(|| self.fallback_response(user_prompt));

        serde_json::from_str(&response).map_err(|e| {
            AmirError::ParseError(format!("Mock parse error: {}", e))
        })
    }

    async fn completion(
        &self,
        _system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, AmirError> {
        Ok(self
            .find_mock_response(user_prompt)
            .unwrap_or_else(|| format!("Mock response for: {}", user_prompt)))
    }

    fn provider_name(&self) -> &str {
        "mock"
    }
}