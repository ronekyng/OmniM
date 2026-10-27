use std::collections::HashMap;

use omnim_core as c;
use serde::{Deserialize, Serialize};

use crate::OmniObject;

#[derive(Serialize, Deserialize, Clone)]
pub struct ObjectStore {
    objects: HashMap<String, OmniObject>,
    #[serde(skip, default = "HashMap::new")]
    children: HashMap<String, Vec<String>>,
}

impl Default for ObjectStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectStore {
    pub fn new() -> Self {
        Self {
            objects: HashMap::new(),
            children: HashMap::new(),
        }
    }

    pub fn rebuild_children(&mut self) {
        self.children.clear();
        for (id, obj) in &self.objects {
            if let Some(pid) = &obj.parent_id {
                self.children.entry(pid.clone()).or_default().push(id.clone());
            }
        }
    }

    pub fn insert(&mut self, obj: OmniObject) -> &mut OmniObject {
        if let Some(pid) = &obj.parent_id {
            self.children.entry(pid.clone()).or_default().push(obj.id.clone());
        }
        let id = obj.id.clone();
        self.objects.insert(id.clone(), obj);
        self.objects.get_mut(&id).unwrap()
    }

    pub fn get(&self, id: &str) -> Option<&OmniObject> {
        self.objects.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut OmniObject> {
        self.objects.get_mut(id)
    }

    pub fn remove(&mut self, id: &str) -> Option<OmniObject> {
        let obj = self.objects.remove(id)?;
        if let Some(pid) = &obj.parent_id {
            if let Some(siblings) = self.children.get_mut(pid) {
                siblings.retain(|s| s != id);
            }
        }
        self.children.remove(id);
        Some(obj)
    }

    pub fn list(&self) -> Vec<&OmniObject> {
        self.objects.values().collect()
    }

    pub fn list_mut(&mut self) -> Vec<&mut OmniObject> {
        self.objects.values_mut().collect()
    }

    pub fn children_of(&self, parent_id: &str) -> Vec<&OmniObject> {
        self.children
            .get(parent_id)
            .map(|ids| ids.iter().filter_map(|id| self.objects.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.objects.contains_key(id)
    }

    pub fn len(&self) -> usize {
        self.objects.len()
    }

    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }

    pub fn find_by_type(&self, object_type: c::ObjectType) -> Vec<&OmniObject> {
        self.objects.values().filter(|o| o.object_type == object_type as i32).collect()
    }

    pub fn find_by_tag(&self, tag: &str) -> Vec<&OmniObject> {
        self.objects.values().filter(|o| o.has_tag(tag)).collect()
    }

    /// 查找同时拥有所有指定 tag 的对象（交集）
    pub fn find_by_tags(&self, tags: &[&str]) -> Vec<&OmniObject> {
        self.objects.values().filter(|o| tags.iter().all(|t| o.has_tag(t))).collect()
    }

    /// 查找拥有任意一个指定 tag 的对象（并集）
    pub fn find_by_any_tag(&self, tags: &[&str]) -> Vec<&OmniObject> {
        self.objects.values().filter(|o| tags.iter().any(|t| o.has_tag(t))).collect()
    }

    pub fn find_by_ifc_class(&self, ifc_class: &str) -> Vec<&OmniObject> {
        self.objects.values().filter(|o| o.ifc_class == ifc_class).collect()
    }

    pub fn roots(&self) -> Vec<&OmniObject> {
        self.objects.values().filter(|o| o.parent_id.is_none()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_get() {
        let mut store = ObjectStore::new();
        let wall = OmniObject::new(c::ObjectType::Wall, "Wall-1");
        let id = wall.id.clone();
        store.insert(wall);
        let found = store.get(&id).unwrap();
        assert_eq!(found.name, "Wall-1");
        assert_eq!(found.object_type, c::ObjectType::Wall as i32);
    }

    #[test]
    fn test_hierarchy() {
        let mut store = ObjectStore::new();
        let site = OmniObject::new(c::ObjectType::Site, "Site-A");
        let site_id = site.id.clone();
        store.insert(site);
        let mut building = OmniObject::new(c::ObjectType::Building, "Building-1");
        building.parent_id = Some(site_id.clone());
        let building_id = building.id.clone();
        store.insert(building);
        let kids = store.children_of(&site_id);
        assert_eq!(kids.len(), 1);
        assert_eq!(kids[0].id, building_id);
    }

    #[test]
    fn test_find_by_type() {
        let mut store = ObjectStore::new();
        store.insert(OmniObject::new(c::ObjectType::Wall, "Wall-1"));
        store.insert(OmniObject::new(c::ObjectType::Wall, "Wall-2"));
        store.insert(OmniObject::new(c::ObjectType::Beam, "Beam-1"));

        assert_eq!(store.find_by_type(c::ObjectType::Wall).len(), 2);
        assert_eq!(store.find_by_type(c::ObjectType::Beam).len(), 1);
        assert_eq!(store.find_by_type(c::ObjectType::Road).len(), 0);
    }

    #[test]
    fn test_find_by_tag() {
        let mut store = ObjectStore::new();
        store.insert(OmniObject::new(c::ObjectType::Wall, "Wall-1").with_tags(["structural", "exterior"]));
        store.insert(OmniObject::new(c::ObjectType::Wall, "Wall-2").with_tags(["interior"]));

        assert_eq!(store.find_by_tag("structural").len(), 1);
        assert_eq!(store.find_by_tag("exterior").len(), 1);
        assert_eq!(store.find_by_tag("interior").len(), 1);
        assert_eq!(store.find_by_tag("nonexistent").len(), 0);
    }

    #[test]
    fn test_find_by_tags_intersection() {
        let mut store = ObjectStore::new();
        store.insert(OmniObject::new(c::ObjectType::Wall, "A").with_tags(["tag1", "tag2"]));
        store.insert(OmniObject::new(c::ObjectType::Wall, "B").with_tags(["tag1"]));
        store.insert(OmniObject::new(c::ObjectType::Wall, "C").with_tags(["tag2"]));

        assert_eq!(store.find_by_tags(&["tag1"]).len(), 2);
        assert_eq!(store.find_by_tags(&["tag1", "tag2"]).len(), 1);
        assert_eq!(store.find_by_tags(&["tag1", "tag2", "tag3"]).len(), 0);
    }

    #[test]
    fn test_find_by_any_tag() {
        let mut store = ObjectStore::new();
        store.insert(OmniObject::new(c::ObjectType::Wall, "A").with_tags(["tag1"]));
        store.insert(OmniObject::new(c::ObjectType::Wall, "B").with_tags(["tag2"]));
        store.insert(OmniObject::new(c::ObjectType::Wall, "C").with_tags(["tag3"]));

        assert_eq!(store.find_by_any_tag(&["tag1", "tag2"]).len(), 2);
        assert_eq!(store.find_by_any_tag(&["tag1", "nonexistent"]).len(), 1);
    }

    #[test]
    fn test_roots() {
        let mut store = ObjectStore::new();
        let site = OmniObject::new(c::ObjectType::Site, "Site");
        let site_id = site.id.clone();
        store.insert(site);

        let mut building = OmniObject::new(c::ObjectType::Building, "Building");
        building.parent_id = Some(site_id.clone());
        store.insert(building);

        let roots = store.roots();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].id, site_id);
    }

    #[test]
    fn test_remove_with_children() {
        let mut store = ObjectStore::new();
        let parent = OmniObject::new(c::ObjectType::Site, "Parent");
        let pid = parent.id.clone();
        store.insert(parent);

        let mut child = OmniObject::new(c::ObjectType::Building, "Child");
        child.parent_id = Some(pid.clone());
        store.insert(child);

        // Remove parent
        store.remove(&pid);
        assert!(!store.contains(&pid));
        // children index for parent is cleared
        assert_eq!(store.children_of(&pid).len(), 0);
    }
}
