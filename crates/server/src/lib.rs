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

    let obj_id = result.created_ids.first().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // If parent_id was specified, set up the containment relationship
    if let Some(ref pid) = req.parent_id {
        if !pid.is_empty() {
            let _ = p.set_parent(obj_id, pid);
        }
    }

    let obj = p.objects.get_mut(obj_id).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Apply extra fields not supported by Omcom CreateObject protocol
    if let Some(ifc_class) = &req.ifc_class {
        obj.ifc_class = ifc_class.clone();
    }
    if let Some(tags) = req.tags {
        obj.add_tags(tags);
    }

    Ok(Json(obj.clone()))
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
) -> Result<(), StatusCode> {
    let mut p = state.project.write().await;
    let obj = p.objects.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    if let Some(name) = req.name { obj.name = name; }
    if let Some(desc) = req.description { obj.description = desc; }
    if let Some(ifc_class) = req.ifc_class { obj.ifc_class = ifc_class; }
    if let Some(tags) = req.tags { obj.add_tags(tags); }
    if let Some(ref path) = state.save_path {
        if let Err(e) = p.save(path) {
            eprintln!("Auto-save failed: {}", e);
        }
    }
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
) -> Result<(), StatusCode> {
    let mut p = state.project.write().await;
    let obj = p.objects.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    if let Some(name) = req.name { obj.name = name; }
    if let Some(desc) = req.description { obj.description = desc; }
    if let Some(ifc_class) = req.ifc_class { obj.ifc_class = ifc_class; }
    if let Some(tags) = req.tags { obj.add_tags(tags); }
    if let Some(properties) = req.properties {
        for (name, value) in properties {
            obj.set_property(&name, string_property(value));
        }
    }
    Ok(())
}

fn string_property(value: String) -> omnim_core::PropertyValue {
    omnim_core::PropertyValue {
        value_type: Some(omnim_core::property_value::ValueType::StringValue(value)),
    }
}

async fn delete_object(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<(), StatusCode> {
    let mut p = state.project.write().await;
    let mut exe = state.executor.write().await;
    let obj = p.objects.remove(&id).ok_or(StatusCode::NOT_FOUND)?;
    exe.record_delete(&obj);
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
) -> Result<(), StatusCode> {
    let mut p = state.project.write().await;
    let obj = p.objects.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    obj.set_property(&req.name, string_property(req.value));
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
) -> Result<(), StatusCode> {
    let mut p = state.project.write().await;
    let obj = p.objects.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    let geom = obj.geometry.as_mut().ok_or(StatusCode::BAD_REQUEST)?;
    let geom_id = geom.id.clone();
    let lod = geom.level_of_detail;
    let mesh = omnim_model::geometry::geometry_to_mesh(geom).ok_or(StatusCode::BAD_REQUEST)?;
    let origin = req.origin.unwrap_or_else(|| mesh_center(&mesh));
    let pre = omnim_core::Transform {
        matrix: vec![],
        translation: Some(omnim_core::Point3D { x: -origin.x, y: -origin.y, z: -origin.z }),
        rotation: None,
        scale: None,
    };
    let main = omnim_core::Transform {
        matrix: vec![],
        translation: req.translation,
        rotation: req.rotation,
        scale: req.scale,
    };
    let post = omnim_core::Transform {
        matrix: vec![],
        translation: Some(origin),
        rotation: None,
        scale: None,
    };
    let shifted = omnim_model::geometry::transform_mesh(&mesh, &pre);
    let transformed = omnim_model::geometry::transform_mesh(&shifted, &main);
    let transformed = omnim_model::geometry::transform_mesh(&transformed, &post);
    let bbox = omnim_model::geometry::compute_bbox(&transformed);
    *geom = omnim_core::Geometry {
        id: geom_id,
        r#type: omnim_core::GeometryType::Mesh as i32,
        representation: Some(omnim_core::geometry::Representation::Mesh(transformed)),
        bbox: Some(bbox),
        level_of_detail: lod,
    };
    Ok(())
}

fn mesh_center(mesh: &omnim_core::MeshGeometry) -> omnim_core::Point3D {
    if mesh.vertices.is_empty() {
        return omnim_core::Point3D { x: 0.0, y: 0.0, z: 0.0 };
    }
    let mut min = omnim_core::Point3D { x: f64::INFINITY, y: f64::INFINITY, z: f64::INFINITY };
    let mut max = omnim_core::Point3D { x: f64::NEG_INFINITY, y: f64::NEG_INFINITY, z: f64::NEG_INFINITY };
    for v in &mesh.vertices {
        min.x = min.x.min(v.x); min.y = min.y.min(v.y); min.z = min.z.min(v.z);
        max.x = max.x.max(v.x); max.y = max.y.max(v.y); max.z = max.z.max(v.z);
    }
    omnim_core::Point3D { x: (min.x + max.x) / 2.0, y: (min.y + max.y) / 2.0, z: (min.z + max.z) / 2.0 }
}

#[derive(Deserialize)]
struct SetParentRequest {
    parent_id: String,
}

async fn set_parent(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<SetParentRequest>,
) -> Result<(), StatusCode> {
    let mut p = state.project.write().await;
    p.set_parent(&id, &req.parent_id).map_err(|_| StatusCode::BAD_REQUEST)
}

async fn connect(
    State(state): State<SharedState>,
    Path((from, to)): Path<(String, String)>,
) -> Result<(), StatusCode> {
    let mut p = state.project.write().await;
    p.connect(&from, &to, omnim_core::ConnectionType::Structural)
        .map_err(|_| StatusCode::BAD_REQUEST)
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
) -> Result<(), StatusCode> {
    let mut p = state.project.write().await;
    let category = parse_category(&req.category);
    p.materials.insert(omnim_core::Material {
        id: req.id,
        name: req.name,
        category: category as i32,
        density: req.density.unwrap_or(0.0),
        ..Default::default()
    });
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
) -> Result<(), StatusCode> {
    let mut p = state.project.write().await;
    p.set_material(&id, &req.material_id).map_err(|_| StatusCode::BAD_REQUEST)
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
) -> Json<omnim_model::Measurement> {
    let mut p = state.project.write().await;
    let mut m = omnim_model::Measurement::new(&req.r#type, req.points, req.value, &req.color.unwrap_or_else(|| "#44ff88".into()));
    if let Some(label) = req.label {
        m.label = label;
    }
    p.measurements.add(m.clone());
    Json(m)
}

async fn delete_measurement(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<(), StatusCode> {
    let mut p = state.project.write().await;
    p.measurements.remove(&id).ok_or(StatusCode::NOT_FOUND)?;
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
) -> Result<(), StatusCode> {
    let mut p = state.project.write().await;
    let phase = parse_phase(&req.phase);
    let id = format!("lr-{}", p.lifecycle.len() + 1);
    p.lifecycle.insert(omnim_core::ProjectLifecycleRecord {
        id,
        phase: phase as i32,
        record_type: req.record_type,
        name: req.name,
        description: req.description.unwrap_or_default(),
        related_object_ids: Vec::new(),
        related_document_ids: Vec::new(),
        properties: Vec::new(),
        status: omnim_core::RecordStatus::Draft as i32,
        planned_start: None, planned_finish: None,
        actual_start: None, actual_finish: None,
        planned_cost: 0.0, actual_cost: 0.0,
        responsible_party: String::new(),
        version_id: String::new(),
    });
    Ok(())
}

async fn link_record(
    State(state): State<SharedState>,
    Path((record_id, obj_id)): Path<(String, String)>,
) -> Result<(), StatusCode> {
    let mut p = state.project.write().await;
    p.link_record_to_object(&record_id, &obj_id).map_err(|_| StatusCode::BAD_REQUEST)
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
) -> Result<Json<AdminDivResponse>, StatusCode> {
    let mut p = state.project.write().await;
    let level = parse_admin_level(&req.level);
    let mut div = omnim_model::AdministrativeDivision::new(level, &req.name, &req.code);
    if let Some(native) = &req.native_name {
        div.native_name = native.clone();
    }
    if let Some(parent_id) = &req.parent_id {
        div.parent_id = Some(parent_id.clone());
    }
    let id = p.administrative.insert(div);
    let d = p.administrative.get(&id).unwrap();
    Ok(Json(admin_to_response(d)))
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
) -> Result<(), StatusCode> {
    let mut p = state.project.write().await;
    p.administrative.remove(&id).ok_or(StatusCode::NOT_FOUND)?;
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
) -> Result<Json<BreakdownResponse>, StatusCode> {
    let mut p = state.project.write().await;
    let level = parse_breakdown_level(&req.level);
    let convention = parse_breakdown_convention(&req.convention);
    let mut node = omnim_model::BreakdownNode::new(level, &req.name, convention);
    if let Some(code) = &req.code {
        node.code = code.clone();
    }
    if let Some(parent_id) = &req.parent_id {
        node.parent_id = Some(parent_id.clone());
    }
    let id = p.breakdown.insert(node);
    let n = p.breakdown.get(&id).unwrap();
    Ok(Json(breakdown_to_response(n)))
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
) -> Result<(), StatusCode> {
    let mut p = state.project.write().await;
    p.breakdown.remove(&id).ok_or(StatusCode::NOT_FOUND)?;
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
) -> Result<(), StatusCode> {
    let mut p = state.project.write().await;
    let node = p.breakdown.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    node.link_object(&req.object_id);
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
) -> Result<(), StatusCode> {
    let mut p = state.project.write().await;
    let node = p.breakdown.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    node.link_administrative_division(&req.division_id);
    Ok(())
}

async fn execute_omcom(
    State(state): State<SharedState>,
    Json(instr): Json<omnim_core::OmniCommand>,
) -> Result<Json<omnim_omcom::ExecutionResult>, StatusCode> {
    let mut p = state.project.write().await;
    let mut exe = state.executor.write().await;
    let result = exe.execute(&instr, &mut p);
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
    Road { name: String, length: f64, width: f64, thickness: f64, #[serde(default)] parent_id: Option<String>, #[serde(default)] series_id: Option<String> },
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
        /// Continuous center-line path (wall): coordinate triples [x, y, z] in metres.
        #[serde(default)]
        path: Vec<[f64; 3]>,
        /// Closed boundary footprint (slab/roof/floor): coordinate triples [x, y, z].
        #[serde(default)]
        boundary: Vec<[f64; 3]>,
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
        #[serde(default)]
        alignment: Vec<[f64; 3]>,
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
        GenRuleRequest::Road { name, length, width, thickness, .. } => {
            let mesh = omnim_model::gen_rule::linear_component::build_road(
                *length, *width, *thickness * 0.7, *thickness * 0.3, 30,
            );
            let geom = omnim_model::geometry::mesh_to_geometry(&mesh);
            let id = p.create_object(omnim_core::ObjectType::Road, name).id.clone();
            let obj = p.objects.get_mut(&id).expect("genrule object was just created");
            obj.geometry = Some(geom);
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
            use omnim_core::Point2D;
            use omnim_model::gen_rule::layer_extrusion::{build_layered_wall, build_layered_slab, build_layered_path, build_layered_slab_path, LayerDef as MLayerDef};
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
            let to_p2 = |p: &[f64; 3]| Point2D { x: p[0], y: p[1] };
            let mut mesh = if mode.as_str() == "wall" {
                if path.len() >= 2 {
                    build_layered_path(&path.iter().map(to_p2).collect::<Vec<_>>(), false, *height, &layer_defs)
                } else {
                    build_layered_wall(*length, *height, &layer_defs)
                }
            } else if boundary.len() >= 3 {
                // Closed footprint: lift each layer along +Z over the boundary.
                build_layered_slab_path(&boundary.iter().map(to_p2).collect::<Vec<_>>(), &layer_defs)
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
            obj.clone()
        }
        GenRuleRequest::Line { name, alignment, diameter, .. } => {
            use omnim_core::Point3D;
            let pts: Vec<Point3D> = alignment.iter().map(|c| Point3D { x: c[0], y: c[1], z: c[2] }).collect();
            if pts.len() < 2 {
                return Err((StatusCode::BAD_REQUEST, "line genrule requires at least 2 alignment points".into()));
            }
            let mesh = omnim_model::gen_rule::linear_component::build_guide_line(&pts, if *diameter > 0.0 { *diameter } else { 0.05 }, 16);
            let geom = omnim_model::geometry::mesh_to_geometry(&mesh);
            // FeatureLine/ContourLine map to the proto ObjectType when available.
            let object_type = omnim_core::ObjectType::FeatureLine;
            let id = p.create_object(object_type, name).id.clone();
            let obj = p.objects.get_mut(&id).expect("genrule object was just created");
            obj.geometry = Some(geom);
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
) -> Result<Json<SketchResponse>, (StatusCode, String)> {
    let mut p = state.project.write().await;
    let origin = req.origin.map(|o| c::Point3D { x: o.x, y: o.y, z: o.z })
        .unwrap_or(c::Point3D { x: 0.0, y: 0.0, z: 0.0 });
    let normal = req.normal.map(|n| c::Vector3D { x: n.x, y: n.y, z: n.z })
        .unwrap_or(c::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
    let mut sketch = omnim_sketch::Sketch::new(&req.name, origin, normal);
    if let Some(ref comp_id) = req.component_id {
        if p.objects.get(comp_id).map(|o| o.object_type != c::ObjectType::Component as i32).unwrap_or(true) {
            return Err((StatusCode::BAD_REQUEST, format!("Component '{}' not found", comp_id)));
        }
        sketch.component_id = Some(comp_id.clone());
    }
    let sketch_id = sketch.id.clone();
    let resp = sketch_to_response(&sketch);
    p.sketches.insert(sketch);
    state.executor.write().await.record_sketch_create(&sketch_id);
    Ok(Json(resp))
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
) -> Result<StatusCode, (StatusCode, String)> {
    let mut p = state.project.write().await;
    match p.sketches.remove(&id) {
        Some(sketch) => {
            state.executor.write().await.record_sketch_delete(&sketch);
            Ok(StatusCode::NO_CONTENT)
        }
        None => Err((StatusCode::NOT_FOUND, format!("Sketch {} not found", id))),
    }
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
) -> Result<Json<omnim_sketch::Sketch>, (StatusCode, String)> {
    let mut p = state.project.write().await;
    let sketch = p.sketches.get_mut(&id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Sketch {} not found", id)))?;
    if let Some(origin) = req.origin {
        sketch.origin = c::Point3D { x: origin.x, y: origin.y, z: origin.z };
    }
    if let Some(name) = req.name {
        sketch.name = name;
    }
    Ok(Json(sketch.clone()))
}

async fn add_sketch_entity(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<AddEntityRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut p = state.project.write().await;
    let sketch = p.sketches.get_mut(&id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Sketch {} not found", id)))?;

    let entity_type = match req.entity_type.as_str() {
        "point" => omnim_sketch::SketchEntityType::Point,
        "line" => omnim_sketch::SketchEntityType::Line,
        "circle" => omnim_sketch::SketchEntityType::Circle,
        "arc" => omnim_sketch::SketchEntityType::Arc,
        "rectangle" => omnim_sketch::SketchEntityType::Rectangle,
        "polygon" => omnim_sketch::SketchEntityType::Polygon,
        "ellipse" => omnim_sketch::SketchEntityType::Ellipse,
        "spline" => omnim_sketch::SketchEntityType::Spline,
        "polyline" => omnim_sketch::SketchEntityType::Polyline,
        _ => return Err((StatusCode::BAD_REQUEST, format!("Unknown entity type: {}", req.entity_type))),
    };

    let points: Vec<c::Point3D> = req.points.iter()
        .map(|p| c::Point3D { x: p.x, y: p.y, z: p.z })
        .collect();

    let radius = req.radius.unwrap_or(0.0);
    let sides = req.sides.unwrap_or(0);

    let entity = omnim_sketch::SketchEntity {
        id: uuid::Uuid::new_v4().to_string(),
        entity_type,
        points,
        radius,
        sides,
        construction: req.construction,
    };

    let idx = sketch.add_entity(entity.clone());
    state.executor.write().await.record_sketch_add_entity(&id, idx, &entity);
    Ok(Json(serde_json::json!({ "index": idx })))
}

async fn add_sketch_constraint(
    State(state): State<SharedState>,
    Path(id): Path<String>,
    Json(req): Json<AddConstraintRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut p = state.project.write().await;
    let sketch = p.sketches.get_mut(&id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Sketch {} not found", id)))?;

    let constraint_type = match req.constraint_type.as_str() {
        "horizontal" => omnim_sketch::SketchConstraintType::Horizontal,
        "vertical" => omnim_sketch::SketchConstraintType::Vertical,
        "coincident" => omnim_sketch::SketchConstraintType::Coincident,
        "parallel" => omnim_sketch::SketchConstraintType::Parallel,
        "perpendicular" => omnim_sketch::SketchConstraintType::Perpendicular,
        "length" => omnim_sketch::SketchConstraintType::Length,
        "distance" => omnim_sketch::SketchConstraintType::Distance,
        "angle" => omnim_sketch::SketchConstraintType::Angle,
        "radius" => omnim_sketch::SketchConstraintType::Radius,
        "diameter" => omnim_sketch::SketchConstraintType::Diameter,
        "tangent" => omnim_sketch::SketchConstraintType::Tangent,
        "symmetric" => omnim_sketch::SketchConstraintType::Symmetric,
        "fixed" => omnim_sketch::SketchConstraintType::Fixed,
        "at_midpoint" => omnim_sketch::SketchConstraintType::AtMidpoint,
        "equal_length" => omnim_sketch::SketchConstraintType::EqualLength,
        "equal_radius" => omnim_sketch::SketchConstraintType::EqualRadius,
        "pt_on_line" => omnim_sketch::SketchConstraintType::PtOnLine,
        "pt_on_circle" => omnim_sketch::SketchConstraintType::PtOnCircle,
        "same_orientation" => omnim_sketch::SketchConstraintType::SameOrientation,
        _ => return Err((StatusCode::BAD_REQUEST, format!("Unknown constraint type: {}", req.constraint_type))),
    };

    let entity_refs: Vec<omnim_sketch::SketchPointRef> = req.entity_refs.iter()
        .map(|(ei, pi)| omnim_sketch::SketchPointRef { entity_index: *ei, point_index: *pi })
        .collect();

    let constraint = omnim_sketch::SketchConstraint {
        id: uuid::Uuid::new_v4().to_string(),
        constraint_type,
        entity_refs,
        value: req.value.unwrap_or(0.0),
    };

    let idx = sketch.add_constraint(constraint.clone());
    state.executor.write().await.record_sketch_add_constraint(&id, idx, &constraint);
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
) -> Result<Json<SolveResponse>, (StatusCode, String)> {
    let mut p = state.project.write().await;
    let sketch = p.sketches.get_mut(&id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Sketch {} not found", id)))?;

    let result = omnim_sketch::solver::solve(sketch);
    Ok(Json(SolveResponse {
        success: result.success,
        dof: result.dof,
        message: result.message,
    }))
}

async fn delete_sketch_entity(
    State(state): State<SharedState>,
    Path((id, idx)): Path<(String, usize)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut p = state.project.write().await;
    let sketch = p.sketches.get_mut(&id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Sketch {} not found", id)))?;
    if idx < sketch.entities.len() {
        let entity = sketch.entities[idx].clone();
        sketch.entities.remove(idx);
        sketch.solved = false;
        state.executor.write().await.record_sketch_delete_entity(&id, &entity);
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, format!("Entity index {} out of range", idx)))
    }
}

async fn update_sketch_entity(
    State(state): State<SharedState>,
    Path((id, idx)): Path<(String, usize)>,
    Json(req): Json<UpdateEntityRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut p = state.project.write().await;
    let sketch = p.sketches.get_mut(&id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Sketch {} not found", id)))?;
    if idx >= sketch.entities.len() {
        return Err((StatusCode::NOT_FOUND, format!("Entity index {} out of range", idx)));
    }
    let entity = &mut sketch.entities[idx];
    if let Some(points) = req.points {
        entity.points = points.iter().map(|p| omnim_core::Point3D { x: p.x, y: p.y, z: p.z }).collect();
    }
    if let Some(radius) = req.radius {
        entity.radius = radius;
    }
    if let Some(sides) = req.sides {
        entity.sides = sides;
    }
    if let Some(construction) = req.construction {
        entity.construction = construction;
    }
    sketch.solved = false;
    Ok(StatusCode::NO_CONTENT)
}

async fn remove_sketch_constraint(
    State(state): State<SharedState>,
    Path((id, idx)): Path<(String, usize)>,
) -> Result<StatusCode, (StatusCode, String)> {
    let mut p = state.project.write().await;
    let sketch = p.sketches.get_mut(&id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Sketch {} not found", id)))?;
    if idx < sketch.constraints.len() {
        let constraint = sketch.constraints[idx].clone();
        sketch.constraints.remove(idx);
        sketch.solved = false;
        state.executor.write().await.record_sketch_remove_constraint(&id, &constraint);
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((StatusCode::NOT_FOUND, format!("Constraint index {} out of range", idx)))
    }
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
        Ok(msg) => Ok(Json(UndoRedoResponse { success: true, message: msg })),
        Err(e) => Ok(Json(UndoRedoResponse { success: false, message: e })),
    }
}

async fn redo(
    State(state): State<SharedState>,
) -> Result<Json<UndoRedoResponse>, (StatusCode, String)> {
    let mut p = state.project.write().await;
    let mut exe = state.executor.write().await;
    match exe.redo(&mut p) {
        Ok(msg) => Ok(Json(UndoRedoResponse { success: true, message: msg })),
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
) -> Result<Json<FeatureResponse>, (StatusCode, String)> {
    let mut p = state.project.write().await;
    let sketch = p.sketches.get_mut(&id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Sketch {} not found", id)))?;

    if !sketch.solved {
        let result = omnim_sketch::solver::solve(sketch);
        if !result.success {
            return Err((StatusCode::BAD_REQUEST, "Sketch is unsolvable".into()));
        }
    }

    let dir = omnim_core::Vector3D {
        x: req.direction_x.unwrap_or(0.0),
        y: req.direction_y.unwrap_or(0.0),
        z: req.direction_z.unwrap_or(1.0),
    };

    match omnim_sketch::profile::sketch_to_profile_2d(sketch)
        .and_then(|profile2d| {
            if profile2d.len() < 3 { return None; }
            let profile3d: Vec<omnim_core::Point3D> = profile2d.iter()
                .map(|p| omnim_core::Point3D { x: p.x, y: p.y, z: 0.0 })
                .collect();
            omnim_model::brep::extrude_brep(&profile3d, req.depth, dir, omnim_model::brep::DEFAULT_TOLERANCE)
        }) {
        Some(mg) => {
            let geom = omnim_model::geometry::mesh_to_geometry(&mg);
            let object_id = uuid::Uuid::new_v4().to_string();
            let obj = omnim_model::OmniObject {
                id: object_id.clone(),
                name: format!("extrude-{}", &sketch.name),
                description: format!("Extruded from sketch '{}' depth={}", sketch.name, req.depth),
                object_type: omnim_core::ObjectType::Part as i32,
                status: 0,
                properties: vec![],
                property_sets: vec![],
                geometry: Some(geom),
                material_id: None,
                parent_id: None,
                tags: vec![],
                ifc_class: "IfcBuildingElementProxy".into(),
                features: vec![],
                sketch_ids: vec![],
                dirty: false,
                catalog_ref: None,
            };
            p.objects.insert(obj);
            Ok(Json(FeatureResponse {
                success: true,
                message: format!("Extruded sketch with {} depth", req.depth),
                vertices: mg.vertices.len(),
                faces: mg.faces.len(),
                object_id,
            }))
        }
        None => Err((StatusCode::BAD_REQUEST, "Failed to extrude sketch — check profile".into())),
    }
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
) -> Result<Json<FeatureResponse>, (StatusCode, String)> {
    let mut p = state.project.write().await;
    let sketch = p.sketches.get_mut(&id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Sketch {} not found", id)))?;

    if !sketch.solved {
        let result = omnim_sketch::solver::solve(sketch);
        if !result.success {
            return Err((StatusCode::BAD_REQUEST, "Sketch is unsolvable".into()));
        }
    }

    let origin = omnim_core::Point3D {
        x: req.axis_origin_x.unwrap_or(0.0),
        y: req.axis_origin_y.unwrap_or(0.0),
        z: req.axis_origin_z.unwrap_or(0.0),
    };
    let axis = omnim_core::Vector3D {
        x: req.axis_direction_x.unwrap_or(0.0),
        y: req.axis_direction_y.unwrap_or(0.0),
        z: req.axis_direction_z.unwrap_or(1.0),
    };

    match omnim_sketch::profile::sketch_to_profile_2d(sketch)
        .and_then(|profile2d| {
            if profile2d.len() < 3 { return None; }
            let profile3d: Vec<omnim_core::Point3D> = profile2d.iter()
                .map(|p| omnim_core::Point3D { x: p.x, y: p.y, z: 0.0 })
                .collect();
            omnim_model::brep::revolve_brep(&profile3d, req.angle, origin, axis, omnim_model::brep::DEFAULT_TOLERANCE)
        }) {
        Some(mg) => {
            let geom = omnim_model::geometry::mesh_to_geometry(&mg);
            let object_id = uuid::Uuid::new_v4().to_string();
            let obj = omnim_model::OmniObject {
                id: object_id.clone(),
                name: format!("revolve-{}", &sketch.name),
                description: format!("Revolved from sketch '{}' angle={}", sketch.name, req.angle),
                object_type: omnim_core::ObjectType::Part as i32,
                status: 0,
                properties: vec![],
                property_sets: vec![],
                geometry: Some(geom),
                material_id: None,
                parent_id: None,
                tags: vec![],
                ifc_class: "IfcBuildingElementProxy".into(),
                features: vec![],
                sketch_ids: vec![],
                dirty: false,
                catalog_ref: None,
            };
            p.objects.insert(obj);
            Ok(Json(FeatureResponse {
                success: true,
                message: format!("Revolved sketch with angle {}", req.angle),
                vertices: mg.vertices.len(),
                faces: mg.faces.len(),
                object_id,
            }))
        }
        None => Err((StatusCode::BAD_REQUEST, "Failed to revolve sketch — check profile".into())),
    }
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
) -> Result<Json<ComponentResponse>, (StatusCode, String)> {
    let mut p = state.project.write().await;
    let id = p.create_component(&req.name, req.parent_id.as_deref());
    let obj = p.objects.get(&id).unwrap();
    Ok(Json(ComponentResponse {
        id: obj.id.clone(),
        name: obj.name.clone(),
        feature_count: 0,
        sketch_ids: vec![],
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
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut p = state.project.write().await;
    let obj = p.objects.get_mut(&id)
        .ok_or_else(|| (StatusCode::NOT_FOUND, format!("Component {} not found", id)))?;
    if obj.object_type != c::ObjectType::Component as i32 {
        return Err((StatusCode::BAD_REQUEST, format!("Object {} is not a Component", id)));
    }
    p.objects.remove(&id);
    p.objects.rebuild_children();
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
) -> Result<Json<FeatureResponseItem>, (StatusCode, String)> {
    let mut p = state.project.write().await;

    // Verify sketch exists
    if p.sketches.get(&req.sketch_id).is_none() {
        return Err((StatusCode::NOT_FOUND, format!("Sketch {} not found", req.sketch_id)));
    }

    let dir = omnim_core::Vector3D {
        x: req.direction_x.unwrap_or(0.0),
        y: req.direction_y.unwrap_or(0.0),
        z: req.direction_z.unwrap_or(1.0),
    };

    let feature_type = match req.feature_type.to_lowercase().as_str() {
        "revolve" => omnim_model::FeatureType::Revolve,
        "sweep" => omnim_model::FeatureType::Sweep,
        "loft" => omnim_model::FeatureType::Loft,
        "fusion" => omnim_model::FeatureType::Fusion,
        "lofted_fusion" => omnim_model::FeatureType::LoftedFusion,
        "boolean" => omnim_model::FeatureType::Boolean,
        "chamfer" => omnim_model::FeatureType::Chamfer,
        "fillet" => omnim_model::FeatureType::Fillet,
        "pattern" => omnim_model::FeatureType::Pattern,
        "shell" => omnim_model::FeatureType::Shell,
        _ => omnim_model::FeatureType::Extrude,
    };

    let feature = omnim_model::Feature {
        id: uuid::Uuid::new_v4().to_string(),
        name: format!("{}-{}", req.feature_type, &req.sketch_id[..8]),
        feature_type: feature_type.clone(),
        sketch_id: req.sketch_id.clone(),
        axis_sketch_id: req.axis_sketch_id.clone().unwrap_or_default(),
        input_sketch_ids: req.input_sketch_ids.clone().unwrap_or_default(),
        dependency_ids: Vec::new(),
        depth: req.depth.unwrap_or(0.0),
        direction: dir,
        angle: req.angle.unwrap_or(0.0),
        axis_origin: omnim_core::Point3D { x: 0.0, y: 0.0, z: 0.0 },
        axis_direction: omnim_core::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        is_cut: req.is_cut.unwrap_or(false),
        order: 0,
        suppressed: false,
        cached_geometry: None,
        dirty: true,
        lod: req.lod.unwrap_or(omnim_model::brep::DEFAULT_TOLERANCE),
        boolean_op: None,
        pattern_count_x: None,
        pattern_count_y: None,
        pattern_count_z: None,
    };

    let fid = p.add_feature(&component_id, feature)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let obj = p.objects.get(&component_id).unwrap();
    let f = obj.features.iter().find(|f| f.id == fid).unwrap();

    let resp = FeatureResponseItem {
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
    };
    if let Err(e) = p.rebuild_component(&component_id) {
        eprintln!("rebuild_component failed after add_feature: {}", e);
    }
    if let Some(ref path) = state.save_path {
        if let Err(e) = p.save(path) {
            eprintln!("Auto-save failed: {}", e);
        }
    }
    Ok(Json(resp))
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
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut p = state.project.write().await;
    p.update_feature(
        &component_id, &feature_id,
        req.sketch_id, req.axis_sketch_id, req.depth, req.angle, req.is_cut, req.suppressed,
        req.input_sketch_ids, req.dependency_ids, req.direction, req.axis_origin, req.axis_direction,
        req.lod,
        req.boolean_op.map(|b| match b {
            0 => omnim_model::BooleanOp::Union,
            2 => omnim_model::BooleanOp::Intersect,
            _ => omnim_model::BooleanOp::Subtract,
        }),
        req.pattern_count_x, req.pattern_count_y, req.pattern_count_z,
    )
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    if let Err(e) = p.rebuild_component(&component_id) {
        eprintln!("rebuild_component failed after update_feature: {}", e);
    }
    if let Some(ref path) = state.save_path {
        if let Err(e) = p.save(path) {
            eprintln!("Auto-save failed: {}", e);
        }
    }
    Ok(Json(serde_json::json!({"success": true})))
}

async fn remove_feature_handler(
    Path((component_id, feature_id)): Path<(String, String)>,
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut p = state.project.write().await;
    p.remove_feature(&component_id, &feature_id)
        .map_err(|e| (StatusCode::NOT_FOUND, e))?;
    if let Err(e) = p.rebuild_component(&component_id) {
        eprintln!("rebuild_component failed after remove_feature: {}", e);
    }
    if let Some(ref path) = state.save_path {
        if let Err(e) = p.save(path) {
            eprintln!("Auto-save failed: {}", e);
        }
    }
    Ok(Json(serde_json::json!({"success": true})))
}

async fn rebuild_component_handler(
    Path(id): Path<String>,
    State(state): State<SharedState>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let mut p = state.project.write().await;
    p.rebuild_component(&id)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    if let Some(ref path) = state.save_path {
        if let Err(e) = p.save(path) {
            eprintln!("Auto-save failed: {}", e);
        }
    }
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
    Json(req): Json<CreateComponentDefRequest>,
) -> Json<omnim_model::catalog::ComponentDef> {
    let mut comp = omnim_model::catalog::ComponentDef::new(&req.name, &req.category);
    comp.description = req.description;
    comp.ifc_class = req.ifc_class;
    comp.standard_reference = req.standard_reference;
    comp.tags = req.tags;
    comp.domain = req.domain;
    let comp_clone = comp.clone();
    LIBRARY.write().unwrap().as_mut().unwrap().catalog.insert(comp);
    Json(comp_clone)
}

async fn get_component_def(
    Path(id): Path<String>,
) -> Result<Json<omnim_model::catalog::ComponentDef>, StatusCode> {
    LIBRARY.read().unwrap().as_ref().unwrap().catalog.get(&id).cloned().map(Json).ok_or(StatusCode::NOT_FOUND)
}

async fn delete_component_def(
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    match LIBRARY.write().unwrap().as_mut().unwrap().catalog.remove(&id) {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(StatusCode::NOT_FOUND),
    }
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
    Path(id): Path<String>,
    Json(req): Json<AddParamRequest>,
) -> Result<StatusCode, StatusCode> {
    let mut library = LIBRARY.write().unwrap();
    let library = library.as_mut().unwrap();
    let comp = library.catalog.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    comp.params.push(omnim_model::catalog::ParamDef {
        name: req.name,
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
}

fn default_dir_z() -> String { "1".into() }

async fn add_connector(
    Path(id): Path<String>,
    Json(req): Json<AddConnectorRequest>,
) -> Result<StatusCode, StatusCode> {
    let mut library = LIBRARY.write().unwrap();
    let library = library.as_mut().unwrap();
    let comp = library.catalog.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    comp.connectors.push(omnim_model::catalog::ConnectorDef {
        id: req.connector_id,
        name: req.connector_name,
        position_x_expr: req.position_x_expr,
        position_y_expr: req.position_y_expr,
        position_z_expr: req.position_z_expr,
        direction_x_expr: req.direction_x_expr,
        direction_y_expr: req.direction_y_expr,
        direction_z_expr: req.direction_z_expr,
        bore_expr: req.bore_expr,
        connection_type_code: req.connection_type_code,
        role: req.role,
    });
    Ok(StatusCode::CREATED)
}

async fn publish_component(
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let mut library = LIBRARY.write().unwrap();
    let library = library.as_mut().unwrap();
    let comp = library.catalog.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    comp.published = true;
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
    Path(id): Path<String>,
    Json(req): Json<UpdateComponentDefRequest>,
) -> Result<StatusCode, StatusCode> {
    let mut library = LIBRARY.write().unwrap();
    let library = library.as_mut().unwrap();
    let comp = library.catalog.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    if let Some(name) = req.name { comp.name = name; }
    if let Some(cat) = req.category { comp.category = cat; }
    if let Some(desc) = req.description { comp.description = desc; }
    if let Some(cls) = req.ifc_class { comp.ifc_class = cls; }
    if let Some(std) = req.standard_reference { comp.standard_reference = std; }
    if let Some(tags) = req.tags { comp.tags = tags; }
    if let Some(domain) = req.domain { comp.domain = domain; }
    if let Some(mat) = req.allowed_materials { comp.allowed_materials = mat; }
    if let Some(def_mat) = req.default_material { comp.default_material = def_mat; }
    if let Some(props) = req.properties { comp.properties = props; }
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
    Path((id, idx)): Path<(String, usize)>,
    Json(req): Json<UpdateParamRequest>,
) -> Result<StatusCode, StatusCode> {
    let mut library = LIBRARY.write().unwrap();
    let library = library.as_mut().unwrap();
    let comp = library.catalog.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    let param = comp.params.get_mut(idx).ok_or(StatusCode::NOT_FOUND)?;
    if let Some(name) = req.name { param.name = name; }
    if let Some(pt) = req.param_type { param.param_type = pt; }
    if let Some(dv) = req.default_value { param.default_value = dv; }
    if let Some(min) = req.min_value { param.min_value = min; }
    if let Some(max) = req.max_value { param.max_value = max; }
    if let Some(st) = req.step { param.step = st; }
    if let Some(u) = req.unit { param.unit = u; }
    if let Some(f) = req.formula { param.formula = f; }
    if let Some(c) = req.choices { param.choices = c; }
    if let Some(d) = req.description { param.description = d; }
    Ok(StatusCode::OK)
}

async fn delete_component_param(
    Path((id, idx)): Path<(String, usize)>,
) -> Result<StatusCode, StatusCode> {
    let mut library = LIBRARY.write().unwrap();
    let library = library.as_mut().unwrap();
    let comp = library.catalog.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    if idx < comp.params.len() {
        comp.params.remove(idx);
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

#[derive(Deserialize)]
struct AssignFeatureTreeRequest {
    feature_tree_id: String,
}

async fn assign_feature_tree(
    Path(id): Path<String>,
    Json(req): Json<AssignFeatureTreeRequest>,
) -> Result<StatusCode, StatusCode> {
    let mut library = LIBRARY.write().unwrap();
    let library = library.as_mut().unwrap();
    let comp = library.catalog.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    comp.feature_tree_id = req.feature_tree_id;
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

async fn create_series_handler(
    State(state): State<SharedState>,
    Json(req): Json<CreateSeriesRequest>,
) -> Json<omnim_model::series_store::Series> {
    let mut p = state.project.write().await;
    let mut series = omnim_model::series_store::Series::new(&req.name);
    series.description = req.description;
    series.domain = req.domain;
    series.standard_reference = req.standard_reference;
    let s = series.clone();
    p.series.insert(series);
    Json(s)
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
) -> Result<StatusCode, StatusCode> {
    let mut p = state.project.write().await;
    let series = p.series.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    if let Some(name) = req.name { series.name = name; }
    if let Some(desc) = req.description { series.description = desc; }
    if let Some(domain) = req.domain { series.domain = domain; }
    if let Some(std) = req.standard_reference { series.standard_reference = std; }
    if let Some(phases) = req.applicable_phases { series.applicable_phases = phases; }
    if let Some(meta) = req.metadata { series.metadata = meta; }
    Ok(StatusCode::OK)
}

async fn delete_series_handler(
    State(state): State<SharedState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let mut p = state.project.write().await;
    match p.series.remove(&id) {
        Some(_) => Ok(StatusCode::NO_CONTENT),
        None => Err(StatusCode::NOT_FOUND),
    }
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
) -> Result<StatusCode, StatusCode> {
    let mut p = state.project.write().await;
    let series = p.series.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    let selector = omnim_model::series_store::Selector::new(&req.selector_id, &req.question);
    if series.root_selector.is_none() {
        series.root_selector = Some(selector);
    }
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
) -> Result<StatusCode, StatusCode> {
    let mut p = state.project.write().await;
    let series = p.series.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    let answer = omnim_model::series_store::Answer {
        id: req.answer_id,
        value: req.answer_value,
        is_numeric_range: req.is_numeric_range,
        range_min: req.range_min,
        range_max: req.range_max,
        outcome: omnim_model::series_store::AnswerOutcome::Component(
            omnim_model::series_store::SpecComponent::new("")
        ),
    };
    // Find the selector and add the answer
    if let Some(ref mut root) = series.root_selector {
        let mut stack = vec![root];
        while let Some(sel) = stack.pop() {
            if sel.id == req.selector_id {
                sel.answers.push(answer);
                return Ok(StatusCode::CREATED);
            }
            for ans in &mut sel.answers {
                if let omnim_model::series_store::AnswerOutcome::NextSelector(ref mut next) = ans.outcome {
                    stack.push(next);
                }
            }
        }
    }
    Err(StatusCode::NOT_FOUND)
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
) -> Result<StatusCode, StatusCode> {
    let mut p = state.project.write().await;
    let series = p.series.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    let spec = omnim_model::series_store::SpecComponent::new(&req.component_def_id)
        .material(&req.material)
        .connection_type_code(&req.connection_type_code);
    if let Some(ref mut root) = series.root_selector {
        let mut stack = vec![root];
        while let Some(sel) = stack.pop() {
            if sel.id == req.selector_id {
                for ans in &mut sel.answers {
                    if ans.id == req.answer_id {
                        ans.outcome = omnim_model::series_store::AnswerOutcome::Component(spec);
                        return Ok(StatusCode::CREATED);
                    }
                }
            }
            for ans in &mut sel.answers {
                if let omnim_model::series_store::AnswerOutcome::NextSelector(ref mut next) = ans.outcome {
                    stack.push(next);
                }
            }
        }
    }
    Err(StatusCode::NOT_FOUND)
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
) -> Result<StatusCode, StatusCode> {
    let mut p = state.project.write().await;
    let series = p.series.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    let constraint = omnim_model::series_store::ParamConstraint::new(&req.param_name)
        .with_default(req.default_value)
        .with_range(req.min_value, req.max_value)
        .with_step(req.step)
        .with_fixed(&req.fixed_value);
    if let Some(ref mut root) = series.root_selector {
        let mut stack = vec![root];
        while let Some(sel) = stack.pop() {
            for ans in &mut sel.answers {
                match &mut ans.outcome {
                    omnim_model::series_store::AnswerOutcome::Component(ref mut sc) => {
                        if sc.component_def_id == req.component_def_id {
                            sc.param_constraints.push(constraint.clone());
                            return Ok(StatusCode::CREATED);
                        }
                    }
                    omnim_model::series_store::AnswerOutcome::NextSelector(ref mut next) => {
                        stack.push(next);
                    }
                }
            }
        }
    }
    Err(StatusCode::NOT_FOUND)
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
) -> Result<StatusCode, StatusCode> {
    let mut p = state.project.write().await;
    let series = p.series.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    if let Some(ref mut root) = series.root_selector {
        let mut stack = vec![root];
        while let Some(sel) = stack.pop() {
            for ans in &mut sel.answers {
                match &mut ans.outcome {
                    omnim_model::series_store::AnswerOutcome::Component(ref mut sc) => {
                        if sc.component_def_id == req.component_def_id {
                            if let Some(c) = sc.param_constraints.get_mut(idx) {
                                if let Some(pn) = req.param_name { c.param_name = pn; }
                                if let Some(dv) = req.default_value { c.default_value = dv; }
                                if let Some(min) = req.min_value { c.min_value = min; }
                                if let Some(max) = req.max_value { c.max_value = max; }
                                if let Some(st) = req.step { c.step = st; }
                                if let Some(fv) = req.fixed_value { c.fixed_value = fv; }
                                return Ok(StatusCode::OK);
                            }
                            return Err(StatusCode::NOT_FOUND);
                        }
                    }
                    omnim_model::series_store::AnswerOutcome::NextSelector(ref mut next) => {
                        stack.push(next);
                    }
                }
            }
        }
    }
    Err(StatusCode::NOT_FOUND)
}

#[derive(Deserialize)]
struct DeleteParamConstraintRequest {
    component_def_id: String,
}

async fn delete_param_constraint_handler(
    State(state): State<SharedState>,
    Path((id, idx)): Path<(String, usize)>,
    Json(req): Json<DeleteParamConstraintRequest>,
) -> Result<StatusCode, StatusCode> {
    let mut p = state.project.write().await;
    let series = p.series.get_mut(&id).ok_or(StatusCode::NOT_FOUND)?;
    if let Some(ref mut root) = series.root_selector {
        let mut stack = vec![root];
        while let Some(sel) = stack.pop() {
            for ans in &mut sel.answers {
                match &mut ans.outcome {
                    omnim_model::series_store::AnswerOutcome::Component(ref mut sc) => {
                        if sc.component_def_id == req.component_def_id {
                            if idx < sc.param_constraints.len() {
                                sc.param_constraints.remove(idx);
                                return Ok(StatusCode::NO_CONTENT);
                            }
                            return Err(StatusCode::NOT_FOUND);
                        }
                    }
                    omnim_model::series_store::AnswerOutcome::NextSelector(ref mut next) => {
                        stack.push(next);
                    }
                }
            }
        }
    }
    Err(StatusCode::NOT_FOUND)
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
    let mut p = state.project.write().await;
    let ext_store = omnim_model::series_store::SeriesStore::load_from_file(&req.path)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;
    let count = ext_store.len();
    let mut names = Vec::new();
    for s in ext_store.list() {
        let id = s.id.clone();
        let name = s.name.clone();
        names.push(name.clone());
        p.series.insert(s.clone());
        let already_ref = p.external_series_refs.iter().any(|r| r.id == id);
        if !already_ref {
            p.external_series_refs.push(omnim_model::SeriesRef {
                id,
                name,
                source: req.source.clone(),
                path: Some(req.path.clone()),
            });
        }
    }
    Ok(Json(LoadSeriesResponse {
        loaded_count: count,
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
    let mut p = state.project.write().await;
    let component_id = if req.skip_series_gate {
        p.instantiate_catalog_component_skip_gate(
            &req.component_def_id,
            &req.instance_name,
            &req.params,
            req.position,
            req.rotation_z,
        )
    } else {
        p.instantiate_catalog_component(
            &req.component_def_id,
            &req.instance_name,
            &req.params,
            req.position,
            req.rotation_z,
        )
    }.map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let obj = p.objects.get(&component_id)
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "created component not found".into()))?;

    Ok(Json(InstantiateResponse {
        component_id: obj.id.clone(),
        object_type: format!("{:?}", c::ObjectType::try_from(obj.object_type).unwrap_or(c::ObjectType::Unspecified)),
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
    State(state): State<SharedState>,
    Json(req): Json<PreviewRequest>,
) -> Result<Json<c::MeshGeometry>, (StatusCode, String)> {
    let mut p = state.project.write().await;
    let preview_name = format!("__preview_{}", req.component_def_id);
    let component_id = p.instantiate_catalog_component(
        &req.component_def_id,
        &preview_name,
        &req.params,
        None,
        0.0,
    ).map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    let obj = p.objects.get(&component_id)
        .ok_or_else(|| (StatusCode::INTERNAL_SERVER_ERROR, "preview component not found".into()))?;

    // Extract mesh from the component's geometry
    let mesh = obj.geometry.as_ref()
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
    let mesh = omnim_model::geometry::scale_mesh(&mesh, 0.001);

    // Clean up the preview component
    p.objects.remove(&component_id);

    Ok(Json(mesh))
}
