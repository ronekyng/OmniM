//! Phase 8-2 (ROADMAP civil): terrain TIN, elevation query, grid earthwork and
//! full-alignment bridge/tunnel generation.

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
    let project: SharedProject = Arc::new(RwLock::new(Project::new("Civil Test")));
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

async fn post(port: u16, body: serde_json::Value) -> (u16, serde_json::Value) {
    let client = reqwest::Client::new();
    let resp = client
        .post(url(port, "/api/genrule"))
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap();
    (
        status,
        serde_json::from_str(&text).unwrap_or(serde_json::Value::String(text)),
    )
}

/// Survey points on a 10×10 m pad: corners z=1, a raised hill at the centre
/// (z=3), and two lateral mids at 2.
fn pad_points(hill: bool) -> Vec<[f64; 3]> {
    let mut pts = vec![
        [0.0, 0.0, 1.0],
        [10.0, 0.0, 1.0],
        [10.0, 10.0, 1.0],
        [0.0, 10.0, 1.0],
        [5.0, 0.0, 2.0],
        [10.0, 5.0, 2.0],
        [5.0, 10.0, 2.0],
        [0.0, 5.0, 2.0],
    ];
    if hill {
        pts.push([5.0, 5.0, 3.0]);
    }
    pts
}

#[tokio::test]
async fn terrain_fitting_stores_tin_and_answers_elevation() {
    let (port, _) = test_server().await;
    let (status, terrain) = post(
        port,
        serde_json::json!({
            "genrule": "terrain",
            "name": "Pad-1",
            "points": pad_points(true)
        }),
    )
    .await;
    assert_eq!(status, 200, "{}", terrain);
    assert_eq!(terrain["object_type"], 210, "{}", terrain);
    let id = terrain["id"].as_str().unwrap().to_string();

    // Fast elevation lookup: centre is a vertex → z = 3.
    let client = reqwest::Client::new();
    let at = client
        .get(url(port, "/api/terrain/{id}/elevation"))
        .query(&[("id", "x"), ("y", "5")])
        .send()
        .await;
    let _ = at;
    let resp = client
        .get(url(port, &format!("/api/terrain/{}/elevation", id)))
        .query(&[("x", "5"), ("y", "5")])
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let z = resp.json::<serde_json::Value>().await.unwrap();
    assert_eq!(z["elevation"], 3.0, "{}", z);

    // Outside the hull → 404.
    let outside = client
        .get(url(port, &format!("/api/terrain/{}/elevation", id)))
        .query(&[("x", "20"), ("y", "20")])
        .send()
        .await
        .unwrap();
    assert_eq!(outside.status().as_u16(), 404);
}

#[tokio::test]
async fn earthwork_balances_cut_fill_on_the_grid() {
    let (port, _) = test_server().await;
    // Design level 2.0 on the hill pad: the hill (z=3) becomes cut.
    let client = reqwest::Client::new();
    let resp = client
        .post(url(port, "/api/earthwork"))
        .json(&serde_json::json!({
            "cell": 1.0,
            "design_z": 2.0,
            "points": pad_points(true)
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let bal = resp.json::<serde_json::Value>().await.unwrap();
    // The hill cell(s) reaching 3.0 contribute cut; corners at 1.0 fill.
    let cut = bal["cut_volume_m3"].as_f64().unwrap();
    let fill = bal["fill_volume_m3"].as_f64().unwrap();
    assert!(cut > 0.0, "{}", bal);
    assert!(fill > 0.0, "{}", bal);
    let net = bal["net_volume_m3"].as_f64().unwrap();
    assert!((net - (cut - fill)).abs() < 1e-6);

    // Grid dimensions + coverage sanity.
    assert_eq!(bal["grid_cols"].as_u64().unwrap(), 10);
    assert_eq!(bal["grid_rows"].as_u64().unwrap(), 10);
    assert!(bal["outside_cells"].as_u64().unwrap() + bal["sampled_cells"].as_u64().unwrap() == 100);

    // Validation: bad cell / no input.
    let bad = client
        .post(url(port, "/api/earthwork"))
        .json(&serde_json::json!({ "cell": 0.0, "design_z": 0.0, "points": pad_points(false) }))
        .send()
        .await
        .unwrap();
    assert_eq!(bad.status().as_u16(), 400);
    let none = client
        .post(url(port, "/api/earthwork"))
        .json(&serde_json::json!({ "cell": 1.0, "design_z": 0.0 }))
        .send()
        .await
        .unwrap();
    assert_eq!(none.status().as_u16(), 400);
}

#[tokio::test]
async fn bridge_and_tunnel_follow_the_full_alignment() {
    let (port, _) = test_server().await;

    // Bridge deck along a polyline (previously: only the first→last segment).
    let (status, bridge) = post(
        port,
        serde_json::json!({
            "genrule": "bridge",
            "name": "BR-2",
            "width": 2.0,
            "thickness": 0.3,
            "path": { "points": [[0.0, 0.0, 5.0], [4.0, 0.0, 5.0], [4.0, 6.0, 5.0]] }
        }),
    )
    .await;
    assert_eq!(status, 200, "{}", bridge);
    assert_eq!(bridge["object_type"], 201, "{}", bridge);
    assert!(bridge["geometry"]["bbox"]["min"].is_object(), "{}", bridge);

    // Tunnel tube along the same plotted polyline.
    let (status, tunnel) = post(
        port,
        serde_json::json!({
            "genrule": "tunnel",
            "name": "TN-1",
            "radius": 1.2,
            "path": { "points": [[0.0, 0.0, 0.0], [5.0, 0.0, 0.0], [5.0, 5.0, 0.0]] }
        }),
    )
    .await;
    assert_eq!(status, 200, "{}", tunnel);
    assert_eq!(tunnel["object_type"], 202, "{}", tunnel);

    // Domain: invalid alignment shape is 400.
    let (status, _) = post(
        port,
        serde_json::json!({
            "genrule": "tunnel",
            "name": "bad",
            "radius": -1.0
        }),
    )
    .await;
    assert_eq!(status, 400);
}
