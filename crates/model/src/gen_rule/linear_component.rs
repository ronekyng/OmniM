//! LinearComponentGen — 线性构件引擎
//!
//! 路径 + 变截面横断面 → MeshGeometry。
//! 适用于土木/管线构件：道路 (Road)、桥梁 (Bridge)、隧道 (Tunnel)、管道 (Pipe)。
//!
//! 原理：
//! - 沿对齐路径（3D 点序列）等距采样
//! - 在每个采样点按截面形状生成顶点环
//! - 相邻环之间生成四边形面片
//! - 两端加盖

use omnim_core as c;
use c::{FaceIndices, MeshGeometry, Point2D, Point3D, Vector3D};

/// 截面形状
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrossSectionShape {
    /// 矩形截面（道路、桥梁、梁）
    Rectangular,
    /// 圆形截面（管道、圆柱）
    Circular,
}

/// 横截面层定义
#[derive(Debug, Clone)]
pub struct CrossSectionLayer {
    /// 层名称
    pub name: String,
    /// 截面形状
    pub shape: CrossSectionShape,
    /// 默认宽度（矩形：总宽；圆形：直径）
    pub default_width: f64,
    /// 默认厚度（矩形：总高；圆形：忽略）
    pub default_thickness: f64,
    /// 材料 ID
    pub material_id: Option<String>,
    /// 功能
    pub function: String,
}

/// 变截面在路径位置处的定义
#[derive(Debug, Clone)]
pub struct CrossSectionAtStation {
    /// 路径位置（0.0 到 1.0）
    pub station: f64,
    /// 该位置处的截面层
    pub layers: Vec<CrossSectionLayer>,
}

/// LinearComponent 输入参数
#[derive(Debug, Clone)]
pub struct LinearComponentInput {
    /// 3D 对齐路径（至少 2 个点）
    pub alignment: Vec<Point3D>,
    /// 各站点截面定义（至少 1 个）
    pub cross_sections: Vec<CrossSectionAtStation>,
    /// 沿路径的分段数（控制精度）
    pub segments: usize,
    /// 截面环分段数（圆形截面用）
    pub ring_segments: usize,
}

impl Default for LinearComponentInput {
    fn default() -> Self {
        Self {
            alignment: vec![
                Point3D { x: 0.0, y: 0.0, z: 0.0 },
                Point3D { x: 10.0, y: 0.0, z: 0.0 },
            ],
            cross_sections: vec![CrossSectionAtStation {
                station: 0.0,
                layers: vec![CrossSectionLayer {
                    name: "RoadSurface".into(),
                    shape: CrossSectionShape::Rectangular,
                    default_width: 3.5,
                    default_thickness: 0.3,
                    material_id: None,
                    function: "pavement".into(),
                }],
            }],
            segments: 20,
            ring_segments: 16,
        }
    }
}

/// 生成线性构件 mesh
///
/// 沿对齐路径采样，在每个采样点根据截面生成顶点环，
/// 然后用四边形连接相邻环。支持多层截面和变截面。
pub fn generate(input: &LinearComponentInput) -> MeshGeometry {
    if input.alignment.len() < 2 || input.cross_sections.is_empty() {
        return MeshGeometry {
            vertices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            faces: Vec::new(),
        };
    }

    let path = interpolate_path(&input.alignment, input.segments);
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut faces = Vec::new();

    // 每个采样点生成所有层的顶点环，记录每层环的起点索引
    let mut ring_starts: Vec<Vec<usize>> = Vec::new();

    for (i, pt) in path.iter().enumerate() {
        let t = i as f64 / (path.len() - 1).max(1) as f64;
        let (forward, right, up) = compute_frenet_frame(&path, i, t);

        let mut layer_starts = Vec::new();

        for layer_idx in 0..max_layers(&input.cross_sections) {
            let cs = interpolate_cross_section_layer(&input.cross_sections, t, layer_idx);
            layer_starts.push(vertices.len());

            match cs.shape {
                CrossSectionShape::Rectangular => {
                    generate_rectangular_ring(pt, &forward, &right, &up, &cs, &mut vertices, &mut normals);
                }
                CrossSectionShape::Circular => {
                    generate_circular_ring(pt, &forward, &right, &up, &cs, input.ring_segments, &mut vertices, &mut normals);
                }
            }
        }

        ring_starts.push(layer_starts);

        // 连接相邻采样点的同一层环
        if i > 0 {
            let prev_starts = &ring_starts[i - 1];
            let curr_starts = &ring_starts[i];

            for layer_idx in 0..max_layers(&input.cross_sections) {
                if layer_idx >= prev_starts.len() || layer_idx >= curr_starts.len() {
                    break;
                }
                let prev_base = prev_starts[layer_idx] as i32;
                let curr_base = curr_starts[layer_idx] as i32;

                // 计算该层的每环顶点数
                let cs = interpolate_cross_section_layer(&input.cross_sections, t, layer_idx);
                let verts_per_ring = match cs.shape {
                    CrossSectionShape::Rectangular => 4,
                    CrossSectionShape::Circular => input.ring_segments,
                };

                for j in 0..verts_per_ring {
                    let nxt = (j + 1) % verts_per_ring;
                    let a = prev_base + j as i32;
                    let b = prev_base + nxt as i32;
                    let c = curr_base + j as i32;
                    let d = curr_base + nxt as i32;
                    faces.push(FaceIndices { vertex_indices: vec![a, c, d] });
                    faces.push(FaceIndices { vertex_indices: vec![a, d, b] });
                }

                // 生成 UV
                let u0 = (i - 1) as f64 / (path.len() - 1).max(1) as f64;
                let u1 = i as f64 / (path.len() - 1).max(1) as f64;
                for j in 0..verts_per_ring {
                    let v = j as f64 / verts_per_ring as f64;
                    uvs.push(Point2D { x: u0, y: v });
                    uvs.push(Point2D { x: u1, y: v });
                }
            }
        }
    }

    // 两端加盖
    if let (Some(first_layer_starts), Some(last_layer_starts)) = (ring_starts.first(), ring_starts.last()) {
        for layer_idx in 0..max_layers(&input.cross_sections) {
            if layer_idx >= first_layer_starts.len() || layer_idx >= last_layer_starts.len() {
                break;
            }
            let cs = interpolate_cross_section_layer(&input.cross_sections, 0.0, layer_idx);
            match cs.shape {
                CrossSectionShape::Rectangular => {
                    cap_rectangular_end(first_layer_starts[layer_idx] as i32, &mut faces, true);
                    cap_rectangular_end(last_layer_starts[layer_idx] as i32, &mut faces, false);
                }
                CrossSectionShape::Circular => {
                    // 圆形端盖：以环中心为顶点的扇形
                    let verts_per_ring = input.ring_segments;
                    cap_circular_end(first_layer_starts[layer_idx], verts_per_ring, &mut vertices, &mut normals, &mut faces, true);
                    cap_circular_end(last_layer_starts[layer_idx], verts_per_ring, &mut vertices, &mut normals, &mut faces, false);
                }
            }
        }
    }

    MeshGeometry {
        vertices,
        normals,
        uvs,
        faces,
    }
}

/// 生成矩形截面环（4 个顶点）
fn generate_rectangular_ring(
    pt: &Point3D,
    _forward: &Vector3D,
    right: &Vector3D,
    up: &Vector3D,
    layer: &CrossSectionLayer,
    vertices: &mut Vec<Point3D>,
    normals: &mut Vec<Vector3D>,
) {
    let hw = layer.default_width / 2.0;
    let ht = layer.default_thickness / 2.0;

    // 4 个角点：顺序为 左下、右下、右上、左上（沿环方向）
    let corners = [
        (-hw, -ht),  // 左下
        ( hw, -ht),  // 右下
        ( hw,  ht),  // 右上
        (-hw,  ht),  // 左上
    ];

    for (dx, dy) in &corners {
        vertices.push(Point3D {
            x: pt.x + right.x * dx + up.x * dy,
            y: pt.y + right.y * dx + up.y * dy,
            z: pt.z + right.z * dx + up.z * dy,
        });
        // 法线指向外侧（沿 dx, dy 方向）
        let out_dir = Vector3D {
            x: right.x * dx.signum() + up.x * dy.signum(),
            y: right.y * dx.signum() + up.y * dy.signum(),
            z: right.z * dx.signum() + up.z * dy.signum(),
        };
        let len = (out_dir.x * out_dir.x + out_dir.y * out_dir.y + out_dir.z * out_dir.z).sqrt();
        if len > 1e-10 {
            normals.push(Vector3D {
                x: out_dir.x / len, y: out_dir.y / len, z: out_dir.z / len,
            });
        } else {
            normals.push(Vector3D { x: 0.0, y: 0.0, z: 1.0 });
        }
    }
}

/// 生成圆形截面环
fn generate_circular_ring(
    pt: &Point3D,
    _forward: &Vector3D,
    right: &Vector3D,
    up: &Vector3D,
    layer: &CrossSectionLayer,
    segments: usize,
    vertices: &mut Vec<Point3D>,
    normals: &mut Vec<Vector3D>,
) {
    let radius = layer.default_width / 2.0;
    let segs = segments.max(4);

    for i in 0..segs {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / segs as f64;
        let (sa, ca) = (angle.sin(), angle.cos());

        // 在局部平面中：right 方向为 x，up 方向为 y
        let lx = radius * ca;
        let ly = radius * sa;

        vertices.push(Point3D {
            x: pt.x + right.x * lx + up.x * ly,
            y: pt.y + right.y * lx + up.y * ly,
            z: pt.z + right.z * lx + up.z * ly,
        });

        // 法线 = 径向方向（单位向量）
        let n_len = (lx * lx + ly * ly).sqrt().max(1e-10);
        let nx = right.x * (lx / n_len) + up.x * (ly / n_len);
        let ny = right.y * (lx / n_len) + up.y * (ly / n_len);
        let nz = right.z * (lx / n_len) + up.z * (ly / n_len);
        let n_mag = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-10);
        normals.push(Vector3D { x: nx / n_mag, y: ny / n_mag, z: nz / n_mag });
    }
}

/// 矩形端盖
fn cap_rectangular_end(base: i32, faces: &mut Vec<FaceIndices>, is_start: bool) {
    if is_start {
        // 起点端面（朝向 -forward）
        faces.push(FaceIndices { vertex_indices: vec![base, base + 1, base + 2] });
        faces.push(FaceIndices { vertex_indices: vec![base, base + 2, base + 3] });
    } else {
        // 终点端面（朝向 +forward）
        faces.push(FaceIndices { vertex_indices: vec![base, base + 2, base + 1] });
        faces.push(FaceIndices { vertex_indices: vec![base, base + 3, base + 2] });
    }
}

/// 圆形端盖（扇形三角剖分）
fn cap_circular_end(
    base: usize,
    segments: usize,
    vertices: &mut Vec<Point3D>,
    normals: &mut Vec<Vector3D>,
    faces: &mut Vec<FaceIndices>,
    is_start: bool,
) {
    // 计算环中心（所有顶点的平均值）
    let ring_verts: Vec<Point3D> = vertices[base..base + segments].to_vec();
    if ring_verts.is_empty() {
        return;
    }
    let center = Point3D {
        x: ring_verts.iter().map(|v| v.x).sum::<f64>() / ring_verts.len() as f64,
        y: ring_verts.iter().map(|v| v.y).sum::<f64>() / ring_verts.len() as f64,
        z: ring_verts.iter().map(|v| v.z).sum::<f64>() / ring_verts.len() as f64,
    };

    // 计算端面法线（基于环上三角面）
    let mut nml = Vector3D { x: 0.0, y: 0.0, z: 0.0 };
    if ring_verts.len() >= 3 {
        let e1 = &ring_verts[1];
        let e2 = &ring_verts[2];
        let ex = e1.x - center.x;
        let ey = e1.y - center.y;
        let ez = e1.z - center.z;
        let fx = e2.x - center.x;
        let fy = e2.y - center.y;
        let fz = e2.z - center.z;
        nml.x = ey * fz - ez * fy;
        nml.y = ez * fx - ex * fz;
        nml.z = ex * fy - ey * fx;
        let len = (nml.x * nml.x + nml.y * nml.y + nml.z * nml.z).sqrt().max(1e-10);
        nml.x /= len; nml.y /= len; nml.z /= len;
        if !is_start {
            nml.x = -nml.x; nml.y = -nml.y; nml.z = -nml.z;
        }
    }

    // 添加中心顶点
    let center_idx = vertices.len() as i32;
    vertices.push(center);
    normals.push(nml);

    // 生成扇形三角面
    for i in 0..segments {
        let nxt = (i + 1) % segments;
        let a = base as i32 + i as i32;
        let b = base as i32 + nxt as i32;
        if is_start {
            faces.push(FaceIndices { vertex_indices: vec![a, b, center_idx] });
        } else {
            faces.push(FaceIndices { vertex_indices: vec![a, center_idx, b] });
        }
    }
}

/// 获取所有截面定义中的最大层数
fn max_layers(css: &[CrossSectionAtStation]) -> usize {
    css.iter().map(|cs| cs.layers.len()).max().unwrap_or(0)
}

/// 根据 t 插值指定层的截面参数
fn interpolate_cross_section_layer(css: &[CrossSectionAtStation], t: f64, layer_idx: usize) -> CrossSectionLayer {
    if css.is_empty() {
        return default_layer();
    }

    // 找到 t 所在的区间
    let lo = find_segment(css, t);

    let cs_lo = &css[lo];
    let cs_hi = if lo + 1 < css.len() { &css[lo + 1] } else { &css[lo] };

    let range = cs_hi.station - cs_lo.station;
    let local_t = if range > 1e-10 {
        ((t - cs_lo.station) / range).clamp(0.0, 1.0)
    } else {
        0.0
    };

    let layer_lo = cs_lo.layers.get(layer_idx);
    let layer_hi = cs_hi.layers.get(layer_idx);

    match (layer_lo, layer_hi) {
        (Some(l0), Some(l1)) => CrossSectionLayer {
            name: l0.name.clone(),
            shape: l0.shape,
            default_width: l0.default_width + (l1.default_width - l0.default_width) * local_t,
            default_thickness: l0.default_thickness + (l1.default_thickness - l0.default_thickness) * local_t,
            material_id: l0.material_id.clone(),
            function: l0.function.clone(),
        },
        (Some(l0), None) => l0.clone(),
        (None, Some(l1)) => l1.clone(),
        _ => default_layer(),
    }
}

/// 找到 t 所在的截面段索引
fn find_segment(css: &[CrossSectionAtStation], t: f64) -> usize {
    if t <= css[0].station {
        return 0;
    }
    if t >= css[css.len() - 1].station {
        return css.len() - 1;
    }
    for i in 0..css.len() - 1 {
        if t >= css[i].station && t <= css[i + 1].station {
            return i;
        }
    }
    css.len() - 1
}

fn default_layer() -> CrossSectionLayer {
    CrossSectionLayer {
        name: "default".into(),
        shape: CrossSectionShape::Rectangular,
        default_width: 1.0,
        default_thickness: 0.3,
        material_id: None,
        function: "default".into(),
    }
}

/// 等距采样路径点
fn interpolate_path(points: &[Point3D], segments: usize) -> Vec<Point3D> {
    if points.len() < 2 {
        return points.to_vec();
    }

    // 计算累积路径长度
    let mut lengths = vec![0.0f64];
    for i in 1..points.len() {
        let dx = points[i].x - points[i - 1].x;
        let dy = points[i].y - points[i - 1].y;
        let dz = points[i].z - points[i - 1].z;
        lengths.push(lengths[i - 1] + (dx * dx + dy * dy + dz * dz).sqrt());
    }
    let total = lengths[lengths.len() - 1];
    if total < 1e-10 {
        return points.to_vec();
    }

    let mut result = vec![points[0].clone()];
    for i in 1..=segments {
        let target = (i as f64 / segments as f64) * total;
        let seg_idx = find_segment_by_length(&lengths, target);
        if seg_idx >= points.len() - 1 {
            result.push(points[points.len() - 1].clone());
        } else {
            let seg_len = lengths[seg_idx + 1] - lengths[seg_idx];
            let local_t = if seg_len > 1e-10 {
                (target - lengths[seg_idx]) / seg_len
            } else {
                0.0
            };
            let p = &points[seg_idx];
            let q = &points[seg_idx + 1];
            result.push(Point3D {
                x: p.x + (q.x - p.x) * local_t,
                y: p.y + (q.y - p.y) * local_t,
                z: p.z + (q.z - p.z) * local_t,
            });
        }
    }
    result
}

fn find_segment_by_length(lengths: &[f64], target: f64) -> usize {
    for i in 0..lengths.len() - 1 {
        if target >= lengths[i] && target <= lengths[i + 1] {
            return i;
        }
    }
    lengths.len() - 2
}

/// 计算 Frenet 坐标系（forward, right, up）
fn compute_frenet_frame(path: &[Point3D], idx: usize, _t: f64) -> (Vector3D, Vector3D, Vector3D) {
    let n = path.len();
    let default_forward = Vector3D { x: 1.0, y: 0.0, z: 0.0 };
    let default_right = Vector3D { x: 0.0, y: 1.0, z: 0.0 };
    let default_up = Vector3D { x: 0.0, y: 0.0, z: 1.0 };

    if n < 2 {
        return (default_forward, default_right, default_up);
    }

    // 选择前后点计算切线方向
    let (prev, next) = if idx == 0 {
        (&path[0], &path[1])
    } else if idx >= n - 1 {
        (&path[n - 2], &path[n - 1])
    } else {
        (&path[idx - 1], &path[idx + 1])
    };

    let dx = next.x - prev.x;
    let dy = next.y - prev.y;
    let dz = next.z - prev.z;
    let len = (dx * dx + dy * dy + dz * dz).sqrt();

    let forward = if len > 1e-10 {
        Vector3D { x: dx / len, y: dy / len, z: dz / len }
    } else {
        default_forward
    };

    // Right = cross(forward, world_up)
    let world_up = Vector3D { x: 0.0, y: 0.0, z: 1.0 };

    // 处理 forward 平行于 world_up 的情况
    let f_up_dot = forward.x * world_up.x + forward.y * world_up.y + forward.z * world_up.z;
    let right = if f_up_dot.abs() > 0.999 {
        // forward 与 world_up 平行，使用 world_right 作为替代
        Vector3D { x: 0.0, y: 1.0, z: 0.0 }
    } else {
        let r = cross(&forward, &world_up);
        v3_normalize(&r)
    };

    // up = cross(right, forward) 以确保正交坐标系
    let up = v3_normalize(&cross(&right, &forward));

    (forward, right, up)
}

fn v3_magnitude(v: &Vector3D) -> f64 {
    (v.x * v.x + v.y * v.y + v.z * v.z).sqrt()
}

fn v3_normalize(v: &Vector3D) -> Vector3D {
    let m = v3_magnitude(v);
    if m > 1e-10 {
        Vector3D { x: v.x / m, y: v.y / m, z: v.z / m }
    } else {
        Vector3D { x: 0.0, y: 0.0, z: 1.0 }
    }
}

fn cross(a: &Vector3D, b: &Vector3D) -> Vector3D {
    Vector3D {
        x: a.y * b.z - a.z * b.y,
        y: a.z * b.x - a.x * b.z,
        z: a.x * b.y - a.y * b.x,
    }
}

// ============================================================
// 便捷构建函数
// ============================================================

/// 建造管道（圆形截面）
///
/// - length: 管道长度
/// - radius: 管道半径
/// - segments: 沿路径分段数
pub fn build_pipe(length: f64, radius: f64, segments: usize) -> MeshGeometry {
    let input = LinearComponentInput {
        alignment: vec![
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Point3D { x: length, y: 0.0, z: 0.0 },
        ],
        cross_sections: vec![CrossSectionAtStation {
            station: 0.0,
            layers: vec![CrossSectionLayer {
                name: "Pipe".into(),
                shape: CrossSectionShape::Circular,
                default_width: radius * 2.0,  // 直径
                default_thickness: 0.0,
                material_id: None,
                function: "pipe".into(),
            }],
        }],
        segments,
        ring_segments: 16,
    };
    generate(&input)
}

/// 建造保温管道（双层：内管 + 保温层）
///
/// - length: 管长
/// - inner_radius: 内管半径
/// - insulation_thickness: 保温层厚度
/// - segments: 分段数
pub fn build_insulated_pipe(length: f64, inner_radius: f64, insulation_thickness: f64, segments: usize) -> MeshGeometry {
    let outer_radius = inner_radius + insulation_thickness;
    let input = LinearComponentInput {
        alignment: vec![
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Point3D { x: length, y: 0.0, z: 0.0 },
        ],
        cross_sections: vec![CrossSectionAtStation {
            station: 0.0,
            layers: vec![
                CrossSectionLayer {
                    name: "InnerPipe".into(),
                    shape: CrossSectionShape::Circular,
                    default_width: inner_radius * 2.0,
                    default_thickness: 0.0,
                    material_id: None,
                    function: "structural".into(),
                },
                CrossSectionLayer {
                    name: "Insulation".into(),
                    shape: CrossSectionShape::Circular,
                    default_width: outer_radius * 2.0,
                    default_thickness: 0.0,
                    material_id: None,
                    function: "insulation".into(),
                },
            ],
        }],
        segments,
        ring_segments: 16,
    };
    generate(&input)
}

/// 建造道路（矩形截面，可设置多层路面结构）
///
/// - length: 道路长度
/// - width: 路面宽度
/// - base_thickness: 路基厚度
/// - surface_thickness: 面层厚度
/// - segments: 分段数
pub fn build_road(length: f64, width: f64, base_thickness: f64, surface_thickness: f64, segments: usize) -> MeshGeometry {
    let input = LinearComponentInput {
        alignment: vec![
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Point3D { x: length, y: 0.0, z: 0.0 },
        ],
        cross_sections: vec![CrossSectionAtStation {
            station: 0.0,
            layers: vec![
                CrossSectionLayer {
                    name: "Base".into(),
                    shape: CrossSectionShape::Rectangular,
                    default_width: width,
                    default_thickness: base_thickness,
                    material_id: None,
                    function: "base".into(),
                },
                CrossSectionLayer {
                    name: "Surface".into(),
                    shape: CrossSectionShape::Rectangular,
                    default_width: width,
                    default_thickness: surface_thickness,
                    material_id: None,
                    function: "surface".into(),
                },
            ],
        }],
        segments,
        ring_segments: 16,
    };
    generate(&input)
}

/// 建造梁（矩形截面）
///
/// - length: 梁长
/// - width: 梁宽
/// - depth: 梁高
pub fn build_beam(length: f64, width: f64, depth: f64) -> MeshGeometry {
    let input = LinearComponentInput {
        alignment: vec![
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Point3D { x: length, y: 0.0, z: 0.0 },
        ],
        cross_sections: vec![CrossSectionAtStation {
            station: 0.0,
            layers: vec![CrossSectionLayer {
                name: "Beam".into(),
                shape: CrossSectionShape::Rectangular,
                default_width: width,
                default_thickness: depth,
                material_id: None,
                function: "structural".into(),
            }],
        }],
        segments: 4,
        ring_segments: 16,
    };
    generate(&input)
}

/// 建造变截面梁（两端截面不同）
///
/// - length: 梁长
/// - start_width, start_depth: 起点截面
/// - end_width, end_depth: 终点截面
pub fn build_tapered_beam(length: f64, start_width: f64, start_depth: f64, end_width: f64, end_depth: f64) -> MeshGeometry {
    let input = LinearComponentInput {
        alignment: vec![
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Point3D { x: length, y: 0.0, z: 0.0 },
        ],
        cross_sections: vec![
            CrossSectionAtStation {
                station: 0.0,
                layers: vec![CrossSectionLayer {
                    name: "Beam".into(),
                    shape: CrossSectionShape::Rectangular,
                    default_width: start_width,
                    default_thickness: start_depth,
                    material_id: None,
                    function: "structural".into(),
                }],
            },
            CrossSectionAtStation {
                station: 1.0,
                layers: vec![CrossSectionLayer {
                    name: "Beam".into(),
                    shape: CrossSectionShape::Rectangular,
                    default_width: end_width,
                    default_thickness: end_depth,
                    material_id: None,
                    function: "structural".into(),
                }],
            },
        ],
        segments: 8,
        ring_segments: 16,
    };
    generate(&input)
}

/// 建造特征线/等高线（薄管，圆形小截面，沿 N 点路径）
///
/// 特征线 (FeatureLine) 与等高线 (ContourLine) 是地形/场地线型对象，
/// 用细小的管状体表示路径走向（无实体厚度语义）。
///
/// - alignment: 连续 N 点 3D 路径（至少 2 点）
/// - diameter: 可视管径（默认小值，体现为线型）
/// - segments: 分段数
pub fn build_guide_line(alignment: &[Point3D], diameter: f64, segments: usize) -> MeshGeometry {
    let input = LinearComponentInput {
        alignment: alignment.to_vec(),
        cross_sections: vec![CrossSectionAtStation {
            station: 0.0,
            layers: vec![CrossSectionLayer {
                name: "GuideLine".into(),
                shape: CrossSectionShape::Circular,
                default_width: diameter.max(1e-3),
                default_thickness: 0.0,
                material_id: None,
                function: "guide_line".into(),
            }],
        }],
        segments: segments.max(2),
        ring_segments: 8,
    };
    generate(&input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_straight_pipe() {
        let mesh = build_pipe(10.0, 0.5, 20);
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.faces.is_empty());
        // 圆形管道：每个截面 segments 个顶点
        assert!(mesh.faces.len() > 20);
    }

    #[test]
    fn test_generate_road() {
        let mesh = build_road(50.0, 3.5, 0.2, 0.1, 10);
        assert!(mesh.vertices.len() > 10);
        assert!(mesh.faces.len() > 10);
        // 2 层路面结构应有 8 顶点/截面
    }

    #[test]
    fn test_interpolate_path() {
        let pts = vec![
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Point3D { x: 10.0, y: 0.0, z: 0.0 },
        ];
        let sampled = interpolate_path(&pts, 4);
        assert_eq!(sampled.len(), 5);
        assert!((sampled[2].x - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_curved_path() {
        let pts = vec![
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Point3D { x: 5.0, y: 5.0, z: 0.0 },
            Point3D { x: 10.0, y: 0.0, z: 0.0 },
        ];
        let input = LinearComponentInput {
            alignment: pts,
            segments: 20,
            ..Default::default()
        };
        let mesh = generate(&input);
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.faces.is_empty());
    }

    #[test]
    fn test_variable_cross_section() {
        let input = LinearComponentInput {
            alignment: vec![
                Point3D { x: 0.0, y: 0.0, z: 0.0 },
                Point3D { x: 20.0, y: 0.0, z: 0.0 },
            ],
            cross_sections: vec![
                CrossSectionAtStation {
                    station: 0.0,
                    layers: vec![CrossSectionLayer {
                        name: "Start".into(),
                        shape: CrossSectionShape::Rectangular,
                        default_width: 3.0,
                        default_thickness: 0.3,
                        material_id: None,
                        function: "road".into(),
                    }],
                },
                CrossSectionAtStation {
                    station: 1.0,
                    layers: vec![CrossSectionLayer {
                        name: "End".into(),
                        shape: CrossSectionShape::Rectangular,
                        default_width: 5.0,
                        default_thickness: 0.5,
                        material_id: None,
                        function: "road".into(),
                    }],
                },
            ],
            segments: 20,
            ring_segments: 16,
        };
        let mesh = generate(&input);
        assert!(!mesh.vertices.is_empty());

        // 验证变截面：起点和终点的宽度应不同
        let max_x = mesh.vertices.iter().map(|v| v.x).fold(f64::NEG_INFINITY, f64::max);
        let min_x = mesh.vertices.iter().map(|v| v.x).fold(f64::INFINITY, f64::min);
        // 找到起点截面（x ≈ min_x）的顶点 Y 范围
        let start_ys: Vec<f64> = mesh.vertices.iter()
            .filter(|v| (v.x - min_x).abs() < 0.1)
            .map(|v| v.y).collect();
        let end_ys: Vec<f64> = mesh.vertices.iter()
            .filter(|v| (v.x - max_x).abs() < 0.1)
            .map(|v| v.y).collect();
        if !start_ys.is_empty() && !end_ys.is_empty() {
            let s_max = start_ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let s_min = start_ys.iter().copied().fold(f64::INFINITY, f64::min);
            let e_max = end_ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let e_min = end_ys.iter().copied().fold(f64::INFINITY, f64::min);
            let s_w = s_max - s_min;
            let e_w = e_max - e_min;
            assert!(e_w > s_w, "end width {} should be > start width {}", e_w, s_w);
        }
    }

    #[test]
    fn test_pipe_circular_cross_section() {
        let mesh = build_pipe(5.0, 1.0, 10);
        // 确保生成的是圆形截面：检查所有顶点到中心轴的距离 ≈ radius
        // 取中间截面的顶点
        let mid_x = 2.5;
        let ring_verts: Vec<&Point3D> = mesh.vertices.iter()
            .filter(|v| (v.x - mid_x).abs() < 0.1)
            .collect();
        if ring_verts.len() >= 4 {
            let avg_radius: f64 = ring_verts.iter()
                .map(|v| (v.y * v.y + v.z * v.z).sqrt())
                .sum::<f64>() / ring_verts.len() as f64;
            assert!((avg_radius - 1.0).abs() < 0.2, "pipe radius should be ~1.0, got {}", avg_radius);
        }
    }

    #[test]
    fn test_insulated_pipe() {
        let mesh = build_insulated_pipe(3.0, 0.5, 0.1, 8);
        assert!(!mesh.vertices.is_empty());
        // 双层管道应比单层有更多顶点
        let single = build_pipe(3.0, 0.5, 8);
        assert!(mesh.vertices.len() > single.vertices.len(),
            "insulated pipe should have more vertices than single pipe");
    }

    #[test]
    fn test_tapered_beam() {
        let mesh = build_tapered_beam(4.0, 0.3, 0.5, 0.5, 0.8);
        assert!(!mesh.vertices.is_empty());
        // 验证两端宽度不同
        let max_x = mesh.vertices.iter().map(|v| v.x).fold(f64::NEG_INFINITY, f64::max);
        let min_x = mesh.vertices.iter().map(|v| v.x).fold(f64::INFINITY, f64::min);
        let start_ys: Vec<f64> = mesh.vertices.iter()
            .filter(|v| (v.x - min_x).abs() < 0.1)
            .map(|v| v.y).collect();
        let end_ys: Vec<f64> = mesh.vertices.iter()
            .filter(|v| (v.x - max_x).abs() < 0.1)
            .map(|v| v.y).collect();
        if !start_ys.is_empty() && !end_ys.is_empty() {
            let s_w = start_ys.iter().copied().fold(f64::NEG_INFINITY, f64::max) - start_ys.iter().copied().fold(f64::INFINITY, f64::min);
            let e_w = end_ys.iter().copied().fold(f64::NEG_INFINITY, f64::max) - end_ys.iter().copied().fold(f64::INFINITY, f64::min);
            assert!(e_w > s_w, "end width {} should be > start width {} for tapered beam", e_w, s_w);
        }
    }

    #[test]
    fn test_z_aligned_path() {
        // 测试垂直方向路径（如竖管或桥墩）
        let input = LinearComponentInput {
            alignment: vec![
                Point3D { x: 0.0, y: 0.0, z: 0.0 },
                Point3D { x: 0.0, y: 0.0, z: 10.0 },
            ],
            cross_sections: vec![CrossSectionAtStation {
                station: 0.0,
                layers: vec![CrossSectionLayer {
                    name: "Column".into(),
                    shape: CrossSectionShape::Circular,
                    default_width: 1.0,
                    default_thickness: 0.0,
                    material_id: None,
                    function: "structural".into(),
                }],
            }],
            segments: 10,
            ring_segments: 8,
        };
        let mesh = generate(&input);
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.faces.is_empty());
    }

    #[test]
    fn test_guide_line_n_point_path() {
        // N 点折线路径的特征线：应沿每一段生成管状几何
        let path = vec![
            Point3D { x: 0.0, y: 0.0, z: 0.0 },
            Point3D { x: 5.0, y: 0.0, z: 0.0 },
            Point3D { x: 5.0, y: 5.0, z: 0.0 },
            Point3D { x: 10.0, y: 5.0, z: 0.0 },
        ];
        // 用足够多的分段采样，使某环落在折点 (5,0) 附近
        let mesh = build_guide_line(&path, 0.05, 64);
        assert!(!mesh.vertices.is_empty());
        // 折点附近 (5,0) 应有顶点落在附近
        let has_near = |x: f64, y: f64| mesh.vertices.iter().any(|v| (v.x - x).abs() < 0.3 && (v.y - y).abs() < 0.3);
        assert!(has_near(5.0, 0.0), "corner point (5,0) area should have vertices");
        assert!(has_near(10.0, 5.0), "end point (10,5) area should have vertices");
    }
}