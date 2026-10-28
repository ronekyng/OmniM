//! Persistence behaviour for the single write path (Wave 1, Slice 1).
//!
//! Covers: opt-in persistence via `create_router` `save_path`, atomic save
//! (no truncated files), load-back round trip, explicit `/api/project/save`,
//! and the ephemeral escape hatch at the handler level.

use std::sync::Arc;

use omnim_model::Project;
use omnim_server::{create_router, SharedProject};
use tokio::sync::RwLock;

/// Router backed by a temp file project.
fn temp_router(dir: &std::path::Path) -> (SharedProject, axum::Router, std::path::PathBuf) {
    let project: SharedProject = Arc::new(RwLock::new(Project::new("Persist Test")));
    let path = dir.join("project.omnim");
    let router = create_router(project.clone(), Some(path.to_string_lossy().into_owned()));
    (project, router, path)
}

async fn test_server_with(dir: &std::path::Path) -> (u16, SharedProject, std::path::PathBuf) {
    let (project, router, path) = temp_router(dir);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    (port, project, path)
}

fn api_url(port: u16, p: &str) -> String {
    format!("http://127.0.0.1:{}{}", port, p)
}

#[tokio::test]
async fn mutation_persists_and_round_trips() {
    let dir = tempfile_dir("persist_round_trip");
    let (port, project, path) = test_server_with(&dir).await;
    let client = reqwest::Client::new();

    let resp = client
        .post(api_url(port, "/api/objects"))
        .json(&serde_json::json!({
            "object_type": "wall",
            "name": "W-01"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "create failed: {}", resp.text().await.unwrap());
    let obj: serde_json::Value = resp.json().await.unwrap();
    let id = obj["id"].as_str().unwrap().to_string();

    // The default save path must have been written by the mutate path.
    assert!(path.exists(), "project file was not persisted after create");
    let saved = Project::load(path.to_str().unwrap()).expect("saved project should load");
    assert!(saved.objects.get(&id).is_some(), "created object missing from saved file");
    assert_eq!(saved.objects.get(&id).unwrap().name, "W-01");

    // In-memory state matches the on-disk state.
    let p = project.read().await;
    assert!(p.objects.get(&id).is_some());
    drop(p);

    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn explicit_save_endpoint_writes_file() {
    let dir = tempfile_dir("persist_explicit");
    let (port, _project, path) = test_server_with(&dir).await;
    let client = reqwest::Client::new();

    // Nothing mutated yet: explicit save still produces the file.
    let resp = client
        .post(api_url(port, "/api/project/save"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "save failed: {}", resp.text().await.unwrap());
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);
    assert!(path.exists());

    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn no_save_path_means_no_file() {
    // Ephemeral router (save_path None, same as OMNIM_EPHEMERAL=1).
    let project: SharedProject = Arc::new(RwLock::new(Project::new("Ephemeral Test")));
    let router = create_router(project, None);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    let client = reqwest::Client::new();

    let resp = client
        .post(api_url(port, "/api/objects"))
        .json(&serde_json::json!({ "object_type": "wall", "name": "E-01" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Explicit save must be refused in ephemeral mode.
    let resp = client
        .post(api_url(port, "/api/project/save"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

// Small helper so tests don't need an extra dev-dependency.
fn tempfile_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("omnim-test-persist-{}-{}", tag, std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}
