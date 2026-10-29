//! P8-S1 acceptance tests — building datum: storey elevation / level
//! attachment and grid datum lines (create / update / delete / undo).

use std::sync::Arc;

use omnim_model::Project;
use omnim_server::{create_router, SharedProject};
use tokio::sync::RwLock;

async fn test_server() -> (u16, SharedProject) {
    let project: SharedProject = Arc::new(RwLock::new(Project::new("Datum Test")));
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

fn json_of(body: &str) -> serde_json::Value {
    serde_json::from_str(body).unwrap_or(serde_json::Value::Null)
}

async fn create_object(port: u16, body: serde_json::Value) -> (u16, String) {
    post_json(port, "/api/objects", body).await
}

// ── storey elevation ─────────────────────────────────────────────────────────

#[tokio::test]
async fn storey_is_created_with_a_datum_elevation() {
    let (port, _) = test_server().await;
    let (status, body) = create_object(
        port,
        serde_json::json!({
            "object_type": "storey", "name": "F2", "elevation": 3.9
        }),
    )
    .await;
    assert_eq!(status, 200, "{}", body);
    let id = json_of(&body)["id"].as_str().unwrap().to_string();
    let obj = get_json(port, &format!("/api/objects/{}", id)).await;
    assert_eq!(obj["elevation"], 3.9, "elevation must round-trip");
    assert_eq!(obj["object_type"], 3, "BuildingStorey");
}

#[tokio::test]
async fn non_storey_objects_have_no_elevation_by_default() {
    let (port, _) = test_server().await;
    let (status, body) = create_object(
        port,
        serde_json::json!({
            "object_type": "building", "name": "Tower"
        }),
    )
    .await;
    assert_eq!(status, 200, "{}", body);
    let id = json_of(&body)["id"].as_str().unwrap().to_string();
    let obj = get_json(port, &format!("/api/objects/{}", id)).await;
    assert!(obj["elevation"].is_null());
}

#[tokio::test]
async fn storey_elevation_is_editable_and_undoable() {
    let (port, _) = test_server().await;
    let (status, body) = create_object(
        port,
        serde_json::json!({
            "object_type": "storey", "name": "F1", "elevation": 0.0
        }),
    )
    .await;
    let id = json_of(&body)["id"].as_str().unwrap().to_string();

    let client = reqwest::Client::new();
    let resp = client
        .put(api_url(port, &format!("/api/objects/{}", id)))
        .json(&serde_json::json!({ "elevation": 4.2 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let obj = get_json(port, &format!("/api/objects/{}", id)).await;
    assert_eq!(obj["elevation"], 4.2);

    // Undo the elevation edit — the ModifyObject snapshot must cover the new
    // datum fields automatically.
    let (status, _) = post_json(port, "/api/undo", serde_json::json!({})).await;
    assert_eq!(status, 200);
    let obj = get_json(port, &format!("/api/objects/{}", id)).await;
    assert_eq!(obj["elevation"], 0.0, "undo must restore the old elevation");
}

// ── level attachment ─────────────────────────────────────────────────────────

#[tokio::test]
async fn object_can_attach_to_a_storey_level() {
    let (port, _) = test_server().await;
    let (status, body) = create_object(
        port,
        serde_json::json!({
            "object_type": "storey", "name": "F1", "elevation": 0.0
        }),
    )
    .await;
    let level_id = json_of(&body)["id"].as_str().unwrap().to_string();

    let (status, body) = create_object(
        port,
        serde_json::json!({
            "object_type": "wall", "name": "W1", "level_id": level_id
        }),
    )
    .await;
    assert_eq!(status, 200, "{}", body);
    let id = json_of(&body)["id"].as_str().unwrap().to_string();
    let obj = get_json(port, &format!("/api/objects/{}", id)).await;
    assert_eq!(
        obj["level_id"], level_id,
        "level attachment must round-trip"
    );
}

#[tokio::test]
async fn level_id_must_reference_an_existing_object() {
    let (port, _) = test_server().await;
    let (status, body) = create_object(
        port,
        serde_json::json!({
            "object_type": "wall", "name": "W-orphan", "level_id": "no-such-level"
        }),
    )
    .await;
    assert_eq!(status, 400, "unknown level must be rejected: {}", body);
    assert!(body.contains("not found"), "{}", body);
}

#[tokio::test]
async fn level_id_must_reference_a_building_storey() {
    let (port, _) = test_server().await;
    let (status, body) = create_object(
        port,
        serde_json::json!({
            "object_type": "building", "name": "Tower"
        }),
    )
    .await;
    let building_id = json_of(&body)["id"].as_str().unwrap().to_string();

    let (status, body) = create_object(
        port,
        serde_json::json!({
            "object_type": "wall", "name": "W-bad", "level_id": building_id
        }),
    )
    .await;
    assert_eq!(status, 400, "non-storey level must be rejected: {}", body);
    assert!(body.contains("not a BuildingStorey"), "{}", body);
}

// ── grid datum ───────────────────────────────────────────────────────────────

async fn create_grid(port: u16, label: &str, axis: &str, coordinate: f64) -> (u16, String) {
    post_json(
        port,
        "/api/grids",
        serde_json::json!({ "label": label, "axis": axis, "coordinate": coordinate }),
    )
    .await
}

#[tokio::test]
async fn grid_lines_crud_roundtrip() {
    let (port, _) = test_server().await;
    let (status, body) = create_grid(port, "1", "along_x", 0.0).await;
    assert_eq!(status, 201, "{}", body);
    let one = json_of(&body);
    assert_eq!(one["label"], "1");
    assert_eq!(one["coordinate"], 0.0);

    let (_, body) = create_grid(port, "2", "along_x", 6.0).await;
    let two_id = json_of(&body)["id"].as_str().unwrap().to_string();
    let (status, body) = create_grid(port, "A", "along_y", -3.0).await;
    assert_eq!(status, 201, "{}", body);
    let _a_id = json_of(&body)["id"].as_str().unwrap().to_string();

    // List: sorted AlongX first (numbered), then AlongY (lettered).
    let list = get_json(port, "/api/grids").await;
    let labels: Vec<&str> = list
        .as_array()
        .unwrap()
        .iter()
        .map(|g| g["label"].as_str().unwrap())
        .collect();
    assert_eq!(labels, vec!["1", "2", "A"], "sorted grid order");

    // Update
    let client = reqwest::Client::new();
    let resp = client
        .put(api_url(port, &format!("/api/grids/{}", two_id)))
        .json(&serde_json::json!({
            "label": "2", "axis": "along_x", "coordinate": 7.5
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let list = get_json(port, "/api/grids").await;
    let two = list
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["id"] == two_id.as_str())
        .unwrap();
    assert_eq!(two["coordinate"], 7.5);
}

#[tokio::test]
async fn grid_delete_is_undoable() {
    let (port, _) = test_server().await;
    let (status, body) = create_grid(port, "A", "along_y", 0.0).await;
    assert_eq!(status, 201, "{}", body);
    let id = json_of(&body)["id"].as_str().unwrap().to_string();

    let client = reqwest::Client::new();
    let resp = client
        .delete(api_url(port, &format!("/api/grids/{}", id)))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let list = get_json(port, "/api/grids").await;
    assert_eq!(list.as_array().unwrap().len(), 0);

    // Undo restores the deleted datum line.
    let (status, _) = post_json(port, "/api/undo", serde_json::json!({})).await;
    assert_eq!(status, 200);
    let list = get_json(port, "/api/grids").await;
    assert_eq!(
        list.as_array().unwrap().len(),
        1,
        "undo must restore the grid line"
    );
    assert_eq!(list[0]["label"], "A");
}

#[tokio::test]
async fn grid_create_is_undoable() {
    let (port, _) = test_server().await;
    let (status, body) = create_grid(port, "B", "along_y", 4.0).await;
    assert_eq!(status, 201, "{}", body);
    let (status, _) = post_json(port, "/api/undo", serde_json::json!({})).await;
    assert_eq!(status, 200);
    let list = get_json(port, "/api/grids").await;
    assert_eq!(
        list.as_array().unwrap().len(),
        0,
        "undo must remove the created grid line"
    );
}

#[tokio::test]
async fn grid_labels_are_unique_per_axis_family() {
    let (port, _) = test_server().await;
    let (status, _) = create_grid(port, "A", "along_y", 0.0).await;
    assert_eq!(status, 201);
    // Same label on the same axis family → rejected by the validator.
    let (status, body) = create_grid(port, "A", "along_y", 9.0).await;
    assert_eq!(status, 400, "{}", body);
    assert!(body.contains("already used"), "{}", body);
    // Same label on the other family is fine (numbered vs lettered).
    let (status, _) = create_grid(port, "A", "along_x", 9.0).await;
    assert_eq!(
        status, 201,
        "labels are unique per axis family, not globally"
    );
}

#[tokio::test]
async fn grid_unknown_axis_is_a_400() {
    let (port, _) = test_server().await;
    let (status, body) = create_grid(port, "C", "diagonal", 0.0).await;
    assert_eq!(status, 400, "{}", body);
    assert!(body.contains("unknown grid axis"), "{}", body);
}

#[tokio::test]
async fn grid_empty_label_is_a_400() {
    let (port, _) = test_server().await;
    let (status, body) = create_grid(port, "  ", "along_x", 0.0).await;
    assert_eq!(status, 400, "{}", body);
    assert!(body.contains("label is empty"), "{}", body);
}

#[tokio::test]
async fn grid_update_and_delete_404_on_unknown_id() {
    let (port, _) = test_server().await;
    let client = reqwest::Client::new();
    let resp = client
        .put(api_url(port, "/api/grids/no-such-grid"))
        .json(&serde_json::json!({ "label": "X", "axis": "along_x", "coordinate": 0.0 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 404);
    let resp = client
        .delete(api_url(port, "/api/grids/no-such-grid"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 404);
}
