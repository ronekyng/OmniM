use omnim_core as c;
use serde::{Deserialize, Serialize};

use crate::CatalogStore;
use crate::feature_template::FeatureTreeRegistry;
use crate::ComponentStore;

/// The shared Library holds the master Component Catalog, Component templates, and feature tree templates.
///
/// Per the architecture update, the Library is a **session-level global static** (`static LIBRARY: RwLock<Option<Library>>`),
/// loaded once per session and shared across all projects. The Project does **not** hold a Library reference.
///
/// The SeriesStore is an **independent module** — its spec decision trees reference Library content
/// by ComponentDef ID. Only components reachable through the project's loaded series may be instantiated.
#[derive(Serialize, Deserialize, Clone)]
pub struct Library {
    pub catalog: CatalogStore,
    pub components: ComponentStore,
    pub feature_trees: FeatureTreeRegistry,
}

impl Default for Library {
    fn default() -> Self {
        Self::new()
    }
}

impl Library {
    /// Create an empty Library.
    pub fn new() -> Self {
        Self {
            catalog: CatalogStore::new(),
            components: ComponentStore::new(),
            feature_trees: FeatureTreeRegistry::new(),
        }
    }

    /// Create and seed a Library with the built-in catalog components,
    /// feature tree templates, and template links.
    pub fn seeded() -> Self {
        let mut lib = Self::new();
        let links = crate::seed_catalog::seed_feature_trees(&mut lib.feature_trees);
        crate::seed_catalog::seed_catalog(&mut lib.catalog);
        crate::seed_catalog::link_templates(&mut lib.catalog, &links);
        lib
    }

    /// Returns the default catalog component name for a given ObjectType.
    ///
    /// Returns `None` if no default template is registered for this type
    /// (e.g. generic containers like Site, Building, Space).
    pub fn default_catalog_component(object_type: c::ObjectType) -> Option<&'static str> {
        crate::gen_rule::default_catalog_component(object_type)
    }

    /// Returns the default series name for a given ObjectType.
    ///
    /// When `create pipe --series CS150` is used, the series drives the
    /// component selection. This provides the fallback series per domain.
    pub fn default_series_for_type(object_type: c::ObjectType) -> Option<&'static str> {
        crate::gen_rule::default_series_for_type(object_type)
    }

    /// Find a ComponentDef by name in the library catalog.
    pub fn find_component_by_name(&self, name: &str) -> Option<&crate::catalog::ComponentDef> {
        self.catalog.find_by_name(name)
    }

    /// Get a ComponentDef by ID from the library catalog.
    pub fn get_component(&self, id: &str) -> Option<&crate::catalog::ComponentDef> {
        self.catalog.get(id)
    }

    /// Get a feature tree template by ID from the library registry.
    pub fn get_feature_tree(&self, id: &str) -> Option<&crate::feature_template::FeatureTreeTemplate> {
        self.feature_trees.get(id)
    }

    /// Load a Library from a `.omnimlib` file (JSONC/JSON5 format).
    pub fn load_from_file(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("read {}: {}", path, e))?;
        json5::from_str(&content)
            .map_err(|e| format!("parse {}: {}", path, e))
    }

    /// Save the Library to a `.omnimlib` file (pretty JSON).
    pub fn save_to_file(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("serialize: {}", e))?;
        std::fs::write(path, json)
            .map_err(|e| format!("write {}: {}", path, e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_roundtrip() {
        let lib = Library::seeded();
        let tmp = std::env::temp_dir().join("omnim_test_library.omnimlib");
        lib.save_to_file(tmp.to_str().unwrap()).unwrap();
        let loaded = Library::load_from_file(tmp.to_str().unwrap()).unwrap();
        assert_eq!(lib.catalog.len(), loaded.catalog.len());
        // Verify a known component survives the round-trip
        assert!(loaded.catalog.find_by_name("Concrete-Wall").is_some());
        assert!(loaded.catalog.find_by_name("Pipe-CS").is_some());
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn test_library_load_jsonc_comments() {
        // JSONC with comments should parse correctly
        let tmp = std::env::temp_dir().join("omnim_test_library_jsonc.omnimlib");
        let content = r#"{
            // This is a comment
            "catalog": {
                "components": {},
                "by_category": {}
            },
            "components": { "components": {} },
            "feature_trees": { "templates": {} }
        }"#;
        std::fs::write(&tmp, content).unwrap();
        let lib = Library::load_from_file(tmp.to_str().unwrap());
        assert!(lib.is_ok(), "JSONC with comments should parse");
        let _ = std::fs::remove_file(&tmp);
    }
}
