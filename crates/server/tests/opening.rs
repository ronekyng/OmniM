//! P8 acceptance tests — standalone opening volumes (IfcOpeningElement):
//! host-relative void box, host validation, undo.

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

async fn test_server() -> u16 {
    init_library();
    let project: SharedProject = Arc::new(RwLock::new(Project::new("Opening Test")));
    let router = create_router(project, None);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    port
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

/// A real layered wall (has bbox geometry) to host the opening.
async fn make_wall(port: u16) -> String {
    let (status, body) = post_json(
        port,
        "/api/genrule",
        serde_json::json!({
            "genrule": "layered",
            "name": "Host wall",
            "mode": "wall",
            "path": { "points": [[0.0, 0.0, 0.0], [5.0, 0.0, 0.0]], "closed": false },
            "layers": [{ "name": "core", "thickness": 0.3 }],
            "height": 3.0
        }),
    )
    .await;
    assert_eq!(status, 200, "wall create failed: {}", body);
    json_of(&body)["id"].as_str().unwrap().to_string()
}

fn bbox_size(obj: &serde_json::Value) -> (f64, f64, f64) {
    let b = &obj["geometry"]["bbox"];
    let mn = &b["min"];
    let mx = &b["max"];
    (
        mx["x"].as_f64().unwrap() - mn["x"].as_f64().unwrap(),
        mx["y"].as_f64().unwrap() - mn["y"].as_f64().unwrap(),
        mx["z"].as_f64().unwrap() - mn["z"].as_f64().unwrap(),
    )
}

#[tokio::test]
async fn opening_is_a_host_relative_void_volume() {
    let port = test_server().await;
    let wall_id = make_wall(port).await;

    let (status, body) = post_json(
        port,
        "/api/genrule",
        serde_json::json!({
            "genrule": "opening",
            "name": "Void-1",
            "wall_id": wall_id,
            "opening_width": 1.0,
            "opening_height": 2.1,
            "position_x": 1.5,
            "position_z": 0.0
        }),
    )
    .await;
    assert_eq!(status, 200, "opening create failed: {}", body);
    let obj = json_of(&body);
    assert_eq!(obj["object_type"], 116, "ObjectType::Opening");
    assert_eq!(obj["parent_id"], wall_id, "opening is hosted by the wall");
    assert!(
        obj["properties"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["name"] == "host_wall_id"),
        "host is also recorded as a property: {}",
        obj["properties"]
    );

    let (w, t, h) = bbox_size(&obj);
    assert!((w - 1.0).abs() < 1e-6, "opening width: {}", w);
    assert!((h - 2.1).abs() < 1e-6, "opening height: {}", h);
    assert!(
        t > 0.3 && t < 0.4,
        "void is wall-thickness + tolerance: {}",
        t
    );
}

#[tokio::test]
async fn door_leaf_is_sized_to_the_opening_not_the_wall() {
    let port = test_server().await;
    let wall_id = make_wall(port).await;

    let (status, body) = post_json(
        port,
        "/api/genrule",
        serde_json::json!({
            "genrule": "door",
            "name": "D1",
            "wall_id": wall_id,
            "opening_width": 1.0,
            "opening_height": 2.1,
            "position_x": 2.5,
            "position_z": 0.0
        }),
    )
    .await;
    assert_eq!(status, 200, "door create failed: {}", body);
    let obj = json_of(&body);
    assert_eq!(obj["object_type"], 104, "ObjectType::Door");
    assert_eq!(obj["parent_id"], wall_id);

    let (w, d, h) = bbox_size(&obj);
    assert!((w - 1.0).abs() < 1e-6, "door width follows the opening: {}", w);
    assert!((h - 2.1).abs() < 1e-6, "door height follows the opening: {}", h);
    assert!(d < 0.4, "door depth is the wall thickness, not the bbox: {}", d);

    let props = obj["properties"].as_array().unwrap();
    assert!(props.iter().any(|p| p["name"] == "host_wall_id"));
    assert!(props.iter().any(|p| p["name"] == "opening_width"));
}

#[tokio::test]
async fn door_follows_a_rotated_wall_centreline() {
    let port = test_server().await;
    let (status, body) = post_json(
        port,
        "/api/genrule",
        serde_json::json!({
            "genrule": "layered",
            "name": "Diagonal wall",
            "mode": "wall",
            "path": { "points": [[0.0, 0.0, 0.0], [10.0, 10.0, 0.0]], "closed": false },
            "layers": [{ "name": "core", "thickness": 0.2 }],
            "height": 3.0
        }),
    )
    .await;
    assert_eq!(status, 200, "wall create failed: {}", body);
    let wall_id = json_of(&body)["id"].as_str().unwrap().to_string();

    // Half of the 14.142 m diagonal.
    let (status, body) = post_json(
        port,
        "/api/genrule",
        serde_json::json!({
            "genrule": "door",
            "name": "D-diag",
            "wall_id": wall_id,
            "opening_width": 1.0,
            "opening_height": 2.1,
            "position_x": 7.0710678,
            "position_z": 0.0
        }),
    )
    .await;
    assert_eq!(status, 200, "door create failed: {}", body);
    let obj = json_of(&body);
    let (w, d, h) = bbox_size(&obj);
    assert!(w < 2.0 && d < 2.0, "rotated door is not wall-sized: {} x {}", w, d);
    assert!((h - 2.1).abs() < 1e-6);

    let mn = &obj["geometry"]["bbox"]["min"];
    let mx = &obj["geometry"]["bbox"]["max"];
    let cx = (mn["x"].as_f64().unwrap() + mx["x"].as_f64().unwrap()) / 2.0;
    let cy = (mn["y"].as_f64().unwrap() + mx["y"].as_f64().unwrap()) / 2.0;
    assert!(
        (cx - 5.0).abs() < 1.0 && (cy - 5.0).abs() < 1.0,
        "door centre follows the wall path, got ({}, {})",
        cx,
        cy
    );
}

#[tokio::test]
async fn window_uses_the_sill_height_as_its_base() {
    let port = test_server().await;
    let wall_id = make_wall(port).await;
    let (status, body) = post_json(
        port,
        "/api/genrule",
        serde_json::json!({
            "genrule": "window",
            "name": "W1",
            "wall_id": wall_id,
            "opening_width": 1.2,
            "opening_height": 1.5,
            "position_x": 2.5,
            "position_z": 0.9
        }),
    )
    .await;
    assert_eq!(status, 200, "window create failed: {}", body);
    let obj = json_of(&body);
    let (w, _d, h) = bbox_size(&obj);
    assert!((w - 1.2).abs() < 1e-6);
    assert!((h - 1.5).abs() < 1e-6);
    let min_z = obj["geometry"]["bbox"]["min"]["z"].as_f64().unwrap();
    assert!((min_z - 0.9).abs() < 1e-6, "sill height: {}", min_z);
}

#[tokio::test]
async fn opening_requires_an_existing_host() {
    let port = test_server().await;
    let (status, body) = post_json(
        port,
        "/api/genrule",
        serde_json::json!({
            "genrule": "opening", "name": "Orphan",
            "wall_id": "no-such-wall",
            "opening_width": 1.0, "opening_height": 2.1,
            "position_x": 0.0, "position_z": 0.0
        }),
    )
    .await;
    assert_eq!(status, 404, "{}", body);
    assert!(body.contains("not found"), "{}", body);
}

#[tokio::test]
async fn opening_host_must_be_a_wall() {
    let port = test_server().await;
    let (status, body) = post_json(
        port,
        "/api/objects",
        serde_json::json!({ "object_type": "storey", "name": "F1", "elevation": 0.0 }),
    )
    .await;
    assert_eq!(status, 200, "{}", body);
    let storey_id = json_of(&body)["id"].as_str().unwrap().to_string();

    let (status, body) = post_json(
        port,
        "/api/genrule",
        serde_json::json!({
            "genrule": "opening", "name": "Bad host",
            "wall_id": storey_id,
            "opening_width": 1.0, "opening_height": 2.1,
            "position_x": 0.0, "position_z": 0.0
        }),
    )
    .await;
    assert_eq!(status, 400, "{}", body);
    assert!(body.contains("not a Wall"), "{}", body);
}

#[tokio::test]
async fn opening_delete_is_undoable() {
    let port = test_server().await;
    let wall_id = make_wall(port).await;
    let (status, body) = post_json(
        port,
        "/api/genrule",
        serde_json::json!({
            "genrule": "opening", "name": "Void-2",
            "wall_id": wall_id,
            "opening_width": 0.9, "opening_height": 2.1,
            "position_x": 2.0, "position_z": 0.0
        }),
    )
    .await;
    assert_eq!(status, 200, "{}", body);
    let id = json_of(&body)["id"].as_str().unwrap().to_string();

    let client = reqwest::Client::new();
    let resp = client
        .delete(api_url(port, &format!("/api/objects/{}", id)))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let objects = get_json(port, "/api/objects").await;
    assert!(
        !objects
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["id"] == id.as_str()),
        "opening deleted"
    );

    let (status, _) = post_json(port, "/api/undo", serde_json::json!({})).await;
    assert_eq!(status, 200);
    let objects = get_json(port, "/api/objects").await;
    assert!(
        objects
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["id"] == id.as_str()),
        "undo restores the opening"
    );
}
