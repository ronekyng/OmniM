use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// ComponentStore holds reusable Component templates in the Library.
///
/// These are full 3D geometric bodies (with feature tree and sketches)
/// promoted from projects or defined directly. They can be referenced
/// by Series spec decision trees and cloned into projects for instantiation.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ComponentStore {
    components: HashMap<String, crate::OmniObject>,
}

impl ComponentStore {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: String, component: crate::OmniObject) {
        self.components.insert(id, component);
    }

    pub fn get(&self, id: &str) -> Option<&crate::OmniObject> {
        self.components.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut crate::OmniObject> {
        self.components.get_mut(id)
    }

    pub fn remove(&mut self, id: &str) -> Option<crate::OmniObject> {
        self.components.remove(id)
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }

    pub fn list(&self) -> Vec<&crate::OmniObject> {
        self.components.values().collect()
    }
}