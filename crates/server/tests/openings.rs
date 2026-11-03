//! Phase 8-1 (ROADMAP building): frame variants (leaves/mullions), sill &
//! lintel extras and the door/window schedule endpoint.

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
    let project: SharedProject = Arc::new(RwLock::new(Project::new("Openings Test")));
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

async fn create_wall(port: u16, name: &str) -> serde_json::Value {
    let client = reqwest::Client::new();
    let resp = client
        .post(url(port, "/api/objects"))
        .json(&serde_json::json!({ "object_type": "wall", "name": name }))
        .send()
        .await
        .unwrap();
    let text = resp.text().await.unwrap();
    serde_json::from_str(&text).unwrap_or_else(|_| panic!("wall failed: {}", text))
}

async fn place(port: u16, kind: &str, wall: &str, extra: serde_json::Value) -> (u16, serde_json::Value) {
    let client = reqwest::Client::new();
    let height = if kind == "door" { 2.1 } else { 1.5 };
    let mut body = serde_json::json!({
        "genrule": kind,
        "name": format!("{}-01", kind),
        "wall_id": wall,
        "opening_width": 1.2,
        "opening_height": height,
        "position_x": 1.5,
        "position_z": 0.0,
    });
    if let serde_json::Value::Object(map) = extra {
        for (k, v) in map {
            body[k] = v;
        }
    }
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

fn prop(obj: &serde_json::Value, name: &str) -> Option<String> {
    obj["properties"]
        .as_array()?
        .iter()
        .find(|p| p["name"] == name)
        .and_then(|p| p["value"]["value_type"]["StringValue"].as_str())
        .map(str::to_string)
}

#[tokio::test]
async fn double_door_and_sill_window_record_variants() {
    let (port, _) = test_server().await;
    let wall = create_wall(port, "W-O1").await;
    let wall_id = wall["id"].as_str().unwrap().to_string();

    // Double door with a lintel.
    let (status, door) = place(
        port,
        "door",
        &wall_id,
        serde_json::json!({ "leaf_count": 2, "with_lintel": true, "name": "D-2L" }),
    )
    .await;
    assert_eq!(status, 200, "{}", door);
    assert_eq!(prop(&door, "leaf_count").as_deref(), Some("2"));
    assert_eq!(prop(&door, "with_lintel").as_deref(), Some("true"));

    // Another double door of the same size: the schedule groups both.
    let (status, second) = place(
        port,
        "door",
        &wall_id,
        serde_json::json!({ "leaf_count": 2, "with_lintel": true, "name": "D-2L-b" }),
    )
    .await;
    assert_eq!(status, 200, "{}", second);

    // Window with sill.
    let (status, win) = place(
        port,
        "window",
        &wall_id,
        serde_json::json!({ "with_sill": true, "name": "W-S" }),
    )
    .await;
    assert_eq!(status, 200, "{}", win);
    assert_eq!(prop(&win, "with_sill").as_deref(), Some("true"));

    // Schedule grouped by (type, size, frame props).
    let client = reqwest::Client::new();
    let rows = client
        .get(url(port, &format!("/api/schedules/openings?root_id={}", wall_id)))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let arr = rows.as_array().unwrap();
    assert_eq!(arr.len(), 2, "{}", rows);
    let door_row = arr.iter().find(|r| r["object_type"] == "door").unwrap();
    assert_eq!(door_row["count"], 2, "grouped duplicate doors");
    assert_eq!(door_row["leaf_count"], 2 as f64);
    assert_eq!(door_row["width_mm"], 1200);
    let win_row = arr.iter().find(|r| r["object_type"] == "window").unwrap();
    assert_eq!(win_row["count"], 1);
    assert_eq!(win_row["with_sill"], true);

    // Unknown root → 404.
    let missing = client
        .get(url(port, "/api/schedules/openings?root_id=nope"))
        .send()
        .await
        .unwrap();
    assert_eq!(missing.status().as_u16(), 404);
}
