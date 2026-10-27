use omnim_core as c;
use c::{FaceIndices, MeshGeometry, Point3D, Vector3D};
use std::f64::consts::PI;
use std::ops::Bound;

use truck_base::cgmath64::{Point3 as TPoint3, Vector3 as TVector3};
use truck_modeling::builder;
use truck_modeling::topology::{Shell, Solid, Wire};
use truck_modeling::{InnerSpace, ParametricSurface};

fn to_tp(p: &Point3D) -> TPoint3 {
    TPoint3::new(p.x, p.y, p.z)
}

fn to_tv(v: &Vector3D) -> TVector3 {
    TVector3::new(v.x, v.y, v.z)
}

fn vec_diff(a: &Point3D, b: &Point3D) -> Vector3D {
    Vector3D { x: a.x - b.x, y: a.y - b.y, z: a.z - b.z }
}

/// Default tessellation tolerance (LOD).
pub const DEFAULT_TOLERANCE: f64 = 0.01;

/// Convert Bound<f64> tuple to (f64, f64) min/max pair.
fn bound_to_range(range: (Bound<f64>, Bound<f64>)) -> (f64, f64) {
    let min = match range.0 {
        Bound::Included(v) | Bound::Excluded(v) => v,
        Bound::Unbounded => 0.0,
    };
    let max = match range.1 {
        Bound::Included(v) | Bound::Excluded(v) => v,
        Bound::Unbounded => 1.0,
    };
    (min, max)
}

/// Build a truck Wire from a sequence of 2D/3D profile points (closed polygon).
///
/// Points are placed in 3D space. Edges connect consecutive points
/// with lines, and the last point connects back to the first.
fn profile_to_wire(points: &[Point3D]) -> Option<Wire> {
    if points.len() < 3 {
        return None;
    }
    let verts: Vec<_> = points.iter()
        .map(|p| builder::vertex(to_tp(p)))
        .collect();
    let mut edges = Vec::with_capacity(verts.len());
    for i in 0..verts.len() {
        let j = (i + 1) % verts.len();
        edges.push(builder::line(&verts[i], &verts[j]));
    }
    Some(edges.into())
}

/// Convert a truck Solid to our MeshGeometry by tessellating each face.
fn solid_to_mesh(solid: &Solid, tolerance: f64) -> MeshGeometry {
    let tol = tolerance;
    let mut all_verts: Vec<Point3D> = Vec::new();
    let mut all_normals: Vec<Vector3D> = Vec::new();
    let mut all_faces: Vec<FaceIndices> = Vec::new();

    for face in solid.face_iter() {
        let surface = face.oriented_surface();
        let (u_range, v_range) = surface.parameter_range();
        let range = (bound_to_range(u_range), bound_to_range(v_range));
        let mesh = truck_polymesh::StructuredMesh::from_surface(&surface, range, tol);
        let poly = mesh.destruct();

        let base = all_verts.len() as i32;

        let positions = poly.positions();
        let normals = poly.normals();

        for pos in positions {
            all_verts.push(Point3D { x: pos.x, y: pos.y, z: pos.z });
        }
        for norm in normals {
            all_normals.push(Vector3D { x: norm.x, y: norm.y, z: norm.z });
        }

        let faces = poly.faces();
        for face_idx in faces.face_iter() {
            let indices: Vec<i32> = face_idx.iter().map(|i| base + i.pos as i32).collect();
            if indices.len() == 3 {
                all_faces.push(FaceIndices {
                    vertex_indices: indices,
                });
            } else if indices.len() >= 3 {
                for i in 1..indices.len() - 1 {
                    all_faces.push(FaceIndices {
                        vertex_indices: vec![indices[0], indices[i], indices[i + 1]],
                    });
                }
            }
        }
    }

    MeshGeometry {
        vertices: all_verts,
        normals: all_normals,
        uvs: Vec::new(),
        faces: all_faces,
    }
}

/// Extract a Shell from a Solid and mesh it.
fn shell_to_mesh(shell: &Shell, tolerance: f64) -> MeshGeometry {
    let tol = tolerance;
    let mut all_verts: Vec<Point3D> = Vec::new();
    let mut all_normals: Vec<Vector3D> = Vec::new();
    let mut all_faces: Vec<FaceIndices> = Vec::new();

    for face in shell.face_iter() {
        let surface = face.oriented_surface();
        let (u_range, v_range) = surface.parameter_range();
        let range = (bound_to_range(u_range), bound_to_range(v_range));
        let mesh = truck_polymesh::StructuredMesh::from_surface(&surface, range, tol);
        let poly = mesh.destruct();

        let base = all_verts.len() as i32;

        let positions = poly.positions();
        let normals = poly.normals();

        for pos in positions {
            all_verts.push(Point3D { x: pos.x, y: pos.y, z: pos.z });
        }
        for norm in normals {
            all_normals.push(Vector3D { x: norm.x, y: norm.y, z: norm.z });
        }

        let faces = poly.faces();
        for face_idx in faces.face_iter() {
            let indices: Vec<i32> = face_idx.iter().map(|i| base + i.pos as i32).collect();
            if indices.len() == 3 {
                all_faces.push(FaceIndices {
                    vertex_indices: indices,
                });
            } else if indices.len() >= 3 {
                for i in 1..indices.len() - 1 {
                    all_faces.push(FaceIndices {
                        vertex_indices: vec![indices[0], indices[i], indices[i + 1]],
                    });
                }
            }
        }
    }

    MeshGeometry {
        vertices: all_verts,
        normals: all_normals,
        uvs: Vec::new(),
        faces: all_faces,
    }
}

/// Extrude a 2D profile along a direction vector to create a solid mesh.
///
/// `profile` - closed polygon points (at least 3)
/// `depth` - extrusion distance along direction
/// `direction` - extrusion direction (will be normalized)
/// `tolerance` - tessellation tolerance (lower = finer mesh)
pub fn extrude_brep(
    profile: &[Point3D],
    depth: f64,
    direction: Vector3D,
    tolerance: f64,
) -> Option<MeshGeometry> {
    let wire = profile_to_wire(profile)?;
    let face = builder::try_attach_plane(&[wire]).ok()?;
    let vec = to_tv(&direction).normalize() * depth;
    let solid: Solid = builder::tsweep(&face, vec);
    Some(solid_to_mesh(&solid, tolerance))
}

/// Revolve a 2D profile around an axis to create a solid mesh.
///
/// `profile` - closed polygon points (at least 3)
/// `angle` - revolution angle in radians (0 = full 2π)
/// `axis_origin` - point on the revolution axis
/// `axis_direction` - direction of the revolution axis (will be normalized)
/// `tolerance` - tessellation tolerance (lower = finer mesh)
pub fn revolve_brep(
    profile: &[Point3D],
    angle: f64,
    axis_origin: Point3D,
    axis_direction: Vector3D,
    tolerance: f64,
) -> Option<MeshGeometry> {
    let wire = profile_to_wire(profile)?;
    let face = builder::try_attach_plane(&[wire]).ok()?;
    let axis = to_tv(&axis_direction).normalize();
    let origin = to_tp(&axis_origin);
    let total_angle = if angle.abs() < 1e-10 { 2.0 * PI } else { angle.abs() };
    let rad = truck_base::cgmath64::Rad(total_angle);
    let solid: Solid = builder::rsweep(&face, origin, axis, rad);
    Some(solid_to_mesh(&solid, tolerance))
}

/// Sweep a 2D profile along a 3D path to create a solid mesh.
///
/// `profile` - closed polygon points (at least 3), placed at the start of the path
/// `path` - ordered path points (at least 2)
/// `tolerance` - tessellation tolerance (lower = finer mesh)
pub fn sweep_brep(
    profile: &[Point3D],
    path: &[Point3D],
    tolerance: f64,
) -> Option<MeshGeometry> {
    if path.len() < 2 {
        return None;
    }
    let profile_wire = profile_to_wire(profile)?;
    let face = builder::try_attach_plane(&[profile_wire]).ok()?;

    // tsweep the face along the first segment → Solid
    let first_dir = to_tv(&vec_diff(&path[1], &path[0]));
    let solid: Solid = builder::tsweep(&face, first_dir);

    // For additional segments, extract boundary and sweep further
    if path.len() <= 2 {
        let boundaries = solid.into_boundaries();
        if let Some(shell) = boundaries.into_iter().next() {
            return Some(shell_to_mesh(&shell, tolerance));
        }
        return None;
    }

    let shells = solid.into_boundaries();
    let mut current_shell = shells.into_iter().next()?;

    for i in 2..path.len() {
        let dir = to_tv(&vec_diff(&path[i], &path[i - 1]));
        let boundaries = current_shell.extract_boundaries();
        if let Some(end_wire) = boundaries.into_iter().find(|w| w.is_closed()) {
            let mut segment_shell: Shell = builder::tsweep(&end_wire, dir);
            current_shell.append(&mut segment_shell);
        }
    }

    Some(shell_to_mesh(&current_shell, tolerance))
}

/// Loft between two closed profiles to create a solid mesh.
///
/// `profile0` - first profile (closed polygon, at least 3 points)
/// `profile1` - second profile (closed polygon, same number of edges as profile0)
/// `tolerance` - tessellation tolerance (lower = finer mesh)
///
/// Note: Both wires must have the same number of edges (segments).
pub fn loft_brep(
    profile0: &[Point3D],
    profile1: &[Point3D],
    tolerance: f64,
) -> Option<MeshGeometry> {
    let wire0 = profile_to_wire(profile0)?;
    let wire1 = profile_to_wire(profile1)?;

    let side_shell = builder::try_wire_homotopy(&wire0, &wire1).ok()?;

    // Close the loft by adding end caps
    let face0 = builder::try_attach_plane(&[wire0]).ok()?;
    let face1 = builder::try_attach_plane(&[wire1]).ok()?;

    let mut shell = side_shell;
    shell.push(face0);
    shell.push(face1);

    Some(shell_to_mesh(&shell, tolerance))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect_profile() -> Vec<Point3D> {
        vec![
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Point3D { x: 2.0, y: 0.0, z: 0.0 },
            Point3D { x: 2.0, y: 1.0, z: 0.0 },
            Point3D { x: 0.0, y: 1.0, z: 0.0 },
        ]
    }

    fn triangle_profile() -> Vec<Point3D> {
        vec![
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Point3D { x: 1.0, y: 0.0, z: 0.0 },
            Point3D { x: 0.5, y: 0.866, z: 0.0 },
        ]
    }

    #[test]
    fn test_extrude_brep_rectangle() {
        let profile = rect_profile();
        let mesh = extrude_brep(&profile, 3.0, Vector3D { x: 0.0, y: 0.0, z: 1.0 }, DEFAULT_TOLERANCE);
        assert!(mesh.is_some());
        let m = mesh.unwrap();
        assert!(m.vertices.len() >= 8);
        assert!(m.faces.len() >= 12);
        assert_eq!(m.normals.len(), m.vertices.len());
    }

    #[test]
    fn test_extrude_brep_triangle() {
        let profile = triangle_profile();
        let mesh = extrude_brep(&profile, 2.0, Vector3D { x: 0.0, y: 0.0, z: 1.0 }, DEFAULT_TOLERANCE);
        assert!(mesh.is_some());
        let m = mesh.unwrap();
        assert!(m.vertices.len() >= 6);
        assert!(m.faces.len() >= 8);
    }

    #[test]
    fn test_revolve_brep_rectangle() {
        let profile = rect_profile();
        let mesh = revolve_brep(
            &profile,
            PI,
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Vector3D { x: 0.0, y: 0.0, z: 1.0 },
            DEFAULT_TOLERANCE,
        );
        assert!(mesh.is_some());
        let m = mesh.unwrap();
        assert!(m.vertices.len() > 0);
        assert!(m.faces.len() > 0);
    }

    #[test]
    fn test_revolve_brep_full_circle() {
        let profile = vec![
            Point3D { x: 3.0, y: 0.0, z: 0.0 },
            Point3D { x: 4.0, y: 0.0, z: 0.0 },
            Point3D { x: 4.0, y: 1.0, z: 0.0 },
            Point3D { x: 3.0, y: 1.0, z: 0.0 },
        ];
        let mesh = revolve_brep(
            &profile,
            0.0,
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Vector3D { x: 0.0, y: 0.0, z: 1.0 },
            DEFAULT_TOLERANCE,
        );
        assert!(mesh.is_some());
        let m = mesh.unwrap();
        assert!(m.vertices.len() > 50);
    }

    #[test]
    fn test_sweep_brep_straight() {
        let profile = rect_profile();
        let path = vec![
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Point3D { x: 0.0, y: 0.0, z: 5.0 },
        ];
        let mesh = sweep_brep(&profile, &path, DEFAULT_TOLERANCE);
        assert!(mesh.is_some());
        let m = mesh.unwrap();
        assert!(m.vertices.len() >= 8);
    }

    #[test]
    fn test_loft_brep_same_shape() {
        let profile0 = rect_profile();
        let profile1 = vec![
            Point3D { x: 0.0, y: 0.0, z: 3.0 },
            Point3D { x: 2.0, y: 0.0, z: 3.0 },
            Point3D { x: 2.0, y: 1.0, z: 3.0 },
            Point3D { x: 0.0, y: 1.0, z: 3.0 },
        ];
        let mesh = loft_brep(&profile0, &profile1, DEFAULT_TOLERANCE);
        assert!(mesh.is_some());
        let m = mesh.unwrap();
        assert!(m.vertices.len() >= 8);
        assert!(m.faces.len() >= 10);
    }

    #[test]
    fn test_profile_too_small_returns_none() {
        let profile = vec![
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Point3D { x: 1.0, y: 0.0, z: 0.0 },
        ];
        assert!(extrude_brep(&profile, 1.0, Vector3D { x: 0.0, y: 0.0, z: 1.0 }, DEFAULT_TOLERANCE).is_none());
    }

    #[test]
    fn test_sweep_insufficient_path_returns_none() {
        let profile = rect_profile();
        let path = vec![Point3D { x: 0.0, y: 0.0, z: 0.0 }];
        assert!(sweep_brep(&profile, &path, DEFAULT_TOLERANCE).is_none());
    }
}
