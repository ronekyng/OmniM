use std::collections::HashMap;

use omnim_core as c;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct RelationStore {
    connections: Vec<c::Connection>,
    containments: Vec<c::Containment>,
    conn_by_object: HashMap<String, Vec<usize>>,
    cont_by_parent: HashMap<String, Vec<usize>>,
    cont_by_child: HashMap<String, usize>,
}

impl Default for RelationStore {
    fn default() -> Self {
        Self::new()
    }
}

impl RelationStore {
    pub fn new() -> Self {
        Self {
            connections: Vec::new(),
            containments: Vec::new(),
            conn_by_object: HashMap::new(),
            cont_by_parent: HashMap::new(),
            cont_by_child: HashMap::new(),
        }
    }

    pub fn add_connection(&mut self, conn: c::Connection) -> usize {
        let idx = self.connections.len();
        self.conn_by_object
            .entry(conn.from_object_id.clone())
            .or_default()
            .push(idx);
        self.conn_by_object
            .entry(conn.to_object_id.clone())
            .or_default()
            .push(idx);
        self.connections.push(conn);
        idx
    }

    pub fn add_containment(&mut self, cont: c::Containment) -> Result<usize, String> {
        if self.cont_by_child.contains_key(&cont.child_id) {
            return Err(format!("object '{}' already has a parent", cont.child_id));
        }
        let idx = self.containments.len();
        self.cont_by_parent
            .entry(cont.parent_id.clone())
            .or_default()
            .push(idx);
        self.cont_by_child.insert(cont.child_id.clone(), idx);
        self.containments.push(cont);
        Ok(idx)
    }

    pub fn remove_containment(&mut self, parent_id: &str, child_id: &str) -> Option<c::Containment> {
        let idx = *self.cont_by_child.get(child_id)?;
        let cont = self.containments[idx].clone();
        if cont.parent_id != parent_id {
            return None;
        }
        self.cont_by_child.remove(child_id);
        if let Some(ids) = self.cont_by_parent.get_mut(parent_id) {
            ids.retain(|&i| i != idx);
        }
        Some(cont)
    }

    pub fn parent_of(&self, child_id: &str) -> Option<&c::Containment> {
        self.cont_by_child
            .get(child_id)
            .map(|&idx| &self.containments[idx])
    }

    pub fn children_of(&self, parent_id: &str) -> Vec<&c::Containment> {
        self.cont_by_parent
            .get(parent_id)
            .map(|ids| ids.iter().map(|&idx| &self.containments[idx]).collect())
            .unwrap_or_default()
    }

    pub fn connections_of(&self, object_id: &str) -> Vec<&c::Connection> {
        self.conn_by_object
            .get(object_id)
            .map(|indices| indices.iter().map(|&idx| &self.connections[idx]).collect())
            .unwrap_or_default()
    }

    pub fn connections(&self) -> &[c::Connection] {
        &self.connections
    }

    pub fn containments(&self) -> &[c::Containment] {
        &self.containments
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_containment() {
        let mut rs = RelationStore::new();
        rs.add_containment(c::Containment {
            parent_id: "site-1".into(),
            child_id: "bldg-1".into(),
            r#type: c::ContainmentType::Spatial as i32,
        }).unwrap();

        assert!(rs.parent_of("bldg-1").is_some());
        assert_eq!(rs.children_of("site-1").len(), 1);

        let err = rs.add_containment(c::Containment {
            parent_id: "site-2".into(),
            child_id: "bldg-1".into(),
            r#type: c::ContainmentType::Spatial as i32,
        });
        assert!(err.is_err());

        rs.remove_containment("site-1", "bldg-1");
        assert!(rs.parent_of("bldg-1").is_none());
    }

    #[test]
    fn test_connection() {
        let mut rs = RelationStore::new();
        rs.add_connection(c::Connection {
            from_object_id: "wall-1".into(),
            to_object_id: "wall-2".into(),
            r#type: c::ConnectionType::Structural as i32,
            from_point: None,
            to_point: None,
            connection_detail: String::new(),
        });

        assert_eq!(rs.connections_of("wall-1").len(), 1);
        assert_eq!(rs.connections_of("wall-2").len(), 1);
    }
}
