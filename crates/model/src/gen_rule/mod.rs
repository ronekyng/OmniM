//! ObjectType + tags → GenRule 映射表 + 几何生成引擎
//!
//! GenRule 是连接领域语义与统一几何内核的翻译层。
//!
//! 五种 GenRule（设计见 docs/specification/DATA_MODEL.md）：
//!   Primitive      — 直接体元拼装（Box/Cylinder/Sphere/Cone + 变换）
//!   LayerExtrusion — 多层拉伸（Wall/Slab 等，路径 + 多层结构）
//!   LinearComponent — 路径+变截面横断面（Road/Bridge/Pipe 等）
//!   Assembly       — 多图元组合装配（设备总成）
//!   BooleanSubtract — 布尔差集（开孔/剪切，Door/Window）

pub mod layer_extrusion;
pub mod linear_component;
pub mod assembly;
pub mod boolean_subtract;

use omnim_core as c;
use c::{MeshGeometry, ParametricGeometry};

pub use layer_extrusion::LayerExtrusionInput;
pub use linear_component::LinearComponentInput;
pub use assembly::AssemblyInput;
pub use boolean_subtract::BooleanSubtractInput;

/// 几何生成规则枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenRuleHint {
    Primitive,
    LayerExtrusion,
    LinearComponent,
    Assembly,
    BooleanSubtract,
}

impl GenRuleHint {
    pub fn as_str(&self) -> &'static str {
        match self {
            GenRuleHint::Primitive => "Primitive",
            GenRuleHint::LayerExtrusion => "LayerExtrusion",
            GenRuleHint::LinearComponent => "LinearComponent",
            GenRuleHint::Assembly => "Assembly",
            GenRuleHint::BooleanSubtract => "BooleanSubtract",
        }
    }
}

/// GenRule 引擎执行结果
pub struct GenRuleOutput {
    /// 生成的参数化几何体（可直接转为 mesh）
    pub parametric: ParametricGeometry,
    /// 或直接生成 mesh（LinearComponent 等复杂类型）
    pub mesh: Option<MeshGeometry>,
}

impl GenRuleOutput {
    pub fn from_parametric(pg: ParametricGeometry) -> Self {
        Self { parametric: pg, mesh: None }
    }

    pub fn from_mesh(mesh: MeshGeometry) -> Self {
        Self { parametric: ParametricGeometry { operations: Vec::new() }, mesh: Some(mesh) }
    }
}

use c::ObjectType;

/// 根据 ObjectType 获取主 GenRule
pub fn primary_gen_rule(object_type: ObjectType) -> GenRuleHint {
    match object_type {
        ObjectType::Unspecified | ObjectType::Generic => GenRuleHint::Primitive,

        ObjectType::Site | ObjectType::Building | ObjectType::BuildingStorey | ObjectType::Space => {
            GenRuleHint::Primitive
        }

        ObjectType::Wall | ObjectType::Slab | ObjectType::Floor | ObjectType::Roof => {
            GenRuleHint::LayerExtrusion
        }
        ObjectType::Stair => GenRuleHint::Assembly,
        ObjectType::Beam | ObjectType::Column => GenRuleHint::LinearComponent,
        ObjectType::Door | ObjectType::Window => GenRuleHint::BooleanSubtract,

        ObjectType::Duct | ObjectType::CableTray | ObjectType::PipePlant | ObjectType::PipeCivil => {
            GenRuleHint::LinearComponent
        }
        ObjectType::AirTerminal | ObjectType::Luminaire => GenRuleHint::Primitive,
        ObjectType::Ahu | ObjectType::Chiller | ObjectType::Transformer => GenRuleHint::Assembly,

        ObjectType::Road | ObjectType::Bridge | ObjectType::Tunnel | ObjectType::Culvert => {
            GenRuleHint::LinearComponent
        }
        ObjectType::Dam | ObjectType::RetainingWall => GenRuleHint::Primitive,
        ObjectType::Pier | ObjectType::Abutment => GenRuleHint::Primitive,
        ObjectType::Manhole => GenRuleHint::Primitive,

        ObjectType::Tank | ObjectType::Pump | ObjectType::Valve | ObjectType::HeatExchanger => {
            GenRuleHint::Primitive
        }
        ObjectType::Fitting | ObjectType::StructuralSteel => GenRuleHint::Primitive,
        ObjectType::Compressor | ObjectType::Instrument => GenRuleHint::Primitive,
        ObjectType::Reactor => GenRuleHint::Assembly,

        ObjectType::Part | ObjectType::Fastener | ObjectType::Gear | ObjectType::Shaft
        | ObjectType::Bearing | ObjectType::Motor | ObjectType::Spring | ObjectType::Seal => {
            GenRuleHint::Primitive
        }
        ObjectType::Assembly => GenRuleHint::Assembly,
        ObjectType::Component => GenRuleHint::Primitive,
        ObjectType::FeatureLine | ObjectType::ContourLine => GenRuleHint::LinearComponent,
    }
}

pub fn adjust_by_tags(hint: GenRuleHint, _tags: &[String]) -> GenRuleHint {
    hint
}

pub fn suggest(object_type: ObjectType, tags: &[String]) -> GenRuleHint {
    adjust_by_tags(primary_gen_rule(object_type), tags)
}

// ============================================================
// Default Template Registry
// Maps ObjectType -> catalog ComponentDef name.
// When `create wall` is called without explicit catalog reference,
// the system instantiates the default catalog component for that type.
// ============================================================

/// Returns the default catalog component name for a given ObjectType.
///
/// Returns `None` if no default template is registered for this type
/// (e.g. generic containers like Site, Building, Space).
pub fn default_catalog_component(object_type: ObjectType) -> Option<&'static str> {
    match object_type {
        // Building domain
        ObjectType::Wall => Some("Concrete-Wall"),
        ObjectType::Slab | ObjectType::Floor | ObjectType::Roof => Some("Concrete-Slab"),
        ObjectType::Beam => Some("Concrete-Beam-Rect"),
        ObjectType::Column => Some("Concrete-Column-Rect"),
        ObjectType::Door => Some("Door-Single"),
        ObjectType::Window => Some("Window-Fixed"),

        // Structural steel
        ObjectType::StructuralSteel => Some("Steel-Beam-IPE"),

        // Piping
        ObjectType::PipePlant | ObjectType::PipeCivil => Some("Pipe-CS"),
        ObjectType::Fitting => Some("Elbow-LR-CS"),
        ObjectType::Valve => Some("GateValve-CS"),
        ObjectType::Pump => Some("CentrifugalPump"),
        ObjectType::Tank => Some("VerticalTank"),

        // Civil
        ObjectType::Road => Some("Road-Asphalt"),
        ObjectType::Manhole => Some("Manhole"),
        ObjectType::Culvert => Some("Manhole"),

        // Plant
        ObjectType::HeatExchanger => Some("HeatExchanger"),
        ObjectType::Compressor => Some("Compressor"),

        // Mechanical
        ObjectType::Gear => Some("Gear"),
        ObjectType::Shaft | ObjectType::Bearing => Some("Gear"),
        ObjectType::Motor => Some("Compressor"),

        // MEP (building)
        ObjectType::Duct => Some("Duct-Rect"),
        ObjectType::CableTray => Some("CableTray"),

        // No default template for spatial containers and other types
        _ => None,
    }
}

/// Returns the default series name for a given ObjectType.
///
/// When `create pipe --series CS150` is used, the series drives the
/// component selection. This provides the fallback series per domain.
pub fn default_series_for_type(object_type: ObjectType) -> Option<&'static str> {
    match object_type {
        ObjectType::Wall | ObjectType::Slab | ObjectType::Floor | ObjectType::Roof
        | ObjectType::Beam | ObjectType::Column => Some("Concrete-C30"),
        ObjectType::StructuralSteel => Some("Steel-Q355"),
        ObjectType::PipePlant | ObjectType::PipeCivil
        | ObjectType::Fitting | ObjectType::Valve => Some("CS150"),
        ObjectType::Road => Some("Highway-GB"),
        _ => None,
    }
}

/// 统一几何生成入口（已废弃）
///
/// **Deprecated**: 请使用 `Project::create_object_from_default_template()` 代替。
/// 此函数保留用于向后兼容，新的默认模板系统通过 Feature Engine 生成几何。
///
/// 根据 ObjectType 自动选择 GenRule 并生成几何体。
/// 对于 Primitive 类型返回空（由调用方自行创建 Box/Cylinder 等），
/// 其他类型调用对应的 GenRule 便捷函数。
///
/// 返回: GenRuleOutput 包含 parametric 或 mesh 几何体
#[deprecated(note = "Use Project::create_object_from_default_template() instead")]
pub fn generate_geometry(object_type: ObjectType, _tags: &[String]) -> GenRuleOutput {
    let hint = suggest(object_type, _tags);

    // 仅为 LayerExtraction / LinearComponent / Assembly / BooleanSubtract 类型生成默认几何体
    // Primitive 类型由调用方通过 --geom 参数指定具体几何体
    let output = match hint {
        GenRuleHint::LayerExtrusion => {
            // 默认生成一个 5x3x0.2 的简单墙体
            GenRuleOutput::from_mesh(
                layer_extrusion::build_wall(5.0, 3.0, 0.2, 0.02)
            )
        }
        GenRuleHint::LinearComponent => {
            // 默认生成一段 5m 长、0.3m 半径的管道
            GenRuleOutput::from_mesh(
                linear_component::build_pipe(5.0, 0.3, 10)
            )
        }
        GenRuleHint::Assembly => {
            // 默认生成一个简单的设备基座
            GenRuleOutput::from_mesh(
                assembly::build_assembly(vec![])
            )
        }
        GenRuleHint::BooleanSubtract => {
            // BooleanSubtract 需要有宿主，默认返回空
            GenRuleOutput::from_mesh(MeshGeometry {
                vertices: Vec::new(), normals: Vec::new(), uvs: Vec::new(), faces: Vec::new(),
            })
        }
        GenRuleHint::Primitive => {
            // Primitive 返回空 Parametric，由调用方自行填充
            GenRuleOutput::from_parametric(ParametricGeometry { operations: Vec::new() })
        }
    };
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wall_is_layer_extrusion() {
        assert_eq!(suggest(ObjectType::Wall, &[]), GenRuleHint::LayerExtrusion);
    }

    #[test]
    fn test_road_is_linear_component() {
        assert_eq!(suggest(ObjectType::Road, &[]), GenRuleHint::LinearComponent);
    }

    #[test]
    fn test_pump_is_primitive() {
        assert_eq!(suggest(ObjectType::Pump, &[]), GenRuleHint::Primitive);
    }

    #[test]
    fn test_door_is_boolean_subtract() {
        assert_eq!(suggest(ObjectType::Door, &[]), GenRuleHint::BooleanSubtract);
    }

    #[test]
    fn test_duct_is_linear() {
        assert_eq!(suggest(ObjectType::Duct, &[]), GenRuleHint::LinearComponent);
    }

    #[test]
    fn test_all_types_have_mapping() {
        let all_types = [
            ObjectType::Wall, ObjectType::Beam, ObjectType::Column, ObjectType::Slab,
            ObjectType::Door, ObjectType::Window, ObjectType::Stair, ObjectType::Roof,
            ObjectType::Floor, ObjectType::Duct, ObjectType::CableTray, ObjectType::AirTerminal,
            ObjectType::Ahu, ObjectType::Chiller, ObjectType::Transformer, ObjectType::Luminaire,
            ObjectType::Road, ObjectType::Bridge, ObjectType::Tunnel, ObjectType::Dam,
            ObjectType::Culvert, ObjectType::RetainingWall, ObjectType::PipeCivil,
            ObjectType::Pier, ObjectType::Manhole, ObjectType::Abutment,
            ObjectType::Tank, ObjectType::Pump, ObjectType::Valve, ObjectType::HeatExchanger,
            ObjectType::Reactor, ObjectType::PipePlant, ObjectType::StructuralSteel,
            ObjectType::Fitting, ObjectType::Compressor, ObjectType::Instrument,
            ObjectType::Part, ObjectType::Assembly, ObjectType::Fastener, ObjectType::Gear,
            ObjectType::Shaft, ObjectType::Bearing, ObjectType::Motor, ObjectType::Spring,
            ObjectType::Seal, ObjectType::Site, ObjectType::Building, ObjectType::BuildingStorey,
            ObjectType::Space, ObjectType::Unspecified, ObjectType::Generic,
        ];
        for t in &all_types {
            let _ = suggest(*t, &[]);
        }
    }

    #[test]
    fn test_default_catalog_component_wall() {
        assert_eq!(default_catalog_component(ObjectType::Wall), Some("Concrete-Wall"));
    }

    #[test]
    fn test_default_catalog_component_beam() {
        assert_eq!(default_catalog_component(ObjectType::Beam), Some("Concrete-Beam-Rect"));
    }

    #[test]
    fn test_default_catalog_component_pipe() {
        assert_eq!(default_catalog_component(ObjectType::PipePlant), Some("Pipe-CS"));
    }

    #[test]
    fn test_default_catalog_component_road() {
        assert_eq!(default_catalog_component(ObjectType::Road), Some("Road-Asphalt"));
    }

    #[test]
    fn test_default_catalog_component_none_for_spatial() {
        assert_eq!(default_catalog_component(ObjectType::Site), None);
        assert_eq!(default_catalog_component(ObjectType::Building), None);
        assert_eq!(default_catalog_component(ObjectType::Generic), None);
    }

    #[test]
    fn test_default_series_for_wall() {
        assert_eq!(default_series_for_type(ObjectType::Wall), Some("Concrete-C30"));
    }

    #[test]
    fn test_default_series_for_pipe() {
        assert_eq!(default_series_for_type(ObjectType::PipePlant), Some("CS150"));
    }

    #[test]
    fn test_default_series_for_road() {
        assert_eq!(default_series_for_type(ObjectType::Road), Some("Highway-GB"));
    }
}
