use std::collections::HashMap;

use omnim_core as c;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GeoBoundary {
    pub boundary_type: String,
    pub coordinates: Vec<c::Point3D>,
    pub crs: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AdministrativeDivision {
    pub id: String,
    pub level: i32,
    pub name: String,
    pub native_name: String,
    pub code: String,
    pub parent_id: Option<String>,
    pub boundary: Option<GeoBoundary>,
    pub properties: Vec<c::Property>,
}

impl AdministrativeDivision {
    pub fn new(level: c::AdministrativeLevel, name: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            level: level as i32,
            name: name.into(),
            native_name: String::new(),
            code: code.into(),
            parent_id: None,
            boundary: None,
            properties: Vec::new(),
        }
    }

    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    pub fn with_native_name(mut self, name: impl Into<String>) -> Self {
        self.native_name = name.into();
        self
    }

    pub fn with_boundary(mut self, boundary: GeoBoundary) -> Self {
        self.boundary = Some(boundary);
        self
    }

    pub fn level_name(&self) -> &'static str {
        c::AdministrativeLevel::try_from(self.level)
            .map(level_name)
            .unwrap_or("UNKNOWN")
    }
}

fn level_name(level: c::AdministrativeLevel) -> &'static str {
    use c::AdministrativeLevel::*;
    match level {
        Unspecified => "Unspecified",
        World => "World",
        Country => "Country",
        Province => "Province",
        Prefecture => "Prefecture",
        County => "County",
        Township => "Township",
        Community => "Community",
        Block => "Block",
        Parcel => "Parcel",
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct AdministrativeStore {
    divisions: HashMap<String, AdministrativeDivision>,
    by_level: HashMap<i32, Vec<String>>,
    by_parent: HashMap<String, Vec<String>>,
    by_code: HashMap<String, String>,
}

impl Default for AdministrativeStore {
    fn default() -> Self {
        Self::new()
    }
}

impl AdministrativeStore {
    pub fn new() -> Self {
        Self {
            divisions: HashMap::new(),
            by_level: HashMap::new(),
            by_parent: HashMap::new(),
            by_code: HashMap::new(),
        }
    }

    pub fn insert(&mut self, division: AdministrativeDivision) -> String {
        let id = division.id.clone();
        let level = division.level;
        if let Some(ref pid) = division.parent_id {
            self.by_parent
                .entry(pid.clone())
                .or_default()
                .push(id.clone());
        }
        self.by_level.entry(level).or_default().push(id.clone());
        if !division.code.is_empty() {
            self.by_code.insert(division.code.clone(), id.clone());
        }
        self.divisions.insert(id.clone(), division);
        id
    }

    pub fn get(&self, id: &str) -> Option<&AdministrativeDivision> {
        self.divisions.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut AdministrativeDivision> {
        self.divisions.get_mut(id)
    }

    pub fn remove(&mut self, id: &str) -> Option<AdministrativeDivision> {
        let div = self.divisions.remove(id)?;
        if let Some(ref pid) = div.parent_id {
            if let Some(ids) = self.by_parent.get_mut(pid) {
                ids.retain(|i| i != id);
            }
        }
        if let Some(ids) = self.by_level.get_mut(&div.level) {
            ids.retain(|i| i != id);
        }
        if !div.code.is_empty() {
            self.by_code.remove(&div.code);
        }
        Some(div)
    }

    pub fn list(&self) -> Vec<&AdministrativeDivision> {
        self.divisions.values().collect()
    }

    pub fn find_by_level(&self, level: c::AdministrativeLevel) -> Vec<&AdministrativeDivision> {
        self.by_level
            .get(&(level as i32))
            .map(|ids| ids.iter().filter_map(|id| self.divisions.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn children_of(&self, parent_id: &str) -> Vec<&AdministrativeDivision> {
        self.by_parent
            .get(parent_id)
            .map(|ids| ids.iter().filter_map(|id| self.divisions.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn find_by_code(&self, code: &str) -> Option<&AdministrativeDivision> {
        self.by_code.get(code).and_then(|id| self.divisions.get(id))
    }

    pub fn ancestors_of(&self, id: &str) -> Vec<&AdministrativeDivision> {
        let mut result = Vec::new();
        let mut current = id;
        while let Some(div) = self.divisions.get(current) {
            if let Some(ref pid) = div.parent_id {
                if let Some(parent) = self.divisions.get(pid) {
                    result.push(parent);
                    current = pid;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        result
    }

    pub fn len(&self) -> usize {
        self.divisions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.divisions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_div(level: c::AdministrativeLevel, name: &str, code: &str) -> AdministrativeDivision {
        AdministrativeDivision::new(level, name, code)
    }

    #[test]
    fn test_insert_and_get() {
        let mut store = AdministrativeStore::new();
        let china = make_div(c::AdministrativeLevel::Country, "China", "156");
        let id = store.insert(china);
        assert!(store.get(&id).is_some());
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_parent_child() {
        let mut store = AdministrativeStore::new();
        let country = make_div(c::AdministrativeLevel::Country, "China", "156");
        let country_id = country.id.clone();
        store.insert(country);

        let mut province = make_div(c::AdministrativeLevel::Province, "Guangdong", "440000");
        province.parent_id = Some(country_id.clone());
        let prov_id = province.id.clone();
        store.insert(province);

        let children = store.children_of(&country_id);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, prov_id);
    }

    #[test]
    fn test_find_by_code() {
        let mut store = AdministrativeStore::new();
        store.insert(make_div(c::AdministrativeLevel::Country, "China", "156"));
        store.insert(make_div(c::AdministrativeLevel::Country, "Japan", "392"));

        let found = store.find_by_code("156");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "China");
    }

    #[test]
    fn test_ancestors() {
        let mut store = AdministrativeStore::new();

        let world = make_div(c::AdministrativeLevel::World, "Earth", "");
        let world_id = world.id.clone();
        store.insert(world);

        let mut country = make_div(c::AdministrativeLevel::Country, "China", "156");
        country.parent_id = Some(world_id.clone());
        let country_id = country.id.clone();
        store.insert(country);

        let mut province = make_div(c::AdministrativeLevel::Province, "Guangdong", "440000");
        province.parent_id = Some(country_id.clone());
        let prov_id = province.id.clone();
        store.insert(province);

        let ancestors = store.ancestors_of(&prov_id);
        assert_eq!(ancestors.len(), 2);
        assert_eq!(ancestors[0].name, "China");
        assert_eq!(ancestors[1].name, "Earth");
    }

    #[test]
    fn test_remove() {
        let mut store = AdministrativeStore::new();
        let id = store.insert(make_div(c::AdministrativeLevel::Country, "China", "156"));
        store.remove(&id);
        assert!(store.get(&id).is_none());
        assert!(store.is_empty());
    }
}
