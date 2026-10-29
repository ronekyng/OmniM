//! Wave 4 acceptance tests — series enforcement (audit P0-5):
//! parameter constraints, spec material application, and connect() hardening.

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
    let project: SharedProject = Arc::new(RwLock::new(Project::new("Enforcement Test")));
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

async fn post_json(port: u16, path: &str, body: serde_json::Value) -> (u16, String) {
    let client = reqwest::Client::new();
    let resp = client
        .post(api_url(port, path))
        .json(&body)
        .send()
        .await
        .unwrap();
    (resp.status().as_u16(), resp.text().await.unwrap())
}

async fn get_json(port: u16, path: &str) -> serde_json::Value {
    let client = reqwest::Client::new();
    client
        .get(api_url(port, path))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

/// Build a series that constrains `Concrete-Beam-Rect` (seeded def with a
/// feature tree): material Zinc + width min/max + depth fixed.
async fn build_beam_series(port: u16) -> String {
    let (status, _) = post_json(port, "/api/materials", serde_json::json!({
        "id": "mat-zinc", "name": "Zinc", "category": "metal", "density": 7140.0
    })).await;
    assert!(status == 200 || status == 400, "material create failed: {}", status);

    let (status, body) = post_json(port, "/api/series", serde_json::json!({
        "name": "Enforcement-Series", "domain": "building"
    })).await;
    assert_eq!(status, 200, "series create failed: {}", body);
    let series_id = json_of(&body)["id"].as_str().unwrap().to_string();

    for (path, body) in [
        ("/api/series/{}/selectors", serde_json::json!({
            "selector_id": "s1", "question": "Pick a beam"
        })),
        ("/api/series/{}/answers", serde_json::json!({
            "selector_id": "s1", "answer_id": "a1", "answer_value": "beam"
        })),
        ("/api/series/{}/spec", serde_json::json!({
            "selector_id": "s1", "answer_id": "a1",
            "component_def_id": "Concrete-Beam-Rect", "material": "Zinc"
        })),
        ("/api/series/{}/constraints", serde_json::json!({
            "component_def_id": "Concrete-Beam-Rect", "param_name": "width",
            "default_value": 300.0, "min_value": 200.0, "max_value": 600.0
        })),
        ("/api/series/{}/constraints", serde_json::json!({
            "component_def_id": "Concrete-Beam-Rect", "param_name": "depth",
            "default_value": 500.0, "fixed_value": "500"
        })),
    ] {
        let (status, body) = post_json(port, &path.replace("{}", &series_id), body).await;
        assert!(status == 200 || status == 201, "series setup failed: {} {}", status, body);
    }
    series_id
}

fn json_of(body: &str) -> serde_json::Value {
    serde_json::from_str(body).unwrap_or(serde_json::Value::Null)
}

#[tokio::test]
async fn param_override_above_series_max_is_rejected() {
    let (port, _) = test_server().await;
    build_beam_series(port).await;
    let (status, body) = post_json(port, "/api/catalog/instantiate", serde_json::json!({
        "component_def_id": "Concrete-Beam-Rect",
        "instance_name": "B-too-wide",
        "params": { "width": 900.0 }
    })).await;
    assert_eq!(status, 400, "expected max enforcement, got: {}", body);
    assert!(body.contains("exceeds the series maximum 600"), "unexpected error: {}", body);
}

#[tokio::test]
async fn param_override_below_series_min_is_rejected() {
    let (port, _) = test_server().await;
    build_beam_series(port).await;
    let (status, body) = post_json(port, "/api/catalog/instantiate", serde_json::json!({
        "component_def_id": "Concrete-Beam-Rect",
        "instance_name": "B-too-narrow",
        "params": { "width": 100.0 }
    })).await;
    assert_eq!(status, 400);
    assert!(body.contains("below the series minimum 200"), "unexpected error: {}", body);
}

#[tokio::test]
async fn fixed_param_rejects_any_other_value() {
    let (port, _) = test_server().await;
    build_beam_series(port).await;
    let (status, body) = post_json(port, "/api/catalog/instantiate", serde_json::json!({
        "component_def_id": "Concrete-Beam-Rect",
        "instance_name": "B-depth",
        "params": { "depth": 400.0 }
    })).await;
    assert_eq!(status, 400);
    assert!(body.contains("fixed to 500"), "unexpected error: {}", body);
}

#[tokio::test]
async fn spec_material_is_applied_to_the_instance() {
    let (port, _) = test_server().await;
    build_beam_series(port).await;
    let (status, body) = post_json(port, "/api/catalog/instantiate", serde_json::json!({
        "component_def_id": "Concrete-Beam-Rect",
        "instance_name": "B-ok",
        "params": { "width": 300.0, "depth": 500.0 }
    })).await;
    assert_eq!(status, 200, "instantiate failed: {}", body);
    let id = json_of(&body)["component_id"].as_str().unwrap().to_string();
    let obj = get_json(port, &format!("/api/objects/{}", id)).await;
    assert_eq!(obj["material_id"], "mat-zinc", "spec material was not applied");
}

#[tokio::test]
async fn unknown_spec_material_is_an_error() {
    let (port, _) = test_server().await;
    // Series with a material that does not exist in the project.
    let (status, body) = post_json(port, "/api/series", serde_json::json!({
        "name": "Bad-Material-Series", "domain": "building"
    })).await;
    assert_eq!(status, 200, "{}", body);
    let series_id = json_of(&body)["id"].as_str().unwrap().to_string();
    for (path, body) in [
        ("/api/series/{}/selectors", serde_json::json!({
            "selector_id": "s1", "question": "Pick"
        })),
        ("/api/series/{}/answers", serde_json::json!({
            "selector_id": "s1", "answer_id": "a1", "answer_value": "x"
        })),
        ("/api/series/{}/spec", serde_json::json!({
            "selector_id": "s1", "answer_id": "a1",
            "component_def_id": "Concrete-Beam-Rect", "material": "Unobtainium"
        })),
    ] {
        let (status, body) = post_json(port, &path.replace("{}", &series_id), body).await;
        assert!(status == 200 || status == 201, "{} {}", status, body);
    }
    let (status, body) = post_json(port, "/api/catalog/instantiate", serde_json::json!({
        "component_def_id": "Concrete-Beam-Rect",
        "instance_name": "B-nomaterial"
    })).await;
    assert_eq!(status, 400);
    assert!(body.contains("series material 'Unobtainium' not found"), "unexpected error: {}", body);
}

#[tokio::test]
async fn no_series_loaded_means_no_enforcement() {
    let (port, _) = test_server().await;
    // Same def, no series in the project: legacy behavior (def defaults only).
    let (status, _) = post_json(port, "/api/catalog/instantiate", serde_json::json!({
        "component_def_id": "Concrete-Beam-Rect",
        "instance_name": "B-free",
        "params": { "width": 900.0 }
    })).await;
    assert_eq!(status, 200, "def defaults must not be series-enforced without a series");
}

#[tokio::test]
async fn connect_rejects_self_duplicate_and_unknown_type() {
    let (port, _) = test_server().await;
    let client = reqwest::Client::new();
    let mut ids = Vec::new();
    for name in ["CA", "CB"] {
        let resp = client
            .post(api_url(port, "/api/objects"))
            .json(&serde_json::json!({ "object_type": "wall", "name": name }))
            .send()
            .await
            .unwrap();
        ids.push(resp.json::<serde_json::Value>().await.unwrap()["id"].as_str().unwrap().to_string());
    }
    let (a, b) = (ids[0].clone(), ids[1].clone());

    // Self-connection
    let (status, body) = post_json(port, &format!("/api/objects/{}/connect/{}", a, a), serde_json::json!({})).await;
    assert_eq!(status, 400, "{}", body);
    assert!(body.contains("itself"));

    // First connect succeeds
    let (status, body) = post_json(port, &format!("/api/objects/{}/connect/{}", a, b), serde_json::json!({})).await;
    assert_eq!(status, 200, "{}", body);

    // Duplicate (either direction)
    let (status, body) = post_json(port, &format!("/api/objects/{}/connect/{}", b, a), serde_json::json!({})).await;
    assert_eq!(status, 400, "{}", body);
    assert!(body.contains("already connected"));

    // Unknown connection type (valid unconnected pair)
    let (status, _) = post_json(port, "/api/objects", serde_json::json!({
        "object_type": "wall", "name": "CC"
    })).await;
    assert_eq!(status, 200);
    let (status, body) = post_json(port, &format!("/api/objects/{}/connect/{}?connection_type=warp", b, a), serde_json::json!({})).await;
    assert_eq!(status, 400, "{}", body);
    assert!(body.contains("unknown connection type"), "unexpected: {}", body);
}

#[tokio::test]
async fn connect_checks_connector_type_code_compatibility() {
    let (port, _) = test_server().await;
    let client = reqwest::Client::new();

    // Find a seeded def that has a feature tree (so it can instantiate).
    let defs = get_json(port, "/api/catalog").await;
    let def = defs.as_array().unwrap().iter()
        .find(|d| d["name"] == "Concrete-Footing-Isolated")
        .expect("seeded footing def");
    let def_id = def["id"].as_str().unwrap().to_string();
    let tree_id = def["feature_tree_id"].as_str().unwrap().to_string();

    // Give it a typed connector.
    let (status, body) = post_json(port, &format!("/api/catalog/{}/connectors", def_id), serde_json::json!({
        "connector_id": "c-flange", "connector_name": "Flange",
        "connection_type_code": "flange-dn50"
    })).await;
    assert!(status == 200 || status == 201, "connector add failed: {}", body);

    let instantiate = |port: u16, name: String, def_for: String| {
        let def_id = def_for;
        async move {
            let client = reqwest::Client::new();
            let resp = client
                .post(api_url(port, "/api/catalog/instantiate"))
                .json(&serde_json::json!({
                    "component_def_id": def_id,
                    "instance_name": name,
                    "skip_series_gate": true
                }))
                .send()
                .await
                .unwrap();
            (resp.status().as_u16(), resp.text().await.unwrap())
        }
    };

    let (s1, b1) = instantiate(port, "Inst-A".into(), def_id.clone()).await;
    assert_eq!(s1, 200, "{}", b1);
    let (s2, b2) = instantiate(port, "Inst-B".into(), def_id.clone()).await;
    assert_eq!(s2, 200, "{}", b2);
    let a = json_of(&b1)["component_id"].as_str().unwrap().to_string();
    let b = json_of(&b2)["component_id"].as_str().unwrap().to_string();

    // Compatible (same def → same code set)
    let (status, body) = post_json(port, &format!("/api/objects/{}/connect/{}", a, b), serde_json::json!({})).await;
    assert_eq!(status, 200, "{}", body);

    // Incompatible: second def reuses the tree but advertises a different code.
    let (status, body) = post_json(port, "/api/catalog", serde_json::json!({
        "name": "Weld-Footing", "category": "metal"
    })).await;
    assert_eq!(status, 200, "{}", body);
    let other_id = json_of(&body)["id"].as_str().unwrap().to_string();
    let resp = client
        .put(api_url(port, &format!("/api/catalog/{}/feature-tree", other_id)))
        .json(&serde_json::json!({ "feature_tree_id": tree_id }))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap();
    assert!(status == 200 || status == 204, "assign tree failed: {} {}", status, body);
    let (status, body) = post_json(port, &format!("/api/catalog/{}/connectors", other_id), serde_json::json!({
        "connector_id": "c-weld", "connector_name": "Weld",
        "connection_type_code": "weld"
    })).await;
    assert!(status == 200 || status == 201, "{}", body);

    let (s3, b3) = instantiate(port, "Inst-C".into(), other_id.clone()).await;
    assert_eq!(s3, 200, "{}", b3);
    let c = json_of(&b3)["component_id"].as_str().unwrap().to_string();

    let (status, body) = post_json(port, &format!("/api/objects/{}/connect/{}", a, c), serde_json::json!({})).await;
    assert_eq!(status, 400);
    assert!(body.contains("incompatible"), "unexpected error: {}", body);
}
