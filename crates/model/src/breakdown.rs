use std::collections::HashMap;

use omnim_core as c;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct BreakdownNode {
    pub id: String,
    pub level: i32,
    pub name: String,
    pub code: String,
    pub parent_id: Option<String>,
    pub convention: i32,
    pub related_object_ids: Vec<String>,
    pub administrative_division_ids: Vec<String>,
    pub properties: Vec<c::Property>,
}

impl BreakdownNode {
    pub fn new(level: c::BreakdownLevel, name: impl Into<String>, convention: c::BreakdownConvention) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            level: level as i32,
            name: name.into(),
            code: String::new(),
            parent_id: None,
            convention: convention as i32,
            related_object_ids: Vec::new(),
            administrative_division_ids: Vec::new(),
            properties: Vec::new(),
        }
    }

    pub fn with_parent(mut self, parent_id: impl Into<String>) -> Self {
        self.parent_id = Some(parent_id.into());
        self
    }

    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = code.into();
        self
    }

    pub fn link_object(&mut self, object_id: impl Into<String>) {
        let id = object_id.into();
        if !self.related_object_ids.contains(&id) {
            self.related_object_ids.push(id);
        }
    }

    pub fn link_administrative_division(&mut self, division_id: impl Into<String>) {
        let id = division_id.into();
        if !self.administrative_division_ids.contains(&id) {
            self.administrative_division_ids.push(id);
        }
    }

    pub fn level_name(&self) -> &'static str {
        c::BreakdownLevel::try_from(self.level)
            .map(level_name)
            .unwrap_or("UNKNOWN")
    }

    pub fn convention_name(&self) -> &'static str {
        c::BreakdownConvention::try_from(self.convention)
            .map(convention_name)
            .unwrap_or("UNKNOWN")
    }
}

fn level_name(level: c::BreakdownLevel) -> &'static str {
    use c::BreakdownLevel::*;
    match level {
        Unspecified => "Unspecified",
        Program => "Program",
        Project => "Project",
        SubProject => "SubProject",
        Unit => "Unit",
        Building => "Building",
        Storey => "Storey",
        Zone => "Zone",
        System => "System",
        Element => "Element",
        Part => "Part",
    }
}

fn convention_name(conv: c::BreakdownConvention) -> &'static str {
    use c::BreakdownConvention::*;
    match conv {
        Unspecified => "Unspecified",
        Building => "Building",
        Plant => "Plant",
        Civil => "Civil",
        Mechanical => "Mechanical",
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BreakdownStore {
    nodes: HashMap<String, BreakdownNode>,
    by_level: HashMap<i32, Vec<String>>,
    by_parent: HashMap<String, Vec<String>>,
    by_convention: HashMap<i32, Vec<String>>,
}

impl Default for BreakdownStore {
    fn default() -> Self {
        Self::new()
    }
}

impl BreakdownStore {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            by_level: HashMap::new(),
            by_parent: HashMap::new(),
            by_convention: HashMap::new(),
        }
    }

    pub fn insert(&mut self, node: BreakdownNode) -> String {
        let id = node.id.clone();
        let level = node.level;
        let convention = node.convention;
        if let Some(ref pid) = node.parent_id {
            self.by_parent
                .entry(pid.clone())
                .or_default()
                .push(id.clone());
        }
        self.by_level.entry(level).or_default().push(id.clone());
        self.by_convention.entry(convention).or_default().push(id.clone());
        self.nodes.insert(id.clone(), node);
        id
    }

    pub fn get(&self, id: &str) -> Option<&BreakdownNode> {
        self.nodes.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut BreakdownNode> {
        self.nodes.get_mut(id)
    }

    pub fn remove(&mut self, id: &str) -> Option<BreakdownNode> {
        let node = self.nodes.remove(id)?;
        if let Some(ref pid) = node.parent_id {
            if let Some(ids) = self.by_parent.get_mut(pid) {
                ids.retain(|i| i != id);
            }
        }
        if let Some(ids) = self.by_level.get_mut(&node.level) {
            ids.retain(|i| i != id);
        }
        if let Some(ids) = self.by_convention.get_mut(&node.convention) {
            ids.retain(|i| i != id);
        }
        Some(node)
    }

    pub fn list(&self) -> Vec<&BreakdownNode> {
        self.nodes.values().collect()
    }

    pub fn find_by_level(&self, level: c::BreakdownLevel) -> Vec<&BreakdownNode> {
        self.by_level
            .get(&(level as i32))
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn find_by_convention(&self, convention: c::BreakdownConvention) -> Vec<&BreakdownNode> {
        self.by_convention
            .get(&(convention as i32))
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn children_of(&self, parent_id: &str) -> Vec<&BreakdownNode> {
        self.by_parent
            .get(parent_id)
            .map(|ids| ids.iter().filter_map(|id| self.nodes.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn ancestors_of(&self, id: &str) -> Vec<&BreakdownNode> {
        let mut result = Vec::new();
        let mut current = id;
        while let Some(node) = self.nodes.get(current) {
            if let Some(ref pid) = node.parent_id {
                if let Some(parent) = self.nodes.get(pid) {
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

    pub fn descendants_of(&self, id: &str) -> Vec<&BreakdownNode> {
        let mut result = Vec::new();
        let mut stack = vec![id];
        while let Some(current) = stack.pop() {
            if let Some(children) = self.by_parent.get(current) {
                for child_id in children {
                    if let Some(child) = self.nodes.get(child_id) {
                        result.push(child);
                        stack.push(child_id);
                    }
                }
            }
        }
        result
    }

    pub fn roots(&self) -> Vec<&BreakdownNode> {
        self.nodes
            .values()
            .filter(|n| n.parent_id.is_none())
            .collect()
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut store = BreakdownStore::new();
        let node = BreakdownNode::new(c::BreakdownLevel::Project, "MyProject", c::BreakdownConvention::Building);
        let id = store.insert(node);
        assert!(store.get(&id).is_some());
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn test_parent_child() {
        let mut store = BreakdownStore::new();
        let proj = BreakdownNode::new(c::BreakdownLevel::Project, "P1", c::BreakdownConvention::Building);
        let proj_id = proj.id.clone();
        store.insert(proj);

        let bldg = BreakdownNode::new(c::BreakdownLevel::Building, "B1", c::BreakdownConvention::Building)
            .with_parent(proj_id.clone());
        let bldg_id = bldg.id.clone();
        store.insert(bldg);

        let children = store.children_of(&proj_id);
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, bldg_id);
    }

    #[test]
    fn test_ancestors() {
        let mut store = BreakdownStore::new();
        let prog = BreakdownNode::new(c::BreakdownLevel::Program, "Program1", c::BreakdownConvention::Plant);
        let prog_id = prog.id.clone();
        store.insert(prog);

        let proj = BreakdownNode::new(c::BreakdownLevel::Project, "Project1", c::BreakdownConvention::Plant)
            .with_parent(prog_id.clone());
        let proj_id = proj.id.clone();
        store.insert(proj);

        let unit = BreakdownNode::new(c::BreakdownLevel::Unit, "Unit1", c::BreakdownConvention::Plant)
            .with_parent(proj_id.clone());
        let unit_id = unit.id.clone();
        store.insert(unit);

        let ancestors = store.ancestors_of(&unit_id);
        assert_eq!(ancestors.len(), 2);
        assert_eq!(ancestors[0].name, "Project1");
        assert_eq!(ancestors[1].name, "Program1");
    }

    #[test]
    fn test_descendants() {
        let mut store = BreakdownStore::new();
        let proj = BreakdownNode::new(c::BreakdownLevel::Project, "P1", c::BreakdownConvention::Civil);
        let proj_id = proj.id.clone();
        store.insert(proj);

        let b1 = BreakdownNode::new(c::BreakdownLevel::Building, "B1", c::BreakdownConvention::Civil)
            .with_parent(proj_id.clone());
        store.insert(b1);
        let b2 = BreakdownNode::new(c::BreakdownLevel::Building, "B2", c::BreakdownConvention::Civil)
            .with_parent(proj_id.clone());
        store.insert(b2);

        let descendants = store.descendants_of(&proj_id);
        assert_eq!(descendants.len(), 2);
    }

    #[test]
    fn test_roots() {
        let mut store = BreakdownStore::new();
        store.insert(BreakdownNode::new(c::BreakdownLevel::Program, "P1", c::BreakdownConvention::Building));
        store.insert(BreakdownNode::new(c::BreakdownLevel::Program, "P2", c::BreakdownConvention::Building));

        let roots = store.roots();
        assert_eq!(roots.len(), 2);
    }

    #[test]
    fn test_link_objects() {
        let mut store = BreakdownStore::new();
        let id = store.insert(BreakdownNode::new(c::BreakdownLevel::Building, "B1", c::BreakdownConvention::Building));
        let node = store.get_mut(&id).unwrap();
        node.link_object("obj-1");
        node.link_object("obj-2");
        node.link_object("obj-1");

        let node = store.get(&id).unwrap();
        assert_eq!(node.related_object_ids.len(), 2);
    }

    #[test]
    fn test_convention_filter() {
        let mut store = BreakdownStore::new();
        store.insert(BreakdownNode::new(c::BreakdownLevel::Building, "B1", c::BreakdownConvention::Building));
        store.insert(BreakdownNode::new(c::BreakdownLevel::Building, "B2", c::BreakdownConvention::Plant));

        let building_nodes = store.find_by_convention(c::BreakdownConvention::Building);
        assert_eq!(building_nodes.len(), 1);
        let plant_nodes = store.find_by_convention(c::BreakdownConvention::Plant);
        assert_eq!(plant_nodes.len(), 1);
    }

    #[test]
    fn test_remove() {
        let mut store = BreakdownStore::new();
        let id = store.insert(BreakdownNode::new(c::BreakdownLevel::Project, "P1", c::BreakdownConvention::Building));
        store.remove(&id);
        assert!(store.get(&id).is_none());
        assert!(store.is_empty());
    }
}
