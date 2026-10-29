//! Pipe routing (P8): a circular section swept along an N-point polyline.
//!
//! Kernel: the LinearComponent engine (`gen_rule::linear_component`), which
//! interpolates the path by arc length and sweeps a ring with recomputed
//! Frenet frames — bends therefore come out mitered/continuous instead of two
//! disjoint cylinders. This is the intended Phase-8 use of the Linear family
//! (ROADMAP: "Elbow … Phase 8 pipe routing"); branches and fitting objects are
//! still out of scope, and the route is stored as a property (the Linear
//! family does not have a feature-tree template yet — documented transitional).

use c::MeshGeometry;
use omnim_core as c;

use super::linear_component::{
    self, CrossSectionAtStation, CrossSectionLayer, CrossSectionShape, LinearComponentInput,
};

/// Minimum ring resolution for a pipe (`ring_segments`).
pub const DEFAULT_RING_SEGMENTS: usize = 16;

/// Sweep a circular pipe along the route. `radius` is in metres.
///
/// Errors (mapped to 400 by callers): fewer than 2 points, non-positive
/// radius, or a path with zero-length/duplicate segments.
pub fn build_pipe_route(
    points: &[[f64; 3]],
    radius: f64,
    ring_segments: usize,
) -> Result<MeshGeometry, String> {
    if points.len() < 2 {
        return Err("pipe route requires at least 2 points".into());
    }
    if !(radius > 0.0) {
        return Err("pipe radius must be positive".into());
    }
    // 3D validation (a vertical riser has identical planar points, so the
    // planar path_geom check would wrongly reject it).
    for w in points.windows(2) {
        let (a, b) = (w[0], w[1]);
        let (dx, dy, dz) = (b[0] - a[0], b[1] - a[1], b[2] - a[2]);
        if (dx * dx + dy * dy + dz * dz).sqrt() < 1e-9 {
            return Err("path has a zero-length segment".into());
        }
    }

    let alignment: Vec<c::Point3D> = points
        .iter()
        .map(|p| c::Point3D {
            x: p[0],
            y: p[1],
            z: p[2],
        })
        .collect();
    // ~8 samples per picked segment keeps bends smooth without exploding the
    // vertex count on long routes.
    let segments = (points.len() * 8).max(16);
    let input = LinearComponentInput {
        alignment,
        cross_sections: vec![CrossSectionAtStation {
            station: 0.0,
            layers: vec![CrossSectionLayer {
                name: "Pipe".into(),
                shape: CrossSectionShape::Circular,
                default_width: radius * 2.0,
                default_thickness: 0.0,
                material_id: None,
                function: "pipe".into(),
            }],
        }],
        segments,
        ring_segments: ring_segments.max(6),
    };
    Ok(linear_component::generate(&input))
}

/// Sum of the route's straight-segment lengths (metres).
pub fn route_length(points: &[[f64; 3]]) -> f64 {
    points
        .windows(2)
        .map(|w| {
            let (a, b) = (w[0], w[1]);
            let (dx, dy, dz) = (b[0] - a[0], b[1] - a[1], b[2] - a[2]);
            (dx * dx + dy * dy + dz * dz).sqrt()
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bbox(mesh: &MeshGeometry) -> ([f64; 3], [f64; 3]) {
        let mut mn = [f64::INFINITY; 3];
        let mut mx = [f64::NEG_INFINITY; 3];
        for v in &mesh.vertices {
            mn[0] = mn[0].min(v.x);
            mn[1] = mn[1].min(v.y);
            mn[2] = mn[2].min(v.z);
            mx[0] = mx[0].max(v.x);
            mx[1] = mx[1].max(v.y);
            mx[2] = mx[2].max(v.z);
        }
        (mn, mx)
    }

    #[test]
    fn straight_route_spans_the_path_and_uses_the_radius() {
        let route = [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]];
        let mesh = build_pipe_route(&route, 0.25, 12).unwrap();
        assert!(!mesh.vertices.is_empty());
        let (mn, mx) = bbox(&mesh);
        assert!(
            mx[0] - mn[0] >= 10.0 - 1e-9,
            "pipe body spans the route: {}",
            mx[0] - mn[0]
        );
        // Cross-section: diameter 0.5 in the section plane.
        assert!(
            mx[1] - mn[1] >= 0.5 - 1e-6,
            "section diameter: {}",
            mx[1] - mn[1]
        );
        assert!((route_length(&route) - 10.0).abs() < 1e-9);
    }

    #[test]
    fn bent_route_sweeps_through_the_elbow() {
        let route = [[0.0, 0.0, 0.0], [6.0, 0.0, 0.0], [6.0, 6.0, 0.0]];
        let mesh = build_pipe_route(&route, 0.2, 12).unwrap();
        let (mn, mx) = bbox(&mesh);
        assert!(mx[0] - mn[0] > 6.0, "x extent");
        assert!(mx[1] - mn[1] > 6.0, "y extent covers the bend");
        assert!(mesh.faces.len() > 100, "bend is subdivided");
        assert!((route_length(&route) - 12.0).abs() < 1e-9);
    }

    #[test]
    fn vertical_route_keeps_z() {
        let route = [[0.0, 0.0, 0.0], [0.0, 0.0, 4.0]];
        let mesh = build_pipe_route(&route, 0.15, 12).unwrap();
        let (mn, mx) = bbox(&mesh);
        assert!((mn[2] - 0.0).abs() < 1e-6);
        assert!((mx[2] - 4.0).abs() < 1e-6);
    }

    #[test]
    fn invalid_routes_are_rejected() {
        assert!(build_pipe_route(&[[0.0; 3]], 0.2, 12)
            .unwrap_err()
            .contains("at least 2"));
        assert!(build_pipe_route(&[[0.0; 3], [1.0, 0.0, 0.0]], 0.0, 12)
            .unwrap_err()
            .contains("positive"));
        let dup = [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]];
        assert!(build_pipe_route(&dup, 0.2, 12)
            .unwrap_err()
            .contains("zero-length"));
    }
}
