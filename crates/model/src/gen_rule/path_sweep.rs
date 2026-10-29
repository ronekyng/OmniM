//! PathSweep — the class-agnostic "constant-section stack swept along a path"
//! primitive (DATA_MODEL §1 governance 3: the primitive lives *below*
//! classification; LayerExtrusion and future Linear families both sit on it).
//!
//! Kernel: the existing, verified LayerExtrusion mesh builders (mitered bends,
//! `base_z` elevation, closed-boundary validation). This module is a thin,
//! parameter-object-shaped façade so that feature trees
//! (`FeatureTemplateStep::LayeredPathSweep`) and the genrule families share one
//! implementation — the geometry-path convergence goal (ROADMAP Wave 3).
//!
//! Orientation contract:
//! - `Vertical` section (walls): the section rectangle stands in the vertical
//!   plane spanned by the path direction (X) and world +Z; layers stack along
//!   the in-plane horizontal normal. Produced by `build_layered_path_elevated`.
//! - `Horizontal` section (slab/roof/floor footprints): closed boundary in the
//!   ground plane, layers stack along +Z. Produced by
//!   `build_layered_slab_path_elevated`.
//! - Roads/linear sections are **not** served here yet (Wave 3 keeps
//!   `build_road_alignment`); extending this façade to Linear is explicitly
//!   out of scope until the Linear family converges.

use omnim_core::{MeshGeometry, Point2D, Point3D};
use serde::{Deserialize, Serialize};

use super::layer_extrusion::{self, LayerDef};

fn to_point2d(points: &[[f64; 3]]) -> Vec<Point2D> {
    points
        .iter()
        .map(|c| Point2D { x: c[0], y: c[1] })
        .collect()
}

/// Section orientation of the swept stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectionOrientation {
    /// Section stands upright (walls): profile = length × height, layers
    /// offset horizontally.
    Vertical,
    /// Section lies flat (slab/roof/floor): closed footprint, layers stack
    /// along +Z.
    Horizontal,
}

/// Input for a layered path sweep.
#[derive(Debug, Clone)]
pub struct PathSweepInput {
    /// Picked path / boundary in world metres (carries its own z per point).
    pub points: Vec<[f64; 3]>,
    pub closed: bool,
    /// Layer stack (thickness in **mm**, matching the LayerEditor contract).
    pub layers: Vec<LayerDef>,
    /// Wall/roof extrusion height in metres (Vertical only).
    pub height: f64,
    pub orientation: SectionOrientation,
}

impl PathSweepInput {
    /// Base elevation: the first point's z (B1 convention — picked z is kept;
    /// differing z along the path degrade to the first point's z, planar
    /// extrusion only).
    pub fn base_z(&self) -> f64 {
        self.points.first().map(|p| p[2]).unwrap_or(0.0)
    }
}

/// Run the sweep. Validation errors (too few points, zero-length segments,
/// degenerate/self-intersecting closed boundaries) return `Err` with the
/// kernel's reason — callers map that to a 400.
pub fn generate(input: &PathSweepInput) -> Result<MeshGeometry, String> {
    let pts = to_point2d(&input.points);
    super::path_geom::validate_path(&pts, input.closed, if input.closed { 3 } else { 2 })?;
    let merged = super::path_geom::merge_collinear(&super::path_geom::normalize_closed(&pts));
    match input.orientation {
        SectionOrientation::Vertical => {
            if merged.len() < 2 {
                return Err("path requires at least 2 distinct points".into());
            }
            Ok(layer_extrusion::build_layered_path_elevated(
                &merged,
                false,
                input.height,
                input.base_z(),
                &input.layers,
            ))
        }
        SectionOrientation::Horizontal => Ok(layer_extrusion::build_layered_slab_path_elevated(
            &merged,
            &input.layers,
            input.base_z(),
        )),
    }
}

/// Convert world points to `Point3D` (used by callers that need the 3D form,
/// e.g. LinearComponent engines).
pub fn to_point3d(points: &[[f64; 3]]) -> Vec<Point3D> {
    points
        .iter()
        .map(|c| Point3D {
            x: c[0],
            y: c[1],
            z: c[2],
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layers() -> Vec<LayerDef> {
        vec![
            LayerDef {
                name: "core".into(),
                thickness: 200.0,
                material_id: Some("C30".into()),
                function: "structural".into(),
            },
            LayerDef {
                name: "finish".into(),
                thickness: 20.0,
                material_id: Some("Mortar".into()),
                function: "finish".into(),
            },
        ]
    }

    #[test]
    fn vertical_wall_matches_legacy_kernel_exactly() {
        // Same input to both entry points must produce identical meshes —
        // this is the geometry-path convergence invariant.
        let path = vec![[0.0, 0.0, 0.5], [4.0, 0.0, 0.5], [4.0, 3.0, 0.5]];
        let legacy = layer_extrusion::build_layered_path_elevated(
            &super::super::path_geom::merge_collinear(&to_point2d(&path)),
            false,
            3.0,
            0.5,
            &layers(),
        );
        let converged = generate(&PathSweepInput {
            points: path,
            closed: false,
            layers: layers(),
            height: 3.0,
            orientation: SectionOrientation::Vertical,
        })
        .unwrap();
        assert_eq!(legacy.vertices, converged.vertices);
        assert_eq!(legacy.faces, converged.faces);
    }

    #[test]
    fn horizontal_slab_matches_legacy_kernel_exactly() {
        let boundary = vec![
            [0.0, 0.0, 2.0],
            [5.0, 0.0, 2.0],
            [5.0, 4.0, 2.0],
            [0.0, 4.0, 2.0],
        ];
        let merged = super::super::path_geom::merge_collinear(&to_point2d(&boundary));
        let legacy = layer_extrusion::build_layered_slab_path_elevated(&merged, &layers(), 2.0);
        let converged = generate(&PathSweepInput {
            points: boundary,
            closed: true,
            layers: layers(),
            height: 0.0,
            orientation: SectionOrientation::Horizontal,
        })
        .unwrap();
        assert_eq!(legacy.vertices, converged.vertices);
    }

    #[test]
    fn validation_errors_surface() {
        let err = generate(&PathSweepInput {
            points: vec![[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            closed: false,
            layers: layers(),
            height: 3.0,
            orientation: SectionOrientation::Vertical,
        })
        .unwrap_err();
        assert!(err.contains("zero-length"), "unexpected: {}", err);

        let err = generate(&PathSweepInput {
            points: vec![[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
            closed: true,
            layers: layers(),
            height: 3.0,
            orientation: SectionOrientation::Horizontal,
        })
        .unwrap_err();
        assert!(err.contains("at least 3"), "unexpected: {}", err);
    }
}
