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
/// Opening panel variants (Phase 8-1): leaf layout plus sill/lintel extras.
#[derive(Clone, Copy, Debug, Default)]
pub struct PanelOptions {
    /// 1 = single leaf, 2 = double leaf (centre mullion), more = casement
    /// banks (windows). Clamped by callers.
    pub leaf_count: usize,
    /// Outward sill slab at the opening base (window convention).
    pub with_sill: bool,
    /// Lintel beam spanning the opening width above the head.
    pub with_lintel: bool,
}

impl PanelOptions {
    pub fn leaf_count_clamped(leaf_count: usize) -> usize {
        leaf_count.clamp(1, 4)
    }
}

pub fn build_opening_panel(
    width: f64,
    height: f64,
    wall_thickness: f64,
    options: PanelOptions,
) -> MeshGeometry {
    let w = width.max(0.1);
    let h = height.max(0.1);
    let depth = wall_thickness.max(0.06);
    let frame = frame_width(w, h);

    let leaf_w = (w - 2.0 * frame).max(0.05);
    let leaf_h = (h - frame).max(0.05);
    let leaf_t = (depth * 0.25).clamp(0.03, 0.06);

    let mut parts = Vec::new();

    // Leaves: `leaf_count` panels spanning the clear opening, separated by
    // mullions (each mullion takes the frame thickness). leaf_count = 1 keeps
    // the historic single leaf.
    let leaves = PanelOptions::leaf_count_clamped(options.leaf_count).max(1);
    let mullions = leaves.saturating_sub(1);
    let mullion_t = frame;
    let leaf_w = ((w - mullions as f64 * mullion_t) / leaves as f64).max(0.05);
    for i in 0..leaves {
        let (lv, ln, lf) = mesh_gen::box_mesh(&c::Box {
            length: leaf_w,
            width: leaf_t,
            height: leaf_h,
        });
        // Centre the leaf span on x = 0.
        let span = leaf_w * leaves as f64 + mullions as f64 * mullion_t;
        let x0 = -span / 2.0;
        let cx = x0 + (i as f64 + 0.5) * leaf_w + i as f64 * mullion_t;
        let leaf = transform_mesh(
            &MeshGeometry {
                vertices: lv,
                normals: ln,
                uvs: Vec::new(),
                faces: lf,
            },
            &translation(cx, 0.0, leaf_h / 2.0),
        );
        parts.push(leaf);
        if i + 1 < leaves {
            // Mullion between leaves i and i+1.
            let (mv, mn, mf) = mesh_gen::box_mesh(&c::Box {
                length: mullion_t,
                width: leaf_t,
                height: leaf_h,
            });
            let mx = x0 + (i + 1) as f64 * leaf_w + (i as f64 + 0.5) * mullion_t;
            parts.push(transform_mesh(
                &MeshGeometry {
                    vertices: mv,
                    normals: mn,
                    uvs: Vec::new(),
                    faces: mf,
                },
                &translation(mx, 0.0, leaf_h / 2.0),
            ));
        }
    }
    // Horizontal filler between the leaf band and the head/frame sides is the
    // frame's own gap: the jamb/head boxes below keep the full opening covered.

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

    // Phase 8-1 extras: sill (outward slab at the base) and lintel (spanning
    // beam above the head). Both stay inside the wall's local frame — the
    // caller rotates/places the whole merged panel.
    if options.with_sill {
        let proj = (depth * 0.2).clamp(0.04, 0.12);
        let th = (depth * 0.1).clamp(0.03, 0.08);
        let (sv, sn, sf) = mesh_gen::box_mesh(&c::Box {
            length: w + 2.0 * proj,
            width: depth + proj,
            height: th,
        });
        parts.push(transform_mesh(
            &MeshGeometry {
                vertices: sv,
                normals: sn,
                uvs: Vec::new(),
                faces: sf,
            },
            &translation(0.0, 0.0, -th / 2.0),
        ));
    }
    if options.with_lintel {
        let lh = (depth * 0.5).clamp(0.08, 0.25);
        let (lv2, ln2, lf2) = mesh_gen::box_mesh(&c::Box {
            length: w,
            width: depth,
            height: lh,
        });
        parts.push(transform_mesh(
            &MeshGeometry {
                vertices: lv2,
                normals: ln2,
                uvs: Vec::new(),
                faces: lf2,
            },
            &translation(0.0, 0.0, h + lh / 2.0),
        ));
    }

    merge_meshes(&parts)
}

/// Outward sill slab (Phase 8-1): handled inside [`build_opening_panel`] via
/// `PanelOptions::with_sill`; kept as a named constant-record for docs parity.
pub const SILL_FRAME_OPTION: PanelOptions = PanelOptions {
    leaf_count: 1,
    with_sill: true,
    with_lintel: false,
};

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
        let mesh = build_opening_panel(0.9, 2.1, 0.2, PanelOptions::default());
        let (mn, mx) = bbox(&mesh);
        assert!((mx[0] - mn[0] - 0.9).abs() < 1e-6, "width");
        assert!((mx[1] - mn[1] - 0.2).abs() < 1e-6, "depth");
        assert!((mx[2] - mn[2] - 2.1).abs() < 1e-6, "height");
        assert!((mn[2]).abs() < 1e-6, "sits on the opening base");
    }

    #[test]
    fn double_leaf_split_and_mullion() {
        let single = build_opening_panel(1.0, 2.1, 0.2, PanelOptions::default());
        let double = build_opening_panel(
            1.0,
            2.1,
            0.2,
            PanelOptions {
                leaf_count: 2,
                ..PanelOptions::default()
            },
        );
        let (mn, mx) = bbox(&double);
        // Same opening coverage as the single leaf panel.
        assert!((mx[0] - mn[0] - 1.0).abs() < 1e-6, "{}..{}", mx[0], mn[0]);
        assert!((mx[2] - mn[2] - 2.1).abs() < 1e-6);
        // More triangles: the extra leaf + mullion add geometry.
        assert!(double.faces.len() > single.faces.len());
        // leaf_count clamped: 0 behaves as 1, 7 clamps to 4.
        let zero = build_opening_panel(
            1.0,
            2.1,
            0.2,
            PanelOptions {
                leaf_count: 0,
                ..PanelOptions::default()
            },
        );
        assert_eq!(bbox(&zero).0, bbox(&single).0);
    }

    #[test]
    fn sill_protrudes_down_and_lintel_spans_above() {
        let plain = build_opening_panel(1.2, 1.5, 0.3, PanelOptions::default());
        let with_sill = build_opening_panel(
            1.2,
            1.5,
            0.3,
            PanelOptions {
                with_sill: true,
                ..PanelOptions::default()
            },
        );
        // Sill drops below z=0 (the plain panel sits on it).
        assert!(
            with_sill_height(&with_sill) < -0.009,
            "{}",
            with_sill_height(&with_sill)
        );

        // The lintel lifts the head above the opening's top by ≥0.08 m.
        let with_lintel = build_opening_panel(
            1.2,
            1.5,
            0.3,
            PanelOptions {
                with_lintel: true,
                ..PanelOptions::default()
            },
        );
        let (mn, mx) = bbox(&with_lintel);
        assert!(mx[2] - 1.5 >= 0.08, "lintel above head: {}", mx[2]);
        assert!(mn[2] < -0.0 || mx[2] > 1.5);
    }

    fn with_sill_height(mesh: &MeshGeometry) -> f64 {
        let (mn, _) = bbox(mesh);
        mn[2]
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
        let base = build_opening_panel(1.0, 2.0, 0.2, PanelOptions::default());
        let placed = place_panel(&base, [5.0, 5.0, 3.0], [0.0, 1.0]);
        let (mn, mx) = bbox(&placed);
        assert!(
            (mx[0] - mn[0] - 0.2).abs() < 1e-6,
            "depth along X after 90°"
        );
        assert!(
            (mx[1] - mn[1] - 1.0).abs() < 1e-6,
            "width along Y after 90°"
        );
        assert!((mn[2] - 3.0).abs() < 1e-6);
    }
}
