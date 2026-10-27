//! AssemblyGen — 装配组合引擎
//!
//! 将多个子组件（含各自的 ParametricGeometry + Transform）组合为一个几何体。
//! 适用于：设备总成 (AHU, Chiller, Reactor)、机械装配 (Assembly)、楼梯 (Stair)。
//!
//! 原理：
//! - 遍历子组件列表
//! - 将每个子组件的 ParametricGeometry 转为 mesh
//! - 应用 Transform（平移、旋转、缩放）
//! - 合并所有 mesh

use omnim_core as c;
use c::{Box, Cylinder, MeshGeometry, ParametricGeometry, ParametricOperation, Point3D, Transform};

/// 子组件定义
#[derive(Debug, Clone)]
pub struct AssemblyComponent {
    /// 子对象名称
    pub name: String,
    /// 子对象的参数化几何体
    pub geometry: ParametricGeometry,
    /// 世界空间变换
    pub transform: Transform,
}

impl AssemblyComponent {
    /// 创建一个简单 Box 子组件
    pub fn box_component(name: &str, l: f64, w: f64, h: f64, x: f64, y: f64, z: f64) -> Self {
        Self {
            name: name.into(),
            geometry: ParametricGeometry {
                operations: vec![ParametricOperation {
                    operation: Some(c::parametric_operation::Operation::Box(Box {
                        length: l, width: w, height: h,
                    })),
                }],
            },
            transform: Transform {
                translation: Some(Point3D { x, y, z }),
                rotation: None, scale: None, matrix: Vec::new(),
            },
        }
    }

    /// 创建一个简单 Cylinder 子组件
    pub fn cylinder_component(name: &str, radius: f64, height: f64, x: f64, y: f64, z: f64) -> Self {
        Self {
            name: name.into(),
            geometry: ParametricGeometry {
                operations: vec![ParametricOperation {
                    operation: Some(c::parametric_operation::Operation::Cylinder(Cylinder {
                        radius, height, segments: 16,
                    })),
                }],
            },
            transform: Transform {
                translation: Some(Point3D { x, y, z }),
                rotation: None, scale: None, matrix: Vec::new(),
            },
        }
    }
}

/// AssemblyGen 输入参数
#[derive(Debug, Clone)]
pub struct AssemblyInput {
    /// 子组件列表
    pub components: Vec<AssemblyComponent>,
}

impl Default for AssemblyInput {
    fn default() -> Self {
        Self { components: Vec::new() }
    }
}

/// 生成装配体几何体
///
/// 将每个子组件转为 mesh、应用变换、合并为一个 mesh。
pub fn generate(input: &AssemblyInput) -> MeshGeometry {
    let component_meshes: Vec<MeshGeometry> = input.components.iter().map(|comp| {
        let param_mesh = crate::geometry::parametric_to_mesh(&comp.geometry);
        if is_identity_transform(&comp.transform) {
            param_mesh
        } else {
            crate::geometry::transform_mesh(&param_mesh, &comp.transform)
        }
    }).collect();

    crate::geometry::merge_meshes(&component_meshes)
}

fn is_identity_transform(t: &Transform) -> bool {
    let no_translation = t.translation.as_ref().map_or(true, |v| {
        v.x.abs() < 1e-10 && v.y.abs() < 1e-10 && v.z.abs() < 1e-10
    });
    let no_rotation = t.rotation.as_ref().map_or(true, |r| {
        (r.w - 1.0).abs() < 1e-10 && r.x.abs() < 1e-10 && r.y.abs() < 1e-10 && r.z.abs() < 1e-10
    });
    let no_scale = t.scale.as_ref().map_or(true, |s| {
        (s.x - 1.0).abs() < 1e-10 && (s.y - 1.0).abs() < 1e-10 && (s.z - 1.0).abs() < 1e-10
    });
    no_translation && no_rotation && no_scale
}

/// 便捷：构建设备装配体
///
/// 将多个 Box/Extrusion/Cylinder 子组件组合为装配体。
pub fn build_assembly(components: Vec<AssemblyComponent>) -> MeshGeometry {
    let input = AssemblyInput { components };
    generate(&input)
}

// ============================================================
// 领域便捷构建函数
// ============================================================

/// 构建 AHU（空调处理机组）模型
///
/// 简化的 AHU 总成：机箱 + 风扇 + 过滤段 + 进出风口
pub fn build_ahu(length: f64, width: f64, height: f64) -> MeshGeometry {
    let components = vec![
        // 机箱主体
        AssemblyComponent::box_component("Casing", length, width, height, 0.0, 0.0, 0.0),
        // 风扇段（圆柱，在机箱顶部）
        AssemblyComponent::cylinder_component("Fan", width * 0.3, height * 0.4, 0.0, 0.0, height * 0.5),
        // 进风口（在 +X 端）
        AssemblyComponent::box_component("AirInlet", 0.05, width * 0.6, height * 0.3, length / 2.0 + 0.025, 0.0, height * 0.2),
        // 出风口（在 -X 端）
        AssemblyComponent::box_component("AirOutlet", 0.05, width * 0.6, height * 0.3, -length / 2.0 - 0.025, 0.0, height * 0.2),
        // 控制箱（小盒子在侧面）
        AssemblyComponent::box_component("ControlBox", length * 0.15, width * 0.1, height * 0.25, length * 0.2, width / 2.0 + 0.05, height * 0.3),
    ];
    build_assembly(components)
}

/// 构建楼梯（简化直跑楼梯）
///
/// - width: 梯段宽度
/// - total_rise: 总高度
/// - total_run: 总水平长度
/// - tread_count: 踏步数量
pub fn build_stair(width: f64, total_rise: f64, total_run: f64, tread_count: usize) -> MeshGeometry {
    let mut components = Vec::new();
    let n = tread_count.max(2);
    let riser_height = total_rise / n as f64;
    let tread_depth = total_run / n as f64;

    for i in 0..n {
        let x = -total_run / 2.0 + (i as f64 + 0.5) * tread_depth;
        let z = (i as f64 + 0.5) * riser_height;
        // 每个踏步 = 一个扁平的踏步板 + 一个踢面
        components.push(AssemblyComponent::box_component(
            &format!("Tread-{}", i),
            tread_depth, width, riser_height * 0.1, // 薄的踏步板
            x, 0.0, z,
        ));
        // 踢面（竖板）
        if i < n - 1 {
            components.push(AssemblyComponent::box_component(
                &format!("Riser-{}", i),
                riser_height * 0.05, width, riser_height,
                x + tread_depth / 2.0, 0.0, z + riser_height * 0.55,
            ));
        }
    }

    // 两侧的梯梁
    components.push(AssemblyComponent::box_component(
        "Stringer-Left", total_run, 0.05, total_rise,
        0.0, -width / 2.0 - 0.025, total_rise / 2.0,
    ));
    components.push(AssemblyComponent::box_component(
        "Stringer-Right", total_run, 0.05, total_rise,
        0.0, width / 2.0 + 0.025, total_rise / 2.0,
    ));

    build_assembly(components)
}

/// 构建反应器（简化表示）
///
/// - height: 总高度
/// - diameter: 主体直径
pub fn build_reactor(height: f64, diameter: f64) -> MeshGeometry {
    let r = diameter / 2.0;
    let components = vec![
        // 主体（圆柱）
        AssemblyComponent::cylinder_component("Vessel", r, height * 0.7, 0.0, 0.0, 0.0),
        // 顶部半球封头
        AssemblyComponent::box_component("TopHead", diameter * 1.1, diameter * 1.1, height * 0.15, 0.0, 0.0, height * 0.425),
        // 底部半球封头
        AssemblyComponent::box_component("BottomHead", diameter * 1.1, diameter * 1.1, height * 0.15, 0.0, 0.0, -height * 0.425),
        // 搅拌器电机（顶部小圆柱）
        AssemblyComponent::cylinder_component("Motor", r * 0.2, height * 0.1, 0.0, 0.0, height * 0.55),
        // 进料口（侧方）
        AssemblyComponent::cylinder_component("Inlet", r * 0.15, 0.1, height * 0.2, r + 0.05, 0.0),
        // 出料口（底部）
        AssemblyComponent::cylinder_component("Outlet", r * 0.15, 0.15, 0.0, 0.0, -height * 0.5),
        // 4 个支腿
        AssemblyComponent::box_component("Leg-1", 0.1, 0.1, height * 0.15, r * 0.7, r * 0.7, -height * 0.5 - height * 0.075),
        AssemblyComponent::box_component("Leg-2", 0.1, 0.1, height * 0.15, -r * 0.7, r * 0.7, -height * 0.5 - height * 0.075),
        AssemblyComponent::box_component("Leg-3", 0.1, 0.1, height * 0.15, r * 0.7, -r * 0.7, -height * 0.5 - height * 0.075),
        AssemblyComponent::box_component("Leg-4", 0.1, 0.1, height * 0.15, -r * 0.7, -r * 0.7, -height * 0.5 - height * 0.075),
    ];
    build_assembly(components)
}

/// 构建泵（简化表示）
///
/// - motor_length, motor_width, motor_height: 电机尺寸
/// - pump_radius: 泵体半径
/// - pump_height: 泵体高度
pub fn build_pump(motor_length: f64, motor_width: f64, motor_height: f64, pump_radius: f64, pump_height: f64) -> MeshGeometry {
    let components = vec![
        // 电机部分
        AssemblyComponent::box_component("Motor", motor_length, motor_width, motor_height, -motor_length / 2.0 - pump_radius, 0.0, 0.0),
        // 泵体（圆柱）
        AssemblyComponent::cylinder_component("PumpBody", pump_radius, pump_height, 0.0, 0.0, 0.0),
        // 进口法兰
        AssemblyComponent::box_component("InletFlange", 0.05, pump_radius * 1.5, pump_radius * 1.5, 0.0, pump_radius + 0.025, 0.0),
        // 出口法兰（顶部）
        AssemblyComponent::box_component("OutletFlange", pump_radius * 1.5, pump_radius * 1.5, 0.05, 0.0, 0.0, pump_radius + 0.025),
        // 底座
        AssemblyComponent::box_component("Base", motor_length + pump_radius * 2.0, motor_width + 0.1, 0.05, 0.0, 0.0, -pump_height / 2.0 - 0.025),
    ];
    build_assembly(components)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_box(name: &str, l: f64, w: f64, h: f64) -> AssemblyComponent {
        AssemblyComponent {
            name: name.into(),
            geometry: ParametricGeometry {
                operations: vec![c::ParametricOperation {
                    operation: Some(c::parametric_operation::Operation::Box(Box {
                        length: l, width: w, height: h,
                    })),
                }],
            },
            transform: Transform {
                translation: None,
                rotation: None,
                scale: None,
                matrix: Vec::new(),
            },
        }
    }

    #[test]
    fn test_single_component_assembly() {
        let comp = make_box("Base", 2.0, 2.0, 1.0);
        let mesh = build_assembly(vec![comp]);
        assert!(!mesh.vertices.is_empty());
    }

    #[test]
    fn test_multi_component_assembly() {
        let base = make_box("Base", 4.0, 3.0, 0.5);
        let body = AssemblyComponent {
            name: "Body".into(),
            geometry: ParametricGeometry {
                operations: vec![c::ParametricOperation {
                    operation: Some(c::parametric_operation::Operation::Cylinder(Cylinder {
                        radius: 0.5, height: 2.0, segments: 16,
                    })),
                }],
            },
            transform: Transform {
                translation: Some(Point3D { x: 0.0, y: 0.0, z: 0.5 }),
                rotation: None,
                scale: None,
                matrix: Vec::new(),
            },
        };
        let mesh = build_assembly(vec![base, body]);
        assert!(mesh.vertices.len() > 10);
    }

    #[test]
    fn test_translated_components() {
        let a = AssemblyComponent {
            name: "A".into(),
            geometry: ParametricGeometry {
                operations: vec![c::ParametricOperation {
                    operation: Some(c::parametric_operation::Operation::Box(Box {
                        length: 1.0, width: 1.0, height: 1.0,
                    })),
                }],
            },
            transform: Transform {
                translation: Some(Point3D { x: 0.0, y: 0.0, z: 0.0 }),
                rotation: None,
                scale: None,
                matrix: Vec::new(),
            },
        };
        let b = AssemblyComponent {
            name: "B".into(),
            geometry: ParametricGeometry {
                operations: vec![c::ParametricOperation {
                    operation: Some(c::parametric_operation::Operation::Box(Box {
                        length: 1.0, width: 1.0, height: 1.0,
                    })),
                }],
            },
            transform: Transform {
                translation: Some(Point3D { x: 3.0, y: 0.0, z: 0.0 }),
                rotation: None,
                scale: None,
                matrix: Vec::new(),
            },
        };
        let mesh = build_assembly(vec![a, b]);
        assert!(mesh.vertices.len() >= 16);
    }

    #[test]
    fn test_ahu_assembly() {
        let mesh = build_ahu(3.0, 1.5, 1.2);
        // AHU 有多个组件，应产生足够的顶点
        assert!(mesh.vertices.len() >= 40);
        assert!(mesh.faces.len() >= 20);
    }

    #[test]
    fn test_stair_assembly() {
        let mesh = build_stair(1.0, 2.8, 4.0, 10);
        // 楼梯应有踏步 + 踢面 + 梯梁
        assert!(mesh.vertices.len() >= 50);
        assert!(mesh.faces.len() >= 20);
    }

    #[test]
    fn test_reactor_assembly() {
        let mesh = build_reactor(5.0, 2.0);
        // 反应器应有多组件
        assert!(mesh.vertices.len() >= 40);
        assert!(mesh.faces.len() >= 20);
    }

    #[test]
    fn test_pump_assembly() {
        let mesh = build_pump(1.0, 0.4, 0.4, 0.3, 0.5);
        // 泵体应有 motor + pump body + flanges + base
        assert!(mesh.vertices.len() >= 40);
        assert!(mesh.faces.len() >= 20);
    }
}