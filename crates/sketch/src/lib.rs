use omnim_core as c;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod solver;
pub mod profile;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SketchPointRef {
    pub entity_index: usize,
    pub point_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SketchEntityType {
    Point,
    Line,
    Circle,
    Arc,
    Rectangle,
    Polygon,
    Ellipse,
    Spline,
    Polyline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SketchEntity {
    pub id: String,
    pub entity_type: SketchEntityType,
    pub points: Vec<c::Point3D>,
    pub radius: f64,
    #[serde(default)]
    pub sides: i32,
    #[serde(default)]
    pub construction: bool,
}

impl SketchEntity {
    pub fn point(x: f64, y: f64, z: f64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            entity_type: SketchEntityType::Point,
            points: vec![c::Point3D { x, y, z }],
            radius: 0.0,
            sides: 0,
            construction: false,
        }
    }

    pub fn line(start: c::Point3D, end: c::Point3D) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            entity_type: SketchEntityType::Line,
            points: vec![start, end],
            radius: 0.0,
            sides: 0,
            construction: false,
        }
    }

    pub fn circle(center: c::Point3D, radius: f64) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            entity_type: SketchEntityType::Circle,
            points: vec![center],
            radius,
            sides: 0,
            construction: false,
        }
    }

    pub fn arc(center: c::Point3D, start: c::Point3D, end: c::Point3D) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            entity_type: SketchEntityType::Arc,
            points: vec![center, start, end],
            radius: 0.0,
            sides: 0,
            construction: false,
        }
    }

    pub fn point_count(&self) -> usize {
        match self.entity_type {
            SketchEntityType::Point => 1,
            SketchEntityType::Line => 2,
            SketchEntityType::Circle => 1,
            SketchEntityType::Arc => 3,
            SketchEntityType::Rectangle => 4,
            SketchEntityType::Polygon => 2,
            SketchEntityType::Ellipse => 3,
            SketchEntityType::Spline => self.points.len().max(2),
            SketchEntityType::Polyline => self.points.len().max(2),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SketchConstraintType {
    Horizontal,
    Vertical,
    Coincident,
    Parallel,
    Perpendicular,
    Length,
    Distance,
    Angle,
    Radius,
    Diameter,
    Tangent,
    Symmetric,
    Fixed,
    AtMidpoint,
    EqualLength,
    EqualRadius,
    PtOnLine,
    PtOnCircle,
    SameOrientation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SketchConstraint {
    pub id: String,
    pub constraint_type: SketchConstraintType,
    pub entity_refs: Vec<SketchPointRef>,
    pub value: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sketch {
    pub id: String,
    pub name: String,
    pub origin: c::Point3D,
    pub normal: c::Vector3D,
    pub entities: Vec<SketchEntity>,
    pub constraints: Vec<SketchConstraint>,
    pub solved: bool,
    pub solved_points: Vec<c::Point3D>,
    #[serde(default)]
    pub component_id: Option<String>,
}

impl Sketch {
    pub fn new(name: &str, origin: c::Point3D, normal: c::Vector3D) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            origin,
            normal,
            entities: Vec::new(),
            constraints: Vec::new(),
            solved: false,
            solved_points: Vec::new(),
            component_id: None,
        }
    }

    pub fn is_3d(&self) -> bool {
        self.normal.x == 0.0 && self.normal.y == 0.0 && self.normal.z == 0.0
    }

    pub fn add_entity(&mut self, entity: SketchEntity) -> usize {
        let idx = self.entities.len();
        self.entities.push(entity);
        self.solved = false;
        idx
    }

    pub fn add_constraint(&mut self, constraint: SketchConstraint) -> usize {
        let idx = self.constraints.len();
        self.constraints.push(constraint);
        self.solved = false;
        idx
    }

    pub fn get_point(&self, r#ref: &SketchPointRef) -> Option<&c::Point3D> {
        let entity = self.entities.get(r#ref.entity_index)?;
        entity.points.get(r#ref.point_index)
    }

    pub fn set_solved_point(&mut self, entity_index: usize, point_index: usize, point: c::Point3D) {
        let idx = entity_index * 3 + point_index;
        if idx >= self.solved_points.len() {
            self.solved_points.resize(idx + 1, c::Point3D { x: 0.0, y: 0.0, z: 0.0 });
        }
        self.solved_points[idx] = point;
    }

    pub fn get_solved_point(&self, entity_index: usize, point_index: usize) -> Option<&c::Point3D> {
        let idx = entity_index * 3 + point_index;
        self.solved_points.get(idx)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SketchStore {
    sketches: HashMap<String, Sketch>,
}

impl Default for SketchStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SketchStore {
    pub fn new() -> Self {
        Self {
            sketches: HashMap::new(),
        }
    }

    pub fn insert(&mut self, sketch: Sketch) -> bool {
        let id = sketch.id.clone();
        if self.sketches.contains_key(&id) {
            return false;
        }
        self.sketches.insert(id, sketch);
        true
    }

    pub fn get(&self, id: &str) -> Option<&Sketch> {
        self.sketches.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Sketch> {
        self.sketches.get_mut(id)
    }

    pub fn remove(&mut self, id: &str) -> Option<Sketch> {
        self.sketches.remove(id)
    }

    pub fn list(&self) -> Vec<&Sketch> {
        self.sketches.values().collect()
    }

    pub fn len(&self) -> usize {
        self.sketches.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sketches.is_empty()
    }

    pub fn find_by_name(&self, name: &str) -> Option<&Sketch> {
        self.sketches.values().find(|s| s.name == name)
    }

    pub fn find_by_name_mut(&mut self, name: &str) -> Option<&mut Sketch> {
        self.sketches.values_mut().find(|s| s.name == name)
    }

    pub fn find_by_component(&self, component_id: &str) -> Vec<&Sketch> {
        self.sketches.values().filter(|s| s.component_id.as_deref() == Some(component_id)).collect()
    }

    pub fn find_by_component_mut(&mut self, component_id: &str) -> Vec<&mut Sketch> {
        self.sketches.values_mut().filter(|s| s.component_id.as_deref() == Some(component_id)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn origin() -> c::Point3D {
        c::Point3D { x: 0.0, y: 0.0, z: 0.0 }
    }

    fn normal_2d() -> c::Vector3D {
        c::Vector3D { x: 0.0, y: 0.0, z: 1.0 }
    }

    #[test]
    fn test_sketch_new() {
        let s = Sketch::new("test", origin(), normal_2d());
        assert_eq!(s.name, "test");
        assert!(!s.is_3d());
        assert!(s.entities.is_empty());
        assert!(s.constraints.is_empty());
    }

    #[test]
    fn test_sketch_3d_mode() {
        let s = Sketch::new("s3d", origin(), c::Vector3D { x: 0.0, y: 0.0, z: 0.0 });
        assert!(s.is_3d());
    }

    #[test]
    fn test_add_entity() {
        let mut s = Sketch::new("s", origin(), normal_2d());
        let e = SketchEntity::line(
            c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
            c::Point3D { x: 10.0, y: 0.0, z: 0.0 },
        );
        let idx = s.add_entity(e);
        assert_eq!(idx, 0);
        assert_eq!(s.entities.len(), 1);
        assert_eq!(s.entities[0].entity_type, SketchEntityType::Line);
    }

    #[test]
    fn test_add_constraint() {
        let mut s = Sketch::new("s", origin(), normal_2d());
        let c = SketchConstraint {
            id: uuid::Uuid::new_v4().to_string(),
            constraint_type: SketchConstraintType::Horizontal,
            entity_refs: vec![SketchPointRef { entity_index: 0, point_index: 0 }],
            value: 0.0,
        };
        let idx = s.add_constraint(c);
        assert_eq!(idx, 0);
        assert_eq!(s.constraints.len(), 1);
    }

    #[test]
    fn test_store_crud() {
        let mut store = SketchStore::new();
        let s = Sketch::new("s1", origin(), normal_2d());
        let id = s.id.clone();
        assert!(store.insert(s));
        assert!(!store.is_empty());
        assert_eq!(store.len(), 1);
        assert!(store.get(&id).is_some());
        assert!(store.remove(&id).is_some());
        assert!(store.is_empty());
    }

    #[test]
    fn test_store_find_by_name() {
        let mut store = SketchStore::new();
        store.insert(Sketch::new("wall_sketch", origin(), normal_2d()));
        store.insert(Sketch::new("floor_sketch", origin(), normal_2d()));
        assert!(store.find_by_name("wall_sketch").is_some());
        assert!(store.find_by_name("nonexistent").is_none());
    }

    #[test]
    fn test_entity_point_counts() {
        assert_eq!(SketchEntity::point(0.0, 0.0, 0.0).point_count(), 1);
        assert_eq!(SketchEntity::line(origin(), origin()).point_count(), 2);
        assert_eq!(SketchEntity::circle(origin(), 1.0).point_count(), 1);
        assert_eq!(SketchEntity::arc(origin(), origin(), origin()).point_count(), 3);
    }

    #[test]
    fn test_get_point_ref() {
        let mut s = Sketch::new("s", origin(), normal_2d());
        let e = SketchEntity::line(
            c::Point3D { x: 1.0, y: 2.0, z: 0.0 },
            c::Point3D { x: 3.0, y: 4.0, z: 0.0 },
        );
        s.add_entity(e);
        let pt = s.get_point(&SketchPointRef { entity_index: 0, point_index: 0 }).unwrap();
        assert_eq!(pt.x, 1.0);
        assert_eq!(pt.y, 2.0);
        let pt2 = s.get_point(&SketchPointRef { entity_index: 0, point_index: 1 }).unwrap();
        assert_eq!(pt2.x, 3.0);
        assert!(s.get_point(&SketchPointRef { entity_index: 0, point_index: 2 }).is_none());
    }
}
