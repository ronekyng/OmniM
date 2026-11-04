use omnim_core as c;

/// Serializes tests that may replace the global `LIBRARY`. The seed-catalog
/// suite re-seeds it; without this lock a parallel test can fetch a component
/// id from one library instance and create objects from another — the
/// long-standing `test_find_and_rebuild_catalog_instances` flake.
#[cfg(test)]
pub(crate) static LIBRARY_TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

use serde::{Deserialize, Serialize};
use std::sync::RwLock;

use crate::object::OmniObject;
use crate::{
    AdministrativeStore, BreakdownStore, Feature, LifecycleStore, MaterialStore, MeasurementStore,
    ObjectStore, RelationStore, SeriesStore,
};

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

    /// Editable metadata (G2 UI-DAT-07, DATA_INTEGRITY §2.3). Display-only
    /// fields; they never re-scale stored geometry.
    #[serde(default)]
    pub description: String,
    /// Author / organisation free text.
    #[serde(default)]
    pub author: String,
    /// Display units setting (`metric` | `imperial`); stored values stay in
    /// metres regardless.
    #[serde(default)]
    pub unit_system: crate::project::UnitSystem,
    #[serde(default)]
    pub external_series_refs: Vec<SeriesRef>,
    #[serde(default)]
    pub measurements: MeasurementStore,
    /// Grid datum lines (轴网), P8-S1. Datum annotations, not objects.
    #[serde(default)]
    pub grids: crate::grid::GridStore,
    /// Motion joints (Phase 8-4, ASSEMBLY §9). Mechanism annotations between
    /// objects, not objects themselves.
    #[serde(default)]
    pub joints: crate::joint::JointStore,
}

/// One object changed by a storey-elevation cascade (P8-S2). Carried inside a
/// single `Command::ModifyObject` so the whole cascade is one undo step.
#[derive(Clone)]
pub struct LevelExtentChange {
    pub object_id: String,
    pub before: OmniObject,
    pub after: OmniObject,
}

/// Reversible reference-cleanup data produced by [`Project::delete_object`].
/// Undo restores every piece in one step: re-parented children, removed
/// containment/connection edges and detached lifecycle/breakdown references.
#[derive(Clone, Default)]
pub struct DeleteCascade {
    /// Children of the deleted object promoted to the deleted object's own
    /// parent; snapshots hold their pre-delete `parent_id`.
    pub promoted_children: Vec<OmniObject>,
    /// Descendants removed together with the object (subtree delete, e.g. an
    /// Assembly container with its parts). Restored by the same undo step.
    pub deleted_children: Vec<OmniObject>,
    /// Containment edges removed (the object as child, and as parent for the
    /// promoted children / deleted subtree).
    pub removed_containments: Vec<c::Containment>,
    /// Connections the object took part in.
    pub removed_connections: Vec<c::Connection>,
    /// (lifecycle record id, object id) pairs detached from records.
    pub lifecycle_refs: Vec<(String, String)>,
    /// (breakdown node id, object id) pairs detached from nodes.
    pub breakdown_refs: Vec<(String, String)>,
    /// Motion joints removed because an endpoint object was deleted
    /// (Phase 8-4). Restored by the same undo step.
    pub removed_joints: Vec<crate::joint::MotionJoint>,
}

/// Reference to an external series (file or Library-provided).
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SeriesRef {
    pub id: String,
    pub name: String,
    pub source: String, // "standalone" | "library"
    pub path: Option<String>,
}

/// Display units system (DATA_INTEGRITY §2.3; UI-DAT-07).
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum UnitSystem {
    #[default]
    Metric,
    Imperial,
}

impl UnitSystem {
    /// `metric` | `imperial`.
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "metric" => Some(UnitSystem::Metric),
            "imperial" => Some(UnitSystem::Imperial),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            UnitSystem::Metric => "metric",
            UnitSystem::Imperial => "imperial",
        }
    }
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
            grids: crate::grid::GridStore::new(),
            joints: crate::joint::JointStore::new(),
            description: String::new(),
            author: String::new(),
            unit_system: crate::project::UnitSystem::Metric,
        };
        crate::seed_materials::seed_default_materials(&mut p.materials);
        p
    }

    pub fn create_object(&mut self, object_type: c::ObjectType, name: &str) -> &mut OmniObject {
        let obj = OmniObject::new(object_type, name);
        self.objects.insert(obj)
    }

    /// Delete an object with reference cleanup (audit 2026-10-30: a plain
    /// `ObjectStore::remove` leaves children invisible and relations,
    /// lifecycle and breakdown references dangling):
    /// - children are re-parented to the deleted object's own parent;
    /// - containment edges (as child and as parent) are removed;
    /// - connections the object took part in are removed;
    /// - lifecycle/breakdown `related_object_ids` pointing at it are detached.
    /// A storey still anchoring objects (`level_id`/`bottom_level_id`/
    /// `top_level_id`) is rejected, matching the server guard.
    /// Returns the removed object plus the reversible [`DeleteCascade`].
    pub fn delete_object(
        &mut self,
        object_id: &str,
    ) -> Result<(OmniObject, DeleteCascade), String> {
        let anchored = self
            .objects
            .list()
            .iter()
            .filter(|o| {
                o.id != object_id
                    && (o.level_id.as_deref() == Some(object_id)
                        || o.bottom_level_id.as_deref() == Some(object_id)
                        || o.top_level_id.as_deref() == Some(object_id))
            })
            .count();
        if anchored > 0 {
            return Err(format!(
                "object '{}' is a level referenced by {} object(s); detach them first",
                object_id, anchored
            ));
        }
        let parent_of_removed = self
            .objects
            .get(object_id)
            .and_then(|o| o.parent_id.clone());
        let promoted_children: Vec<OmniObject> = self
            .objects
            .children_of(object_id)
            .into_iter()
            .cloned()
            .collect();
        let mut cascade = DeleteCascade {
            promoted_children,
            ..Default::default()
        };
        let obj = self
            .objects
            .remove(object_id)
            .ok_or_else(|| format!("object '{}' not found", object_id))?;
        for child in &cascade.promoted_children {
            if let Some(o) = self.objects.get_mut(&child.id) {
                o.parent_id = parent_of_removed.clone();
            }
        }
        self.objects.rebuild_children();

        // Containment: the object as child, then as parent for promoted
        // children (index lookups must run before each removal rebuild).
        if let Some(cont) = self.relations.parent_of(object_id) {
            let parent_id = cont.parent_id.clone();
            let child_id = cont.child_id.clone();
            if let Some(removed) = self.relations.remove_containment(&parent_id, &child_id) {
                cascade.removed_containments.push(removed);
            }
        }
        let as_parent_edges: Vec<c::Containment> = self
            .relations
            .children_of(object_id)
            .into_iter()
            .cloned()
            .collect();
        for cont in as_parent_edges {
            if let Some(removed) = self
                .relations
                .remove_containment(&cont.parent_id, &cont.child_id)
            {
                cascade.removed_containments.push(removed);
            }
        }
        // Connections: remove every edge the object took part in.
        let conns: Vec<c::Connection> = self
            .relations
            .connections_of(object_id)
            .into_iter()
            .cloned()
            .collect();
        for conn in conns {
            if let Some(removed) = self
                .relations
                .remove_connection(&conn.from_object_id, &conn.to_object_id)
            {
                cascade.removed_connections.push(removed);
            }
        }
        // Lifecycle / breakdown references.
        cascade.lifecycle_refs = self.lifecycle.detach_object(object_id);
        cascade.breakdown_refs = self.breakdown.detach_object(object_id);
        // Motion joints (Phase 8-4): a joint on a deleted endpoint is removed
        // with the object and restored by the same undo step.
        cascade.removed_joints = self
            .joints
            .remove_many(|j| j.from_object_id == object_id || j.to_object_id == object_id);
        Ok((obj, cascade))
    }

    /// Delete an object **and every descendant** (subtree delete), used for
    /// Assembly containers and exposed as `DELETE_OBJECT_SUBTREE`. All removed
    /// objects, their containment/connection edges and detached
    /// lifecycle/breakdown references are captured in one [`DeleteCascade`],
    /// so undo restores the whole subtree in a single step.
    pub fn delete_object_subtree(
        &mut self,
        object_id: &str,
    ) -> Result<(OmniObject, DeleteCascade), String> {
        if !self.objects.contains(object_id) {
            return Err(format!("object '{}' not found", object_id));
        }
        // Collect the subtree breadth-first (children before grandchildren).
        let mut removed_ids: Vec<String> = Vec::new();
        let mut queue: Vec<String> = vec![object_id.to_string()];
        while let Some(id) = queue.pop() {
            if removed_ids.iter().any(|x| x == &id) {
                continue;
            }
            removed_ids.push(id.clone());
            for child in self.objects.children_of(&id) {
                queue.push(child.id.clone());
            }
        }
        let removed_set: std::collections::HashSet<&str> =
            removed_ids.iter().map(|s| s.as_str()).collect();
        // Reject when an object **outside** the subtree anchors a removed one
        // as a level (same guard as single delete).
        let anchored = self
            .objects
            .list()
            .iter()
            .filter(|o| {
                !removed_set.contains(o.id.as_str())
                    && (o
                        .level_id
                        .as_deref()
                        .map(|l| removed_set.contains(l))
                        .unwrap_or(false)
                        || o.bottom_level_id
                            .as_deref()
                            .map(|l| removed_set.contains(l))
                            .unwrap_or(false)
                        || o.top_level_id
                            .as_deref()
                            .map(|l| removed_set.contains(l))
                            .unwrap_or(false))
            })
            .count();
        if anchored > 0 {
            return Err(format!(
                "subtree contains a level referenced by {} object(s); detach them first",
                anchored
            ));
        }

        let deleted_children: Vec<OmniObject> = removed_ids
            .iter()
            .skip(1)
            .filter_map(|id| self.objects.get(id).cloned())
            .collect();
        let mut cascade = DeleteCascade {
            deleted_children,
            ..Default::default()
        };
        let obj = self
            .objects
            .remove(object_id)
            .ok_or_else(|| format!("object '{}' not found", object_id))?;
        for id in removed_ids.iter().skip(1) {
            self.objects.remove(id);
        }
        self.objects.rebuild_children();

        // Snapshot every containment edge touching the subtree.
        let conts: Vec<c::Containment> = self
            .relations
            .containments()
            .iter()
            .filter(|cont| {
                removed_set.contains(cont.parent_id.as_str())
                    || removed_set.contains(cont.child_id.as_str())
            })
            .cloned()
            .collect();
        for cont in conts {
            if let Some(removed) = self
                .relations
                .remove_containment(&cont.parent_id, &cont.child_id)
            {
                cascade.removed_containments.push(removed);
            }
        }
        // Snapshot every connection touching the subtree.
        let conns: Vec<c::Connection> = self
            .relations
            .connections()
            .iter()
            .filter(|conn| {
                removed_set.contains(conn.from_object_id.as_str())
                    || removed_set.contains(conn.to_object_id.as_str())
            })
            .cloned()
            .collect();
        for conn in conns {
            if let Some(removed) = self
                .relations
                .remove_connection(&conn.from_object_id, &conn.to_object_id)
            {
                cascade.removed_connections.push(removed);
            }
        }
        // Detach lifecycle / breakdown references for every removed id.
        for id in &removed_ids {
            cascade
                .lifecycle_refs
                .extend(self.lifecycle.detach_object(id));
            cascade
                .breakdown_refs
                .extend(self.breakdown.detach_object(id));
        }
        cascade.removed_joints = self.joints.remove_many(|j| {
            removed_set.contains(j.from_object_id.as_str())
                || removed_set.contains(j.to_object_id.as_str())
        });
        Ok((obj, cascade))
    }

    /// Restore a previously deleted object together with its cascade (undo of
    /// the delete): re-insert, re-parent children to their pre-delete
    /// parents, re-add relations and re-attach lifecycle/breakdown refs.
    pub fn restore_deleted(&mut self, object: OmniObject, cascade: &DeleteCascade) {
        self.objects.insert(object);
        for child in &cascade.promoted_children {
            if let Some(o) = self.objects.get_mut(&child.id) {
                o.parent_id = child.parent_id.clone();
            }
        }
        // Subtree delete: re-insert descendants with their original parents.
        for child in &cascade.deleted_children {
            self.objects.insert(child.clone());
        }
        self.objects.rebuild_children();
        for cont in &cascade.removed_containments {
            let _ = self.relations.add_containment(cont.clone());
        }
        for conn in &cascade.removed_connections {
            self.relations.add_connection(conn.clone());
        }
        for (record_id, object_id) in &cascade.lifecycle_refs {
            if let Some(record) = self.lifecycle.get_mut(record_id) {
                if !record.related_object_ids.contains(object_id) {
                    record.related_object_ids.push(object_id.clone());
                }
            }
        }
        for (node_id, object_id) in &cascade.breakdown_refs {
            if let Some(node) = self.breakdown.get_mut(node_id) {
                if !node.related_object_ids.contains(object_id) {
                    node.related_object_ids.push(object_id.clone());
                }
            }
        }
        // Motion joints (Phase 8-4): a joint removed with its endpoint comes
        // back when the object does (order-insensitive re-insert).
        for joint in &cascade.removed_joints {
            self.joints.insert(joint.clone());
        }
    }

    pub fn set_parent(&mut self, child_id: &str, parent_id: &str) -> Result<(), String> {
        if !self.objects.contains(child_id) {
            return Err(format!("object '{}' not found", child_id));
        }
        if !self.objects.contains(parent_id) {
            return Err(format!("parent '{}' not found", parent_id));
        }
        // Cycle guard: the parent must not be a descendant of the child.
        let mut cursor = Some(parent_id.to_string());
        while let Some(id) = cursor {
            if id == child_id {
                return Err(format!(
                    "reparent would create a cycle: '{}' is already an ancestor of '{}'",
                    child_id, parent_id
                ));
            }
            cursor = self.objects.get(&id).and_then(|o| o.parent_id.clone());
        }
        // Remove the previous containment first, so an existing-child error
        // from `add_containment` cannot leave the tree re-parented while the
        // relation store still holds the stale old parent (audit: half
        // transaction on reparent).
        if let Some(old) = self.relations.parent_of(child_id).map(|c| c.clone()) {
            self.relations
                .remove_containment(&old.parent_id, &old.child_id);
        }
        let added = self.relations.add_containment(c::Containment {
            parent_id: parent_id.to_string(),
            child_id: child_id.to_string(),
            r#type: c::ContainmentType::Spatial as i32,
        });
        if added.is_ok() {
            if let Some(obj) = self.objects.get_mut(child_id) {
                obj.parent_id = Some(parent_id.to_string());
            }
            self.objects.rebuild_children();
        }
        added.map(|_| ())
    }

    /// Series Gate: verify that a ComponentDef is reachable from at least one
    /// loaded series in the project.
    ///
    /// Behavior:
    /// - Series loaded + component reachable -> allowed (Ok)
    /// - Series loaded + component NOT reachable -> rejected (Err)
    /// - No series loaded -> open mode, all components allowed (Ok)
    pub fn check_series_gate(
        &self,
        component_def_id: &str,
        component_name: &str,
    ) -> Result<(), String> {
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

    pub fn connect(
        &mut self,
        from_id: &str,
        to_id: &str,
        conn_type: c::ConnectionType,
    ) -> Result<(), String> {
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

    pub fn add_lifecycle_record(
        &mut self,
        record: c::ProjectLifecycleRecord,
    ) -> &mut c::ProjectLifecycleRecord {
        self.lifecycle.insert(record)
    }

    pub fn link_record_to_object(
        &mut self,
        record_id: &str,
        object_id: &str,
    ) -> Result<(), String> {
        let record = self
            .lifecycle
            .get_mut(record_id)
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
            catalog_component_count: LIBRARY
                .read()
                .unwrap()
                .as_ref()
                .map(|l| l.catalog.len())
                .unwrap_or(0),
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
        self.objects
            .find_by_type(c::ObjectType::Component)
            .into_iter()
            .map(|o| o.id.clone())
            .collect()
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
        self.instantiate_catalog_component_impl(
            component_def_id,
            instance_name,
            param_overrides,
            position,
            rotation_angle,
            true,
            None,
        )
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
        self.instantiate_catalog_component_impl(
            component_def_id,
            instance_name,
            param_overrides,
            position,
            rotation_angle,
            false,
            None,
        )
    }

    /// Catalog instantiation with an explicit local rotation quaternion.
    ///
    /// `rotation` is applied to the template geometry **before** the position
    /// translation (same local frame as `rotation_angle`). Linear components
    /// (beam/pipe/duct) extrude along local +Z, so callers use this to lay the
    /// length axis along a picked segment instead of leaving it vertical.
    pub fn instantiate_catalog_component_oriented(
        &mut self,
        component_def_id: &str,
        instance_name: &str,
        param_overrides: &std::collections::HashMap<String, f64>,
        position: Option<c::Point3D>,
        rotation_angle: f64,
        rotation: Option<c::Rotation>,
    ) -> Result<String, String> {
        self.instantiate_catalog_component_impl(
            component_def_id,
            instance_name,
            param_overrides,
            position,
            rotation_angle,
            true,
            rotation,
        )
    }

    /// [`Self::instantiate_catalog_component_oriented`] without the series gate.
    pub fn instantiate_catalog_component_oriented_skip_gate(
        &mut self,
        component_def_id: &str,
        instance_name: &str,
        param_overrides: &std::collections::HashMap<String, f64>,
        position: Option<c::Point3D>,
        rotation_angle: f64,
        rotation: Option<c::Rotation>,
    ) -> Result<String, String> {
        self.instantiate_catalog_component_impl(
            component_def_id,
            instance_name,
            param_overrides,
            position,
            rotation_angle,
            false,
            rotation,
        )
    }

    fn instantiate_catalog_component_impl(
        &mut self,
        component_def_id: &str,
        instance_name: &str,
        param_overrides: &std::collections::HashMap<String, f64>,
        position: Option<c::Point3D>,
        rotation_angle: f64,
        check_gate: bool,
        rotation: Option<c::Rotation>,
    ) -> Result<String, String> {
        let library = LIBRARY.read().unwrap();
        let library = library.as_ref().expect("LIBRARY not initialized");
        let comp_def = library
            .catalog
            .get(component_def_id)
            .or_else(|| library.catalog.find_by_name(component_def_id))
            .ok_or_else(|| format!("component def '{}' not found in catalog", component_def_id))?
            .clone();

        // Series Gate: reachability + spec enforcement (Wave 4).
        // A reachable component always originates from a Component(spec) leaf,
        // so the gate can locate the SpecComponent and apply its parameter
        // constraints and material — that is what "series-approved" means.
        let mut gate_params: Option<std::collections::HashMap<String, f64>> = None;
        let mut series_material: Option<String> = None;
        if check_gate {
            self.check_series_gate(&comp_def.id, &comp_def.name)?;
            // Enforcement only applies when a series is actually loaded —
            // def defaults stay unenforced otherwise (legacy behavior).
            if self.series.is_any_series_loaded() {
                let mut params_view = param_overrides.clone();
                // Resolve the spec through the loaded series (id-or-name match).
                let found = {
                    let store = &self.series;
                    store.find_spec_for_component(&comp_def.id, &comp_def.name)
                };
                let (series_id, _series_name, spec) = found.ok_or_else(|| {
                    format!(
                        "series gate: component '{}' ({}) is reachable but no SpecComponent matched; the series store may mix ids and names",
                        comp_def.name, comp_def.id
                    )
                })?;
                let mut merged = params_view.clone();
                crate::series_store::SeriesStore::enforce_param_constraints(
                    &spec.param_constraints,
                    &mut merged,
                )?;
                params_view = merged;
                gate_params = Some(params_view);
                if !spec.material.is_empty() {
                    series_material = Some(spec.material.clone());
                }
                let _ = series_id;
            }
        }

        let tree_id = comp_def.feature_tree_id.clone();
        if tree_id.is_empty() {
            return Err(format!(
                "component def '{}' has no feature tree template",
                comp_def.name
            ));
        }
        let template = library
            .feature_trees
            .get(&tree_id)
            .ok_or_else(|| format!("feature tree template '{}' not found", tree_id))?
            .clone();

        // Resolve full parameter set: defaults from ComponentDef, then the
        // gate-enforced spec-merged overrides (constraints may replace
        // defaults), then any remaining provided values.
        let mut params: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
        for p in &comp_def.params {
            params.insert(p.name.clone(), p.default_value);
        }
        let effective_overrides = if check_gate {
            gate_params
                .take()
                .unwrap_or_else(|| param_overrides.clone())
        } else {
            param_overrides.clone()
        };
        for (k, v) in &effective_overrides {
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
                value_type: Some(c::property_value::ValueType::StringValue(
                    component_def_id.to_string(),
                )),
            }),
            unit: String::new(),
            description: String::new(),
        });
        obj.properties.push(c::Property {
            name: "catalog_component_name".to_string(),
            value: Some(c::PropertyValue {
                value_type: Some(c::property_value::ValueType::StringValue(
                    comp_def.name.clone(),
                )),
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
        // Series material (Wave 4): resolve by id first, then by name
        // (case-insensitive). Unknown material names are an error — silently
        // dropping the spec's material would be the P0-5 deviation again.
        if let Some(material_ref) = &series_material {
            let resolved = if self.materials.get(material_ref).is_some() {
                Some(material_ref.clone())
            } else {
                self.materials
                    .list()
                    .iter()
                    .find(|m| m.name.eq_ignore_ascii_case(material_ref))
                    .map(|m| m.id.clone())
            };
            match resolved {
                Some(id) => obj.material_id = Some(id),
                None => {
                    let available: Vec<String> = self
                        .materials
                        .list()
                        .iter()
                        .take(5)
                        .map(|m| m.name.clone())
                        .collect();
                    return Err(format!(
                        "series material '{}' not found in project materials (available: {})",
                        material_ref,
                        available.join(", ")
                    ));
                }
            }
        }
        self.objects.insert(obj);

        // Instantiate the feature tree
        crate::feature_template::instantiate_template(
            self,
            &component_id,
            &template,
            &params,
            &crate::feature_template::TemplateContext::default(),
        )?;

        // Rebuild the component to generate geometry
        self.rebuild_component(&component_id)?;

        // Apply the requested placement — rebuild_component merged the feature
        // geometry at the sketch origin, so rotate (Z yaw, then any explicit
        // local quaternion) and translate the resulting mesh.
        let has_rotation = rotation_angle != 0.0 || rotation.is_some();
        if position.is_some() || has_rotation {
            let obj = self.objects.get_mut(&component_id).unwrap();
            if let Some(ref mut geom) = obj.geometry {
                if let Some(c::geometry::Representation::Mesh(ref mut m)) = &mut geom.representation
                {
                    // Rotate around Z axis (plan yaw)
                    if rotation_angle != 0.0 {
                        let (sin, cos) = rotation_angle.sin_cos();
                        for v in &mut m.vertices {
                            let x = v.x * cos - v.y * sin;
                            let y = v.x * sin + v.y * cos;
                            v.x = x;
                            v.y = y;
                        }
                    }
                    // Explicit local rotation (e.g. lay a +Z-extruded beam
                    // along a picked segment) — same frame as the yaw above.
                    if let Some(rot) = rotation {
                        *m = crate::geometry::transform_mesh(
                            m,
                            &c::Transform {
                                matrix: Vec::new(),
                                translation: None,
                                rotation: Some(rot),
                                scale: None,
                            },
                        );
                    }
                    // Translate to position (metres → mm)
                    if let Some(pos) = position {
                        for v in &mut m.vertices {
                            v.x += pos.x * 1000.0;
                            v.y += pos.y * 1000.0;
                            v.z += pos.z * 1000.0;
                        }
                    }
                    if geom.bbox.is_some() {
                        geom.bbox = Some(crate::geometry::compute_bbox(m));
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
        let series = self
            .series
            .find_by_name(series_name)
            .ok_or_else(|| format!("series '{}' not found", series_name))?;

        let series_id = series.id.clone();
        let result = crate::series_store::SelectionEngine::select(series, &[type_answer])
            .map_err(|e| format!("series selection failed: {}", e))?;

        let library = LIBRARY.read().unwrap();
        let library = library.as_ref().expect("LIBRARY not initialized");
        let comp_def_id = library
            .catalog
            .get(&result.component_def_id)
            .or_else(|| library.catalog.find_by_name(&result.component_def_id))
            .ok_or_else(|| {
                format!(
                    "component def '{}' not found in catalog (tried id and name)",
                    result.component_def_id
                )
            })?
            .id
            .clone();

        // Apply series param constraints as overrides
        let mut merged_params = param_overrides.clone();
        for constraint in &result.spec_component.param_constraints {
            if !merged_params.contains_key(&constraint.param_name) {
                merged_params.insert(constraint.param_name.clone(), constraint.default_value);
            }
        }

        let component_id = self.instantiate_catalog_component(
            &comp_def_id,
            instance_name,
            &merged_params,
            position,
            0.0,
        )?;

        // Enrich catalog_ref with series information
        if let Some(obj) = self.objects.get_mut(&component_id) {
            if let Some(ref mut cr) = obj.catalog_ref {
                cr.series_id = series_id;
                cr.selector_path = result.selector_path.clone();
            }
        }

        Ok(component_id)
    }

    /// Wave 3 geometry convergence: instantiate a **domain-typed** object from
    /// a path-driven (PathSweep) ComponentDef template.
    ///
    /// Unlike [`Self::instantiate_catalog_component`] (which creates
    /// `ObjectType::Component` assembly instances), this creates the semantic
    /// object type itself (`Wall`/`Slab`/`Floor`/`Roof`), carries the picked
    /// path + LayerEditor layer stack through `TemplateContext`, and records
    /// `catalog_ref`/`param_values` for traceability.
    #[allow(clippy::too_many_arguments)]
    pub fn instantiate_domain_object(
        &mut self,
        object_type: c::ObjectType,
        component_def_id: &str,
        instance_name: &str,
        param_values: &std::collections::HashMap<String, f64>,
        path: Vec<[f64; 3]>,
        path_closed: bool,
        layers: Vec<crate::gen_rule::layer_extrusion::LayerDef>,
        height: f64,
        series_id: Option<&str>,
    ) -> Result<String, String> {
        let (comp_def, template) = {
            let library = LIBRARY.read().unwrap();
            let library = library.as_ref().ok_or("LIBRARY not initialized")?;
            let comp_def = library
                .catalog
                .get(component_def_id)
                .or_else(|| library.catalog.find_by_name(component_def_id))
                .ok_or_else(|| {
                    format!("component def '{}' not found in catalog", component_def_id)
                })?
                .clone();
            let tree_id = comp_def.feature_tree_id.clone();
            if tree_id.is_empty() {
                return Err(format!(
                    "component def '{}' has no feature tree template",
                    comp_def.name
                ));
            }
            let template = library
                .feature_trees
                .get(&tree_id)
                .ok_or_else(|| format!("feature tree template '{}' not found", tree_id))?
                .clone();
            (comp_def, template)
        };

        let mut obj = crate::OmniObject::new(object_type, instance_name);
        let component_id = obj.id.clone();
        obj.properties.push(c::Property {
            name: "catalog_component_id".to_string(),
            value: Some(c::PropertyValue {
                value_type: Some(c::property_value::ValueType::StringValue(
                    comp_def.id.clone(),
                )),
            }),
            unit: String::new(),
            description: String::new(),
        });
        obj.properties.push(c::Property {
            name: "catalog_component_name".to_string(),
            value: Some(c::PropertyValue {
                value_type: Some(c::property_value::ValueType::StringValue(
                    comp_def.name.clone(),
                )),
            }),
            unit: String::new(),
            description: String::new(),
        });
        obj.catalog_ref = Some(crate::catalog::CatalogRef {
            component_def_id: comp_def.id.clone(),
            series_id: series_id.unwrap_or_default().to_string(),
            selector_path: Vec::new(),
            param_values: param_values.clone(),
        });
        self.objects.insert(obj);

        let ctx = crate::feature_template::TemplateContext {
            path,
            path_closed,
            layers,
            height,
        };
        crate::feature_template::instantiate_template(
            self,
            &component_id,
            &template,
            param_values,
            &ctx,
        )?;
        self.rebuild_component(&component_id)?;
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
            let comp_def_id = library
                .catalog
                .find_by_name(comp_name)
                .ok_or_else(|| format!("default catalog component '{}' not found", comp_name))?
                .id
                .clone();
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
        let comp_def = match library
            .catalog
            .get(component_def_id_or_name)
            .or_else(|| library.catalog.find_by_name(component_def_id_or_name))
        {
            Some(d) => d,
            None => {
                errors.push(format!(
                    "component def '{}' not found in catalog",
                    component_def_id_or_name
                ));
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
            let comp_errors =
                self.validate_catalog_params(&selection.spec_component.component_def_id, params);
            errors.extend(comp_errors);
        }
        errors
    }

    /// Find all objects that are instances of a given catalog component
    /// (identified by the `catalog_component_id` property).
    ///
    /// Returns a list of object IDs.
    pub fn find_catalog_instances(&self, component_def_id: &str) -> Vec<String> {
        self.objects
            .list()
            .iter()
            .filter(|obj| {
                obj.properties.iter().any(|p| {
                    p.name == "catalog_component_id"
                        && p.value
                            .as_ref()
                            .and_then(|v| v.value_type.as_ref())
                            .map(|vt| match vt {
                                c::property_value::ValueType::StringValue(s) => {
                                    s == component_def_id
                                }
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
    pub fn rebuild_catalog_instances(
        &mut self,
        component_def_id: &str,
    ) -> Result<Vec<String>, String> {
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
            let obj = self
                .objects
                .get(component_id)
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

    pub fn remove_feature(
        &mut self,
        component_id: &str,
        feature_id: &str,
    ) -> Result<Feature, String> {
        let obj = self
            .objects
            .get_mut(component_id)
            .ok_or_else(|| format!("component '{}' not found", component_id))?;
        let idx = obj
            .features
            .iter()
            .position(|f| f.id == feature_id)
            .ok_or_else(|| {
                format!(
                    "feature '{}' not found in component '{}'",
                    feature_id, component_id
                )
            })?;
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
        let obj = self
            .objects
            .get_mut(component_id)
            .ok_or_else(|| format!("component '{}' not found", component_id))?;
        let idx = obj
            .features
            .iter()
            .position(|f| f.id == feature_id)
            .ok_or_else(|| format!("feature '{}' not found", feature_id))?;
        if let Some(sid) = sketch_id {
            obj.features[idx].sketch_id = sid;
        }
        if let Some(asid) = axis_sketch_id {
            obj.features[idx].axis_sketch_id = asid;
        }
        if let Some(d) = depth {
            obj.features[idx].depth = d;
        }
        if let Some(a) = angle {
            obj.features[idx].angle = a;
        }
        if let Some(c) = is_cut {
            obj.features[idx].is_cut = c;
        }
        if let Some(s) = suppressed {
            obj.features[idx].suppressed = s;
        }
        if let Some(ids) = input_sketch_ids {
            obj.features[idx].input_sketch_ids = ids;
        }
        if let Some(ids) = dependency_ids {
            obj.features[idx].dependency_ids = ids;
        }
        if let Some(d) = direction {
            obj.features[idx].direction = d;
        }
        if let Some(o) = axis_origin {
            obj.features[idx].axis_origin = o;
        }
        if let Some(d) = axis_direction {
            obj.features[idx].axis_direction = d;
        }
        if let Some(l) = lod {
            obj.features[idx].lod = l;
        }
        if let Some(b) = boolean_op {
            obj.features[idx].boolean_op = Some(b);
        }
        if let Some(c) = pattern_count_x {
            obj.features[idx].pattern_count_x = Some(c);
        }
        if let Some(c) = pattern_count_y {
            obj.features[idx].pattern_count_y = Some(c);
        }
        if let Some(c) = pattern_count_z {
            obj.features[idx].pattern_count_z = Some(c);
        }
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
        let obj = self
            .objects
            .get(component_id)
            .ok_or_else(|| format!("component '{}' not found", component_id))?;
        Ok(&obj.features)
    }

    // ── Building datum: level-driven vertical extent (P8-S2) ─────────────

    /// Validate a bottom/top level pair without needing an object (P8-S2).
    ///
    /// Both-or-neither, each reference must be an existing `BuildingStorey`,
    /// the two ids must differ, and top's elevation must be above bottom's
    /// (a storey without an explicit elevation counts as 0.0 m).
    pub fn validate_level_extent_refs(
        &self,
        bottom: Option<&str>,
        top: Option<&str>,
    ) -> Result<(), String> {
        let b = bottom.filter(|s| !s.is_empty());
        let t = top.filter(|s| !s.is_empty());
        match (b, t) {
            (None, None) => Ok(()),
            (Some(_), None) | (None, Some(_)) => {
                Err("bottom_level_id and top_level_id must be set together".into())
            }
            (Some(b), Some(t)) => {
                if b == t {
                    return Err(format!("bottom and top levels must be different ('{}')", b));
                }
                let elevation = |id: &str| -> Result<f64, String> {
                    let level = self
                        .objects
                        .get(id)
                        .ok_or_else(|| format!("level '{}' not found", id))?;
                    if level.object_type != c::ObjectType::BuildingStorey as i32 {
                        return Err(format!(
                            "level_id '{}' is not a BuildingStorey (type {})",
                            id, level.object_type
                        ));
                    }
                    Ok(level.elevation.unwrap_or(0.0))
                };
                let (bottom_elev, top_elev) = (elevation(b)?, elevation(t)?);
                if top_elev <= bottom_elev {
                    return Err(format!(
                        "top level elevation ({}) must be above bottom level elevation ({})",
                        top_elev, bottom_elev
                    ));
                }
                Ok(())
            }
        }
    }

    /// Resolve `bottom_level_id` / `top_level_id` into the object's datum.
    ///
    /// Contract (DATA_MODEL §5.8):
    /// - refs are **both-or-neither** and must target `BuildingStorey` objects;
    /// - `top.elevation > bottom.elevation` (a storey without an explicit
    ///   elevation counts as 0.0 m);
    /// - `elevation` becomes bottom's elevation (base datum);
    /// - for features carrying `PathSweepParams` geometry (the formal Wave-3
    ///   building path), the path's lowest z shifts to the base and the sweep
    ///   height becomes `top − bottom`. Feature caches are rebuilt.
    ///
    /// Plain-geometry objects (no PathSweep feature) receive the `elevation`
    /// datum only — their mesh is not rewritten (recorded as 🔀 in the matrix).
    ///
    /// Returns `Ok(true)` when the object changed.
    pub fn resolve_level_extent(&mut self, object_id: &str) -> Result<bool, String> {
        let (bottom_id, top_id) = {
            let obj = self
                .objects
                .get(object_id)
                .ok_or_else(|| format!("object '{}' not found", object_id))?;
            (
                obj.bottom_level_id.clone().filter(|s| !s.is_empty()),
                obj.top_level_id.clone().filter(|s| !s.is_empty()),
            )
        };
        if bottom_id.is_none() && top_id.is_none() {
            return Ok(false);
        }
        // Single source of truth for the pair contract (also used by the
        // omcom validator and the genrule handler's pre-check).
        self.validate_level_extent_refs(bottom_id.as_deref(), top_id.as_deref())
            .map_err(|e| format!("object '{}': {}", object_id, e))?;
        let bottom_id = bottom_id.expect("validated both-or-neither");
        let top_id = top_id.expect("validated both-or-neither");
        let elevation_of = |project: &Self, id: &str| -> f64 {
            project
                .objects
                .get(id)
                .and_then(|l| l.elevation)
                .unwrap_or(0.0)
        };
        let bottom_elev = elevation_of(self, &bottom_id);
        let top_elev = elevation_of(self, &top_id);
        let target_height = top_elev - bottom_elev;

        // Collect PathSweep feature indices and their current base z.
        let sweep_indices: Vec<usize> = {
            let obj = self.objects.get(object_id).unwrap();
            obj.features
                .iter()
                .enumerate()
                .filter(|(_, f)| {
                    f.feature_type == crate::FeatureType::PathSweep && f.path_sweep.is_some()
                })
                .map(|(i, _)| i)
                .collect()
        };

        let mut changed = false;
        {
            let obj = self.objects.get_mut(object_id).unwrap();
            let elevation_needs_update = match obj.elevation {
                Some(current) => (current - bottom_elev).abs() > 1e-9,
                None => true,
            };
            if elevation_needs_update {
                obj.elevation = Some(bottom_elev);
                changed = true;
            }
            for &idx in &sweep_indices {
                if let Some(params) = obj.features[idx].path_sweep.as_mut() {
                    let current_base = params
                        .points
                        .iter()
                        .map(|p| p[2])
                        .fold(f64::INFINITY, f64::min);
                    if current_base.is_finite() {
                        let shift = bottom_elev - current_base;
                        if shift.abs() > 1e-9 {
                            for p in params.points.iter_mut() {
                                p[2] += shift;
                            }
                            changed = true;
                        }
                    }
                    // Height only applies to layered sections (walls).
                    let is_layered =
                        matches!(params.section, crate::feature::PathSweepSection::Layered);
                    if is_layered && (params.height - target_height).abs() > 1e-9 {
                        params.height = target_height;
                        changed = true;
                    }
                }
            }
        }

        if changed {
            // Rebuild the sweeps so cached geometry reflects the new extent.
            for idx in sweep_indices {
                self.rebuild_feature(object_id, idx).map_err(|e| {
                    format!(
                        "object '{}': extent resolved but sweep rebuild failed: {}",
                        object_id, e
                    )
                })?;
            }
        }
        Ok(changed)
    }

    /// Re-resolve every object constrained to `level_id` after that storey's
    /// elevation changed. Returns the before/after pairs for a single undo
    /// entry (P8-S2 cascade).
    pub fn cascade_level_extents(&mut self, level_id: &str) -> Vec<LevelExtentChange> {
        let affected: Vec<String> = self
            .objects
            .list()
            .iter()
            .filter(|o| {
                o.bottom_level_id.as_deref() == Some(level_id)
                    || o.top_level_id.as_deref() == Some(level_id)
            })
            .map(|o| o.id.clone())
            .collect();
        let mut changes = Vec::new();
        for id in affected {
            let before = match self.objects.get(&id) {
                Some(o) => o.clone(),
                None => continue,
            };
            match self.resolve_level_extent(&id) {
                Ok(true) => {
                    if let Some(after) = self.objects.get(&id).cloned() {
                        changes.push(LevelExtentChange {
                            object_id: id,
                            before,
                            after,
                        });
                    }
                }
                Ok(false) => {}
                Err(e) => eprintln!("[WARN] level cascade skipped object {}: {}", id, e),
            }
        }
        changes
    }

    /// True when the object has both vertical-extent constraints set.
    pub fn has_level_extent(&self, object_id: &str) -> bool {
        self.objects
            .get(object_id)
            .map(|o| {
                o.bottom_level_id.as_deref().is_some_and(|s| !s.is_empty())
                    && o.top_level_id.as_deref().is_some_and(|s| !s.is_empty())
            })
            .unwrap_or(false)
    }

    pub fn rebuild_feature(
        &mut self,
        component_id: &str,
        feature_idx: usize,
    ) -> Result<(), String> {
        let obj = self
            .objects
            .get(component_id)
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
                            let profile3d: Vec<c::Point3D> = profile2d
                                .iter()
                                .map(|p| c::Point3D {
                                    x: p.x,
                                    y: p.y,
                                    z: 0.0,
                                })
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
                let mut resolved_origin = c::Point3D {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                };
                let mut resolved_dir = c::Vector3D {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                };
                if !axis_sketch_id.is_empty() {
                    if let Some(axis_sketch) = self.sketches.get(&axis_sketch_id) {
                        if let Some(line_entity) = axis_sketch
                            .entities
                            .iter()
                            .find(|e| matches!(e.entity_type, omnim_sketch::SketchEntityType::Line))
                        {
                            if line_entity.points.len() >= 2 {
                                let start = &line_entity.points[0];
                                let end = &line_entity.points[1];
                                let dx = end.x - start.x;
                                let dy = end.y - start.y;
                                let dz = end.z - start.z;
                                let len = (dx * dx + dy * dy + dz * dz).sqrt();
                                if len > 1e-10 {
                                    resolved_origin = c::Point3D {
                                        x: start.x,
                                        y: start.y,
                                        z: start.z,
                                    };
                                    resolved_dir = c::Vector3D {
                                        x: dx / len,
                                        y: dy / len,
                                        z: dz / len,
                                    };
                                }
                            }
                        }
                    }
                }
                if let Some(sketch) = self.sketches.get(&sketch_id) {
                    if let Some(profile2d) = omnim_sketch::profile::sketch_to_profile_2d(sketch) {
                        if profile2d.len() >= 3 {
                            let profile3d: Vec<c::Point3D> = profile2d
                                .iter()
                                .map(|p| c::Point3D {
                                    x: p.x,
                                    y: p.y,
                                    z: 0.0,
                                })
                                .collect();
                            crate::brep::revolve_brep(
                                &profile3d,
                                angle,
                                resolved_origin,
                                resolved_dir,
                                lod,
                            )
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
                let path_sketch_id = obj.features[feature_idx]
                    .path_sketch_id()
                    .map(|s| s.to_string());
                let path_sid = match path_sketch_id {
                    Some(id) => id,
                    None => return Err("sweep feature requires a path sketch".into()),
                };
                let profile_sketch = self.sketches.get(&sketch_id);
                let path_sketch = self.sketches.get(&path_sid);
                if let (Some(ps), Some(pp)) = (profile_sketch, path_sketch) {
                    let profile2d = omnim_sketch::profile::sketch_to_profile_2d(ps);
                    let path_points: Vec<c::Point3D> = pp
                        .entities
                        .iter()
                        .flat_map(|e| e.points.iter().cloned())
                        .collect();
                    if let (Some(prof), true) = (profile2d, path_points.len() >= 2) {
                        if prof.len() >= 3 {
                            let profile3d: Vec<c::Point3D> = prof
                                .iter()
                                .map(|p| c::Point3D {
                                    x: p.x,
                                    y: p.y,
                                    z: 0.0,
                                })
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
                    let p0 = self
                        .sketches
                        .get(&section_ids[0])
                        .and_then(|s| omnim_sketch::profile::sketch_to_profile_2d(s));
                    let p1 = self
                        .sketches
                        .get(&section_ids[1])
                        .and_then(|s| omnim_sketch::profile::sketch_to_profile_2d(s));
                    if let (Some(prof0), Some(prof1)) = (p0, p1) {
                        if prof0.len() >= 3 && prof1.len() >= 3 {
                            let profile0: Vec<c::Point3D> = prof0
                                .iter()
                                .map(|p| c::Point3D {
                                    x: p.x,
                                    y: p.y,
                                    z: 0.0,
                                })
                                .collect();
                            let profile1: Vec<c::Point3D> = prof1
                                .iter()
                                .map(|p| c::Point3D {
                                    x: p.x,
                                    y: p.y,
                                    z: 0.0,
                                })
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
                            let profile3d: Vec<c::Point3D> = profile2d
                                .iter()
                                .map(|p| c::Point3D {
                                    x: p.x,
                                    y: p.y,
                                    z: 0.0,
                                })
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
            crate::FeatureType::PathSweep => {
                // Wave 3 geometry convergence: the layered constant-section
                // kernel (gen_rule::path_sweep) generates the whole layer
                // stack from the feature's params — no sketches involved.
                let params = obj.features[feature_idx].path_sweep.clone();
                match params {
                    Some(params) => crate::gen_rule::path_sweep::generate(
                        &crate::gen_rule::path_sweep::PathSweepInput {
                            points: params.points,
                            closed: params.closed,
                            layers: params.layers,
                            height: params.height,
                            orientation: params.orientation,
                            section: params.section,
                        },
                    )
                    .ok(),
                    None => None,
                }
            }
            crate::FeatureType::LoftedFusion => {
                let section_ids = obj.features[feature_idx].section_sketch_ids();
                if section_ids.len() >= 2 {
                    let p0 = self
                        .sketches
                        .get(&section_ids[0])
                        .and_then(|s| omnim_sketch::profile::sketch_to_profile_2d(s));
                    let p1 = self
                        .sketches
                        .get(&section_ids[1])
                        .and_then(|s| omnim_sketch::profile::sketch_to_profile_2d(s));
                    if let (Some(prof0), Some(prof1)) = (p0, p1) {
                        if prof0.len() >= 3 && prof1.len() >= 3 {
                            let profile0: Vec<c::Point3D> = prof0
                                .iter()
                                .map(|p| c::Point3D {
                                    x: p.x,
                                    y: p.y,
                                    z: 0.0,
                                })
                                .collect();
                            let profile1: Vec<c::Point3D> = prof1
                                .iter()
                                .map(|p| c::Point3D {
                                    x: p.x,
                                    y: p.y,
                                    z: 0.0,
                                })
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
                        if let Some(profile2d) = omnim_sketch::profile::sketch_to_profile_2d(sketch)
                        {
                            if profile2d.len() >= 3 {
                                let profile3d: Vec<c::Point3D> = profile2d
                                    .iter()
                                    .map(|p| c::Point3D {
                                        x: p.x,
                                        y: p.y,
                                        z: 0.0,
                                    })
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
                        if let Some(profile2d) = omnim_sketch::profile::sketch_to_profile_2d(sketch)
                        {
                            if profile2d.len() >= 3 {
                                let profile3d: Vec<c::Point3D> = profile2d
                                    .iter()
                                    .map(|p| c::Point3D {
                                        x: p.x,
                                        y: p.y,
                                        z: 0.0,
                                    })
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
                                new_faces.push(c::FaceIndices {
                                    vertex_indices: vec![base, base + 3, base + 5],
                                });
                                new_faces.push(c::FaceIndices {
                                    vertex_indices: vec![base + 3, base + 1, base + 4],
                                });
                                new_faces.push(c::FaceIndices {
                                    vertex_indices: vec![base + 5, base + 4, base + 2],
                                });
                                new_faces.push(c::FaceIndices {
                                    vertex_indices: vec![base + 3, base + 4, base + 5],
                                });
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
                            // Instance cap (audit 2026-10-30): a 1e9-count
                            // pattern would clone the base mesh per element
                            // and OOM before the user sees anything. Over-cap
                            // patterns are scaled down proportionally.
                            const MAX_PATTERN_INSTANCES: usize = 10_000;
                            let (cx, cy, cz) =
                                (cx.max(1) as usize, cy.max(1) as usize, cz.max(1) as usize);
                            let total = cx * cy * cz;
                            let (cx, cy, cz) = if total > MAX_PATTERN_INSTANCES {
                                eprintln!(
                                    "[WARN] pattern capped: {} requested instances > limit {}",
                                    total, MAX_PATTERN_INSTANCES
                                );
                                let scale = (MAX_PATTERN_INSTANCES as f64 / total as f64).cbrt();
                                (
                                    (cx as f64 * scale).max(1.0) as usize,
                                    (cy as f64 * scale).max(1.0) as usize,
                                    (cz as f64 * scale).max(1.0) as usize,
                                )
                            } else {
                                (cx, cy, cz)
                            };
                            let sx = depth;
                            let sy = direction.y;
                            let sz = direction.z;
                            let mut result = base_mesh.clone();
                            for ix in 0..cx {
                                for iy in 0..cy.max(1) {
                                    for iz in 0..cz.max(1) {
                                        if ix == 0 && iy == 0 && iz == 0 {
                                            continue;
                                        }
                                        let tx = ix as f64 * sx;
                                        let ty = iy as f64 * sy;
                                        let tz = iz as f64 * sz;
                                        let offset = result.vertices.len() as i32;
                                        for v in &base_mesh.vertices {
                                            result.vertices.push(c::Point3D {
                                                x: v.x + tx,
                                                y: v.y + ty,
                                                z: v.z + tz,
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
        let mut normals = vec![
            c::Vector3D {
                x: 0.0,
                y: 0.0,
                z: 0.0
            };
            mesh.vertices.len()
        ];
        for face in &mesh.faces {
            if face.vertex_indices.len() < 3 {
                continue;
            }
            let i0 = face.vertex_indices[0] as usize;
            let i1 = face.vertex_indices[1] as usize;
            let i2 = face.vertex_indices[2] as usize;
            if i0 >= mesh.vertices.len() || i1 >= mesh.vertices.len() || i2 >= mesh.vertices.len() {
                continue;
            }
            let v0 = &mesh.vertices[i0];
            let v1 = &mesh.vertices[i1];
            let v2 = &mesh.vertices[i2];
            let e1x = v1.x - v0.x;
            let e1y = v1.y - v0.y;
            let e1z = v1.z - v0.z;
            let e2x = v2.x - v0.x;
            let e2y = v2.y - v0.y;
            let e2z = v2.z - v0.z;
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
                n.x /= len;
                n.y /= len;
                n.z /= len;
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
            let obj = self
                .objects
                .get(component_id)
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

    /// Atomic save: serialize to `{path}.tmp` then rename over `{path}`.
    /// A crash mid-write can never leave a truncated project file behind.
    pub fn save(&self, path: &str) -> Result<(), String> {
        let json =
            serde_json::to_string_pretty(self).map_err(|e| format!("serialize error: {}", e))?;
        if let Some(parent) = std::path::Path::new(path).parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| format!("mkdir error: {}", e))?;
            }
        }
        let tmp = format!("{}.tmp", path);
        std::fs::write(&tmp, json).map_err(|e| format!("write error: {}", e))?;
        std::fs::rename(&tmp, path).map_err(|e| format!("rename error: {}", e))?;
        Ok(())
    }

    pub fn load(path: &str) -> Result<Self, String> {
        let content = std::fs::read_to_string(path).map_err(|e| format!("read error: {}", e))?;
        let mut proj: Self =
            serde_json::from_str(&content).map_err(|e| format!("deserialize error: {}", e))?;
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
        let content = std::fs::read_to_string(path).map_err(|e| format!("read error: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("deserialize error: {}", e))
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
    let mut min = Point3D {
        x: f64::MAX,
        y: f64::MAX,
        z: f64::MAX,
    };
    let mut max = Point3D {
        x: f64::NEG_INFINITY,
        y: f64::NEG_INFINITY,
        z: f64::NEG_INFINITY,
    };
    for v in &intersect_mesh.vertices {
        min.x = min.x.min(v.x);
        min.y = min.y.min(v.y);
        min.z = min.z.min(v.z);
        max.x = max.x.max(v.x);
        max.y = max.y.max(v.y);
        max.z = max.z.max(v.z);
    }
    // Expand bbox slightly for numerical tolerance
    let eps = 0.001;
    min.x -= eps;
    min.y -= eps;
    min.z -= eps;
    max.x += eps;
    max.y += eps;
    max.z += eps;

    let mut result = c::MeshGeometry {
        vertices: Vec::new(),
        normals: Vec::new(),
        uvs: Vec::new(),
        faces: Vec::new(),
    };

    for face in &host.faces {
        if face.vertex_indices.len() < 3 {
            continue;
        }
        let i0 = face.vertex_indices[0] as usize;
        let i1 = face.vertex_indices[1] as usize;
        let i2 = face.vertex_indices[2] as usize;
        if i0 >= host.vertices.len() || i1 >= host.vertices.len() || i2 >= host.vertices.len() {
            continue;
        }
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
            if i0 < host.normals.len() {
                result.normals.push(host.normals[i0].clone());
            }
            if i1 < host.normals.len() {
                result.normals.push(host.normals[i1].clone());
            }
            if i2 < host.normals.len() {
                result.normals.push(host.normals[i2].clone());
            }
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
        writeln!(
            f,
            "  Admin Divisions: {}",
            self.administrative_division_count
        )?;
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
        assert_eq!(
            stats.material_count,
            crate::seed_materials::seed_default_materials_len() + 1
        );
        assert_eq!(stats.containment_count, 2);
    }

    #[test]
    fn test_set_parent_replaces_and_cycle_guards() {
        setup_library();
        let mut proj = Project::new("TestProject");
        let site_id = proj.create_object(c::ObjectType::Site, "S").id.clone();
        let bldg_id = proj.create_object(c::ObjectType::Building, "B").id.clone();
        let room_id = proj.create_object(c::ObjectType::Space, "R").id.clone();
        proj.set_parent(&bldg_id, &site_id).unwrap();
        proj.set_parent(&room_id, &bldg_id).unwrap();
        assert_eq!(
            proj.relations.parent_of(&room_id).unwrap().parent_id,
            bldg_id
        );

        // Reparent to another parent: the old containment edge must be gone,
        // not duplicated (the half-transaction hole this fixes).
        proj.set_parent(&room_id, &site_id).unwrap();
        assert_eq!(
            proj.relations.parent_of(&room_id).unwrap().parent_id,
            site_id
        );
        assert_eq!(proj.relations.children_of(&bldg_id).len(), 0);
        assert_eq!(proj.relations.children_of(&site_id).len(), 2);
        assert_eq!(
            proj.objects.get(&room_id).unwrap().parent_id.as_deref(),
            Some(site_id.as_str())
        );
        assert_eq!(proj.stats().containment_count, 2);

        // Cycle guard: reparenting an ancestor under a descendant is refused.
        assert!(proj.set_parent(&site_id, &room_id).is_err());
        assert_eq!(proj.objects.get(&site_id).unwrap().parent_id, None);
    }

    #[test]
    fn test_delete_object_cascade_and_restore() {
        setup_library();
        let mut proj = Project::new("TestProject");
        let site_id = proj.create_object(c::ObjectType::Site, "S").id.clone();
        let bldg_id = proj.create_object(c::ObjectType::Building, "B").id.clone();
        let b2_id = proj.create_object(c::ObjectType::Building, "B2").id.clone();
        proj.set_parent(&bldg_id, &site_id).unwrap();
        proj.set_parent(&b2_id, &site_id).unwrap();
        // awaitable connection + lifecycle + breakdown links
        proj.connect(&bldg_id, &b2_id, c::ConnectionType::Structural)
            .unwrap();
        let record = c::ProjectLifecycleRecord {
            id: "lc-1".into(),
            phase: c::LifecyclePhase::Design as i32,
            record_type: "note".into(),
            name: "note".into(),
            description: String::new(),
            related_object_ids: vec![bldg_id.clone()],
            related_document_ids: Vec::new(),
            ..Default::default()
        };
        proj.add_lifecycle_record(record);

        let (obj, cascade) = proj.delete_object(&bldg_id).unwrap();
        assert_eq!(obj.id, bldg_id);
        // Child edges gone, promoted children keep the site visible.
        assert!(proj.objects.get(&bldg_id).is_none());
        assert!(proj.objects.get(&b2_id).is_some());
        assert!(proj.relations.parent_of(&bldg_id).is_none());
        assert!(
            proj.relations.connections().is_empty(),
            "connections cleaned"
        );
        assert_eq!(
            proj.lifecycle.get("lc-1").unwrap().related_object_ids.len(),
            0
        );
        assert_eq!(cascade.removed_containments.len(), 1);
        assert_eq!(cascade.removed_connections.len(), 1);
        assert_eq!(cascade.lifecycle_refs.len(), 1);

        // Also refuse deleting a storey that still anchors objects.
        let storey_id = proj
            .create_object(c::ObjectType::Floor, "Storey")
            .id
            .clone();
        let wall_id = proj.create_object(c::ObjectType::Wall, "W").id.clone();
        proj.objects.get_mut(&wall_id).unwrap().level_id = Some(storey_id.clone());
        assert!(proj.delete_object(&storey_id).is_err());
        proj.objects.get_mut(&wall_id).unwrap().level_id = None;

        // One-step undo restores object, children, edges and references.
        proj.restore_deleted(obj, &cascade);
        assert!(proj.objects.get(&bldg_id).is_some());
        assert_eq!(
            proj.relations.parent_of(&bldg_id).unwrap().parent_id,
            site_id
        );
        assert_eq!(proj.relations.connections().len(), 1);
        assert_eq!(
            proj.lifecycle.get("lc-1").unwrap().related_object_ids.len(),
            1
        );
        assert_eq!(proj.stats().containment_count, 2);
    }

    #[test]
    fn test_delete_subtree_removes_and_restores_all_descendants() {
        let mut proj = Project::new("TestProject");
        let site_id = proj.create_object(c::ObjectType::Site, "S").id.clone();
        let asm_id = proj.create_object(c::ObjectType::Assembly, "A").id.clone();
        let p1 = proj
            .create_object(c::ObjectType::Component, "P1")
            .id
            .clone();
        let p2 = proj.create_object(c::ObjectType::Part, "P2").id.clone();
        proj.set_parent(&asm_id, &site_id).unwrap();
        proj.set_parent(&p1, &asm_id).unwrap();
        proj.set_parent(&p2, &asm_id).unwrap();
        proj.connect(&p1, &p2, c::ConnectionType::Structural)
            .unwrap();

        let (obj, cascade) = proj.delete_object_subtree(&asm_id).unwrap();
        assert_eq!(obj.id, asm_id);
        assert_eq!(cascade.deleted_children.len(), 2);
        assert_eq!(cascade.removed_containments.len(), 3); // site→A, A→P1, A→P2
        assert_eq!(cascade.removed_connections.len(), 1);
        assert!(proj.objects.get(&asm_id).is_none());
        assert!(proj.objects.get(&p1).is_none());
        assert!(proj.objects.get(&p2).is_none());
        assert!(
            proj.objects.get(&site_id).is_some(),
            "outside parent survives"
        );
        assert!(proj.relations.connections().is_empty());

        proj.restore_deleted(obj, &cascade);
        assert!(proj.objects.get(&asm_id).is_some());
        assert!(proj.objects.get(&p1).is_some());
        assert!(proj.objects.get(&p2).is_some());
        assert_eq!(
            proj.relations.parent_of(&asm_id).unwrap().parent_id,
            site_id
        );
        assert_eq!(proj.relations.parent_of(&p1).unwrap().parent_id, asm_id);
        assert_eq!(proj.relations.children_of(&asm_id).len(), 2);
        assert_eq!(proj.relations.connections().len(), 1);
        assert_eq!(proj.stats().containment_count, 3);
    }

    #[test]
    fn test_delete_subtree_rejects_anchored_level() {
        let mut proj = Project::new("TestProject");
        let storey = proj
            .create_object(c::ObjectType::Floor, "Storey")
            .id
            .clone();
        // Anchored from OUTSIDE the subtree: deleting the storey would leave a
        // dangling datum reference, so it must be refused.
        let outside = proj
            .create_object(c::ObjectType::Wall, "W-outside")
            .id
            .clone();
        proj.objects.get_mut(&outside).unwrap().level_id = Some(storey.clone());
        assert!(proj.delete_object_subtree(&storey).is_err());
        assert!(proj.objects.get(&storey).is_some());
        assert!(proj.objects.get(&outside).is_some());
    }

    #[test]
    fn test_create_wall_from_default_template() {
        setup_library();
        let mut proj = Project::new("TestProject");

        let params = std::collections::HashMap::new();
        let wall_id = proj
            .create_object_from_default_template(c::ObjectType::Wall, "Wall-1", &params, None)
            .unwrap();

        let obj = proj.objects.get(&wall_id).unwrap();
        assert_eq!(obj.object_type, c::ObjectType::Component as i32);
        assert!(
            obj.features.len() > 0,
            "wall should have features from default template"
        );
        assert!(
            obj.geometry.is_some(),
            "wall should have geometry after rebuild"
        );
    }

    #[test]
    fn test_create_beam_from_default_template() {
        setup_library();
        let mut proj = Project::new("TestProject");

        let mut params = std::collections::HashMap::new();
        params.insert("length".into(), 3000.0);
        let beam_id = proj
            .create_object_from_default_template(c::ObjectType::Beam, "Beam-1", &params, None)
            .unwrap();

        let obj = proj.objects.get(&beam_id).unwrap();
        assert!(obj.features.len() > 0);
    }

    #[test]
    fn test_create_site_no_default_template() {
        setup_library();
        let mut proj = Project::new("TestProject");

        let params = std::collections::HashMap::new();
        let site_id = proj
            .create_object_from_default_template(c::ObjectType::Site, "Site-1", &params, None)
            .unwrap();

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
        assert!(
            errors.is_empty(),
            "params in range should have no errors: {:?}",
            errors
        );
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
        assert!(
            errors.len() >= 2,
            "should have at least 2 errors: {:?}",
            errors
        );
    }

    #[test]
    fn test_find_and_rebuild_catalog_instances() {
        let _library_guard = super::LIBRARY_TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        setup_library();
        let mut proj = Project::new("TestProject");

        let params = std::collections::HashMap::new();
        let _id1 = proj
            .create_object_from_default_template(c::ObjectType::Wall, "Wall-A", &params, None)
            .unwrap();
        let _id2 = proj
            .create_object_from_default_template(c::ObjectType::Wall, "Wall-B", &params, None)
            .unwrap();

        let wall_comp_id = {
            let lib = LIBRARY.read().unwrap();
            let lib = lib.as_ref().unwrap();
            lib.catalog
                .find_by_name("Concrete-Wall")
                .unwrap()
                .id
                .clone()
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
        let id = proj
            .instantiate_from_series("CS150", "PIPE", "Pipe-1", &params, None)
            .unwrap();
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
        let id = proj
            .instantiate_from_series("Steel-Q355", "beam", "Beam-1", &params, None)
            .unwrap();
        let obj = proj.objects.get(&id).unwrap();
        assert!(obj.features.len() > 0);
    }

    #[test]
    fn test_instantiate_from_series_road() {
        setup_library();
        let mut proj = Project::new("TestProject");
        setup_series(&mut proj);
        let params = std::collections::HashMap::new();
        let id = proj
            .instantiate_from_series("Highway-GB", "ASPHALT", "Road-1", &params, None)
            .unwrap();
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
        assert!(
            errors.is_empty(),
            "valid params should have no errors: {:?}",
            errors
        );
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
        let id = proj
            .instantiate_catalog_component_skip_gate("Concrete-Wall", "Wall-1", &params, None, 0.0)
            .expect("instantiate by name should work");
        let obj = proj.objects.get(&id).unwrap();
        assert!(obj.features.len() > 0);
    }
}
