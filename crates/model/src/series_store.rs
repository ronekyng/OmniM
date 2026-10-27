use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// Series - spec decision tree (SPECON)
// ============================================================

fn default_source() -> String {
    "seed".to_string()
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Series {
    pub id: String,
    pub name: String,
    pub description: String,
    pub domain: String,
    pub standard_reference: String,
    pub root_selector: Option<Selector>,
    pub applicable_phases: Vec<String>,
    pub metadata: HashMap<String, String>,
    /// Origin of this series: "project" | "external" | "seed".
    /// Used for resolution priority (project > external > seed).
    #[serde(default = "default_source")]
    pub source: String,
}

impl Series {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            description: String::new(),
            domain: String::new(),
            standard_reference: String::new(),
            root_selector: None,
            applicable_phases: Vec::new(),
            metadata: HashMap::new(),
            source: "project".to_string(),
        }
    }
}

// ============================================================
// Selector - decision node in the series tree
// ============================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Selector {
    pub id: String,
    pub question: String,
    pub answers: Vec<Answer>,
}

impl Selector {
    pub fn new(id: impl Into<String>, question: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            question: question.into(),
            answers: Vec::new(),
        }
    }
}

// ============================================================
// Answer - branch in the selector tree
// ============================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Answer {
    pub id: String,
    pub value: String,
    pub is_numeric_range: bool,
    pub range_min: f64,
    pub range_max: f64,
    pub outcome: AnswerOutcome,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AnswerOutcome {
    NextSelector(Selector),
    Component(SpecComponent),
}

// ============================================================
// SpecComponent - leaf node referencing a ComponentDef with constraints
// ============================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SpecComponent {
    pub id: String,
    pub component_def_id: String,
    pub param_constraints: Vec<ParamConstraint>,
    pub material: String,
    pub connection_type_code: String,
    pub design_params: HashMap<String, String>,
}

impl SpecComponent {
    pub fn new(component_def_id: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            component_def_id: component_def_id.into(),
            param_constraints: Vec::new(),
            material: String::new(),
            connection_type_code: String::new(),
            design_params: HashMap::new(),
        }
    }

    pub fn material(mut self, m: impl Into<String>) -> Self { self.material = m.into(); self }
    pub fn connection_type_code(mut self, c: impl Into<String>) -> Self { self.connection_type_code = c.into(); self }
}

// ============================================================
// ParamConstraint - restricts parameter range in a series
// ============================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ParamConstraint {
    pub param_name: String,
    pub default_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub step: f64,
    pub fixed_value: String,
}

impl ParamConstraint {
    pub fn new(param_name: impl Into<String>) -> Self {
        Self {
            param_name: param_name.into(),
            default_value: 0.0,
            min_value: 0.0,
            max_value: f64::MAX,
            step: 0.0,
            fixed_value: String::new(),
        }
    }

    pub fn with_default(mut self, v: f64) -> Self { self.default_value = v; self }
    pub fn with_range(mut self, min: f64, max: f64) -> Self { self.min_value = min; self.max_value = max; self }
    pub fn with_step(mut self, s: f64) -> Self { self.step = s; self }
    pub fn with_fixed(mut self, f: impl Into<String>) -> Self { self.fixed_value = f.into(); self }
}

// ============================================================
// SeriesStore - in-memory series storage
// ============================================================

#[derive(Serialize, Deserialize, Clone)]
pub struct SeriesStore {
    series: HashMap<String, Series>,
}

impl Default for SeriesStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SeriesStore {
    pub fn new() -> Self {
        Self {
            series: HashMap::new(),
        }
    }

    /// Create a SeriesStore pre-loaded with the built-in seeded series.
    pub fn new_seeded() -> Self {
        let mut store = Self::new();
        crate::seed_catalog::seed_series(&mut store);
        // Mark all seeded series as source "seed"
        for s in store.series.values_mut() {
            s.source = "seed".to_string();
        }
        store
    }

    pub fn insert(&mut self, s: Series) -> &mut Series {
        let id = s.id.clone();
        // Priority: project(3) > external(2) > seed(1)
        // Only insert if the new series has higher or equal priority.
        // If a series with the same ID already exists and has higher priority, keep it.
        let new_priority = Self::source_priority(&s.source);
        if let Some(existing) = self.series.get(&id) {
            let existing_priority = Self::source_priority(&existing.source);
            if new_priority < existing_priority {
                // Keep the higher-priority existing series
                return self.series.get_mut(&id).unwrap();
            }
        }
        self.series.insert(id.clone(), s);
        self.series.get_mut(&id).unwrap()
    }

    /// Return the numeric priority of a series source.
    /// project(3) > external(2) > seed(1) > unknown(0)
    fn source_priority(source: &str) -> u8 {
        match source {
            "project" => 3,
            "external" => 2,
            "seed" => 1,
            _ => 0,
        }
    }

    pub fn get(&self, id: &str) -> Option<&Series> {
        self.series.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Series> {
        self.series.get_mut(id)
    }

    pub fn remove(&mut self, id: &str) -> Option<Series> {
        self.series.remove(id)
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Series> {
        self.series.values().find(|s| s.name == name)
    }

    pub fn list(&self) -> Vec<&Series> {
        self.series.values().collect()
    }

    pub fn list_by_domain(&self, domain: &str) -> Vec<&Series> {
        self.series.values().filter(|s| s.domain == domain).collect()
    }

    pub fn len(&self) -> usize {
        self.series.len()
    }

    pub fn is_empty(&self) -> bool {
        self.series.is_empty()
    }

    /// Check if any series is loaded in this store.
    pub fn is_any_series_loaded(&self) -> bool {
        !self.series.is_empty()
    }

    /// Collect all component_def_ids reachable from all loaded series'
    /// decision trees.
    pub fn all_reachable_components(&self) -> std::collections::HashSet<String> {
        let mut set = std::collections::HashSet::new();
        for series in self.series.values() {
            if let Some(root) = &series.root_selector {
                walk_selector_for_components(root, &mut set);
            }
        }
        set
    }

    /// Check if a specific component_def_id or name is reachable from any
    /// loaded series in this store. The SpecComponent stores the component
    /// identifier which may be either a UUID or a name, so we check both.
    pub fn is_component_reachable(&self, component_def_id_or_name: &str) -> bool {
        let reachable = self.all_reachable_components();
        if reachable.contains(component_def_id_or_name) {
            return true;
        }
        // Also check by name: the reachable set may contain names while the
        // caller passes a UUID, or vice versa.
        // We can't resolve names to IDs here without the catalog, so we
        // rely on the caller passing whichever identifier matches.
        // As a fallback, check if any reachable string matches case-insensitively.
        reachable.iter().any(|r| r.eq_ignore_ascii_case(component_def_id_or_name))
    }

    /// Load a SeriesStore from a `.omnimseries` file (JSONC/JSON5 format).
    /// All loaded series are marked with `source: "external"`.
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("read {}: {}", path, e))?;
        let mut store: SeriesStore = json5::from_str(&content)
            .map_err(|e| format!("parse {}: {}", path, e))?;
        // Mark all externally-loaded series as "external" source
        for s in store.series.values_mut() {
            s.source = "external".to_string();
        }
        Ok(store)
    }

    /// Save the SeriesStore to a `.omnimseries` file (pretty JSON).
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("serialize: {}", e))?;
        std::fs::write(path, json)
            .map_err(|e| format!("write {}: {}", path, e))?;
        Ok(())
    }
}

/// Recursively walk a selector tree, collecting all component_def_ids
/// found at the leaf nodes (AnswerOutcome::Component).
fn walk_selector_for_components(
    sel: &Selector,
    set: &mut std::collections::HashSet<String>,
) {
    for ans in &sel.answers {
        match &ans.outcome {
            AnswerOutcome::NextSelector(next) => {
                walk_selector_for_components(next, set);
            }
            AnswerOutcome::Component(spec) => {
                set.insert(spec.component_def_id.clone());
            }
        }
    }
}

// ============================================================
// SelectionResult - output of the selection engine
// ============================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SelectionResult {
    pub spec_component: SpecComponent,
    pub selector_path: Vec<String>,
    pub component_def_id: String,
}

// ============================================================
// SelectionEngine - walks the decision tree
// ============================================================

pub struct SelectionEngine;

impl SelectionEngine {
    /// Walk the selector tree starting from the root selector.
    /// `answers` is a list of answer values to match at each selector level.
    /// Returns the SpecComponent found, or an error.
    pub fn select(series: &Series, answers: &[&str]) -> Result<SelectionResult, String> {
        let root = series.root_selector.as_ref()
            .ok_or_else(|| format!("series '{}' has no root selector", series.name))?;

        let mut selector_path = Vec::new();
        let mut current_selector = root;
        let mut answer_idx = 0;

        loop {
            selector_path.push(current_selector.id.clone());

            let answer_value = answers.get(answer_idx)
                .ok_or_else(|| format!(
                    "not enough answers for selector '{}' (question: '{}')",
                    current_selector.id, current_selector.question
                ))?;

            let matched_answer = current_selector.answers.iter()
                .find(|a| {
                    if a.is_numeric_range {
                        answer_value.parse::<f64>()
                            .map(|v| v >= a.range_min && v <= a.range_max)
                            .unwrap_or(false)
                    } else {
                        a.value.eq_ignore_ascii_case(answer_value)
                    }
                })
                .ok_or_else(|| format!(
                    "no answer matches '{}' for selector '{}' (question: '{}')",
                    answer_value, current_selector.id, current_selector.question
                ))?;

            match &matched_answer.outcome {
                AnswerOutcome::NextSelector(next) => {
                    current_selector = next;
                    answer_idx += 1;
                }
                AnswerOutcome::Component(spec_comp) => {
                    return Ok(SelectionResult {
                        spec_component: spec_comp.clone(),
                        selector_path,
                        component_def_id: spec_comp.component_def_id.clone(),
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_series() -> Series {
        let mut root = Selector::new("root", "element_type?");

        let mut beam_answers = Vec::new();
        let beam_spec = SpecComponent::new("Steel-Beam-IPE");
        beam_answers.push(Answer {
            id: "a1".into(),
            value: "beam".into(),
            is_numeric_range: false,
            range_min: 0.0,
            range_max: 0.0,
            outcome: AnswerOutcome::Component(beam_spec),
        });
        root.answers = beam_answers;

        Series {
            id: "test-series".into(),
            name: "Test".into(),
            description: String::new(),
            domain: String::new(),
            standard_reference: String::new(),
            root_selector: Some(root),
            applicable_phases: Vec::new(),
            metadata: HashMap::new(),
            source: "project".to_string(),
        }
    }

    #[test]
    fn test_select_simple() {
        let series = make_test_series();
        let result = SelectionEngine::select(&series, &["beam"]).unwrap();
        assert_eq!(result.component_def_id, "Steel-Beam-IPE");
        assert_eq!(result.selector_path.len(), 1);
    }

    #[test]
    fn test_select_no_match() {
        let series = make_test_series();
        let err = SelectionEngine::select(&series, &["wall"]);
        assert!(err.is_err());
    }

    #[test]
    fn test_series_store_basic() {
        let mut store = SeriesStore::new();
        let s = Series::new("CS150");
        let id = s.id.clone();
        store.insert(s);
        assert_eq!(store.len(), 1);
        assert!(store.get(&id).is_some());
    }

    #[test]
    fn test_series_store_roundtrip() {
        let store = SeriesStore::new_seeded();
        assert!(store.len() > 0);
        let tmp = std::env::temp_dir().join("omnim_test_series.omnimseries");
        store.save_to_file(tmp.to_str().unwrap()).unwrap();
        let loaded = SeriesStore::load_from_file(tmp.to_str().unwrap()).unwrap();
        assert_eq!(store.len(), loaded.len());
        // Verify a known series survives the round-trip
        assert!(loaded.find_by_name("CS150").is_some());
        assert!(loaded.find_by_name("Steel-Q355").is_some());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_series_store_load_jsonc_comments() {
        let tmp = std::env::temp_dir().join("omnim_test_series_jsonc.omnimseries");
        let content = r#"{
            // This is a comment
            "series": {}
        }"#;
        std::fs::write(&tmp, content).unwrap();
        let store = SeriesStore::load_from_file(tmp.to_str().unwrap());
        assert!(store.is_ok(), "JSONC with comments should parse");
        assert!(store.unwrap().is_empty());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_series_priority_project_over_seed() {
        let mut store = SeriesStore::new_seeded();
        // Seed series exist with source "seed"
        let cs150 = store.find_by_name("CS150").unwrap().clone();
        assert_eq!(cs150.source, "seed");

        // Insert a project series with the same ID
        let mut project_series = cs150.clone();
        project_series.source = "project".to_string();
        project_series.description = "project override".to_string();
        store.insert(project_series);

        // The project version should have won
        let result = store.get(&cs150.id).unwrap();
        assert_eq!(result.source, "project");
        assert_eq!(result.description, "project override");
    }

    #[test]
    fn test_series_priority_seed_does_not_override_project() {
        let mut store = SeriesStore::new();
        // Insert a project series
        let mut s = Series::new("TestSeries");
        s.source = "project".to_string();
        s.description = "project version".to_string();
        let id = s.id.clone();
        store.insert(s);

        // Try to insert a seed version with the same ID
        let mut seed_series = Series::new("Other");
        seed_series.id = id.clone();
        seed_series.source = "seed".to_string();
        seed_series.description = "seed version".to_string();
        store.insert(seed_series);

        // The project version should still be there
        let result = store.get(&id).unwrap();
        assert_eq!(result.source, "project");
        assert_eq!(result.description, "project version");
    }
}
