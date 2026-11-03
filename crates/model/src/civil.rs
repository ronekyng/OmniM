//! Civil kernels (Phase 8-2, ROADMAP): terrain TIN fitting, ground-elevation
//! queries and grid-based cut/fill earthwork balancing.
//!
//! [`TinModel`] triangulates survey points with Bowyer–Watson on the XY plane
//! and carries the elevations on the same vertices — the TIN **is** the
//! terrain model (DOMIANS… see docs/specification/DOMAINS.zh-CN.md §3, terrain
//! row). The earthwork grid compares cell-centre terrain elevations against a
//! design level and integrates cut (terrain above design) and fill (excavation
//! below design) as prisms; outside-hull cells are counted, not guessed.

use c::MeshGeometry;
use omnim_core as c;

/// A triangulated irregular network over the XY plane.
#[derive(Clone, Debug, Default)]
pub struct TinModel {
    /// Distinct survey vertices in metres.
    pub points: Vec<[f64; 3]>,
    /// Triangles as indices into `points` (counter-clockwise in XY).
    pub triangles: Vec<[usize; 3]>,
    /// Points rejected as duplicates (same XY as an earlier point).
    pub dropped_duplicate_count: usize,
}

impl TinModel {
    /// Triangulate survey points (Bowyer–Watson).
    ///
    /// Errors: fewer than 3 points, non-finite coordinates, fewer than 3
    /// distinct XY points (after de-duplication), or all points collinear.
    pub fn from_survey_points(points: &[[f64; 3]]) -> Result<Self, String> {
        if points.len() < 3 {
            return Err(format!(
                "terrain needs at least 3 survey points, got {}",
                points.len()
            ));
        }
        for p in points {
            if !p.iter().all(|v| v.is_finite()) {
                return Err("terrain points must be finite".into());
            }
        }
        // De-duplicate by XY: the first elevation wins (survey order is
        // authoritative).
        let mut unique: Vec<[f64; 3]> = Vec::with_capacity(points.len());
        let mut duplicates = 0usize;
        'outer: for p in points {
            for existing in &unique {
                if existing[0] == p[0] && existing[1] == p[1] {
                    duplicates += 1;
                    continue 'outer;
                }
            }
            unique.push(*p);
        }
        if unique.len() < 3 {
            return Err(format!(
                "terrain needs at least 3 distinct XY points, got {} ({} duplicates dropped)",
                unique.len(),
                duplicates
            ));
        }

        // Bounding super-triangle: 10× the extent is far beyond any survey
        // point's circumcircle, so the degenerate-circle check stays exact.
        let (minx, maxx, miny, maxy) = extents(&unique);
        let dx = maxx - minx;
        let dy = maxy - miny;
        let cx = (minx + maxx) / 2.0;
        let cy = (miny + maxy) / 2.0;
        let d = dx.max(dy).max(1.0) * 10.0;
        // 0..n: survey vertices; n..n+3: the super triangle.
        let mut verts: Vec<(f64, f64)> = unique.iter().map(|p| (p[0], p[1])).collect();
        verts.push((cx, cy + d * 3.0));
        verts.push((cx - d * 3.0, cy - d));
        verts.push((cx + d * 3.0, cy - d));
        let n = unique.len();

        let mut triangles: Vec<[usize; 3]> = vec![[n, n + 1, n + 2]];
        for (i, &p) in verts.iter().enumerate().take(n) {
            // The cavity: triangles whose circumcircle strictly contains p.
            let mut cavity: Vec<[usize; 3]> = Vec::new();
            triangles.retain(|t| {
                if in_circumcircle(&verts, *t, p) {
                    cavity.push(*t);
                    false
                } else {
                    true
                }
            });
            // Boundary edges of the cavity = edges appearing exactly once.
            let mut edges: Vec<[usize; 2]> = Vec::new();
            for t in &cavity {
                for e in [[t[0], t[1]], [t[1], t[2]], [t[2], t[0]]] {
                    let key = if e[0] < e[1] { e } else { [e[1], e[0]] };
                    if let Some(pos) = edges.iter().position(|x| *x == key) {
                        edges.remove(pos);
                    } else {
                        edges.push(key);
                    }
                }
            }
            for e in edges {
                triangles.push([e[0], e[1], i]);
            }
        }

        // Drop everything still touching the super-triangle.
        triangles.retain(|t| t[0] < n && t[1] < n && t[2] < n);
        if triangles.is_empty() {
            return Err("terrain points are collinear (no triangles)".into());
        }
        Ok(Self {
            points: unique,
            triangles,
            dropped_duplicate_count: duplicates,
        })
    }

    /// Triangle count.
    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    /// Raw surface mesh: one pair of wound triangles per TIN triangle (the
    /// terrain is a surface, the viewer renders both faces).
    pub fn to_mesh(&self) -> MeshGeometry {
        let vertices = self
            .points
            .iter()
            .map(|p| c::Point3D {
                x: p[0],
                y: p[1],
                z: p[2],
            })
            .collect();
        let mut faces = Vec::with_capacity(self.triangles.len() * 2);
        for t in &self.triangles {
            faces.push(c::FaceIndices {
                vertex_indices: vec![t[0] as i32, t[1] as i32, t[2] as i32],
            });
            faces.push(c::FaceIndices {
                vertex_indices: vec![t[0] as i32, t[2] as i32, t[1] as i32],
            });
        }
        MeshGeometry {
            vertices,
            normals: Vec::new(),
            uvs: Vec::new(),
            faces,
        }
    }

    /// Barycentric elevation at (x, y); `None` outside the convex hull or on
    /// a degenerate triangle.
    pub fn elevation_at(&self, x: f64, y: f64) -> Option<f64> {
        self.locate(x, y).map(|(t, w0, w1)| {
            let a = &self.points[t[0]];
            let b = &self.points[t[1]];
            let cc = &self.points[t[2]];
            w0 * a[2] + w1 * b[2] + (1.0 - w0 - w1) * cc[2]
        })
    }

    /// Does (x, y) lie inside the hull?
    pub fn contains(&self, x: f64, y: f64) -> bool {
        self.elevation_at(x, y).is_some()
    }

    /// Grid cut/fill against a design level; see [`EarthworkBalance`].
    pub fn earthwork(&self, design_z: f64, cell: f64) -> EarthworkBalance {
        let (minx, maxx, miny, maxy) = extents(&self.points);
        let area = cell * cell;
        let cols = ((maxx - minx) / cell).ceil().max(1.0) as usize;
        let rows = ((maxy - miny) / cell).max(1.0) as usize;
        let mut cut = 0.0;
        let mut fill = 0.0;
        let mut sampled = 0usize;
        let mut outside = 0usize;
        for r in 0..rows {
            for c in 0..cols {
                let x = minx + (c as f64 + 0.5) * cell;
                let y = miny + (r as f64 + 0.5) * cell;
                match self.elevation_at(x, y) {
                    Some(h) => {
                        sampled += 1;
                        let dz = h - design_z;
                        if dz > 0.0 {
                            cut += dz * area;
                        } else {
                            fill += (-dz) * area;
                        }
                    }
                    None => outside += 1,
                }
            }
        }
        let net = cut - fill;
        EarthworkBalance {
            design_z,
            cell_size: cell,
            grid_cols: cols,
            grid_rows: rows,
            sampled_cells: sampled,
            outside_cells: outside,
            cut_volume_m3: round3(cut),
            fill_volume_m3: round3(fill),
            net_volume_m3: round3(net),
        }
    }

    /// Vertex/triangle counts and bounding box.
    pub fn stats(&self) -> TerrainStats {
        let (minx, maxx, miny, maxy) = extents(&self.points);
        let mut zmin = f64::INFINITY;
        let mut zmax = f64::NEG_INFINITY;
        for p in &self.points {
            zmin = zmin.min(p[2]);
            zmax = zmax.max(p[2]);
        }
        TerrainStats {
            vertex_count: self.points.len(),
            triangle_count: self.triangles.len(),
            min: [minx, miny, zmin],
            max: [maxx, maxy, zmax],
        }
    }

    fn locate(&self, x: f64, y: f64) -> Option<([usize; 3], f64, f64)> {
        for t in &self.triangles {
            let a = &self.points[t[0]];
            let b = &self.points[t[1]];
            let cc = &self.points[t[2]];
            let denom = (b[1] - cc[1]) * (a[0] - cc[0]) + (cc[0] - b[0]) * (a[1] - cc[1]);
            if denom.abs() < 1e-15 {
                continue;
            }
            let w0 = ((b[1] - cc[1]) * (x - cc[0]) + (cc[0] - b[0]) * (y - cc[1])) / denom;
            let w1 = ((cc[1] - a[1]) * (x - cc[0]) + (a[0] - cc[0]) * (y - cc[1])) / denom;
            // Inside with an epsilon so hull-edge queries resolve.
            if w0 > -1e-9 && w1 > -1e-9 && w0 + w1 < 1.0 + 1e-9 {
                return Some((*t, w0, w1));
            }
        }
        None
    }
}

/// Cut/fill balance (Phase 8-2, `docs/specification/DOMAINS` earthwork row).
#[derive(Debug, Clone, serde::Serialize)]
pub struct EarthworkBalance {
    pub design_z: f64,
    pub cell_size: f64,
    pub grid_cols: usize,
    pub grid_rows: usize,
    /// Cell centres inside the hull.
    pub sampled_cells: usize,
    /// Cell centres outside the hull (skipped, not extrapolated).
    pub outside_cells: usize,
    /// Material to remove (terrain above design), m³.
    pub cut_volume_m3: f64,
    /// Material to add (terrain below design), m³.
    pub fill_volume_m3: f64,
    /// cut − fill, signed, m³.
    pub net_volume_m3: f64,
}

/// Summary facts for a fitted terrain (response metadata).
#[derive(Debug, Clone, serde::Serialize)]
pub struct TerrainStats {
    pub vertex_count: usize,
    pub triangle_count: usize,
    /// Bounding box min [x, y, z].
    pub min: [f64; 3],
    /// Bounding box max [x, y, z].
    pub max: [f64; 3],
}

fn round3(v: f64) -> f64 {
    (v * 1000.0).round() / 1000.0
}

fn extents(points: &[[f64; 3]]) -> (f64, f64, f64, f64) {
    let mut minx = f64::INFINITY;
    let mut maxx = f64::NEG_INFINITY;
    let mut miny = f64::INFINITY;
    let mut maxy = f64::NEG_INFINITY;
    for p in points {
        minx = minx.min(p[0]);
        maxx = maxx.max(p[0]);
        miny = miny.min(p[1]);
        maxy = maxy.max(p[1]);
    }
    (minx, maxx, miny, maxy)
}

/// Strict inside test of the triangle's circumcircle (works with either
/// winding: the determinant sign is normalised by the triangle orientation).
fn in_circumcircle(verts: &[(f64, f64)], t: [usize; 3], p: (f64, f64)) -> bool {
    let (ax, ay) = verts[t[0]];
    let (bx, by) = verts[t[1]];
    let (cx, cy) = verts[t[2]];
    let (px, py) = p;
    let (adx, ady) = (ax - px, ay - py);
    let (bdx, bdy) = (bx - px, by - py);
    let (cdx, cdy) = (cx - px, cy - py);
    let az = adx * adx + ady * ady;
    let bz = bdx * bdx + bdy * bdy;
    let cz = cdx * cdx + cdy * cdy;
    let det =
        adx * (bdy * cz - bz * cdy) - ady * (bdx * cz - bz * cdx) + az * (bdx * cdy - bdy * cdx);
    // Strictly inside for counter-clockwise triangles (det > 0); the sign
    // flips with the winding. Degenerate (collinear) triangles never enclose.
    let orient = (bx - ax) * (cy - ay) - (by - ay) * (cx - ax);
    let inside = det > 1e-15 * (az + bz + cz).max(1.0);
    if orient > 0.0 {
        inside
    } else {
        !inside
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_grid_is_two_triangles_and_interpolates_z() {
        // Unit square corners at z=1,2,3,4: the diagonal splits into two
        // triangles; a planar surface yields exact barycentric lookups.
        let pts = [
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 2.0],
            [1.0, 1.0, 3.0],
            [0.0, 1.0, 2.0],
        ];
        let tin = TinModel::from_survey_points(&pts).unwrap();
        assert_eq!(tin.triangle_count(), 2);
        assert_eq!(tin.stats().vertex_count, 4);
        // The corners are coplanar (z = 1 + x + y): interior points resolve
        // exactly on the plane.
        let h = tin.elevation_at(0.5, 0.5).unwrap();
        assert!((h - 2.0).abs() < 1e-9, "centre elevation: {}", h);
        assert!(
            (tin.elevation_at(0.25, 0.75).unwrap() - 2.0).abs() < 1e-9,
            "off-diagonal: {}",
            tin.elevation_at(0.25, 0.75).unwrap()
        );
        assert!(!tin.contains(2.0, 2.0));
        assert!(tin.contains(0.001, 0.001));
    }

    #[test]
    fn rejects_degenerate_surveys() {
        assert!(TinModel::from_survey_points(&[[0.0, 0.0, 0.0]]).is_err());
        // Two duplicate XY + one other point → fewer than 3 distinct.
        let err =
            TinModel::from_survey_points(&[[0.0, 0.0, 0.0], [0.0, 0.0, 5.0], [1.0, 0.0, 0.0]])
                .unwrap_err();
        assert!(err.contains("distinct"), "{}", err);
        assert!(
            TinModel::from_survey_points(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]])
                .is_err()
        );
    }

    #[test]
    fn duplicate_xy_drops_the_replay() {
        let pts = [
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 2.0],
            [1.0, 1.0, 3.0],
            [0.0, 0.0, 9.0], // replay
            [0.0, 1.0, 4.0],
        ];
        let tin = TinModel::from_survey_points(&pts).unwrap();
        assert_eq!(tin.dropped_duplicate_count, 1);
        assert_eq!(tin.stats().vertex_count, 4);
        // The first elevation wins.
        let h = tin.elevation_at(0.0, 0.0).unwrap();
        assert!((h - 1.0).abs() < 1e-9, "{}", h);
    }

    #[test]
    fn earthwork_counts_cut_fill_and_outside() {
        // Terrain: two plateaus — a plane over the unit square z = 1 + x.
        let pts = [
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 2.0],
            [1.0, 1.0, 2.0],
            [0.0, 1.0, 1.0],
        ];
        let tin = TinModel::from_survey_points(&pts).unwrap();
        let bal = tin.earthwork(1.5, 0.5);
        assert_eq!(bal.grid_cols, 2);
        assert_eq!(bal.grid_rows, 2);
        assert_eq!(bal.sampled_cells, 4, "{}", {
            let mut count = 0;
            for (x, y) in [(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)] {
                if tin.contains(x, y) {
                    count += 1;
                }
            }
            format!("{count}")
        });
        assert_eq!(bal.outside_cells, 0);
        // Corner cells: z = 1.25 (cut 0) and z = 1.75 (cut 0.25 × 0.25 m²).
        // Cut on the +x cells, fill on the −x cells: 2×0.0625 each.
        assert!(
            (bal.cut_volume_m3 - 0.125).abs() < 1e-6,
            "{}",
            bal.cut_volume_m3
        );
        assert!(
            (bal.fill_volume_m3 - 0.125).abs() < 1e-6,
            "{}",
            bal.fill_volume_m3
        );
        assert!(bal.net_volume_m3.abs() < 1e-6, "{}", bal.net_volume_m3);
        // Design below the whole terrain → all cut, no fill.
        let bal = tin.earthwork(0.0, 1.0);
        assert!(bal.fill_volume_m3 == 0.0 && bal.cut_volume_m3 > 0.0);
    }
}
