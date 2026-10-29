//! Project-home acceptance tests (UI-DAT-01): session gating, new/open,
//! recent listing, sample content and allowed-root enforcement.

use std::sync::Arc;

use omnim_model::Project;
use omnim_server::{create_router, SharedProject};
use tokio::sync::RwLock;

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "omnim-test-projects-{}-{}",
        tag,
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

async fn test_server(dir: &std::path::Path) -> u16 {
    let project: SharedProject = Arc::new(RwLock::new(Project::new("Session Test")));
    let save_path = dir.join("project.omnim");
    let router = create_router(project, Some(save_path.to_string_lossy().into_owned()));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    port
}

fn object_values(project: &serde_json::Value) -> Vec<&serde_json::Value> {
    project["objects"]["objects"]
        .as_object()
        .expect("project objects map")
        .values()
        .collect()
}

fn url(port: u16, path: &str) -> String {
    format!("http://127.0.0.1:{}{}", port, path)
}

async fn post(port: u16, path: &str, body: serde_json::Value) -> (u16, serde_json::Value) {
    let resp = reqwest::Client::new()
        .post(url(port, path))
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap();
    (
        status,
        serde_json::from_str(&text).unwrap_or(serde_json::Value::Null),
    )
}

async fn get(port: u16, path: &str) -> (u16, serde_json::Value) {
    let resp = reqwest::Client::new()
        .get(url(port, path))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap();
    (
        status,
        serde_json::from_str(&text).unwrap_or(serde_json::Value::Null),
    )
}

#[tokio::test]
async fn session_starts_closed_and_new_project_opens_it() {
    let dir = temp_dir("new");
    let port = test_server(&dir).await;

    let (status, current) = get(port, "/api/project/current").await;
    assert_eq!(status, 200);
    assert_eq!(current["open"], false, "workspace must not be implicit");

    let (status, projects) = get(port, "/api/project/recent").await;
    assert_eq!(status, 200);
    assert_eq!(projects.as_array().unwrap().len(), 0);

    let (status, created) = post(
        port,
        "/api/project/new",
        serde_json::json!({ "name": "Alpha" }),
    )
    .await;
    assert_eq!(status, 200, "{}", created);
    assert_eq!(created["open"], true);
    assert_eq!(created["name"], "Alpha");
    let path = created["path"].as_str().unwrap().to_string();
    assert!(path.ends_with("alpha.omnim"), "{}", path);
    assert!(std::path::Path::new(&path).exists());

    let (_, current) = get(port, "/api/project/current").await;
    assert_eq!(current["open"], true);
    assert_eq!(current["path"], path);

    let (_, listed) = get(port, "/api/project/recent").await;
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(listed[0]["name"], "Alpha");

    let (status, duplicate) = post(
        port,
        "/api/project/new",
        serde_json::json!({ "name": "Alpha" }),
    )
    .await;
    assert_eq!(status, 409, "{}", duplicate);

    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn sample_project_contains_storeys_and_grids() {
    let dir = temp_dir("sample");
    let port = test_server(&dir).await;

    let (status, created) = post(
        port,
        "/api/project/new",
        serde_json::json!({ "name": "Demo", "sample": true }),
    )
    .await;
    assert_eq!(status, 200, "{}", created);

    let (status, project) = get(port, "/api/project").await;
    assert_eq!(status, 200);
    let storeys = object_values(&project)
        .iter()
        .filter(|o| o["object_type"] == 3)
        .count();
    assert_eq!(storeys, 2, "sample has two storeys");
    assert_eq!(project["grids"]["grid_lines"].as_array().unwrap().len(), 4);

    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn open_loads_a_project_and_clears_history() {
    let dir = temp_dir("open");
    let port = test_server(&dir).await;

    // Create A with an object, then create B so A is not current.
    let (_, a) = post(port, "/api/project/new", serde_json::json!({ "name": "A" })).await;
    let a_path = a["path"].as_str().unwrap().to_string();
    let (status, _) = post(
        port,
        "/api/objects",
        serde_json::json!({
            "object_type": "wall", "name": "W-01"
        }),
    )
    .await;
    assert_eq!(status, 200);
    let (_, b) = post(port, "/api/project/new", serde_json::json!({ "name": "B" })).await;
    assert_eq!(b["name"], "B");

    let (status, opened) = post(
        port,
        "/api/project/open",
        serde_json::json!({ "path": a_path }),
    )
    .await;
    assert_eq!(status, 200, "{}", opened);
    assert_eq!(opened["name"], "A");

    let (_, project) = get(port, "/api/project").await;
    assert!(object_values(&project).iter().any(|o| o["name"] == "W-01"));

    let (_, undo) = get(port, "/api/undo-status").await;
    assert_eq!(
        undo["can_undo"], false,
        "opening a project clears undo history"
    );

    let (status, _) = post(
        port,
        "/api/project/open",
        serde_json::json!({ "path": dir.join("missing.omnim").to_string_lossy() }),
    )
    .await;
    assert_eq!(status, 404);

    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn path_traversal_and_save_as_outside_the_root_are_rejected() {
    let dir = temp_dir("traversal");
    let port = test_server(&dir).await;

    let (status, _) = post(
        port,
        "/api/project/open",
        serde_json::json!({ "path": "../escape.omnim" }),
    )
    .await;
    assert_eq!(status, 400);

    let outside = std::env::temp_dir().join(format!("omnim-escape-{}.omnim", std::process::id()));
    let (status, _) = post(
        port,
        "/api/project/save-as",
        serde_json::json!({ "path": outside.to_string_lossy() }),
    )
    .await;
    assert_eq!(status, 400);
    assert!(!outside.exists());

    // A path is also rejected when it is not a project file.
    let (status, _) = post(
        port,
        "/api/project/save-as",
        serde_json::json!({ "path": dir.join("notes.txt").to_string_lossy() }),
    )
    .await;
    assert_eq!(status, 400);

    std::fs::remove_dir_all(&dir).ok();
}
