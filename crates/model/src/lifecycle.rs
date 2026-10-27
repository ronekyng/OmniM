use std::collections::HashMap;

use omnim_core as c;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct LifecycleStore {
    records: HashMap<String, c::ProjectLifecycleRecord>,
    by_phase: HashMap<i32, Vec<String>>,
    by_object: HashMap<String, Vec<String>>,
}

impl Default for LifecycleStore {
    fn default() -> Self {
        Self::new()
    }
}

impl LifecycleStore {
    pub fn new() -> Self {
        Self {
            records: HashMap::new(),
            by_phase: HashMap::new(),
            by_object: HashMap::new(),
        }
    }

    pub fn insert(&mut self, record: c::ProjectLifecycleRecord) -> &mut c::ProjectLifecycleRecord {
        let id = record.id.clone();
        let phase = record.phase;
        for obj_id in &record.related_object_ids {
            self.by_object
                .entry(obj_id.clone())
                .or_default()
                .push(id.clone());
        }
        self.by_phase.entry(phase).or_default().push(id.clone());
        self.records.insert(id.clone(), record);
        self.records.get_mut(&id).unwrap()
    }

    pub fn get(&self, id: &str) -> Option<&c::ProjectLifecycleRecord> {
        self.records.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut c::ProjectLifecycleRecord> {
        self.records.get_mut(id)
    }

    pub fn remove(&mut self, id: &str) -> Option<c::ProjectLifecycleRecord> {
        let record = self.records.remove(id)?;
        if let Some(ids) = self.by_phase.get_mut(&record.phase) {
            ids.retain(|i| i != id);
        }
        for obj_id in &record.related_object_ids {
            if let Some(ids) = self.by_object.get_mut(obj_id) {
                ids.retain(|i| i != id);
            }
        }
        Some(record)
    }

    pub fn list(&self) -> Vec<&c::ProjectLifecycleRecord> {
        self.records.values().collect()
    }

    pub fn find_by_phase(&self, phase: c::LifecyclePhase) -> Vec<&c::ProjectLifecycleRecord> {
        self.by_phase
            .get(&(phase as i32))
            .map(|ids| ids.iter().filter_map(|id| self.records.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn find_by_object(&self, object_id: &str) -> Vec<&c::ProjectLifecycleRecord> {
        self.by_object
            .get(object_id)
            .map(|ids| ids.iter().filter_map(|id| self.records.get(id)).collect())
            .unwrap_or_default()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_record(phase: c::LifecyclePhase, name: &str, obj_ids: Vec<&str>) -> c::ProjectLifecycleRecord {
        c::ProjectLifecycleRecord {
            id: name.to_string(),
            phase: phase as i32,
            record_type: "test".into(),
            name: name.into(),
            description: String::new(),
            related_object_ids: obj_ids.into_iter().map(String::from).collect(),
            related_document_ids: Vec::new(),
            properties: Vec::new(),
            status: c::RecordStatus::Draft as i32,
            planned_start: None,
            planned_finish: None,
            actual_start: None,
            actual_finish: None,
            planned_cost: 0.0,
            actual_cost: 0.0,
            responsible_party: String::new(),
            version_id: String::new(),
        }
    }

    #[test]
    fn test_lifecycle_store() {
        let mut store = LifecycleStore::new();
        store.insert(make_record(c::LifecyclePhase::Planning, "r1", vec!["obj-1"]));
        store.insert(make_record(c::LifecyclePhase::Design, "r2", vec!["obj-1", "obj-2"]));

        assert_eq!(store.len(), 2);
        let planning = store.find_by_phase(c::LifecyclePhase::Planning);
        assert_eq!(planning.len(), 1);
        let obj1_records = store.find_by_object("obj-1");
        assert_eq!(obj1_records.len(), 2);
    }

    #[test]
    fn test_remove_record() {
        let mut store = LifecycleStore::new();
        store.insert(make_record(c::LifecyclePhase::Construction, "cr1", vec!["site-1"]));
        store.insert(make_record(c::LifecyclePhase::Construction, "cr2", vec!["site-1"]));

        assert_eq!(store.len(), 2);
        store.remove("cr1");
        assert_eq!(store.len(), 1);
        assert!(store.get("cr1").is_none());
        assert!(store.get("cr2").is_some());
    }

    #[test]
    fn test_all_phases() {
        let mut store = LifecycleStore::new();
        let phases = [
            c::LifecyclePhase::Planning,
            c::LifecyclePhase::Design,
            c::LifecyclePhase::Procurement,
            c::LifecyclePhase::Construction,
            c::LifecyclePhase::CommissioningAcceptance,
            c::LifecyclePhase::Asset,
            c::LifecyclePhase::OperationMaintenance,
            c::LifecyclePhase::Production,
            c::LifecyclePhase::Decommissioning,
        ];
        for (i, phase) in phases.iter().enumerate() {
            store.insert(make_record(*phase, &format!("r{}", i), vec![]));
        }
        assert_eq!(store.len(), 9);
        for phase in &phases {
            assert_eq!(store.find_by_phase(*phase).len(), 1);
        }
    }

    #[test]
    fn test_remove_updates_indexes() {
        let mut store = LifecycleStore::new();
        store.insert(make_record(c::LifecyclePhase::Design, "d1", vec!["obj-a"]));
        store.insert(make_record(c::LifecyclePhase::Design, "d2", vec!["obj-a", "obj-b"]));

        store.remove("d1");
        assert_eq!(store.find_by_phase(c::LifecyclePhase::Design).len(), 1);
        assert_eq!(store.find_by_object("obj-a").len(), 1);
    }
}
