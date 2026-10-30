//! Wave 2 seed cases — `path { points, closed }` contract, legacy aliases and
//! the `(objectType, form)` authorization matrix (DATA_MODEL §1).

use std::sync::Arc;

use omnim_model::Project;
use omnim_server::{create_router, SharedProject};
use tokio::sync::RwLock;

fn init_library() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        *omnim_model::LIBRARY.write().unwrap() = Some(omnim_model::Library::seeded());
    });
}

async fn test_server() -> (u16, SharedProject) {
    init_library();
    let project: SharedProject = Arc::new(RwLock::new(Project::new("Contract Test")));
    let router = create_router(project.clone(), None);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    (port, project)
}

fn api_url(port: u16, path: &str) -> String {
    format!("http://127.0.0.1:{}{}", port, path)
}

/// Returns (status, body). Success bodies are JSON objects; error bodies are
/// plain strings (the handler's error type is `(StatusCode, String)`).
async fn genrule(port: u16, body: serde_json::Value) -> (u16, String) {
    let client = reqwest::Client::new();
    let resp = client
        .post(api_url(port, "/api/genrule"))
        .json(&body)
        .send()
        .await
        .unwrap();
    (resp.status().as_u16(), resp.text().await.unwrap())
}

async fn undo(port: u16) {
    let client = reqwest::Client::new();
    let resp = client
        .post(api_url(port, "/api/undo"))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
}

fn json_of(body: &str) -> serde_json::Value {
    serde_json::from_str(body).unwrap_or(serde_json::Value::Null)
}

fn layers() -> serde_json::Value {
    serde_json::json!([
        { "name": "core", "thickness": 0.15, "material_id": "concrete-c30" },
        { "name": "finish", "thickness": 0.02 }
    ])
}

#[tokio::test]
async fn wall_panel_via_contract_path() {
    let (port, _) = test_server().await;
    let (status, obj) = genrule(
        port,
        serde_json::json!({
            "genrule": "layered",
            "mode": "wall",
            "name": "W-contract",
            "height": 3.0,
            "path": { "points": [
                [0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [4.0, 3.0, 0.0]
            ], "closed": false },
            "layers": layers()
        }),
    )
    .await;
    assert_eq!(status, 200, "contract wall failed: {}", obj);
    let obj = json_of(&obj);
    assert_eq!(
        obj["properties"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["name"] == "genesis_class"
                && p["value"]["value_type"]["StringValue"] == "Surface"),
        true
    );
    assert_eq!(
        obj["properties"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["name"] == "genesis_form"
                && p["value"]["value_type"]["StringValue"] == "panel"),
        true
    );
}

#[tokio::test]
async fn path_wall_mesh_keeps_metre_scale() {
    // Regression (2026-10-30): path-driven walls carry a `catalog_ref` for
    // traceability but their geometry is metres. The mesh endpoint must not
    // shrink them by 0.001 (that made new walls/slabs invisible).
    let (port, _) = test_server().await;
    let (status, obj) = genrule(
        port,
        serde_json::json!({
            "genrule": "layered",
            "mode": "wall",
            "name": "W-mesh",
            "height": 3.0,
            "path": { "points": [[0.0, 0.0, 0.0], [5.0, 0.0, 0.0]], "closed": false },
            "layers": layers()
        }),
    )
    .await;
    assert_eq!(status, 200, "contract wall failed: {}", obj);
    let id = json_of(&obj)["id"].as_str().unwrap().to_string();

    let mesh: serde_json::Value = reqwest::Client::new()
        .get(api_url(port, "/api/mesh"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let entry = mesh["objects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["id"] == id.as_str())
        .expect("wall mesh entry");
    let min = entry["bbox_min"].as_array().unwrap();
    let max = entry["bbox_max"].as_array().unwrap();
    let span = |axis: usize| max[axis].as_f64().unwrap() - min[axis].as_f64().unwrap();
    assert!(
        span(0) > 4.0,
        "wall length should stay ~5 m, got {}",
        span(0)
    );
    assert!(
        span(2) > 2.0,
        "wall height should stay ~3 m, got {}",
        span(2)
    );
}

#[tokio::test]
async fn linear_catalog_part_rotation_lays_beam_along_segment() {
    // Regression (2026-10-30): beam templates extrude along local +Z. With only
    // a Z yaw the beam stayed vertical; the explicit `rotation` quaternion must
    // lay it along the picked (horizontal) segment.
    use omnim_model::LIBRARY;
    let (port, _) = test_server().await;
    let beam_def_id = {
        let lib = LIBRARY.read().unwrap();
        lib.as_ref()
            .unwrap()
            .catalog
            .find_by_name("Concrete-Beam-Rect")
            .unwrap()
            .id
            .clone()
    };
    // +Z → +X: 90° about +Y.
    let s = std::f64::consts::FRAC_1_SQRT_2;
    let resp = reqwest::Client::new()
        .post(api_url(port, "/api/catalog/instantiate"))
        .json(&serde_json::json!({
            "component_def_id": beam_def_id,
            "instance_name": "Beam-X",
            "params": { "length": 4000.0, "width": 300.0, "depth": 500.0 },
            "position": { "x": 0.0, "y": 0.0, "z": 0.0 },
            "rotation": { "x": 0.0, "y": s, "z": 0.0, "w": s },
            "skip_series_gate": true
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
    let body: serde_json::Value = resp.json().await.unwrap();
    let id = body["component_id"].as_str().unwrap().to_string();

    let mesh: serde_json::Value = reqwest::Client::new()
        .get(api_url(port, "/api/mesh"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let entry = mesh["objects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o["id"] == id.as_str())
        .expect("beam mesh entry");
    let min = entry["bbox_min"].as_array().unwrap();
    let max = entry["bbox_max"].as_array().unwrap();
    let span = |axis: usize| max[axis].as_f64().unwrap() - min[axis].as_f64().unwrap();
    assert!(
        span(0) > 3.5,
        "beam length should lie along X (~4 m), got {}",
        span(0)
    );
    assert!(
        span(2) < 1.0,
        "beam vertical extent should be the section depth (<1 m), got {}",
        span(2)
    );
}

#[tokio::test]
async fn wall_panel_via_legacy_path_alias() {
    let (port, _) = test_server().await;
    let (status, obj) = genrule(
        port,
        serde_json::json!({
            "genrule": "layered",
            "mode": "wall",
            "name": "W-legacy",
            "height": 3.0,
            "path": [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0]],
            "layers": layers()
        }),
    )
    .await;
    assert_eq!(status, 200, "legacy path alias failed: {}", obj);
}

#[tokio::test]
async fn wall_region_via_closed_contract() {
    let (port, _) = test_server().await;
    let (status, obj) = genrule(
        port,
        serde_json::json!({
            "genrule": "layered",
            "mode": "wall",
            "name": "W-region",
            "height": 3.0,
            "boundary": { "points": [
                [0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [4.0, 0.2, 0.0], [0.0, 0.2, 0.0]
            ], "closed": true },
            "layers": layers()
        }),
    )
    .await;
    assert_eq!(status, 200, "wall region failed: {}", obj);
    let obj = json_of(&obj);
    assert_eq!(
        obj["properties"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["name"] == "genesis_form"
                && p["value"]["value_type"]["StringValue"] == "region"),
        true
    );
}

#[tokio::test]
async fn slab_region_ok_but_panel_rejected_by_matrix() {
    let (port, _) = test_server().await;
    let (status, obj) = genrule(
        port,
        serde_json::json!({
            "genrule": "layered",
            "mode": "slab",
            "name": "S-region",
            "boundary": { "points": [
                [0.0, 0.0, 0.0], [5.0, 0.0, 0.0], [5.0, 4.0, 0.0], [0.0, 4.0, 0.0]
            ], "closed": true },
            "layers": layers()
        }),
    )
    .await;
    assert_eq!(status, 200, "slab region failed: {}", obj);

    // Slab has no open-path form — the matrix rejects it with the contract reason.
    let (status, obj) = genrule(
        port,
        serde_json::json!({
            "genrule": "layered",
            "mode": "slab",
            "name": "S-panel",
            "path": { "points": [[0.0, 0.0, 0.0], [5.0, 0.0, 0.0]], "closed": false },
            "layers": layers()
        }),
    )
    .await;
    assert_eq!(status, 400);
    assert!(
        obj.contains("creation method 'panel' is not authorized for object type 'slab'"),
        "unexpected error: {}",
        obj
    );
}

#[tokio::test]
async fn roof_profile_requires_exactly_two_points() {
    let (port, _) = test_server().await;
    let (status, obj) = genrule(
        port,
        serde_json::json!({
            "genrule": "layered",
            "mode": "roof",
            "name": "R-profile",
            "width": 3.0,
            "path": { "points": [[0.0, 0.0, 2.0], [6.0, 0.0, 2.0]], "closed": false },
            "layers": layers()
        }),
    )
    .await;
    assert_eq!(status, 200, "roof profile failed: {}", obj);

    let (status, obj) = genrule(port, serde_json::json!({
        "genrule": "layered",
        "mode": "roof",
        "name": "R-bad",
        "width": 3.0,
        "path": { "points": [[0.0, 0.0, 2.0], [6.0, 0.0, 2.0], [7.0, 0.0, 2.0]], "closed": false },
        "layers": layers()
    })).await;
    assert_eq!(status, 400);
    assert!(
        obj.contains("exactly 2 distinct points"),
        "unexpected error: {}",
        obj
    );
}

#[tokio::test]
async fn road_accepts_contract_and_legacy_alias_but_not_closed() {
    let (port, _) = test_server().await;
    // Contract form
    let (status, obj) = genrule(port, serde_json::json!({
        "genrule": "road",
        "name": "RD-contract",
        "width": 6.0, "thickness": 0.4, "length": 10.0,
        "path": { "points": [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [10.0, 8.0, 0.0]], "closed": false }
    })).await;
    assert_eq!(status, 200, "road contract failed: {}", obj);
    let obj = json_of(&obj);
    assert_eq!(
        obj["properties"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["name"] == "genesis_class"
                && p["value"]["value_type"]["StringValue"] == "Linear"),
        true
    );

    // Legacy alias
    let (status, _) = genrule(
        port,
        serde_json::json!({
            "genrule": "road",
            "name": "RD-legacy",
            "width": 6.0, "thickness": 0.4, "length": 10.0,
            "alignment": [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]]
        }),
    )
    .await;
    assert_eq!(status, 200);

    // Road is linear, not a region
    let (status, obj) = genrule(port, serde_json::json!({
        "genrule": "road",
        "name": "RD-closed",
        "width": 6.0, "thickness": 0.4, "length": 10.0,
        "path": { "points": [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0], [10.0, 8.0, 0.0], [0.0, 0.0, 0.0]], "closed": true }
    })).await;
    assert_eq!(status, 400);
    assert!(obj.contains("must be open"), "unexpected error: {}", obj);
}

#[tokio::test]
async fn line_guide_via_contract_and_rejection_of_closed() {
    let (port, _) = test_server().await;
    let (status, obj) = genrule(port, serde_json::json!({
        "genrule": "line",
        "name": "FL-contract",
        "path": { "points": [[0.0, 0.0, 0.0], [3.0, 1.0, 0.5], [6.0, 0.0, 1.0]], "closed": false }
    })).await;
    assert_eq!(status, 200, "line contract failed: {}", obj);
    let obj = json_of(&obj);
    assert_eq!(
        obj["properties"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["name"] == "genesis_class"
                && p["value"]["value_type"]["StringValue"] == "Point"),
        true
    );

    let (status, _) = genrule(port, serde_json::json!({
        "genrule": "line",
        "name": "FL-closed",
        "path": { "points": [[0.0, 0.0, 0.0], [3.0, 1.0, 0.5], [0.0, 0.0, 0.0]], "closed": true }
    })).await;
    assert_eq!(status, 400);
}

#[tokio::test]
async fn ambiguous_input_is_rejected() {
    let (port, _) = test_server().await;
    let (status, obj) = genrule(
        port,
        serde_json::json!({
            "genrule": "layered",
            "mode": "wall",
            "name": "W-both",
            "height": 3.0,
            "path": [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0]],
            "boundary": [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [4.0, 0.2, 0.0], [0.0, 0.2, 0.0]],
            "layers": layers()
        }),
    )
    .await;
    assert_eq!(status, 400);
    assert!(obj.contains("not both"), "unexpected error: {}", obj);

    let (status, obj) = genrule(
        port,
        serde_json::json!({
            "genrule": "road",
            "name": "RD-both",
            "width": 6.0, "thickness": 0.4, "length": 10.0,
            "path": { "points": [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]], "closed": false },
            "alignment": [[0.0, 0.0, 0.0], [10.0, 0.0, 0.0]]
        }),
    )
    .await;
    assert_eq!(status, 400);
    assert!(obj.contains("not both"), "unexpected error: {}", obj);
}

#[tokio::test]
async fn closed_wall_boundary_with_bowtie_is_rejected() {
    let (port, _) = test_server().await;
    let (status, obj) = genrule(
        port,
        serde_json::json!({
            "genrule": "layered",
            "mode": "wall",
            "name": "W-bowtie",
            "height": 3.0,
            "boundary": { "points": [
                [0.0, 0.0, 0.0], [4.0, 4.0, 0.0], [4.0, 0.0, 0.0], [0.0, 4.0, 0.0]
            ], "closed": true },
            "layers": layers()
        }),
    )
    .await;
    assert_eq!(status, 400);
    // The validation stack may fire on degenerate area or self-intersection;
    // both are polygon rejections.
    assert!(
        obj.contains("self-intersecting") || obj.contains("degenerate"),
        "unexpected error: {}",
        obj
    );
}

// ── Wave 3: geometry-path convergence (path-driven ⇒ domain type + features) ──

#[tokio::test]
async fn path_driven_wall_is_domain_typed_with_feature_tree() {
    let (port, project) = test_server().await;
    let (status, body) = genrule(
        port,
        serde_json::json!({
            "genrule": "layered",
            "mode": "wall",
            "name": "W-conv",
            "height": 3.0,
            "path": { "points": [[0.0, 0.0, 0.0], [4.0, 0.0, 0.0]], "closed": false },
            "layers": layers()
        }),
    )
    .await;
    assert_eq!(status, 200, "converged wall failed: {}", body);
    let obj = json_of(&body);

    // Domain object type, NOT ObjectType::Component.
    assert_eq!(obj["object_type"], 100, "wall must be ObjectType::Wall");

    // Feature tree attached: one PathSweep feature with cached geometry.
    let features = obj["features"].as_array().expect("features array");
    assert_eq!(features.len(), 1);
    assert_eq!(features[0]["feature_type"], "PathSweep");
    assert!(
        features[0]["cached_geometry"].is_object(),
        "feature must carry cached geometry"
    );

    // Catalog traceability.
    let props = obj["properties"].as_array().unwrap();
    // catalog_component_id is the def's UUID; the name is the stable handle.
    assert!(props.iter().any(|p| p["name"] == "catalog_component_id"));
    assert!(props.iter().any(|p| p["name"] == "catalog_component_name"
        && p["value"]["value_type"]["StringValue"] == "Wall-Path"));

    // Genesis recorded.
    assert!(
        props
            .iter()
            .any(|p| p["name"] == "genesis_form"
                && p["value"]["value_type"]["StringValue"] == "panel")
    );

    // Non-empty geometry on the object itself (rebuild merged the feature).
    assert!(
        obj["geometry"]["representation"].is_object(),
        "wall must have geometry"
    );

    // Undo removes it (it went through record_create on the executor).
    undo(port).await;
    let p = project.read().await;
    assert!(
        p.objects.get(obj["id"].as_str().unwrap()).is_none(),
        "converged wall must be undoable"
    );
}

#[tokio::test]
async fn slab_via_closed_contract_is_domain_typed() {
    let (port, _) = test_server().await;
    let (status, body) = genrule(
        port,
        serde_json::json!({
            "genrule": "layered",
            "mode": "slab",
            "name": "S-conv",
            "boundary": { "points": [
                [0.0, 0.0, 1.0], [5.0, 0.0, 1.0], [5.0, 4.0, 1.0], [0.0, 4.0, 1.0]
            ], "closed": true },
            "layers": layers()
        }),
    )
    .await;
    assert_eq!(status, 200, "converged slab failed: {}", body);
    let obj = json_of(&body);
    assert_eq!(obj["object_type"], 103, "slab must be ObjectType::Slab");
    assert_eq!(obj["features"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn invalid_path_fails_the_instantiation_with_kernel_reason() {
    let (port, _) = test_server().await;
    let (status, body) = genrule(port, serde_json::json!({
        "genrule": "layered",
        "mode": "wall",
        "name": "W-bad",
        "height": 3.0,
        "path": { "points": [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]], "closed": false },
        "layers": layers()
    })).await;
    assert_eq!(status, 400);
    assert!(body.contains("zero-length"), "unexpected error: {}", body);
}
