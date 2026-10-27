//! LayerExtrusionGen — 多层拉伸引擎
//!
//! 将路径 + 层结构展开为多个几何基元（Box/Extrusion）。
//! 适用于建筑构件：墙体 (Wall)、楼板 (Slab)、地板 (Floor)、屋面 (Roof)。
//!
//! 原理：
//! - 路径定义中心线，各层沿中心线法向偏移
//! - 每层生成一个带偏移的 Box 或 Extrusion
//! - 最终返回合并的 MeshGeometry（含偏移定位后的顶点）

use omnim_core as c;
use c::{Box, MeshGeometry, Point2D, Point3D, Vector3D};

/// 层定义
#[derive(Debug, Clone)]
pub struct LayerDef {
    /// 层名称（如 "Core", "Finish", "Insulation"）
    pub name: String,
    /// 层厚度（米）
    pub thickness: f64,
    /// 材料 ID
    pub material_id: Option<String>,
    /// 功能描述
    pub function: String,
}

/// LayerExtrusion 输入参数
#[derive(Debug, Clone)]
pub struct LayerExtrusionInput {
    /// 路径点序列（二维平面路径，z 由 height 控制）
    pub path: Vec<Point2D>,
    /// 路径是否封闭（楼板/屋面轮廓）：封闭时末段连接回到起点
    pub is_closed: bool,
    /// 层定义列表（从内到外）
    pub layers: Vec<LayerDef>,
    /// 总高度（米）
    pub height: f64,
    /// 挤出方向（默认 Z 轴向上）
    pub direction: Vector3D,
}

impl Default for LayerExtrusionInput {
    fn default() -> Self {
        Self {
            path: vec![Point2D { x: 0.0, y: 0.0 }, Point2D { x: 5.0, y: 0.0 }],
            is_closed: false,
            layers: vec![LayerDef {
                name: "Core".into(),
                thickness: 0.2,
                material_id: None,
                function: "structural".into(),
            }],
            height: 3.0,
            direction: Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        }
    }
}

/// 生成多层拉伸几何体（返回 MeshGeometry 以确保各层正确定位）
///
/// 为每一层生成一个沿路径的几何体，各层按厚度依次偏移。
/// 路径定义中心线，层沿路径法向方向依次排列：
///   层0（最内）从中心线开始，宽度 = thickness[0]
///   层1  从  thickness[0]/2 + thickness[1]/2 处开始
///   以此类推
///
/// 与旧实现不同（旧实现取路径 bbox 生成单个 Box），本实现将路径
/// 拆分为逐段（segment）基元，每段各自沿段方向定向并偏移，从而支持
/// 连续 N 点路径（墙体/道路走向）与封闭轮廓（楼板/屋面 footprint）。
pub fn generate(input: &LayerExtrusionInput) -> MeshGeometry {
    let mut meshes = Vec::new();
    let mut offset = 0.0;

    for layer in &input.layers {
        let mesh = build_single_layer_mesh(input, layer, offset);
        meshes.push(mesh);
        offset += layer.thickness;
    }

    crate::geometry::merge_meshes(&meshes)
}

/// 为单层构建 mesh，offset 为该层中心相对路径中心线的累积偏移量
///
/// 层沿路径法向堆叠（与段方向垂直），每段生成一个定向 Box。
fn build_single_layer_mesh(input: &LayerExtrusionInput, layer: &LayerDef, cumulative_offset: f64) -> MeshGeometry {
    // 所有层在路径中心线两侧对称排列
    let total_thick: f64 = input.layers.iter().map(|l| l.thickness).sum();
    let total_half = total_thick / 2.0;
    let layer_center_y = cumulative_offset - total_half + layer.thickness / 2.0;

    let meshes: Vec<MeshGeometry> = iter_segments(input).map(|(a, b)| {
        let dx = b.x - a.x;
        let dy = b.y - a.y;
        let len = dx.hypot(dy);
        if len < 1e-9 { return crate::geometry::merge_meshes(&[]); }
        let angle = dy.atan2(dx);
        let cos = angle.cos();
        let sin = angle.sin();

        // 本地 X 方向 = 段方向；本地 Y 方向 = (-sin, cos) 为法向。
        let (verts, norms, faces) = crate::mesh_gen::box_mesh(&Box {
            length: len,
            width: layer.thickness,
            height: input.height,
        });

        let mid_x = (a.x + b.x) / 2.0;
        let mid_y = (a.y + b.y) / 2.0;
        let off_x = -sin * layer_center_y;
        let off_y = cos * layer_center_y;

        // 绕 Z 旋转 angle 使 Box 的 X 轴与段方向对齐，再平移至段中点 + 法向偏移
        let half = angle / 2.0;
        let (sin_h, cos_h) = half.sin_cos();
        let transform = c::Transform {
            translation: Some(Point3D {
                x: mid_x + off_x,
                y: mid_y + off_y,
                z: input.height / 2.0,
            }),
            rotation: Some(c::Rotation { x: 0.0, y: 0.0, z: sin_h, w: cos_h }),
            scale: None,
            matrix: Vec::new(),
        };

        let mesh = MeshGeometry {
            vertices: verts,
            normals: norms,
            uvs: Vec::new(),
            faces,
        };

        crate::geometry::transform_mesh(&mesh, &transform)
    }).collect();

    crate::geometry::merge_meshes(&meshes)
}

/// 迭代路径的连续段；封闭路径时末段回到起点。
fn iter_segments<'a>(input: &'a LayerExtrusionInput) -> impl Iterator<Item = (Point2D, Point2D)> + 'a {
    let n = input.path.len();
    input.path.iter().enumerate().filter_map(move |(i, a)| {
        if i + 1 < n {
            Some((*a, input.path[i + 1]))
        } else if input.is_closed && n > 1 {
            Some((*a, input.path[0]))
        } else {
            None
        }
    })
}

/// 按任意层列表 + 连续路径生成几何体（LayerEditor 后端支撑）
///
/// - path: 平面连续路径（米）
/// - height: 拉伸高度
/// - layers: 层定义列表（从内到外，沿法向堆叠）
pub fn build_layered_path(path: &[Point2D], is_closed: bool, height: f64, layers: &[LayerDef]) -> MeshGeometry {
    let input = LayerExtrusionInput {
        path: path.to_vec(),
        is_closed,
        layers: layers.to_vec(),
        height,
        direction: Vector3D { x: 0.0, y: 0.0, z: 1.0 },
    };
    generate(&input)
}

/// 按任意层列表 + 封闭轮廓生成板式构件（slab/roof/floor footprint）
///
/// 与墙体（层沿法向堆叠）不同，板式构件各层沿 +Z 纵向堆叠：
/// 每一层是封闭轮廓沿垂直方向挤出的棱柱，逐层置于上一层的顶面上。
pub fn build_layered_slab_path(boundary: &[Point2D], layers: &[LayerDef]) -> MeshGeometry {
    let mut meshes: Vec<MeshGeometry> = Vec::new();
    let mut z_offset = 0.0;
    for layer in layers {
        let input = LayerExtrusionInput {
            path: boundary.to_vec(),
            is_closed: true,
            layers: vec![LayerDef {
                thickness: layer.thickness.max(1e-3),
                ..layer.clone()
            }],
            height: layer.thickness.max(1e-3),
            direction: Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        };
        let m = generate(&input);
        let t = c::Transform {
            translation: Some(Point3D { x: 0.0, y: 0.0, z: z_offset }),
            rotation: None,
            scale: None,
            matrix: Vec::new(),
        };
        meshes.push(crate::geometry::transform_mesh(&m, &t));
        z_offset += layer.thickness;
    }
    crate::geometry::merge_meshes(&meshes)
}

/// 构建墙体几何体（便捷方法）
///
/// 生成常见的多层墙体：结构层 + 饰面层（两侧）
/// - length: 墙体长度
/// - height: 墙体高度
/// - core_thickness: 结构层厚度
/// - finish_thickness: 饰面层厚度
pub fn build_wall(length: f64, height: f64, core_thickness: f64, finish_thickness: f64) -> MeshGeometry {
    let mut meshes = Vec::new();

    // 结构层（核心）— 居中
    let (cv, cn, cf) = crate::mesh_gen::box_mesh(&Box { length, width: core_thickness, height });
    let core_mesh = MeshGeometry {
        vertices: cv.iter().map(|v| Point3D {
            x: v.x,
            y: v.y,  // Box 以中心为原点，所以 y 在 [-core_thickness/2, core_thickness/2]
            z: v.z + height / 2.0,
        }).collect(),
        normals: cn,
        uvs: Vec::new(),
        faces: cf,
    };
    meshes.push(core_mesh);

    // 饰面层 A（+Y 侧）
    let finish_offset = core_thickness / 2.0 + finish_thickness / 2.0;
    let (fv, fnml, ff) = crate::mesh_gen::box_mesh(&Box { length, width: finish_thickness, height });
    let finish_a_mesh = MeshGeometry {
        vertices: fv.iter().map(|v| Point3D {
            x: v.x,
            y: v.y + finish_offset,  // 偏移到核心层外侧 +Y
            z: v.z + height / 2.0,
        }).collect(),
        normals: fnml.clone(),
        uvs: Vec::new(),
        faces: ff.clone(),
    };
    meshes.push(finish_a_mesh);

    // 饰面层 B（-Y 侧）
    let finish_b_mesh = MeshGeometry {
        vertices: fv.iter().map(|v| Point3D {
            x: v.x,
            y: v.y - finish_offset,  // 偏移到核心层外侧 -Y
            z: v.z + height / 2.0,
        }).collect(),
        normals: fnml,
        uvs: Vec::new(),
        faces: ff,
    };
    meshes.push(finish_b_mesh);

    crate::geometry::merge_meshes(&meshes)
}

/// 构建楼板/屋面几何体
///
/// - length, width: 平面尺寸
/// - structural_thickness: 结构层厚度
/// - finish_thickness: 饰面层厚度（在 +Z 方向）
pub fn build_slab(length: f64, width: f64, structural_thickness: f64, finish_thickness: f64) -> MeshGeometry {
    let mut meshes = Vec::new();

    // 结构层 — 底部对齐
    let (cv, cn, cf) = crate::mesh_gen::box_mesh(&Box { length, width, height: structural_thickness });
    let struct_mesh = MeshGeometry {
        vertices: cv.iter().map(|v| Point3D {
            x: v.x,
            y: v.y,
            z: v.z + structural_thickness / 2.0,
        }).collect(),
        normals: cn,
        uvs: Vec::new(),
        faces: cf,
    };
    meshes.push(struct_mesh);

    // 饰面层 — 在结构层顶部
    if finish_thickness > 0.0 {
        let (fv, fnml, ff) = crate::mesh_gen::box_mesh(&Box { length, width, height: finish_thickness });
        let finish_mesh = MeshGeometry {
            vertices: fv.iter().map(|v| Point3D {
                x: v.x,
                y: v.y,
                z: v.z + structural_thickness + finish_thickness / 2.0,
            }).collect(),
            normals: fnml,
            uvs: Vec::new(),
            faces: ff,
        };
        meshes.push(finish_mesh);
    }

    crate::geometry::merge_meshes(&meshes)
}

/// 按任意层列表构建墙体几何体（LayerEditor 后端支撑）
///
/// - length: 墙体长度（路径 X 方向）
/// - height: 墙体高度
/// - layers: 层定义列表（从内到外，沿 Y 方向堆叠）
pub fn build_layered_wall(length: f64, height: f64, layers: &[LayerDef]) -> MeshGeometry {
    let input = LayerExtrusionInput {
        path: vec![Point2D { x: 0.0, y: 0.0 }, Point2D { x: length, y: 0.0 }],
        is_closed: false,
        layers: layers.to_vec(),
        height,
        direction: Vector3D { x: 0.0, y: 0.0, z: 1.0 },
    };
    generate(&input)
}

/// 按任意层列表构建楼板/屋面几何体（层沿 +Z 方向堆叠）
///
/// - length, width: 平面尺寸
/// - layers: 层定义列表（自下而上）
pub fn build_layered_slab(length: f64, width: f64, layers: &[LayerDef]) -> MeshGeometry {
    let mut meshes = Vec::new();
    let mut z_offset = 0.0;
    for layer in layers {
        let (v, n, f) = crate::mesh_gen::box_mesh(&Box {
            length,
            width,
            height: layer.thickness,
        });
        let mesh = MeshGeometry {
            vertices: v.iter().map(|pt| Point3D {
                x: pt.x,
                y: pt.y,
                z: pt.z + z_offset + layer.thickness / 2.0,
            }).collect(),
            normals: n,
            uvs: Vec::new(),
            faces: f,
        };
        meshes.push(mesh);
        z_offset += layer.thickness;
    }
    crate::geometry::merge_meshes(&meshes)
}

/// 构建外墙（带保温层）
///
/// - length: 墙体长度
/// - height: 墙体高度
/// - core_thickness: 结构层厚度
/// - insulation_thickness: 保温层厚度
/// - finish_thickness: 内外饰面厚度
pub fn build_exterior_wall(
    length: f64,
    height: f64,
    core_thickness: f64,
    insulation_thickness: f64,
    finish_thickness: f64,
) -> MeshGeometry {
    let layers = vec![
        LayerDef { name: "InnerFinish".into(), thickness: finish_thickness, material_id: None, function: "finish".into() },
        LayerDef { name: "Core".into(), thickness: core_thickness, material_id: None, function: "structural".into() },
        LayerDef { name: "Insulation".into(), thickness: insulation_thickness, material_id: None, function: "insulation".into() },
        LayerDef { name: "OuterFinish".into(), thickness: finish_thickness, material_id: None, function: "finish".into() },
    ];

    let input = LayerExtrusionInput {
        path: vec![Point2D { x: 0.0, y: 0.0 }, Point2D { x: length, y: 0.0 }],
        is_closed: false,
        layers,
        height,
        direction: Vector3D { x: 0.0, y: 0.0, z: 1.0 },
    };

    generate(&input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_simple_wall() {
        let input = LayerExtrusionInput::default();
        let mesh = generate(&input);
        assert!(!mesh.vertices.is_empty());
        assert!(!mesh.faces.is_empty());
    }

    #[test]
    fn test_build_wall_with_layers() {
        let mesh = build_wall(5.0, 3.0, 0.2, 0.02);
        // 3 层合并：core + 2 finishes，应有足够多的顶点
        assert!(mesh.vertices.len() >= 24); // 3 boxes × 8 vertices
        // 验证总宽度 = core + 2×finish = 0.24
        let max_y = mesh.vertices.iter().map(|v| v.y).fold(f64::NEG_INFINITY, f64::max);
        let min_y = mesh.vertices.iter().map(|v| v.y).fold(f64::INFINITY, f64::min);
        let total_width = max_y - min_y;
        assert!((total_width - 0.24).abs() < 0.001, "expected width 0.24, got {}", total_width);
        // 验证总长度和总高度
        let max_x = mesh.vertices.iter().map(|v| v.x).fold(f64::NEG_INFINITY, f64::max);
        let min_x = mesh.vertices.iter().map(|v| v.x).fold(f64::INFINITY, f64::min);
        let max_z = mesh.vertices.iter().map(|v| v.z).fold(f64::NEG_INFINITY, f64::max);
        let min_z = mesh.vertices.iter().map(|v| v.z).fold(f64::INFINITY, f64::min);
        let total_length = max_x - min_x;
        let total_height = max_z - min_z;
        assert!((total_length - 5.0).abs() < 0.001, "expected length 5.0, got {}", total_length);
        assert!((total_height - 3.0).abs() < 0.001, "expected height 3.0, got {}", total_height);
        eprintln!("[CHECK] build_wall(5,3,0.2,0.02): x[{:.3},{:.3}] y[{:.3},{:.3}] z[{:.3},{:.3}] = {:.3}x{:.3}x{:.3}",
            min_x, max_x, min_y, max_y, min_z, max_z, total_length, total_width, total_height);
    }

    #[test]
    fn test_build_slab() {
        let mesh = build_slab(6.0, 4.0, 0.15, 0.05);
        assert!(mesh.vertices.len() >= 16); // 2 boxes × 8 vertices

        // 验证总厚度 = structural + finish = 0.20
        let max_z = mesh.vertices.iter().map(|v| v.z).fold(f64::NEG_INFINITY, f64::max);
        let min_z = mesh.vertices.iter().map(|v| v.z).fold(f64::INFINITY, f64::min);
        let total_thick = max_z - min_z;
        assert!((total_thick - 0.20).abs() < 0.001, "expected thickness 0.20, got {}", total_thick);
    }

    #[test]
    fn test_multilayer_custom_path() {
        let input = LayerExtrusionInput {
            path: vec![
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: 3.0, y: 0.0 },
                Point2D { x: 3.0, y: 2.0 },
            ],
            is_closed: false,
            layers: vec![
                LayerDef { name: "Core".into(), thickness: 0.2, material_id: None, function: "structure".into() },
                LayerDef { name: "Cladding".into(), thickness: 0.05, material_id: None, function: "finish".into() },
            ],
            height: 4.0,
            direction: Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        };
        let mesh = generate(&input);
        assert!(!mesh.vertices.is_empty());
        // 2 层，应有合理数量的顶点
        assert!(mesh.vertices.len() >= 16);
    }

    #[test]
    fn test_exterior_wall() {
        let mesh = build_exterior_wall(4.0, 2.8, 0.2, 0.08, 0.015);
        assert!(!mesh.vertices.is_empty());
        // 4 层：inner finish + core + insulation + outer finish
        let max_y = mesh.vertices.iter().map(|v| v.y).fold(f64::NEG_INFINITY, f64::max);
        let min_y = mesh.vertices.iter().map(|v| v.y).fold(f64::INFINITY, f64::min);
        let expected = 0.015 + 0.2 + 0.08 + 0.015;
        let actual = max_y - min_y;
        assert!((actual - expected).abs() < 0.001,
            "exterior wall total thickness: expected {:.3}, got {:.3}", expected, actual);
    }

    #[test]
    fn test_wall_positions_not_overlapping() {
        // 确保饰面层的 Y 范围不重叠
        let mesh = build_wall(3.0, 2.4, 0.2, 0.02);
        // 所有顶点 Y 值应分布在三个分散的区间内
        let mut y_vals: Vec<f64> = mesh.vertices.iter().map(|v| v.y).collect();
        y_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        // 检查没有大量顶点在 0 附近（如果是重叠的，大量顶点会在原点附近密集分布）
        let near_zero = y_vals.iter().filter(|y| y.abs() < 0.001).count();
        // Box mesh 每层 8 个顶点，如果有大量顶点在 0 附近说明重叠
        assert!(near_zero <= 8, "too many vertices near zero: {}, layers are overlapping", near_zero);
    }

    #[test]
    fn test_path_loft_follows_axis_l_shape() {
        // 连续 N 点路径（L 形）：两段都应被挤出，且覆盖 (0,0)->(3,0)->(3,2)
        let layers = vec![LayerDef { name: "Core".into(), thickness: 0.2, material_id: None, function: "structure".into() }];
        let mesh = build_layered_path(
            &[Point2D { x: 0.0, y: 0.0 }, Point2D { x: 3.0, y: 0.0 }, Point2D { x: 3.0, y: 2.0 }],
            false, 3.0, &layers,
        );
        // 段1 沿 X (0..3)，段2 沿 Y (0..2)：检验各段端点区域存在顶点
        let has_around = |x: f64, y: f64| mesh.vertices.iter().any(|v| (v.x - x).abs() < 0.4 && (v.y - y).abs() < 0.4);
        assert!(has_around(0.1, 0.0), "segment 1 start area should have vertices");
        assert!(has_around(2.9, 0.0), "segment 1 end area should have vertices");
        assert!(has_around(3.0, 0.1), "segment 2 start area should have vertices");
        assert!(has_around(3.0, 1.9), "segment 2 end area should have vertices");
        assert!(mesh.vertices.len() >= 16);
    }

    #[test]
    fn test_closed_polygon_prism() {
        // 封闭轮廓（矩形楼板）：末段回到起点形成一个环
        let layers = vec![LayerDef { name: "Core".into(), thickness: 0.2, material_id: None, function: "structure".into() }];
        let mesh = build_layered_path(
            &[Point2D { x: 0.0, y: 0.0 }, Point2D { x: 4.0, y: 0.0 }, Point2D { x: 4.0, y: 3.0 }, Point2D { x: 0.0, y: 3.0 }],
            true, 0.3, &layers,
        );
        // 4 段闭合环，z 高度约 0.3
        assert!(mesh.vertices.len() >= 4 * 8);
        let max_z = mesh.vertices.iter().map(|v| v.z).fold(f64::NEG_INFINITY, f64::max);
        let min_z = mesh.vertices.iter().map(|v| v.z).fold(f64::INFINITY, f64::min);
        assert!((max_z - min_z - 0.3).abs() < 0.001);
        // 角落 (0,0) 附近和 (4,0) 附近都应有顶点（包括闭合环段）
        let has_near = |x: f64, y: f64| mesh.vertices.iter().any(|v| (v.x - x).abs() < 0.4 && (v.y - y).abs() < 0.4);
        assert!(has_near(0.0, 0.0));
        assert!(has_near(4.0, 0.0));
    }
}