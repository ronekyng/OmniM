//! Wave 1 Slice 5 acceptance tests: catalog preview/instantiate and IFC import.
//!
//! These are written ahead of the implementation (TDD):
//! - preview must not mutate the shared project;
//! - catalog instantiation must be one undoable transaction;
//! - IFC import (merge and replace) must be one undoable transaction.

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
    let project: SharedProject = Arc::new(RwLock::new(Project::new("Slice5 Test")));
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

async fn object_count(project: &SharedProject) -> usize {
    project.read().await.objects.len()
}

async fn undo(port: u16) {
    let client = reqwest::Client::new();
    let resp = client.post(api_url(port, "/api/undo")).send().await.unwrap();
    assert!(resp.status().is_success());
}

async fn redo(port: u16) {
    let client = reqwest::Client::new();
    let resp = client.post(api_url(port, "/api/redo")).send().await.unwrap();
    assert!(resp.status().is_success());
}

async fn seeded_def_with_tree(port: u16) -> Option<serde_json::Value> {
    let client = reqwest::Client::new();
    let defs: Vec<serde_json::Value> = client
        .get(api_url(port, "/api/catalog"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    defs.into_iter()
        .find(|d| d["feature_tree_id"].as_str().map(|s| !s.is_empty()).unwrap_or(false))
}

const SIMPLE_IFC: &str = include_str!("../../../fixtures/ifc/simple.ifc");

#[tokio::test]
async fn preview_does_not_mutate_the_project() {
    let (port, project) = test_server().await;
    let client = reqwest::Client::new();
    let def = seeded_def_with_tree(port).await.expect("seeded catalog has a feature-tree def");

    let before = object_count(&project).await;
    let resp = client
        .post(api_url(port, "/api/catalog/preview"))
        .json(&serde_json::json!({ "component_def_id": def["id"], "params": {} }))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "preview failed: {}", resp.text().await.unwrap());

    assert_eq!(object_count(&project).await, before, "preview leaked an object into the project");
    let leaked = {
        let p = project.read().await;
        p.objects.list().iter().any(|o| o.name.starts_with("__preview_"))
    };
    assert!(!leaked, "preview object was left behind");
}

#[tokio::test]
async fn catalog_instantiation_is_one_undoable_transaction() {
    let (port, project) = test_server().await;
    let client = reqwest::Client::new();
    let def = seeded_def_with_tree(port).await.expect("seeded catalog has a feature-tree def");

    let resp = client
        .post(api_url(port, "/api/catalog/instantiate"))
        .json(&serde_json::json!({
            "component_def_id": def["id"],
            "instance_name": "inst-undo",
            "params": {},
            "skip_series_gate": true
        }))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "instantiate failed: {}", resp.text().await.unwrap());
    let body: serde_json::Value = resp.json().await.unwrap();
    let id = body["component_id"].as_str().unwrap().to_string();

    assert_eq!(client.get(api_url(port, &format!("/api/objects/{}", id))).send().await.unwrap().status(), 200);
    assert_eq!(object_count(&project).await, 1);

    undo(port).await;
    assert_eq!(object_count(&project).await, 0, "instantiation left the object behind after undo");
    assert_eq!(client.get(api_url(port, &format!("/api/objects/{}", id))).send().await.unwrap().status(), 404);

    redo(port).await;
    assert_eq!(object_count(&project).await, 1, "redo did not restore the instance");
}

#[tokio::test]
async fn ifc_merge_import_is_undoable() {
    let (port, project) = test_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(api_url(port, "/api/import/ifc?mode=merge"))
        .header("Content-Type", "text/plain")
        .body(SIMPLE_IFC)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true, "import failed: {}", body);

    let imported = object_count(&project).await;
    assert!(imported > 0);

    undo(port).await;
    assert_eq!(object_count(&project).await, 0, "IFC import was not undone as one transaction");

    redo(port).await;
    assert_eq!(object_count(&project).await, imported, "IFC import redo did not restore objects");
}

#[tokio::test]
async fn ifc_replace_import_is_undoable() {
    let (port, project) = test_server().await;
    let client = reqwest::Client::new();

    // Seed two objects so replace has something to destroy.
    for name in ["Seed-A", "Seed-B"] {
        client
            .post(api_url(port, "/api/objects"))
            .json(&serde_json::json!({ "object_type": "wall", "name": name }))
            .send()
            .await
            .unwrap();
    }
    assert_eq!(object_count(&project).await, 2);

    let resp = client
        .post(api_url(port, "/api/import/ifc?mode=replace"))
        .header("Content-Type", "text/plain")
        .body(SIMPLE_IFC)
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true, "import failed: {}", body);
    let imported = object_count(&project).await;
    assert_ne!(imported, 2, "replace import did not change the project");

    undo(port).await;
    assert_eq!(object_count(&project).await, 2, "replace import undo did not restore the old project");

    redo(port).await;
    assert_eq!(object_count(&project).await, imported, "replace import redo did not reapply");
}

#[tokio::test]
async fn import_persists_when_save_path_is_set() {
    let dir = std::env::temp_dir().join(format!("omnim-slice5-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("project.omnim");
    init_library();
    let project: SharedProject = Arc::new(RwLock::new(Project::new("Persist Import")));
    let router = create_router(project, Some(path.to_string_lossy().into_owned()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });
    let client = reqwest::Client::new();

    let resp = client
        .post(api_url(port, "/api/import/ifc?mode=merge"))
        .header("Content-Type", "text/plain")
        .body(SIMPLE_IFC)
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    assert!(path.exists(), "IFC import did not persist the project");
    let loaded = Project::load(path.to_str().unwrap()).unwrap();
    assert!(loaded.objects.len() > 0);
    std::fs::remove_dir_all(&dir).ok();
}
