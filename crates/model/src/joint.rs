use serde::{Deserialize, Serialize};
use uuid::Uuid;

use omnim_core as c;

/// Mechanical joint type between two objects (Phase 8-4,
/// docs/specification/ASSEMBLY.zh-CN.md §9). Mirrors the proto `JointKind`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum JointKind {
    /// Rotation about `axis` through `origin` (limits in radians).
    Revolute,
    /// Translation along `axis` (limits in metres).
    Prismatic,
    /// Welded pair: no relative motion.
    Fixed,
    /// Ball joint about `origin`, free rotation.
    Spherical,
}

impl JointKind {
    pub fn from_proto(v: i32) -> Option<Self> {
        match omnim_core::JointKind::try_from(v).ok()? {
            omnim_core::JointKind::JointRevolute => Some(JointKind::Revolute),
            omnim_core::JointKind::JointPrismatic => Some(JointKind::Prismatic),
            omnim_core::JointKind::JointFixed => Some(JointKind::Fixed),
            omnim_core::JointKind::JointSpherical => Some(JointKind::Spherical),
            _ => None,
        }
    }

    pub fn to_proto(self) -> i32 {
        match self {
            JointKind::Revolute => omnim_core::JointKind::JointRevolute as i32,
            JointKind::Prismatic => omnim_core::JointKind::JointPrismatic as i32,
            JointKind::Fixed => omnim_core::JointKind::JointFixed as i32,
            JointKind::Spherical => omnim_core::JointKind::JointSpherical as i32,
        }
    }

    /// Revolute/prismatic carry a motion axis; fixed/spherical do not
    /// (spherical rotates freely, fixed does not move).
    pub fn needs_axis(self) -> bool {
        matches!(self, JointKind::Revolute | JointKind::Prismatic)
    }

    /// Only axis-driven kinds take numeric limits (radians/millimetres).
    pub fn takes_limits(self) -> bool {
        matches!(self, JointKind::Revolute | JointKind::Prismatic)
    }
}

/// A motion joint between two objects. Joints are mechanism annotations
/// (like grids/measurements): no geometry, and not part of the object tree.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MotionJoint {
    pub id: String,
    pub name: String,
    pub kind: JointKind,
    /// First object of the pair (driving side by convention).
    pub from_object_id: String,
    /// Second object of the pair.
    pub to_object_id: String,
    /// Joint origin in metres (rotation centre / slide datum).
    pub origin: c::Point3D,
    /// Unit axis of motion; `None` for fixed/spherical.
    pub axis: Option<c::Vector3D>,
    /// Motion limits: radians (revolute) or metres (prismatic). Both set or
    /// both absent (P8-S2 extent convention).
    pub limit_min: Option<f64>,
    pub limit_max: Option<f64>,
}

impl MotionJoint {
    /// Create a joint. `axis` is normalized here; validation of domain rules
    /// (axis presence, limit sanity) happens in the Validator.
    pub fn new(
        name: impl Into<String>,
        kind: JointKind,
        from_object_id: impl Into<String>,
        to_object_id: impl Into<String>,
        origin: c::Point3D,
        axis: Option<c::Vector3D>,
    ) -> Self {
        let axis = axis
            .filter(|v| v.x * v.x + v.y * v.y + v.z * v.z > f64::EPSILON)
            .map(|v| {
                let len = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
                c::Vector3D {
                    x: v.x / len,
                    y: v.y / len,
                    z: v.z / len,
                }
            });
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            kind,
            from_object_id: from_object_id.into(),
            to_object_id: to_object_id.into(),
            origin,
            axis,
            limit_min: None,
            limit_max: None,
        }
    }
}

/// In-memory motion joint store (`MeasurementStore` pattern).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct JointStore {
    pub joints: Vec<MotionJoint>,
}

impl JointStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn list(&self) -> &[MotionJoint] {
        &self.joints
    }

    pub fn get(&self, id: &str) -> Option<&MotionJoint> {
        self.joints.iter().find(|j| j.id == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut MotionJoint> {
        self.joints.iter_mut().find(|j| j.id == id)
    }

    pub fn insert(&mut self, joint: MotionJoint) {
        self.joints.push(joint);
    }

    pub fn remove(&mut self, id: &str) -> Option<MotionJoint> {
        let pos = self.joints.iter().position(|j| j.id == id)?;
        Some(self.joints.remove(pos))
    }

    pub fn remove_many<F>(&mut self, mut predicate: F) -> Vec<MotionJoint>
    where
        F: FnMut(&MotionJoint) -> bool,
    {
        let (removed, kept): (Vec<_>, Vec<_>) = self.joints.drain(..).partition(|j| predicate(j));
        self.joints = kept;
        removed
    }

    /// Every joint that touches either endpoint.
    pub fn by_object(&self, object_id: &str) -> Vec<&MotionJoint> {
        self.joints
            .iter()
            .filter(|j| j.from_object_id == object_id || j.to_object_id == object_id)
            .collect()
    }

    /// Same unordered object pair with the same kind already joined?
    pub fn duplicate_pair(&self, from: &str, to: &str, kind: JointKind) -> bool {
        self.joints.iter().any(|j| {
            j.kind == kind
                && ((j.from_object_id == from && j.to_object_id == to)
                    || (j.from_object_id == to && j.to_object_id == from))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f64, y: f64, z: f64) -> c::Point3D {
        c::Point3D { x, y, z }
    }

    fn v(x: f64, y: f64, z: f64) -> c::Vector3D {
        c::Vector3D { x, y, z }
    }

    #[test]
    fn axis_is_normalized() {
        let j = MotionJoint::new(
            "hinge",
            JointKind::Revolute,
            "a",
            "b",
            p(0.0, 0.0, 1.0),
            Some(v(0.0, 0.0, 10.0)),
        );
        let axis = j.axis.unwrap();
        assert!((axis.z - 1.0).abs() < 1e-12);
        assert!((axis.y * axis.y + axis.x * axis.x).abs() < 1e-12);
    }

    #[test]
    fn zero_axis_is_dropped() {
        let j = MotionJoint::new(
            "hinge",
            JointKind::Revolute,
            "a",
            "b",
            p(0.0, 0.0, 0.0),
            Some(v(0.0, 0.0, 0.0)),
        );
        assert!(j.axis.is_none());
    }

    #[test]
    fn duplicate_pair_is_order_insensitive() {
        let mut store = JointStore::new();
        let j = MotionJoint::new(
            "j1",
            JointKind::Revolute,
            "a",
            "b",
            p(0.0, 0.0, 0.0),
            Some(v(0.0, 0.0, 1.0)),
        );
        store.insert(j);
        assert!(store.duplicate_pair("a", "b", JointKind::Revolute));
        assert!(store.duplicate_pair("b", "a", JointKind::Revolute));
        assert!(!store.duplicate_pair("a", "b", JointKind::Prismatic));
        assert!(!store.duplicate_pair("a", "c", JointKind::Revolute));
    }

    #[test]
    fn remove_many_collects_touched_joints() {
        let mut store = JointStore::new();
        store.insert(MotionJoint::new(
            "j1",
            JointKind::Fixed,
            "gone",
            "keep",
            p(0.0, 0.0, 0.0),
            None,
        ));
        store.insert(MotionJoint::new(
            "j2",
            JointKind::Fixed,
            "other",
            "keep",
            p(0.0, 0.0, 0.0),
            None,
        ));
        let removed = store.remove_many(|j| j.from_object_id == "gone" || j.to_object_id == "gone");
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].name, "j1");
        assert_eq!(store.joints.len(), 1);
        assert!(store.by_object("other").len() == 1);
    }
}
