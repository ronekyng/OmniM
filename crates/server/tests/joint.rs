//! Phase 8-4 (ASSEMBLY §9): motion joint endpoint + cascade + undo coverage.

use std::sync::Arc;
use std::sync::Once;

use omnim_model::Project;
use omnim_server::{create_router, SharedProject};
use tokio::sync::RwLock;

fn init_library() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        *omnim_model::LIBRARY.write().unwrap() = Some(omnim_model::Library::seeded());
    });
}

async fn test_server() -> (u16, SharedProject) {
    init_library();
    let project: SharedProject = Arc::new(RwLock::new(Project::new("Joint Test")));
    let router = create_router(project.clone(), None);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    (port, project)
}

fn url(port: u16, path: &str) -> String {
    format!("http://127.0.0.1:{}{}", port, path)
}

async fn create_assembly(port: u16) -> serde_json::Value {
    let client = reqwest::Client::new();
    let resp = client
        .post(url(port, "/api/assemblies"))
        .json(&serde_json::json!({
            "name": "Hinge ASM",
            "skip_series_gate": true,
            "parts": [
                { "name": "Leaf-A", "shape": "box", "p1": 0.4, "p2": 0.3, "p3": 0.2 },
                { "name": "Leaf-B", "shape": "box", "p1": 0.4, "p2": 0.3, "p3": 0.2 }
            ]
        }))
        .send()
        .await
        .unwrap();
    let text = resp.text().await.unwrap();
    serde_json::from_str(&text).unwrap_or_else(|_| panic!("assembly failed: {}", text))
}

async fn create_joint(port: u16, body: serde_json::Value) -> (u16, serde_json::Value) {
    let client = reqwest::Client::new();
    let resp = client
        .post(url(port, "/api/joints"))
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap();
    (
        status,
        serde_json::from_str(&body).unwrap_or(serde_json::Value::String(body.clone())),
    )
}

fn revolute(a: &str, b: &str) -> serde_json::Value {
    serde_json::json!({
        "name": "hinge",
        "kind": "revolute",
        "from_object_id": a,
        "to_object_id": b,
        "origin": { "x": 0.0, "y": 0.0, "z": 1.0 },
        "axis": { "x": 0.0, "y": 0.0, "z": 10.0 },
        "has_limits": true,
        "limit_min": 0.0,
        "limit_max": 2.0
    })
}

#[tokio::test]
async fn revolute_joint_crud_and_filter() {
    let (port, _) = test_server().await;
    let asm = create_assembly(port).await;
    let parts = asm["parts"].as_array().unwrap();
    let (a, b) = (
        parts[0]["id"].as_str().unwrap(),
        parts[1]["id"].as_str().unwrap(),
    );

    // Create: axis normalized to unit length.
    let (status, joint) = create_joint(port, revolute(a, b)).await;
    assert_eq!(status, 201, "{}", joint);
    let id = joint["id"].as_str().unwrap().to_string();
    assert_eq!(joint["kind"], "revolute");
    let axis = &joint["axis"];
    assert!((axis["z"].as_f64().unwrap() - 1.0).abs() < 1e-12);

    // List + filter.
    let filtered = reqwest::get(url(port, &format!("/api/joints?object_id={}", a)))
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let arr = filtered.as_array().unwrap();
    assert_eq!(arr.len(), 1, "{}", filtered);
    assert_eq!(arr[0]["id"], id.as_str());

    // Update: switch kind to prismatic with metre limits.
    let client = reqwest::Client::new();
    let updated = client
        .put(url(port, &format!("/api/joints/{}", id)))
        .json(&serde_json::json!({
            "name": "slider",
            "kind": "prismatic",
            "from_object_id": a,
            "to_object_id": b,
            "origin": { "x": 0.0, "y": 0.0, "z": 1.0 },
            "axis": { "x": 0.0, "y": 1.0, "z": 0.0 },
            "has_limits": true,
            "limit_min": -0.5,
            "limit_max": 0.5
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(updated.status().as_u16(), 200);
    let up_json: serde_json::Value = updated.json().await.unwrap();
    assert_eq!(up_json["kind"], "prismatic");
    assert_eq!(up_json["limit_max"], 0.5);

    // Delete + 404 afterwards.
    let deleted = client
        .delete(url(port, &format!("/api/joints/{}", id)))
        .send()
        .await
        .unwrap();
    assert_eq!(deleted.status().as_u16(), 204);
    let gone = client
        .delete(url(port, &format!("/api/joints/{}", id)))
        .send()
        .await
        .unwrap();
    assert_eq!(gone.status().as_u16(), 404);
}

#[tokio::test]
async fn joint_validation_rejects_bad_requests() {
    let (port, _) = test_server().await;
    let asm = create_assembly(port).await;
    let parts = asm["parts"].as_array().unwrap();
    let (a, b) = (
        parts[0]["id"].as_str().unwrap(),
        parts[1]["id"].as_str().unwrap(),
    );

    // Unknown kind.
    let (status, _) = create_joint(
        port,
        serde_json::json!({
            "name": "bad", "kind": "hyper", "from_object_id": a, "to_object_id": b,
            "origin": { "x": 0.0, "y": 0.0, "z": 0.0 }
        }),
    )
    .await;
    assert_eq!(status, 400);

    // Same endpoints.
    let (status, _) = create_joint(
        port,
        serde_json::json!({
            "name": "bad", "kind": "revolute", "from_object_id": a, "to_object_id": a,
            "origin": { "x": 0.0, "y": 0.0, "z": 0.0 }, "axis": { "x": 0.0, "y": 0.0, "z": 1.0 }
        }),
    )
    .await;
    assert_eq!(status, 400);

    // Revolute without an axis.
    let (status, _) = create_joint(
        port,
        serde_json::json!({
            "name": "bad", "kind": "revolute", "from_object_id": a, "to_object_id": b,
            "origin": { "x": 0.0, "y": 0.0, "z": 0.0 }
        }),
    )
    .await;
    assert_eq!(status, 400);

    // Unknown object id.
    let (status, _) = create_joint(port, revolute(a, "no-such-object")).await;
    assert_eq!(status, 400);

    // First joint succeeds; a duplicate of the same pair+kind is refused.
    let (first, _) = create_joint(port, revolute(a, b)).await;
    assert_eq!(first, 201);
    let (status, body) = create_joint(port, revolute(b, a)).await;
    assert_eq!(status, 400, "{}", body);
}

#[tokio::test]
async fn deleting_an_endpoint_removes_joint_and_undo_restores_it() {
    let (port, _) = test_server().await;
    let asm = create_assembly(port).await;
    let parts = asm["parts"].as_array().unwrap();
    let (a, b) = (
        parts[0]["id"].as_str().unwrap(),
        parts[1]["id"].as_str().unwrap(),
    );

    let (status, joint) = create_joint(port, revolute(a, b)).await;
    assert_eq!(status, 201, "{}", joint);
    let joint_id = joint["id"].as_str().unwrap().to_string();

    // Single-object delete cascades the joint away.
    let client = reqwest::Client::new();
    let deleted = client
        .delete(url(port, &format!("/api/objects/{}", a)))
        .send()
        .await
        .unwrap();
    assert_eq!(deleted.status().as_u16(), 200);
    let remaining: serde_json::Value = reqwest::get(url(port, "/api/joints"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(remaining.as_array().unwrap().len(), 0, "{}", remaining);

    // One undo step brings the object and its joint back.
    let undone = client.post(url(port, "/api/undo")).send().await.unwrap();
    assert!(undone.status().is_success());
    let restored: serde_json::Value = reqwest::get(url(port, "/api/joints"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let arr = restored.as_array().unwrap();
    assert_eq!(arr.len(), 1, "{}", restored);
    assert_eq!(arr[0]["id"], joint_id.as_str());
    assert_eq!(arr[0]["from_object_id"], a);
}
