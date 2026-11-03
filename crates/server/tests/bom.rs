//! Phase 8-4 (ASSEMBLY §1): BOM / quantity endpoint coverage.

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
    let project: SharedProject = Arc::new(RwLock::new(Project::new("BOM Test")));
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

async fn create_wall(port: u16, name: &str) -> (u16, serde_json::Value) {
    let client = reqwest::Client::new();
    let resp = client
        .post(api_url(port, "/api/objects"))
        .json(&serde_json::json!({ "object_type": "wall", "name": name }))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap())
}

async fn create_assembly(port: u16, name: &str, def_id: &str) -> serde_json::Value {
    let client = reqwest::Client::new();
    let resp = client
        .post(api_url(port, "/api/assemblies"))
        .json(&serde_json::json!({
            "name": name,
            "skip_series_gate": true,
            "parts": [
                { "name": "Comp-1", "component_def_id": def_id },
                { "name": "Box-1", "shape": "box", "p1": 0.5, "p2": 0.4, "p3": 0.3 }
            ]
        }))
        .send()
        .await
        .unwrap();
    let text = resp.text().await.unwrap();
    serde_json::from_str(&text).unwrap_or_else(|_| panic!("create assembly failed: {}", text))
}

async fn bom(port: u16, id: &str) -> serde_json::Value {
    let client = reqwest::Client::new();
    let resp = client
        .get(api_url(port, &format!("/api/objects/{}/bom", id)))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    assert_eq!(status, 200, "bom failed: {}", resp.text().await.unwrap());
    resp.json().await.unwrap()
}

#[tokio::test]
async fn bom_lists_subtree_rows_with_kinds_and_volume() {
    let (port, _) = test_server().await;
    let client = reqwest::Client::new();
    // A seeded catalog def ships with a feature-tree template, which the
    // assembly instantiation requires.
    let list: Vec<serde_json::Value> = client
        .get(api_url(port, "/api/catalog"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let def_id = list
        .first()
        .and_then(|d| d["id"].as_str())
        .expect("seeded library must contain a component def")
        .to_string();

    let asm = create_assembly(port, "ASM-BOM", &def_id).await;
    let id = asm["assembly_id"].as_str().unwrap().to_string();

    let response = bom(port, &id).await;
    assert_eq!(response["root_id"], id);
    let rows = response["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "{}", response);

    // No container row: the Assembly itself has no geometry.
    assert!(
        rows.iter().all(|r| r["object_type"] != "Assembly"),
        "{}",
        response
    );
    // The catalog part carries its source def; the primitive part does not.
    let comp = rows.iter().find(|r| r["name"] == "Comp-1").unwrap();
    assert_eq!(comp["kind"], "component");
    assert_eq!(comp["component_def_id"], def_id);
    let part = rows.iter().find(|r| r["name"] == "Box-1").unwrap();
    assert_eq!(part["kind"], "part");
    assert!(part["component_def_id"].is_null(), "{}", response);

    // Volume: the box part has a bbox; the totals must include it.
    assert!(
        part["bbox_volume_m3"].as_f64().unwrap() > 0.0,
        "{}",
        response
    );
    assert!(
        response["totals"]["total_volume_m3"].as_f64().unwrap()
            >= part["bbox_volume_m3"].as_f64().unwrap(),
        "{}",
        response
    );
}

#[tokio::test]
async fn leaf_object_bom_lists_itself() {
    let (port, _) = test_server().await;
    let (status, wall) = create_wall(port, "W-01").await;
    assert_eq!(status, 200);
    let id = wall["id"].as_str().unwrap();

    let response = bom(port, id).await;
    assert_eq!(response["root_id"], id, "{}", response);
    let rows = response["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1, "{}", response);
    assert_eq!(rows[0]["name"], "W-01");
    assert_eq!(rows[0]["kind"], "proxy");
    assert!(rows[0]["component_def_id"].is_null());
}

#[tokio::test]
async fn bom_unknown_object_is_404() {
    let (port, _) = test_server().await;
    let client = reqwest::Client::new();
    let resp = client
        .get(api_url(port, "/api/objects/nope/bom"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 404);
}
