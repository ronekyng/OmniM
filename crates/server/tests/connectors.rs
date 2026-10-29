//! P8-S4 acceptance tests — routed-pipe instance connectors, elbow fittings
//! and connector binding in `connect()`.

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
    let project: SharedProject = Arc::new(RwLock::new(Project::new("Connector Test")));
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

fn route_json(points: &[[f64; 3]], radius: f64, name: &str) -> serde_json::Value {
    serde_json::json!({
        "genrule": "pipe",
        "name": name,
        "radius": radius,
        "length": 1.0,
        "path": { "points": points, "closed": false }
    })
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-6
}

#[tokio::test]
async fn routed_pipe_carries_two_outward_connectors() {
    let port = test_server().await;
    let (status, body) = post_json(
        port,
        "/api/genrule",
        route_json(&[[0.0, 0.0, 0.0], [5.0, 0.0, 0.0]], 0.1, "P-run"),
    )
    .await;
    assert_eq!(status, 200, "{}", body);
    let connectors = &json_of(&body)["instance_connectors"];
    let arr = connectors.as_array().expect("instance connectors");
    assert_eq!(arr.len(), 2);
    let start = arr.iter().find(|c| c["id"] == "start").unwrap();
    let end = arr.iter().find(|c| c["id"] == "end").unwrap();
    assert_eq!(start["position"][0], 0.0);
    assert_eq!(end["position"][0], 5.0);
    // Outward directions: start points -X, end points +X.
    assert!(close(start["direction"][0].as_f64().unwrap(), -1.0));
    assert!(close(end["direction"][0].as_f64().unwrap(), 1.0));
    assert!(close(start["bore"].as_f64().unwrap(), 0.2));
    assert_eq!(start["connection_type_code"], "pipe");
}

#[tokio::test]
async fn connect_binds_the_nearest_connector_pair_and_records_the_points() {
    let port = test_server().await;
    let (_, a_body) = post_json(
        port,
        "/api/genrule",
        route_json(&[[0.0, 0.0, 0.0], [5.0, 0.0, 0.0]], 0.1, "P-a"),
    )
    .await;
    let (_, b_body) = post_json(
        port,
        "/api/genrule",
        route_json(&[[5.0, 0.0, 0.0], [10.0, 0.0, 0.0]], 0.1, "P-b"),
    )
    .await;
    let a = json_of(&a_body)["id"].as_str().unwrap().to_string();
    let b = json_of(&b_body)["id"].as_str().unwrap().to_string();

    let (status, body) = post_json(
        port,
        &format!(
            "/api/objects/{}/connect/{}?connection_type=mechanical",
            a, b
        ),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, 200, "{}", body);

    let project = get_json(port, "/api/project").await;
    let conn = &project["relations"]["connections"].as_array().unwrap()[0];
    assert_eq!(conn["from_object_id"], a.as_str());
    assert_eq!(conn["to_object_id"], b.as_str());
    assert_eq!(conn["connection_detail"], "from:end;to:start");
    assert!(close(conn["from_point"]["x"].as_f64().unwrap(), 5.0));
    assert!(close(conn["from_point"]["y"].as_f64().unwrap(), 0.0));
    assert!(close(conn["to_point"]["x"].as_f64().unwrap(), 5.0));
    assert_eq!(conn["type"], 2, "mechanical connection type");
}

#[tokio::test]
async fn connect_rejects_bore_mismatch() {
    let port = test_server().await;
    let (_, a_body) = post_json(
        port,
        "/api/genrule",
        route_json(&[[0.0, 0.0, 0.0], [5.0, 0.0, 0.0]], 0.1, "P-small"),
    )
    .await;
    let (_, b_body) = post_json(
        port,
        "/api/genrule",
        route_json(&[[5.0, 0.0, 0.0], [10.0, 0.0, 0.0]], 0.2, "P-large"),
    )
    .await;
    let a = json_of(&a_body)["id"].as_str().unwrap().to_string();
    let b = json_of(&b_body)["id"].as_str().unwrap().to_string();

    let (status, body) = post_json(
        port,
        &format!("/api/objects/{}/connect/{}", a, b),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, 400, "{}", body);
    assert!(body.contains("bore mismatch"), "{}", body);
}

#[tokio::test]
async fn bends_produce_elbow_fittings_parented_to_the_pipe() {
    let port = test_server().await;
    let (status, body) = post_json(
        port,
        "/api/genrule",
        route_json(
            &[[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [4.0, 4.0, 0.0]],
            0.1,
            "P-bend",
        ),
    )
    .await;
    assert_eq!(status, 200, "{}", body);
    let pipe = json_of(&body);
    let pipe_id = pipe["id"].as_str().unwrap().to_string();

    let objects = get_json(port, "/api/objects").await;
    let fittings: Vec<&serde_json::Value> = objects
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["object_type"] == 307)
        .collect();
    assert_eq!(fittings.len(), 1, "one internal bend → one elbow fitting");
    assert_eq!(fittings[0]["parent_id"], pipe_id.as_str());
    assert!(fittings[0]["properties"]
        .as_array()
        .unwrap()
        .iter()
        .any(|p| p["name"] == "fitting_type"));
}

#[tokio::test]
async fn pipe_plus_fittings_is_one_undo_step() {
    let port = test_server().await;
    let (status, body) = post_json(
        port,
        "/api/genrule",
        route_json(
            &[[0.0, 0.0, 0.0], [4.0, 0.0, 0.0], [4.0, 4.0, 0.0]],
            0.1,
            "P-undo",
        ),
    )
    .await;
    assert_eq!(status, 200, "{}", body);
    let pipe_id = json_of(&body)["id"].as_str().unwrap().to_string();

    let (status, _) = post_json(port, "/api/undo", serde_json::json!({})).await;
    assert_eq!(status, 200);
    let objects = get_json(port, "/api/objects").await;
    assert!(!objects
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o["id"] == pipe_id.as_str()));
    assert!(
        !objects
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o["object_type"] == 307),
        "the elbow fitting is removed in the same undo step"
    );
}
