use std::collections::HashMap;

use omnim_core as c;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct MaterialStore {
    materials: HashMap<String, c::Material>,
}

impl Default for MaterialStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MaterialStore {
    pub fn new() -> Self {
        Self {
            materials: HashMap::new(),
        }
    }

    pub fn insert(&mut self, material: c::Material) -> &mut c::Material {
        let id = material.id.clone();
        self.materials.insert(id.clone(), material);
        self.materials.get_mut(&id).unwrap()
    }

    pub fn get(&self, id: &str) -> Option<&c::Material> {
        self.materials.get(id)
    }

    pub fn remove(&mut self, id: &str) -> Option<c::Material> {
        self.materials.remove(id)
    }

    pub fn list(&self) -> Vec<&c::Material> {
        self.materials.values().collect()
    }

    pub fn len(&self) -> usize {
        self.materials.len()
    }

    pub fn is_empty(&self) -> bool {
        self.materials.is_empty()
    }

    pub fn find_by_category(&self, category: c::MaterialCategory) -> Vec<&c::Material> {
        self.materials
            .values()
            .filter(|m| m.category == category as i32)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_material_store() {
        let mut store = MaterialStore::new();
        let mat = c::Material {
            id: "mat-1".into(),
            name: "C30 Concrete".into(),
            category: c::MaterialCategory::Concrete as i32,
            density: 2400.0,
            ..Default::default()
        };
        store.insert(mat);
        assert_eq!(store.len(), 1);
        assert!(store.get("mat-1").is_some());
        let concretes = store.find_by_category(c::MaterialCategory::Concrete);
        assert_eq!(concretes.len(), 1);
    }
}
