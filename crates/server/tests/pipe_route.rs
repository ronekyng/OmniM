//! P8 acceptance tests — pipe routing (`genrule: "pipe"` with a picked path).

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
    let project: SharedProject = Arc::new(RwLock::new(Project::new("Pipe Route Test")));
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

fn prop<'a>(obj: &'a serde_json::Value, name: &str) -> Option<&'a serde_json::Value> {
    obj["properties"]
        .as_array()?
        .iter()
        .find(|p| p["name"] == name)
        .and_then(|p| p["value"]["value_type"].as_object())
        .and_then(|v| v.values().next())
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

async fn object_count(port: u16) -> usize {
    get_json(port, "/api/objects")
        .await
        .as_array()
        .unwrap()
        .len()
}

#[tokio::test]
async fn routed_pipe_sweeps_the_polyline_and_records_the_route() {
    let port = test_server().await;
    let (status, body) = post_json(
        port,
        "/api/genrule",
        serde_json::json!({
            "genrule": "pipe",
            "name": "Riser",
            "radius": 0.15,
            "length": 1.0,
            "path": { "points": [[0.0, 0.0, 0.0], [6.0, 0.0, 0.0], [6.0, 6.0, 0.0]], "closed": false }
        }),
    )
    .await;
    assert_eq!(status, 200, "pipe route failed: {}", body);
    let obj = json_of(&body);
    assert_eq!(obj["object_type"], 305, "ObjectType::PipePlant");
    let (w, d, _h) = bbox_size(&obj);
    assert!(w > 6.0, "x extent covers the route: {}", w);
    assert!(d > 6.0, "y extent covers the bend: {}", d);
    assert_eq!(
        prop(&obj, "route_points")
            .and_then(|v| v.as_str())
            .is_some(),
        true,
        "route must be recorded"
    );
    assert_eq!(
        prop(&obj, "pipe_radius").and_then(|v| v.as_str()),
        Some("0.15")
    );
    let sweep = &obj["features"][0]["path_sweep"];
    assert_eq!(
        sweep["section"]["Circular"]["radius"].as_f64(),
        Some(0.15),
        "routed pipe must be a PathSweep circular feature: {}",
        obj["features"]
    );
    let len: f64 = prop(&obj, "route_length")
        .and_then(|v| v.as_str())
        .unwrap()
        .parse()
        .unwrap();
    assert!((len - 12.0).abs() < 1e-9, "route length 6+6: {}", len);
}

#[tokio::test]
async fn legacy_straight_pipe_still_works_without_a_path() {
    let port = test_server().await;
    let (status, body) = post_json(
        port,
        "/api/genrule",
        serde_json::json!({ "genrule": "pipe", "name": "Straight", "length": 10.0, "radius": 0.2 }),
    )
    .await;
    assert_eq!(status, 200, "{}", body);
    let obj = json_of(&body);
    let (w, _d, _h) = bbox_size(&obj);
    assert!((w - 10.0).abs() < 1e-6, "legacy pipe length: {}", w);
    assert!(prop(&obj, "route_points").is_none());
}

#[tokio::test]
async fn routed_pipe_requires_two_points_and_positive_radius() {
    let port = test_server().await;
    let before = object_count(port).await;

    let (status, body) = post_json(
        port,
        "/api/genrule",
        serde_json::json!({
            "genrule": "pipe", "name": "Bad-1", "radius": 0.1, "length": 1.0,
            "path": { "points": [[0.0, 0.0, 0.0]], "closed": false }
        }),
    )
    .await;
    assert_eq!(status, 400, "{}", body);

    let (status, body) = post_json(
        port,
        "/api/genrule",
        serde_json::json!({
            "genrule": "pipe", "name": "Bad-2", "radius": 0.0, "length": 1.0,
            "path": { "points": [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]], "closed": false }
        }),
    )
    .await;
    assert_eq!(status, 400, "{}", body);
    assert!(body.contains("positive"), "{}", body);

    let (status, body) = post_json(
        port,
        "/api/genrule",
        serde_json::json!({
            "genrule": "pipe", "name": "Bad-3", "radius": 0.1, "length": 1.0,
            "path": { "points": [[0.0, 0.0, 0.0], [0.0, 0.0, 0.0], [1.0, 0.0, 0.0]], "closed": false }
        }),
    )
    .await;
    assert_eq!(status, 400, "{}", body);

    assert_eq!(
        object_count(port).await,
        before,
        "rejections must not leave orphan objects"
    );
}

#[tokio::test]
async fn routed_pipe_is_undoable() {
    let port = test_server().await;
    let (status, body) = post_json(
        port,
        "/api/genrule",
        serde_json::json!({
            "genrule": "pipe", "name": "Undo-me", "radius": 0.1, "length": 1.0,
            "path": { "points": [[0.0, 0.0, 0.0], [3.0, 0.0, 0.0]], "closed": false }
        }),
    )
    .await;
    assert_eq!(status, 200, "{}", body);
    let id = json_of(&body)["id"].as_str().unwrap().to_string();

    let (status, _) = post_json(port, "/api/undo", serde_json::json!({})).await;
    assert_eq!(status, 200);
    let objects = get_json(port, "/api/objects").await;
    assert!(
        !objects
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["id"] == id.as_str()),
        "undo removes the routed pipe"
    );
}
