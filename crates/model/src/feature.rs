use omnim_core as c;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Feature {
    pub id: String,
    pub name: String,
    pub feature_type: FeatureType,
    pub sketch_id: String,
    #[serde(default)]
    pub input_sketch_ids: Vec<String>,
    #[serde(default)]
    pub dependency_ids: Vec<String>,
    #[serde(default)]
    pub axis_sketch_id: String,
    pub depth: f64,
    pub direction: c::Vector3D,
    pub angle: f64,
    pub axis_origin: c::Point3D,
    pub axis_direction: c::Vector3D,
    pub is_cut: bool,
    pub order: i32,
    #[serde(default)]
    pub suppressed: bool,
    #[serde(default)]
    pub cached_geometry: Option<c::MeshGeometry>,
    #[serde(default = "default_dirty")]
    pub dirty: bool,
    #[serde(default = "default_lod")]
    pub lod: f64,
    #[serde(default)]
    pub boolean_op: Option<BooleanOp>,
    #[serde(default)]
    pub pattern_count_x: Option<usize>,
    #[serde(default)]
    pub pattern_count_y: Option<usize>,
    #[serde(default)]
    pub pattern_count_z: Option<usize>,
}

fn default_dirty() -> bool {
    true
}

fn default_lod() -> f64 {
    crate::brep::DEFAULT_TOLERANCE
}

impl Feature {
    pub fn new_extrude(
        name: impl Into<String>,
        sketch_id: impl Into<String>,
        depth: f64,
        direction: c::Vector3D,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            feature_type: FeatureType::Extrude,
            sketch_id: sketch_id.into(),
            axis_sketch_id: String::new(),
            input_sketch_ids: Vec::new(),
            dependency_ids: Vec::new(),
            depth,
            direction,
            angle: 0.0,
            axis_origin: c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
            axis_direction: c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
            is_cut: false,
            order: 0,
            suppressed: false,
            cached_geometry: None,
            dirty: true,
            lod: default_lod(),
            boolean_op: None,
            pattern_count_x: None,
            pattern_count_y: None,
            pattern_count_z: None,
        }
    }

    pub fn new_revolve(
        name: impl Into<String>,
        sketch_id: impl Into<String>,
        axis_sketch_id: impl Into<String>,
        angle: f64,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            feature_type: FeatureType::Revolve,
            sketch_id: sketch_id.into(),
            axis_sketch_id: axis_sketch_id.into(),
            input_sketch_ids: Vec::new(),
            dependency_ids: Vec::new(),
            depth: 0.0,
            direction: c::Vector3D { x: 0.0, y: 0.0, z: 0.0 },
            angle,
            axis_origin: c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
            axis_direction: c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
            is_cut: false,
            order: 0,
            suppressed: false,
            cached_geometry: None,
            dirty: true,
            lod: default_lod(),
            boolean_op: None,
            pattern_count_x: None,
            pattern_count_y: None,
            pattern_count_z: None,
        }
    }

    pub fn new_sweep(
        name: impl Into<String>,
        sketch_id: impl Into<String>,
        path_sketch_id: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            feature_type: FeatureType::Sweep,
            sketch_id: sketch_id.into(),
            axis_sketch_id: String::new(),
            input_sketch_ids: vec![path_sketch_id.into()],
            dependency_ids: Vec::new(),
            depth: 0.0,
            direction: c::Vector3D { x: 0.0, y: 0.0, z: 0.0 },
            angle: 0.0,
            axis_origin: c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
            axis_direction: c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
            is_cut: false,
            order: 0,
            suppressed: false,
            cached_geometry: None,
            dirty: true,
            lod: default_lod(),
            boolean_op: None,
            pattern_count_x: None,
            pattern_count_y: None,
            pattern_count_z: None,
        }
    }

    pub fn new_loft(
        name: impl Into<String>,
        section_sketch_ids: Vec<String>,
    ) -> Self {
        let first = section_sketch_ids.first().cloned().unwrap_or_default();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            feature_type: FeatureType::Loft,
            sketch_id: first,
            axis_sketch_id: String::new(),
            input_sketch_ids: section_sketch_ids,
            dependency_ids: Vec::new(),
            depth: 0.0,
            direction: c::Vector3D { x: 0.0, y: 0.0, z: 0.0 },
            angle: 0.0,
            axis_origin: c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
            axis_direction: c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
            is_cut: false,
            order: 0,
            suppressed: false,
            cached_geometry: None,
            dirty: true,
            lod: default_lod(),
            boolean_op: None,
            pattern_count_x: None,
            pattern_count_y: None,
            pattern_count_z: None,
        }
    }

    pub fn new_boolean(
        name: impl Into<String>,
        operation: BooleanOp,
        _target_orders: Vec<i32>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            feature_type: FeatureType::Boolean,
            sketch_id: String::new(),
            axis_sketch_id: String::new(),
            input_sketch_ids: Vec::new(),
            dependency_ids: Vec::new(),
            depth: 0.0,
            direction: c::Vector3D { x: 0.0, y: 0.0, z: 0.0 },
            angle: 0.0,
            axis_origin: c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
            axis_direction: c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
            is_cut: operation == BooleanOp::Subtract,
            order: 0,
            suppressed: false,
            cached_geometry: None,
            dirty: true,
            lod: default_lod(),
            boolean_op: Some(operation),
            pattern_count_x: None,
            pattern_count_y: None,
            pattern_count_z: None,
        }
    }

    pub fn new_fusion(
        name: impl Into<String>,
        section_sketch_ids: Vec<String>,
    ) -> Self {
        let first = section_sketch_ids.first().cloned().unwrap_or_default();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            feature_type: FeatureType::Fusion,
            sketch_id: first,
            axis_sketch_id: String::new(),
            input_sketch_ids: section_sketch_ids,
            dependency_ids: Vec::new(),
            depth: 0.0,
            direction: c::Vector3D { x: 0.0, y: 0.0, z: 0.0 },
            angle: 0.0,
            axis_origin: c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
            axis_direction: c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
            is_cut: false,
            order: 0,
            suppressed: false,
            cached_geometry: None,
            dirty: true,
            lod: default_lod(),
            boolean_op: None,
            pattern_count_x: None,
            pattern_count_y: None,
            pattern_count_z: None,
        }
    }

    pub fn new_lofted_fusion(
        name: impl Into<String>,
        section_sketch_ids: Vec<String>,
    ) -> Self {
        let first = section_sketch_ids.first().cloned().unwrap_or_default();
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            feature_type: FeatureType::LoftedFusion,
            sketch_id: first,
            axis_sketch_id: String::new(),
            input_sketch_ids: section_sketch_ids,
            dependency_ids: Vec::new(),
            depth: 0.0,
            direction: c::Vector3D { x: 0.0, y: 0.0, z: 0.0 },
            angle: 0.0,
            axis_origin: c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
            axis_direction: c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
            is_cut: false,
            order: 0,
            suppressed: false,
            cached_geometry: None,
            dirty: true,
            lod: default_lod(),
            boolean_op: None,
            pattern_count_x: None,
            pattern_count_y: None,
            pattern_count_z: None,
        }
    }

    pub fn new_chamfer(
        name: impl Into<String>,
        distance: f64,
        dependency_ids: Vec<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            feature_type: FeatureType::Chamfer,
            sketch_id: String::new(),
            axis_sketch_id: String::new(),
            input_sketch_ids: Vec::new(),
            dependency_ids,
            depth: distance,
            direction: c::Vector3D { x: 0.0, y: 0.0, z: 0.0 },
            angle: 0.0,
            axis_origin: c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
            axis_direction: c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
            is_cut: true,
            order: 0,
            suppressed: false,
            cached_geometry: None,
            dirty: true,
            lod: default_lod(),
            boolean_op: None,
            pattern_count_x: None,
            pattern_count_y: None,
            pattern_count_z: None,
        }
    }

    pub fn new_fillet(
        name: impl Into<String>,
        radius: f64,
        dependency_ids: Vec<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            feature_type: FeatureType::Fillet,
            sketch_id: String::new(),
            axis_sketch_id: String::new(),
            input_sketch_ids: Vec::new(),
            dependency_ids,
            depth: radius,
            direction: c::Vector3D { x: 0.0, y: 0.0, z: 0.0 },
            angle: 0.0,
            axis_origin: c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
            axis_direction: c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
            is_cut: true,
            order: 0,
            suppressed: false,
            cached_geometry: None,
            dirty: true,
            lod: default_lod(),
            boolean_op: None,
            pattern_count_x: None,
            pattern_count_y: None,
            pattern_count_z: None,
        }
    }

    pub fn new_pattern(
        name: impl Into<String>,
        count_x: usize,
        count_y: usize,
        count_z: usize,
        spacing_x: f64,
        spacing_y: f64,
        spacing_z: f64,
        dependency_ids: Vec<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            feature_type: FeatureType::Pattern,
            sketch_id: String::new(),
            axis_sketch_id: String::new(),
            input_sketch_ids: Vec::new(),
            dependency_ids,
            depth: spacing_x,
            direction: c::Vector3D { x: spacing_x, y: spacing_y, z: spacing_z },
            angle: 0.0,
            axis_origin: c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
            axis_direction: c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
            is_cut: false,
            order: 0,
            suppressed: false,
            cached_geometry: None,
            dirty: true,
            lod: default_lod(),
            boolean_op: None,
            pattern_count_x: Some(count_x),
            pattern_count_y: Some(count_y),
            pattern_count_z: Some(count_z),
        }
    }

    pub fn new_shell(
        name: impl Into<String>,
        thickness: f64,
        dependency_ids: Vec<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            feature_type: FeatureType::Shell,
            sketch_id: String::new(),
            axis_sketch_id: String::new(),
            input_sketch_ids: Vec::new(),
            dependency_ids,
            depth: thickness,
            direction: c::Vector3D { x: 0.0, y: 0.0, z: 0.0 },
            angle: 0.0,
            axis_origin: c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
            axis_direction: c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
            is_cut: true,
            order: 0,
            suppressed: false,
            cached_geometry: None,
            dirty: true,
            lod: default_lod(),
            boolean_op: None,
            pattern_count_x: None,
            pattern_count_y: None,
            pattern_count_z: None,
        }
    }

    pub fn path_sketch_id(&self) -> Option<&str> {
        self.input_sketch_ids.first().map(|s| s.as_str())
    }

    pub fn section_sketch_ids(&self) -> &[String] {
        &self.input_sketch_ids
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
        self.cached_geometry = None;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum FeatureType {
    Extrude,
    Revolve,
    Sweep,
    Loft,
    Fusion,
    LoftedFusion,
    Boolean,
    Chamfer,
    Fillet,
    Pattern,
    Shell,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum BooleanOp {
    Union = 0,
    Subtract = 1,
    Intersect = 2,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_extrude() {
        let f = Feature::new_extrude("ext1", "sketch-1", 5.0, c::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
        assert_eq!(f.name, "ext1");
        assert_eq!(f.feature_type, FeatureType::Extrude);
        assert_eq!(f.depth, 5.0);
        assert_eq!(f.sketch_id, "sketch-1");
        assert!(f.dirty);
        assert!(!f.id.is_empty());
    }

    #[test]
    fn test_new_revolve() {
        let f = Feature::new_revolve("rev1", "sketch-1", "axis-1", 3.14159);
        assert_eq!(f.name, "rev1");
        assert_eq!(f.feature_type, FeatureType::Revolve);
        assert_eq!(f.angle, 3.14159);
        assert_eq!(f.axis_sketch_id, "axis-1");
    }

    #[test]
    fn test_new_sweep() {
        let f = Feature::new_sweep("swp1", "sketch-1", "path-1");
        assert_eq!(f.feature_type, FeatureType::Sweep);
        assert_eq!(f.path_sketch_id(), Some("path-1"));
    }

    #[test]
    fn test_new_loft() {
        let f = Feature::new_loft("loft1", vec!["sketch-1".into(), "sketch-2".into()]);
        assert_eq!(f.feature_type, FeatureType::Loft);
        assert_eq!(f.section_sketch_ids().len(), 2);
    }

    #[test]
    fn test_new_boolean() {
        let f = Feature::new_boolean("bool1", BooleanOp::Subtract, vec![0, 1]);
        assert_eq!(f.feature_type, FeatureType::Boolean);
        assert!(f.is_cut);
        assert_eq!(f.boolean_op, Some(BooleanOp::Subtract));
        let f2 = Feature::new_boolean("union1", BooleanOp::Union, vec![]);
        assert!(!f2.is_cut);
        assert_eq!(f2.boolean_op, Some(BooleanOp::Union));
        let f3 = Feature::new_boolean("inter1", BooleanOp::Intersect, vec![]);
        assert!(!f3.is_cut);
        assert_eq!(f3.boolean_op, Some(BooleanOp::Intersect));
    }

    #[test]
    fn test_new_chamfer() {
        let f = Feature::new_chamfer("cham1", 2.0, vec!["dep-1".into()]);
        assert_eq!(f.feature_type, FeatureType::Chamfer);
        assert_eq!(f.depth, 2.0);
        assert!(f.is_cut);
        assert_eq!(f.dependency_ids, vec!["dep-1"]);
    }

    #[test]
    fn test_new_fillet() {
        let f = Feature::new_fillet("fil1", 1.5, vec![]);
        assert_eq!(f.feature_type, FeatureType::Fillet);
        assert_eq!(f.depth, 1.5);
    }

    #[test]
    fn test_new_pattern() {
        let f = Feature::new_pattern("pat1", 2, 3, 1, 5.0, 5.0, 0.0, vec![]);
        assert_eq!(f.feature_type, FeatureType::Pattern);
        assert_eq!(f.pattern_count_x, Some(2));
        assert_eq!(f.pattern_count_y, Some(3));
        assert_eq!(f.pattern_count_z, Some(1));
        assert_eq!(f.depth, 5.0);
    }

    #[test]
    fn test_new_shell() {
        let f = Feature::new_shell("shell1", 0.5, vec![]);
        assert_eq!(f.feature_type, FeatureType::Shell);
        assert_eq!(f.depth, 0.5);
        assert!(f.is_cut);
    }

    #[test]
    fn test_mark_dirty() {
        let mut f = Feature::new_extrude("ext1", "s1", 1.0, c::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
        assert!(f.dirty);
        f.dirty = false;
        f.cached_geometry = Some(c::MeshGeometry {
            vertices: vec![c::Point3D { x: 0.0, y: 0.0, z: 0.0 }],
            normals: Vec::new(), uvs: Vec::new(), faces: Vec::new(),
        });
        f.mark_dirty();
        assert!(f.dirty);
        assert!(f.cached_geometry.is_none());
    }

    #[test]
    fn test_serde_defaults() {
        let json = r#"{"id":"test-1","name":"test","feature_type":"Extrude","sketch_id":"","depth":0.0,"direction":{"x":0,"y":0,"z":0},"angle":0.0,"axis_origin":{"x":0,"y":0,"z":0},"axis_direction":{"x":0,"y":0,"z":1},"is_cut":false,"order":0,"suppressed":false,"dirty":true,"lod":0.01}"#;
        let f: Feature = serde_json::from_str(json).unwrap();
        assert_eq!(f.name, "test");
        assert!(f.dependency_ids.is_empty());
        assert!(f.input_sketch_ids.is_empty());
        assert!(f.axis_sketch_id.is_empty());
        assert!(f.boolean_op.is_none());
        assert!(f.pattern_count_x.is_none());
    }
}
