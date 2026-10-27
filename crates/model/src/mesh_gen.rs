use omnim_core as c;
use c::{FaceIndices, Point3D, Vector3D};

type MeshData = (Vec<Point3D>, Vec<Vector3D>, Vec<FaceIndices>);

fn tri(a: i32, b: i32, c: i32) -> FaceIndices {
    FaceIndices { vertex_indices: vec![a, b, c] }
}

pub fn box_mesh(b: &c::Box) -> MeshData {
    let (l, w, h) = (b.length / 2.0, b.width / 2.0, b.height / 2.0);
    // Each of the 6 faces gets its own 4 vertices with the correct per-face normal.
    // 6 faces × 4 vertices = 24 vertices, 24 normals, 12 triangles.
    let mut v = Vec::with_capacity(24);
    let mut n = Vec::with_capacity(24);
    let mut f = Vec::with_capacity(12);

    // Face data: (normal_x, normal_y, normal_z,
    //              v0.x, v0.y, v0.z, v1.x, v1.y, v1.z,
    //              v2.x, v2.y, v2.z, v3.x, v3.y, v3.z)
    // Vertex order within each face is CCW from the outside.
    let faces: [(f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64); 6] = [
        // back  face: z = -h, normal ( 0,  0, -1)
        (0.0, 0.0, -1.0,  -l, -w, -h,   l, -w, -h,   l,  w, -h,  -l,  w, -h),
        // front face: z =  h, normal ( 0,  0,  1)
        (0.0, 0.0,  1.0,  -l, -w,  h,   l, -w,  h,   l,  w,  h,  -l,  w,  h),
        // bottom face: y = -w, normal ( 0, -1,  0)
        (0.0, -1.0, 0.0,  -l, -w, -h,   l, -w, -h,   l, -w,  h,  -l, -w,  h),
        // right face: x =  l, normal ( 1,  0,  0)
        (1.0, 0.0, 0.0,    l, -w, -h,   l,  w, -h,   l,  w,  h,   l, -w,  h),
        // top face: y =  w, normal ( 0,  1,  0)
        (0.0, 1.0, 0.0,   -l,  w, -h,   l,  w, -h,   l,  w,  h,  -l,  w,  h),
        // left face: x = -l, normal (-1,  0,  0)
        (-1.0, 0.0, 0.0,  -l, -w, -h,  -l,  w, -h,  -l,  w,  h,  -l, -w,  h),
    ];

    let mut base: i32 = 0;
    for (nx, ny, nz, x0, y0, z0, x1, y1, z1, x2, y2, z2, x3, y3, z3) in faces {
        v.push(Point3D { x: x0, y: y0, z: z0 });
        v.push(Point3D { x: x1, y: y1, z: z1 });
        v.push(Point3D { x: x2, y: y2, z: z2 });
        v.push(Point3D { x: x3, y: y3, z: z3 });
        let normal = Vector3D { x: nx, y: ny, z: nz };
        n.push(normal.clone());
        n.push(normal.clone());
        n.push(normal.clone());
        n.push(normal);
        // Quad → 2 triangles (CCW from outside)
        f.push(tri(base, base + 1, base + 2));
        f.push(tri(base, base + 2, base + 3));
        base += 4;
    }

    (v, n, f)
}

pub fn cylinder_mesh(c: &c::Cylinder) -> MeshData {
    let segs = if c.segments > 3 { c.segments as usize } else { 16 };
    let (r, h) = (c.radius, c.height / 2.0);
    let mut v = Vec::new();
    let mut n = Vec::new();
    let mut f = Vec::new();
    for i in 0..segs {
        let a = 2.0 * std::f64::consts::PI * i as f64 / segs as f64;
        let (ca, sa) = (a.cos(), a.sin());
        v.push(Point3D { x: r * ca, y: r * sa, z: -h });
        v.push(Point3D { x: r * ca, y: r * sa, z: h });
        let normal = Vector3D { x: ca, y: sa, z: 0.0 };
        n.push(normal.clone());
        n.push(normal);
    }
    for i in 0..segs {
        let nxt = (i + 1) % segs;
        let (a, b) = (i as i32 * 2, i as i32 * 2 + 1);
        let (ci, d) = (nxt as i32 * 2, nxt as i32 * 2 + 1);
        f.push(tri(a, ci, b));
        f.push(tri(b, ci, d));
    }
    let cb = v.len() as i32;
    v.push(Point3D { x: 0.0, y: 0.0, z: -h });
    n.push(Vector3D { x: 0.0, y: 0.0, z: -1.0 });
    let ct = v.len() as i32;
    v.push(Point3D { x: 0.0, y: 0.0, z: h });
    n.push(Vector3D { x: 0.0, y: 0.0, z: 1.0 });
    for i in 0..segs {
        let nxt = (i + 1) % segs;
        // Bottom cap: center → current → next (CCW when viewed from below, matching (0,0,-1) normal)
        f.push(tri(cb, i as i32 * 2, nxt as i32 * 2));
        // Top cap: center → current → next (CCW when viewed from above, matching (0,0,1) normal)
        f.push(tri(ct, i as i32 * 2 + 1, nxt as i32 * 2 + 1));
    }
    (v, n, f)
}

pub fn sphere_mesh(s: &c::Sphere) -> MeshData {
    let segs = if s.segments > 3 { s.segments as usize } else { 16 };
    let rings = segs / 2 + 1;
    let r = s.radius;
    let mut v = Vec::new();
    let mut n = Vec::new();
    let mut f = Vec::new();
    for ring in 0..=rings {
        let phi = std::f64::consts::PI * ring as f64 / rings as f64;
        for seg in 0..segs {
            let theta = 2.0 * std::f64::consts::PI * seg as f64 / segs as f64;
            let (x, y, z) = (r * phi.sin() * theta.cos(), r * phi.sin() * theta.sin(), r * phi.cos());
            v.push(Point3D { x, y, z });
            n.push(Vector3D { x: x / r, y: y / r, z: z / r });
        }
    }
    for ring in 0..rings {
        for seg in 0..segs {
            let ns = (seg + 1) % segs;
            let (a, b) = ((ring * segs + seg) as i32, (ring * segs + ns) as i32);
            let (ci, d) = (((ring + 1) * segs + seg) as i32, ((ring + 1) * segs + ns) as i32);
            f.push(tri(a, ci, b));
            f.push(tri(b, ci, d));
        }
    }
    (v, n, f)
}

pub fn cone_mesh(cone: &c::Cone) -> MeshData {
    let segs = if cone.segments > 3 { cone.segments as usize } else { 16 };
    let (rb, rt, h) = (cone.radius_bottom, cone.radius_top, cone.height / 2.0);
    // Side normal: for a cone frustum, the outward normal has a vertical component.
    // When rb != rt, the side surface slants: compute the slant angle.
    let dr = rt - rb;
    let slant_len = (dr * dr + cone.height * cone.height).sqrt();
    let (slant_cos, slant_sin) = if slant_len > 1e-10 {
        (cone.height / slant_len, -dr / slant_len)
    } else {
        (1.0, 0.0)
    };
    let mut v = Vec::new();
    let mut n = Vec::new();
    let mut f = Vec::new();
    for i in 0..segs {
        let a = 2.0 * std::f64::consts::PI * i as f64 / segs as f64;
        let (ca, sa) = (a.cos(), a.sin());
        v.push(Point3D { x: rb * ca, y: rb * sa, z: -h });
        v.push(Point3D { x: rt * ca, y: rt * sa, z: h });
        // Correct outward normal for a slanted cone side surface
        let normal = Vector3D {
            x: ca * slant_cos,
            y: sa * slant_cos,
            z: slant_sin,
        };
        n.push(normal.clone());
        n.push(normal);
    }
    for i in 0..segs {
        let nxt = (i + 1) % segs;
        let (a, b) = (i as i32 * 2, i as i32 * 2 + 1);
        let (ci, d) = (nxt as i32 * 2, nxt as i32 * 2 + 1);
        f.push(tri(a, ci, b));
        f.push(tri(b, ci, d));
    }
    let cb = v.len() as i32;
    v.push(Point3D { x: 0.0, y: 0.0, z: -h });
    n.push(Vector3D { x: 0.0, y: 0.0, z: -1.0 });
    let ct = v.len() as i32;
    v.push(Point3D { x: 0.0, y: 0.0, z: h });
    n.push(Vector3D { x: 0.0, y: 0.0, z: 1.0 });
    for i in 0..segs {
        let nxt = (i + 1) % segs;
        // Bottom cap: center → current → next (CCW from below)
        f.push(tri(cb, i as i32 * 2, nxt as i32 * 2));
        // Top cap: center → current → next (CCW from above), only if top radius > 0
        if rt > 0.0 {
            f.push(tri(ct, i as i32 * 2 + 1, nxt as i32 * 2 + 1));
        }
    }
    (v, n, f)
}

pub fn torus_mesh(t: &c::Torus) -> MeshData {
    let segs = if t.segments > 3 { t.segments as usize } else { 24 };
    let rings = if t.rings > 3 { t.rings as usize } else { 12 };
    let (major, minor) = (t.major_radius, t.minor_radius);
    let mut v = Vec::new();
    let mut n = Vec::new();
    let mut f = Vec::new();
    for ring in 0..rings {
        let phi = 2.0 * std::f64::consts::PI * ring as f64 / rings as f64;
        let (cp, sp) = (phi.cos(), phi.sin());
        for seg in 0..segs {
            let theta = 2.0 * std::f64::consts::PI * seg as f64 / segs as f64;
            let (ct, st) = (theta.cos(), theta.sin());
            let r = major + minor * cp;
            let x = r * ct;
            let y = r * st;
            let z = minor * sp;
            v.push(Point3D { x, y, z });
            n.push(Vector3D { x: ct * cp, y: st * cp, z: sp });
        }
    }
    for ring in 0..rings {
        let nxt_ring = (ring + 1) % rings;
        for seg in 0..segs {
            let nxt_seg = (seg + 1) % segs;
            let a = (ring * segs + seg) as i32;
            let b = (ring * segs + nxt_seg) as i32;
            let c = (nxt_ring * segs + seg) as i32;
            let d = (nxt_ring * segs + nxt_seg) as i32;
            f.push(tri(a, c, b));
            f.push(tri(b, c, d));
        }
    }
    (v, n, f)
}

pub fn dish_mesh(d: &c::Dish) -> MeshData {
    let segs = if d.segments > 3 { d.segments as usize } else { 24 };
    let (crown, knuckle, diameter) = (d.crown_radius, d.knuckle_radius, d.diameter);
    let a = diameter / 2.0;
    if crown <= knuckle || a <= knuckle {
        return (Vec::new(), Vec::new(), Vec::new());
    }
    // Crown-knuckle tangency angle: sin(phi_m) = (a - r) / (R - r)
    let sin_phi = ((a - knuckle) / (crown - knuckle)).clamp(-1.0, 1.0);
    let phi_m = sin_phi.asin();
    // Crown center C = (0, C_z), knuckle center K = (a - r, 0) with rim at z=0.
    // |C - K| = R - r  →  C_z = -(R - r) * cos(phi_m)
    let cz = -(crown - knuckle) * phi_m.cos();
    // Meridian sampling: from pole (phi=0) through crown to tangency, then knuckle to rim.
    let merid = (segs / 2).max(8) + 1;
    let mut profile: Vec<(f64, f64, f64, f64)> = Vec::new(); // (x, z, nx_merid, nz_merid)
    // Crown arc: angle phi from 0 to phi_m, outward normal = (sin phi, cos phi)
    for i in 0..merid {
        let t = i as f64 / (merid - 1) as f64;
        let phi = phi_m * t;
        let (sp, cp) = (phi.sin(), phi.cos());
        let x = crown * sp;
        let z = cz + crown * cp;
        profile.push((x, z, sp, cp));
    }
    // Knuckle arc: psi from 0 (rim) to (pi/2 - phi_m) (tangency)
    // point = K + r * (cos psi, sin psi), normal = (cos psi, sin psi)
    let psi_max = std::f64::consts::FRAC_PI_2 - phi_m;
    for i in 1..merid {
        let t = i as f64 / (merid - 1) as f64;
        let psi = psi_max * t;
        let (cpsi, spsi) = (psi.cos(), psi.sin());
        let x = (a - knuckle) + knuckle * cpsi;
        let z = knuckle * spsi;
        profile.push((x, z, cpsi, spsi));
    }
    // Revolve meridian profile around Z axis.
    let mut v = Vec::new();
    let mut n = Vec::new();
    let mut f = Vec::new();
    let rows = profile.len();
    for r in 0..rows {
        let (px, _pz, nx_m, nz_m) = profile[r];
        for s in 0..segs {
            let theta = 2.0 * std::f64::consts::PI * s as f64 / segs as f64;
            let (ct, st) = (theta.cos(), theta.sin());
            v.push(Point3D { x: px * ct, y: px * st, z: _pz });
            n.push(Vector3D { x: nx_m * ct, y: nx_m * st, z: nz_m });
        }
    }
    for r in 0..rows - 1 {
        for s in 0..segs {
            let ns = (s + 1) % segs;
            let a_idx = (r * segs + s) as i32;
            let b = (r * segs + ns) as i32;
            let c = ((r + 1) * segs + s) as i32;
            let d = ((r + 1) * segs + ns) as i32;
            f.push(tri(a_idx, c, b));
            f.push(tri(b, c, d));
        }
    }
    (v, n, f)
}

/// Generate a reinforcing bar cage mesh for a concrete member.
///
/// `diameter` - rebar bar diameter (mm)
/// `main_bars` - number of longitudinal main bars around the perimeter
/// `length` - cage length along Z
/// `bar_ring_radius` - radius of the ring on which main bars are placed
/// `stirrup_spacing` - spacing between stirrups/ties along the length
/// `stirrup_radius` - radius of each stirrup ring
pub fn rebar_cage_mesh(
    bar_ring_radius: f64,
    main_bars: usize,
    length: f64,
    diameter: f64,
    stirrup_spacing: f64,
    stirrup_radius: f64,
) -> MeshData {
    let segs = main_bars.max(4);
    let mut v = Vec::new();
    let mut n = Vec::new();
    let mut f = Vec::new();

    // Main longitudinal bars: each is a thin cylinder along Z at radius `bar_ring_radius`.
    for b in 0..segs {
        let a = 2.0 * std::f64::consts::PI * b as f64 / segs as f64;
        let (ca, sa) = (a.cos(), a.sin());
        let cx = bar_ring_radius * ca;
        let cy = bar_ring_radius * sa;
        // 8 segments around each bar's cross-section
        let bar_segs = 8usize;
        let mut bar_v = Vec::new();
        for s in 0..bar_segs {
            let ta = 2.0 * std::f64::consts::PI * s as f64 / bar_segs as f64;
            let (ct, st) = (ta.cos(), ta.sin());
            let r = diameter / 2.0;
            bar_v.push(Point3D {
                x: cx + r * ct * ca,
                y: cy + r * ct * sa,
                z: st * r,
            });
            bar_v.push(Point3D {
                x: cx + r * ct * ca,
                y: cy + r * ct * sa,
                z: length + st * r,
            });
            let nrm = Vector3D { x: ct * ca, y: ct * sa, z: 0.0 };
            n.push(nrm.clone());
            n.push(nrm);
        }
        let base = v.len() as i32;
        for s in 0..bar_segs {
            let ns = (s + 1) % bar_segs;
            let (a_i, b_i) = (s as i32 * 2, s as i32 * 2 + 1);
            let (c_i, d_i) = (ns as i32 * 2, ns as i32 * 2 + 1);
            f.push(tri(base + a_i, base + c_i, base + b_i));
            f.push(tri(base + b_i, base + c_i, base + d_i));
        }
        v.extend(bar_v);
    }

    // Stirrups: thin torus rings spaced along the length.
    if stirrup_spacing > 0.0 && stirrup_radius > 0.0 && diameter > 0.0 {
        let n_stirrups = ((length / stirrup_spacing).floor() as usize + 1).max(2);
        let ring_segs = 16usize;
        let bar_segs = 6usize;
        let r = diameter / 2.0;
        for k in 0..n_stirrups {
            let _z = k as f64 * stirrup_spacing;
            let base = v.len() as i32;
            for s in 0..ring_segs {
                let a = 2.0 * std::f64::consts::PI * s as f64 / ring_segs as f64;
                let (ca, sa) = (a.cos(), a.sin());
                for t in 0..bar_segs {
                    let ta = 2.0 * std::f64::consts::PI * t as f64 / bar_segs as f64;
                    let (ct, st) = (ta.cos(), ta.sin());
                    v.push(Point3D {
                        x: (stirrup_radius + r * ct) * ca,
                        y: (stirrup_radius + r * ct) * sa,
                        z: r * st,
                    });
                    n.push(Vector3D { x: ct * ca, y: ct * sa, z: st });
                }
            }
            for s in 0..ring_segs {
                let ns = (s + 1) % ring_segs;
                for t in 0..bar_segs {
                    let nt = (t + 1) % bar_segs;
                    let a_i = s * bar_segs + t;
                    let b_i = s * bar_segs + nt;
                    let c_i = ns * bar_segs + t;
                    let d_i = ns * bar_segs + nt;
                    f.push(tri(base + a_i as i32, base + c_i as i32, base + b_i as i32));
                    f.push(tri(base + b_i as i32, base + c_i as i32, base + d_i as i32));
                }
            }
        }
    }

    (v, n, f)
}

/// Extrude a solved sketch profile into a 3D mesh.
///
/// `sketch`: must have `solved == true` with valid solved points.
/// `depth`: extrusion depth.
/// `direction`: extrusion direction vector (will be normalized; defaults to Z if zero).
pub fn extrude_sketch(
    sketch: &omnim_sketch::Sketch,
    depth: f64,
    direction: Vector3D,
) -> Option<MeshData> {
    let profile2d = omnim_sketch::profile::sketch_to_profile_2d(sketch)?;
    if profile2d.len() < 3 {
        return None;
    }
    let profile3d: Vec<Point3D> = profile2d.iter()
        .map(|p| Point3D { x: p.x, y: p.y, z: 0.0 })
        .collect();
    let dir = if direction.x == 0.0 && direction.y == 0.0 && direction.z == 0.0 {
        Vector3D { x: 0.0, y: 0.0, z: 1.0 }
    } else {
        let len = (direction.x.powi(2) + direction.y.powi(2) + direction.z.powi(2)).sqrt();
        Vector3D { x: direction.x / len, y: direction.y / len, z: direction.z / len }
    };
    crate::brep::extrude_brep(&profile3d, depth, dir, crate::brep::DEFAULT_TOLERANCE)
        .map(|mesh| (mesh.vertices, mesh.normals, mesh.faces))
}

/// Revolve a solved sketch profile around an axis into a 3D mesh.
///
/// `sketch`: must have `solved == true` with valid solved points.
/// `angle`: revolution angle in radians (0 = full 360°).
/// `axis_origin`: point on the revolution axis.
/// `axis_direction`: direction of the revolution axis (will be normalized).
pub fn revolve_sketch(
    sketch: &omnim_sketch::Sketch,
    angle: f64,
    axis_origin: Point3D,
    axis_direction: Vector3D,
) -> Option<MeshData> {
    let profile2d = omnim_sketch::profile::sketch_to_profile_2d(sketch)?;
    if profile2d.len() < 2 {
        return None;
    }

    let len = (axis_direction.x.powi(2) + axis_direction.y.powi(2) + axis_direction.z.powi(2)).sqrt();
    let (ndx, ndy, ndz) = if len > 1e-10 {
        (axis_direction.x / len, axis_direction.y / len, axis_direction.z / len)
    } else {
        (0.0, 0.0, 1.0)
    };

    // Build orthonormal basis for the revolution plane (u ⊥ axis, v = axis × u)
    let (ux, uy, uz) = if ndx.abs() < 0.9 {
        (ndz, 0.0, -ndx)
    } else {
        (0.0, -ndz, ndy)
    };
    let ulen = (ux * ux + uy * uy + uz * uz).sqrt();
    let (ux, uy, uz) = if ulen > 1e-10 {
        (ux / ulen, uy / ulen, uz / ulen)
    } else {
        (1.0, 0.0, 0.0)
    };
    let (vx, vy, vz) = (
        uy * ndz - uz * ndy,
        uz * ndx - ux * ndz,
        ux * ndy - uy * ndx,
    );

    let profile3d: Vec<Point3D> = profile2d.iter().map(|p| Point3D {
        x: axis_origin.x + p.x * ux + p.y * vx,
        y: axis_origin.y + p.x * uy + p.y * vy,
        z: axis_origin.z + p.x * uz + p.y * vz,
    }).collect();

    crate::brep::revolve_brep(&profile3d, angle, axis_origin, axis_direction, crate::brep::DEFAULT_TOLERANCE)
        .map(|mesh| (mesh.vertices, mesh.normals, mesh.faces))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::mesh_to_geometry;

    fn tuple_to_meshgeom(v: &[Point3D], n: &[Vector3D], f: &[FaceIndices]) -> c::MeshGeometry {
        c::MeshGeometry {
            vertices: v.to_vec(),
            normals: n.to_vec(),
            uvs: vec![],
            faces: f.to_vec(),
        }
    }

    #[test]
    fn test_extrude_sketch_empty() {
        let sketch = omnim_sketch::Sketch::new(
            "empty",
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        );
        let result = extrude_sketch(&sketch, 1.0, Vector3D { x: 0.0, y: 0.0, z: 1.0 });
        assert!(result.is_none());
}

    #[test]
    fn test_torus_mesh_basic() {
        let torus = c::Torus {
            major_radius: 5.0,
            minor_radius: 2.0,
            segments: 16,
            rings: 8,
        };
        let (v, n, f) = torus_mesh(&torus);
        assert!(!v.is_empty());
        assert!(!n.is_empty());
        assert!(!f.is_empty());
        assert_eq!(v.len(), n.len());
        // 8 rings * 16 segments = 128 vertices
        assert_eq!(v.len(), 128);
    }

    #[test]
    fn test_dish_mesh_basic() {
        let dish = c::Dish {
            crown_radius: 100.0,
            knuckle_radius: 10.0,
            diameter: 100.0,
            segments: 24,
        };
        let (v, n, f) = dish_mesh(&dish);
        assert!(!v.is_empty());
        assert!(!n.is_empty());
        assert!(!f.is_empty());
        assert_eq!(v.len(), n.len());
    }

    #[test]
    fn test_dish_mesh_invalid_params() {
        let dish = c::Dish {
            crown_radius: 5.0,  // crown < knuckle → invalid
            knuckle_radius: 10.0,
            diameter: 100.0,
            segments: 24,
        };
        let (v, n, f) = dish_mesh(&dish);
        assert!(v.is_empty());
        assert!(n.is_empty());
        assert!(f.is_empty());
    }

    #[test]
    fn test_extrude_sketch_triangle() {
        let mut sketch = omnim_sketch::Sketch::new(
            "tri",
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        );
        sketch.add_entity(omnim_sketch::SketchEntity::line(
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Point3D { x: 1.0, y: 0.0, z: 0.0 },
        ));
        sketch.add_entity(omnim_sketch::SketchEntity::line(
            Point3D { x: 1.0, y: 0.0, z: 0.0 },
            Point3D { x: 0.5, y: 1.0, z: 0.0 },
        ));
        sketch.add_entity(omnim_sketch::SketchEntity::line(
            Point3D { x: 0.5, y: 1.0, z: 0.0 },
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
        ));

        sketch.solved = true;
        sketch.set_solved_point(0, 0, Point3D { x: 0.0, y: 0.0, z: 0.0 });
        sketch.set_solved_point(0, 1, Point3D { x: 1.0, y: 0.0, z: 0.0 });
        sketch.set_solved_point(1, 0, Point3D { x: 1.0, y: 0.0, z: 0.0 });
        sketch.set_solved_point(1, 1, Point3D { x: 0.5, y: 1.0, z: 0.0 });
        sketch.set_solved_point(2, 0, Point3D { x: 0.5, y: 1.0, z: 0.0 });
        sketch.set_solved_point(2, 1, Point3D { x: 0.0, y: 0.0, z: 0.0 });

        let (v, n, f) = extrude_sketch(&sketch, 2.0, Vector3D { x: 0.0, y: 0.0, z: 1.0 }).unwrap();
        assert!(!v.is_empty());
        assert!(!n.is_empty());
        assert!(!f.is_empty());
        assert_eq!(v.len(), n.len());

        let mg = tuple_to_meshgeom(&v, &n, &f);
        let geom = mesh_to_geometry(&mg);
        match &geom.representation {
            Some(c::geometry::Representation::Mesh(m)) => assert_eq!(m.vertices.len(), v.len()),
            _ => {},
        }
    }

    #[test]
    fn test_rebar_cage_mesh() {
        let (v, n, f) = rebar_cage_mesh(
            200.0,   // bar ring radius
            8,       // main bars
            3000.0,  // length
            16.0,    // bar diameter
            200.0,   // stirrup spacing
            190.0,   // stirrup radius
        );
        assert!(!v.is_empty());
        assert!(!n.is_empty());
        assert!(!f.is_empty());
        assert_eq!(v.len(), n.len());
    }

    #[test]
    fn test_rebar_cage_no_stirrups() {
        let (v, n, f) = rebar_cage_mesh(200.0, 8, 3000.0, 16.0, 0.0, 0.0);
        assert!(!v.is_empty());
        assert_eq!(v.len(), n.len());
        assert!(!f.is_empty());
    }

    #[test]
    fn test_revolve_sketch_rectangle() {
        let mut sketch = omnim_sketch::Sketch::new(
            "rect",
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        );
        // Closed rectangle profile: 4 edges
        sketch.add_entity(omnim_sketch::SketchEntity::line(
            Point3D { x: 2.0, y: 0.0, z: 0.0 },
            Point3D { x: 3.0, y: 0.0, z: 0.0 },
        ));
        sketch.add_entity(omnim_sketch::SketchEntity::line(
            Point3D { x: 3.0, y: 0.0, z: 0.0 },
            Point3D { x: 3.0, y: 1.0, z: 0.0 },
        ));
        sketch.add_entity(omnim_sketch::SketchEntity::line(
            Point3D { x: 3.0, y: 1.0, z: 0.0 },
            Point3D { x: 2.0, y: 1.0, z: 0.0 },
        ));
        sketch.add_entity(omnim_sketch::SketchEntity::line(
            Point3D { x: 2.0, y: 1.0, z: 0.0 },
            Point3D { x: 2.0, y: 0.0, z: 0.0 },
        ));

        sketch.solved = true;
        sketch.set_solved_point(0, 0, Point3D { x: 2.0, y: 0.0, z: 0.0 });
        sketch.set_solved_point(0, 1, Point3D { x: 3.0, y: 0.0, z: 0.0 });
        sketch.set_solved_point(1, 0, Point3D { x: 3.0, y: 0.0, z: 0.0 });
        sketch.set_solved_point(1, 1, Point3D { x: 3.0, y: 1.0, z: 0.0 });
        sketch.set_solved_point(2, 0, Point3D { x: 3.0, y: 1.0, z: 0.0 });
        sketch.set_solved_point(2, 1, Point3D { x: 2.0, y: 1.0, z: 0.0 });
        sketch.set_solved_point(3, 0, Point3D { x: 2.0, y: 1.0, z: 0.0 });
        sketch.set_solved_point(3, 1, Point3D { x: 2.0, y: 0.0, z: 0.0 });

        let result = revolve_sketch(
            &sketch,
            0.0,
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        );
        assert!(result.is_some());
        let (v, _n, f) = result.unwrap();
        assert!(!v.is_empty());
        assert!(!f.is_empty());
    }
}
