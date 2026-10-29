//! Opening volumes (P8): a host-relative void box (IfcOpeningElement).
//!
//! Doors/windows already carry the *host wall patch with the hole* as their
//! own geometry (`boolean_subtract::build_door_opening`). A standalone opening
//! is the complementary object: the void volume itself, placed in the host
//! wall's local frame. The host mesh is deliberately **not** re-cut here —
//! feature-tree geometry stays authoritative (Wave 3); see CODE_DEVIATIONS
//! for the recorded 🔀.

use c::{MeshGeometry, Point3D, Transform};
use omnim_core as c;

/// Build the positioned void box for an opening.
///
/// `wall_length`/`wall_height` come from the host wall's bbox (same fallback
/// as doors/windows); `position_x`/`position_z` are measured from the wall's
/// minimum corner, matching `build_door_opening`.
pub fn build_opening_volume(
    wall_length: f64,
    wall_height: f64,
    wall_thickness: f64,
    opening_width: f64,
    opening_height: f64,
    position_x: f64,
    position_z: f64,
) -> MeshGeometry {
    use crate::geometry::transform_mesh;
    use crate::mesh_gen;

    let (v, n, f) = mesh_gen::box_mesh(&c::Box {
        length: opening_width,
        width: wall_thickness * 1.1,
        height: opening_height,
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
                x: position_x - wall_length / 2.0 + opening_width / 2.0,
                y: 0.0,
                z: position_z - wall_height / 2.0 + opening_height / 2.0,
            }),
            rotation: None,
            scale: None,
            matrix: Vec::new(),
        },
    )
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
    fn opening_volume_has_requested_dimensions_and_position() {
        let mesh = build_opening_volume(5.0, 3.0, 0.3, 1.0, 2.1, 1.0, 0.5);
        let (mn, mx) = bbox(&mesh);
        assert!((mx[0] - mn[0] - 1.0).abs() < 1e-9, "width");
        assert!(
            (mx[1] - mn[1] - 0.33).abs() < 1e-9,
            "wall thickness + tolerance"
        );
        assert!((mx[2] - mn[2] - 2.1).abs() < 1e-9, "height");
        // position_x/position_z are measured from the wall's min corner
        // (box_mesh is origin-centred, wall spans -2.5..2.5 in X).
        assert!((mn[0] - (-1.5)).abs() < 1e-9, "x offset: {}", mn[0]);
        assert!((mn[2] - (-1.0)).abs() < 1e-9, "z offset: {}", mn[2]);
    }

    #[test]
    fn opening_volume_scales_with_the_request() {
        let small = build_opening_volume(5.0, 3.0, 0.3, 0.8, 1.2, 1.0, 0.5);
        let large = build_opening_volume(5.0, 3.0, 0.3, 1.6, 2.4, 1.0, 0.5);
        assert!(large.vertices.len() == small.vertices.len());
        let (smn, smx) = bbox(&small);
        let (lmn, lmx) = bbox(&large);
        assert!(lmx[0] - lmn[0] > smx[0] - smn[0]);
        assert!(lmx[2] - lmn[2] > smx[2] - smn[2]);
    }
}
