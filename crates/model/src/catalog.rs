use std::collections::HashMap;

use omnim_core as c;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================
// ComponentDef - a parametric 3D component definition (PARAGON)
// ============================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ComponentDef {
    pub id: String,
    pub name: String,
    pub category: String,
    pub description: String,
    pub params: Vec<ParamDef>,
    pub feature_tree_id: String,
    pub connectors: Vec<ConnectorDef>,
    pub allowed_materials: Vec<String>,
    pub default_material: String,
    pub ifc_class: String,
    pub standard_reference: String,
    pub tags: Vec<String>,
    pub domain: String,
    pub properties: HashMap<String, String>,
    pub published: bool,
}

impl ComponentDef {
    pub fn new(name: impl Into<String>, category: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            category: category.into(),
            description: String::new(),
            params: Vec::new(),
            feature_tree_id: String::new(),
            connectors: Vec::new(),
            allowed_materials: Vec::new(),
            default_material: String::new(),
            ifc_class: String::new(),
            standard_reference: String::new(),
            tags: Vec::new(),
            domain: String::new(),
            properties: HashMap::new(),
            published: false,
        }
    }
}

impl From<&c::ComponentDef> for ComponentDef {
    fn from(p: &c::ComponentDef) -> Self {
        Self {
            id: p.id.clone(),
            name: p.name.clone(),
            category: p.category.clone(),
            description: p.description.clone(),
            params: p.params.iter().map(ParamDef::from).collect(),
            feature_tree_id: p.feature_tree_id.clone(),
            connectors: p.connectors.iter().map(ConnectorDef::from).collect(),
            allowed_materials: p.allowed_materials.clone(),
            default_material: p.default_material.clone(),
            ifc_class: p.ifc_class.clone(),
            standard_reference: p.standard_reference.clone(),
            tags: p.tags.clone(),
            domain: p.domain.clone(),
            properties: p.properties.clone(),
            published: p.published,
        }
    }
}

// ============================================================
// ParamDef - typed parameter definition
// ============================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ParamDef {
    pub name: String,
    pub param_type: i32,
    pub default_value: f64,
    pub min_value: f64,
    pub max_value: f64,
    pub step: f64,
    pub unit: String,
    pub formula: String,
    pub choices: Vec<String>,
    pub description: String,
}

impl ParamDef {
    pub fn new(name: impl Into<String>, param_type: i32) -> Self {
        Self {
            name: name.into(),
            param_type,
            default_value: 0.0,
            min_value: 0.0,
            max_value: f64::MAX,
            step: 0.0,
            unit: String::new(),
            formula: String::new(),
            choices: Vec::new(),
            description: String::new(),
        }
    }

    pub fn with_default(mut self, v: f64) -> Self { self.default_value = v; self }
    pub fn with_range(mut self, min: f64, max: f64) -> Self { self.min_value = min; self.max_value = max; self }
    pub fn with_step(mut self, s: f64) -> Self { self.step = s; self }
    pub fn with_unit(mut self, u: impl Into<String>) -> Self { self.unit = u.into(); self }
    pub fn with_choices(mut self, c: Vec<String>) -> Self { self.choices = c; self }
}

impl From<&c::ParamDef> for ParamDef {
    fn from(p: &c::ParamDef) -> Self {
        Self {
            name: p.name.clone(),
            param_type: p.r#type,
            default_value: p.default_value,
            min_value: p.min_value,
            max_value: p.max_value,
            step: p.step,
            unit: p.unit.clone(),
            formula: p.formula.clone(),
            choices: p.choices.clone(),
            description: p.description.clone(),
        }
    }
}

// ============================================================
// ConnectorDef - connection point definition
// ============================================================

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConnectorDef {
    pub id: String,
    pub name: String,
    pub position_x_expr: String,
    pub position_y_expr: String,
    pub position_z_expr: String,
    pub direction_x_expr: String,
    pub direction_y_expr: String,
    pub direction_z_expr: String,
    pub bore_expr: String,
    pub connection_type_code: String,
    pub role: i32,
}

impl ConnectorDef {
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            position_x_expr: "0".into(),
            position_y_expr: "0".into(),
            position_z_expr: "0".into(),
            direction_x_expr: "0".into(),
            direction_y_expr: "0".into(),
            direction_z_expr: "1".into(),
            bore_expr: String::new(),
            connection_type_code: String::new(),
            role: c::ConnectorRole::Bidirectional as i32,
        }
    }

    pub fn with_position(mut self, x: &str, y: &str, z: &str) -> Self {
        self.position_x_expr = x.into();
        self.position_y_expr = y.into();
        self.position_z_expr = z.into();
        self
    }

    pub fn with_direction(mut self, x: &str, y: &str, z: &str) -> Self {
        self.direction_x_expr = x.into();
        self.direction_y_expr = y.into();
        self.direction_z_expr = z.into();
        self
    }

    pub fn with_bore(mut self, bore: impl Into<String>) -> Self { self.bore_expr = bore.into(); self }
    pub fn with_type_code(mut self, code: impl Into<String>) -> Self { self.connection_type_code = code.into(); self }
    pub fn with_role(mut self, role: i32) -> Self { self.role = role; self }
}

impl From<&c::ConnectorDef> for ConnectorDef {
    fn from(p: &c::ConnectorDef) -> Self {
        Self {
            id: p.id.clone(),
            name: p.name.clone(),
            position_x_expr: p.position_x_expr.clone(),
            position_y_expr: p.position_y_expr.clone(),
            position_z_expr: p.position_z_expr.clone(),
            direction_x_expr: p.direction_x_expr.clone(),
            direction_y_expr: p.direction_y_expr.clone(),
            direction_z_expr: p.direction_z_expr.clone(),
            bore_expr: p.bore_expr.clone(),
            connection_type_code: p.connection_type_code.clone(),
            role: p.role,
        }
    }
}

// ============================================================
// CatalogStore - in-memory catalog storage
// ============================================================

#[derive(Serialize, Deserialize, Clone)]
pub struct CatalogStore {
    components: HashMap<String, ComponentDef>,
    #[serde(skip)]
    by_category: HashMap<String, Vec<String>>,
}

impl Default for CatalogStore {
    fn default() -> Self {
        Self::new()
    }
}

impl CatalogStore {
    pub fn new() -> Self {
        Self {
            components: HashMap::new(),
            by_category: HashMap::new(),
        }
    }

    pub fn insert(&mut self, comp: ComponentDef) -> &mut ComponentDef {
        let id = comp.id.clone();
        let category = comp.category.clone();
        self.components.insert(id.clone(), comp);
        self.by_category.entry(category).or_default().push(id.clone());
        self.components.get_mut(&id).unwrap()
    }

    pub fn get(&self, id: &str) -> Option<&ComponentDef> {
        self.components.get(id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut ComponentDef> {
        self.components.get_mut(id)
    }

    pub fn remove(&mut self, id: &str) -> Option<ComponentDef> {
        let comp = self.components.remove(id)?;
        if let Some(ids) = self.by_category.get_mut(&comp.category) {
            ids.retain(|i| i != id);
        }
        Some(comp)
    }

    pub fn find_by_name(&self, name: &str) -> Option<&ComponentDef> {
        self.components.values().find(|c| c.name == name)
    }

    pub fn find_by_name_mut(&mut self, name: &str) -> Option<&mut ComponentDef> {
        self.components.values_mut().find(|c| c.name == name)
    }

    pub fn list(&self) -> Vec<&ComponentDef> {
        self.components.values().collect()
    }

    pub fn list_by_category(&self, category: &str) -> Vec<&ComponentDef> {
        self.components.values()
            .filter(|c| c.category == category || c.category.starts_with(&format!("{}/", category)))
            .collect()
    }

    pub fn list_published(&self) -> Vec<&ComponentDef> {
        self.components.values().filter(|c| c.published).collect()
    }

    pub fn len(&self) -> usize {
        self.components.len()
    }

    pub fn is_empty(&self) -> bool {
        self.components.is_empty()
    }
}

// ============================================================
// CatalogRef - reference from a project object to its catalog origin
// ============================================================

/// Records the catalog origin of a project object created from a catalog
/// component through a series. This enables traceability, regeneration,
/// and standards enforcement.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CatalogRef {
    /// The ComponentDef ID this object was instantiated from.
    pub component_def_id: String,
    /// The Series (spec) ID that approved this component, if any.
    pub series_id: String,
    /// The path through the series decision tree (selector IDs).
    pub selector_path: Vec<String>,
    /// The instance parameter values used during instantiation.
    pub param_values: HashMap<String, f64>,
}

impl Default for CatalogRef {
    fn default() -> Self {
        Self {
            component_def_id: String::new(),
            series_id: String::new(),
            selector_path: Vec::new(),
            param_values: HashMap::new(),
        }
    }
}

/// Connector type taxonomy for cross-platform mapping.
///
/// Maps between Civil3D Point Codes, OmniM ConnectorTypes, and E3D P-Points.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConnectorType {
    PipeEnd,
    FlangeFace,
    ElbowEnd,
    TeeRun,
    TeeBranch,
    StructuralEnd,
    StructuralBase,
    EquipmentNozzle,
    CableEnd,
    DuctEnd,
    ValveEnd,
    ValveStem,
    Generic,
}

/// Cross-platform connector type code mapping.
///
/// Returns a map from platform → (code → ConnectorType).
/// Platforms: "C3D" (Civil3D), "OmniM", "E3D"
pub fn connector_code_map() -> std::collections::HashMap<String, std::collections::HashMap<String, ConnectorType>> {
    let mut map = std::collections::HashMap::new();

    // OmniM native codes
    let mut omnim = std::collections::HashMap::new();
    omnim.insert("PIPE-END".into(), ConnectorType::PipeEnd);
    omnim.insert("FLANGE".into(), ConnectorType::FlangeFace);
    omnim.insert("ELBOW".into(), ConnectorType::ElbowEnd);
    omnim.insert("TEE-RUN".into(), ConnectorType::TeeRun);
    omnim.insert("TEE-BRANCH".into(), ConnectorType::TeeBranch);
    omnim.insert("STRUCT-END".into(), ConnectorType::StructuralEnd);
    omnim.insert("STRUCT-BASE".into(), ConnectorType::StructuralBase);
    omnim.insert("NOZZLE".into(), ConnectorType::EquipmentNozzle);
    omnim.insert("CABLE".into(), ConnectorType::CableEnd);
    omnim.insert("DUCT".into(), ConnectorType::DuctEnd);
    omnim.insert("VALVE-END".into(), ConnectorType::ValveEnd);
    omnim.insert("VALVE-STEM".into(), ConnectorType::ValveStem);
    map.insert("OmniM".into(), omnim);

    // Civil3D Point Code → OmniM ConnectorType
    let mut c3d = std::collections::HashMap::new();
    c3d.insert("PIPE-END-C".into(), ConnectorType::PipeEnd);
    c3d.insert("PIPE-END-S".into(), ConnectorType::PipeEnd);
    c3d.insert("FLG-FACE".into(), ConnectorType::FlangeFace);
    c3d.insert("ELB-END".into(), ConnectorType::ElbowEnd);
    c3d.insert("TEE-RUN".into(), ConnectorType::TeeRun);
    c3d.insert("TEE-BR".into(), ConnectorType::TeeBranch);
    c3d.insert("COL-BASE".into(), ConnectorType::StructuralBase);
    c3d.insert("BEM-END".into(), ConnectorType::StructuralEnd);
    c3d.insert("NOZ".into(), ConnectorType::EquipmentNozzle);
    c3d.insert("CBL-END".into(), ConnectorType::CableEnd);
    c3d.insert("VALVE-END".into(), ConnectorType::ValveEnd);
    map.insert("C3D".into(), c3d);

    // E3D P-Point code → OmniM ConnectorType
    let mut e3d = std::collections::HashMap::new();
    e3d.insert("P-END".into(), ConnectorType::PipeEnd);
    e3d.insert("P-FLAN".into(), ConnectorType::FlangeFace);
    e3d.insert("P-ELB".into(), ConnectorType::ElbowEnd);
    e3d.insert("P-TEE".into(), ConnectorType::TeeRun);
    e3d.insert("P-TBR".into(), ConnectorType::TeeBranch);
    e3d.insert("P-STRU".into(), ConnectorType::StructuralEnd);
    e3d.insert("P-BASE".into(), ConnectorType::StructuralBase);
    e3d.insert("P-NOZ".into(), ConnectorType::EquipmentNozzle);
    e3d.insert("P-CBL".into(), ConnectorType::CableEnd);
    e3d.insert("P-VALVE".into(), ConnectorType::ValveEnd);
    e3d.insert("P-VSTEM".into(), ConnectorType::ValveStem);
    map.insert("E3D".into(), e3d);

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_catalog_store_basic() {
        let mut store = CatalogStore::new();
        let comp = ComponentDef::new("IPE240", "Structural/Steel/Beam");
        let id = comp.id.clone();
        store.insert(comp);
        assert_eq!(store.len(), 1);
        assert!(store.get(&id).is_some());
        assert_eq!(store.get(&id).unwrap().name, "IPE240");
    }

    #[test]
    fn test_catalog_store_by_category() {
        let mut store = CatalogStore::new();
        store.insert(ComponentDef::new("IPE240", "Structural/Steel/Beam"));
        store.insert(ComponentDef::new("HEA200", "Structural/Steel/Column"));
        store.insert(ComponentDef::new("Pipe-CS", "Piping/Pipe"));
        let structural = store.list_by_category("Structural");
        assert_eq!(structural.len(), 2);
    }
}
