use std::sync::RwLock;
use omnim_core as c;
use serde::{Deserialize, Serialize};

use crate::{AdministrativeStore, BreakdownStore, Feature, LifecycleStore, MaterialStore, MeasurementStore, ObjectStore, RelationStore, SeriesStore};
use crate::object::OmniObject;

/// Session-level global static Library, shared across all projects.
/// Project does not hold a Library reference; all access goes through this static.
pub static LIBRARY: RwLock<Option<crate::library::Library>> = RwLock::new(None);

#[derive(Serialize, Deserialize, Clone)]
pub struct Project {
    pub name: String,
    pub objects: ObjectStore,
    pub materials: MaterialStore,
    pub lifecycle: LifecycleStore,
    pub relations: RelationStore,
    pub administrative: AdministrativeStore,
    pub breakdown: BreakdownStore,
    pub sketches: omnim_sketch::SketchStore,
    #[serde(default)]
    pub series: SeriesStore,
    #[serde(default)]
    pub external_series_refs: Vec<SeriesRef>,
    #[serde(default)]
    pub measurements: MeasurementStore,
}

/// Reference to an external series (file or Library-provided).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SeriesRef {
    pub id: String,
    pub name: String,
    pub source: String,  // "standalone" | "library"
    pub path: Option<String>,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        let mut p = Self {
            name: name.into(),
            objects: ObjectStore::new(),
            materials: MaterialStore::new(),
            lifecycle: LifecycleStore::new(),
            relations: RelationStore::new(),
            administrative: AdministrativeStore::new(),
            breakdown: BreakdownStore::new(),
            sketches: omnim_sketch::SketchStore::new(),
            series: SeriesStore::new(),
            external_series_refs: Vec::new(),
            measurements: MeasurementStore::new(),
        };
        crate::seed_materials::seed_default_materials(&mut p.materials);
        p
    }

    pub fn create_object(&mut self, object_type: c::ObjectType, name: &str) -> &mut OmniObject {
        let obj = OmniObject::new(object_type, name);
        self.objects.insert(obj)
    }

    pub fn set_parent(&mut self, child_id: &str, parent_id: &str) -> Result<(), String> {
        if !self.objects.contains(child_id) {
            return Err(format!("object '{}' not found", child_id));
        }
        if !self.objects.contains(parent_id) {
            return Err(format!("parent '{}' not found", parent_id));
        }
        if let Some(obj) = self.objects.get_mut(child_id) {
            obj.parent_id = Some(parent_id.to_string());
        }
        self.objects.rebuild_children();
        self.relations.add_containment(c::Containment {
            parent_id: parent_id.to_string(),
            child_id: child_id.to_string(),
            r#type: c::ContainmentType::Spatial as i32,
        }).map(|_| ())
    }

    /// Series Gate: verify that a ComponentDef is reachable from at least one
    /// loaded series in the project.
    ///
    /// Behavior:
    /// - Series loaded + component reachable -> allowed (Ok)
    /// - Series loaded + component NOT reachable -> rejected (Err)
    /// - No series loaded -> open mode, all components allowed (Ok)
    pub fn check_series_gate(&self, component_def_id: &str, component_name: &str) -> Result<(), String> {
        if self.series.is_any_series_loaded() {
            // SpecComponent stores the component identifier which may be a UUID
            // or a name. Check both to be safe.
            if !self.series.is_component_reachable(component_def_id)
                && !self.series.is_component_reachable(component_name)
            {
                return Err(format!(
                    "series gate: component '{}' ({}) is not reachable from any loaded series. \
                    Load the appropriate series or use a series-approved component.",
                    component_name, component_def_id
                ));
            }
        }
        Ok(())
    }

    pub fn connect(&mut self, from_id: &str, to_id: &str, conn_type: c::ConnectionType) -> Result<(), String> {
        if !self.objects.contains(from_id) {
            return Err(format!("object '{}' not found", from_id));
        }
        if !self.objects.contains(to_id) {
            return Err(format!("object '{}' not found", to_id));
        }
        self.relations.add_connection(c::Connection {
            from_object_id: from_id.to_string(),
            to_object_id: to_id.to_string(),
            r#type: conn_type as i32,
            from_point: None,
            to_point: None,
            connection_detail: String::new(),
        });
        Ok(())
    }

    pub fn set_material(&mut self, object_id: &str, material_id: &str) -> Result<(), String> {
        if !self.objects.contains(object_id) {
            return Err(format!("object '{}' not found", object_id));
        }
        if !self.materials.get(material_id).is_some() {
            return Err(format!("material '{}' not found", material_id));
        }
        if let Some(obj) = self.objects.get_mut(object_id) {
            obj.material_id = Some(material_id.to_string());
        }
        Ok(())
    }

    pub fn add_lifecycle_record(&mut self, record: c::ProjectLifecycleRecord) -> &mut c::ProjectLifecycleRecord {
        self.lifecycle.insert(record)
    }

    pub fn link_record_to_object(&mut self, record_id: &str, object_id: &str) -> Result<(), String> {
        let record = self.lifecycle.get_mut(record_id)
            .ok_or_else(|| format!("record '{}' not found", record_id))?;
        if !record.related_object_ids.contains(&object_id.to_string()) {
            record.related_object_ids.push(object_id.to_string());
        }
        Ok(())
    }

    pub fn stats(&self) -> ProjectStats {
        ProjectStats {
            name: self.name.clone(),
            object_count: self.objects.len(),
            material_count: self.materials.len(),
            lifecycle_record_count: self.lifecycle.len(),
            connection_count: self.relations.connections().len(),
            containment_count: self.relations.containments().len(),
            administrative_division_count: self.administrative.len(),
            breakdown_node_count: self.breakdown.len(),
            sketch_count: self.sketches.len(),
            component_count: self.objects.find_by_type(c::ObjectType::Component).len(),
            catalog_component_count: LIBRARY.read().unwrap().as_ref().map(|l| l.catalog.len()).unwrap_or(0),
            series_count: self.series.len(),
        }
    }

    pub fn create_component(&mut self, name: &str, parent_id: Option<&str>) -> String {
        let mut obj = OmniObject::new(c::ObjectType::Component, name);
        let id = obj.id.clone();
        if let Some(pid) = parent_id {
            obj.parent_id = Some(pid.to_string());
        }
        self.objects.insert(obj);
        self.objects.rebuild_children();
        id
    }

    pub fn component_ids(&self) -> Vec<String> {
        self.objects.find_by_type(c::ObjectType::Component)
            .into_iter().map(|o| o.id.clone()).collect()
    }

    /// Instantiate a catalog component from a ComponentDef, creating a Component
    /// object with feature tree, sketches, and connectors.
    ///
    /// **Series Gate**: If the project has series loaded and the target ComponentDef
    /// is not reachable from any loaded series, the instantiation is rejected.
    /// If no series is loaded, all catalog components are allowed (open mode).
    ///
    /// Returns the new component's object ID on success.
    pub fn instantiate_catalog_component(
        &mut self,
        component_def_id: &str,
        instance_name: &str,
        param_overrides: &std::collections::HashMap<String, f64>,
        position: Option<c::Point3D>,
        rotation_angle: f64,
    ) -> Result<String, String> {
        self.instantiate_catalog_component_impl(component_def_id, instance_name, param_overrides, position, rotation_angle, true)
    }

    /// Same as `instantiate_catalog_component` but skips the series gate check.
    /// Used by domain-panel quick-create which bypasses series governance.
    pub fn instantiate_catalog_component_skip_gate(
        &mut self,
        component_def_id: &str,
        instance_name: &str,
        param_overrides: &std::collections::HashMap<String, f64>,
        position: Option<c::Point3D>,
        rotation_angle: f64,
    ) -> Result<String, String> {
        self.instantiate_catalog_component_impl(component_def_id, instance_name, param_overrides, position, rotation_angle, false)
    }

    fn instantiate_catalog_component_impl(
        &mut self,
        component_def_id: &str,
        instance_name: &str,
        param_overrides: &std::collections::HashMap<String, f64>,
        position: Option<c::Point3D>,
        rotation_angle: f64,
        check_gate: bool,
    ) -> Result<String, String> {
        let library = LIBRARY.read().unwrap();
        let library = library.as_ref().expect("LIBRARY not initialized");
        let comp_def = library.catalog.get(component_def_id)
            .or_else(|| library.catalog.find_by_name(component_def_id))
            .ok_or_else(|| format!("component def '{}' not found in catalog", component_def_id))?
            .clone();

        // Series Gate: check reachability
        if check_gate {
            self.check_series_gate(&comp_def.id, &comp_def.name)?;
        }

        let tree_id = comp_def.feature_tree_id.clone();
        if tree_id.is_empty() {
            return Err(format!("component def '{}' has no feature tree template", comp_def.name));
        }
        let template = library.feature_trees.get(&tree_id)
            .ok_or_else(|| format!("feature tree template '{}' not found", tree_id))?
            .clone();

        // Resolve full parameter set: use defaults from ComponentDef, overridden by provided values
        let mut params: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        for p in &comp_def.params {
            params.insert(p.name.clone(), p.default_value);
        }
        for (k, v) in param_overrides {
            params.insert(k.clone(), *v);
        }

        // Create the Component object
        let mut obj = OmniObject::new(c::ObjectType::Component, instance_name);
        let component_id = obj.id.clone();

        if let Some(pos) = position {
            obj.geometry = Some(c::Geometry {
                id: String::new(),
                r#type: c::GeometryType::Mesh as i32,
                bbox: None,
                level_of_detail: 0,
                representation: Some(c::geometry::Representation::Mesh(c::MeshGeometry {
                    vertices: vec![pos],
                    normals: Vec::new(),
                    uvs: Vec::new(),
                    faces: Vec::new(),
                })),
            });
        }
        obj.properties.push(c::Property {
            name: "catalog_component_id".to_string(),
            value: Some(c::PropertyValue {
                value_type: Some(c::property_value::ValueType::StringValue(component_def_id.to_string())),
            }),
            unit: String::new(),
            description: String::new(),
        });
        obj.properties.push(c::Property {
            name: "catalog_component_name".to_string(),
            value: Some(c::PropertyValue {
                value_type: Some(c::property_value::ValueType::StringValue(comp_def.name.clone())),
            }),
            unit: String::new(),
            description: String::new(),
        });
        // Record catalog reference for traceability
        obj.catalog_ref = Some(crate::catalog::CatalogRef {
            component_def_id: comp_def.id.clone(),
            series_id: String::new(),
            selector_path: Vec::new(),
            param_values: params.clone(),
        });
        self.objects.insert(obj);

        // Instantiate the feature tree
        crate::feature_template::instantiate_template(self, &component_id, &template, &params)?;

        // Rebuild the component to generate geometry
        self.rebuild_component(&component_id)?;

        // Apply the requested position — rebuild_component merged the feature
        // geometry at the sketch origin, so translate the resulting mesh.
        // Also apply rotation around Z if requested.
        if let Some(pos) = position {
            let obj = self.objects.get_mut(&component_id).unwrap();
            if let Some(ref mut geom) = obj.geometry {
                if let Some(c::geometry::Representation::Mesh(ref mut m)) = &mut geom.representation {
                    // Rotate around Z axis
                    if rotation_angle != 0.0 {
                        let (sin, cos) = rotation_angle.sin_cos();
                        for v in &mut m.vertices {
                            let x = v.x * cos - v.y * sin;
                            let y = v.x * sin + v.y * cos;
                            v.x = x; v.y = y;
                        }
                    }
                    // Translate to position (metres → mm)
                    for v in &mut m.vertices {
                        v.x += pos.x * 1000.0;
                        v.y += pos.y * 1000.0;
                        v.z += pos.z * 1000.0;
                    }
                    // Update bbox
                    if let Some(ref mut bb) = geom.bbox {
                        let (mut mn, mut mx) = (pos.clone(), pos.clone());
                        for v in &m.vertices {
                            mn.x = mn.x.min(v.x); mn.y = mn.y.min(v.y); mn.z = mn.z.min(v.z);
                            mx.x = mx.x.max(v.x); mx.y = mx.y.max(v.y); mx.z = mx.z.max(v.z);
                        }
                        *bb = c::BoundingBox { min: Some(mn), max: Some(mx) };
                    }
                }
            }
        }

        Ok(component_id)
    }

    /// Instantiate a component from a series with type-driven answers.
    ///
    /// `series_name`: name of the series (e.g. "CS150", "Steel-Q355").
    /// `type_answer`: the answer value matching the component type (e.g. "PIPE", "beam").
    /// `instance_name`: name for the new instance.
    /// `param_overrides`: parameter overrides.
    /// `position`: optional position.
    pub fn instantiate_from_series(
        &mut self,
        series_name: &str,
        type_answer: &str,
        instance_name: &str,
        param_overrides: &std::collections::HashMap<String, f64>,
        position: Option<c::Point3D>,
    ) -> Result<String, String> {
        let series = self.series.find_by_name(series_name)
            .ok_or_else(|| format!("series '{}' not found", series_name))?;

        let series_id = series.id.clone();
        let result = crate::series_store::SelectionEngine::select(series, &[type_answer])
            .map_err(|e| format!("series selection failed: {}", e))?;

        let library = LIBRARY.read().unwrap();
        let library = library.as_ref().expect("LIBRARY not initialized");
        let comp_def_id = library.catalog.get(&result.component_def_id)
            .or_else(|| library.catalog.find_by_name(&result.component_def_id))
            .ok_or_else(|| format!("component def '{}' not found in catalog (tried id and name)", result.component_def_id))?
            .id.clone();

        // Apply series param constraints as overrides
        let mut merged_params = param_overrides.clone();
        for constraint in &result.spec_component.param_constraints {
            if !merged_params.contains_key(&constraint.param_name) {
                merged_params.insert(constraint.param_name.clone(), constraint.default_value);
            }
        }

        let component_id = self.instantiate_catalog_component(&comp_def_id, instance_name, &merged_params, position, 0.0)?;

        // Enrich catalog_ref with series information
        if let Some(obj) = self.objects.get_mut(&component_id) {
            if let Some(ref mut cr) = obj.catalog_ref {
                cr.series_id = series_id;
                cr.selector_path = result.selector_path.clone();
            }
        }

        Ok(component_id)
    }

    /// Map an ObjectType to the series answer value for series selection.
    pub fn object_type_to_series_answer_pub(object_type: c::ObjectType) -> Option<&'static str> {
        Self::object_type_to_series_answer(object_type)
    }

    /// Map an ObjectType to the series answer value for series selection.
    fn object_type_to_series_answer(object_type: c::ObjectType) -> Option<&'static str> {
        match object_type {
            // Steel-Q355
            c::ObjectType::Beam | c::ObjectType::StructuralSteel => Some("beam"),
            c::ObjectType::Column => Some("column"),
            // Concrete-C30
            c::ObjectType::Wall => Some("wall"),
            c::ObjectType::Slab | c::ObjectType::Floor | c::ObjectType::Roof => Some("slab"),
            c::ObjectType::Door => Some("door"),
            c::ObjectType::Window => Some("window"),
            // CS150
            c::ObjectType::PipePlant | c::ObjectType::PipeCivil => Some("PIPE"),
            c::ObjectType::Fitting => Some("PIPE"),
            c::ObjectType::Valve => Some("VALVE"),
            // Highway-GB
            c::ObjectType::Road => Some("ASPHALT"),
            _ => None,
        }
    }

    /// Create an object from a series default template for the given ObjectType.
    pub fn create_object_from_series_template(
        &mut self,
        object_type: c::ObjectType,
        name: &str,
        param_overrides: &std::collections::HashMap<String, f64>,
        position: Option<c::Point3D>,
    ) -> Result<String, String> {
        let series_name = crate::gen_rule::default_series_for_type(object_type)
            .ok_or_else(|| format!("no default series for type {:?}", object_type))?;
        let answer = Self::object_type_to_series_answer(object_type)
            .ok_or_else(|| format!("no series answer for type {:?}", object_type))?;
        self.instantiate_from_series(series_name, answer, name, param_overrides, position)
    }

    /// Create an object from the default catalog template for the given ObjectType.
    ///
    /// Implements the "default template registry" behavior (per-OBJECT-TYPE template selection):
    /// `create wall` -> instantiates `Concrete-Wall` from the catalog.
    /// If no default template exists for this type, a plain object is created
    /// (no catalog reference, no feature tree).
    ///
    /// Returns the new object's ID.
    pub fn create_object_from_default_template(
        &mut self,
        object_type: c::ObjectType,
        name: &str,
        param_overrides: &std::collections::HashMap<String, f64>,
        position: Option<c::Point3D>,
    ) -> Result<String, String> {
        let default_name = crate::gen_rule::default_catalog_component(object_type);

        if let Some(comp_name) = default_name {
            let library = LIBRARY.read().unwrap();
        let library = library.as_ref().expect("LIBRARY not initialized");
            let comp_def_id = library.catalog.find_by_name(comp_name)
                .ok_or_else(|| format!("default catalog component '{}' not found", comp_name))?
                .id.clone();
            self.instantiate_catalog_component(&comp_def_id, name, param_overrides, position, 0.0)
        } else {
            let mut obj = OmniObject::new(object_type, name);
            if let Some(pos) = position {
                obj.geometry = Some(c::Geometry {
                    id: String::new(),
                    r#type: c::GeometryType::Mesh as i32,
                    bbox: None,
                    level_of_detail: 0,
                    representation: Some(c::geometry::Representation::Mesh(c::MeshGeometry {
                        vertices: vec![pos],
                        normals: Vec::new(),
                        uvs: Vec::new(),
                        faces: Vec::new(),
                    })),
                });
            }
            let id = obj.id.clone();
            self.objects.insert(obj);
            Ok(id)
        }
    }

    /// Validate parameters against a catalog component definition's parameter ranges.
    ///
    /// Returns a list of validation error messages (empty if all valid).
    pub fn validate_catalog_params(
        &self,
        component_def_id_or_name: &str,
        params: &std::collections::HashMap<String, f64>,
    ) -> Vec<String> {
        let mut errors = Vec::new();
        let library = LIBRARY.read().unwrap();
        let library = library.as_ref().expect("LIBRARY not initialized");
        let comp_def = match library.catalog.get(component_def_id_or_name)
            .or_else(|| library.catalog.find_by_name(component_def_id_or_name))
        {
            Some(d) => d,
            None => {
                errors.push(format!("component def '{}' not found in catalog", component_def_id_or_name));
                return errors;
            }
        };
        for param_def in &comp_def.params {
            if let Some(&value) = params.get(&param_def.name) {
                if value < param_def.min_value {
                    errors.push(format!(
                        "parameter '{}' value {} is below minimum {}",
                        param_def.name, value, param_def.min_value
                    ));
                }
                if value > param_def.max_value {
                    errors.push(format!(
                        "parameter '{}' value {} exceeds maximum {}",
                        param_def.name, value, param_def.max_value
                    ));
                }
                if param_def.step > 0.0 && param_def.param_type == 2 {
                    let remainder = (value - param_def.min_value) % param_def.step;
                    if remainder.abs() > 1e-6 && (param_def.step - remainder).abs() > 1e-6 {
                        errors.push(format!(
                            "parameter '{}' value {} is not a valid step (step={})",
                            param_def.name, value, param_def.step
                        ));
                    }
                }
            }
        }
        errors
    }

    /// Validate parameters against series param constraints.
    ///
    /// Returns a list of validation error messages (empty if all valid).
    pub fn validate_series_params(
        &self,
        series_name: &str,
        answer: &str,
        params: &std::collections::HashMap<String, f64>,
    ) -> Vec<String> {
        let mut errors = Vec::new();
        let series = match self.series.find_by_name(series_name) {
            Some(s) => s,
            None => {
                errors.push(format!("series '{}' not found", series_name));
                return errors;
            }
        };
        let selection = match crate::series_store::SelectionEngine::select(series, &[answer]) {
            Ok(r) => r,
            Err(e) => {
                errors.push(format!("series selection failed: {}", e));
                return errors;
            }
        };
        for constraint in &selection.spec_component.param_constraints {
            if let Some(&value) = params.get(&constraint.param_name) {
                if value < constraint.min_value {
                    errors.push(format!(
                        "parameter '{}' value {} is below series minimum {}",
                        constraint.param_name, value, constraint.min_value
                    ));
                }
                if value > constraint.max_value {
                    errors.push(format!(
                        "parameter '{}' value {} exceeds series maximum {}",
                        constraint.param_name, value, constraint.max_value
                    ));
                }
                if constraint.step > 0.0 {
                    let remainder = (value - constraint.min_value) % constraint.step;
                    if remainder.abs() > 1e-6 && (constraint.step - remainder).abs() > 1e-6 {
                        errors.push(format!(
                            "parameter '{}' value {} is not a valid series step (step={})",
                            constraint.param_name, value, constraint.step
                        ));
                    }
                }
            }
        }
        // Also validate against the underlying component def
        if !selection.spec_component.component_def_id.is_empty() {
            let comp_errors = self.validate_catalog_params(
                &selection.spec_component.component_def_id, params,
            );
            errors.extend(comp_errors);
        }
        errors
    }

    /// Find all objects that are instances of a given catalog component
    /// (identified by the `catalog_component_id` property).
    ///
    /// Returns a list of object IDs.
    pub fn find_catalog_instances(&self, component_def_id: &str) -> Vec<String> {
        self.objects.list()
            .iter()
            .filter(|obj| {
                obj.properties.iter().any(|p| {
                    p.name == "catalog_component_id"
                        && p.value.as_ref()
                            .and_then(|v| v.value_type.as_ref())
                            .map(|vt| match vt {
                                c::property_value::ValueType::StringValue(s) => s == component_def_id,
                                _ => false,
                            })
                            .unwrap_or(false)
                })
            })
            .map(|obj| obj.id.clone())
            .collect()
    }

    /// Rebuild all instances of a catalog component after the component def
    /// or its feature tree template has been modified.
    ///
    /// This implements the "存量更新" (instance update) behavior.
    pub fn rebuild_catalog_instances(&mut self, component_def_id: &str) -> Result<Vec<String>, String> {
        let instance_ids = self.find_catalog_instances(component_def_id);
        let mut rebuilt = Vec::new();
        for id in &instance_ids {
            match self.rebuild_component(id) {
                Ok(()) => rebuilt.push(id.clone()),
                Err(e) => eprintln!("Warning: failed to rebuild instance '{}': {}", id, e),
            }
        }
        Ok(rebuilt)
    }

    pub fn add_feature(&mut self, component_id: &str, feature: Feature) -> Result<String, String> {
        let object_type = {
            let obj = self.objects.get(component_id)
                .ok_or_else(|| format!("component '{}' not found", component_id))?;
            obj.object_type
        };
        if object_type != c::ObjectType::Component as i32 {
            return Err(format!("'{}' is not a Component", component_id));
        }
        let order = {
            let obj = self.objects.get(component_id).unwrap();
            obj.features.len() as i32
        };
        let mut f = feature;
        if f.id.is_empty() {
            f.id = uuid::Uuid::new_v4().to_string();
        }
        f.order = order;
        let fid = f.id.clone();
        let obj = self.objects.get_mut(component_id).unwrap();
        obj.features.push(f);
        obj.dirty = true;
        Ok(fid)
    }

    pub fn remove_feature(&mut self, component_id: &str, feature_id: &str) -> Result<Feature, String> {
        let obj = self.objects.get_mut(component_id)
            .ok_or_else(|| format!("component '{}' not found", component_id))?;
        let idx = obj.features.iter().position(|f| f.id == feature_id)
            .ok_or_else(|| format!("feature '{}' not found in component '{}'", feature_id, component_id))?;
        let removed = obj.features.remove(idx);
        // Re-order remaining features and mark dirty
        for (i, f) in obj.features.iter_mut().enumerate() {
            f.order = i as i32;
            f.mark_dirty();
        }
        obj.dirty = true;
        Ok(removed)
    }

    pub fn update_feature(
        &mut self,
        component_id: &str,
        feature_id: &str,
        sketch_id: Option<String>,
        axis_sketch_id: Option<String>,
        depth: Option<f64>,
        angle: Option<f64>,
        is_cut: Option<bool>,
        suppressed: Option<bool>,
        input_sketch_ids: Option<Vec<String>>,
        dependency_ids: Option<Vec<String>>,
        direction: Option<c::Vector3D>,
        axis_origin: Option<c::Point3D>,
        axis_direction: Option<c::Vector3D>,
        lod: Option<f64>,
        boolean_op: Option<crate::feature::BooleanOp>,
        pattern_count_x: Option<usize>,
        pattern_count_y: Option<usize>,
        pattern_count_z: Option<usize>,
    ) -> Result<(), String> {
        let obj = self.objects.get_mut(component_id)
            .ok_or_else(|| format!("component '{}' not found", component_id))?;
        let idx = obj.features.iter().position(|f| f.id == feature_id)
            .ok_or_else(|| format!("feature '{}' not found", feature_id))?;
        if let Some(sid) = sketch_id { obj.features[idx].sketch_id = sid; }
        if let Some(asid) = axis_sketch_id { obj.features[idx].axis_sketch_id = asid; }
        if let Some(d) = depth { obj.features[idx].depth = d; }
        if let Some(a) = angle { obj.features[idx].angle = a; }
        if let Some(c) = is_cut { obj.features[idx].is_cut = c; }
        if let Some(s) = suppressed { obj.features[idx].suppressed = s; }
        if let Some(ids) = input_sketch_ids { obj.features[idx].input_sketch_ids = ids; }
        if let Some(ids) = dependency_ids { obj.features[idx].dependency_ids = ids; }
        if let Some(d) = direction { obj.features[idx].direction = d; }
        if let Some(o) = axis_origin { obj.features[idx].axis_origin = o; }
        if let Some(d) = axis_direction { obj.features[idx].axis_direction = d; }
        if let Some(l) = lod { obj.features[idx].lod = l; }
        if let Some(b) = boolean_op { obj.features[idx].boolean_op = Some(b); }
        if let Some(c) = pattern_count_x { obj.features[idx].pattern_count_x = Some(c); }
        if let Some(c) = pattern_count_y { obj.features[idx].pattern_count_y = Some(c); }
        if let Some(c) = pattern_count_z { obj.features[idx].pattern_count_z = Some(c); }
        obj.features[idx].mark_dirty();
        // Cascade to downstream features
        let order = obj.features[idx].order;
        for f in obj.features.iter_mut().skip(idx + 1) {
            if f.order > order {
                f.mark_dirty();
            }
        }
        obj.dirty = true;
        Ok(())
    }

    pub fn get_features(&self, component_id: &str) -> Result<&Vec<Feature>, String> {
        let obj = self.objects.get(component_id)
            .ok_or_else(|| format!("component '{}' not found", component_id))?;
        Ok(&obj.features)
    }

    pub fn rebuild_feature(&mut self, component_id: &str, feature_idx: usize) -> Result<(), String> {
        let obj = self.objects.get(component_id)
            .ok_or_else(|| format!("component '{}' not found", component_id))?;
        if feature_idx >= obj.features.len() {
            return Err(format!("feature index {} out of range", feature_idx));
        }

        let feature_type = obj.features[feature_idx].feature_type.clone();
        let sketch_id = obj.features[feature_idx].sketch_id.clone();
        let _input_sketch_ids = obj.features[feature_idx].input_sketch_ids.clone();
        let axis_sketch_id = obj.features[feature_idx].axis_sketch_id.clone();
        let depth = obj.features[feature_idx].depth;
        let direction = obj.features[feature_idx].direction.clone();
        let angle = obj.features[feature_idx].angle;
        let _axis_origin = obj.features[feature_idx].axis_origin.clone();
        let suppressed = obj.features[feature_idx].suppressed;
        let lod = obj.features[feature_idx].lod;

        if suppressed {
            let obj = self.objects.get_mut(component_id).unwrap();
            obj.features[feature_idx].cached_geometry = None;
            obj.features[feature_idx].dirty = false;
            return Ok(());
        }

        // Auto-solve any unsolved sketches before geometry generation
        {
            let sketch_ids: Vec<String> = {
                let mut ids = vec![sketch_id.clone()];
                if !axis_sketch_id.is_empty() {
                    ids.push(axis_sketch_id.clone());
                }
                ids
            };
            for sid in &sketch_ids {
                if let Some(sketch) = self.sketches.get_mut(sid) {
                    if !sketch.solved {
                        omnim_sketch::solver::solve(sketch);
                    }
                }
            }
        }

        let geometry = match feature_type {
            crate::FeatureType::Extrude => {
                if let Some(sketch) = self.sketches.get(&sketch_id) {
                    if let Some(profile2d) = omnim_sketch::profile::sketch_to_profile_2d(sketch) {
                        if profile2d.len() >= 3 {
                            let profile3d: Vec<c::Point3D> = profile2d.iter()
                                .map(|p| c::Point3D { x: p.x, y: p.y, z: 0.0 })
                                .collect();
                            crate::brep::extrude_brep(&profile3d, depth, direction, lod)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            crate::FeatureType::Revolve => {
                let mut resolved_origin = c::Point3D { x: 0.0, y: 0.0, z: 0.0 };
                let mut resolved_dir = c::Vector3D { x: 0.0, y: 0.0, z: 1.0 };
                if !axis_sketch_id.is_empty() {
                    if let Some(axis_sketch) = self.sketches.get(&axis_sketch_id) {
                        if let Some(line_entity) = axis_sketch.entities.iter().find(|e| matches!(e.entity_type, omnim_sketch::SketchEntityType::Line)) {
                            if line_entity.points.len() >= 2 {
                                let start = &line_entity.points[0];
                                let end = &line_entity.points[1];
                                let dx = end.x - start.x;
                                let dy = end.y - start.y;
                                let dz = end.z - start.z;
                                let len = (dx * dx + dy * dy + dz * dz).sqrt();
                                if len > 1e-10 {
                                    resolved_origin = c::Point3D { x: start.x, y: start.y, z: start.z };
                                    resolved_dir = c::Vector3D { x: dx / len, y: dy / len, z: dz / len };
                                }
                            }
                        }
                    }
                }
                if let Some(sketch) = self.sketches.get(&sketch_id) {
                    if let Some(profile2d) = omnim_sketch::profile::sketch_to_profile_2d(sketch) {
                        if profile2d.len() >= 3 {
                            let profile3d: Vec<c::Point3D> = profile2d.iter()
                                .map(|p| c::Point3D { x: p.x, y: p.y, z: 0.0 })
                                .collect();
                            crate::brep::revolve_brep(&profile3d, angle, resolved_origin, resolved_dir, lod)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            crate::FeatureType::Sweep => {
                let path_sketch_id = obj.features[feature_idx].path_sketch_id()
                    .map(|s| s.to_string());
                let path_sid = match path_sketch_id {
                    Some(id) => id,
                    None => return Err("sweep feature requires a path sketch".into()),
                };
                let profile_sketch = self.sketches.get(&sketch_id);
                let path_sketch = self.sketches.get(&path_sid);
                if let (Some(ps), Some(pp)) = (profile_sketch, path_sketch) {
                    let profile2d = omnim_sketch::profile::sketch_to_profile_2d(ps);
                    let path_points: Vec<c::Point3D> = pp.entities.iter()
                        .flat_map(|e| e.points.iter().cloned())
                        .collect();
                    if let (Some(prof), true) = (profile2d, path_points.len() >= 2) {
                        if prof.len() >= 3 {
                            let profile3d: Vec<c::Point3D> = prof.iter()
                                .map(|p| c::Point3D { x: p.x, y: p.y, z: 0.0 })
                                .collect();
                            crate::brep::sweep_brep(&profile3d, &path_points, lod)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            crate::FeatureType::Loft => {
                let section_ids = obj.features[feature_idx].section_sketch_ids();
                if section_ids.len() >= 2 {
                    let p0 = self.sketches.get(&section_ids[0])
                        .and_then(|s| omnim_sketch::profile::sketch_to_profile_2d(s));
                    let p1 = self.sketches.get(&section_ids[1])
                        .and_then(|s| omnim_sketch::profile::sketch_to_profile_2d(s));
                    if let (Some(prof0), Some(prof1)) = (p0, p1) {
                        if prof0.len() >= 3 && prof1.len() >= 3 {
                            let profile0: Vec<c::Point3D> = prof0.iter()
                                .map(|p| c::Point3D { x: p.x, y: p.y, z: 0.0 })
                                .collect();
                            let profile1: Vec<c::Point3D> = prof1.iter()
                                .map(|p| c::Point3D { x: p.x, y: p.y, z: 0.0 })
                                .collect();
                            crate::brep::loft_brep(&profile0, &profile1, lod)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            crate::FeatureType::Fusion => {
                // Fusion merges geometries from multiple input sketches.
                // Each input_sketch is extruded with the feature's depth.
                if let Some(sketch) = self.sketches.get(&sketch_id) {
                    if let Some(profile2d) = omnim_sketch::profile::sketch_to_profile_2d(sketch) {
                        if profile2d.len() >= 3 {
                            let profile3d: Vec<c::Point3D> = profile2d.iter()
                                .map(|p| c::Point3D { x: p.x, y: p.y, z: 0.0 })
                                .collect();
                            crate::brep::extrude_brep(&profile3d, depth, direction, lod)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            crate::FeatureType::LoftedFusion => {
                let section_ids = obj.features[feature_idx].section_sketch_ids();
                if section_ids.len() >= 2 {
                    let p0 = self.sketches.get(&section_ids[0])
                        .and_then(|s| omnim_sketch::profile::sketch_to_profile_2d(s));
                    let p1 = self.sketches.get(&section_ids[1])
                        .and_then(|s| omnim_sketch::profile::sketch_to_profile_2d(s));
                    if let (Some(prof0), Some(prof1)) = (p0, p1) {
                        if prof0.len() >= 3 && prof1.len() >= 3 {
                            let profile0: Vec<c::Point3D> = prof0.iter()
                                .map(|p| c::Point3D { x: p.x, y: p.y, z: 0.0 })
                                .collect();
                            let profile1: Vec<c::Point3D> = prof1.iter()
                                .map(|p| c::Point3D { x: p.x, y: p.y, z: 0.0 })
                                .collect();
                            crate::brep::loft_brep(&profile0, &profile1, lod)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            crate::FeatureType::Boolean => {
                // Boolean subtract: generate the operand mesh from the sketch,
                // return it as cached_geometry. The merge logic in rebuild_component
                // will subtract it from the accumulated mesh.
                // Boolean union: just concatenate (same as non-boolean).
                if obj.features[feature_idx].is_cut {
                    // Subtract mode: generate the subtraction volume
                    if let Some(sketch) = self.sketches.get(&sketch_id) {
                        if let Some(profile2d) = omnim_sketch::profile::sketch_to_profile_2d(sketch) {
                            if profile2d.len() >= 3 {
                                let profile3d: Vec<c::Point3D> = profile2d.iter()
                                    .map(|p| c::Point3D { x: p.x, y: p.y, z: 0.0 })
                                    .collect();
                                let dir = direction;
                                // Generate the operand mesh (will be subtracted during merge)
                                crate::brep::extrude_brep(&profile3d, depth, dir, lod)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    // Union mode: generate the union operand as normal geometry
                    if let Some(sketch) = self.sketches.get(&sketch_id) {
                        if let Some(profile2d) = omnim_sketch::profile::sketch_to_profile_2d(sketch) {
                            if profile2d.len() >= 3 {
                                let profile3d: Vec<c::Point3D> = profile2d.iter()
                                    .map(|p| c::Point3D { x: p.x, y: p.y, z: 0.0 })
                                    .collect();
                            crate::brep::extrude_brep(&profile3d, depth, direction, lod)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }
            }
            crate::FeatureType::Chamfer
            | crate::FeatureType::Fillet
            | crate::FeatureType::Pattern
            | crate::FeatureType::Shell => {
                // Compute from dependent features' cached geometry
                let deps = obj.features[feature_idx].dependency_ids.clone();
                let mut dep_meshes: Vec<c::MeshGeometry> = Vec::new();
                for (fi, f) in obj.features.iter().enumerate() {
                    if fi != feature_idx && deps.contains(&f.id) {
                        if let Some(ref g) = f.cached_geometry {
                            dep_meshes.push(g.clone());
                        }
                    }
                }
                // Merge all dependent meshes
                let merged = if dep_meshes.is_empty() {
                    None
                } else {
                    let mut acc = dep_meshes.remove(0);
                    for m in dep_meshes {
                        let offset = acc.vertices.len() as i32;
                        let mut new_faces = m.faces.clone();
                        for face in &mut new_faces {
                            for idx in &mut face.vertex_indices {
                                *idx += offset;
                            }
                        }
                        acc.vertices.extend(m.vertices);
                        acc.normals.extend(m.normals);
                        acc.faces.extend(new_faces);
                    }
                    Some(acc)
                };

                match feature_type {
                    crate::FeatureType::Chamfer => {
                        // Basic chamfer: offset vertices along edges by depth/2
                        if let Some(mut mesh) = merged {
                            let dist = depth * 0.5;
                            for v in &mut mesh.vertices {
                                // Simple inward offset along average normal direction
                                v.x += v.x.signum() * dist * 0.1;
                                v.y += v.y.signum() * dist * 0.1;
                                v.z += v.z.signum() * dist * 0.1;
                            }
                            // Recompute normals
                            mesh.normals = Self::recompute_normals(&mesh);
                            Some(mesh)
                        } else {
                            None
                        }
                    }
                    crate::FeatureType::Fillet => {
                        // Basic fillet: subdivide faces (split each triangle into 4)
                        if let Some(mesh) = merged {
                            let mut new_verts = Vec::new();
                            let mut new_faces = Vec::new();
                            for face in &mesh.faces {
                                let i0 = face.vertex_indices[0] as usize;
                                let i1 = face.vertex_indices[1] as usize;
                                let i2 = face.vertex_indices[2] as usize;
                                let v0 = &mesh.vertices[i0];
                                let v1 = &mesh.vertices[i1];
                                let v2 = &mesh.vertices[i2];
                                let m0 = c::Point3D {
                                    x: (v0.x + v1.x) * 0.5,
                                    y: (v0.y + v1.y) * 0.5,
                                    z: (v0.z + v1.z) * 0.5,
                                };
                                let m1 = c::Point3D {
                                    x: (v1.x + v2.x) * 0.5,
                                    y: (v1.y + v2.y) * 0.5,
                                    z: (v1.z + v2.z) * 0.5,
                                };
                                let m2 = c::Point3D {
                                    x: (v0.x + v2.x) * 0.5,
                                    y: (v0.y + v2.y) * 0.5,
                                    z: (v0.z + v2.z) * 0.5,
                                };
                                let base = new_verts.len() as i32;
                                new_verts.push(v0.clone());
                                new_verts.push(v1.clone());
                                new_verts.push(v2.clone());
                                new_verts.push(m0);
                                new_verts.push(m1);
                                new_verts.push(m2);
                                // 4 sub-triangles
                                new_faces.push(c::FaceIndices { vertex_indices: vec![base, base + 3, base + 5] });
                                new_faces.push(c::FaceIndices { vertex_indices: vec![base + 3, base + 1, base + 4] });
                                new_faces.push(c::FaceIndices { vertex_indices: vec![base + 5, base + 4, base + 2] });
                                new_faces.push(c::FaceIndices { vertex_indices: vec![base + 3, base + 4, base + 5] });
                            }
                            // Push vertices inward slightly for rounded look
                            let r = depth * 0.3;
                            let (cx, cy, cz) = {
                                let n = new_verts.len() as f64;
                                (
                                    new_verts.iter().map(|p| p.x).sum::<f64>() / n,
                                    new_verts.iter().map(|p| p.y).sum::<f64>() / n,
                                    new_verts.iter().map(|p| p.z).sum::<f64>() / n,
                                )
                            };
                            for v in &mut new_verts {
                                let dx = v.x - cx;
                                let dy = v.y - cy;
                                let dz = v.z - cz;
                                let len = (dx * dx + dy * dy + dz * dz).sqrt();
                                if len > 1e-10 {
                                    v.x -= dx / len * r;
                                    v.y -= dy / len * r;
                                    v.z -= dz / len * r;
                                }
                            }
                            let mut result = c::MeshGeometry {
                                vertices: new_verts,
                                normals: Vec::new(),
                                uvs: Vec::new(),
                                faces: new_faces,
                            };
                            result.normals = Self::recompute_normals(&result);
                            Some(result)
                        } else {
                            None
                        }
                    }
                    crate::FeatureType::Pattern => {
                        // Array pattern: copy merged geometry with spacing
                        if let Some(base_mesh) = merged {
                            let cx = obj.features[feature_idx].pattern_count_x.unwrap_or(1);
                            let cy = obj.features[feature_idx].pattern_count_y.unwrap_or(1);
                            let cz = obj.features[feature_idx].pattern_count_z.unwrap_or(1);
                            let sx = depth;
                            let sy = direction.y;
                            let sz = direction.z;
                            let mut result = base_mesh.clone();
                            for ix in 0..cx.max(1) {
                                for iy in 0..cy.max(1) {
                                    for iz in 0..cz.max(1) {
                                        if ix == 0 && iy == 0 && iz == 0 { continue; }
                                        let tx = ix as f64 * sx;
                                        let ty = iy as f64 * sy;
                                        let tz = iz as f64 * sz;
                                        let offset = result.vertices.len() as i32;
                                        for v in &base_mesh.vertices {
                                            result.vertices.push(c::Point3D {
                                                x: v.x + tx, y: v.y + ty, z: v.z + tz,
                                            });
                                        }
                                        result.normals.extend(base_mesh.normals.iter().cloned());
                                        for face in &base_mesh.faces {
                                            let mut new_face = face.clone();
                                            for idx in &mut new_face.vertex_indices {
                                                *idx += offset;
                                            }
                                            result.faces.push(new_face);
                                        }
                                    }
                                }
                            }
                            Some(result)
                        } else {
                            None
                        }
                    }
                    crate::FeatureType::Shell => {
                        // Shell: offset vertices inward along normals, generate inner faces
                        if let Some(mut mesh) = merged {
                            let thick = depth;
                            let outer_verts = mesh.vertices.clone();
                            mesh.normals = Self::recompute_normals(&mesh);
                            // Offset vertices inward
                            for (i, v) in mesh.vertices.iter_mut().enumerate() {
                                if i < mesh.normals.len() {
                                    let n = &mesh.normals[i];
                                    v.x -= n.x * thick;
                                    v.y -= n.y * thick;
                                    v.z -= n.z * thick;
                                }
                            }
                            // Combine outer shell + inner shell
                            let inner_offset = outer_verts.len() as i32;
                            let original_faces: Vec<c::FaceIndices> = mesh.faces.clone();
                            // Add inner faces (reversed winding)
                            for face in &original_faces {
                                let mut inner_face = face.clone();
                                // Reverse winding for inward-facing normals
                                inner_face.vertex_indices.reverse();
                                for idx in &mut inner_face.vertex_indices {
                                    *idx += inner_offset;
                                }
                                mesh.faces.push(inner_face);
                            }
                            // Add original vertices (outer shell) at the end
                            for v in &outer_verts {
                                mesh.vertices.push(v.clone());
                            }
                            // Add side faces connecting outer to inner
                            for face in &original_faces {
                                for w in 0..3 {
                                    let a = face.vertex_indices[w];
                                    let b = face.vertex_indices[(w + 1) % 3];
                                    let c = b + inner_offset;
                                    let d = a + inner_offset;
                                    mesh.faces.push(c::FaceIndices {
                                        vertex_indices: vec![a, b, c, d],
                                    });
                                }
                            }
                            mesh.normals = Self::recompute_normals(&mesh);
                            Some(mesh)
                        } else {
                            None
                        }
                    }
                    _ => {
                        eprintln!("Warning: rebuild_feature encountered unknown FeatureType variant — returning None geometry");
                        None
                    }
                }
            }
        };

        let obj = self.objects.get_mut(component_id).unwrap();
        obj.features[feature_idx].cached_geometry = geometry;
        obj.features[feature_idx].dirty = false;
        Ok(())
    }

    /// Recompute vertex normals for a mesh via face-weighted averaging.
    pub fn recompute_normals(mesh: &c::MeshGeometry) -> Vec<c::Vector3D> {
        let mut normals = vec![c::Vector3D { x: 0.0, y: 0.0, z: 0.0 }; mesh.vertices.len()];
        for face in &mesh.faces {
            if face.vertex_indices.len() < 3 { continue; }
            let i0 = face.vertex_indices[0] as usize;
            let i1 = face.vertex_indices[1] as usize;
            let i2 = face.vertex_indices[2] as usize;
            if i0 >= mesh.vertices.len() || i1 >= mesh.vertices.len() || i2 >= mesh.vertices.len() { continue; }
            let v0 = &mesh.vertices[i0];
            let v1 = &mesh.vertices[i1];
            let v2 = &mesh.vertices[i2];
            let e1x = v1.x - v0.x; let e1y = v1.y - v0.y; let e1z = v1.z - v0.z;
            let e2x = v2.x - v0.x; let e2y = v2.y - v0.y; let e2z = v2.z - v0.z;
            let nx = e1y * e2z - e1z * e2y;
            let ny = e1z * e2x - e1x * e2z;
            let nz = e1x * e2y - e1y * e2x;
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            if len > 1e-10 {
                for idx in &face.vertex_indices {
                    let i = *idx as usize;
                    if i < normals.len() {
                        normals[i].x += nx / len;
                        normals[i].y += ny / len;
                        normals[i].z += nz / len;
                    }
                }
            }
        }
        for n in &mut normals {
            let len = (n.x * n.x + n.y * n.y + n.z * n.z).sqrt();
            if len > 1e-10 {
                n.x /= len; n.y /= len; n.z /= len;
            }
        }
        normals
    }

    /// Mark all features that depend on the given feature as dirty.
    pub fn propagate_dirty(&mut self, component_id: &str, feature_id: &str) {
        if let Some(obj) = self.objects.get_mut(component_id) {
            for f in obj.features.iter_mut() {
                if f.dependency_ids.contains(&feature_id.to_string()) {
                    f.mark_dirty();
                }
            }
        }
    }

    pub fn rebuild_component(&mut self, component_id: &str) -> Result<(), String> {
        let feature_count = {
            let obj = self.objects.get(component_id)
                .ok_or_else(|| format!("component '{}' not found", component_id))?;
            obj.features.len()
        };

        for i in 0..feature_count {
            let needs_rebuild = {
                let obj = self.objects.get(component_id).unwrap();
                obj.features[i].dirty || obj.features[i].cached_geometry.is_none()
            };
            if needs_rebuild {
                self.rebuild_feature(component_id, i)?;
            }
        }

        let merged = {
            let obj = self.objects.get(component_id).unwrap();
            let mut accumulated: Option<c::MeshGeometry> = None;

            for f in &obj.features {
                if let Some(ref geom) = f.cached_geometry {
                    if f.feature_type == crate::FeatureType::Boolean {
                        let op = f.boolean_op.unwrap_or(crate::BooleanOp::Subtract);
                        match op {
                            crate::BooleanOp::Subtract => {
                                // Boolean subtract: remove geometry from accumulated
                                if let Some(ref acc) = accumulated {
                                    if !geom.vertices.is_empty() && !acc.vertices.is_empty() {
                                        let result = crate::gen_rule::boolean_subtract::generate(
                                            &crate::gen_rule::boolean_subtract::BooleanSubtractInput {
                                                host_mesh: acc.clone(),
                                                subtract_mesh: geom.clone(),
                                            },
                                        );
                                        accumulated = Some(result);
                                    }
                                }
                            }
                            crate::BooleanOp::Intersect => {
                                // Boolean intersect: keep only overlapping triangles
                                if let Some(ref acc) = accumulated {
                                    if !geom.vertices.is_empty() && !acc.vertices.is_empty() {
                                        let result = boolean_intersect(acc, geom);
                                        accumulated = Some(result);
                                    }
                                }
                            }
                            crate::BooleanOp::Union => {
                                // Boolean union: concatenate geometry
                                if !geom.vertices.is_empty() {
                                    accumulated = Some(match accumulated {
                                        Some(mut acc) => {
                                            let offset = acc.vertices.len() as i32;
                                            let mut new_faces = geom.faces.clone();
                                            for face in &mut new_faces {
                                                for idx in &mut face.vertex_indices {
                                                    *idx += offset;
                                                }
                                            }
                                            acc.vertices.extend(geom.vertices.clone());
                                            acc.normals.extend(geom.normals.clone());
                                            acc.faces.append(&mut new_faces);
                                            acc
                                        }
                                        None => geom.clone(),
                                    });
                                }
                            }
                        }
                    } else if !geom.vertices.is_empty() {
                        // Normal geometry: concatenate
                        accumulated = Some(match accumulated {
                            Some(mut acc) => {
                                let offset = acc.vertices.len() as i32;
                                let mut new_faces = geom.faces.clone();
                                for face in &mut new_faces {
                                    for idx in &mut face.vertex_indices {
                                        *idx += offset;
                                    }
                                }
                                acc.vertices.extend(geom.vertices.iter().cloned());
                                acc.normals.extend(geom.normals.iter().cloned());
                                acc.faces.extend(new_faces);
                                acc
                            }
                            None => geom.clone(),
                        });
                    }
                }
            }

            accumulated
        };

        let obj = self.objects.get_mut(component_id).unwrap();
        if let Some(ref mut geom) = obj.geometry {
            if let Some(c::geometry::Representation::Mesh(_)) = &geom.representation {
                // Update existing mesh geometry
            }
        }
        if let Some(ref mut geom) = obj.geometry {
            if let Some(ref m) = merged {
                geom.representation = Some(c::geometry::Representation::Mesh(m.clone()));
            }
        } else if let Some(ref m) = merged {
            obj.geometry = Some(crate::geometry::mesh_to_geometry(m));
        }

        obj.dirty = false;
        Ok(())
    }

    pub fn save(&self, path: &str) -> Result<(), String> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("serialize error: {}", e))?;
        std::fs::write(path, json)
            .map_err(|e| format!("write error: {}", e))?;
        Ok(())
    }

    pub fn load(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("read error: {}", e))?;
        let mut proj: Self = serde_json::from_str(&content)
            .map_err(|e| format!("deserialize error: {}", e))?;
        for obj in proj.objects.list_mut() {
            for f in obj.features.iter_mut() {
                if f.id.is_empty() {
                    f.id = uuid::Uuid::new_v4().to_string();
                }
            }
        }
        proj.objects.rebuild_children();
        // Backward-compatible: ensure old project files get the default material
        // library so the material browser and layer editor have data.
        if proj.materials.is_empty() {
            crate::seed_materials::seed_default_materials(&mut proj.materials);
        }
        // Auto-load external series referenced in the project file
        proj.load_series_from_refs();
        Ok(proj)
    }

    pub fn load_geom(path: &str) -> Result<omnim_core::ParametricGeometry, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("read error: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("deserialize error: {}", e))
    }

    /// Load external series referenced in `external_series_refs` into the
    /// project's private SeriesStore.
    ///
    /// For each ref with `source == "standalone"` and a valid path, the series
    /// file is loaded and merged. The priority logic in `SeriesStore::insert()`
    /// ensures project-private series take precedence over external ones.
    ///
    /// This should be called after `Project::load()` to restore external series.
    pub fn load_series_from_refs(&mut self) {
        let refs = self.external_series_refs.clone();
        for sr in &refs {
            if sr.source == "standalone" {
                if let Some(path) = &sr.path {
                    match crate::series_store::SeriesStore::load_from_file(path) {
                        Ok(ext_store) => {
                            for s in ext_store.list() {
                                // insert() handles priority - won't override project series
                                self.series.insert(s.clone());
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: failed to load series from '{}': {}", path, e);
                        }
                    }
                }
            }
        }
    }
}

/// Boolean intersection: keep triangles from host that fall within the intersect mesh's bounding box.
fn boolean_intersect(host: &c::MeshGeometry, intersect_mesh: &c::MeshGeometry) -> c::MeshGeometry {
    use c::Point3D;
    let mut min = Point3D { x: f64::MAX, y: f64::MAX, z: f64::MAX };
    let mut max = Point3D { x: f64::NEG_INFINITY, y: f64::NEG_INFINITY, z: f64::NEG_INFINITY };
    for v in &intersect_mesh.vertices {
        min.x = min.x.min(v.x); min.y = min.y.min(v.y); min.z = min.z.min(v.z);
        max.x = max.x.max(v.x); max.y = max.y.max(v.y); max.z = max.z.max(v.z);
    }
    // Expand bbox slightly for numerical tolerance
    let eps = 0.001;
    min.x -= eps; min.y -= eps; min.z -= eps;
    max.x += eps; max.y += eps; max.z += eps;

    let mut result = c::MeshGeometry {
        vertices: Vec::new(),
        normals: Vec::new(),
        uvs: Vec::new(),
        faces: Vec::new(),
    };

    for face in &host.faces {
        if face.vertex_indices.len() < 3 { continue; }
        let i0 = face.vertex_indices[0] as usize;
        let i1 = face.vertex_indices[1] as usize;
        let i2 = face.vertex_indices[2] as usize;
        if i0 >= host.vertices.len() || i1 >= host.vertices.len() || i2 >= host.vertices.len() { continue; }
        let v0 = &host.vertices[i0];
        let v1 = &host.vertices[i1];
        let v2 = &host.vertices[i2];
        let cx = (v0.x + v1.x + v2.x) / 3.0;
        let cy = (v0.y + v1.y + v2.y) / 3.0;
        let cz = (v0.z + v1.z + v2.z) / 3.0;
        // Keep triangle if its centroid is inside the intersect bbox
        if cx >= min.x && cx <= max.x && cy >= min.y && cy <= max.y && cz >= min.z && cz <= max.z {
            let base = result.vertices.len() as i32;
            result.vertices.push(v0.clone());
            result.vertices.push(v1.clone());
            result.vertices.push(v2.clone());
            if i0 < host.normals.len() { result.normals.push(host.normals[i0].clone()); }
            if i1 < host.normals.len() { result.normals.push(host.normals[i1].clone()); }
            if i2 < host.normals.len() { result.normals.push(host.normals[i2].clone()); }
            result.faces.push(c::FaceIndices {
                vertex_indices: vec![base, base + 1, base + 2],
            });
        }
    }
    result
}

#[derive(Debug)]
pub struct ProjectStats {
    pub name: String,
    pub object_count: usize,
    pub material_count: usize,
    pub lifecycle_record_count: usize,
    pub connection_count: usize,
    pub containment_count: usize,
    pub administrative_division_count: usize,
    pub breakdown_node_count: usize,
    pub sketch_count: usize,
    pub component_count: usize,
    pub catalog_component_count: usize,
    pub series_count: usize,
}

impl std::fmt::Display for ProjectStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Project: {}", self.name)?;
        writeln!(f, "  Objects:     {}", self.object_count)?;
        writeln!(f, "  Materials:   {}", self.material_count)?;
        writeln!(f, "  Lifecycle:   {}", self.lifecycle_record_count)?;
        writeln!(f, "  Connections: {}", self.connection_count)?;
        writeln!(f, "  Containments: {}", self.containment_count)?;
        writeln!(f, "  Admin Divisions: {}", self.administrative_division_count)?;
        writeln!(f, "  Breakdown Nodes: {}", self.breakdown_node_count)?;
        writeln!(f, "  Sketches:    {}", self.sketch_count)?;
        writeln!(f, "  Components:  {}", self.component_count)?;
        writeln!(f, "  Catalog Defs: {}", self.catalog_component_count)?;
        writeln!(f, "  Series:      {}", self.series_count)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static LIBRARY_INIT: Once = Once::new();

    /// Initialize the global LIBRARY static with seeded data if not already set.
    /// Tests that access catalog/series functionality must call this first.
    fn setup_library() {
        LIBRARY_INIT.call_once(|| {
            *LIBRARY.write().unwrap() = Some(crate::library::Library::seeded());
        });
    }

    /// Seed the project's private SeriesStore with built-in series.
    fn setup_series(proj: &mut Project) {
        crate::seed_catalog::seed_series(&mut proj.series);
    }

    #[test]
    fn test_project_workflow() {
        let mut proj = Project::new("TestProject");

        let site = proj.create_object(c::ObjectType::Site, "Site-A");
        let site_id = site.id.clone();
        let bldg = proj.create_object(c::ObjectType::Building, "Building-1");
        let bldg_id = bldg.id.clone();
        proj.set_parent(&bldg_id, &site_id).unwrap();

        let wall = proj.create_object(c::ObjectType::Wall, "Wall-1");
        let wall_id = wall.id.clone();

        proj.set_parent(&wall_id, &bldg_id).unwrap();

        let mat = c::Material {
            id: "mat-concrete".into(),
            name: "C30 Concrete".into(),
            category: c::MaterialCategory::Concrete as i32,
            density: 2400.0,
            ..Default::default()
        };
        proj.materials.insert(mat);
        proj.set_material(&wall_id, "mat-concrete").unwrap();

        let stats = proj.stats();
        assert_eq!(stats.object_count, 3);
        // Project::new seeds a default material library (22 built-ins) + 1 inserted.
        assert_eq!(stats.material_count, crate::seed_materials::seed_default_materials_len() + 1);
        assert_eq!(stats.containment_count, 2);
    }

    #[test]
    fn test_create_wall_from_default_template() {
        setup_library();
        let mut proj = Project::new("TestProject");

        let params = std::collections::HashMap::new();
        let wall_id = proj.create_object_from_default_template(
            c::ObjectType::Wall, "Wall-1", &params, None,
        ).unwrap();

        let obj = proj.objects.get(&wall_id).unwrap();
        assert_eq!(obj.object_type, c::ObjectType::Component as i32);
        assert!(obj.features.len() > 0, "wall should have features from default template");
        assert!(obj.geometry.is_some(), "wall should have geometry after rebuild");
    }

    #[test]
    fn test_create_beam_from_default_template() {
        setup_library();
        let mut proj = Project::new("TestProject");

        let mut params = std::collections::HashMap::new();
        params.insert("length".into(), 3000.0);
        let beam_id = proj.create_object_from_default_template(
            c::ObjectType::Beam, "Beam-1", &params, None,
        ).unwrap();

        let obj = proj.objects.get(&beam_id).unwrap();
        assert!(obj.features.len() > 0);
    }

    #[test]
    fn test_create_site_no_default_template() {
        setup_library();
        let mut proj = Project::new("TestProject");

        let params = std::collections::HashMap::new();
        let site_id = proj.create_object_from_default_template(
            c::ObjectType::Site, "Site-1", &params, None,
        ).unwrap();

        let obj = proj.objects.get(&site_id).unwrap();
        assert_eq!(obj.object_type, c::ObjectType::Site as i32);
        assert_eq!(obj.features.len(), 0, "site should have no features");
    }

    #[test]
    fn test_validate_catalog_params_in_range() {
        setup_library();
        let proj = Project::new("TestProject");
        let pipe_id = {
            let lib = LIBRARY.read().unwrap();
            let lib = lib.as_ref().unwrap();
            lib.catalog.find_by_name("Pipe-CS").unwrap().id.clone()
        };

        let mut params = std::collections::HashMap::new();
        params.insert("DN".into(), 100.0);
        params.insert("length".into(), 6000.0);

        let errors = proj.validate_catalog_params(&pipe_id, &params);
        assert!(errors.is_empty(), "params in range should have no errors: {:?}", errors);
    }

    #[test]
    fn test_validate_catalog_params_out_of_range() {
        setup_library();
        let proj = Project::new("TestProject");
        let pipe_id = {
            let lib = LIBRARY.read().unwrap();
            let lib = lib.as_ref().unwrap();
            lib.catalog.find_by_name("Pipe-CS").unwrap().id.clone()
        };

        let mut params = std::collections::HashMap::new();
        params.insert("DN".into(), 5.0); // below min 15.0
        params.insert("length".into(), 999999.0); // above max 12000.0

        let errors = proj.validate_catalog_params(&pipe_id, &params);
        assert!(errors.len() >= 2, "should have at least 2 errors: {:?}", errors);
    }

    #[test]
    fn test_find_and_rebuild_catalog_instances() {
        setup_library();
        let mut proj = Project::new("TestProject");

        let params = std::collections::HashMap::new();
        let _id1 = proj.create_object_from_default_template(
            c::ObjectType::Wall, "Wall-A", &params, None,
        ).unwrap();
        let _id2 = proj.create_object_from_default_template(
            c::ObjectType::Wall, "Wall-B", &params, None,
        ).unwrap();

        let wall_comp_id = {
            let lib = LIBRARY.read().unwrap();
            let lib = lib.as_ref().unwrap();
            lib.catalog.find_by_name("Concrete-Wall").unwrap().id.clone()
        };

        let instances = proj.find_catalog_instances(&wall_comp_id);
        assert_eq!(instances.len(), 2, "should find 2 wall instances");

        let rebuilt = proj.rebuild_catalog_instances(&wall_comp_id).unwrap();
        assert_eq!(rebuilt.len(), 2, "should rebuild 2 instances");
    }

    #[test]
    fn test_instantiate_from_series_pipe() {
        setup_library();
        let mut proj = Project::new("TestProject");
        setup_series(&mut proj);
        let params = std::collections::HashMap::new();
        let id = proj.instantiate_from_series("CS150", "PIPE", "Pipe-1", &params, None).unwrap();
        let obj = proj.objects.get(&id).unwrap();
        assert!(obj.features.len() > 0);
        assert!(obj.geometry.is_some(), "pipe should have geometry");
    }

    #[test]
    fn test_instantiate_from_series_beam() {
        setup_library();
        let mut proj = Project::new("TestProject");
        setup_series(&mut proj);
        let params = std::collections::HashMap::new();
        let id = proj.instantiate_from_series("Steel-Q355", "beam", "Beam-1", &params, None).unwrap();
        let obj = proj.objects.get(&id).unwrap();
        assert!(obj.features.len() > 0);
    }

    #[test]
    fn test_instantiate_from_series_road() {
        setup_library();
        let mut proj = Project::new("TestProject");
        setup_series(&mut proj);
        let params = std::collections::HashMap::new();
        let id = proj.instantiate_from_series("Highway-GB", "ASPHALT", "Road-1", &params, None).unwrap();
        let obj = proj.objects.get(&id).unwrap();
        assert!(obj.geometry.is_some());
    }

    #[test]
    fn test_validate_series_params_valid() {
        setup_library();
        let mut proj = Project::new("TestProject");
        setup_series(&mut proj);
        let mut params = std::collections::HashMap::new();
        params.insert("DN".into(), 100.0);
        let errors = proj.validate_series_params("CS150", "PIPE", &params);
        assert!(errors.is_empty(), "valid params should have no errors: {:?}", errors);
    }

    #[test]
    fn test_validate_series_params_invalid_series() {
        setup_library();
        let mut proj = Project::new("TestProject");
        setup_series(&mut proj);
        let params = std::collections::HashMap::new();
        let errors = proj.validate_series_params("NonExistent", "PIPE", &params);
        assert!(!errors.is_empty(), "should error on unknown series");
    }

    #[test]
    fn test_instantiate_catalog_by_name() {
        setup_library();
        let mut proj = Project::new("TestProject");
        let mut params = std::collections::HashMap::new();
        params.insert("length".into(), 5000.0);
        params.insert("thickness".into(), 200.0);
        params.insert("height".into(), 3000.0);
        // Use the display name (not UUID) to ensure find_by_name fallback works
        let id = proj.instantiate_catalog_component_skip_gate("Concrete-Wall", "Wall-1", &params, None, 0.0)
            .expect("instantiate by name should work");
        let obj = proj.objects.get(&id).unwrap();
        assert!(obj.features.len() > 0);
    }
}
