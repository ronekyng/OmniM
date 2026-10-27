use omnim_core as c;
use serde::Serialize;

use c::{BoundingBox, FaceIndices, Geometry, GeometryType, MeshGeometry, Point3D, Transform, Vector3D};
use crate::{brep, mesh_gen};

type MeshData = (Vec<Point3D>, Vec<Vector3D>, Vec<FaceIndices>);

fn dispatch(op: &c::parametric_operation::Operation) -> MeshData {
    use c::parametric_operation::Operation::*;
    match op {
        Box(b) => mesh_gen::box_mesh(b),
        Cylinder(cyl) => mesh_gen::cylinder_mesh(cyl),
        Sphere(s) => mesh_gen::sphere_mesh(s),
        Cone(cone) => mesh_gen::cone_mesh(cone),
        Extrusion(ext) => {
            let profile: Vec<Point3D> = ext.profile.iter()
                .map(|p| Point3D { x: p.x, y: p.y, z: 0.0 })
                .collect();
            let dir = ext.direction.clone().unwrap_or(Vector3D { x: 0.0, y: 0.0, z: 1.0 });
            match brep::extrude_brep(&profile, ext.depth, dir, brep::DEFAULT_TOLERANCE) {
                Some(mesh) => (mesh.vertices, mesh.normals, mesh.faces),
                None => (Vec::new(), Vec::new(), Vec::new()),
            }
        }
        Revolve(rev) => {
            let profile: Vec<Point3D> = rev.profile.iter()
                .map(|p| Point3D { x: p.x, y: p.y, z: 0.0 })
                .collect();
            let origin = rev.axis_origin.clone().unwrap_or(Point3D { x: 0.0, y: 0.0, z: 0.0 });
            let dir = rev.axis_direction.clone().unwrap_or(Vector3D { x: 0.0, y: 0.0, z: 1.0 });
            match brep::revolve_brep(&profile, rev.angle, origin, dir, brep::DEFAULT_TOLERANCE) {
                Some(mesh) => (mesh.vertices, mesh.normals, mesh.faces),
                None => (Vec::new(), Vec::new(), Vec::new()),
            }
        }
        Sweep(sweep) => {
            let profile: Vec<Point3D> = sweep.profile.iter()
                .map(|p| Point3D { x: p.x, y: p.y, z: 0.0 })
                .collect();
            let path: Vec<Point3D> = sweep.path.iter().cloned().collect();
            match brep::sweep_brep(&profile, &path, brep::DEFAULT_TOLERANCE) {
                Some(mesh) => (mesh.vertices, mesh.normals, mesh.faces),
                None => (Vec::new(), Vec::new(), Vec::new()),
            }
        }
        Loft(loft) => {
            let profile0: Vec<Point3D> = loft.profile0.iter()
                .map(|p| Point3D { x: p.x, y: p.y, z: 0.0 })
                .collect();
            let profile1: Vec<Point3D> = loft.profile1.iter()
                .map(|p| Point3D { x: p.x, y: p.y, z: 0.0 })
                .collect();
            match brep::loft_brep(&profile0, &profile1, brep::DEFAULT_TOLERANCE) {
                Some(mesh) => (mesh.vertices, mesh.normals, mesh.faces),
                None => (Vec::new(), Vec::new(), Vec::new()),
            }
        }
        Torus(torus) => mesh_gen::torus_mesh(torus),
        Dish(dish) => mesh_gen::dish_mesh(dish),
    }
}

pub fn parametric_to_mesh(pg: &c::ParametricGeometry) -> MeshGeometry {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut faces = Vec::new();
    for op in &pg.operations {
        if let Some(kind) = &op.operation {
            let (mut v, mut n, mut f) = dispatch(kind);
            let offset = vertices.len() as i32;
            for face in &mut f {
                for idx in &mut face.vertex_indices {
                    *idx += offset;
                }
            }
            vertices.append(&mut v);
            normals.append(&mut n);
            faces.append(&mut f);
        }
    }
    MeshGeometry { vertices, normals, uvs: Vec::new(), faces }
}

pub fn create_geometry(pg: c::ParametricGeometry) -> Geometry {
    let mesh = parametric_to_mesh(&pg);
    let bbox = compute_bbox(&mesh);
    Geometry {
        id: String::new(),
        r#type: GeometryType::Parametric as i32,
        representation: Some(c::geometry::Representation::Parametric(pg)),
        bbox: Some(bbox),
        level_of_detail: 0,
    }
}

pub fn geometry_to_mesh(geom: &Geometry) -> Option<MeshGeometry> {
    match &geom.representation {
        Some(c::geometry::Representation::Parametric(pg)) => Some(parametric_to_mesh(pg)),
        Some(c::geometry::Representation::Mesh(m)) => Some(m.clone()),
        _ => None,
    }
}

/// 将 MeshGeometry 包装为 Geometry
pub fn mesh_to_geometry(mesh: &MeshGeometry) -> Geometry {
    let bbox = compute_bbox(mesh);
    Geometry {
        id: uuid::Uuid::new_v4().to_string(),
        r#type: c::GeometryType::Mesh as i32,
        representation: Some(c::geometry::Representation::Mesh(mesh.clone())),
        bbox: Some(bbox),
        level_of_detail: 0,
    }
}

/// Uniformly scale a mesh geometry by a constant factor.
///
/// Catalog component feature-tree geometry is authored in millimetres while
/// the 3D scene / viewport works in metres. The render path scales catalog
/// meshes by `1/1000` so a 5000 mm wall displays as 5 m.
pub fn scale_mesh(mesh: &MeshGeometry, factor: f64) -> MeshGeometry {
    let mut out = mesh.clone();
    for v in &mut out.vertices {
        v.x *= factor;
        v.y *= factor;
        v.z *= factor;
    }
    out
}

pub fn compute_bbox(mesh: &MeshGeometry) -> BoundingBox {
    if mesh.vertices.is_empty() {
        return BoundingBox {
            min: Some(Point3D { x: 0.0, y: 0.0, z: 0.0 }),
            max: Some(Point3D { x: 0.0, y: 0.0, z: 0.0 }),
        };
    }
    let mut mn = Point3D { x: f64::INFINITY, y: f64::INFINITY, z: f64::INFINITY };
    let mut mx = Point3D { x: f64::NEG_INFINITY, y: f64::NEG_INFINITY, z: f64::NEG_INFINITY };
    for v in &mesh.vertices {
        mn.x = mn.x.min(v.x); mn.y = mn.y.min(v.y); mn.z = mn.z.min(v.z);
        mx.x = mx.x.max(v.x); mx.y = mx.y.max(v.y); mx.z = mx.z.max(v.z);
    }
    BoundingBox { min: Some(mn), max: Some(mx) }
}

pub fn merge_bbox(a: &BoundingBox, b: &BoundingBox) -> BoundingBox {
    fn p(f: f64) -> f64 { if f == 0.0 { 0.0 } else { f } }
    BoundingBox {
        min: Some(Point3D {
            x: p(a.min.as_ref().map(|m| m.x).unwrap_or(f64::INFINITY).min(b.min.as_ref().map(|m| m.x).unwrap_or(f64::INFINITY))),
            y: p(a.min.as_ref().map(|m| m.y).unwrap_or(f64::INFINITY).min(b.min.as_ref().map(|m| m.y).unwrap_or(f64::INFINITY))),
            z: p(a.min.as_ref().map(|m| m.z).unwrap_or(f64::INFINITY).min(b.min.as_ref().map(|m| m.z).unwrap_or(f64::INFINITY))),
        }),
        max: Some(Point3D {
            x: a.max.as_ref().map(|m| m.x).unwrap_or(f64::NEG_INFINITY).max(b.max.as_ref().map(|m| m.x).unwrap_or(f64::NEG_INFINITY)),
            y: a.max.as_ref().map(|m| m.y).unwrap_or(f64::NEG_INFINITY).max(b.max.as_ref().map(|m| m.y).unwrap_or(f64::NEG_INFINITY)),
            z: a.max.as_ref().map(|m| m.z).unwrap_or(f64::NEG_INFINITY).max(b.max.as_ref().map(|m| m.z).unwrap_or(f64::NEG_INFINITY)),
        }),
    }
}

pub fn bbox_intersects(a: &BoundingBox, b: &BoundingBox) -> bool {
    let (amin, amax) = match (&a.min, &a.max) { (Some(n), Some(x)) => (n, x), _ => return false };
    let (bmin, bmax) = match (&b.min, &b.max) { (Some(n), Some(x)) => (n, x), _ => return false };
    amin.x <= bmax.x && amax.x >= bmin.x && amin.y <= bmax.y && amax.y >= bmin.y && amin.z <= bmax.z && amax.z >= bmin.z
}

pub fn transform_point(p: &Point3D, t: &Transform) -> Point3D {
    let (sx, sy, sz) = t.scale.as_ref().map(|s| (s.x, s.y, s.z)).unwrap_or((1.0, 1.0, 1.0));
    let (tx, ty, tz) = t.translation.as_ref().map(|t| (t.x, t.y, t.z)).unwrap_or((0.0, 0.0, 0.0));
    let (px, py, pz) = (p.x * sx, p.y * sy, p.z * sz);
    if let Some(rot) = &t.rotation {
        let (rx, ry, rz, rw) = (rot.x, rot.y, rot.z, rot.w);
        Point3D {
            x: tx + px * (1.0 - 2.0*(ry*ry+rz*rz)) + py * (2.0*(rx*ry-rw*rz)) + pz * (2.0*(rx*rz+rw*ry)),
            y: ty + px * (2.0*(rx*ry+rw*rz)) + py * (1.0 - 2.0*(rx*rx+rz*rz)) + pz * (2.0*(ry*rz-rw*rx)),
            z: tz + px * (2.0*(rx*rz-rw*ry)) + py * (2.0*(ry*rz+rw*rx)) + pz * (1.0 - 2.0*(rx*rx+ry*ry)),
        }
    } else {
        Point3D { x: px + tx, y: py + ty, z: pz + tz }
    }
}

pub fn transform_mesh(mesh: &MeshGeometry, t: &Transform) -> MeshGeometry {
    let vertices: Vec<Point3D> = mesh.vertices.iter().map(|v| transform_point(v, t)).collect();
    let normals = if let Some(rot) = &t.rotation {
        mesh.normals.iter().map(|n| {
            let (rx, ry, rz, rw) = (rot.x, rot.y, rot.z, rot.w);
            Vector3D {
                x: n.x * (1.0 - 2.0*(ry*ry+rz*rz)) + n.y * (2.0*(rx*ry-rw*rz)) + n.z * (2.0*(rx*rz+rw*ry)),
                y: n.x * (2.0*(rx*ry+rw*rz)) + n.y * (1.0 - 2.0*(rx*rx+rz*rz)) + n.z * (2.0*(ry*rz-rw*rx)),
                z: n.x * (2.0*(rx*rz-rw*ry)) + n.y * (2.0*(ry*rz+rw*rx)) + n.z * (1.0 - 2.0*(rx*rx+ry*ry)),
            }
        }).collect()
    } else {
        mesh.normals.clone()
    };
    MeshGeometry { vertices, normals, uvs: mesh.uvs.clone(), faces: mesh.faces.clone() }
}

pub fn merge_meshes(meshes: &[MeshGeometry]) -> MeshGeometry {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut faces = Vec::new();
    for mesh in meshes {
        let offset = vertices.len() as i32;
        vertices.extend(mesh.vertices.iter().cloned());
        normals.extend(mesh.normals.iter().cloned());
        for face in &mesh.faces {
            faces.push(FaceIndices {
                vertex_indices: face.vertex_indices.iter().map(|i| i + offset).collect(),
            });
        }
    }
    MeshGeometry { vertices, normals, uvs: Vec::new(), faces }
}

#[derive(Serialize)]
pub struct SectionLine {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
}

pub fn compute_section_lines(mesh: &MeshGeometry, plane_z: f64) -> Vec<SectionLine> {
    let eps = 1e-6;
    let mut lines = Vec::new();

    for face in &mesh.faces {
        if face.vertex_indices.len() < 3 { continue; }
        let i0 = face.vertex_indices[0] as usize;
        let i1 = face.vertex_indices[1] as usize;
        let i2 = face.vertex_indices[2] as usize;
        if i0 >= mesh.vertices.len() || i1 >= mesh.vertices.len() || i2 >= mesh.vertices.len() {
            continue;
        }

        let v = [
            &mesh.vertices[i0],
            &mesh.vertices[i1],
            &mesh.vertices[i2],
        ];

        let d = [
            v[0].z - plane_z,
            v[1].z - plane_z,
            v[2].z - plane_z,
        ];

        let all_above = d[0] > eps && d[1] > eps && d[2] > eps;
        let all_below = d[0] < -eps && d[1] < -eps && d[2] < -eps;
        if all_above || all_below { continue; }

        let all_on_plane = d[0].abs() <= eps && d[1].abs() <= eps && d[2].abs() <= eps;

        if all_on_plane {
            for i in 0..3 {
                let j = (i + 1) % 3;
                lines.push(SectionLine {
                    x1: v[i].x, y1: v[i].y,
                    x2: v[j].x, y2: v[j].y,
                });
            }
            continue;
        }

        let mut pts = Vec::new();
        for i in 0..3 {
            let j = (i + 1) % 3;
            let di = d[i];
            let dj = d[j];

            let on_i = di.abs() <= eps;
            let on_j = dj.abs() <= eps;

            if on_i && on_j {
                pts.push((v[i].x, v[i].y));
                pts.push((v[j].x, v[j].y));
            } else if on_i {
                pts.push((v[i].x, v[i].y));
            } else if on_j {
                pts.push((v[j].x, v[j].y));
            } else if (di > eps) != (dj > eps) {
                let t = di / (di - dj);
                pts.push((
                    v[i].x + t * (v[j].x - v[i].x),
                    v[i].y + t * (v[j].y - v[i].y),
                ));
            }
        }

        let mut unique: Vec<(f64, f64)> = Vec::new();
        for p in &pts {
            if let Some(last) = unique.last() {
                if (p.0 - last.0).abs() < 1e-10 && (p.1 - last.1).abs() < 1e-10 {
                    continue;
                }
            }
            unique.push(*p);
        }

        if unique.len() >= 2 {
            lines.push(SectionLine {
                x1: unique[0].0, y1: unique[0].1,
                x2: unique[1].0, y2: unique[1].1,
            });
        }
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use c::parametric_operation::Operation;

    #[test]
    fn test_dispatch_box() {
        let op = Operation::Box(c::Box { length: 2.0, width: 1.0, height: 3.0 });
        let (v, n, f) = dispatch(&op);
        assert_eq!(v.len(), 24);
        assert_eq!(v.len(), n.len());
        assert!(!f.is_empty());
    }

    #[test]
    fn test_scale_mesh_uniform() {
        let op = Operation::Box(c::Box { length: 2000.0, width: 1000.0, height: 500.0 });
        let (v, _n, f) = dispatch(&op);
        let mesh = c::MeshGeometry { vertices: v, normals: Vec::new(), uvs: Vec::new(), faces: f };
        let scaled = scale_mesh(&mesh, 0.001);
        assert_eq!(scaled.vertices.len(), mesh.vertices.len());
        assert_eq!(scaled.faces.len(), mesh.faces.len());
        for (orig, s) in mesh.vertices.iter().zip(scaled.vertices.iter()) {
            assert!((s.x - orig.x * 0.001).abs() < 1e-9);
            assert!((s.y - orig.y * 0.001).abs() < 1e-9);
            assert!((s.z - orig.z * 0.001).abs() < 1e-9);
        }
        // A 2000×1000×500 mm box should become 2×1×0.5 m.
        let bb = compute_bbox(&scaled);
        let mn = bb.min.unwrap();
        let mx = bb.max.unwrap();
        assert!((mx.x - mn.x - 2.0).abs() < 1e-6, "width {} != 2.0 m", mx.x - mn.x);
        assert!((mx.y - mn.y - 1.0).abs() < 1e-6, "depth {} != 1.0 m", mx.y - mn.y);
        assert!((mx.z - mn.z - 0.5).abs() < 1e-6, "height {} != 0.5 m", mx.z - mn.z);
    }

    #[test]
    fn test_dispatch_cylinder() {
        let op = Operation::Cylinder(c::Cylinder { radius: 1.0, height: 2.0, segments: 16 });
        let (v, n, f) = dispatch(&op);
        assert!(v.len() > 10);
        assert_eq!(v.len(), n.len());
        assert!(!f.is_empty());
    }

    #[test]
    fn test_dispatch_torus() {
        let op = Operation::Torus(c::Torus { major_radius: 5.0, minor_radius: 2.0, segments: 16, rings: 8 });
        let (v, n, f) = dispatch(&op);
        assert_eq!(v.len(), 128);
        assert_eq!(v.len(), n.len());
        assert!(!f.is_empty());
    }

    #[test]
    fn test_dispatch_dish() {
        let op = Operation::Dish(c::Dish { crown_radius: 100.0, knuckle_radius: 10.0, diameter: 100.0, segments: 24 });
        let (v, n, f) = dispatch(&op);
        assert!(v.len() > 100);
        assert_eq!(v.len(), n.len());
        assert!(!f.is_empty());
    }

    #[test]
    fn test_dispatch_sweep() {
        let profile = vec![
            c::Point2D { x: 0.0, y: 0.0 },
            c::Point2D { x: 2.0, y: 0.0 },
            c::Point2D { x: 2.0, y: 1.0 },
            c::Point2D { x: 0.0, y: 1.0 },
        ];
        let path = vec![
            c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
            c::Point3D { x: 0.0, y: 0.0, z: 5.0 },
        ];
        let op = Operation::Sweep(c::Sweep { profile, path, segments: 16 });
        let (v, n, f) = dispatch(&op);
        assert!(v.len() >= 8);
        assert_eq!(v.len(), n.len());
        assert!(!f.is_empty());
    }

    #[test]
    fn test_dispatch_loft() {
        let profile0 = vec![
            c::Point2D { x: 0.0, y: 0.0 },
            c::Point2D { x: 2.0, y: 0.0 },
            c::Point2D { x: 2.0, y: 1.0 },
            c::Point2D { x: 0.0, y: 1.0 },
        ];
        let profile1 = vec![
            c::Point2D { x: 0.0, y: 3.0 },
            c::Point2D { x: 2.0, y: 3.0 },
            c::Point2D { x: 2.0, y: 4.0 },
            c::Point2D { x: 0.0, y: 4.0 },
        ];
        let op = Operation::Loft(c::Loft { profile0, profile1, segments: 16 });
        let (v, n, f) = dispatch(&op);
        assert!(v.len() >= 8);
        assert_eq!(v.len(), n.len());
        assert!(!f.is_empty());
    }

    #[test]
    fn test_parametric_to_mesh_sweep() {
        let op = c::ParametricOperation {
            operation: Some(Operation::Sweep(c::Sweep {
                profile: vec![
                    c::Point2D { x: 0.0, y: 0.0 },
                    c::Point2D { x: 1.0, y: 0.0 },
                    c::Point2D { x: 0.5, y: 0.866 },
                ],
                path: vec![
                    c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
                    c::Point3D { x: 0.0, y: 0.0, z: 3.0 },
                ],
                segments: 16,
            })),
        };
        let pg = c::ParametricGeometry { operations: vec![op] };
        let mesh = parametric_to_mesh(&pg);
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.faces.is_empty());
        assert_eq!(mesh.vertices.len(), mesh.normals.len());
    }
}
