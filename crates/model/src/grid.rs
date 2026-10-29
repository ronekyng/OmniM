use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Axis direction of a grid datum line (P8-S1).
///
/// `AlongX` means the line runs parallel to the world X axis at
/// `y = coordinate` (the usual "numbered" grid, 1, 2, 3…); `AlongY` runs
/// parallel to Y at `x = coordinate` (the usual "lettered" grid, A, B, C…).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridAxis {
    AlongX,
    AlongY,
}

impl GridAxis {
    pub fn from_proto(v: i32) -> Option<Self> {
        match omnim_core::GridAxis::try_from(v).ok()? {
            omnim_core::GridAxis::AlongX => Some(GridAxis::AlongX),
            omnim_core::GridAxis::AlongY => Some(GridAxis::AlongY),
            _ => None,
        }
    }

    pub fn to_proto(self) -> i32 {
        match self {
            GridAxis::AlongX => omnim_core::GridAxis::AlongX as i32,
            GridAxis::AlongY => omnim_core::GridAxis::AlongY as i32,
        }
    }
}

/// A single grid datum line (轴网轴线). Grids are datum annotations, not
/// objects: they carry no geometry/mesh and are not part of the object tree
/// (mirrors `MeasurementStore`, P8-S1).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GridLine {
    pub id: String,
    /// Axis label, e.g. "A"/"B"/… or "1"/"2"/…
    pub label: String,
    /// Which axis the line runs along (the fixed coordinate is the other one).
    pub axis: GridAxis,
    /// Fixed coordinate in meters (y for AlongX, x for AlongY).
    pub coordinate: f64,
    /// Line extent along its running direction (meters); `None`-like 0/0
    /// means unbounded in the UI.
    pub extent_min: f64,
    pub extent_max: f64,
}

impl GridLine {
    pub fn new(label: impl Into<String>, axis: GridAxis, coordinate: f64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            label: label.into(),
            axis,
            coordinate,
            extent_min: 0.0,
            extent_max: 0.0,
        }
    }
}

/// In-memory grid datum store.
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct GridStore {
    pub grid_lines: Vec<GridLine>,
}

impl GridStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, line: GridLine) {
        self.grid_lines.push(line);
    }

    pub fn get(&self, id: &str) -> Option<&GridLine> {
        self.grid_lines.iter().find(|g| g.id == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut GridLine> {
        self.grid_lines.iter_mut().find(|g| g.id == id)
    }

    pub fn remove(&mut self, id: &str) -> Option<GridLine> {
        let pos = self.grid_lines.iter().position(|g| g.id == id)?;
        Some(self.grid_lines.remove(pos))
    }

    pub fn all(&self) -> &[GridLine] {
        &self.grid_lines
    }

    /// Sorted view: AlongX (numbered) first by coordinate, then AlongY
    /// (lettered) by coordinate — the conventional drawing order.
    pub fn sorted(&self) -> Vec<&GridLine> {
        let mut lines: Vec<&GridLine> = self.grid_lines.iter().collect();
        lines.sort_by(|a, b| {
            (a.axis as u8).cmp(&(b.axis as u8)).then_with(|| {
                a.coordinate
                    .partial_cmp(&b.coordinate)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });
        lines
    }

    pub fn find_by_label(&self, label: &str) -> Option<&GridLine> {
        self.grid_lines.iter().find(|g| g.label == label)
    }

    /// Uniqueness guard: one label per axis family (A, B… / 1, 2…).
    pub fn label_taken(&self, label: &str, axis: GridAxis) -> bool {
        self.grid_lines
            .iter()
            .any(|g| g.label == label && g.axis == axis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_store_roundtrip_and_uniqueness() {
        let mut store = GridStore::new();
        let mut a = GridLine::new("A", GridAxis::AlongY, 0.0);
        a.coordinate = 0.0;
        let b = GridLine::new("B", GridAxis::AlongY, 6.0);
        let one = GridLine::new("1", GridAxis::AlongX, 0.0);
        store.insert(a.clone());
        store.insert(b);
        store.insert(one);

        assert_eq!(store.all().len(), 3);
        assert!(store.get(&a.id).is_some());
        assert!(store.label_taken("A", GridAxis::AlongY));
        assert!(
            !store.label_taken("A", GridAxis::AlongX),
            "labels are unique per axis family"
        );

        let removed = store.remove(&a.id).unwrap();
        assert_eq!(removed.label, "A");
        assert!(store.get(&a.id).is_none());

        // serde roundtrip (project persistence path)
        let json = serde_json::to_string(&store).unwrap();
        let back: GridStore = serde_json::from_str(&json).unwrap();
        assert_eq!(back.all().len(), 2);
    }

    #[test]
    fn project_persistence_includes_grids_and_defaults_for_old_files() {
        let mut p = crate::Project::new("P");
        p.grids.insert(GridLine::new("A", GridAxis::AlongY, 0.0));
        let mut v = serde_json::to_value(&p).unwrap();
        assert_eq!(
            v["grids"]["grid_lines"].as_array().unwrap().len(),
            1,
            "grids must be persisted with the project"
        );
        // Old project files (pre P8-S1) have no `grids` field — the store
        // must default to empty instead of failing deserialization.
        v.as_object_mut().unwrap().remove("grids");
        let back: crate::Project = serde_json::from_value(v).unwrap();
        assert_eq!(back.grids.all().len(), 0);
    }

    #[test]
    fn grid_axis_proto_conversion_rejects_unspecified() {
        assert_eq!(
            GridAxis::from_proto(omnim_core::GridAxis::AlongX as i32),
            Some(GridAxis::AlongX)
        );
        assert_eq!(
            GridAxis::from_proto(0),
            None,
            "UNSPECIFIED must not map to a datum axis"
        );
    }
}
