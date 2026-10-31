//! Hosted door/window panels (P8 regression fix).
//!
//! Historically a Door/Window object carried the **whole host wall with a hole
//! cut into it** (`boolean_subtract::build_door_opening`) and was never
//! transformed into the wall's frame. That made the door's bounding box equal
//! to the wall and left the patch floating at the origin.
//!
//! This module builds a real leaf + frame in the wall's local frame instead:
//! local +X runs along the wall, +Y is the wall normal, +Z is up from the
//! opening base. The host wall itself is still not re-cut (feature-tree
//! geometry stays authoritative); an explicit `Opening` object is the void.

use omnim_core as c;
use omnim_core::MeshGeometry;

use crate::geometry::{merge_meshes, transform_mesh};
use crate::mesh_gen;

/// Frame member thickness (m). Capped relative to the opening so small
/// windows do not end up as a frame with no leaf.
fn frame_width(width: f64, height: f64) -> f64 {
    0.06_f64.min(width * 0.15).min(height * 0.15).max(0.01)
}

/// Leaf + frame in the wall's local frame.
///
/// * `width`, `height` — clear opening size (m)
/// * `wall_thickness` — host wall thickness (m); the panel is centred in it
///
/// The result sits on `z = 0` (opening base) and is centred on `x = 0`.
pub fn build_opening_panel(width: f64, height: f64, wall_thickness: f64) -> MeshGeometry {
    let w = width.max(0.1);
    let h = height.max(0.1);
    let depth = wall_thickness.max(0.06);
    let frame = frame_width(w, h);

    let leaf_w = (w - 2.0 * frame).max(0.05);
    let leaf_h = (h - frame).max(0.05);
    let leaf_t = (depth * 0.25).clamp(0.03, 0.06);

    let mut parts = Vec::new();

    // Leaf: centred in the wall, bottom on the opening base.
    let (lv, ln, lf) = mesh_gen::box_mesh(&c::Box {
        length: leaf_w,
        width: leaf_t,
        height: leaf_h,
    });
    let leaf = transform_mesh(
        &MeshGeometry {
            vertices: lv,
            normals: ln,
            uvs: Vec::new(),
            faces: lf,
        },
        &translation(0.0, 0.0, leaf_h / 2.0),
    );
    parts.push(leaf);

    // Jambs (left/right) + head.
    let jamb_x = w / 2.0 - frame / 2.0;
    for x in [-jamb_x, jamb_x] {
        let (v, n, f) = mesh_gen::box_mesh(&c::Box {
            length: frame,
            width: depth,
            height: h,
        });
        parts.push(transform_mesh(
            &MeshGeometry {
                vertices: v,
                normals: n,
                uvs: Vec::new(),
                faces: f,
            },
            &translation(x, 0.0, h / 2.0),
        ));
    }
    let (hv, hn, hf) = mesh_gen::box_mesh(&c::Box {
        length: w,
        width: depth,
        height: frame,
    });
    parts.push(transform_mesh(
        &MeshGeometry {
            vertices: hv,
            normals: hn,
            uvs: Vec::new(),
            faces: hf,
        },
        &translation(0.0, 0.0, h - frame / 2.0),
    ));

    merge_meshes(&parts)
}

/// Point + unit direction (XY) at `distance` along an open polyline.
///
/// `distance` is clamped to the polyline; the z of the containing segment is
/// interpolated so sloped walls keep their datum.
pub fn frame_at_path(points: &[[f64; 3]], distance: f64) -> Option<([f64; 3], [f64; 2])> {
    if points.len() < 2 {
        return None;
    }
    let mut remaining = distance.max(0.0);
    for (index, pair) in points.windows(2).enumerate() {
        let a = pair[0];
        let b = pair[1];
        let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
        let len = (dx * dx + dy * dy).sqrt();
        if len <= 1e-9 {
            continue;
        }
        let is_last = index + 2 == points.len();
        if remaining <= len || is_last {
            let t = (remaining / len).clamp(0.0, 1.0);
            let point = [a[0] + dx * t, a[1] + dy * t, a[2] + (b[2] - a[2]) * t];
            return Some((point, [dx / len, dy / len]));
        }
        remaining -= len;
    }
    None
}

/// Rotate the local panel (X along wall, Z up) and move it to `point`.
pub fn place_panel(mesh: &MeshGeometry, point: [f64; 3], direction: [f64; 2]) -> MeshGeometry {
    let angle = direction[1].atan2(direction[0]);
    let (half_sin, half_cos) = ((angle * 0.5).sin(), (angle * 0.5).cos());
    transform_mesh(
        mesh,
        &c::Transform {
            matrix: Vec::new(),
            translation: Some(c::Point3D {
                x: point[0],
                y: point[1],
                z: point[2],
            }),
            rotation: Some(c::Rotation {
                x: 0.0,
                y: 0.0,
                z: half_sin,
                w: half_cos,
            }),
            scale: None,
        },
    )
}

fn translation(x: f64, y: f64, z: f64) -> c::Transform {
    c::Transform {
        matrix: Vec::new(),
        translation: Some(c::Point3D { x, y, z }),
        rotation: None,
        scale: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bbox(mesh: &MeshGeometry) -> ([f64; 3], [f64; 3]) {
        let b = crate::geometry::compute_bbox(mesh);
        let mn = b.min.map(|p| [p.x, p.y, p.z]).unwrap();
        let mx = b.max.map(|p| [p.x, p.y, p.z]).unwrap();
        (mn, mx)
    }

    #[test]
    fn panel_matches_the_opening_not_the_host_wall() {
        let mesh = build_opening_panel(0.9, 2.1, 0.2);
        let (mn, mx) = bbox(&mesh);
        assert!((mx[0] - mn[0] - 0.9).abs() < 1e-6, "width");
        assert!((mx[1] - mn[1] - 0.2).abs() < 1e-6, "depth");
        assert!((mx[2] - mn[2] - 2.1).abs() < 1e-6, "height");
        assert!((mn[2]).abs() < 1e-6, "sits on the opening base");
    }

    #[test]
    fn frame_at_path_interpolates_and_clamps() {
        let path = [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]];
        let (p, d) = frame_at_path(&path, 5.0).unwrap();
        assert!((p[0] - 5.0).abs() < 1e-9 && p[1].abs() < 1e-9);
        assert!((d[0] - 1.0).abs() < 1e-9 && d[1].abs() < 1e-9);
        let (p, _) = frame_at_path(&path, 99.0).unwrap();
        assert!((p[0] - 10.0).abs() < 1e-9, "clamped to the end");
    }

    #[test]
    fn frame_at_path_follows_a_bend() {
        let path = [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [4.0, 3.0, 0.0]];
        let (p, d) = frame_at_path(&path, 5.0).unwrap();
        assert!((p[0] - 4.0).abs() < 1e-9 && (p[1] - 1.0).abs() < 1e-9);
        assert!(d[0].abs() < 1e-9 && (d[1] - 1.0).abs() < 1e-9);
    }

    #[test]
    fn place_panel_rotates_local_x_onto_the_wall() {
        let base = build_opening_panel(1.0, 2.0, 0.2);
        let placed = place_panel(&base, [5.0, 5.0, 3.0], [0.0, 1.0]);
        let (mn, mx) = bbox(&placed);
        assert!((mx[0] - mn[0] - 0.2).abs() < 1e-6, "depth along X after 90°");
        assert!((mx[1] - mn[1] - 1.0).abs() < 1e-6, "width along Y after 90°");
        assert!((mn[2] - 3.0).abs() < 1e-6);
    }
}
