use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A persisted measurement annotation in the 3D scene.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Measurement {
    pub id: String,
    /// "distance" | "angle" | "area"
    pub r#type: String,
    /// The 3D points defining the measurement (2 for distance, 3 for angle, 3+ for area)
    pub points: Vec<Point3D>,
    /// Computed value (distance in m, angle in degrees, area in m²)
    pub value: f64,
    /// Optional user label
    pub label: String,
    /// Hex color string (e.g. "#44ff88")
    pub color: String,
    /// Creation timestamp (millis since epoch)
    pub created_at: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Point3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Measurement {
    pub fn new(r#type: &str, points: Vec<Point3D>, value: f64, color: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            r#type: r#type.into(),
            points,
            value,
            label: String::new(),
            color: color.into(),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64,
        }
    }
}

/// In-memory measurement store (just a Vec wrapper).
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct MeasurementStore {
    pub measurements: Vec<Measurement>,
}

impl MeasurementStore {
    pub fn new() -> Self {
        Self { measurements: Vec::new() }
    }

    pub fn all(&self) -> &[Measurement] {
        &self.measurements
    }

    pub fn add(&mut self, m: Measurement) {
        self.measurements.push(m);
    }

    pub fn remove(&mut self, id: &str) -> Option<Measurement> {
        let idx = self.measurements.iter().position(|m| m.id == id)?;
        Some(self.measurements.remove(idx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_list() {
        let mut store = MeasurementStore::new();
        let m = Measurement::new(
            "distance",
            vec![Point3D { x: 0.0, y: 0.0, z: 0.0 }, Point3D { x: 3.0, y: 4.0, z: 0.0 }],
            5.0,
            "#44ff88",
        );
        store.add(m.clone());
        assert_eq!(store.all().len(), 1);
        assert_eq!(store.all()[0].id, m.id);
        assert_eq!(store.all()[0].value, 5.0);
    }

    #[test]
    fn test_remove() {
        let mut store = MeasurementStore::new();
        let m = Measurement::new("area", vec![], 10.0, "#44aaff");
        store.add(m.clone());
        let removed = store.remove(&m.id).unwrap();
        assert_eq!(removed.id, m.id);
        assert!(store.all().is_empty());
        assert!(store.remove("nonexistent").is_none());
    }

    #[test]
    fn test_new_sets_metadata() {
        let m = Measurement::new("angle", vec![], 90.0, "#ffaa44");
        assert!(!m.id.is_empty());
        assert_eq!(m.r#type, "angle");
        assert!(m.created_at > 0);
    }
}