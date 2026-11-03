//! Elbow fitting placeholders for routed pipes (P8-S4).
//!
//! A routed pipe already sweeps continuously through bends; the fitting object
//! is a distinct, selectable part at each internal bend so that the bill of
//! materials and connector binding have something to attach to. Geometry is a
//! sphere of 1.15× the pipe radius — an honest stand-in until real mitered/arc
//! elbow geometry lands (documented in the capability matrix).

use c::{MeshGeometry, Point3D, Transform};
use omnim_core as c;

/// Build the elbow fitting volume centred on a bend point.
pub fn build_elbow_fitting(center: [f64; 3], radius: f64) -> MeshGeometry {
    use crate::geometry::transform_mesh;
    use crate::mesh_gen;

    let (v, n, f) = mesh_gen::sphere_mesh(&c::Sphere {
        radius: radius.max(1e-3),
        segments: 16,
    });
    transform_mesh(
        &MeshGeometry {
            vertices: v,
            normals: n,
            uvs: Vec::new(),
            faces: f,
        },
        &Transform {
            translation: Some(Point3D {
                x: center[0],
                y: center[1],
                z: center[2],
            }),
            rotation: None,
            scale: None,
            matrix: Vec::new(),
        },
    )
}

/// Build the tee fitting volume centred on a junction point (Phase 8-3).
///
/// Placeholder geometry like [`build_elbow_fitting`]: a sphere marking where
/// the branch meets the run. A swept three-way body (run-through + branch
/// stud) is the future refinement.
pub fn build_tee_fitting(center: [f64; 3], radius: f64) -> MeshGeometry {
    build_elbow_fitting(center, radius)
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
    fn tee_fitting_is_centred_on_the_junction() {
        let mesh = build_tee_fitting([0.5, 0.5, 0.0], 0.15);
        assert!(!mesh.vertices.is_empty());
        let (mn, mx) = bbox(&mesh);
        for d in 0..3 {
            let mid = (mx[d] + mn[d]) / 2.0;
            assert!((mid - [0.5, 0.5, 0.0][d]).abs() < 1e-6, "axis {d} centred");
        }
        assert!(mx[0] - mn[0] > 0.0);
    }

    #[test]
    fn elbow_fitting_is_centred_on_the_bend() {
        let mesh = build_elbow_fitting([1.0, 2.0, 3.0], 0.2);
        assert!(!mesh.vertices.is_empty());
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
        let center = [
            (mn[0] + mx[0]) / 2.0,
            (mn[1] + mx[1]) / 2.0,
            (mn[2] + mx[2]) / 2.0,
        ];
        assert!((center[0] - 1.0).abs() < 1e-6);
        assert!((center[1] - 2.0).abs() < 1e-6);
        assert!((center[2] - 3.0).abs() < 1e-6);
        // Tessellated sphere: never wider than 2r, and close to it.
        let width = mx[0] - mn[0];
        assert!(
            width <= 0.4 + 1e-6 && width > 0.35,
            "diameter ~2r: {}",
            width
        );
    }
}
