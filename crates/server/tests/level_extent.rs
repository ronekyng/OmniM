//! P8-S2 acceptance tests — level-driven vertical extent:
//! bottom/top level constraints, base + height resolution, storey-elevation
//! cascade as a **single undo step**, and the referenced-level delete guard.

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
    let project: SharedProject = Arc::new(RwLock::new(Project::new("Level Extent Test")));
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

async fn put_json(port: u16, path: &str, body: serde_json::Value) -> (u16, String) {
    let client = reqwest::Client::new();
    let resp = client
        .put(api_url(port, path))
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

async fn make_storey(port: u16, name: &str, elevation: f64) -> String {
    let (status, body) = post_json(
        port,
        "/api/objects",
        serde_json::json!({ "object_type": "storey", "name": name, "elevation": elevation }),
    )
    .await;
    assert_eq!(status, 200, "storey create failed: {}", body);
    json_of(&body)["id"].as_str().unwrap().to_string()
}

/// Create a path-driven layered wall with vertical-extent constraints.
async fn make_wall(port: u16, name: &str, bottom: &str, top: &str) -> (u16, String) {
    post_json(
        port,
        "/api/genrule",
        serde_json::json!({
            "genrule": "layered",
            "name": name,
            "mode": "wall",
            "path": { "points": [[0.0, 0.0, 0.0], [5.0, 0.0, 0.0]], "closed": false },
            "layers": [{ "name": "core", "thickness": 0.2 }],
            "bottom_level_id": bottom,
            "top_level_id": top,
        }),
    )
    .await
}

/// The single PathSweep payload of a wall object.
fn path_sweep(obj: &serde_json::Value) -> &serde_json::Value {
    obj["features"]
        .as_array()
        .expect("features array")
        .iter()
        .find_map(|f| f.get("path_sweep").filter(|p| !p.is_null()))
        .expect("PathSweep feature")
}

fn min_z(params: &serde_json::Value) -> f64 {
    params["points"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p[2].as_f64().unwrap())
        .fold(f64::INFINITY, f64::min)
}

#[tokio::test]
async fn bottom_and_top_levels_drive_base_and_height() {
    let port = test_server().await;
    let f1 = make_storey(port, "F1", 0.0).await;
    let f2 = make_storey(port, "F2", 3.9).await;

    let (status, body) = make_wall(port, "W-extent", &f1, &f2).await;
    assert_eq!(status, 200, "wall create failed: {}", body);
    let wall = json_of(&body);
    assert_eq!(wall["elevation"], 0.0, "base resolves to the bottom level");
    assert_eq!(wall["bottom_level_id"], f1);
    assert_eq!(wall["top_level_id"], f2);
    let sweep = path_sweep(&wall);
    assert_eq!(
        sweep["height"], 3.9,
        "height must be top - bottom, not the request's 3.0"
    );
    assert_eq!(min_z(sweep), 0.0, "path base sits on the bottom level");
}

#[tokio::test]
async fn storey_elevation_change_cascades_as_one_undo_step() {
    let port = test_server().await;
    let f1 = make_storey(port, "F1", 0.0).await;
    let f2 = make_storey(port, "F2", 3.9).await;
    let (status, body) = make_wall(port, "W-cascade", &f1, &f2).await;
    assert_eq!(status, 200, "{}", body);
    let wall_id = json_of(&body)["id"].as_str().unwrap().to_string();

    // Move the bottom storey up by 0.5 m.
    let (status, body) = put_json(
        port,
        &format!("/api/objects/{}", f1),
        serde_json::json!({ "elevation": 0.5 }),
    )
    .await;
    assert_eq!(status, 200, "{}", body);

    let wall = get_json(port, &format!("/api/objects/{}", wall_id)).await;
    assert_eq!(wall["elevation"], 0.5, "wall base follows the storey");
    let sweep = path_sweep(&wall);
    assert_eq!(sweep["height"], 3.4, "height shrinks to top - bottom");
    assert_eq!(min_z(sweep), 0.5, "path points shift with the storey");

    // One undo must revert both the storey and the cascaded wall.
    let (status, _) = post_json(port, "/api/undo", serde_json::json!({})).await;
    assert_eq!(status, 200);
    let level = get_json(port, &format!("/api/objects/{}", f1)).await;
    assert_eq!(level["elevation"], 0.0, "storey elevation restored");
    let wall = get_json(port, &format!("/api/objects/{}", wall_id)).await;
    assert_eq!(wall["elevation"], 0.0, "cascade restored in the same step");
    let sweep = path_sweep(&wall);
    assert_eq!(sweep["height"], 3.9);
    assert_eq!(min_z(sweep), 0.0);

    // Redo re-applies the cascade.
    let (status, _) = post_json(port, "/api/redo", serde_json::json!({})).await;
    assert_eq!(status, 200);
    let wall = get_json(port, &format!("/api/objects/{}", wall_id)).await;
    assert_eq!(wall["elevation"], 0.5);
    assert_eq!(path_sweep(&wall)["height"], 3.4);
}

#[tokio::test]
async fn extent_requires_both_refs() {
    let port = test_server().await;
    let f1 = make_storey(port, "F1", 0.0).await;
    let (status, body) = make_wall(port, "W-bottom-only", &f1, "").await;
    assert_eq!(status, 400, "{}", body);
    assert!(body.contains("must be set together"), "{}", body);
}

#[tokio::test]
async fn extent_refs_must_be_building_storeys() {
    let port = test_server().await;
    let (_, body) = post_json(
        port,
        "/api/objects",
        serde_json::json!({ "object_type": "building", "name": "Tower" }),
    )
    .await;
    let building = json_of(&body)["id"].as_str().unwrap().to_string();
    let f2 = make_storey(port, "F2", 3.9).await;

    let (status, body) = make_wall(port, "W-bad-ref", &building, &f2).await;
    assert_eq!(status, 400, "{}", body);
    assert!(body.contains("not a BuildingStorey"), "{}", body);
}

#[tokio::test]
async fn extent_rejects_inverted_levels() {
    let port = test_server().await;
    let high = make_storey(port, "HIGH", 5.0).await;
    let low = make_storey(port, "LOW", 0.0).await;
    let (status, body) = make_wall(port, "W-inverted", &high, &low).await;
    assert_eq!(status, 400, "{}", body);
    assert!(body.contains("must be above"), "{}", body);
}

#[tokio::test]
async fn extent_rejects_identical_levels() {
    let port = test_server().await;
    let f1 = make_storey(port, "F1", 0.0).await;
    let (status, body) = make_wall(port, "W-same", &f1, &f1).await;
    assert_eq!(status, 400, "{}", body);
    assert!(body.contains("must be different"), "{}", body);
}

#[tokio::test]
async fn plain_objects_get_the_elevation_datum_only() {
    let port = test_server().await;
    let f1 = make_storey(port, "F1", 1.5).await;
    let f2 = make_storey(port, "F2", 4.5).await;
    // A plain box wall (no PathSweep feature): base datum resolves, mesh untouched.
    let (status, body) = post_json(
        port,
        "/api/objects",
        serde_json::json!({
            "object_type": "wall", "name": "W-plain",
            "bottom_level_id": f1, "top_level_id": f2
        }),
    )
    .await;
    assert_eq!(status, 200, "{}", body);
    let wall = json_of(&body);
    assert_eq!(wall["elevation"], 1.5);
    assert_eq!(wall["features"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn extent_can_be_attached_after_creation_and_cleared() {
    let port = test_server().await;
    let f1 = make_storey(port, "F1", 0.0).await;
    let f2 = make_storey(port, "F2", 3.0).await;
    let (status, body) = post_json(
        port,
        "/api/objects",
        serde_json::json!({ "object_type": "wall", "name": "W-late" }),
    )
    .await;
    assert_eq!(status, 200, "{}", body);
    let id = json_of(&body)["id"].as_str().unwrap().to_string();

    let (status, body) = put_json(
        port,
        &format!("/api/objects/{}", id),
        serde_json::json!({ "bottom_level_id": f1, "top_level_id": f2 }),
    )
    .await;
    assert_eq!(status, 200, "{}", body);
    let wall = get_json(port, &format!("/api/objects/{}", id)).await;
    assert_eq!(wall["elevation"], 0.0);
    assert_eq!(wall["top_level_id"], f2);

    // Clearing one side is rejected (both-or-neither)…
    let (status, body) = put_json(
        port,
        &format!("/api/objects/{}", id),
        serde_json::json!({ "top_level_id": "" }),
    )
    .await;
    assert_eq!(status, 400, "{}", body);
    assert!(body.contains("must be set together"), "{}", body);

    // …clearing both is fine.
    let (status, body) = put_json(
        port,
        &format!("/api/objects/{}", id),
        serde_json::json!({ "bottom_level_id": "", "top_level_id": "" }),
    )
    .await;
    assert_eq!(status, 200, "{}", body);
    let wall = get_json(port, &format!("/api/objects/{}", id)).await;
    assert!(wall["bottom_level_id"].is_null());
    assert!(wall["top_level_id"].is_null());
}

#[tokio::test]
async fn deleting_a_referenced_storey_is_rejected() {
    let port = test_server().await;
    let f1 = make_storey(port, "F1", 0.0).await;
    let f2 = make_storey(port, "F2", 3.9).await;
    let (status, body) = make_wall(port, "W-hosted", &f1, &f2).await;
    assert_eq!(status, 200, "{}", body);
    let wall_id = json_of(&body)["id"].as_str().unwrap().to_string();

    let client = reqwest::Client::new();
    let resp = client
        .delete(api_url(port, &format!("/api/objects/{}", f1)))
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let body = resp.text().await.unwrap();
    assert_eq!(status, 400, "{}", body);
    assert!(body.contains("referenced by 1 object"), "{}", body);

    // Detach the wall, then the storey can go.
    let (status, body) = put_json(
        port,
        &format!("/api/objects/{}", wall_id),
        serde_json::json!({ "bottom_level_id": "", "top_level_id": "" }),
    )
    .await;
    assert_eq!(status, 200, "{}", body);
    let resp = client
        .delete(api_url(port, &format!("/api/objects/{}", f1)))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
}
