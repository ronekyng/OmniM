use std::{collections::HashMap, sync::Arc};
use axum::{
    extract::{DefaultBodyLimit, Path, Query, State},
    http::StatusCode,
    response::{Json, IntoResponse},
    routing::{get, post, put, delete},
    Router,
};
use omnim_model::{Project, LIBRARY};
use omnim_core as c;
use omnim_amir::{AmirEngine, AmirConfig, AmirError};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

pub mod mutation;
use mutation::ApiError;

/// Shared application state: project + Omcom executor + AMIR AI engine
pub struct AppState {
    pub project: SharedProject,
    pub executor: RwLock<omnim_omcom::Executor>,
    pub amir: Option<AmirEngine>,
    pub save_path: Option<String>,
}

pub type SharedProject = Arc<RwLock<Project>>;
pub type SharedState = Arc<AppState>;

pub fn create_router(project: SharedProject, save_path: Option<String>) -> Router {
    let amir = AmirConfig::from_env()
        .ok()
        .and_then(|cfg| {
            let provider_name = cfg.provider.clone();
            match AmirEngine::new(cfg) {
                Ok(engine) => {
                    println!("  AMIR AI engine initialized (provider: {})", provider_name);
                    Some(engine)
                }
                Err(e) => {
                    println!("  AMIR AI engine initialization failed: {}", e);
                    None
                }
            }
        });

    let state = Arc::new(AppState {
        project,
        executor: RwLock::new(omnim_omcom::Executor::new()),
        amir,
        save_path,
    });
    build_router(state)
}

fn build_router(state: SharedState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/project", get(get_project))
        .route("/api/project/save", post(save_project))
        .route("/api/tree", get(get_tree))
        .route("/api/stats", get(get_stats))
        .route("/api/objects", get(list_objects))
        .route("/api/objects/{id}", get(get_object).put(update_object).delete(delete_object))
        .route("/api/objects", post(create_object))
        .route("/api/objects/{id}/prop", put(set_property))
        .route("/api/object-types/{type}/properties", get(get_object_type_properties))
        .route("/api/objects/{id}/properties", put(update_properties))
        .route("/api/objects/{id}/transform", put(transform_object))
        .route("/api/objects/{id}/parent", put(set_parent))
        .route("/api/objects/{from}/connect/{to}", post(connect))
        .route("/api/materials", get(list_materials).post(create_material))
        .route("/api/objects/{id}/material", put(assign_material))
        .route("/api/measurements", get(list_measurements).post(create_measurement))
        .route("/api/measurements/{id}", delete(delete_measurement))
        .route("/api/lifecycle", get(list_lifecycle).post(create_lifecycle))
        .route("/api/lifecycle/{id}/link/{obj_id}", post(link_record))
        .route("/api/administrative", get(list_administrative).post(create_administrative))
        .route("/api/administrative/{id}", get(get_administrative).delete(delete_administrative))
        .route("/api/administrative/{id}/children", get(get_admin_children))
        .route("/api/administrative/ancestors/{id}", get(get_admin_ancestors))
        .route("/api/administrative/code/{code}", get(get_admin_by_code))
        .route("/api/breakdown", get(list_breakdown).post(create_breakdown))
        .route("/api/breakdown/{id}", get(get_breakdown).delete(delete_breakdown))
        .route("/api/breakdown/{id}/children", get(get_breakdown_children))
        .route("/api/breakdown/{id}/descendants", get(get_breakdown_descendants))
        .route("/api/breakdown/{id}/ancestors", get(get_breakdown_ancestors))
        .route("/api/breakdown/{id}/link-obj", post(link_breakdown_object))
        .route("/api/breakdown/{id}/link-admin", post(link_breakdown_admin))
        .route("/api/omcom", post(execute_omcom))
        .route("/api/mesh", get(get_mesh))
        .route("/api/export/ifc", get(export_ifc))
        .route("/api/import/ifc", post(import_ifc))
        .route("/api/genrule", post(genrule_create))
        .route("/api/undo", post(undo))
        .route("/api/redo", post(redo))
        .route("/api/undo-status", get(undo_status))
        .route("/api/amir/translate", post(amir_translate))
        .route("/api/amir/chat", post(amir_chat))
        .route("/api/amir/describe", post(amir_describe))
        .route("/api/amir/explain", post(amir_explain))
        .route("/api/sketches", get(list_sketches).post(create_sketch))
        .route("/api/sketches/{id}", get(get_sketch).put(update_sketch).delete(delete_sketch))
        .route("/api/sketches/{id}/entities", post(add_sketch_entity))
        .route("/api/sketches/{id}/constraints", post(add_sketch_constraint))
        .route("/api/sketches/{id}/solve", post(solve_sketch))
        .route("/api/sketches/{id}/entities/{idx}", put(update_sketch_entity).delete(delete_sketch_entity))
        .route("/api/sketches/{id}/constraints/{idx}", delete(remove_sketch_constraint))
        .route("/api/sketches/{id}/extrude", post(extrude_sketch_handler))
        .route("/api/sketches/{id}/revolve", post(revolve_sketch_handler))
        .route("/api/components", get(list_components).post(create_component_handler))
        .route("/api/components/{id}", get(get_component).delete(delete_component))
        .route("/api/components/{id}/features", get(list_features).post(add_feature_handler))
        .route("/api/components/{id}/features/{fid}", put(update_feature_handler).delete(remove_feature_handler))
        .route("/api/components/{id}/rebuild", post(rebuild_component_handler))
        .route("/api/components/{id}/section", get(section_handler))
        // === Catalog routes ===
        .route("/api/catalog", get(list_catalog).post(create_component_def))
        .route("/api/catalog/{id}", get(get_component_def).put(update_component_def).delete(delete_component_def))
        .route("/api/catalog/{id}/params", post(add_component_param))
        .route("/api/catalog/{id}/params/{idx}", put(update_component_param).delete(delete_component_param))
        .route("/api/catalog/{id}/connectors", post(add_connector))
        .route("/api/catalog/{id}/feature-tree", put(assign_feature_tree))
        .route("/api/catalog/{id}/publish", post(publish_component))
        .route("/api/catalog/export", post(export_catalog_handler))
        .route("/api/catalog/import", post(import_catalog_handler))
        // === Series routes ===
        .route("/api/series", get(list_series).post(create_series_handler))
        .route("/api/series/save", post(save_series_handler))
        .route("/api/series/load", post(load_series_handler))
        .route("/api/series/{id}", get(get_series).put(update_series_handler).delete(delete_series_handler))
        .route("/api/series/{id}/selectors", post(add_selector_handler))
        .route("/api/series/{id}/answers", post(add_answer_handler))
        .route("/api/series/{id}/spec", post(add_spec_component_handler))
        .route("/api/series/{id}/constraints", post(add_param_constraint_handler))
        .route("/api/series/{id}/constraints/{idx}", put(update_param_constraint_handler).delete(delete_param_constraint_handler))
        .route("/api/series/{id}/select", post(select_from_series))
        .route("/api/catalog/instantiate", post(instantiate_from_catalog))
        .route("/api/catalog/preview", post(preview_catalog_component))
        .layer(DefaultBodyLimit::max(100 * 1024 * 1024)) // 100MB for IFC import
        .fallback_service(ServeDir::new("web"))
        .with_state(state)
}

async fn health() -> &'static str { "ok" }

async fn get_project(State(state): State<SharedState>) -> Json<Project> {
    Json(state.project.read().await.clone())
}

/// POST /api/project/save — explicit persistence point.
///
/// Independent of `OMNIM_EPHEMERAL`-driven auto-save, the client can force a
/// save. Responds 200 with the path that was written, or 400 when persistence
/// is disabled, 500 on write failure.
async fn save_project(
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let Some(path) = state.save_path.clone() else {
        return Err((StatusCode::BAD_REQUEST, "persistence is disabled (OMNIM_EPHEMERAL=1)".into()));
    };
    let p = state.project.read().await;
    p.save(&path).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("save failed: {}", e)))?;
    Ok(Json(serde_json::json!({ "success": true, "saved": path })))
}

#[derive(Serialize, Clone)]
struct TreeNode {
    id: String,
    name: String,
    object_type: String,
    ifc_class: String,
    has_geometry: bool,
    children: Vec<TreeNode>,
}

async fn get_tree(State(state): State<SharedState>) -> Json<Vec<TreeNode>> {
    let p = state.project.read().await;
    let roots = p.objects.roots();
    Json(roots.into_iter().map(|obj| build_tree_node(&p, obj)).collect())
}

fn build_tree_node(project: &Project, obj: &omnim_model::OmniObject) -> TreeNode {
    TreeNode {
        id: obj.id.clone(),
        name: obj.name.clone(),
        object_type: obj.object_type_name().to_string(),
        ifc_class: obj.ifc_class.clone(),
        has_geometry: obj.geometry.is_some(),
        children: project.objects.children_of(&obj.id).into_iter().map(|child| build_tree_node(project, child)).collect(),
    }
}

#[derive(Serialize)]
struct StatsResponse {
    name: String,
    object_count: usize,
    material_count: usize,
    lifecycle_record_count: usize,
    connection_count: usize,
    containment_count: usize,
}

async fn get_stats(State(state): State<SharedState>) -> Json<StatsResponse> {
    let p = state.project.read().await;
    Json(StatsResponse {
        name: p.name.clone(),
        object_count: p.objects.len(),
        material_count: p.materials.len(),
        lifecycle_record_count: p.lifecycle.len(),
        connection_count: p.relations.connections().len(),
        containment_count: p.relations.containments().len(),
    })
}

async fn list_objects(State(state): State<SharedState>) -> Json<Vec<omnim_model::OmniObject>> {
    let p = state.project.read().await;
    Json(p.objects.list().into_iter().cloned().collect())
}

async fn get_object(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<omnim_model::OmniObject>, StatusCode> {
    let p = state.project.read().await;
    p.objects.get(&id).cloned().map(Json).ok_or(StatusCode::NOT_FOUND)
}

#[derive(Deserialize)]
struct CreateObjectRequest {
    object_type: String,
    name: String,
    description: Option<String>,
    ifc_class: Option<String>,
    tags: Option<Vec<String>>,
    geometry: Option<omnim_core::ParametricGeometry>,
    parent_id: Option<String>,
}

async fn create_object(
    State(state): State<SharedState>,
    Json(req): Json<CreateObjectRequest>,
) -> Result<Json<omnim_model::OmniObject>, StatusCode> {
    let mut p = state.project.write().await;
    let mut exe = state.executor.write().await;

    let parent_id = req.parent_id.clone().unwrap_or_default();
    let instr = c::OmniCommand {
        version: "1.0".into(),
        id: format!("create-{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()),
        action: c::Action::CreateObject as i32,
        actor: None,
        target: None,
        operations: vec![c::Operation {
            op: Some(c::operation::Op::CreateObject(c::CreateObjectOperation {
                r#type: parse_type(&req.object_type) as i32,
                name: req.name.clone(),
                description: req.description.clone().unwrap_or_default(),
                geometry: req.geometry.clone(),
                properties: vec![],
                parent_id,
            })),
        }],
        validation: None,
        timestamp: None,
        metadata: Default::default(),
    };

    let result = exe.execute(&instr, &mut p);
    if !result.success {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }

    let obj_id = result.created_ids.first().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?.clone();

    // If parent_id was specified, set up the containment relationship through
    // the executor (MoveObject) so it lands on the undo stack too.
    if let Some(ref pid) = req.parent_id {
        if !pid.is_empty() {
            let op = c::Operation {
                op: Some(c::operation::Op::MoveObject(c::MoveObjectOperation {
                    object_id: obj_id.clone(),
                    new_parent_id: pid.clone(),
                })),
            };
            exe.execute(&mutation::envelope(c::Action::MoveObject, vec![op]), &mut p);
        }
    }

    let obj = p.objects.get(&obj_id).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?.clone();

    // Apply extra fields not supported by Omcom CreateObject protocol
    if req.ifc_class.is_some() || req.tags.is_some() {
        if let Some(obj) = p.objects.get_mut(&obj_id) {
            if let Some(ifc_class) = &req.ifc_class {
                obj.ifc_class = ifc_class.clone();
            }
            if let Some(tags) = req.tags.clone() {
                obj.add_tags(tags);
            }
        }
    }

    mutation::persist(&state.save_path, &p);
    Ok(Json(obj))
}

#[derive(Deserialize)]
struct UpdateObjectRequest {
    name: Option<String>,
    description: Option<String>,
    ifc_class: Option<String>,
    tags: Option<Vec<String>>,
}

async fn update_object(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateObjectRequest>,
) -> Result<(), ApiError> {
    if !state.project.read().await.objects.contains(&id) {
        return Err(ApiError::not_found(format!("object '{}' not found", id)));
    }
    let result = mutation::apply(
        &state,
        c::Action::ModifyObject,
        vec![mutation::op_modify_object(
            &id,
            req.name,
            req.description,
            req.ifc_class,
            req.tags,
        )],
    )
    .await?;
    mutation::require_success(result)?;
    Ok(())
}

#[derive(Deserialize)]
struct UpdatePropertiesRequest {
    name: Option<String>,
    description: Option<String>,
    ifc_class: Option<String>,
    tags: Option<Vec<String>>,
    properties: Option<HashMap<String, String>>,
}

async fn update_properties(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<UpdatePropertiesRequest>,
) -> Result<(), ApiError> {
    if !state.project.read().await.objects.contains(&id) {
        return Err(ApiError::not_found(format!("object '{}' not found", id)));
    }
    let mut ops = vec![mutation::op_modify_object(
        &id,
        req.name,
        req.description,
        req.ifc_class,
        req.tags,
    )];
    if let Some(properties) = req.properties {
        for (name, value) in properties {
            ops.push(mutation::op_set_property(&id, &name, &value));
        }
    }
    let result = mutation::apply(&state, c::Action::SetProperty, ops).await?;
    mutation::require_success(result)?;
    Ok(())
}

async fn delete_object(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<(), StatusCode> {
    let mut p = state.project.write().await;
    let mut exe = state.executor.write().await;
    let obj = p.objects.remove(&id).ok_or(StatusCode::NOT_FOUND)?;
    exe.record_delete(&obj);
    mutation::persist(&state.save_path, &p);
    Ok(())
}

#[derive(Deserialize)]
struct SetPropertyRequest {
    name: String,
    value: String,
}

async fn set_property(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<SetPropertyRequest>,
) -> Result<(), ApiError> {
    if !state.project.read().await.objects.contains(&id) {
        return Err(ApiError::not_found(format!("object '{}' not found", id)));
    }
    let result = mutation::apply(
        &state,
        c::Action::SetProperty,
        vec![mutation::op_set_property(&id, &req.name, &req.value)],
    )
    .await?;
    mutation::require_success(result)?;
    Ok(())
}

fn type_name_to_object_type(name: &str) -> Option<i32> {
    use omnim_core::ObjectType::*;
    let t = match name.to_lowercase().as_str() {
        "wall" | "wallstandardcase" => Wall as i32,
        "slab" => Slab as i32,
        "roof" => Roof as i32,
        "floor" => Floor as i32,
        "beam" => Beam as i32,
        "column" => Column as i32,
        "door" => Door as i32,
        "window" => Window as i32,
        "road" => Road as i32,
        "pipe" | "pipeplant" | "pipe_civil" => PipePlant as i32,
        "duct" => Duct as i32,
        "cabletray" | "cable_tray" => CableTray as i32,
        "tank" => Tank as i32,
        "pump" => Pump as i32,
        "valve" => Valve as i32,
        "gear" => Gear as i32,
        "part" => Part as i32,
        "assembly" => Assembly as i32,
        "component" => Component as i32,
        _ => return None,
    };
    Some(t)
}

async fn get_object_type_properties(
    Path(type_name): Path<String>,
) -> Json<Vec<omnim_model::ObjectPropertyDef>> {
    let defs = match type_name_to_object_type(&type_name) {
        Some(ty) => omnim_model::property_defs_for_type(ty),
        None => Vec::new(),
    };
    Json(defs)
}

#[derive(Deserialize)]
struct TransformObjectRequest {
    origin: Option<omnim_core::Point3D>,
    translation: Option<omnim_core::Point3D>,
    rotation: Option<omnim_core::Rotation>,
    scale: Option<omnim_core::Vector3D>,
}

async fn transform_object(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<TransformObjectRequest>,
) -> Result<(), ApiError> {
    if !state.project.read().await.objects.contains(&id) {
        return Err(ApiError::not_found(format!("object '{}' not found", id)));
    }
    let op = c::Operation {
        op: Some(c::operation::Op::TransformGeometry(c::TransformGeometryOperation {
            object_id: id.clone(),
            translation: req.translation,
            rotation: req.rotation,
            scale: req.scale,
            pivot: req.origin,
        })),
    };
    let result = mutation::apply(&state, c::Action::TransformGeometry, vec![op]).await?;
    mutation::require_success(result)?;
    Ok(())
}

#[derive(Deserialize)]
struct SetParentRequest {
    parent_id: String,
}

async fn set_parent(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<SetParentRequest>,
) -> Result<(), ApiError> {
    // Same semantics as Project::set_parent: reparent + spatial containment,
    // now as two undoable ops.
    if !state.project.read().await.objects.contains(&id) {
        return Err(ApiError::not_found(format!("object '{}' not found", id)));
    }
    let ops = vec![
        mutation::op_relationship(c::operation::Op::MoveObject(c::MoveObjectOperation {
            object_id: id.clone(),
            new_parent_id: req.parent_id.clone(),
        })),
        mutation::op_relationship(c::operation::Op::AddContainment(c::Containment {
            parent_id: req.parent_id,
            child_id: id,
            r#type: c::ContainmentType::Spatial as i32,
        })),
    ];
    let result = mutation::apply(&state, c::Action::MoveObject, ops).await?;
    mutation::require_success(result)?;
    Ok(())
}

#[derive(Deserialize)]
struct ConnectQuery {
    /// Connection type name (structural | mechanical | electrical | hydraulic |
    /// spatial_adjacent). Defaults to structural — the historical behavior.
    #[serde(default)]
    connection_type: Option<String>,
}

/// Connection-type codes advertised by an object's ComponentDef connectors
/// (empty when the object is not a catalog instance or has no typed
/// connectors). Used by the connector-compatibility check.
fn connector_type_codes(project: &Project, object_id: &str) -> Option<Vec<String>> {
    let obj = project.objects.get(object_id)?;
    let def_id = obj.catalog_ref.as_ref()?.component_def_id.clone();
    let library = LIBRARY.read().unwrap();
    let library = library.as_ref()?;
    let def = library.catalog.get(&def_id)
        .or_else(|| library.catalog.find_by_name(&def_id))?;
    let codes: Vec<String> = def
        .connectors
        .iter()
        .map(|c| c.connection_type_code.trim().to_lowercase())
        .filter(|c| !c.is_empty())
        .collect();
    if codes.is_empty() { None } else { Some(codes) }
}

fn parse_connection_type(name: &str) -> Result<omnim_core::ConnectionType, String> {
    match name.trim().to_lowercase().as_str() {
        "" | "structural" => Ok(omnim_core::ConnectionType::Structural),
        "mechanical" => Ok(omnim_core::ConnectionType::Mechanical),
        "electrical" => Ok(omnim_core::ConnectionType::Electrical),
        "hydraulic" => Ok(omnim_core::ConnectionType::Hydraulic),
        "spatial" | "spatial_adjacent" => Ok(omnim_core::ConnectionType::SpatialAdjacent),
        other => Err(format!(
            "unknown connection type '{}' (structural | mechanical | electrical | hydraulic | spatial_adjacent)",
            other
        )),
    }
}

async fn connect(
    State(state): State<SharedState>,
    Path((from, to)): Path<(String, String)>,
    Query(query): Query<ConnectQuery>,
) -> Result<(), ApiError> {
    // Enforcement added in Wave 4 (audit P0-5: "connect() validates nothing"):
    // no self-connection, no duplicate pair, known connection type, and
    // connector type-code compatibility for catalog instances.
    if from == to {
        return Err(ApiError::bad_request("cannot connect an object to itself"));
    }
    if !state.project.read().await.objects.contains(&from) {
        return Err(ApiError::not_found(format!("object '{}' not found", from)));
    }
    if !state.project.read().await.objects.contains(&to) {
        return Err(ApiError::not_found(format!("object '{}' not found", to)));
    }
    let conn_type = match &query.connection_type {
        Some(name) => parse_connection_type(name).map_err(ApiError::bad_request)?,
        None => omnim_core::ConnectionType::Structural,
    };
    {
        let p = state.project.read().await;
        let duplicate = p.relations.connections().iter().any(|c| {
            (c.from_object_id == from && c.to_object_id == to)
                || (c.from_object_id == to && c.to_object_id == from)
        });
        if duplicate {
            return Err(ApiError::bad_request(format!(
                "objects '{}' and '{}' are already connected",
                from, to
            )));
        }
        let codes_from = connector_type_codes(&p, &from);
        let codes_to = connector_type_codes(&p, &to);
        if let (Some(cf), Some(ct)) = (&codes_from, &codes_to) {
            if cf.iter().all(|c| !ct.contains(c)) {
                return Err(ApiError::bad_request(format!(
                    "connector type codes are incompatible: {} advertises {:?}, {} advertises {:?}",
                    from, cf, to, ct
                )));
            }
        }
    }
    let ops = vec![mutation::op_relationship(c::operation::Op::AddConnection(c::Connection {
        from_object_id: from.clone(),
        to_object_id: to.clone(),
        r#type: conn_type as i32,
        from_point: None,
        to_point: None,
        connection_detail: String::new(),
    }))];
    let result = mutation::apply(&state, c::Action::AddConnection, ops).await?;
    mutation::require_success(result)?;
    Ok(())
}

#[derive(Deserialize)]
struct CreateMaterialRequest {
    id: String,
    name: String,
    category: String,
    density: Option<f64>,
}

async fn list_materials(State(state): State<SharedState>) -> Json<Vec<omnim_core::Material>> {
    let p = state.project.read().await;
    Json(p.materials.list().into_iter().cloned().collect())
}

async fn create_material(
    State(state): State<SharedState>,
    Json(req): Json<CreateMaterialRequest>,
) -> Result<(), ApiError> {
    let material = omnim_core::Material {
        id: req.id.clone(),
        name: req.name,
        category: parse_category(&req.category) as i32,
        density: req.density.unwrap_or(0.0),
        ..Default::default()
    };
    let result = mutation::apply(&state, c::Action::CreateMaterial, vec![mutation::op_create_material(material)]).await?;
    mutation::require_success(result)?;
    Ok(())
}

#[derive(Deserialize)]
struct AssignMaterialRequest {
    material_id: String,
}

async fn assign_material(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<AssignMaterialRequest>,
) -> Result<(), ApiError> {
    if !state.project.read().await.objects.contains(&id) {
        return Err(ApiError::not_found(format!("object '{}' not found", id)));
    }
    let ops = vec![mutation::op_relationship(c::operation::Op::SetMaterial(c::SetMaterialOperation {
        object_id: id,
        material_id: req.material_id,
    }))];
    let result = mutation::apply(&state, c::Action::SetMaterial, ops).await?;
    mutation::require_success(result)?;
    Ok(())
}

// ── Measurement endpoints ──

async fn list_measurements(State(state): State<SharedState>) -> Json<Vec<omnim_model::Measurement>> {
    let p = state.project.read().await;
    Json(p.measurements.all().to_vec())
}

#[derive(Deserialize)]
struct CreateMeasurementRequest {
    r#type: String,
    points: Vec<omnim_model::Point3D>,
    value: f64,
    label: Option<String>,
    color: Option<String>,
}

async fn create_measurement(
    State(state): State<SharedState>,
    Json(req): Json<CreateMeasurementRequest>,
) -> Result<Json<omnim_model::Measurement>, ApiError> {
    let op = c::operation::Op::CreateMeasurement(c::CreateMeasurementOperation {
        measurement_type: req.r#type,
        points: req
            .points
            .iter()
            .map(|p| c::Point3D { x: p.x, y: p.y, z: p.z })
            .collect(),
        value: req.value,
        label: req.label,
        color: req.color,
    });
    let result =
        mutation::apply(&state, c::Action::CreateMeasurement, vec![mutation::op(op)]).await?;
    let result = mutation::require_success(result)?;
    let id = result
        .created_ids
        .first()
        .cloned()
        .ok_or_else(|| ApiError::internal("measurement creation returned no id"))?;
    let p = state.project.read().await;
    let measurement = p
        .measurements
        .all()
        .iter()
        .find(|m| m.id == id)
        .cloned()
        .ok_or_else(|| ApiError::internal("created measurement vanished"))?;
    Ok(Json(measurement))
}

async fn delete_measurement(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<(), ApiError> {
    if !state
        .project
        .read()
        .await
        .measurements
        .all()
        .iter()
        .any(|m| m.id == id)
    {
        return Err(ApiError::not_found(format!("measurement '{}' not found", id)));
    }
    let op = c::operation::Op::DeleteMeasurement(c::DeleteMeasurementOperation {
        measurement_id: id,
    });
    let result =
        mutation::apply(&state, c::Action::DeleteMeasurement, vec![mutation::op(op)]).await?;
    mutation::require_success(result)?;
    Ok(())
}

#[derive(Deserialize)]
struct CreateLifecycleRequest {
    phase: String,
    record_type: String,
    name: String,
    description: Option<String>,
}

async fn list_lifecycle(State(state): State<SharedState>) -> Json<Vec<omnim_core::ProjectLifecycleRecord>> {
    let p = state.project.read().await;
    Json(p.lifecycle.list().into_iter().cloned().collect())
}

async fn create_lifecycle(
    State(state): State<SharedState>,
    Json(req): Json<CreateLifecycleRequest>,
) -> Result<(), ApiError> {
    let op = c::operation::Op::CreateLifecycleRecord(c::CreateLifecycleRecordOperation {
        phase: parse_phase(&req.phase) as i32,
        name: req.name,
        description: req.description.unwrap_or_default(),
        record_type: req.record_type,
        related_object_ids: Vec::new(),
        properties: Vec::new(),
    });
    let result = mutation::apply(
        &state,
        c::Action::CreateLifecycleRecord,
        vec![mutation::op(op)],
    )
    .await?;
    mutation::require_success(result)?;
    Ok(())
}

async fn link_record(
    State(state): State<SharedState>,
    Path((record_id, obj_id)): Path<(String, String)>,
) -> Result<(), ApiError> {
    if state
        .project
        .read()
        .await
        .lifecycle
        .get(&record_id)
        .is_none()
    {
        return Err(ApiError::bad_request(format!(
            "record '{}' not found",
            record_id
        )));
    }
    let op = c::operation::Op::LinkRecordToObject(c::LinkRecordToObjectOperation {
        record_id,
        object_id: obj_id,
        link_role: String::new(),
    });
    let result = mutation::apply(
        &state,
        c::Action::LinkRecordToObject,
        vec![mutation::op(op)],
    )
    .await?;
    mutation::require_success(result)?;
    Ok(())
}

// ============================================================
// Administrative Division endpoints
// ============================================================

#[derive(Serialize)]
struct AdminDivResponse {
    id: String,
    level: String,
    name: String,
    native_name: String,
    code: String,
    parent_id: Option<String>,
}

fn admin_to_response(d: &omnim_model::AdministrativeDivision) -> AdminDivResponse {
    AdminDivResponse {
        id: d.id.clone(),
        level: d.level_name().to_string(),
        name: d.name.clone(),
        native_name: d.native_name.clone(),
        code: d.code.clone(),
        parent_id: d.parent_id.clone(),
    }
}

async fn list_administrative(State(state): State<SharedState>) -> Json<Vec<AdminDivResponse>> {
    let p = state.project.read().await;
    Json(p.administrative.list().into_iter().map(admin_to_response).collect())
}

#[derive(Deserialize)]
struct CreateAdminRequest {
    level: String,
    name: String,
    code: String,
    native_name: Option<String>,
    parent_id: Option<String>,
}

async fn create_administrative(
    State(state): State<SharedState>,
    Json(req): Json<CreateAdminRequest>,
) -> Result<Json<AdminDivResponse>, ApiError> {
    let op = c::operation::Op::CreateAdministrativeDivision(
        c::CreateAdministrativeDivisionOperation {
            level: parse_admin_level(&req.level) as i32,
            name: req.name,
            code: req.code,
            native_name: req.native_name,
            parent_id: req.parent_id,
        },
    );
    let result = mutation::apply(
        &state,
        c::Action::CreateAdministrativeDivision,
        vec![mutation::op(op)],
    )
    .await?;
    let result = mutation::require_success(result)?;
    let id = result
        .created_ids
        .first()
        .cloned()
        .ok_or_else(|| ApiError::internal("administrative division creation returned no id"))?;
    let p = state.project.read().await;
    let division = p
        .administrative
        .get(&id)
        .ok_or_else(|| ApiError::internal("created administrative division vanished"))?;
    Ok(Json(admin_to_response(division)))
}

async fn get_administrative(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<AdminDivResponse>, StatusCode> {
    let p = state.project.read().await;
    p.administrative.get(&id)
        .map(admin_to_response)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn delete_administrative(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<(), ApiError> {
    if state.project.read().await.administrative.get(&id).is_none() {
        return Err(ApiError::not_found(format!(
            "administrative division '{}' not found",
            id
        )));
    }
    let op = c::operation::Op::DeleteAdministrativeDivision(
        c::DeleteAdministrativeDivisionOperation { division_id: id },
    );
    let result = mutation::apply(
        &state,
        c::Action::DeleteAdministrativeDivision,
        vec![mutation::op(op)],
    )
    .await?;
    mutation::require_success(result)?;
    Ok(())
}

async fn get_admin_children(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Json<Vec<AdminDivResponse>> {
    let p = state.project.read().await;
    Json(p.administrative.children_of(&id).into_iter().map(admin_to_response).collect())
}

async fn get_admin_ancestors(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Json<Vec<AdminDivResponse>> {
    let p = state.project.read().await;
    Json(p.administrative.ancestors_of(&id).into_iter().map(admin_to_response).collect())
}

async fn get_admin_by_code(
    State(state): State<SharedState>,
    Path(code): Path<String>,
) -> Result<Json<AdminDivResponse>, StatusCode> {
    let p = state.project.read().await;
    p.administrative.find_by_code(&code)
        .map(admin_to_response)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

// ============================================================
// Breakdown Structure endpoints
// ============================================================

#[derive(Serialize)]
struct BreakdownResponse {
    id: String,
    level: String,
    name: String,
    code: String,
    convention: String,
    parent_id: Option<String>,
    related_object_ids: Vec<String>,
    administrative_division_ids: Vec<String>,
}

fn breakdown_to_response(n: &omnim_model::BreakdownNode) -> BreakdownResponse {
    BreakdownResponse {
        id: n.id.clone(),
        level: n.level_name().to_string(),
        name: n.name.clone(),
        code: n.code.clone(),
        convention: n.convention_name().to_string(),
        parent_id: n.parent_id.clone(),
        related_object_ids: n.related_object_ids.clone(),
        administrative_division_ids: n.administrative_division_ids.clone(),
    }
}

async fn list_breakdown(State(state): State<SharedState>) -> Json<Vec<BreakdownResponse>> {
    let p = state.project.read().await;
    Json(p.breakdown.list().into_iter().map(breakdown_to_response).collect())
}

#[derive(Deserialize)]
struct CreateBreakdownRequest {
    level: String,
    name: String,
    convention: String,
    code: Option<String>,
    parent_id: Option<String>,
}

async fn create_breakdown(
    State(state): State<SharedState>,
    Json(req): Json<CreateBreakdownRequest>,
) -> Result<Json<BreakdownResponse>, ApiError> {
    let op = c::operation::Op::CreateBreakdownNode(c::CreateBreakdownNodeOperation {
        level: parse_breakdown_level(&req.level) as i32,
        name: req.name,
        convention: parse_breakdown_convention(&req.convention) as i32,
        code: req.code,
        parent_id: req.parent_id,
    });
    let result = mutation::apply(
        &state,
        c::Action::CreateBreakdownNode,
        vec![mutation::op(op)],
    )
    .await?;
    let result = mutation::require_success(result)?;
    let id = result
        .created_ids
        .first()
        .cloned()
        .ok_or_else(|| ApiError::internal("breakdown node creation returned no id"))?;
    let p = state.project.read().await;
    let node = p
        .breakdown
        .get(&id)
        .ok_or_else(|| ApiError::internal("created breakdown node vanished"))?;
    Ok(Json(breakdown_to_response(node)))
}

async fn get_breakdown(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<BreakdownResponse>, StatusCode> {
    let p = state.project.read().await;
    p.breakdown.get(&id)
        .map(breakdown_to_response)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn delete_breakdown(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<(), ApiError> {
    if state.project.read().await.breakdown.get(&id).is_none() {
        return Err(ApiError::not_found(format!(
            "breakdown node '{}' not found",
            id
        )));
    }
    let op = c::operation::Op::DeleteBreakdownNode(c::DeleteBreakdownNodeOperation {
        node_id: id,
    });
    let result =
        mutation::apply(&state, c::Action::DeleteBreakdownNode, vec![mutation::op(op)]).await?;
    mutation::require_success(result)?;
    Ok(())
}

async fn get_breakdown_children(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Json<Vec<BreakdownResponse>> {
    let p = state.project.read().await;
    Json(p.breakdown.children_of(&id).into_iter().map(breakdown_to_response).collect())
}

async fn get_breakdown_descendants(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Json<Vec<BreakdownResponse>> {
    let p = state.project.read().await;
    Json(p.breakdown.descendants_of(&id).into_iter().map(breakdown_to_response).collect())
}

async fn get_breakdown_ancestors(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Json<Vec<BreakdownResponse>> {
    let p = state.project.read().await;
    Json(p.breakdown.ancestors_of(&id).into_iter().map(breakdown_to_response).collect())
}

#[derive(Deserialize)]
struct LinkObjectRequest {
    object_id: String,
}

async fn link_breakdown_object(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<LinkObjectRequest>,
) -> Result<(), ApiError> {
    if state.project.read().await.breakdown.get(&id).is_none() {
        return Err(ApiError::not_found(format!(
            "breakdown node '{}' not found",
            id
        )));
    }
    let op = c::operation::Op::LinkBreakdownObject(c::LinkBreakdownObjectOperation {
        node_id: id,
        object_id: req.object_id,
    });
    let result =
        mutation::apply(&state, c::Action::LinkBreakdownObject, vec![mutation::op(op)]).await?;
    mutation::require_success(result)?;
    Ok(())
}

#[derive(Deserialize)]
struct LinkAdminRequest {
    division_id: String,
}

async fn link_breakdown_admin(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<LinkAdminRequest>,
) -> Result<(), ApiError> {
    if state.project.read().await.breakdown.get(&id).is_none() {
        return Err(ApiError::not_found(format!(
            "breakdown node '{}' not found",
            id
        )));
    }
    let op =
        c::operation::Op::LinkBreakdownAdministrative(c::LinkBreakdownAdministrativeOperation {
            node_id: id,
            division_id: req.division_id,
        });
    let result = mutation::apply(
        &state,
        c::Action::LinkBreakdownAdministrative,
        vec![mutation::op(op)],
    )
    .await?;
    mutation::require_success(result)?;
    Ok(())
}

async fn execute_omcom(
    State(state): State<SharedState>,
    Json(instr): Json<omnim_core::OmniCommand>,
) -> Result<Json<omnim_omcom::ExecutionResult>, StatusCode> {
    let mut p = state.project.write().await;
    let mut exe = state.executor.write().await;
    let result = exe.execute(&instr, &mut p);
    mutation::persist(&state.save_path, &p);
    Ok(Json(result))
}

// ============================================================
// AMIR AI Translation endpoints
// ============================================================

/// Build a short project context string from the current project state
fn build_project_context(project: &omnim_model::Project) -> String {
    let obj_count = project.objects.len();
    let mat_count = project.materials.len();
    let lifecycle_count = project.lifecycle.len();
    let conn_count = project.relations.connections().len();
    let containment_count = project.relations.containments().len();

    format!(
        "Project '{}' has {} objects, {} materials, {} lifecycle records, {} connections, {} containments.",
        project.name, obj_count, mat_count, lifecycle_count, conn_count, containment_count,
    )
}

/// POST /api/amir/translate — natural language → Omcom commands
async fn amir_translate(
    State(state): State<SharedState>,
    Json(req): Json<omnim_amir::AmirTranslateRequest>,
) -> Result<Json<omnim_amir::AmirTranslateResponse>, (StatusCode, String)> {
    let engine = state.amir.as_ref().ok_or((
        StatusCode::NOT_IMPLEMENTED,
        "AMIR not configured. Set AMIR_PROVIDER and AMIR_API_KEY env vars.".into(),
    ))?;

    let context = {
        let p = state.project.read().await;
        build_project_context(&p)
    };

    let selected = req.selected_ids.as_deref().unwrap_or(&[]);
    engine.translate_with_context(&req.text, selected, Some(&context)).await
        .map(Json)
        .map_err(|e| map_amir_error(e))
}

/// POST /api/amir/chat — multi-turn conversation
async fn amir_chat(
    State(state): State<SharedState>,
    Json(req): Json<omnim_amir::AmirChatRequest>,
) -> Result<Json<omnim_amir::AmirChatResponse>, (StatusCode, String)> {
    let engine = state.amir.as_ref().ok_or((
        StatusCode::NOT_IMPLEMENTED,
        "AMIR not configured. Set AMIR_PROVIDER and AMIR_API_KEY env vars.".into(),
    ))?;

    let context = {
        let p = state.project.read().await;
        build_project_context(&p)
    };

    let selected = req.selected_ids.as_deref().unwrap_or(&[]);
    engine.chat_with_context(req.session_id, &req.text, selected, Some(&context)).await
        .map(Json)
        .map_err(|e| map_amir_error(e))
}

/// POST /api/amir/describe — model objects → natural language description
async fn amir_describe(
    State(state): State<SharedState>,
    Json(req): Json<omnim_amir::DescribeRequest>,
) -> Result<Json<omnim_amir::DescribeResponse>, (StatusCode, String)> {
    let engine = state.amir.as_ref().ok_or((
        StatusCode::NOT_IMPLEMENTED,
        "AMIR not configured. Set AMIR_PROVIDER and AMIR_API_KEY env vars.".into(),
    ))?;

    let ids = req.object_ids.as_deref();
    engine.describe(ids, &req.format.unwrap_or_else(|| "text".into())).await
        .map(Json)
        .map_err(|e| map_amir_error(e))
}

/// POST /api/amir/explain — validation errors → natural language explanations
async fn amir_explain(
    State(state): State<SharedState>,
    Json(req): Json<omnim_amir::ExplainRequest>,
) -> Json<omnim_amir::ExplainResponse> {
    let engine = match state.amir.as_ref() {
        Some(e) => e,
        None => return Json(omnim_amir::ExplainResponse {
            explanations: vec!["AMIR not configured".into()],
        }),
    };
    Json(engine.explain_errors(&req.errors, req.context.as_deref()))
}

fn map_amir_error(err: AmirError) -> (StatusCode, String) {
    let code = match &err {
        AmirError::LlmError(_) | AmirError::ProviderNotConfigured(_) => StatusCode::BAD_GATEWAY,
        AmirError::ParseError(_) => StatusCode::UNPROCESSABLE_ENTITY,
        AmirError::UnsupportedIntent(_) => StatusCode::BAD_REQUEST,
        AmirError::MissingParameter(_) => StatusCode::BAD_REQUEST,
        AmirError::ReferenceNotFound(_) => StatusCode::NOT_FOUND,
        AmirError::ConfigError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        AmirError::SerdeError(_) => StatusCode::UNPROCESSABLE_ENTITY,
        AmirError::PlanningError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        AmirError::ConflictError(_) => StatusCode::CONFLICT,
    };
    (code, err.to_string())
}

// ============================================================
// GenRule endpoint — smart object creation via GenRule engines
// ============================================================

#[derive(Deserialize)]
#[serde(tag = "genrule")]
enum GenRuleRequest {
    #[serde(rename = "wall")]
    Wall { name: String, length: f64, height: f64, core_thickness: f64, finish_thickness: f64, #[serde(default)] parent_id: Option<String>, #[serde(default)] series_id: Option<String>, #[serde(default)] start_x: f64, #[serde(default)] start_y: f64, #[serde(default)] start_z: f64, #[serde(default)] end_x: f64, #[serde(default)] end_y: f64, #[serde(default)] end_z: f64 },
    #[serde(rename = "slab")]
    Slab { name: String, length: f64, width: f64, structural_thickness: f64, finish_thickness: f64, #[serde(default)] parent_id: Option<String>, #[serde(default)] series_id: Option<String> },
    #[serde(rename = "road")]
    /// Road (Road): when only `length` is given a straight road is generated;
    /// when the picked `alignment` ([x,y,z] N-point broken polyline, serde default empty,
    /// swept one by one per segment with length ≥ 2) is present, a segmented road mesh is generated along the broken
    /// polyline. The frontend convention: `path` is only used by the wall category (layered wall), the road uses `alignment`.
    Road {
        name: String,
        length: f64,
        width: f64,
        thickness: f64,
        /// Contract form `path { points, closed: false }` — the canonical alignment input.
        #[serde(default)]
        path: Option<PathInput>,
        /// Legacy alias for `path` (N-point broken polyline).
        #[serde(default)]
        alignment: Option<PathInput>,
        #[serde(default)]
        parent_id: Option<String>,
        #[serde(default)]
        series_id: Option<String>,
    },
    #[serde(rename = "pipe")]
    Pipe { name: String, length: f64, radius: f64, #[serde(default)] parent_id: Option<String>, #[serde(default)] series_id: Option<String> },
    #[serde(rename = "door")]
    Door { name: String, wall_id: String, opening_width: f64, opening_height: f64, position_x: f64, position_z: f64, #[serde(default)] parent_id: Option<String> },
    #[serde(rename = "window")]
    Window { name: String, wall_id: String, opening_width: f64, opening_height: f64, position_x: f64, position_z: f64, #[serde(default)] parent_id: Option<String> },
    #[serde(rename = "assembly")]
    Assembly { name: String, components: Vec<AssemblyComponentDef>, #[serde(default)] parent_id: Option<String> },
    /// Layered surface component (wall/slab/roof/floor) driven by an arbitrary
    /// layer stack, generated via the LayerExtrusion engine. This is the product
    /// path for the LayerEditor UI.
    #[serde(rename = "layered")]
    Layered {
        name: String,
        #[serde(default)]
        mode: String, // "wall" | "slab" | "roof" | "floor"
        #[serde(default)]
        length: f64,
        #[serde(default)]
        width: f64,   // slab footprint width
        #[serde(default)]
        height: f64,  // wall/roof height
        layers: Vec<LayerDefRequest>,
        /// Center-line path (wall / roof-profile): contract `{ points, closed }`
        /// or legacy bare array. Closed form for walls = region pieces.
        #[serde(default)]
        path: Option<PathInput>,
        /// Closed boundary footprint (slab/roof/floor): contract or legacy array.
        #[serde(default)]
        boundary: Option<PathInput>,
        #[serde(default)]
        parent_id: Option<String>,
        #[serde(default)]
        series_id: Option<String>,
        #[serde(default)]
        start_x: f64,
        #[serde(default)]
        start_y: f64,
        #[serde(default)]
        start_z: f64,
        #[serde(default)]
        end_x: f64,
        #[serde(default)]
        end_y: f64,
        #[serde(default)]
        end_z: f64,
    },
    /// Feature line (feature_line) / contour line (contour_line): a thin guide
    /// tube along an N-point alignment. Uses the LinearComponent engine.
    #[serde(rename = "line")]
    Line {
        name: String,
        /// Contract form `path { points, closed: false }`.
        #[serde(default)]
        path: Option<PathInput>,
        /// Legacy alias for `path`.
        #[serde(default)]
        alignment: Option<PathInput>,
        #[serde(default)]
        diameter: f64,
        #[serde(default)]
        parent_id: Option<String>,
    },
}

#[derive(Deserialize, Clone)]
struct LayerDefRequest {
    name: String,
    #[serde(default)]
    thickness: f64,
    #[serde(default)]
    material_id: Option<String>,
    #[serde(default)]
    function: String,
}

impl LayerDefRequest {
    fn to_layer_def(&self) -> omnim_model::gen_rule::layer_extrusion::LayerDef {
        omnim_model::gen_rule::layer_extrusion::LayerDef {
            name: self.name.clone(),
            thickness: self.thickness,
            material_id: self.material_id.clone(),
            function: self.function.clone(),
        }
    }
}

/// `path` accepts the contract form `{ "points": [[x,y,z]...], "closed": bool }`
/// or a legacy bare array (`path[]` / `boundary[]` / `alignment[]`).
/// Contract: DATA_MODEL §1 dual-axis model; both spellings normalize to `PathSpec`.
#[derive(Deserialize, Clone)]
#[serde(untagged)]
enum PathInput {
    Contract {
        points: Vec<[f64; 3]>,
        #[serde(default)]
        closed: bool,
    },
    Legacy(Vec<[f64; 3]>),
}

#[derive(Debug, Clone)]
struct PathSpec {
    points: Vec<[f64; 3]>,
    closed: bool,
}

impl PathInput {
    fn normalize(self, legacy_closed: bool) -> PathSpec {
        match self {
            PathInput::Contract { points, closed } => PathSpec { points, closed },
            PathInput::Legacy(points) => PathSpec { points, closed: legacy_closed },
        }
    }
}

/// Validate a normalized spec through the shared pure-geometry layer
/// (`gen_rule::path_geom`): open ≥ min points / closed polygon checks.
fn validate_spec(spec: &PathSpec, min: usize) -> Result<(), String> {
    use omnim_model::gen_rule::path_geom;
    let pts: Vec<omnim_core::Point2D> = spec
        .points
        .iter()
        .map(|c| omnim_core::Point2D { x: c[0], y: c[1] })
        .collect();
    path_geom::validate_path(&pts, spec.closed, min)
}

/// String-typed property value for metadata properties (genesis recording).
fn str_prop(v: &str) -> c::PropertyValue {
    c::PropertyValue {
        value_type: Some(c::property_value::ValueType::StringValue(v.to_string())),
    }
}

#[derive(Deserialize)]
struct AssemblyComponentDef {
    name: String,
    shape: String,
    p1: f64,
    p2: f64,
    p3: f64,
    tx: f64,
    ty: f64,
    tz: f64,
}

async fn genrule_create(
    State(state): State<SharedState>,
    Json(req): Json<GenRuleRequest>,
) -> Result<Json<omnim_model::OmniObject>, (StatusCode, String)> {
    let mut p = state.project.write().await;
    let mut exe = state.executor.write().await;

    // Helper: extract parent_id from any variant
    let parent_id: Option<&str> = match &req {
        GenRuleRequest::Wall { parent_id, .. } => parent_id.as_deref(),
        GenRuleRequest::Slab { parent_id, .. } => parent_id.as_deref(),
        GenRuleRequest::Road { parent_id, .. } => parent_id.as_deref(),
        GenRuleRequest::Pipe { parent_id, .. } => parent_id.as_deref(),
        GenRuleRequest::Door { parent_id, .. } => parent_id.as_deref(),
        GenRuleRequest::Window { parent_id, .. } => parent_id.as_deref(),
        GenRuleRequest::Assembly { parent_id, .. } => parent_id.as_deref(),
        GenRuleRequest::Layered { parent_id, .. } => parent_id.as_deref(),
        GenRuleRequest::Line { parent_id, .. } => parent_id.as_deref(),
    };

    // Helper: extract series_id from any variant (wall/slab/road/pipe/layered only)
    let series_id: Option<&str> = match &req {
        GenRuleRequest::Wall { series_id, .. } => series_id.as_deref(),
        GenRuleRequest::Slab { series_id, .. } => series_id.as_deref(),
        GenRuleRequest::Road { series_id, .. } => series_id.as_deref(),
        GenRuleRequest::Pipe { series_id, .. } => series_id.as_deref(),
        GenRuleRequest::Layered { series_id, .. } => series_id.as_deref(),
        _ => None,
    };
    // If a series is explicitly selected, use the catalog/series path
    if let Some(sid) = series_id {
        if !sid.is_empty() {
            let obj = genrule_create_catalog(
                &mut p, &req, sid, parent_id,
            )?;
            exe.record_create(&obj);
            return Ok(Json(obj));
        }
    }

    // No series selected: use the old mesh generation path (creates correct geometry)
    let obj = match &req {
        GenRuleRequest::Wall { name, length, height, core_thickness, finish_thickness, start_x, start_y, start_z, end_x, end_y, end_z, .. } => {
            let miter_ext = 3.0 * (*core_thickness + 2.0 * *finish_thickness);
            let mut mesh = omnim_model::gen_rule::layer_extrusion::build_wall(
                *length + miter_ext, *height, *core_thickness, *finish_thickness,
            );
            // Transform wall to align with start->end direction
            if *end_x != *start_x || *end_y != *start_y || *end_z != *start_z {
                let dx = *end_x - *start_x;
                let dy = *end_y - *start_y;
                let angle = dy.atan2(dx);
                let (half_cos, half_sin) = ((angle * 0.5).cos(), (angle * 0.5).sin());
                let t = omnim_core::Transform {
                    matrix: Vec::new(),
                    translation: Some(omnim_core::Point3D {
                        x: (*start_x + *end_x) / 2.0,
                        y: (*start_y + *end_y) / 2.0,
                        z: (*start_z + *end_z) / 2.0,
                    }),
                    rotation: Some(omnim_core::Rotation { x: 0.0, y: 0.0, z: half_sin, w: half_cos }),
                    scale: None,
                };
                mesh = omnim_model::geometry::transform_mesh(&mesh, &t);
            }
            let geom = omnim_model::geometry::mesh_to_geometry(&mesh);
            let id = p.create_object(omnim_core::ObjectType::Wall, name).id.clone();
            let obj = p.objects.get_mut(&id).expect("genrule object was just created");
            obj.geometry = Some(geom);
            if let Some(pid) = parent_id { if !pid.is_empty() { obj.parent_id = Some(pid.to_string()); } }
            obj.clone()
        }
        GenRuleRequest::Slab { name, length, width, structural_thickness, finish_thickness, .. } => {
            let mesh = omnim_model::gen_rule::layer_extrusion::build_slab(
                *length, *width, *structural_thickness, *finish_thickness,
            );
            let geom = omnim_model::geometry::mesh_to_geometry(&mesh);
            let id = p.create_object(omnim_core::ObjectType::Slab, name).id.clone();
            let obj = p.objects.get_mut(&id).expect("genrule object was just created");
            obj.geometry = Some(geom);
            if let Some(pid) = parent_id { if !pid.is_empty() { obj.parent_id = Some(pid.to_string()); } }
            obj.clone()
        }
        GenRuleRequest::Road { name, length, width, thickness, path, alignment, .. } => {
            // 路径契约：path { points, closed:false } 为规范输入，alignment[] 为遗留别名；
            // 两者同时出现视为歧义。授权矩阵：road × linear。
            let road_spec: Option<PathSpec> = match (path, alignment) {
                (Some(_), Some(_)) => {
                    return Err((StatusCode::BAD_REQUEST,
                        "provide either `path` or legacy `alignment`, not both".into()));
                }
                (Some(p), None) => Some(p.clone().normalize(false)),
                (None, Some(a)) => Some(a.clone().normalize(false)),
                (None, None) => None,
            };
            if let Some(ref spec) = road_spec {
                if spec.closed {
                    return Err((StatusCode::BAD_REQUEST, "road path must be open (a road is a linear component, not a region)".into()));
                }
                omnim_model::gen_rule::matrix::authorize("road", "linear")
                    .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
                validate_spec(spec, 2).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
            }
            // B3：拾取折线（≥ 2 点）→ 逐段道路网格（绕段角旋转、平移到段起点）；
            // 否则退化为旧的单段直路
            let mesh = match &road_spec {
                Some(spec) if spec.points.len() >= 2 => {
                    omnim_model::gen_rule::linear_component::build_road_alignment(
                        &spec.points, *width, *thickness, 30,
                    )
                }
                _ => omnim_model::gen_rule::linear_component::build_road(
                    *length, *width, *thickness * 0.7, *thickness * 0.3, 30,
                ),
            };
            let geom = omnim_model::geometry::mesh_to_geometry(&mesh);
            let id = p.create_object(omnim_core::ObjectType::Road, name).id.clone();
            let obj = p.objects.get_mut(&id).expect("genrule object was just created");
            obj.geometry = Some(geom);
            if road_spec.is_some() {
                obj.set_property("genesis_class", str_prop("Linear"));
                obj.set_property("genesis_form", str_prop("linear"));
            }
            if let Some(pid) = parent_id { if !pid.is_empty() { obj.parent_id = Some(pid.to_string()); } }
            obj.clone()
        }
        GenRuleRequest::Pipe { name, length, radius, .. } => {
            let mesh = omnim_model::gen_rule::linear_component::build_pipe(*length, *radius, 20);
            let geom = omnim_model::geometry::mesh_to_geometry(&mesh);
            let id = p.create_object(omnim_core::ObjectType::PipePlant, name).id.clone();
            let obj = p.objects.get_mut(&id).expect("genrule object was just created");
            obj.geometry = Some(geom);
            if let Some(pid) = parent_id { if !pid.is_empty() { obj.parent_id = Some(pid.to_string()); } }
            obj.clone()
        }
        GenRuleRequest::Door { name, wall_id, opening_width, opening_height, position_x, position_z, .. } => {
            let wall = p.objects.get(wall_id).ok_or((
                StatusCode::NOT_FOUND,
                format!("Wall {} not found", wall_id),
            ))?;
            let bb = wall.geometry.as_ref()
                .and_then(|g| g.bbox.as_ref())
                .map(|b| {
                    let mn = b.min.as_ref().expect("bbox min must exist");
                    let mx = b.max.as_ref().expect("bbox max must exist");
                    (mx.x - mn.x, mx.z - mn.z, (mx.y - mn.y).abs())
                })
                .unwrap_or((5.0, 3.0, 0.3));
            let _ = wall;

            let mesh = omnim_model::gen_rule::boolean_subtract::build_door_opening(
                bb.0, bb.1, bb.2, *opening_width, *opening_height, *position_x, *position_z,
            );
            let geom = omnim_model::geometry::mesh_to_geometry(&mesh);
            let id = p.create_object(omnim_core::ObjectType::Door, name).id.clone();
            let obj = p.objects.get_mut(&id).expect("genrule object was just created");
            obj.geometry = Some(geom);
            obj.parent_id = Some(wall_id.clone());
            obj.clone()
        }
        GenRuleRequest::Window { name, wall_id, opening_width, opening_height, position_x, position_z, .. } => {
            let wall = p.objects.get(wall_id).ok_or((
                StatusCode::NOT_FOUND,
                format!("Wall {} not found", wall_id),
            ))?;
            let bb = wall.geometry.as_ref()
                .and_then(|g| g.bbox.as_ref())
                .map(|b| {
                    let mn = b.min.as_ref().expect("bbox min must exist");
                    let mx = b.max.as_ref().expect("bbox max must exist");
                    (mx.x - mn.x, mx.z - mn.z, (mx.y - mn.y).abs())
                })
                .unwrap_or((5.0, 3.0, 0.3));
            let _ = wall;

            let mesh = omnim_model::gen_rule::boolean_subtract::build_door_opening(
                bb.0, bb.1, bb.2, *opening_width, *opening_height, *position_x, *position_z,
            );
            let geom = omnim_model::geometry::mesh_to_geometry(&mesh);
            let id = p.create_object(omnim_core::ObjectType::Window, name).id.clone();
            let obj = p.objects.get_mut(&id).expect("genrule object was just created");
            obj.geometry = Some(geom);
            obj.parent_id = Some(wall_id.clone());
            obj.clone()
        }
        GenRuleRequest::Layered { name, mode, length, width, height, layers, path, boundary, start_x, start_y, start_z, end_x, end_y, end_z, .. } => {
            use omnim_model::gen_rule::layer_extrusion::{build_layered_wall, build_layered_slab, build_layered_roof_profile, LayerDef as MLayerDef};
            if layers.is_empty() {
                return Err((StatusCode::BAD_REQUEST, "layered genrule requires at least one layer".into()));
            }
            let layer_defs: Vec<MLayerDef> = layers.iter().map(|l| l.to_layer_def()).collect();
            let object_type = match mode.as_str() {
                "slab" => omnim_core::ObjectType::Slab,
                "roof" => omnim_core::ObjectType::Roof,
                "floor" => omnim_core::ObjectType::Floor,
                _ => omnim_core::ObjectType::Wall,
            };
            let kind = match mode.as_str() {
                "slab" => "slab",
                "roof" => "roof",
                "floor" => "floor",
                _ => "wall",
            };

            // ── Path contract + (objectType, form) authorization (DATA_MODEL §1) ──
            let open = path.clone().map(|p| p.normalize(false));
            let closed = boundary.clone().map(|b| b.normalize(true));
            if open.is_some() && closed.is_some() {
                return Err((StatusCode::BAD_REQUEST,
                    "provide either `path` or `boundary`, not both".into()));
            }
            let is_roof_profile = open.as_ref().is_some_and(|spec| {
                spec.points.len() == 2
                    && (spec.points[0][0], spec.points[0][1]) != (spec.points[1][0], spec.points[1][1])
            });
            let form: Option<&'static str> = if open.is_some() {
                if kind == "roof" {
                    if !is_roof_profile {
                        return Err((StatusCode::BAD_REQUEST,
                            "roof 'panel' form requires exactly 2 distinct points; use `boundary` for a footprint roof".into()));
                    }
                    Some("panel")
                } else {
                    Some("panel")
                }
            } else if closed.is_some() {
                Some("region")
            } else {
                None // parameter quick-form (rect), no path input — outside the matrix
            };
            if let Some(f) = form {
                omnim_model::gen_rule::matrix::authorize(kind, f)
                    .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
            }

            // ── Wave 3 geometry convergence ──
            // Path-driven layered objects instantiate through the catalog
            // feature tree (`PathSweep` step → gen_rule::path_sweep kernel),
            // producing the **domain object type** (Wall/Slab/Roof/Floor) with
            // `catalog_ref`/`param_values` — the same object a catalog
            // placement produces, so all three entry points converge.
            let def_id = match kind {
                "wall" => "Wall-Path",
                "slab" => "Slab-Path",
                "floor" => "Floor-Path",
                _ => "Roof-Path",
            };
            // Rect parameter quick-forms (no picked path) keep the legacy
            // in-handler builders — documented transitional (CODE_DEVIATIONS §6.3).
            let has_path_input = open.is_some() || closed.is_some();
            if !has_path_input {
                let mut mesh = if kind == "wall" {
                    build_layered_wall(*length, *height, &layer_defs)
                } else {
                    build_layered_slab(*length, *width, &layer_defs)
                };
                if *end_x != *start_x || *end_y != *start_y || *end_z != *start_z {
                    let dx = *end_x - *start_x;
                    let dy = *end_y - *start_y;
                    let angle = dy.atan2(dx);
                    let (half_cos, half_sin) = ((angle * 0.5).cos(), (angle * 0.5).sin());
                    let t = omnim_core::Transform {
                        matrix: Vec::new(),
                        translation: Some(omnim_core::Point3D {
                            x: (*start_x + *end_x) / 2.0,
                            y: (*start_y + *end_y) / 2.0,
                            z: (*start_z + *end_z) / 2.0,
                        }),
                        rotation: Some(omnim_core::Rotation { x: 0.0, y: 0.0, z: half_sin, w: half_cos }),
                        scale: None,
                    };
                    mesh = omnim_model::geometry::transform_mesh(&mesh, &t);
                }
                let geom = omnim_model::geometry::mesh_to_geometry(&mesh);
                let id = p.create_object(object_type, name).id.clone();
                let obj = p.objects.get_mut(&id).expect("genrule object was just created");
                obj.geometry = Some(geom);
                if let Some(pid) = parent_id { if !pid.is_empty() { obj.parent_id = Some(pid.to_string()); } }
                return Ok(Json(obj.clone()));
            }

            // Roof profile (2-point) has no PathSweep template yet — keep the
            // legacy profile builder (transitional, CODE_DEVIATIONS §6.3).
            if kind == "roof" && is_roof_profile {
                let spec = open.as_ref().expect("roof profile implies open path");
                let w = if *width <= 0.0 { 1.0 } else { *width };
                let mesh = build_layered_roof_profile(
                    [spec.points[0][0], spec.points[0][1]],
                    [spec.points[1][0], spec.points[1][1]], w, &layer_defs, spec.points[0][2],
                );
                let geom = omnim_model::geometry::mesh_to_geometry(&mesh);
                let id = p.create_object(object_type, name).id.clone();
                let obj = p.objects.get_mut(&id).expect("genrule object was just created");
                obj.geometry = Some(geom);
                if let Some(f) = form {
                    let cf = omnim_model::gen_rule::matrix::authorize(kind, f).expect("authorized above");
                    obj.set_property("genesis_class", str_prop(cf.class.as_str()));
                    obj.set_property("genesis_form", str_prop(cf.form));
                }
                if let Some(pid) = parent_id { if !pid.is_empty() { obj.parent_id = Some(pid.to_string()); } }
                return Ok(Json(obj.clone()));
            }

            // Path points (world metres, z preserved — B1 convention).
            let spec = open.clone().or_else(|| closed.clone())
                .expect("has_path_input guarantees one of path/boundary");
            let param_values: std::collections::HashMap<String, f64> = [
                ("length".to_string(), *length * 1000.0),
                ("width".to_string(), *width * 1000.0),
                ("height".to_string(), *height * 1000.0),
            ]
            .into_iter()
            .filter(|(_, v)| *v > 0.0)
            .collect();
            let height_m = if kind == "wall" { *height } else { 0.0 };
            let component_id = p.instantiate_domain_object(
                object_type, def_id, name, &param_values,
                spec.points.clone(), spec.closed, layer_defs.clone(), height_m, None,
            ).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
            let obj = p.objects.get_mut(&component_id)
                .expect("domain object was just created");
            if let Some(f) = form {
                let cf = omnim_model::gen_rule::matrix::authorize(kind, f).expect("authorized above");
                obj.set_property("genesis_class", str_prop(cf.class.as_str()));
                obj.set_property("genesis_form", str_prop(cf.form));
            }
            if let Some(pid) = parent_id { if !pid.is_empty() { obj.parent_id = Some(pid.to_string()); } }
            obj.clone()
        }
        GenRuleRequest::Line { name, path, alignment, diameter, .. } => {
            use omnim_core::Point3D;
            // 路径契约同 Road：path 为规范输入，alignment[] 为遗留别名。授权矩阵：line × point。
            let line_spec: PathSpec = match (path, alignment) {
                (Some(_), Some(_)) => {
                    return Err((StatusCode::BAD_REQUEST,
                        "provide either `path` or legacy `alignment`, not both".into()));
                }
                (Some(p), None) => p.clone().normalize(false),
                (None, Some(a)) => a.clone().normalize(false),
                (None, None) => {
                    return Err((StatusCode::BAD_REQUEST, "line genrule requires at least 2 points".into()));
                }
            };
            if line_spec.closed {
                return Err((StatusCode::BAD_REQUEST, "line path must be open".into()));
            }
            omnim_model::gen_rule::matrix::authorize("line", "point")
                .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
            validate_spec(&line_spec, 2).map_err(|e| (StatusCode::BAD_REQUEST, e))?;
            let pts: Vec<Point3D> = line_spec.points.iter().map(|c| Point3D { x: c[0], y: c[1], z: c[2] }).collect();
            let mesh = omnim_model::gen_rule::linear_component::build_guide_line(&pts, if *diameter > 0.0 { *diameter } else { 0.05 }, 16);
            let geom = omnim_model::geometry::mesh_to_geometry(&mesh);
            // FeatureLine/ContourLine map to the proto ObjectType when available.
            let object_type = omnim_core::ObjectType::FeatureLine;
            let id = p.create_object(object_type, name).id.clone();
            let obj = p.objects.get_mut(&id).expect("genrule object was just created");
            obj.geometry = Some(geom);
            obj.set_property("genesis_class", str_prop("Point"));
            obj.set_property("genesis_form", str_prop("point"));
            if let Some(pid) = parent_id { if !pid.is_empty() { obj.parent_id = Some(pid.to_string()); } }
            obj.clone()
        }
        GenRuleRequest::Assembly { name, components, .. } => {
            use omnim_core::parametric_operation::Operation;
            use omnim_model::gen_rule::assembly::{AssemblyComponent as Ac, build_assembly};
            use omnim_core::{ParametricGeometry, ParametricOperation, Transform, Point3D};
            let assy_components: Vec<Ac> = components.iter().map(|c| {
                let op = match c.shape.as_str() {
                    "cylinder" => Operation::Cylinder(omnim_core::Cylinder { radius: c.p1, height: c.p2, segments: c.p3.max(3.0) as i32 }),
                    "sphere" => Operation::Sphere(omnim_core::Sphere { radius: c.p1, segments: c.p2.max(3.0) as i32 }),
                    "cone" => Operation::Cone(omnim_core::Cone { radius_bottom: c.p1, radius_top: c.p2, height: c.p3, segments: 16 }),
                    _ => Operation::Box(omnim_core::Box { length: c.p1, width: c.p2, height: c.p3 }),
                };
                Ac {
                    name: c.name.clone(),
                    geometry: ParametricGeometry {
                        operations: vec![ParametricOperation { operation: Some(op) }],
                    },
                    transform: Transform {
                        translation: Some(Point3D { x: c.tx, y: c.ty, z: c.tz }),
                        rotation: None,
                        scale: None,
                        matrix: Vec::new(),
                    },
                }
            }).collect();

            let mesh = build_assembly(assy_components);
            let geom = omnim_model::geometry::mesh_to_geometry(&mesh);
            let id = p.create_object(omnim_core::ObjectType::Assembly, name).id.clone();
            let obj = p.objects.get_mut(&id).expect("genrule object was just created");
            obj.geometry = Some(geom);
            if let Some(pid) = parent_id { if !pid.is_empty() { obj.parent_id = Some(pid.to_string()); } }
            obj.clone()
        }
    };

    // Ensure children index is in sync after setting parent_ids
    p.objects.rebuild_children();

    // Record undo command
    exe.record_create(&obj);

    Ok(Json(obj))
}

/// Helper: create an object via the catalog/series system.
/// Only used when the user explicitly selects a series.
///
/// Creates a plain object (not a Component) with correct geometry from
/// the old mesh generation code, and sets catalog_ref for traceability.
fn genrule_create_catalog(
    p: &mut omnim_model::Project,
    req: &GenRuleRequest,
    series_id: &str,
    parent_id: Option<&str>,
) -> Result<omnim_model::OmniObject, (StatusCode, String)> {
    use omnim_model::catalog::CatalogRef;
    use omnim_model::series_store::SelectionEngine;

    let (name, object_type) = match req {
        GenRuleRequest::Wall { name, .. } => (name.as_str(), omnim_core::ObjectType::Wall),
        GenRuleRequest::Slab { name, .. } => (name.as_str(), omnim_core::ObjectType::Slab),
        GenRuleRequest::Road { name, .. } => (name.as_str(), omnim_core::ObjectType::Road),
        GenRuleRequest::Pipe { name, .. } => (name.as_str(), omnim_core::ObjectType::PipePlant),
        _ => return Err((StatusCode::BAD_REQUEST, "series not supported for this type".into())),
    };

    let answer = omnim_model::Project::object_type_to_series_answer_pub(object_type)
        .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("no series answer for type {:?}", object_type)))?;

    // Domain check
    let (_series_domain, _series_name_clone, series_id_clone) = {
        let s = p.series.find_by_name(series_id)
            .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("series '{}' not found", series_id)))?;
        let expected_domain = object_type_to_domain(object_type);
        if s.domain != expected_domain {
            return Err((
                StatusCode::BAD_REQUEST,
                format!(
                    "series '{}' has domain '{}', but '{}' requires domain '{}'. \
                     Select a series from the '{}' domain (e.g., Concrete-C30 for building walls).",
                    s.name, s.domain,
                    object_type_name(object_type), expected_domain,
                    expected_domain,
                ),
            ));
        }
        (s.domain.clone(), s.name.clone(), s.id.clone())
    };

    // Select the component def from the series
    let comp_def_id = {
        let s = p.series.find_by_name(series_id)
            .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("series '{}' not found", series_id)))?;
        let result = SelectionEngine::select(s, &[answer])
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("series selection failed: {}", e)))?;
        let library = LIBRARY.read().unwrap();
        let library = library.as_ref().unwrap();
        let comp_def = library.catalog.get(&result.component_def_id)
            .or_else(|| library.catalog.find_by_name(&result.component_def_id))
            .ok_or_else(|| (StatusCode::BAD_REQUEST, format!("component def '{}' not found", result.component_def_id)))?;
        (comp_def.id.clone(), result.selector_path)
    };

    // Create a plain object (not a Component) with correct geometry
    let mesh = match req {
        GenRuleRequest::Wall { length, height, core_thickness, finish_thickness, start_x, start_y, start_z, end_x, end_y, end_z, .. } => {
            let miter_ext = 3.0 * (*core_thickness + 2.0 * *finish_thickness);
            let mut m = omnim_model::gen_rule::layer_extrusion::build_wall(
                *length + miter_ext, *height, *core_thickness, *finish_thickness,
            );
            // Transform wall to align with start->end direction
            if *end_x != *start_x || *end_y != *start_y || *end_z != *start_z {
                let dx = *end_x - *start_x;
                let dy = *end_y - *start_y;
                let angle = dy.atan2(dx);
                let (half_cos, half_sin) = ((angle * 0.5).cos(), (angle * 0.5).sin());
                let t = omnim_core::Transform {
                    matrix: Vec::new(),
                    translation: Some(omnim_core::Point3D {
                        x: (*start_x + *end_x) / 2.0,
                        y: (*start_y + *end_y) / 2.0,
                        z: (*start_z + *end_z) / 2.0,
                    }),
                    rotation: Some(omnim_core::Rotation { x: 0.0, y: 0.0, z: half_sin, w: half_cos }),
                    scale: None,
                };
                m = omnim_model::geometry::transform_mesh(&m, &t);
            }
            m
        }
        GenRuleRequest::Slab { length, width, structural_thickness, finish_thickness, .. } => {
            omnim_model::gen_rule::layer_extrusion::build_slab(
                *length, *width, *structural_thickness, *finish_thickness,
            )
        }
        GenRuleRequest::Road { length, width, thickness, .. } => {
            omnim_model::gen_rule::linear_component::build_road(
                *length, *width, *thickness * 0.7, *thickness * 0.3, 30,
            )
        }
        GenRuleRequest::Pipe { length, radius, .. } => {
            omnim_model::gen_rule::linear_component::build_pipe(*length, *radius, 20)
        }
        _ => unreachable!(),
    };
    let geom = omnim_model::geometry::mesh_to_geometry(&mesh);

    let id = p.create_object(object_type, name).id.clone();
    {
        let obj = p.objects.get_mut(&id).expect("object was just created");
        obj.geometry = Some(geom);
        obj.catalog_ref = Some(CatalogRef {
            component_def_id: comp_def_id.0,
            series_id: series_id_clone,
            selector_path: comp_def_id.1,
            param_values: std::collections::HashMap::new(),
        });

        if let Some(pid) = parent_id {
            if !pid.is_empty() {
                obj.parent_id = Some(pid.to_string());
            }
        }
    }

    p.objects.rebuild_children();

    Ok(p.objects.get(&id).unwrap().clone())
}

/// Map an ObjectType to its expected domain string.
fn object_type_to_domain(ot: omnim_core::ObjectType) -> &'static str {
    use omnim_core::ObjectType::*;
    match ot {
        Wall | Slab | Floor | Door | Window | Beam | Column | Stair | Roof
        | Building | BuildingStorey | Space | Site => "building",
        Road | Bridge | Tunnel | Dam | Culvert | RetainingWall | PipeCivil
        | Pier | Manhole | Abutment => "civil",
        PipePlant | Duct | CableTray | AirTerminal | Tank | Pump | Valve
        | HeatExchanger | Reactor | StructuralSteel | Fitting | Luminaire
        | Ahu | Chiller | Transformer => "piping",
        FeatureLine | ContourLine => "general",
        _ => "general",
    }
}

/// Map an ObjectType to a human-readable name.
fn object_type_name(ot: omnim_core::ObjectType) -> &'static str {
    use omnim_core::ObjectType::*;
    match ot {
        Wall => "wall",
        Slab => "slab",
        Floor => "floor",
        Door => "door",
        Window => "window",
        Beam => "beam",
        Column => "column",
        Stair => "stair",
        Roof => "roof",
        Building => "building",
        BuildingStorey => "storey",
        Space => "space",
        Site => "site",
        Road => "road",
        Bridge => "bridge",
        Tunnel => "tunnel",
        Dam => "dam",
        Culvert => "culvert",
        RetainingWall => "retaining wall",
        PipeCivil => "civil pipe",
        Pier => "pier",
        Manhole => "manhole",
        Abutment => "abutment",
        PipePlant => "pipe",
        Duct => "duct",
        CableTray => "cable tray",
        AirTerminal => "air terminal",
        Tank => "tank",
        Pump => "pump",
        Valve => "valve",
        HeatExchanger => "heat exchanger",
        Reactor => "reactor",
        StructuralSteel => "structural steel",
        Fitting => "fitting",
        Luminaire => "luminaire",
        Ahu => "AHU",
        Chiller => "chiller",
        Transformer => "transformer",
        FeatureLine => "feature line",
        ContourLine => "contour line",
        _ => "unknown",
    }
}

// ============================================================
// Sketch endpoints
// ============================================================

#[derive(Serialize)]
struct SketchResponse {
    id: String,
    name: String,
    origin: Point3DJson,
    normal: Vector3DJson,
    entity_count: usize,
    constraint_count: usize,
    solved: bool,
    component_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Point3DJson {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Serialize, Deserialize)]
struct Vector3DJson {
    x: f64,
    y: f64,
    z: f64,
}

#[derive(Deserialize)]
struct CreateSketchRequest {
    name: String,
    origin: Option<Point3DJson>,
    normal: Option<Vector3DJson>,
    component_id: Option<String>,
}

#[derive(Deserialize)]
struct AddEntityRequest {
    entity_type: String,
    points: Vec<Point3DJson>,
    radius: Option<f64>,
    sides: Option<i32>,
    #[serde(default)]
    construction: bool,
}

#[derive(Deserialize)]
struct UpdateEntityRequest {
    points: Option<Vec<Point3DJson>>,
    radius: Option<f64>,
    sides: Option<i32>,
    construction: Option<bool>,
}

#[derive(Deserialize)]
struct AddConstraintRequest {
    constraint_type: String,
    entity_refs: Vec<(usize, usize)>,
    value: Option<f64>,
}

fn sketch_to_response(s: &omnim_sketch::Sketch) -> SketchResponse {
    SketchResponse {
        id: s.id.clone(),
        name: s.name.clone(),
        origin: Point3DJson { x: s.origin.x, y: s.origin.y, z: s.origin.z },
        normal: Vector3DJson { x: s.normal.x, y: s.normal.y, z: s.normal.z },
        entity_count: s.entities.len(),
        constraint_count: s.constraints.len(),
        solved: s.solved,
        component_id: s.component_id.clone(),
    }
}

async fn list_sketches(
    State(state): State<SharedState>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Vec<SketchResponse>> {
    let p = state.project.read().await;
    if let Some(comp_id) = params.get("component_id") {
        Json(p.sketches.find_by_component(comp_id).iter().map(|s| sketch_to_response(s)).collect())
    } else {
        Json(p.sketches.list().iter().map(|s| sketch_to_response(s)).collect())
    }
}

async fn create_sketch(
    State(state): State<SharedState>,
    Json(req): Json<CreateSketchRequest>,
) -> Result<Json<SketchResponse>, ApiError> {
    let origin = req.origin.map(|o| c::Point3D { x: o.x, y: o.y, z: o.z })
        .unwrap_or(c::Point3D { x: 0.0, y: 0.0, z: 0.0 });
    let normal = req.normal.map(|n| c::Vector3D { x: n.x, y: n.y, z: n.z })
        .unwrap_or(c::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
    let component_id = req.component_id.clone().unwrap_or_default();
    if !component_id.is_empty() {
        let bad = state.project.read().await.objects.get(&component_id)
            .map(|o| o.object_type != c::ObjectType::Component as i32).unwrap_or(true);
        if bad {
            return Err(ApiError::bad_request(format!("Component '{}' not found", component_id)));
        }
    }
    let op = c::operation::Op::CreateSketch(c::CreateSketchOperation {
        name: req.name,
        origin: Some(origin),
        normal: Some(normal),
        component_id,
    });
    let result = mutation::apply(&state, c::Action::CreateSketch, vec![mutation::op(op)]).await?;
    let result = mutation::require_success(result)?;
    let sketch_id = result.created_ids.first().cloned()
        .ok_or_else(|| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "sketch creation returned no id"))?;
    let p = state.project.read().await;
    let sketch = p.sketches.get(&sketch_id)
        .ok_or_else(|| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "created sketch vanished"))?;
    Ok(Json(sketch_to_response(sketch)))
}

async fn get_sketch(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<omnim_sketch::Sketch>, (StatusCode, String)> {
    let p = state.project.read().await;
    p.sketches.get(&id)
        .cloned()
        .map(Json)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Sketch {} not found", id)))
}

async fn delete_sketch(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    if state.project.read().await.sketches.get(&id).is_none() {
        return Err(ApiError::not_found(format!("Sketch {} not found", id)));
    }
    let op = c::operation::Op::DeleteSketch(c::DeleteSketchOperation { sketch_id: id });
    let result = mutation::apply(&state, c::Action::DeleteSketch, vec![mutation::op(op)]).await?;
    mutation::require_success(result)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct UpdateSketchRequest {
    origin: Option<Point3DJson>,
    name: Option<String>,
}

async fn update_sketch(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateSketchRequest>,
) -> Result<Json<omnim_sketch::Sketch>, ApiError> {
    if state.project.read().await.sketches.get(&id).is_none() {
        return Err(ApiError::not_found(format!("Sketch {} not found", id)));
    }
    let op = c::operation::Op::UpdateSketch(c::UpdateSketchOperation {
        sketch_id: id.clone(),
        origin: req.origin.map(|o| c::Point3D { x: o.x, y: o.y, z: o.z }),
        name: req.name,
    });
    let result = mutation::apply(&state, c::Action::UpdateSketch, vec![mutation::op(op)]).await?;
    mutation::require_success(result)?;
    let p = state.project.read().await;
    let sketch = p.sketches.get(&id)
        .ok_or_else(|| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "sketch vanished"))?;
    Ok(Json(sketch.clone()))
}

async fn add_sketch_entity(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<AddEntityRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if state.project.read().await.sketches.get(&id).is_none() {
        return Err(ApiError::not_found(format!("Sketch {} not found", id)));
    }

    let entity_type = match req.entity_type.as_str() {
        "point" => c::SketchEntityType::SketchPoint,
        "line" => c::SketchEntityType::SketchLine,
        "circle" => c::SketchEntityType::SketchCircle,
        "arc" => c::SketchEntityType::SketchArc,
        "rectangle" => c::SketchEntityType::SketchRectangle,
        "polygon" => c::SketchEntityType::SketchPolygon,
        "ellipse" => c::SketchEntityType::SketchEllipse,
        "spline" => c::SketchEntityType::SketchSpline,
        "polyline" => c::SketchEntityType::SketchPolyline,
        _ => return Err(ApiError::bad_request(format!("Unknown entity type: {}", req.entity_type))),
    };

    let op = c::operation::Op::AddSketchEntity(c::AddSketchEntityOperation {
        sketch_id: id.clone(),
        entity: Some(c::SketchEntityData {
            id: String::new(),
            entity_type: entity_type as i32,
            points: req.points.iter().map(|p| c::Point3D { x: p.x, y: p.y, z: p.z }).collect(),
            radius: req.radius.unwrap_or(0.0),
            sides: req.sides.unwrap_or(0),
            construction: req.construction,
        }),
    });
    let result = mutation::apply(&state, c::Action::AddSketchEntity, vec![mutation::op(op)]).await?;
    mutation::require_success(result)?;
    let p = state.project.read().await;
    let idx = p.sketches.get(&id).and_then(|s| s.entities.len().checked_sub(1))
        .ok_or_else(|| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "sketch entity creation returned no index"))?;
    Ok(Json(serde_json::json!({ "index": idx })))
}

async fn add_sketch_constraint(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<AddConstraintRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    if state.project.read().await.sketches.get(&id).is_none() {
        return Err(ApiError::not_found(format!("Sketch {} not found", id)));
    }

    let constraint_type = match req.constraint_type.as_str() {
        "horizontal" => c::SketchConstraintType::SketchHorizontal,
        "vertical" => c::SketchConstraintType::SketchVertical,
        "coincident" => c::SketchConstraintType::SketchCoincident,
        "parallel" => c::SketchConstraintType::SketchParallel,
        "perpendicular" => c::SketchConstraintType::SketchPerpendicular,
        "length" => c::SketchConstraintType::SketchLength,
        "distance" => c::SketchConstraintType::SketchDistance,
        "angle" => c::SketchConstraintType::SketchAngle,
        "radius" => c::SketchConstraintType::SketchRadius,
        "diameter" => c::SketchConstraintType::SketchDiameter,
        "tangent" => c::SketchConstraintType::SketchTangent,
        "symmetric" => c::SketchConstraintType::SketchSymmetric,
        "fixed" => c::SketchConstraintType::SketchFixed,
        "at_midpoint" => c::SketchConstraintType::SketchAtMidpoint,
        "equal_length" => c::SketchConstraintType::SketchEqualLength,
        "equal_radius" => c::SketchConstraintType::SketchEqualRadius,
        "pt_on_line" => c::SketchConstraintType::SketchPtOnLine,
        "pt_on_circle" => c::SketchConstraintType::SketchPtOnCircle,
        "same_orientation" => c::SketchConstraintType::SketchSameOrientation,
        _ => return Err(ApiError::bad_request(format!("Unknown constraint type: {}", req.constraint_type))),
    };

    // SketchPointRef indices are 1-based on the wire; the REST contract is 0-based.
    let entity_refs: Vec<c::SketchPointRef> = req.entity_refs.iter()
        .map(|(ei, pi)| c::SketchPointRef {
            entity_index: (ei + 1) as i32,
            point_index: (pi + 1) as i32,
        })
        .collect();

    let op = c::operation::Op::AddSketchConstraint(c::AddSketchConstraintOperation {
        sketch_id: id.clone(),
        constraint: Some(c::SketchConstraintData {
            id: String::new(),
            constraint_type: constraint_type as i32,
            entity_refs,
            value: req.value.unwrap_or(0.0),
        }),
    });
    let result = mutation::apply(&state, c::Action::AddSketchConstraint, vec![mutation::op(op)]).await?;
    mutation::require_success(result)?;
    let p = state.project.read().await;
    let idx = p.sketches.get(&id).and_then(|s| s.constraints.len().checked_sub(1))
        .ok_or_else(|| ApiError::new(StatusCode::INTERNAL_SERVER_ERROR, "sketch constraint creation returned no index"))?;
    Ok(Json(serde_json::json!({ "index": idx })))
}

#[derive(Serialize)]
struct SolveResponse {
    success: bool,
    dof: i32,
    message: String,
}

async fn solve_sketch(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<SolveResponse>, ApiError> {
    // Solve runs through the Executor (undoable, matches post-image), but the
    // response body needs the SolveResult details that the command protocol
    // does not carry — derive them deterministically from a probe copy.
    let probe = {
        let p = state.project.read().await;
        let sketch = p.sketches.get(&id)
            .ok_or_else(|| ApiError::not_found(format!("Sketch {} not found", id)))?;
        sketch.clone()
    };
    let mut probe = probe;
    let result = omnim_sketch::solver::solve(&mut probe);
    let response = SolveResponse {
        success: result.success,
        dof: result.dof,
        message: result.message,
    };

    let op = c::operation::Op::SolveSketch(c::SolveSketchOperation { sketch_id: id });
    let result = mutation::apply(&state, c::Action::SolveSketch, vec![mutation::op(op)]).await?;
    mutation::require_success(result)?;
    Ok(Json(response))
}

async fn delete_sketch_entity(
    State(state): State<SharedState>,
    Path((id, idx)): Path<(String, usize)>,
) -> Result<StatusCode, ApiError> {
    {
        let p = state.project.read().await;
        let sketch = p.sketches.get(&id)
            .ok_or_else(|| ApiError::not_found(format!("Sketch {} not found", id)))?;
        if idx >= sketch.entities.len() {
            return Err(ApiError::not_found(format!("Entity index {} out of range", idx)));
        }
    }
    let op = c::operation::Op::DeleteSketchEntity(c::DeleteSketchEntityOperation {
        sketch_id: id,
        entity_index: (idx + 1) as i32,
    });
    let result = mutation::apply(&state, c::Action::DeleteSketchEntity, vec![mutation::op(op)]).await?;
    mutation::require_success(result)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn update_sketch_entity(
    State(state): State<SharedState>,
    Path((id, idx)): Path<(String, usize)>,
    Json(req): Json<UpdateEntityRequest>,
) -> Result<StatusCode, ApiError> {
    let p = state.project.read().await;
    if p.sketches.get(&id).is_none() {
        return Err(ApiError::not_found(format!("Sketch {} not found", id)));
    }
    if p.sketches.get(&id).map(|s| idx >= s.entities.len()).unwrap_or(true) {
        return Err(ApiError::not_found(format!("Entity index {} out of range", idx)));
    }
    drop(p);
    let op = c::operation::Op::UpdateSketchEntity(c::UpdateSketchEntityOperation {
        sketch_id: id,
        entity_index: (idx + 1) as i32,
        points: req.points.map(|points| c::PointList {
            values: points.iter().map(|p| c::Point3D { x: p.x, y: p.y, z: p.z }).collect(),
        }),
        radius: req.radius,
        sides: req.sides,
        construction: req.construction,
    });
    let result = mutation::apply(&state, c::Action::UpdateSketchEntity, vec![mutation::op(op)]).await?;
    mutation::require_success(result)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_sketch_constraint(
    State(state): State<SharedState>,
    Path((id, idx)): Path<(String, usize)>,
) -> Result<StatusCode, ApiError> {
    {
        let p = state.project.read().await;
        let sketch = p.sketches.get(&id)
            .ok_or_else(|| ApiError::not_found(format!("Sketch {} not found", id)))?;
        if idx >= sketch.constraints.len() {
            return Err(ApiError::not_found(format!("Constraint index {} out of range", idx)));
        }
    }
    let op = c::operation::Op::RemoveSketchConstraint(c::RemoveSketchConstraintOperation {
        sketch_id: id,
        constraint_index: (idx + 1) as i32,
    });
    let result = mutation::apply(&state, c::Action::RemoveSketchConstraint, vec![mutation::op(op)]).await?;
    mutation::require_success(result)?;
    Ok(StatusCode::NO_CONTENT)
}

// ============================================================
// Undo / Redo endpoints
// ============================================================

#[derive(Serialize)]
struct UndoRedoResponse {
    success: bool,
    message: String,
}

async fn undo(
    State(state): State<SharedState>,
) -> Result<Json<UndoRedoResponse>, (StatusCode, String)> {
    let mut p = state.project.write().await;
    let mut exe = state.executor.write().await;
    match exe.undo(&mut p) {
        Ok(msg) => {
            mutation::persist(&state.save_path, &p);
            Ok(Json(UndoRedoResponse { success: true, message: msg }))
        }
        Err(e) => Ok(Json(UndoRedoResponse { success: false, message: e })),
    }
}

async fn redo(
    State(state): State<SharedState>,
) -> Result<Json<UndoRedoResponse>, (StatusCode, String)> {
    let mut p = state.project.write().await;
    let mut exe = state.executor.write().await;
    match exe.redo(&mut p) {
        Ok(msg) => {
            mutation::persist(&state.save_path, &p);
            Ok(Json(UndoRedoResponse { success: true, message: msg }))
        }
        Err(e) => Ok(Json(UndoRedoResponse { success: false, message: e })),
    }
}

#[derive(Serialize)]
struct UndoStatusResponse {
    can_undo: bool,
    can_redo: bool,
    undo_depth: usize,
}

async fn undo_status(
    State(state): State<SharedState>,
) -> Json<UndoStatusResponse> {
    let exe = state.executor.read().await;
    Json(UndoStatusResponse {
        can_undo: exe.can_undo(),
        can_redo: exe.can_redo(),
        undo_depth: exe.undo_depth(),
    })
}

#[derive(Serialize)]
struct MeshResponse {
    objects: Vec<MeshObject>,
}

#[derive(Serialize)]
struct MeshObject {
    id: String,
    name: String,
    object_type: String,
    vertex_count: u32,
    triangle_count: u32,
    positions: Vec<f32>,
    normals: Vec<f32>,
    bbox_min: [f32; 3],
    bbox_max: [f32; 3],
}

async fn get_mesh(State(state): State<SharedState>) -> Json<MeshResponse> {
    let p = state.project.read().await;
    let mut objects = Vec::new();

    for obj in p.objects.list() {
        if let Some(geom) = &obj.geometry {
            if let Some(mut mesh) = omnim_model::geometry::geometry_to_mesh(geom) {
                // Catalog component geometry is authored in millimetres; the
                // viewport is in metres, so scale it down for display.
                let is_catalog = obj.catalog_ref.is_some();
                if is_catalog {
                    mesh = omnim_model::geometry::scale_mesh(&mesh, 0.001);
                }
                let mut positions = Vec::new();
                let mut normals = Vec::new();
                let mut triangle_count = 0u32;

                for face in &mesh.faces {
                    let indices = &face.vertex_indices;
                    if indices.len() < 3 { continue; }
                    for i in 1..indices.len() - 1 {
                        for &idx in &[indices[0], indices[i], indices[i + 1]] {
                            let idx = idx as usize;
                            if idx < mesh.vertices.len() {
                                let p = &mesh.vertices[idx];
                                // Rust uses Z-up, Three.js now uses Z-up — no swap needed
                                positions.push(p.x as f32);
                                positions.push(p.y as f32);
                                positions.push(p.z as f32);
                            }
                            if idx < mesh.normals.len() {
                                let n = &mesh.normals[idx];
                                // No swap needed
                                normals.push(n.x as f32);
                                normals.push(n.y as f32);
                                normals.push(n.z as f32);
                            }
                        }
                        triangle_count += 1;
                    }
                }

                let bbox = omnim_model::geometry::compute_bbox(&mesh);
                let bbox_min = bbox.min.as_ref().map(|m| [m.x as f32, m.y as f32, m.z as f32]).unwrap_or([0.0; 3]);
                let bbox_max = bbox.max.as_ref().map(|m| [m.x as f32, m.y as f32, m.z as f32]).unwrap_or([0.0; 3]);

                objects.push(MeshObject {
                    id: obj.id.clone(),
                    name: obj.name.clone(),
                    object_type: obj.object_type_name().to_string(),
                    vertex_count: positions.len() as u32 / 3,
                    triangle_count,
                    positions,
                    normals,
                    bbox_min,
                    bbox_max,
                });
            }
        }
    }

    Json(MeshResponse { objects })
}

fn parse_type(s: &str) -> omnim_core::ObjectType {
    match s.to_lowercase().as_str() {
        "site" => omnim_core::ObjectType::Site,
        "building" => omnim_core::ObjectType::Building,
        "storey" | "floor" => omnim_core::ObjectType::BuildingStorey,
        "space" => omnim_core::ObjectType::Space,
        "wall" => omnim_core::ObjectType::Wall,
        "beam" => omnim_core::ObjectType::Beam,
        "column" => omnim_core::ObjectType::Column,
        "slab" => omnim_core::ObjectType::Slab,
        "door" => omnim_core::ObjectType::Door,
        "window" => omnim_core::ObjectType::Window,
        "stair" => omnim_core::ObjectType::Stair,
        "roof" => omnim_core::ObjectType::Roof,
        "duct" => omnim_core::ObjectType::Duct,
        "cable_tray" | "cabletray" => omnim_core::ObjectType::CableTray,
        "air_terminal" | "airterminal" | "diffuser" => omnim_core::ObjectType::AirTerminal,
        "ahu" | "air_handler" => omnim_core::ObjectType::Ahu,
        "chiller" => omnim_core::ObjectType::Chiller,
        "transformer" => omnim_core::ObjectType::Transformer,
        "luminaire" | "light" | "lighting" => omnim_core::ObjectType::Luminaire,
        "road" => omnim_core::ObjectType::Road,
        "bridge" => omnim_core::ObjectType::Bridge,
        "tunnel" => omnim_core::ObjectType::Tunnel,
        "dam" => omnim_core::ObjectType::Dam,
        "tank" => omnim_core::ObjectType::Tank,
        "pump" => omnim_core::ObjectType::Pump,
        "valve" => omnim_core::ObjectType::Valve,
        "pipe" => omnim_core::ObjectType::PipePlant,
        "pier" => omnim_core::ObjectType::Pier,
        "manhole" => omnim_core::ObjectType::Manhole,
        "abutment" => omnim_core::ObjectType::Abutment,
        "part" => omnim_core::ObjectType::Part,
        "assembly" => omnim_core::ObjectType::Assembly,
        "gear" => omnim_core::ObjectType::Gear,
        "shaft" => omnim_core::ObjectType::Shaft,
        "bearing" => omnim_core::ObjectType::Bearing,
        "motor" => omnim_core::ObjectType::Motor,
        "fitting" => omnim_core::ObjectType::Fitting,
        "compressor" => omnim_core::ObjectType::Compressor,
        "instrument" | "sensor" | "transmitter" => omnim_core::ObjectType::Instrument,
        "spring" => omnim_core::ObjectType::Spring,
        "seal" => omnim_core::ObjectType::Seal,
        _ => omnim_core::ObjectType::Generic,
    }
}

fn parse_category(s: &str) -> omnim_core::MaterialCategory {
    match s.to_lowercase().as_str() {
        "metal" => omnim_core::MaterialCategory::Metal,
        "concrete" => omnim_core::MaterialCategory::Concrete,
        "wood" => omnim_core::MaterialCategory::Wood,
        "masonry" => omnim_core::MaterialCategory::Masonry,
        "plastic" => omnim_core::MaterialCategory::Plastic,
        "composite" => omnim_core::MaterialCategory::Composite,
        "glass" => omnim_core::MaterialCategory::Glass,
        "insulation" => omnim_core::MaterialCategory::Insulation,
        "soil" => omnim_core::MaterialCategory::Soil,
        "water" => omnim_core::MaterialCategory::Water,
        _ => omnim_core::MaterialCategory::Other,
    }
}

fn parse_phase(s: &str) -> omnim_core::LifecyclePhase {
    match s.to_lowercase().as_str() {
        "planning" => omnim_core::LifecyclePhase::Planning,
        "design" => omnim_core::LifecyclePhase::Design,
        "procurement" => omnim_core::LifecyclePhase::Procurement,
        "construction" => omnim_core::LifecyclePhase::Construction,
        "commissioning" => omnim_core::LifecyclePhase::CommissioningAcceptance,
        "asset" => omnim_core::LifecyclePhase::Asset,
        "operation" => omnim_core::LifecyclePhase::OperationMaintenance,
        "production" => omnim_core::LifecyclePhase::Production,
        "decommissioning" => omnim_core::LifecyclePhase::Decommissioning,
        _ => omnim_core::LifecyclePhase::Unspecified,
    }
}

fn parse_admin_level(s: &str) -> omnim_core::AdministrativeLevel {
    match s.to_lowercase().as_str() {
        "world" => omnim_core::AdministrativeLevel::World,
        "country" => omnim_core::AdministrativeLevel::Country,
        "province" => omnim_core::AdministrativeLevel::Province,
        "prefecture" => omnim_core::AdministrativeLevel::Prefecture,
        "county" => omnim_core::AdministrativeLevel::County,
        "township" => omnim_core::AdministrativeLevel::Township,
        "community" => omnim_core::AdministrativeLevel::Community,
        "block" => omnim_core::AdministrativeLevel::Block,
        "parcel" => omnim_core::AdministrativeLevel::Parcel,
        _ => omnim_core::AdministrativeLevel::Unspecified,
    }
}

fn parse_breakdown_level(s: &str) -> omnim_core::BreakdownLevel {
    match s.to_lowercase().as_str() {
        "program" => omnim_core::BreakdownLevel::Program,
        "project" => omnim_core::BreakdownLevel::Project,
        "sub_project" | "subproject" => omnim_core::BreakdownLevel::SubProject,
        "unit" => omnim_core::BreakdownLevel::Unit,
        "building" => omnim_core::BreakdownLevel::Building,
        "storey" => omnim_core::BreakdownLevel::Storey,
        "zone" => omnim_core::BreakdownLevel::Zone,
        "system" => omnim_core::BreakdownLevel::System,
        "element" => omnim_core::BreakdownLevel::Element,
        "part" => omnim_core::BreakdownLevel::Part,
        _ => omnim_core::BreakdownLevel::Unspecified,
    }
}

fn parse_breakdown_convention(s: &str) -> omnim_core::BreakdownConvention {
    match s.to_lowercase().as_str() {
        "building" => omnim_core::BreakdownConvention::Building,
        "plant" => omnim_core::BreakdownConvention::Plant,
        "civil" => omnim_core::BreakdownConvention::Civil,
        "mechanical" => omnim_core::BreakdownConvention::Mechanical,
        _ => omnim_core::BreakdownConvention::Unspecified,
    }
}

pub async fn start(server_cfg: ServerConfig) {
    let addr = format!("{}:{}", server_cfg.host, server_cfg.port);
    let sock_addr: std::net::SocketAddr = addr.parse().expect("parse addr");
    let socket = tokio::net::TcpSocket::new_v4().expect("create socket");
    if let Err(e) = socket.set_reuseaddr(true) {
        eprintln!("Warning: could not set SO_REUSEADDR: {}", e);
    }
    socket.bind(sock_addr).expect("bind address");
    let listener = socket.listen(1024).expect("listen");

    println!("OmniM Server running at http://{}:{}", server_cfg.host, server_cfg.port);

    let router = create_router(server_cfg.project, server_cfg.save_path);
    axum::serve(listener, router).await.expect("server error");
}

pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub project: SharedProject,
    pub save_path: Option<String>,
}

// IFC Export Handler
async fn export_ifc(State(state): State<SharedState>) -> impl IntoResponse {
    let p = state.project.read().await;
    let ifc_content = omnim_ifc::export_ifc(&p);
    ([
        ("Content-Type", "application/ifc"),
        ("Content-Disposition", "attachment; filename=\"project.ifc\""),
    ], ifc_content)
}

// Import uses raw string body, not this struct

#[derive(Serialize)]
struct ImportIfcResponse {
    success: bool,
    imported: usize,
    objects: usize,
    message: String,
}

#[derive(Deserialize)]
struct ImportIfcQuery {
    normalize: Option<bool>,
    mode: Option<String>,
}

// IFC Import Handler
async fn import_ifc(
    State(state): State<SharedState>,
    Query(query): Query<ImportIfcQuery>,
    body: String,
) -> Result<Json<ImportIfcResponse>, StatusCode> {
    let imported = omnim_ifc::import_ifc_with_options(&body, omnim_ifc::ImportOptions {
        normalize_to_origin: query.normalize.unwrap_or(true),
    });

    match imported {
        Ok(imported_proj) => {
            let imported_count = imported_proj.objects.len();
            let mode = query.mode.as_deref().unwrap_or("merge");
            let mut p = state.project.write().await;

            // Bulk import is one whole-project transaction: snapshot before the
            // write, then record the before/after pair on the undo stack.
            let before = p.clone();
            if mode == "replace" {
                *p = imported_proj;
            } else {
                for obj in imported_proj.objects.list() {
                    p.objects.insert(obj.clone());
                }
                for mat in imported_proj.materials.list() {
                    p.materials.insert(mat.clone());
                }
                for record in imported_proj.lifecycle.list() {
                    p.lifecycle.insert(record.clone());
                }
                p.objects.rebuild_children();
            }
            {
                let mut exe = state.executor.write().await;
                exe.record_project_replace(before, &p);
            }
            mutation::persist(&state.save_path, &p);

            Ok(Json(ImportIfcResponse {
                success: true,
                imported: imported_count,
                objects: p.objects.len(),
                message: format!("Successfully imported {} objects", imported_count),
            }))
        }
        Err(e) => {
            let p = state.project.read().await;
            Ok(Json(ImportIfcResponse {
                success: false,
                imported: 0,
                objects: p.objects.len(),
                message: format!("Import failed: {}", e),
            }))
        }
    }
}

// ============================================================
// Sketch → 3D Feature Endpoints
// ============================================================

#[derive(Deserialize)]
struct ExtrudeSketchRequest {
    depth: f64,
    direction_x: Option<f64>,
    direction_y: Option<f64>,
    direction_z: Option<f64>,
}

#[derive(Serialize)]
struct FeatureResponse {
    success: bool,
    message: String,
    vertices: usize,
    faces: usize,
    object_id: String,
}

async fn extrude_sketch_handler(
    Path(id): Path<String>,
    State(state): State<SharedState>,
    Json(req): Json<ExtrudeSketchRequest>,
) -> Result<Json<FeatureResponse>, ApiError> {
    // Precheck: 404 for unknown sketch; unsolved sketches are test-solved on a
    // probe copy so an unsolvable profile fails with the same 400 as before.
    let must_solve = {
        let p = state.project.read().await;
        let sketch = p.sketches.get(&id)
            .ok_or_else(|| ApiError::not_found(format!("Sketch {} not found", id)))?;
        if sketch.solved {
            false
        } else {
            let mut probe = sketch.clone();
            let result = omnim_sketch::solver::solve(&mut probe);
            if !result.success {
                return Err(ApiError::bad_request("Sketch is unsolvable"));
            }
            true
        }
    };

    let mut ops = Vec::new();
    if must_solve {
        ops.push(mutation::op(c::operation::Op::SolveSketch(
            c::SolveSketchOperation { sketch_id: id.clone() },
        )));
    }
    ops.push(mutation::op(c::operation::Op::ExtrudeSketch(
        c::ExtrudeSketchOperation {
            sketch_id: id,
            depth: req.depth,
            direction: Some(c::Vector3D {
                x: req.direction_x.unwrap_or(0.0),
                y: req.direction_y.unwrap_or(0.0),
                z: req.direction_z.unwrap_or(1.0),
            }),
        },
    )));
    let result = mutation::apply(&state, c::Action::ExtrudeSketch, ops).await?;
    let result = mutation::require_success(result)?;
    let object_id = result.created_ids.last().cloned()
        .ok_or_else(|| ApiError::internal("extrude created no object"))?;

    let p = state.project.read().await;
    let obj = p.objects.get(&object_id)
        .ok_or_else(|| ApiError::internal("extruded object vanished"))?;
    let mesh = obj.geometry.as_ref().and_then(omnim_model::geometry::geometry_to_mesh);
    let (vertices, faces) = mesh.map(|m| (m.vertices.len(), m.faces.len()))
        .unwrap_or((0, 0));
    Ok(Json(FeatureResponse {
        success: true,
        message: format!("Extruded sketch with {} depth", req.depth),
        vertices,
        faces,
        object_id,
    }))
}

#[derive(Deserialize)]
struct RevolveSketchRequest {
    angle: f64,
    axis_origin_x: Option<f64>,
    axis_origin_y: Option<f64>,
    axis_origin_z: Option<f64>,
    axis_direction_x: Option<f64>,
    axis_direction_y: Option<f64>,
    axis_direction_z: Option<f64>,
}

async fn revolve_sketch_handler(
    Path(id): Path<String>,
    State(state): State<SharedState>,
    Json(req): Json<RevolveSketchRequest>,
) -> Result<Json<FeatureResponse>, ApiError> {
    // Precheck: 404 for unknown sketch; unsolved sketches are test-solved on a
    // probe copy so an unsolvable profile fails with the same 400 as before.
    let must_solve = {
        let p = state.project.read().await;
        let sketch = p.sketches.get(&id)
            .ok_or_else(|| ApiError::not_found(format!("Sketch {} not found", id)))?;
        if sketch.solved {
            false
        } else {
            let mut probe = sketch.clone();
            let result = omnim_sketch::solver::solve(&mut probe);
            if !result.success {
                return Err(ApiError::bad_request("Sketch is unsolvable"));
            }
            true
        }
    };

    let mut ops = Vec::new();
    if must_solve {
        ops.push(mutation::op(c::operation::Op::SolveSketch(
            c::SolveSketchOperation { sketch_id: id.clone() },
        )));
    }
    ops.push(mutation::op(c::operation::Op::RevolveSketch(
        c::RevolveSketchOperation {
            sketch_id: id,
            angle: req.angle,
            axis_origin: Some(c::Point3D {
                x: req.axis_origin_x.unwrap_or(0.0),
                y: req.axis_origin_y.unwrap_or(0.0),
                z: req.axis_origin_z.unwrap_or(0.0),
            }),
            axis_direction: Some(c::Vector3D {
                x: req.axis_direction_x.unwrap_or(0.0),
                y: req.axis_direction_y.unwrap_or(0.0),
                z: req.axis_direction_z.unwrap_or(1.0),
            }),
        },
    )));
    let result = mutation::apply(&state, c::Action::RevolveSketch, ops).await?;
    let result = mutation::require_success(result)?;
    let object_id = result.created_ids.last().cloned()
        .ok_or_else(|| ApiError::internal("revolve created no object"))?;

    let p = state.project.read().await;
    let obj = p.objects.get(&object_id)
        .ok_or_else(|| ApiError::internal("revolved object vanished"))?;
    let mesh = obj.geometry.as_ref().and_then(omnim_model::geometry::geometry_to_mesh);
    let (vertices, faces) = mesh.map(|m| (m.vertices.len(), m.faces.len()))
        .unwrap_or((0, 0));
    Ok(Json(FeatureResponse {
        success: true,
        message: format!("Revolved sketch with angle {}", req.angle),
        vertices,
        faces,
        object_id,
    }))
}

// ============================================================
// Component Endpoints
// ============================================================

#[derive(Serialize, Clone)]
struct ComponentResponse {
    id: String,
    name: String,
    feature_count: usize,
    sketch_ids: Vec<String>,
    parent_id: Option<String>,
}

#[derive(Deserialize)]
struct CreateComponentRequest {
    name: String,
    parent_id: Option<String>,
}

async fn list_components(State(state): State<SharedState>) -> Json<Vec<ComponentResponse>> {
    let p = state.project.read().await;
    let comps = p.component_ids();
    Json(comps.iter().map(|id| {
        let obj = p.objects.get(id).unwrap();
        ComponentResponse {
            id: obj.id.clone(),
            name: obj.name.clone(),
            feature_count: obj.features.len(),
            sketch_ids: obj.sketch_ids.clone(),
            parent_id: obj.parent_id.clone(),
        }
    }).collect())
}

async fn create_component_handler(
    State(state): State<SharedState>,
    Json(req): Json<CreateComponentRequest>,
) -> Result<Json<ComponentResponse>, ApiError> {
    let op = c::operation::Op::CreateComponent(c::CreateComponentOperation {
        name: req.name,
        parent_id: req.parent_id.clone().unwrap_or_default(),
    });
    let result = mutation::apply(&state, c::Action::CreateComponent, vec![mutation::op(op)]).await?;
    let result = mutation::require_success(result)?;
    let id = result.created_ids.first().cloned()
        .ok_or_else(|| ApiError::internal("component creation returned no id"))?;
    let p = state.project.read().await;
    let obj = p.objects.get(&id)
        .ok_or_else(|| ApiError::internal("created component vanished"))?;
    Ok(Json(ComponentResponse {
        id: obj.id.clone(),
        name: obj.name.clone(),
        feature_count: 0,
        sketch_ids: obj.sketch_ids.clone(),
        parent_id: obj.parent_id.clone(),
    }))
}

async fn get_component(
    Path(id): Path<String>,
    State(state): State<SharedState>,
) -> Result<Json<ComponentResponse>, (StatusCode, String)> {
    let p = state.project.read().await;
    let obj = p.objects.get(&id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Component {} not found", id)))?;
    if obj.object_type != c::ObjectType::Component as i32 {
        return Err((StatusCode::BAD_REQUEST, format!("Object {} is not a Component", id)));
    }
    Ok(Json(ComponentResponse {
        id: obj.id.clone(),
        name: obj.name.clone(),
        feature_count: obj.features.len(),
        sketch_ids: obj.sketch_ids.clone(),
        parent_id: obj.parent_id.clone(),
    }))
}

async fn delete_component(
    Path(id): Path<String>,
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    {
        let p = state.project.read().await;
        let obj = p.objects.get(&id)
            .ok_or_else(|| ApiError::not_found(format!("Component {} not found", id)))?;
        if obj.object_type != c::ObjectType::Component as i32 {
            return Err(ApiError::bad_request(format!("Object {} is not a Component", id)));
        }
    }
    let op = c::operation::Op::DeleteComponent(c::DeleteComponentOperation { component_id: id.clone() });
    let result = mutation::apply(&state, c::Action::DeleteComponent, vec![mutation::op(op)]).await?;
    mutation::require_success(result)?;
    Ok(Json(serde_json::json!({"success": true, "id": id})))
}

// ============================================================
// Feature Endpoints (on Components)
// ============================================================

#[derive(Serialize, Clone)]
struct FeatureResponseItem {
    id: String,
    name: String,
    feature_type: String,
    sketch_id: String,
    input_sketch_ids: Vec<String>,
    axis_sketch_id: String,
    depth: f64,
    angle: f64,
    is_cut: bool,
    order: i32,
    suppressed: bool,
    dirty: bool,
}

#[derive(Deserialize)]
struct AddFeatureRequest {
    feature_type: String,
    sketch_id: String,
    input_sketch_ids: Option<Vec<String>>,
    axis_sketch_id: Option<String>,
    depth: Option<f64>,
    direction_x: Option<f64>,
    direction_y: Option<f64>,
    direction_z: Option<f64>,
    angle: Option<f64>,
    is_cut: Option<bool>,
    lod: Option<f64>,
}

async fn list_features(
    Path(id): Path<String>,
    State(state): State<SharedState>,
) -> Result<Json<Vec<FeatureResponseItem>>, (StatusCode, String)> {
    let p = state.project.read().await;
    let features = p.get_features(&id)
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    Ok(Json(features.iter().map(|f| FeatureResponseItem {
        id: f.id.clone(),
        name: f.name.clone(),
        feature_type: format!("{:?}", f.feature_type).to_lowercase(),
        sketch_id: f.sketch_id.clone(),
        input_sketch_ids: f.input_sketch_ids.clone(),
        axis_sketch_id: f.axis_sketch_id.clone(),
        depth: f.depth,
        angle: f.angle,
        is_cut: f.is_cut,
        order: f.order,
        suppressed: f.suppressed,
        dirty: f.dirty,
    }).collect()))
}

async fn add_feature_handler(
    Path(component_id): Path<String>,
    State(state): State<SharedState>,
    Json(req): Json<AddFeatureRequest>,
) -> Result<Json<FeatureResponseItem>, ApiError> {
    // Verify sketch exists (same 404 as before; component errors surface as 400)
    if state.project.read().await.sketches.get(&req.sketch_id).is_none() {
        return Err(ApiError::not_found(format!("Sketch {} not found", req.sketch_id)));
    }

    let feature_type = match req.feature_type.to_lowercase().as_str() {
        "revolve" => c::FeatureType::FeatureRevolve,
        "sweep" => c::FeatureType::FeatureSweep,
        "loft" => c::FeatureType::FeatureLoft,
        "fusion" => c::FeatureType::FeatureFusion,
        "lofted_fusion" => c::FeatureType::FeatureLoftedFusion,
        "boolean" => c::FeatureType::FeatureBoolean,
        "chamfer" => c::FeatureType::FeatureChamfer,
        "fillet" => c::FeatureType::FeatureFillet,
        "pattern" => c::FeatureType::FeaturePattern,
        "shell" => c::FeatureType::FeatureShell,
        _ => c::FeatureType::FeatureExtrude,
    };

    // Two ops in one command: add the feature, then rebuild the component —
    // an invalid rebuild now rolls the whole command back instead of leaving
    // a newly added feature behind.
    let feature_id = {
        let result = mutation::apply(
            &state,
            c::Action::AddFeature,
            vec![
                mutation::op(c::operation::Op::AddFeature(c::AddFeatureOperation {
                    component_id: component_id.clone(),
                    feature_type: feature_type as i32,
                    sketch_id: req.sketch_id.clone(),
                    depth: req.depth.unwrap_or(0.0),
                    direction: Some(c::Vector3D {
                        x: req.direction_x.unwrap_or(0.0),
                        y: req.direction_y.unwrap_or(0.0),
                        z: req.direction_z.unwrap_or(1.0),
                    }),
                    angle: req.angle.unwrap_or(0.0),
                    axis_origin: Some(c::Point3D { x: 0.0, y: 0.0, z: 0.0 }),
                    axis_direction: Some(c::Vector3D { x: 0.0, y: 0.0, z: 1.0 }),
                    is_cut: req.is_cut.unwrap_or(false),
                    input_sketch_ids: req.input_sketch_ids.clone().unwrap_or_default(),
                    axis_sketch_id: req.axis_sketch_id.clone().unwrap_or_default(),
                    name: Some(format!("{}-{}", req.feature_type, &req.sketch_id[..8])),
                    lod: Some(req.lod.unwrap_or(omnim_model::brep::DEFAULT_TOLERANCE)),
                })),
                mutation::op(c::operation::Op::RebuildFeatures(c::RebuildFeaturesOperation {
                    component_id: component_id.clone(),
                    feature_id: None,
                })),
            ],
        )
        .await?;
        let result = mutation::require_success(result)?;
        result.created_ids.first().cloned()
            .ok_or_else(|| ApiError::internal("add_feature returned no feature id"))?
    };

    let p = state.project.read().await;
    let obj = p.objects.get(&component_id)
        .ok_or_else(|| ApiError::internal("component vanished"))?;
    let f = obj.features.iter().find(|f| f.id == feature_id)
        .ok_or_else(|| ApiError::internal("added feature vanished"))?;

    Ok(Json(FeatureResponseItem {
        id: f.id.clone(),
        name: f.name.clone(),
        feature_type: format!("{:?}", f.feature_type).to_lowercase(),
        sketch_id: f.sketch_id.clone(),
        input_sketch_ids: f.input_sketch_ids.clone(),
        axis_sketch_id: f.axis_sketch_id.clone(),
        depth: f.depth,
        angle: f.angle,
        is_cut: f.is_cut,
        order: f.order,
        suppressed: f.suppressed,
        dirty: f.dirty,
    }))
}

#[derive(Deserialize)]
struct UpdateFeatureRequest {
    sketch_id: Option<String>,
    axis_sketch_id: Option<String>,
    depth: Option<f64>,
    angle: Option<f64>,
    is_cut: Option<bool>,
    suppressed: Option<bool>,
    input_sketch_ids: Option<Vec<String>>,
    dependency_ids: Option<Vec<String>>,
    direction: Option<omnim_core::Vector3D>,
    axis_origin: Option<omnim_core::Point3D>,
    axis_direction: Option<omnim_core::Vector3D>,
    lod: Option<f64>,
    boolean_op: Option<i32>,
    pattern_count_x: Option<usize>,
    pattern_count_y: Option<usize>,
    pattern_count_z: Option<usize>,
}

async fn update_feature_handler(
    Path((component_id, feature_id)): Path<(String, String)>,
    State(state): State<SharedState>,
    Json(req): Json<UpdateFeatureRequest>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Preserve the old 404s from Project::update_feature before touching state.
    {
        let p = state.project.read().await;
        if let Some(obj) = p.objects.get(&component_id) {
            if !obj.features.iter().any(|f| f.id == feature_id) {
                return Err(ApiError::not_found(format!("feature '{}' not found", feature_id)));
            }
        } else {
            return Err(ApiError::not_found(format!("component '{}' not found", component_id)));
        }
    }
    // Two ops in one command: edit the feature, then rebuild the component.
    let ops = vec![
        mutation::op(c::operation::Op::UpdateFeature(c::UpdateFeatureOperation {
            component_id: component_id.clone(),
            feature_id: feature_id.clone(),
            sketch_id: req.sketch_id.clone(),
            axis_sketch_id: req.axis_sketch_id.clone(),
            depth: req.depth,
            angle: req.angle,
            is_cut: req.is_cut,
            suppressed: req.suppressed,
            input_sketch_ids: req.input_sketch_ids.clone().map(|ids| c::TagList { values: ids }),
            dependency_ids: req.dependency_ids.clone().map(|ids| c::TagList { values: ids }),
            direction: req.direction.clone(),
            axis_origin: req.axis_origin.clone(),
            axis_direction: req.axis_direction.clone(),
            lod: req.lod,
            boolean_op: req.boolean_op,
            pattern_count_x: req.pattern_count_x.map(|c| c as i32),
            pattern_count_y: req.pattern_count_y.map(|c| c as i32),
            pattern_count_z: req.pattern_count_z.map(|c| c as i32),
        })),
        mutation::op(c::operation::Op::RebuildFeatures(c::RebuildFeaturesOperation {
            component_id: component_id.clone(),
            feature_id: None,
        })),
    ];
    let result = mutation::apply(&state, c::Action::UpdateFeature, ops).await?;
    mutation::require_success(result)?;
    Ok(Json(serde_json::json!({"success": true})))
}

async fn remove_feature_handler(
    Path((component_id, feature_id)): Path<(String, String)>,
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Preserve the old 404s from Project::remove_feature before touching state.
    {
        let p = state.project.read().await;
        if let Some(obj) = p.objects.get(&component_id) {
            if !obj.features.iter().any(|f| f.id == feature_id) {
                return Err(ApiError::not_found(format!(
                    "feature '{}' not found in component '{}'", feature_id, component_id
                )));
            }
        } else {
            return Err(ApiError::not_found(format!("component '{}' not found", component_id)));
        }
    }
    // Two ops in one command: remove the feature, then rebuild the component.
    let ops = vec![
        mutation::op(c::operation::Op::RemoveFeature(c::RemoveFeatureOperation {
            component_id: component_id.clone(),
            feature_id: feature_id.clone(),
        })),
        mutation::op(c::operation::Op::RebuildFeatures(c::RebuildFeaturesOperation {
            component_id: component_id.clone(),
            feature_id: None,
        })),
    ];
    let result = mutation::apply(&state, c::Action::RemoveFeature, ops).await?;
    mutation::require_success(result)?;
    Ok(Json(serde_json::json!({"success": true})))
}

async fn rebuild_component_handler(
    Path(id): Path<String>,
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let op = mutation::op(c::operation::Op::RebuildFeatures(c::RebuildFeaturesOperation {
        component_id: id.clone(),
        feature_id: None,
    }));
    let result = mutation::apply(&state, c::Action::RebuildFeatures, vec![op]).await?;
    mutation::require_success(result)?;
    Ok(Json(serde_json::json!({"success": true})))
}

#[derive(Deserialize)]
struct SectionQuery {
    plane_z: Option<f64>,
}

async fn section_handler(
    Path(id): Path<String>,
    State(state): State<SharedState>,
    Query(query): Query<SectionQuery>,
) -> Result<Json<Vec<omnim_model::geometry::SectionLine>>, (StatusCode, String)> {
    let plane_z = query.plane_z.unwrap_or(0.0);

    // Always rebuild to ensure ALL features have cached_geometry
    {
        let mut p = state.project.write().await;
        let _ = p.rebuild_component(&id);
    }

    let p = state.project.read().await;
    let obj = p.objects.get(&id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("component '{}' not found", id)))?;

    let meshes: Vec<c::MeshGeometry> = obj.features.iter()
        .filter_map(|f| f.cached_geometry.clone())
        .collect();


    let merged = if meshes.len() > 1 {
        omnim_model::geometry::merge_meshes(&meshes)
    } else if let Some(m) = meshes.into_iter().next() {
        m
    } else {
        return Ok(Json(Vec::new()));
    };

    let lines = omnim_model::geometry::compute_section_lines(&merged, plane_z);
    Ok(Json(lines))
}

// ============================================================
// Catalog REST API
// ============================================================

async fn list_catalog() -> Json<Vec<omnim_model::catalog::ComponentDef>> {
    Json(LIBRARY.read().unwrap().as_ref().unwrap().catalog.list().into_iter().cloned().collect())
}

#[derive(Deserialize)]
struct CreateComponentDefRequest {
    name: String,
    category: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    ifc_class: String,
    #[serde(default)]
    standard_reference: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    domain: String,
}

async fn create_component_def(
    State(state): State<SharedState>,
    Json(req): Json<CreateComponentDefRequest>,
) -> Result<Json<omnim_model::catalog::ComponentDef>, ApiError> {
    let op = c::operation::Op::DefineComponent(c::DefineComponentOperation {
        name: req.name,
        category: req.category,
        description: req.description,
        ifc_class: req.ifc_class,
        standard_reference: req.standard_reference,
        tags: req.tags,
        domain: req.domain,
    });
    let result = mutation::apply(&state, c::Action::DefineComponent, vec![mutation::op(op)]).await?;
    let result = mutation::require_success(result)?;
    let id = result.created_ids.first().cloned()
        .ok_or_else(|| ApiError::internal("define_component returned no id"))?;
    let comp = LIBRARY.read().unwrap().as_ref().unwrap().catalog.get(&id).cloned()
        .ok_or_else(|| ApiError::internal("defined component vanished"))?;
    Ok(Json(comp))
}

async fn get_component_def(
    Path(id): Path<String>,
) -> Result<Json<omnim_model::catalog::ComponentDef>, StatusCode> {
    LIBRARY.read().unwrap().as_ref().unwrap().catalog.get(&id).cloned().map(Json).ok_or(StatusCode::NOT_FOUND)
}

async fn delete_component_def(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    if LIBRARY.read().unwrap().as_ref().unwrap().catalog.get(&id).is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    let op = c::operation::Op::DeleteComponentDef(c::DeleteComponentDefOperation { component_id: id });
    let result = mutation::apply(&state, c::Action::DeleteComponentDef, vec![mutation::op(op)]).await.map_err(|e| e.status)?;
    if !result.success {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct AddParamRequest {
    name: String,
    #[serde(default = "default_param_type")]
    param_type: i32,
    #[serde(default)]
    default_value: f64,
    #[serde(default)]
    min_value: f64,
    #[serde(default = "default_max_value")]
    max_value: f64,
    #[serde(default)]
    step: f64,
    #[serde(default)]
    unit: String,
    #[serde(default)]
    formula: String,
    #[serde(default)]
    choices: Vec<String>,
    #[serde(default)]
    description: String,
}

fn default_param_type() -> i32 { 1 }
fn default_max_value() -> f64 { f64::MAX }

async fn add_component_param(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<AddParamRequest>,
) -> Result<StatusCode, StatusCode> {
    if LIBRARY.read().unwrap().as_ref().unwrap().catalog.get(&id).is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    let op = c::operation::Op::AddComponentParam(c::AddComponentParamOperation {
        component_id: id,
        param_name: req.name,
        param_type: req.param_type,
        default_value: req.default_value,
        min_value: req.min_value,
        max_value: req.max_value,
        step: req.step,
        unit: req.unit,
        formula: req.formula,
        choices: req.choices,
        description: req.description,
    });
    let result = mutation::apply(&state, c::Action::AddComponentParam, vec![mutation::op(op)]).await.map_err(|e| e.status)?;
    if !result.success {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(StatusCode::CREATED)
}

#[derive(Deserialize)]
struct AddConnectorRequest {
    connector_id: String,
    connector_name: String,
    #[serde(default)]
    position_x_expr: String,
    #[serde(default)]
    position_y_expr: String,
    #[serde(default)]
    position_z_expr: String,
    #[serde(default)]
    direction_x_expr: String,
    #[serde(default)]
    direction_y_expr: String,
    #[serde(default = "default_dir_z")]
    direction_z_expr: String,
    #[serde(default)]
    bore_expr: String,
    #[serde(default)]
    connection_type_code: String,
    #[serde(default)]
    role: i32,
    #[serde(default)]
    up_x_expr: String,
    #[serde(default)]
    up_y_expr: String,
    #[serde(default)]
    up_z_expr: String,
}

fn default_dir_z() -> String { "1".into() }

async fn add_connector(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<AddConnectorRequest>,
) -> Result<StatusCode, StatusCode> {
    if LIBRARY.read().unwrap().as_ref().unwrap().catalog.get(&id).is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    let op = c::operation::Op::AddConnector(c::AddConnectorOperation {
        component_id: id,
        connector_id: req.connector_id,
        connector_name: req.connector_name,
        position_x_expr: req.position_x_expr,
        position_y_expr: req.position_y_expr,
        position_z_expr: req.position_z_expr,
        direction_x_expr: req.direction_x_expr,
        direction_y_expr: req.direction_y_expr,
        direction_z_expr: req.direction_z_expr,
        bore_expr: req.bore_expr,
        connection_type_code: req.connection_type_code,
        role: req.role,
        up_x_expr: req.up_x_expr,
        up_y_expr: req.up_y_expr,
        up_z_expr: req.up_z_expr,
    });
    let result = mutation::apply(&state, c::Action::AddConnector, vec![mutation::op(op)]).await.map_err(|e| e.status)?;
    if !result.success {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(StatusCode::CREATED)
}

async fn publish_component(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    if LIBRARY.read().unwrap().as_ref().unwrap().catalog.get(&id).is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    let op = c::operation::Op::PublishComponent(c::PublishComponentOperation { component_id: id });
    let result = mutation::apply(&state, c::Action::PublishComponent, vec![mutation::op(op)]).await.map_err(|e| e.status)?;
    if !result.success {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
struct UpdateComponentDefRequest {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    ifc_class: Option<String>,
    #[serde(default)]
    standard_reference: Option<String>,
    #[serde(default)]
    tags: Option<Vec<String>>,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    allowed_materials: Option<Vec<String>>,
    #[serde(default)]
    default_material: Option<String>,
    #[serde(default)]
    properties: Option<HashMap<String, String>>,
}

async fn update_component_def(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateComponentDefRequest>,
) -> Result<StatusCode, StatusCode> {
    if LIBRARY.read().unwrap().as_ref().unwrap().catalog.get(&id).is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    let op = c::operation::Op::UpdateComponentDef(c::UpdateComponentDefOperation {
        component_id: id,
        name: req.name,
        category: req.category,
        description: req.description,
        ifc_class: req.ifc_class,
        standard_reference: req.standard_reference,
        tags: req.tags.map(|tags| c::TagList { values: tags }),
        domain: req.domain,
        allowed_materials: req.allowed_materials.map(|m| c::TagList { values: m }),
        default_material: req.default_material,
        properties: req.properties.unwrap_or_default(),
    });
    let result = mutation::apply(&state, c::Action::UpdateComponentDef, vec![mutation::op(op)]).await.map_err(|e| e.status)?;
    if !result.success {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
struct UpdateParamRequest {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    param_type: Option<i32>,
    #[serde(default)]
    default_value: Option<f64>,
    #[serde(default)]
    min_value: Option<f64>,
    #[serde(default)]
    max_value: Option<f64>,
    #[serde(default)]
    step: Option<f64>,
    #[serde(default)]
    unit: Option<String>,
    #[serde(default)]
    formula: Option<String>,
    #[serde(default)]
    choices: Option<Vec<String>>,
    #[serde(default)]
    description: Option<String>,
}

async fn update_component_param(
    State(state): State<SharedState>,
    Path((id, idx)): Path<(String, usize)>,
    Json(req): Json<UpdateParamRequest>,
) -> Result<StatusCode, StatusCode> {
    if LIBRARY.read().unwrap().as_ref().unwrap().catalog.get(&id)
        .map(|comp| idx >= comp.params.len())
        .unwrap_or(true)
    {
        return Err(StatusCode::NOT_FOUND);
    }
    let op = c::operation::Op::UpdateComponentParam(c::UpdateComponentParamOperation {
        component_id: id,
        param_index: (idx + 1) as i32,
        name: req.name,
        param_type: req.param_type,
        default_value: req.default_value,
        min_value: req.min_value,
        max_value: req.max_value,
        step: req.step,
        unit: req.unit,
        formula: req.formula,
        choices: req.choices.map(|c| c::TagList { values: c }),
        description: req.description,
    });
    let result = mutation::apply(&state, c::Action::UpdateComponentParam, vec![mutation::op(op)]).await.map_err(|e| e.status)?;
    if !result.success {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(StatusCode::OK)
}

async fn delete_component_param(
    State(state): State<SharedState>,
    Path((id, idx)): Path<(String, usize)>,
) -> Result<StatusCode, StatusCode> {
    if LIBRARY.read().unwrap().as_ref().unwrap().catalog.get(&id)
        .map(|comp| idx >= comp.params.len())
        .unwrap_or(true)
    {
        return Err(StatusCode::NOT_FOUND);
    }
    let op = c::operation::Op::DeleteComponentParam(c::DeleteComponentParamOperation {
        component_id: id,
        param_index: (idx + 1) as i32,
    });
    let result = mutation::apply(&state, c::Action::DeleteComponentParam, vec![mutation::op(op)]).await.map_err(|e| e.status)?;
    if !result.success {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct AssignFeatureTreeRequest {
    feature_tree_id: String,
}

async fn assign_feature_tree(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<AssignFeatureTreeRequest>,
) -> Result<StatusCode, StatusCode> {
    if LIBRARY.read().unwrap().as_ref().unwrap().catalog.get(&id).is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    let op = c::operation::Op::AssignFeatureTree(c::AssignFeatureTreeOperation {
        component_id: id,
        feature_tree_id: req.feature_tree_id,
    });
    let result = mutation::apply(&state, c::Action::AssignFeatureTree, vec![mutation::op(op)]).await.map_err(|e| e.status)?;
    if !result.success {
        return Err(StatusCode::BAD_REQUEST);
    }
    Ok(StatusCode::OK)
}

#[derive(Serialize)]
struct ExportCatalogResponse {
    path: String,
    component_count: usize,
}

async fn export_catalog_handler(
) -> Result<Json<ExportCatalogResponse>, (StatusCode, String)> {
    let lib = LIBRARY.read().unwrap();
    let lib = lib.as_ref().unwrap();
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs();
    let path = format!("/tmp/omnimlib-{}.omnimlib", ts);
    lib.save_to_file(&path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let count = lib.catalog.len();
    Ok(Json(ExportCatalogResponse { path, component_count: count }))
}

#[derive(Deserialize)]
struct ImportCatalogRequest {
    path: String,
}

#[derive(Serialize)]
struct ImportCatalogResponse {
    component_count: usize,
    names: Vec<String>,
}

async fn import_catalog_handler(
    Json(req): Json<ImportCatalogRequest>,
) -> Result<Json<ImportCatalogResponse>, (StatusCode, String)> {
    let imported = omnim_model::Library::load_from_file(&req.path)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    let mut library = LIBRARY.write().unwrap();
    let library = library.as_mut().unwrap();
    let mut names = Vec::new();
    for comp in imported.catalog.list() {
        names.push(comp.name.clone());
        library.catalog.insert(comp.clone());
    }
    let count = names.len();
    Ok(Json(ImportCatalogResponse { component_count: count, names }))
}

// ============================================================
// Series REST API
// ============================================================

async fn list_series(State(state): State<SharedState>) -> Json<Vec<omnim_model::series_store::Series>> {
    let p = state.project.read().await;
    Json(p.series.list().into_iter().cloned().collect())
}

#[derive(Deserialize)]
struct CreateSeriesRequest {
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    domain: String,
    #[serde(default)]
    standard_reference: String,
}

// 2026-10-28 Wave 1 Slice 4: read-only tree walks used to preserve the
// handlers' 404 semantics before a command is applied.

fn series_selector<'a>(
    series: &'a omnim_model::series_store::Series,
    selector_id: &str,
) -> Option<&'a omnim_model::series_store::Selector> {
    fn find<'a>(
        selector: &'a omnim_model::series_store::Selector,
        id: &str,
    ) -> Option<&'a omnim_model::series_store::Selector> {
        if selector.id == id {
            return Some(selector);
        }
        for answer in &selector.answers {
            if let omnim_model::series_store::AnswerOutcome::NextSelector(next) = &answer.outcome {
                if let Some(found) = find(next, id) {
                    return Some(found);
                }
            }
        }
        None
    }
    series.root_selector.as_ref().and_then(|root| find(root, selector_id))
}

fn series_has_spec_component(
    series: &omnim_model::series_store::Series,
    component_def_id: &str,
) -> bool {
    fn walk(
        selector: &omnim_model::series_store::Selector,
        component_def_id: &str,
    ) -> bool {
        selector.answers.iter().any(|answer| match &answer.outcome {
            omnim_model::series_store::AnswerOutcome::Component(spec) => {
                spec.component_def_id == component_def_id
            }
            omnim_model::series_store::AnswerOutcome::NextSelector(next) => {
                walk(next, component_def_id)
            }
        })
    }
    series
        .root_selector
        .as_ref()
        .map(|root| walk(root, component_def_id))
        .unwrap_or(false)
}

fn series_has_param_constraint(
    series: &omnim_model::series_store::Series,
    component_def_id: &str,
    param_index: usize,
) -> bool {
    fn walk(
        selector: &omnim_model::series_store::Selector,
        component_def_id: &str,
        param_index: usize,
    ) -> Option<bool> {
        for answer in &selector.answers {
            match &answer.outcome {
                omnim_model::series_store::AnswerOutcome::Component(spec) => {
                    if spec.component_def_id == component_def_id {
                        return Some(param_index < spec.param_constraints.len());
                    }
                }
                omnim_model::series_store::AnswerOutcome::NextSelector(next) => {
                    if let Some(found) = walk(next, component_def_id, param_index) {
                        return Some(found);
                    }
                }
            }
        }
        None
    }
    series
        .root_selector
        .as_ref()
        .and_then(|root| walk(root, component_def_id, param_index))
        .unwrap_or(false)
}

async fn create_series_handler(
    State(state): State<SharedState>,
    Json(req): Json<CreateSeriesRequest>,
) -> Result<Json<omnim_model::series_store::Series>, ApiError> {
    let op = c::operation::Op::CreateSeries(c::CreateSeriesOperation {
        name: req.name,
        description: req.description,
        domain: req.domain,
        standard_reference: req.standard_reference,
    });
    let result =
        mutation::apply(&state, c::Action::CreateSeries, vec![mutation::op(op)]).await?;
    let result = mutation::require_success(result)?;
    let id = result
        .created_ids
        .first()
        .cloned()
        .ok_or_else(|| ApiError::internal("series creation returned no id"))?;
    let p = state.project.read().await;
    let series = p
        .series
        .get(&id)
        .cloned()
        .ok_or_else(|| ApiError::internal("created series vanished"))?;
    Ok(Json(series))
}

async fn get_series(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<Json<omnim_model::series_store::Series>, StatusCode> {
    let p = state.project.read().await;
    p.series.get(&id).cloned().map(Json).ok_or(StatusCode::NOT_FOUND)
}

#[derive(Deserialize)]
struct UpdateSeriesRequest {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    domain: Option<String>,
    #[serde(default)]
    standard_reference: Option<String>,
    #[serde(default)]
    applicable_phases: Option<Vec<String>>,
    #[serde(default)]
    metadata: Option<HashMap<String, String>>,
}

async fn update_series_handler(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateSeriesRequest>,
) -> Result<StatusCode, ApiError> {
    if state.project.read().await.series.get(&id).is_none() {
        return Err(ApiError::not_found(format!("series '{}' not found", id)));
    }
    let (replace_applicable_phases, applicable_phases) = match req.applicable_phases {
        Some(phases) => (true, phases),
        None => (false, Vec::new()),
    };
    let (replace_metadata, metadata) = match req.metadata {
        Some(metadata) => (true, metadata),
        None => (false, HashMap::new()),
    };
    let op = c::operation::Op::UpdateSeries(c::UpdateSeriesOperation {
        series_id: id,
        name: req.name,
        description: req.description,
        domain: req.domain,
        standard_reference: req.standard_reference,
        replace_applicable_phases,
        applicable_phases,
        replace_metadata,
        metadata,
    });
    let result =
        mutation::apply(&state, c::Action::UpdateSeries, vec![mutation::op(op)]).await?;
    mutation::require_success(result)?;
    Ok(StatusCode::OK)
}

async fn delete_series_handler(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<StatusCode, ApiError> {
    if state.project.read().await.series.get(&id).is_none() {
        return Err(ApiError::not_found(format!("series '{}' not found", id)));
    }
    let op = c::operation::Op::DeleteSeries(c::DeleteSeriesOperation { series_id: id });
    let result =
        mutation::apply(&state, c::Action::DeleteSeries, vec![mutation::op(op)]).await?;
    mutation::require_success(result)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct AddSelectorRequest {
    selector_id: String,
    question: String,
}

async fn add_selector_handler(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<AddSelectorRequest>,
) -> Result<StatusCode, ApiError> {
    if state.project.read().await.series.get(&id).is_none() {
        return Err(ApiError::not_found(format!("series '{}' not found", id)));
    }
    let op = c::operation::Op::AddSelector(c::AddSelectorOperation {
        series_id: id,
        selector_id: req.selector_id,
        question: req.question,
    });
    let result =
        mutation::apply(&state, c::Action::AddSelector, vec![mutation::op(op)]).await?;
    mutation::require_success(result)?;
    Ok(StatusCode::CREATED)
}

#[derive(Deserialize)]
struct AddAnswerRequest {
    selector_id: String,
    answer_id: String,
    answer_value: String,
    #[serde(default)]
    is_numeric_range: bool,
    #[serde(default)]
    range_min: f64,
    #[serde(default)]
    range_max: f64,
}

async fn add_answer_handler(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<AddAnswerRequest>,
) -> Result<StatusCode, ApiError> {
    let series_exists = {
        let p = state.project.read().await;
        match p.series.get(&id) {
            None => false,
            Some(series) => series_selector(series, &req.selector_id).is_some(),
        }
    };
    if !series_exists {
        return Err(ApiError::not_found(format!(
            "series '{}' or selector '{}' not found",
            id, req.selector_id
        )));
    }
    let op = c::operation::Op::AddAnswer(c::AddAnswerOperation {
        series_id: id,
        selector_id: req.selector_id,
        answer_id: req.answer_id,
        answer_value: req.answer_value,
        is_numeric_range: req.is_numeric_range,
        range_min: req.range_min,
        range_max: req.range_max,
    });
    let result = mutation::apply(&state, c::Action::AddAnswer, vec![mutation::op(op)]).await?;
    mutation::require_success(result)?;
    Ok(StatusCode::CREATED)
}

#[derive(Deserialize)]
struct AddSpecComponentRequest {
    selector_id: String,
    answer_id: String,
    component_def_id: String,
    #[serde(default)]
    material: String,
    #[serde(default)]
    connection_type_code: String,
}

async fn add_spec_component_handler(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<AddSpecComponentRequest>,
) -> Result<StatusCode, ApiError> {
    let series_exists = {
        let p = state.project.read().await;
        match p.series.get(&id) {
            None => false,
            Some(series) => series_selector(series, &req.selector_id)
                .map(|selector| selector.answers.iter().any(|a| a.id == req.answer_id))
                .unwrap_or(false),
        }
    };
    if !series_exists {
        return Err(ApiError::not_found(format!(
            "series '{}', selector '{}' or answer '{}' not found",
            id, req.selector_id, req.answer_id
        )));
    }
    let op = c::operation::Op::AddSpecComponent(c::AddSpecComponentOperation {
        series_id: id,
        selector_id: req.selector_id,
        answer_id: req.answer_id,
        component_def_id: req.component_def_id,
        material: req.material,
        connection_type_code: req.connection_type_code,
    });
    let result = mutation::apply(
        &state,
        c::Action::AddSpecComponent,
        vec![mutation::op(op)],
    )
    .await?;
    mutation::require_success(result)?;
    Ok(StatusCode::CREATED)
}

#[derive(Deserialize)]
struct AddParamConstraintRequest {
    component_def_id: String,
    param_name: String,
    #[serde(default)]
    default_value: f64,
    #[serde(default)]
    min_value: f64,
    #[serde(default = "default_max_value")]
    max_value: f64,
    #[serde(default)]
    step: f64,
    #[serde(default)]
    fixed_value: String,
}

async fn add_param_constraint_handler(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<AddParamConstraintRequest>,
) -> Result<StatusCode, ApiError> {
    let series_exists = {
        let p = state.project.read().await;
        match p.series.get(&id) {
            None => false,
            Some(series) => series_has_spec_component(series, &req.component_def_id),
        }
    };
    if !series_exists {
        return Err(ApiError::not_found(format!(
            "series '{}' or spec component '{}' not found",
            id, req.component_def_id
        )));
    }
    let op = c::operation::Op::AddParamConstraint(c::AddParamConstraintOperation {
        series_id: id,
        component_def_id: req.component_def_id,
        param_name: req.param_name,
        default_value: req.default_value,
        min_value: req.min_value,
        max_value: req.max_value,
        step: req.step,
        fixed_value: req.fixed_value,
    });
    let result = mutation::apply(
        &state,
        c::Action::AddParamConstraint,
        vec![mutation::op(op)],
    )
    .await?;
    mutation::require_success(result)?;
    Ok(StatusCode::CREATED)
}

#[derive(Deserialize)]
struct UpdateParamConstraintRequest {
    component_def_id: String,
    #[serde(default)]
    param_name: Option<String>,
    #[serde(default)]
    default_value: Option<f64>,
    #[serde(default)]
    min_value: Option<f64>,
    #[serde(default)]
    max_value: Option<f64>,
    #[serde(default)]
    step: Option<f64>,
    #[serde(default)]
    fixed_value: Option<String>,
}

async fn update_param_constraint_handler(
    State(state): State<SharedState>,
    Path((id, idx)): Path<(String, usize)>,
    Json(req): Json<UpdateParamConstraintRequest>,
) -> Result<StatusCode, ApiError> {
    let series_exists = {
        let p = state.project.read().await;
        match p.series.get(&id) {
            None => false,
            Some(series) => {
                series_has_param_constraint(series, &req.component_def_id, idx)
            }
        }
    };
    if !series_exists {
        return Err(ApiError::not_found(format!(
            "series '{}' or param constraint {} for '{}' not found",
            id, idx, req.component_def_id
        )));
    }
    let op = c::operation::Op::UpdateParamConstraint(c::UpdateParamConstraintOperation {
        series_id: id,
        component_def_id: req.component_def_id,
        param_index: idx as u32,
        param_name: req.param_name,
        default_value: req.default_value,
        min_value: req.min_value,
        max_value: req.max_value,
        step: req.step,
        fixed_value: req.fixed_value,
    });
    let result = mutation::apply(
        &state,
        c::Action::UpdateParamConstraint,
        vec![mutation::op(op)],
    )
    .await?;
    mutation::require_success(result)?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
struct DeleteParamConstraintRequest {
    component_def_id: String,
}

async fn delete_param_constraint_handler(
    State(state): State<SharedState>,
    Path((id, idx)): Path<(String, usize)>,
    Json(req): Json<DeleteParamConstraintRequest>,
) -> Result<StatusCode, ApiError> {
    let series_exists = {
        let p = state.project.read().await;
        match p.series.get(&id) {
            None => false,
            Some(series) => {
                series_has_param_constraint(series, &req.component_def_id, idx)
            }
        }
    };
    if !series_exists {
        return Err(ApiError::not_found(format!(
            "series '{}' or param constraint {} for '{}' not found",
            id, idx, req.component_def_id
        )));
    }
    let op = c::operation::Op::DeleteParamConstraint(c::DeleteParamConstraintOperation {
        series_id: id,
        component_def_id: req.component_def_id,
        param_index: idx as u32,
    });
    let result = mutation::apply(
        &state,
        c::Action::DeleteParamConstraint,
        vec![mutation::op(op)],
    )
    .await?;
    mutation::require_success(result)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct SelectRequest {
    answers: Vec<String>,
}

#[derive(Serialize)]
struct SelectResponse {
    component_def_id: String,
    selector_path: Vec<String>,
    spec_component: omnim_model::series_store::SpecComponent,
}

async fn select_from_series(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<SelectRequest>,
) -> Result<Json<SelectResponse>, (StatusCode, String)> {
    let p = state.project.read().await;
    let series = p.series.get(&id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("series '{}' not found", id)))?;
    let answer_refs: Vec<&str> = req.answers.iter().map(|s| s.as_str()).collect();
    match omnim_model::series_store::SelectionEngine::select(series, &answer_refs) {
        Ok(result) => Ok(Json(SelectResponse {
            component_def_id: result.component_def_id,
            selector_path: result.selector_path,
            spec_component: result.spec_component,
        })),
        Err(e) => Err((StatusCode::BAD_REQUEST, e)),
    }
}

#[derive(Deserialize)]
pub struct LoadSeriesRequest {
    pub path: String,
    #[serde(default = "default_load_source")]
    pub source: String,
}

fn default_load_source() -> String {
    "standalone".to_string()
}

#[derive(Serialize)]
pub struct LoadSeriesResponse {
    pub loaded_count: usize,
    pub series_names: Vec<String>,
}

#[derive(Serialize)]
struct SaveSeriesResponse {
    path: String,
    saved_count: usize,
}

async fn save_series_handler(
    State(state): State<SharedState>,
) -> Result<Json<SaveSeriesResponse>, (StatusCode, String)> {
    let p = state.project.read().await;
    let path = format!("/tmp/omnimseries-{}.omnimseries", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs());
    let store = p.series.clone();
    store.save_to_file(&path)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let count = store.len();
    Ok(Json(SaveSeriesResponse { path, saved_count: count }))
}

/// Load series from an external `.omnimseries` file.
async fn load_series_handler(
    State(state): State<SharedState>,
    Json(req): Json<LoadSeriesRequest>,
) -> Result<Json<LoadSeriesResponse>, (StatusCode, String)> {
    let op = mutation::op(c::operation::Op::LoadSeries(c::LoadSeriesOperation {
        path: req.path.clone(),
        source: req.source.clone(),
    }));
    let result = mutation::apply(&state, c::Action::LoadSeries, vec![op])
        .await
        .map_err(|e| (e.status, e.message))?;
    if !result.success {
        return Err((StatusCode::BAD_REQUEST, result.message));
    }

    let p = state.project.read().await;
    let names: Vec<String> = result
        .created_ids
        .iter()
        .filter_map(|id| p.series.get(id).map(|s| s.name.clone()))
        .collect();
    Ok(Json(LoadSeriesResponse {
        loaded_count: names.len(),
        series_names: names,
    }))
}

#[derive(Deserialize)]
pub struct InstantiateRequest {
    pub component_def_id: String,
    pub instance_name: String,
    #[serde(default)]
    pub params: HashMap<String, f64>,
    pub position: Option<c::Point3D>,
    /// Rotation around Z axis (radians), applied about the position point.
    #[serde(default)]
    pub rotation_z: f64,
    #[serde(default)]
    pub skip_series_gate: bool,
}

#[derive(Serialize)]
pub struct InstantiateResponse {
    pub component_id: String,
    pub object_type: String,
    pub name: String,
}

/// Instantiate a catalog component into the project.
async fn instantiate_from_catalog(
    State(state): State<SharedState>,
    Json(req): Json<InstantiateRequest>,
) -> Result<Json<InstantiateResponse>, (StatusCode, String)> {
    let op = mutation::op(c::operation::Op::InstantiateCatalog(c::InstantiateCatalogOperation {
        component_def_id: req.component_def_id.clone(),
        instance_name: req.instance_name.clone(),
        params: req.params.clone(),
        position: req.position.clone(),
        rotation_z: req.rotation_z,
        skip_series_gate: req.skip_series_gate,
    }));
    let result = mutation::apply(&state, c::Action::InstantiateCatalog, vec![op])
        .await
        .map_err(|e| (e.status, e.message))?;
    if !result.success {
        return Err((StatusCode::BAD_REQUEST, result.message));
    }
    let component_id = result
        .created_ids
        .first()
        .cloned()
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "instantiate returned no id".into()))?;

    let p = state.project.read().await;
    let obj = p
        .objects
        .get(&component_id)
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "created component not found".into()))?;

    Ok(Json(InstantiateResponse {
        component_id: obj.id.clone(),
        object_type: format!(
            "{:?}",
            c::ObjectType::try_from(obj.object_type).unwrap_or(c::ObjectType::Unspecified)
        ),
        name: obj.name.clone(),
    }))
}

#[derive(Deserialize)]
pub struct PreviewRequest {
    pub component_def_id: String,
    #[serde(default)]
    pub params: HashMap<String, f64>,
}

/// Preview a catalog component by creating a temporary instance and returning its mesh.
async fn preview_catalog_component(
    State(_state): State<SharedState>,
    Json(req): Json<PreviewRequest>,
) -> Result<Json<c::MeshGeometry>, (StatusCode, String)> {
    // Preview runs on a throwaway project: no write lock, no temporary object in
    // the live project, and no transient state visible to concurrent readers.
    let mut preview_project = Project::new("preview");
    let preview_name = format!("__preview_{}", req.component_def_id);
    let component_id = preview_project
        .instantiate_catalog_component_skip_gate(
            &req.component_def_id,
            &preview_name,
            &req.params,
            None,
            0.0,
        )
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let obj = preview_project
        .objects
        .get(&component_id)
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "preview component not found".into()))?;

    let mesh = obj
        .geometry
        .as_ref()
        .and_then(|g| g.representation.as_ref())
        .and_then(|rep| {
            if let c::geometry::Representation::Mesh(m) = rep {
                Some(m.clone())
            } else {
                None
            }
        })
        .unwrap_or(c::MeshGeometry {
            vertices: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            faces: Vec::new(),
        });

    // Catalog geometry is millimetres; scale to metres for the preview scene.
    Ok(Json(omnim_model::geometry::scale_mesh(&mesh, 0.001)))
}
