//! BooleanSubtractGen — 布尔差集引擎
//!
//! 在宿主几何体上减除一个子几何体（开孔/剪切）。
//! 适用于：门 (Door)、窗 (Window) 在墙体上的开孔。
//!
//! 原理：
//! - 计算子 mesh 的包围盒和主轴线方向
//! - 移除宿主 mesh 中质心落在子包围盒内的三角形
//! - 在剪切边界处沿检测到的主轴线方向生成封闭面

use omnim_core as c;
use c::{FaceIndices, MeshGeometry, Point3D, Vector3D};

/// 布尔差集输入参数
#[derive(Debug, Clone)]
pub struct BooleanSubtractInput {
    /// 宿主几何体
    pub host_mesh: MeshGeometry,
    /// 减除几何体
    pub subtract_mesh: MeshGeometry,
}

/// 包围盒
#[derive(Debug, Clone)]
struct BBox {
    min: Point3D,
    max: Point3D,
}

impl BBox {
    fn from_mesh(mesh: &MeshGeometry) -> Self {
        if mesh.vertices.is_empty() {
            return Self {
                min: Point3D { x: 0.0, y: 0.0, z: 0.0 },
                max: Point3D { x: 0.0, y: 0.0, z: 0.0 },
            };
        }
        let mut mn = Point3D { x: f64::INFINITY, y: f64::INFINITY, z: f64::INFINITY };
        let mut mx = Point3D { x: f64::NEG_INFINITY, y: f64::NEG_INFINITY, z: f64::NEG_INFINITY };
        for v in &mesh.vertices {
            mn.x = mn.x.min(v.x); mn.y = mn.y.min(v.y); mn.z = mn.z.min(v.z);
            mx.x = mx.x.max(v.x); mx.y = mx.y.max(v.y); mx.z = mx.z.max(v.z);
        }
        Self { min: mn, max: mx }
    }

    fn contains(&self, p: &Point3D, tolerance: f64) -> bool {
        p.x >= self.min.x - tolerance && p.x <= self.max.x + tolerance
            && p.y >= self.min.y - tolerance && p.y <= self.max.y + tolerance
            && p.z >= self.min.z - tolerance && p.z <= self.max.z + tolerance
    }

    fn center(&self) -> Point3D {
        Point3D {
            x: (self.min.x + self.max.x) / 2.0,
            y: (self.min.y + self.max.y) / 2.0,
            z: (self.min.z + self.max.z) / 2.0,
        }
    }

    /// 检测主轴线方向（最长轴索引: 0=x, 1=y, 2=z）
    fn dominant_axis(&self) -> usize {
        let sx = self.max.x - self.min.x;
        let sy = self.max.y - self.min.y;
        let sz = self.max.z - self.min.z;
        if sx >= sy && sx >= sz { 0 }
        else if sy >= sx && sy >= sz { 1 }
        else { 2 }
    }

    /// 沿主轴方向的两个端面中心
    fn axis_end_centers(&self, axis: usize) -> (Point3D, Point3D) {
        let center = self.center();
        let half_extent = match axis {
            0 => (self.max.x - self.min.x) / 2.0,
            1 => (self.max.y - self.min.y) / 2.0,
            _ => (self.max.z - self.min.z) / 2.0,
        };
        let mut neg = center.clone();
        let mut pos = center.clone();
        match axis {
            0 => { neg.x -= half_extent; pos.x += half_extent; }
            1 => { neg.y -= half_extent; pos.y += half_extent; }
            _ => { neg.z -= half_extent; pos.z += half_extent; }
        }
        (neg, pos)
    }
}

/// 生成布尔差集结果
///
/// 将宿主 mesh 中落在减除 mesh AABB 内的三角形移除，
/// 并在沿减除体主轴方向的剪切边界处添加封闭面。
pub fn generate(input: &BooleanSubtractInput) -> MeshGeometry {
    if input.host_mesh.vertices.is_empty() {
        return MeshGeometry {
            vertices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            faces: Vec::new(),
        };
    }
    if input.subtract_mesh.vertices.is_empty() {
        return input.host_mesh.clone();
    }

    let sub_bbox = BBox::from_mesh(&input.subtract_mesh);
    let tolerance = 1e-6;

    let mut vertices: Vec<Point3D> = Vec::new();
    let mut normals: Vec<Vector3D> = Vec::new();
    let mut faces: Vec<FaceIndices> = Vec::new();

    // 用于 cap 边界检测：收集被移除三角形的边界边
    let mut boundary_edges: Vec<(i32, i32)> = Vec::new();

    for face in &input.host_mesh.faces {
        if face.vertex_indices.len() < 3 {
            continue;
        }
        let centroid = triangle_centroid(&input.host_mesh.vertices, &face.vertex_indices);
        if !sub_bbox.contains(&centroid, tolerance) {
            // 保留这个三角形
            let base = vertices.len() as i32;
            for vi in &face.vertex_indices {
                let idx = *vi as usize;
                if idx < input.host_mesh.vertices.len() {
                    vertices.push(input.host_mesh.vertices[idx].clone());
                    if idx < input.host_mesh.normals.len() {
                        normals.push(input.host_mesh.normals[idx].clone());
                    } else {
                        normals.push(Vector3D { x: 0.0, y: 0.0, z: 1.0 });
                    }
                }
            }
            faces.push(FaceIndices {
                vertex_indices: (0..face.vertex_indices.len() as i32).map(|i| base + i).collect(),
            });
        } else {
            // 移除的三角形：收集其边界边（与保留三角形的共享边）
            let n = face.vertex_indices.len();
            for i in 0..n {
                let a = face.vertex_indices[i as usize];
                let b = face.vertex_indices[(i + 1) % n];
                boundary_edges.push((a.min(b), a.max(b)));
            }
        }
    }

    // 去重边界边（保留只出现一次的边 = 剪切边界）
    boundary_edges.sort();
    let mut cut_edges: Vec<(i32, i32)> = Vec::new();
    let mut i = 0;
    while i < boundary_edges.len() {
        if i + 1 < boundary_edges.len() && boundary_edges[i] == boundary_edges[i + 1] {
            i += 2; // 共享边，不是边界
        } else {
            cut_edges.push(boundary_edges[i]);
            i += 1;
        }
    }

    // 沿减除体主轴方向生成 cap 封闭面
    let dominant_axis = sub_bbox.dominant_axis();
    let (neg_center, pos_center) = sub_bbox.axis_end_centers(dominant_axis);

    // 从剪切边界边构建封闭多边形
    if cut_edges.len() >= 3 {
        let cap_mesh = generate_cap_from_edges(
            &input.host_mesh, &cut_edges, &sub_bbox,
            &neg_center, &pos_center, dominant_axis, tolerance,
        );
        let offset = vertices.len() as i32;
        vertices.extend(cap_mesh.vertices.iter().cloned());
        normals.extend(cap_mesh.normals.iter().cloned());
        for face in &cap_mesh.faces {
            faces.push(FaceIndices {
                vertex_indices: face.vertex_indices.iter().map(|vi| vi + offset).collect(),
            });
        }
    }

    // 兜底：如果没有找到足够的边界边，用 AABB 端面代替
    if cut_edges.len() < 3 {
        let fallback_caps = generate_bbox_cap_faces(&input.subtract_mesh, dominant_axis);
        let offset = vertices.len() as i32;
        vertices.extend(fallback_caps.0.iter().cloned());
        normals.extend(fallback_caps.1.iter().cloned());
        for face in &fallback_caps.2 {
            faces.push(FaceIndices {
                vertex_indices: face.vertex_indices.iter().map(|vi| vi + offset).collect(),
            });
        }
    }

    MeshGeometry {
        vertices,
        normals,
        uvs: Vec::new(),
        faces,
    }
}

/// 从剪切边界边生成封闭面
///
/// 将边界边映射回原始顶点，然后分别在减除体两端生成封闭面
fn generate_cap_from_edges(
    host: &MeshGeometry,
    cut_edges: &[(i32, i32)],
    sub_bbox: &BBox,
    neg_center: &Point3D,
    pos_center: &Point3D,
    axis: usize,
    _tolerance: f64,
) -> MeshGeometry {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut faces = Vec::new();

    // 收集边界顶点索引（去重）
    let mut vert_indices: Vec<i32> = cut_edges.iter()
        .flat_map(|(a, b)| vec![*a, *b])
        .collect();
    vert_indices.sort();
    vert_indices.dedup();

    if vert_indices.len() < 3 {
        return MeshGeometry { vertices, normals, uvs: Vec::new(), faces };
    }

    // 对边界顶点按位置排序，形成环
    // 按角度排序：计算中心，然后按绕中心的角度排列
    let center = sub_bbox.center();
    let edge_verts: Vec<Point3D> = vert_indices.iter()
        .map(|idx| {
            if *idx >= 0 && (*idx as usize) < host.vertices.len() {
                host.vertices[*idx as usize].clone()
            } else {
                center.clone()
            }
        })
        .collect();

    // 按绕主轴的角度排序
    let mut indexed: Vec<(usize, f64)> = vert_indices.iter().enumerate()
        .map(|(i, _)| {
            let v = &edge_verts[i];
            let angle = match axis {
                0 => (v.y - center.y).atan2(v.z - center.z),
                1 => (v.z - center.z).atan2(v.x - center.x),
                _ => (v.x - center.x).atan2(v.y - center.y),
            };
            (i, angle)
        })
        .collect();
    indexed.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    if indexed.len() < 3 {
        return MeshGeometry { vertices, normals, uvs: Vec::new(), faces };
    }

    // 两端各生成一个封闭面
    let n_verts = indexed.len();
    for (cap_center, is_neg) in [(neg_center, true), (pos_center, false)] {
        let base = vertices.len() as i32;

        // 法线方向（沿主轴）
        let normal = if is_neg {
            match axis {
                0 => Vector3D { x: -1.0, y: 0.0, z: 0.0 },
                1 => Vector3D { x: 0.0, y: -1.0, z: 0.0 },
                _ => Vector3D { x: 0.0, y: 0.0, z: -1.0 },
            }
        } else {
            match axis {
                0 => Vector3D { x: 1.0, y: 0.0, z: 0.0 },
                1 => Vector3D { x: 0.0, y: 1.0, z: 0.0 },
                _ => Vector3D { x: 0.0, y: 0.0, z: 1.0 },
            }
        };

        // 添加边界顶点（投影到 cap 平面）
        for (i, _) in &indexed {
            let v = &edge_verts[*i];
            let projected = match axis {
                0 => Point3D { x: cap_center.x, y: v.y, z: v.z },
                1 => Point3D { x: v.x, y: cap_center.y, z: v.z },
                _ => Point3D { x: v.x, y: v.y, z: cap_center.z },
            };
            vertices.push(projected);
            normals.push(normal.clone());
        }

        // 中心顶点
        let center_idx = vertices.len() as i32;
        vertices.push(cap_center.clone());
        normals.push(normal);

        // 扇形三角剖分
        for i in 0..n_verts {
            let nxt = (i + 1) % n_verts;
            let a = base + i as i32;
            let b = base + nxt as i32;
            if is_neg {
                faces.push(FaceIndices { vertex_indices: vec![center_idx, a, b] });
            } else {
                faces.push(FaceIndices { vertex_indices: vec![a, center_idx, b] });
            }
        }
    }

    MeshGeometry { vertices, normals, uvs: Vec::new(), faces }
}

/// 基于减除体的 AABB 生成端面（兜底方案）
fn generate_bbox_cap_faces(subtract: &MeshGeometry, axis: usize) -> (Vec<Point3D>, Vec<Vector3D>, Vec<FaceIndices>) {
    let sub_bbox = BBox::from_mesh(subtract);
    let (mut vertices, mut normals, mut faces) = (Vec::new(), Vec::new(), Vec::new());

    // 两个端面
    for is_neg in [true, false] {
        let base = vertices.len() as i32;

        let (cx, cy, cz, normal) = if is_neg {
            match axis {
                0 => (sub_bbox.min.x, 0.0, 0.0, Vector3D { x: -1.0, y: 0.0, z: 0.0 }),
                1 => (0.0, sub_bbox.min.y, 0.0, Vector3D { x: 0.0, y: -1.0, z: 0.0 }),
                _ => (0.0, 0.0, sub_bbox.min.z, Vector3D { x: 0.0, y: 0.0, z: -1.0 }),
            }
        } else {
            match axis {
                0 => (sub_bbox.max.x, 0.0, 0.0, Vector3D { x: 1.0, y: 0.0, z: 0.0 }),
                1 => (0.0, sub_bbox.max.y, 0.0, Vector3D { x: 0.0, y: 1.0, z: 0.0 }),
                _ => (0.0, 0.0, sub_bbox.max.z, Vector3D { x: 0.0, y: 0.0, z: 1.0 }),
            }
        };

        let (y0, y1, z0, z1) = match axis {
            0 => (sub_bbox.min.y, sub_bbox.max.y, sub_bbox.min.z, sub_bbox.max.z),
            1 => (sub_bbox.min.z, sub_bbox.max.z, sub_bbox.min.x, sub_bbox.max.x),
            _ => (sub_bbox.min.x, sub_bbox.max.x, sub_bbox.min.y, sub_bbox.max.y),
        };

        let corners = [
            Point3D { x: cx + (if axis == 1 { z0 } else { 0.0 }), y: cy + (if axis == 1 { 0.0 } else { y0 }), z: cz + (if axis == 1 { y0 } else { z0 }) },
            Point3D { x: cx + (if axis == 1 { z1 } else { 0.0 }), y: cy + (if axis == 1 { 0.0 } else { y1 }), z: cz + (if axis == 1 { y0 } else { z0 }) },
            Point3D { x: cx + (if axis == 1 { z1 } else { 0.0 }), y: cy + (if axis == 1 { 0.0 } else { y1 }), z: cz + (if axis == 1 { y1 } else { z1 }) },
            Point3D { x: cx + (if axis == 1 { z0 } else { 0.0 }), y: cy + (if axis == 1 { 0.0 } else { y0 }), z: cz + (if axis == 1 { y1 } else { z0 }) },
        ];

        for c in &corners {
            vertices.push(c.clone());
            normals.push(normal.clone());
        }

        if is_neg {
            faces.push(FaceIndices { vertex_indices: vec![base, base + 2, base + 1] });
            faces.push(FaceIndices { vertex_indices: vec![base, base + 3, base + 2] });
        } else {
            faces.push(FaceIndices { vertex_indices: vec![base, base + 1, base + 2] });
            faces.push(FaceIndices { vertex_indices: vec![base, base + 2, base + 3] });
        }
    }

    (vertices, normals, faces)
}

/// 计算三角形的质心
fn triangle_centroid(vertices: &[Point3D], indices: &[i32]) -> Point3D {
    let mut sum = Point3D { x: 0.0, y: 0.0, z: 0.0 };
    let mut count = 0;
    for idx in indices {
        if *idx >= 0 && (*idx as usize) < vertices.len() {
            sum.x += vertices[*idx as usize].x;
            sum.y += vertices[*idx as usize].y;
            sum.z += vertices[*idx as usize].z;
            count += 1;
        }
    }
    if count > 0 {
        Point3D { x: sum.x / count as f64, y: sum.y / count as f64, z: sum.z / count as f64 }
    } else {
        Point3D { x: 0.0, y: 0.0, z: 0.0 }
    }
}

// ============================================================
// 便捷构建函数
// ============================================================

/// 在墙体上开门洞
///
/// - wall_length, wall_height, wall_thickness: 墙体尺寸
/// - opening_width, opening_height: 开口尺寸
/// - position_x: 开口沿 X 轴位置（从墙起点算起）
/// - position_z: 开口沿 Z 轴位置（从墙底部算起）
pub fn build_door_opening(
    wall_length: f64,
    wall_height: f64,
    wall_thickness: f64,
    opening_width: f64,
    opening_height: f64,
    position_x: f64,
    position_z: f64,
) -> MeshGeometry {
    use crate::mesh_gen;
    use crate::geometry::transform_mesh;

    let (h_v, h_n, h_f) = mesh_gen::box_mesh(&c::Box {
        length: wall_length,
        width: wall_thickness,
        height: wall_height,
    });
    let host_mesh = MeshGeometry {
        vertices: h_v, normals: h_n, uvs: Vec::new(), faces: h_f,
    };

    let (s_v, s_n, s_f) = mesh_gen::box_mesh(&c::Box {
        length: opening_width,
        width: wall_thickness * 1.1,
        height: opening_height,
    });
    let positioned_sub = transform_mesh(
        &MeshGeometry { vertices: s_v, normals: s_n, uvs: Vec::new(), faces: s_f },
        &c::Transform {
            translation: Some(Point3D {
                x: position_x - wall_length / 2.0 + opening_width / 2.0,
                y: 0.0,
                z: position_z - wall_height / 2.0 + opening_height / 2.0,
            }),
            rotation: None, scale: None, matrix: Vec::new(),
        },
    );

    generate(&BooleanSubtractInput { host_mesh, subtract_mesh: positioned_sub })
}

/// 在墙上开窗洞
///
/// 与开门洞类似但窗洞底部抬升（sill height）
pub fn build_window_opening(
    wall_length: f64,
    wall_height: f64,
    wall_thickness: f64,
    opening_width: f64,
    opening_height: f64,
    position_x: f64,
    sill_height: f64,
) -> MeshGeometry {
    build_door_opening(wall_length, wall_height, wall_thickness, opening_width, opening_height, position_x, sill_height)
}

/// 在楼板上开洞
///
/// - slab_length, slab_width, slab_thickness: 楼板尺寸
/// - hole_width, hole_length: 洞口尺寸
/// - position_x, position_y: 洞口中心位置
pub fn build_slab_opening(
    slab_length: f64,
    slab_width: f64,
    slab_thickness: f64,
    hole_width: f64,
    hole_length: f64,
    position_x: f64,
    position_y: f64,
) -> MeshGeometry {
    use crate::mesh_gen;
    use crate::geometry::transform_mesh;

    let (h_v, h_n, h_f) = mesh_gen::box_mesh(&c::Box {
        length: slab_length, width: slab_width, height: slab_thickness,
    });
    let host_mesh = MeshGeometry {
        vertices: h_v, normals: h_n, uvs: Vec::new(), faces: h_f,
    };

    let (s_v, s_n, s_f) = mesh_gen::box_mesh(&c::Box {
        length: hole_width, width: hole_length, height: slab_thickness * 1.1,
    });
    let positioned_sub = transform_mesh(
        &MeshGeometry { vertices: s_v, normals: s_n, uvs: Vec::new(), faces: s_f },
        &c::Transform {
            translation: Some(Point3D { x: position_x, y: position_y, z: 0.0 }),
            rotation: None, scale: None, matrix: Vec::new(),
        },
    );

    generate(&BooleanSubtractInput { host_mesh, subtract_mesh: positioned_sub })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh_gen;

    #[test]
    fn test_subtract_empty_subtract() {
        let (v, n, f) = mesh_gen::box_mesh(&c::Box { length: 2.0, width: 2.0, height: 2.0 });
        let host = MeshGeometry { vertices: v, normals: n, uvs: Vec::new(), faces: f };
        let result = generate(&BooleanSubtractInput {
            host_mesh: host.clone(),
            subtract_mesh: MeshGeometry {
                vertices: Vec::new(), normals: Vec::new(), uvs: Vec::new(), faces: Vec::new(),
            },
        });
        assert_eq!(result.vertices.len(), host.vertices.len());
    }

    #[test]
    fn test_door_opening() {
        let mesh = build_door_opening(5.0, 3.0, 0.2, 0.9, 2.1, 2.0, 0.0);
        assert!(!mesh.vertices.is_empty());
        assert!(mesh.faces.len() > 0);
        // 应有 cap 面
        assert!(mesh.faces.len() > 4, "expected more faces for caps, got {}", mesh.faces.len());
    }

    #[test]
    fn test_subtract_full_overlap() {
        let (v, n, f) = mesh_gen::box_mesh(&c::Box { length: 1.0, width: 1.0, height: 1.0 });
        let host = MeshGeometry { vertices: v, normals: n, uvs: Vec::new(), faces: f };
        let result = generate(&BooleanSubtractInput {
            host_mesh: host,
            subtract_mesh: {
                let (v2, n2, f2) = mesh_gen::box_mesh(&c::Box { length: 3.0, width: 3.0, height: 3.0 });
                MeshGeometry { vertices: v2, normals: n2, uvs: Vec::new(), faces: f2 }
            },
        });
        // 所有三角形被移除，只剩 cap 面
        assert!(result.faces.len() >= 2, "expected at least 2 cap faces, got {}", result.faces.len());
    }

    #[test]
    fn test_slab_opening() {
        let mesh = build_slab_opening(6.0, 4.0, 0.2, 1.0, 0.5, 0.0, 0.0);
        assert!(!mesh.vertices.is_empty());
        assert!(mesh.faces.len() > 4);
    }

    #[test]
    fn test_window_opening() {
        let mesh = build_window_opening(4.0, 2.8, 0.2, 1.2, 1.5, 1.5, 0.9);
        assert!(!mesh.vertices.is_empty());
        assert!(mesh.faces.len() > 4);
    }

    #[test]
    fn test_partial_overlap() {
        // 减除体只与宿主部分重叠
        let (v, n, f) = mesh_gen::box_mesh(&c::Box { length: 4.0, width: 4.0, height: 4.0 });
        let host = MeshGeometry { vertices: v, normals: n, uvs: Vec::new(), faces: f };
        let (sv, sn, sf) = mesh_gen::box_mesh(&c::Box { length: 2.0, width: 6.0, height: 2.0 });
        let sub = crate::geometry::transform_mesh(
            &MeshGeometry { vertices: sv, normals: sn, uvs: Vec::new(), faces: sf },
            &c::Transform {
                translation: Some(Point3D { x: 0.0, y: 0.0, z: 0.0 }),
                rotation: None, scale: None, matrix: Vec::new(),
            },
        );
        let result = generate(&BooleanSubtractInput { host_mesh: host, subtract_mesh: sub });
        assert!(!result.vertices.is_empty());
        // 至少应有几根三角形保留 + cap 面
        assert!(result.faces.len() >= 2);
    }
}