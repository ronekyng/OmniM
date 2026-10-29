//! LayerExtrusionGen — Surface 家族的生成引擎（DATA_MODEL §1/§6.2）
//!
//! 将路径 + 层结构展开为多个几何基元（Box/Extrusion）。
//! 适用于建筑构件：墙体 (Wall)、楼板 (Slab)、地板 (Floor)、屋面 (Roof)。
//!
//! **授权边界（§1 授权矩阵 + §6.4 边界注记）**：本模块只承载 Surface 类的
//! `panel`（开放中心线）与 `region`（封闭轮廓）两种 Form。禁止把其它类的
//! 语义带进来——DN/弯头/地形目标属于 Linear 家族（linear_component.rs），
//! 布尔/自由雕塑属于 Body 家族。跨类组合一律先过 `gen_rule::matrix::authorize`，
//! 矩阵未授权的组合直接 400。
//!
//! 底层纯几何（分段/共线合并/校验）位于 `path_geom.rs`，不带任何类语义。
//!
//! 原理：
//! - 路径定义中心线，各层沿中心线法向偏移
//! - 每层生成一个带偏移的 Box 或 Extrusion
//! - 最终返回合并的 MeshGeometry（含偏移定位后的顶点）

use c::{Box, MeshGeometry, Point2D, Point3D, Vector3D};
use omnim_core as c;

/// 层定义
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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
    /// 基准高程（米）：几何体 z 范围为 [base_z, base_z + height]。
    /// B1：保留拾取点的 z，不再静默丢弃（默认 0 保持旧行为）。
    pub base_z: f64,
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
            direction: Vector3D {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            base_z: 0.0,
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
fn build_single_layer_mesh(
    input: &LayerExtrusionInput,
    layer: &LayerDef,
    cumulative_offset: f64,
) -> MeshGeometry {
    // 所有层在路径中心线两侧对称排列
    let total_thick: f64 = input.layers.iter().map(|l| l.thickness).sum();
    let total_half = total_thick / 2.0;
    let layer_center_y = cumulative_offset - total_half + layer.thickness / 2.0;

    // C1 转角斜接：先收集段序列，计算每个端点处与相邻段的转角，
    // 在转角两侧同时延长矩形端部，使墙面在拐角处无缝衔接
    // （不斜接时两段矩形在弯折处重叠/留缺口）。
    let segs: Vec<(Point2D, Point2D)> = iter_segments(input).collect();
    let directions: Vec<(f64, f64)> = segs
        .iter()
        .map(|(a, b)| {
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let len = dx.hypot(dy);
            if len < 1e-9 {
                (1.0, 0.0)
            } else {
                (dx / len, dy / len)
            }
        })
        .collect();

    let meshes: Vec<MeshGeometry> = segs
        .iter()
        .enumerate()
        .map(|(i, &(a, b))| {
            let dx = b.x - a.x;
            let dy = b.y - a.y;
            let len = dx.hypot(dy);
            if len < 1e-9 {
                return crate::geometry::merge_meshes(&[]);
            }
            let angle = dy.atan2(dx);
            let sin = angle.sin();
            let cos = angle.cos();

            // C1：起点端与前一段的转角延伸量，终点端与后一段的转角延伸量
            let ext_start = if i > 0 {
                miter_extension(
                    total_thick,
                    turn_angle(&directions[i - 1], &directions[i]),
                    len.min(seg_len(&segs[i - 1].0, &segs[i - 1].1)),
                )
            } else {
                0.0
            };
            let ext_end = if i + 1 < segs.len() {
                miter_extension(
                    total_thick,
                    turn_angle(&directions[i], &directions[i + 1]),
                    len.min(seg_len(&segs[i + 1].0, &segs[i + 1].1)),
                )
            } else {
                0.0
            };

            let (verts, norms, faces) = crate::mesh_gen::box_mesh(&Box {
                length: len + ext_start + ext_end,
                width: layer.thickness,
                height: input.height,
            });

            // 延伸后段中点沿段方向偏移 (ext_end - ext_start) / 2
            let shift = (ext_end - ext_start) / 2.0;
            let mid_x = (a.x + b.x) / 2.0 + cos * shift;
            let mid_y = (a.y + b.y) / 2.0 + sin * shift;
            let off_x = -sin * layer_center_y;
            let off_y = cos * layer_center_y;

            // 绕 Z 旋转 angle 使 Box 的 X 轴与段方向对齐，再平移至段中点 + 法向偏移；
            // z 平移包含基准高程 base_z（B1：保留拾取高程，不再落到 z=0）
            let half = angle / 2.0;
            let (sin_h, cos_h) = half.sin_cos();
            let transform = c::Transform {
                translation: Some(Point3D {
                    x: mid_x + off_x,
                    y: mid_y + off_y,
                    z: input.base_z + input.height / 2.0,
                }),
                rotation: Some(c::Rotation {
                    x: 0.0,
                    y: 0.0,
                    z: sin_h,
                    w: cos_h,
                }),
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
        })
        .collect();

    crate::geometry::merge_meshes(&meshes)
}

/// C1 计算两相邻段方向在拐点处的转角（0..=π）
fn turn_angle(d0: &(f64, f64), d1: &(f64, f64)) -> f64 {
    let dot = d0.0 * d1.0 + d0.1 * d1.1;
    let cross = d0.0 * d1.1 - d0.1 * d1.0;
    cross.atan2(dot).abs()
}

fn seg_len(a: &Point2D, b: &Point2D) -> f64 {
    (b.x - a.x).hypot(b.y - a.y)
}

/// C1 转角斜接延伸量：转角 φ 时端部延长 e = (t/2)·tan(φ/2)，使两段矩形
/// 恰好延伸到外角交界，90°/45° 等常见 L 形转角精确无缝隙。
///
/// 局限（务实实现）：
/// - 以整墙总厚度 t 计算延伸量，各层使用同一数值，多层墙的内侧层在拐角处
///   可能出现少量重叠（重合面合并后不影响渲染，但并非精确逐层 miter）；
/// - 锐转角（φ ≥ 135°，如 U 形回折）tan(φ/2) 发散，延伸量截断为
///   min(相邻短边长度 × 0.45, t × 2)，此时拐角可能留细小缝隙；
///   精确覆盖需按角平分线做真正的 miter 放样。
fn miter_extension(total_thick: f64, turn: f64, adjacent_len: f64) -> f64 {
    if turn < 1e-6 || adjacent_len < 1e-9 {
        return 0.0;
    }
    let cap = (adjacent_len * 0.45).min(total_thick * 2.0);
    if turn >= std::f64::consts::FRAC_PI_2 + std::f64::consts::FRAC_PI_4 {
        // ≥135°：斜接长度发散，直接取上限
        cap
    } else {
        ((total_thick / 2.0) * (turn / 2.0).tan()).min(cap)
    }
}

/// 迭代路径的连续段；封闭路径时末段回到起点。
fn iter_segments<'a>(
    input: &'a LayerExtrusionInput,
) -> impl Iterator<Item = (Point2D, Point2D)> + 'a {
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
pub fn build_layered_path(
    path: &[Point2D],
    is_closed: bool,
    height: f64,
    layers: &[LayerDef],
) -> MeshGeometry {
    let input = LayerExtrusionInput {
        path: path.to_vec(),
        is_closed,
        layers: layers.to_vec(),
        height,
        direction: Vector3D {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        base_z: 0.0,
    };
    generate(&input)
}

/// `build_layered_path` 的带高程版本（B1）：`base_z` 为基准高程，
/// 几何体 z 范围为 [base_z, base_z + height]，不再落到 z=0。
pub fn build_layered_path_elevated(
    path: &[Point2D],
    is_closed: bool,
    height: f64,
    base_z: f64,
    layers: &[LayerDef],
) -> MeshGeometry {
    let input = LayerExtrusionInput {
        path: path.to_vec(),
        is_closed,
        layers: layers.to_vec(),
        height,
        direction: Vector3D {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        base_z,
    };
    generate(&input)
}

/// 按任意层列表 + 封闭轮廓生成板式构件（slab/roof/floor footprint）
///
/// 与墙体（层沿法向堆叠）不同，板式构件各层沿 +Z 纵向堆叠：
/// 每一层是封闭轮廓沿垂直方向挤出的棱柱，逐层置于上一层的顶面上。
pub fn build_layered_slab_path(boundary: &[Point2D], layers: &[LayerDef]) -> MeshGeometry {
    build_layered_slab_path_elevated(boundary, layers, 0.0)
}

/// `build_layered_slab_path` 的带高程版本（B1）：`base_z` 为最底层底面高程，
/// 通常取轮廓第一个拾取点的 z。
pub fn build_layered_slab_path_elevated(
    boundary: &[Point2D],
    layers: &[LayerDef],
    base_z: f64,
) -> MeshGeometry {
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
            direction: Vector3D {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            base_z: base_z + z_offset,
        };
        let m = generate(&input);
        meshes.push(m);
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
pub fn build_wall(
    length: f64,
    height: f64,
    core_thickness: f64,
    finish_thickness: f64,
) -> MeshGeometry {
    let mut meshes = Vec::new();

    // 结构层（核心）— 居中
    let (cv, cn, cf) = crate::mesh_gen::box_mesh(&Box {
        length,
        width: core_thickness,
        height,
    });
    let core_mesh = MeshGeometry {
        vertices: cv
            .iter()
            .map(|v| Point3D {
                x: v.x,
                y: v.y, // Box 以中心为原点，所以 y 在 [-core_thickness/2, core_thickness/2]
                z: v.z + height / 2.0,
            })
            .collect(),
        normals: cn,
        uvs: Vec::new(),
        faces: cf,
    };
    meshes.push(core_mesh);

    // 饰面层 A（+Y 侧）
    let finish_offset = core_thickness / 2.0 + finish_thickness / 2.0;
    let (fv, fnml, ff) = crate::mesh_gen::box_mesh(&Box {
        length,
        width: finish_thickness,
        height,
    });
    let finish_a_mesh = MeshGeometry {
        vertices: fv
            .iter()
            .map(|v| Point3D {
                x: v.x,
                y: v.y + finish_offset, // 偏移到核心层外侧 +Y
                z: v.z + height / 2.0,
            })
            .collect(),
        normals: fnml.clone(),
        uvs: Vec::new(),
        faces: ff.clone(),
    };
    meshes.push(finish_a_mesh);

    // 饰面层 B（-Y 侧）
    let finish_b_mesh = MeshGeometry {
        vertices: fv
            .iter()
            .map(|v| Point3D {
                x: v.x,
                y: v.y - finish_offset, // 偏移到核心层外侧 -Y
                z: v.z + height / 2.0,
            })
            .collect(),
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
pub fn build_slab(
    length: f64,
    width: f64,
    structural_thickness: f64,
    finish_thickness: f64,
) -> MeshGeometry {
    let mut meshes = Vec::new();

    // 结构层 — 底部对齐
    let (cv, cn, cf) = crate::mesh_gen::box_mesh(&Box {
        length,
        width,
        height: structural_thickness,
    });
    let struct_mesh = MeshGeometry {
        vertices: cv
            .iter()
            .map(|v| Point3D {
                x: v.x,
                y: v.y,
                z: v.z + structural_thickness / 2.0,
            })
            .collect(),
        normals: cn,
        uvs: Vec::new(),
        faces: cf,
    };
    meshes.push(struct_mesh);

    // 饰面层 — 在结构层顶部
    if finish_thickness > 0.0 {
        let (fv, fnml, ff) = crate::mesh_gen::box_mesh(&Box {
            length,
            width,
            height: finish_thickness,
        });
        let finish_mesh = MeshGeometry {
            vertices: fv
                .iter()
                .map(|v| Point3D {
                    x: v.x,
                    y: v.y,
                    z: v.z + structural_thickness + finish_thickness / 2.0,
                })
                .collect(),
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
        direction: Vector3D {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        base_z: 0.0,
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
            vertices: v
                .iter()
                .map(|pt| Point3D {
                    x: pt.x,
                    y: pt.y,
                    z: pt.z + z_offset + layer.thickness / 2.0,
                })
                .collect(),
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
        LayerDef {
            name: "InnerFinish".into(),
            thickness: finish_thickness,
            material_id: None,
            function: "finish".into(),
        },
        LayerDef {
            name: "Core".into(),
            thickness: core_thickness,
            material_id: None,
            function: "structural".into(),
        },
        LayerDef {
            name: "Insulation".into(),
            thickness: insulation_thickness,
            material_id: None,
            function: "insulation".into(),
        },
        LayerDef {
            name: "OuterFinish".into(),
            thickness: finish_thickness,
            material_id: None,
            function: "finish".into(),
        },
    ];

    let input = LayerExtrusionInput {
        path: vec![Point2D { x: 0.0, y: 0.0 }, Point2D { x: length, y: 0.0 }],
        is_closed: false,
        layers,
        height,
        direction: Vector3D {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        },
        base_z: 0.0,
    };

    generate(&input)
}

/// B2 两点轮廓屋面（profile roof）
///
/// - start/end: 屋面跨度两端点的平面坐标（米）。跨度方向为本地 X 轴；
/// - width: 垂直于跨度方向的延伸宽度（米），width ≤ 0 时回落为 1.0；
/// - layers: 自下而上沿层法向（绕跨度旋转后的竖向）堆叠，层厚逐层累加；
/// - base_z: 屋面底面基准高程（B1，取第一拾取点的 z）。
///
/// 轮廓本身是矩形截面（无坡度造型），坡度/朝向由调用点处既有的段角度
/// Transform（绕 Z 轴按段方向旋转）负责；变换后 z 平移包含 base_z。
pub fn build_layered_roof_profile(
    start: [f64; 2],
    end: [f64; 2],
    width: f64,
    layers: &[LayerDef],
    base_z: f64,
) -> MeshGeometry {
    let width = if width <= 0.0 { 1.0 } else { width };
    let dx = end[0] - start[0];
    let dy = end[1] - start[1];
    let len = dx.hypot(dy);
    if len < 1e-9 {
        return crate::geometry::merge_meshes(&[]);
    }
    let angle = dy.atan2(dx);
    let mut meshes: Vec<MeshGeometry> = Vec::new();
    let mut z_offset = 0.0;
    for layer in layers {
        let (v, n, f) = crate::mesh_gen::box_mesh(&Box {
            length: len,
            width,
            height: layer.thickness.max(1e-3),
        });
        let mesh = MeshGeometry {
            vertices: v
                .iter()
                .map(|pt| Point3D {
                    x: pt.x,
                    y: pt.y,
                    z: pt.z + base_z + z_offset + layer.thickness.max(1e-3) / 2.0,
                })
                .collect(),
            normals: n,
            uvs: Vec::new(),
            faces: f,
        };
        meshes.push(mesh);
        z_offset += layer.thickness;
    }
    let mut mesh = crate::geometry::merge_meshes(&meshes);

    // 绕 Z 按段角度旋转并平移到跨度中点（与调用点既有 Transform 约定一致；
    // z 已按 base_z 堆叠，平移 z = 0 保持層序）
    let half = angle / 2.0;
    let (sin_h, cos_h) = half.sin_cos();
    let t = c::Transform {
        translation: Some(Point3D {
            x: (start[0] + end[0]) / 2.0,
            y: (start[1] + end[1]) / 2.0,
            z: 0.0,
        }),
        rotation: Some(c::Rotation {
            x: 0.0,
            y: 0.0,
            z: sin_h,
            w: cos_h,
        }),
        scale: None,
        matrix: Vec::new(),
    };
    mesh = crate::geometry::transform_mesh(&mesh, &t);
    mesh
}

/// C2 轮廓校验：封闭轮廓（楼板/屋面/地板 footprint）有效性检查。
///
/// 检查项：
/// - 至少 3 个点（重复点/零长度边视为无效）；
/// - 不共线（存在非零叉积）；
/// - 鞋带公式面积 > ε；
/// - 自相交检测（O(n²) 逐对边判定，n 为轮廓点数，规模很小可接受）。
pub fn validate_boundary(boundary: &[Point2D]) -> Result<(), String> {
    let n = boundary.len();
    if n < 3 {
        return Err(format!("boundary requires at least 3 points, got {}", n));
    }
    for i in 0..n {
        let a = &boundary[i];
        let b = &boundary[(i + 1) % n];
        if seg_len(a, b) < 1e-9 {
            return Err(format!("boundary has a zero-length edge at point {}", i));
        }
    }
    // 共线检查：任意相邻边叉积均 ≈ 0 → 退化为线段
    if boundary.windows(3).all(|w| {
        let (a, b, c) = (&w[0], &w[1], &w[2]);
        let cr = (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x);
        cr.abs() < 1e-9
    }) {
        return Err("boundary points are collinear".into());
    }
    // 鞋带面积
    let area2: f64 = boundary
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let q = &boundary[(i + 1) % n];
            p.x * q.y - q.x * p.y
        })
        .sum();
    if area2.abs() < 1e-6 {
        return Err(format!(
            "boundary area is degenerate (|area| = {:.6} m²)",
            (area2 / 2.0).abs()
        ));
    }
    // 自相交检测（正确相交 = 交点在两线段内部，共享端点的相邻边除外）
    let edges: Vec<(&Point2D, &Point2D)> = (0..n)
        .map(|i| (&boundary[i], &boundary[(i + 1) % n]))
        .collect();
    for i in 0..n {
        for j in (i + 1)..n {
            if j == i || (i == 0 && j == n - 1) || j == i + 1 {
                continue; // 相邻边共享端点，允许
            }
            if segments_properly_intersect(edges[i].0, edges[i].1, edges[j].0, edges[j].1) {
                return Err(format!(
                    "boundary is self-intersecting at edges {} and {}",
                    i, j
                ));
            }
        }
    }
    Ok(())
}

/// 线段严格相交判定（交点不落在任一线段端点处）
fn segments_properly_intersect(p1: &Point2D, p2: &Point2D, p3: &Point2D, p4: &Point2D) -> bool {
    fn orient(a: &Point2D, b: &Point2D, c: &Point2D) -> f64 {
        (b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x)
    }
    let d1 = orient(p1, p2, p3);
    let d2 = orient(p1, p2, p4);
    let d3 = orient(p3, p4, p1);
    let d4 = orient(p3, p4, p2);
    ((d1 > 1e-9 && d2 < -1e-9) || (d1 < -1e-9 && d2 > 1e-9))
        && ((d3 > 1e-9 && d4 < -1e-9) || (d3 < -1e-9 && d4 > 1e-9))
}

/// C3 共线合并（兼容入口）：实现已下沉到无类语义的 `path_geom::merge_collinear`。
pub fn merge_collinear(points: &[Point2D]) -> Vec<Point2D> {
    crate::gen_rule::path_geom::merge_collinear(points)
}

/// 屋面各层与跨度两端点在分段后共线退化检查（见 merge_collinear 文档）。
///
/// 注意：roof profile 分支要求 start/end 两点在平面投影上互异，
/// 由服务端在调用前保证。

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
        let max_y = mesh
            .vertices
            .iter()
            .map(|v| v.y)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_y = mesh
            .vertices
            .iter()
            .map(|v| v.y)
            .fold(f64::INFINITY, f64::min);
        let total_width = max_y - min_y;
        assert!(
            (total_width - 0.24).abs() < 0.001,
            "expected width 0.24, got {}",
            total_width
        );
        // 验证总长度和总高度
        let max_x = mesh
            .vertices
            .iter()
            .map(|v| v.x)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_x = mesh
            .vertices
            .iter()
            .map(|v| v.x)
            .fold(f64::INFINITY, f64::min);
        let max_z = mesh
            .vertices
            .iter()
            .map(|v| v.z)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_z = mesh
            .vertices
            .iter()
            .map(|v| v.z)
            .fold(f64::INFINITY, f64::min);
        let total_length = max_x - min_x;
        let total_height = max_z - min_z;
        assert!(
            (total_length - 5.0).abs() < 0.001,
            "expected length 5.0, got {}",
            total_length
        );
        assert!(
            (total_height - 3.0).abs() < 0.001,
            "expected height 3.0, got {}",
            total_height
        );
        eprintln!("[CHECK] build_wall(5,3,0.2,0.02): x[{:.3},{:.3}] y[{:.3},{:.3}] z[{:.3},{:.3}] = {:.3}x{:.3}x{:.3}",
            min_x, max_x, min_y, max_y, min_z, max_z, total_length, total_width, total_height);
    }

    #[test]
    fn test_build_slab() {
        let mesh = build_slab(6.0, 4.0, 0.15, 0.05);
        assert!(mesh.vertices.len() >= 16); // 2 boxes × 8 vertices

        // 验证总厚度 = structural + finish = 0.20
        let max_z = mesh
            .vertices
            .iter()
            .map(|v| v.z)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_z = mesh
            .vertices
            .iter()
            .map(|v| v.z)
            .fold(f64::INFINITY, f64::min);
        let total_thick = max_z - min_z;
        assert!(
            (total_thick - 0.20).abs() < 0.001,
            "expected thickness 0.20, got {}",
            total_thick
        );
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
                LayerDef {
                    name: "Core".into(),
                    thickness: 0.2,
                    material_id: None,
                    function: "structure".into(),
                },
                LayerDef {
                    name: "Cladding".into(),
                    thickness: 0.05,
                    material_id: None,
                    function: "finish".into(),
                },
            ],
            height: 4.0,
            direction: Vector3D {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            base_z: 0.0,
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
        let max_y = mesh
            .vertices
            .iter()
            .map(|v| v.y)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_y = mesh
            .vertices
            .iter()
            .map(|v| v.y)
            .fold(f64::INFINITY, f64::min);
        let expected = 0.015 + 0.2 + 0.08 + 0.015;
        let actual = max_y - min_y;
        assert!(
            (actual - expected).abs() < 0.001,
            "exterior wall total thickness: expected {:.3}, got {:.3}",
            expected,
            actual
        );
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
        assert!(
            near_zero <= 8,
            "too many vertices near zero: {}, layers are overlapping",
            near_zero
        );
    }

    #[test]
    fn test_path_loft_follows_axis_l_shape() {
        // 连续 N 点路径（L 形）：两段都应被挤出，且覆盖 (0,0)->(3,0)->(3,2)
        let layers = vec![LayerDef {
            name: "Core".into(),
            thickness: 0.2,
            material_id: None,
            function: "structure".into(),
        }];
        let mesh = build_layered_path(
            &[
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: 3.0, y: 0.0 },
                Point2D { x: 3.0, y: 2.0 },
            ],
            false,
            3.0,
            &layers,
        );
        // 段1 沿 X (0..3)，段2 沿 Y (0..2)：检验各段端点区域存在顶点
        let has_around = |x: f64, y: f64| {
            mesh.vertices
                .iter()
                .any(|v| (v.x - x).abs() < 0.4 && (v.y - y).abs() < 0.4)
        };
        assert!(
            has_around(0.1, 0.0),
            "segment 1 start area should have vertices"
        );
        assert!(
            has_around(2.9, 0.0),
            "segment 1 end area should have vertices"
        );
        assert!(
            has_around(3.0, 0.1),
            "segment 2 start area should have vertices"
        );
        assert!(
            has_around(3.0, 1.9),
            "segment 2 end area should have vertices"
        );
        assert!(mesh.vertices.len() >= 16);
    }

    #[test]
    fn test_closed_polygon_prism() {
        // 封闭轮廓（矩形楼板）：末段回到起点形成一个环
        let layers = vec![LayerDef {
            name: "Core".into(),
            thickness: 0.2,
            material_id: None,
            function: "structure".into(),
        }];
        let mesh = build_layered_path(
            &[
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: 4.0, y: 0.0 },
                Point2D { x: 4.0, y: 3.0 },
                Point2D { x: 0.0, y: 3.0 },
            ],
            true,
            0.3,
            &layers,
        );
        // 4 段闭合环，z 高度约 0.3
        assert!(mesh.vertices.len() >= 4 * 8);
        let max_z = mesh
            .vertices
            .iter()
            .map(|v| v.z)
            .fold(f64::NEG_INFINITY, f64::max);
        let min_z = mesh
            .vertices
            .iter()
            .map(|v| v.z)
            .fold(f64::INFINITY, f64::min);
        assert!((max_z - min_z - 0.3).abs() < 0.001);
        // 角落 (0,0) 附近和 (4,0) 附近都应有顶点（包括闭合环段）
        let has_near = |x: f64, y: f64| {
            mesh.vertices
                .iter()
                .any(|v| (v.x - x).abs() < 0.4 && (v.y - y).abs() < 0.4)
        };
        assert!(has_near(0.0, 0.0));
        assert!(has_near(4.0, 0.0));
    }

    #[test]
    fn test_base_z_preserved_for_path() {
        // B1：路径拾取点的 z 不再被丢弃——所有 z 相等时，几何底面等于该高程
        let layers = vec![LayerDef {
            name: "Core".into(),
            thickness: 0.2,
            material_id: None,
            function: "structure".into(),
        }];
        let mesh = build_layered_path_elevated(
            &[Point2D { x: 0.0, y: 0.0 }, Point2D { x: 4.0, y: 0.0 }],
            false,
            3.0,
            2.5,
            &layers,
        );
        let min_z = mesh
            .vertices
            .iter()
            .map(|v| v.z)
            .fold(f64::INFINITY, f64::min);
        let max_z = mesh
            .vertices
            .iter()
            .map(|v| v.z)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            (min_z - 2.5).abs() < 1e-6,
            "bbox min-z should equal base_z 2.5, got {}",
            min_z
        );
        assert!(
            (max_z - 5.5).abs() < 1e-6,
            "bbox max-z should equal base_z + height, got {}",
            max_z
        );
    }

    #[test]
    fn test_base_z_preserved_for_slab_boundary() {
        // B1：封闭轮廓分支同样以第一个轮廓点的高程作为基准 z
        let layers = vec![LayerDef {
            name: "Slab".into(),
            thickness: 0.3,
            material_id: None,
            function: "structure".into(),
        }];
        let mesh = build_layered_slab_path_elevated(
            &[
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: 4.0, y: 0.0 },
                Point2D { x: 4.0, y: 3.0 },
                Point2D { x: 0.0, y: 3.0 },
            ],
            &layers,
            2.5,
        );
        let min_z = mesh
            .vertices
            .iter()
            .map(|v| v.z)
            .fold(f64::INFINITY, f64::min);
        assert!(
            (min_z - 2.5).abs() < 1e-6,
            "slab min-z should equal base_z 2.5, got {}",
            min_z
        );
    }

    #[test]
    fn test_roof_profile_layers_and_span() {
        // B2：两点轮廓屋面——各层沿层法向自下而上堆叠，跨度沿段方向
        let layers = vec![
            LayerDef {
                name: "Deck".into(),
                thickness: 0.1,
                material_id: None,
                function: "structure".into(),
            },
            LayerDef {
                name: "Finish".into(),
                thickness: 0.05,
                material_id: None,
                function: "finish".into(),
            },
        ];
        // 斜跨度 (0,0) → (4,3)
        let mesh = build_layered_roof_profile([0.0, 0.0], [4.0, 3.0], 2.0, &layers, 2.5);
        // 2 层 × 24 顶点（每层 box 6 面 × 4 顶点）
        assert_eq!(
            mesh.vertices.len(),
            48,
            "profile roof should have 2 layers × 24 vertices"
        );
        let min_z = mesh
            .vertices
            .iter()
            .map(|v| v.z)
            .fold(f64::INFINITY, f64::min);
        let max_z = mesh
            .vertices
            .iter()
            .map(|v| v.z)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            (min_z - 2.5).abs() < 1e-6,
            "roof bottom at base_z, got {}",
            min_z
        );
        assert!(
            (max_z - 2.65).abs() < 1e-6,
            "roof top at base_z + total thickness, got {}",
            max_z
        );
        // 层序：底层 Deck 比 Finish 更低（每层 24 顶点，6 面 × 4）
        let lower = &mesh.vertices[..24];
        let upper = &mesh.vertices[24..];
        let lower_max_z = lower.iter().map(|v| v.z).fold(f64::NEG_INFINITY, f64::max);
        let upper_min_z = upper.iter().map(|v| v.z).fold(f64::INFINITY, f64::min);
        assert!(
            lower_max_z <= upper_min_z + 1e-9,
            "layers should stack bottom-up"
        );
        // 跨度旋转正确：沿 (4,3)/5 方向展开
        let dir = (4.0f64 / 5.0, 3.0f64 / 5.0);
        let ts: Vec<f64> = mesh
            .vertices
            .iter()
            .map(|v| v.x * dir.0 + v.y * dir.1)
            .collect();
        let ns: Vec<f64> = mesh
            .vertices
            .iter()
            .map(|v| -v.x * dir.1 + v.y * dir.0)
            .collect();
        assert!(
            (ts.iter().cloned().fold(f64::INFINITY, f64::min) - 0.0).abs() < 1e-6,
            "span starts at start point projection"
        );
        assert!(
            (ts.iter().cloned().fold(f64::NEG_INFINITY, f64::max) - 5.0).abs() < 1e-6,
            "span ends at end point projection"
        );
        assert!(
            (ns.iter().cloned().fold(f64::INFINITY, f64::min) + 1.0).abs() < 1e-6,
            "width centered on span line"
        );
        assert!((ns.iter().cloned().fold(f64::NEG_INFINITY, f64::max) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_roof_profile_width_default() {
        // width <= 0 时回落为 1.0
        let layers = vec![LayerDef {
            name: "Deck".into(),
            thickness: 0.1,
            material_id: None,
            function: "structure".into(),
        }];
        let mesh = build_layered_roof_profile([0.0, 0.0], [2.0, 0.0], 0.0, &layers, 0.0);
        let max_y = mesh
            .vertices
            .iter()
            .map(|v| v.y)
            .fold(f64::NEG_INFINITY, f64::max);
        assert!(
            (max_y - 0.5).abs() < 1e-6,
            "default width 1.0, got y max {}",
            max_y
        );
    }

    #[test]
    fn test_miter_90_corner_no_gap() {
        // C1：90°L 形转角，外角交界处应被延伸后的矩形精确覆盖
        let t = 0.4;
        let layers = vec![LayerDef {
            name: "Core".into(),
            thickness: t,
            material_id: None,
            function: "structure".into(),
        }];
        let mesh = build_layered_path(
            &[
                Point2D { x: 0.0, y: 0.0 },
                Point2D { x: 3.0, y: 0.0 },
                Point2D { x: 3.0, y: 2.0 },
            ],
            false,
            1.0,
            &layers,
        );
        // 外角点 (3.0 + t/2, t/2)：两段矩形都应恰好达到该点
        let has_outer_corner = mesh
            .vertices
            .iter()
            .any(|v| (v.x - (3.0 + t / 2.0)).abs() < 1e-6 && (v.y - t / 2.0).abs() < 1e-6);
        assert!(
            has_outer_corner,
            "90° outer miter corner vertex (3.2, 0.2) should exist"
        );
        // 垂直段起点也被延伸到 y = -t/2（向下补齐转角）
        let min_y = mesh
            .vertices
            .iter()
            .map(|v| v.y)
            .fold(f64::INFINITY, f64::min);
        assert!(
            (min_y - (-t / 2.0)).abs() < 1e-6,
            "mitered wall bottom unextended at {}, expected -{}",
            min_y,
            t / 2.0
        );
    }

    #[test]
    fn test_validate_boundary_cases() {
        use Point2D as P;
        // 合法矩形
        assert!(validate_boundary(&[
            P { x: 0.0, y: 0.0 },
            P { x: 4.0, y: 0.0 },
            P { x: 4.0, y: 3.0 },
            P { x: 0.0, y: 3.0 }
        ])
        .is_ok());
        // 点数不足
        assert!(validate_boundary(&[P { x: 0.0, y: 0.0 }, P { x: 1.0, y: 0.0 }]).is_err());
        // 共线
        assert!(validate_boundary(&[
            P { x: 0.0, y: 0.0 },
            P { x: 1.0, y: 0.0 },
            P { x: 2.0, y: 0.0 }
        ])
        .is_err());
        // 零长度边
        assert!(validate_boundary(&[
            P { x: 0.0, y: 0.0 },
            P { x: 1.0, y: 0.0 },
            P { x: 1.0, y: 0.0 },
            P { x: 0.0, y: 1.0 }
        ])
        .is_err());
        // 自相交（蝴蝶结）
        assert!(validate_boundary(&[
            P { x: 0.0, y: 0.0 },
            P { x: 2.0, y: 2.0 },
            P { x: 2.0, y: 0.0 },
            P { x: 0.0, y: 2.0 }
        ])
        .is_err());
    }

    #[test]
    fn test_merge_collinear_removes_middle_points() {
        // 全共线路径：中间点全部合并，保留首尾
        let pts = vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 1.0, y: 0.0 }, // 共线中间点
            Point2D { x: 2.0, y: 0.0 }, // 共线中间点
            Point2D { x: 3.0, y: 0.0 },
        ];
        let merged = merge_collinear(&pts);
        assert_eq!(
            merged.len(),
            2,
            "collinear middle points should merge, got {:?}",
            merged
        );
        assert_eq!(merged[1], Point2D { x: 3.0, y: 0.0 });
        // 折线：每段方向拐弯处的点保留，同一方向上共线点合并
        // (1,0) 与 (2,0) 在 X 方向共线 → 合并为 (2,0)；(3,1) 转向保留
        let elbow = vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 1.0, y: 0.0 },
            Point2D { x: 2.0, y: 0.0 },
            Point2D { x: 3.0, y: 1.0 },
        ];
        let merged2 = merge_collinear(&elbow);
        assert_eq!(
            merged2.len(),
            3,
            "turn point and segment end should stay: {:?}",
            merged2
        );
        // 重复点也被去除
        let dup = vec![
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 0.0, y: 0.0 },
            Point2D { x: 1.0, y: 0.0 },
        ];
        assert_eq!(merge_collinear(&dup).len(), 2);
    }
}
