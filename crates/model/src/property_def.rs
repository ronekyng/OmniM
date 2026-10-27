use serde::{Deserialize, Serialize};

use omnim_core as c;

/// The value type a property can carry. Mirrors the richer `PropertyValue`
/// variants the protobuf layer defines, exposed as a small enum for the UI.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PropertyKind {
    /// double-precision number (renders as slider / number input)
    Double,
    /// free-form text
    String,
    /// boolean toggle
    Bool,
    /// one-of-N choice from `choices`
    Choice,
}

/// A typed, UI-friendly definition of one editable property on an object type.
/// This is the data-driven replacement for the frontend's hardcoded `TYPE_PARAMS`
/// table: the panel renders inputs from these definitions instead of a hand
/// maintained map.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ObjectPropertyDef {
    pub name: String,
    pub label: String,
    pub kind: PropertyKind,
    pub unit: String,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub choices: Vec<String>,
    pub description: String,
    /// Logical grouping so the UI can fold related properties together.
    pub group: String,
}

impl ObjectPropertyDef {
    pub fn double(name: &str, label: &str) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            kind: PropertyKind::Double,
            unit: String::new(),
            min: f64::NEG_INFINITY,
            max: f64::INFINITY,
            step: 0.0,
            choices: Vec::new(),
            description: String::new(),
            group: "几何参数".into(),
        }
    }

    pub fn with_unit(mut self, unit: &str) -> Self {
        self.unit = unit.into();
        self
    }

    pub fn with_range(mut self, min: f64, max: f64, step: f64) -> Self {
        self.min = min;
        self.max = max;
        self.step = step;
        self
    }

    pub fn string(name: &str, label: &str) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            kind: PropertyKind::String,
            unit: String::new(),
            min: f64::NEG_INFINITY,
            max: f64::INFINITY,
            step: 0.0,
            choices: Vec::new(),
            description: String::new(),
            group: "属性".into(),
        }
    }

    pub fn choice(name: &str, label: &str, choices: &[&str]) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            kind: PropertyKind::Choice,
            unit: String::new(),
            min: f64::NEG_INFINITY,
            max: f64::INFINITY,
            step: 0.0,
            choices: choices.iter().map(|s| s.to_string()).collect(),
            description: String::new(),
            group: "属性".into(),
        }
    }

    pub fn boolean(name: &str, label: &str) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            kind: PropertyKind::Bool,
            unit: String::new(),
            min: f64::NEG_INFINITY,
            max: f64::INFINITY,
            step: 0.0,
            choices: Vec::new(),
            description: String::new(),
            group: "属性".into(),
        }
    }
}

/// Returns the full set of editable property definitions for a given object
/// type. This is the data-driven schema consumed by the property panel.
///
/// For Component-typed objects the panel should prefer the catalog
/// `ComponentDef.params` (via `params_to_property_defs`); for built-in object
/// types we provide a curated schema here.
pub fn property_defs_for_type(object_type: i32) -> Vec<ObjectPropertyDef> {
    use c::ObjectType::*;
    let t = c::ObjectType::try_from(object_type).unwrap_or(Generic);
    match t {
        Wall | Slab | Roof | Floor => vec![
            ObjectPropertyDef::double("height", "高度").with_unit("m").with_range(0.0, 100.0, 0.01),
            ObjectPropertyDef::double("core_thickness", "核心层厚度").with_unit("mm").with_range(0.0, 1000.0, 1.0),
            ObjectPropertyDef::double("finish_thickness", "饰面层厚度").with_unit("mm").with_range(0.0, 1000.0, 1.0),
            ObjectPropertyDef::string("layers", "层结构 (JSON)"),
        ],
        Beam | Column => vec![
            ObjectPropertyDef::double("length", "长度").with_unit("m").with_range(0.0, 100.0, 0.01),
            ObjectPropertyDef::double("width", "宽度").with_unit("mm").with_range(0.0, 2000.0, 1.0),
            ObjectPropertyDef::double("height", "高度").with_unit("mm").with_range(0.0, 2000.0, 1.0),
        ],
        Road | Duct | CableTray => vec![
            ObjectPropertyDef::double("length", "长度").with_unit("m").with_range(0.0, 1000.0, 0.1),
            ObjectPropertyDef::double("width", "宽度").with_unit("m").with_range(0.0, 50.0, 0.1),
            ObjectPropertyDef::double("thickness", "厚度").with_unit("mm").with_range(0.0, 1000.0, 1.0),
        ],
        PipePlant | PipeCivil => vec![
            ObjectPropertyDef::double("length", "长度").with_unit("m").with_range(0.0, 1000.0, 0.1),
            ObjectPropertyDef::double("radius", "半径").with_unit("mm").with_range(0.0, 2000.0, 1.0),
            ObjectPropertyDef::string("start_point", "起点 (x,y,z)"),
            ObjectPropertyDef::string("end_point", "终点 (x,y,z)"),
        ],
        Tank => vec![
            ObjectPropertyDef::double("diameter", "直径").with_unit("m").with_range(0.0, 50.0, 0.1),
            ObjectPropertyDef::double("height", "高度").with_unit("m").with_range(0.0, 50.0, 0.1),
        ],
        Pump => vec![
            ObjectPropertyDef::double("DN", "DN").with_unit("mm").with_range(0.0, 2000.0, 1.0),
            ObjectPropertyDef::double("height", "高度").with_unit("m").with_range(0.0, 10.0, 0.01),
        ],
        Valve => vec![
            ObjectPropertyDef::double("DN", "DN").with_unit("mm").with_range(0.0, 2000.0, 1.0),
            ObjectPropertyDef::string("pressure_class", "压力等级"),
        ],
        Door | Window => vec![
            ObjectPropertyDef::double("opening_width", "开口宽度").with_unit("m").with_range(0.0, 10.0, 0.01),
            ObjectPropertyDef::double("opening_height", "开口高度").with_unit("m").with_range(0.0, 10.0, 0.01),
        ],
        Part => vec![
            ObjectPropertyDef::double("width", "宽度").with_unit("m").with_range(0.0, 10.0, 0.01),
            ObjectPropertyDef::double("depth", "深度").with_unit("m").with_range(0.0, 10.0, 0.01),
            ObjectPropertyDef::double("height", "高度").with_unit("m").with_range(0.0, 10.0, 0.01),
        ],
        Gear => vec![
            ObjectPropertyDef::double("module", "模数").with_unit("mm").with_range(0.0, 100.0, 0.5),
            ObjectPropertyDef::double("teeth", "齿数").with_range(1.0, 500.0, 1.0),
        ],
        _ => Vec::new(),
    }
}

/// Convert a catalog `ComponentDef.params` list into `ObjectPropertyDef`s so
/// Component-instances render their parameter inputs from the library rather
/// than from a hardcoded table.
pub fn params_to_property_defs(params: &[crate::catalog::ParamDef]) -> Vec<ObjectPropertyDef> {
    params
        .iter()
        .map(|p| {
            let mut def = if p.choices.is_empty() {
                ObjectPropertyDef::double(&p.name, &p.name)
            } else {
                ObjectPropertyDef::choice(&p.name, &p.name, &p.choices.iter().map(String::as_str).collect::<Vec<_>>())
            };
            def.unit = p.unit.clone();
            if p.min_value.is_finite() {
                def.min = p.min_value;
            }
            if p.max_value.is_finite() {
                def.max = p.max_value;
            }
            def.step = p.step;
            def.description = p.description.clone();
            def.group = "构件参数".into();
            def
        })
        .collect()
}

/// Merge catalog-derived defs (which take precedence) with the built-in schema,
/// de-duplicating by name.
pub fn merge_defs(builtin: Vec<ObjectPropertyDef>, catalog_params: &[crate::catalog::ParamDef]) -> Vec<ObjectPropertyDef> {
    let mut catalog = params_to_property_defs(catalog_params);
    let mut names: std::collections::HashSet<String> = catalog.iter().map(|d| d.name.clone()).collect();
    for d in builtin {
        if names.insert(d.name.clone()) {
            catalog.push(d);
        }
    }
    catalog
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wall_defs() {
        let defs = property_defs_for_type(c::ObjectType::Wall as i32);
        assert!(!defs.is_empty());
        assert!(defs.iter().any(|d| d.name == "height"));
    }

    #[test]
    fn test_unknown_type_empty() {
        assert!(property_defs_for_type(9999).is_empty());
    }

    #[test]
    fn test_params_to_defs() {
        let params = vec![crate::catalog::ParamDef::new("width", 0)];
        let defs = params_to_property_defs(&params);
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].name, "width");
        assert_eq!(defs[0].group, "构件参数");
    }

    #[test]
    fn test_merge_dedup() {
        let builtin = vec![
            ObjectPropertyDef::double("height", "高度"),
            ObjectPropertyDef::double("width", "宽度"),
        ];
        let params = vec![crate::catalog::ParamDef::new("width", 0)];
        let merged = merge_defs(builtin, &params);
        let names: Vec<_> = merged.iter().map(|d| d.name.as_str()).collect();
        assert_eq!(names, vec!["width", "height"]);
    }
}