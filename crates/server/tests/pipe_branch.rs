//! Phase 8-3 (ROADMAP): branch a routed pipe — tee fitting + second pipe.

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
    let project: SharedProject = Arc::new(RwLock::new(Project::new("Branch Test")));
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

async fn objects(port: u16) -> Vec<serde_json::Value> {
    reqwest::get(url(port, "/api/objects"))
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

async fn create_run_pipe(port: u16) -> serde_json::Value {
    let client = reqwest::Client::new();
    let resp = client
        .post(url(port, "/api/genrule"))
        .json(&serde_json::json!({
            "genrule": "pipe",
            "name": "Run-1",
            "length": 10.0,
            "radius": 0.15,
            "path": {
                "points": [[0.0, 0.0, 2.0], [6.0, 0.0, 2.0], [6.0, 4.0, 2.0]],
                "closed": false
            }
        }))
        .send()
        .await
        .unwrap();
    let text = resp.text().await.unwrap();
    serde_json::from_str(&text).unwrap_or_else(|_| panic!("run pipe failed: {}", text))
}

async fn branch(port: u16, body: serde_json::Value) -> (u16, serde_json::Value) {
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

fn branch_of(host: &str, point: [f64; 3], target: [f64; 3]) -> serde_json::Value {
    serde_json::json!({
        "genrule": "pipe",
        "name": "BR-1",
        "branch_of": host,
        "branch_point": point,
        "target": target
    })
}

#[tokio::test]
async fn branch_creates_tee_and_routed_branch_pipe() {
    let (port, _) = test_server().await;
    let host = create_run_pipe(port).await;
    let host_id = host["id"].as_str().unwrap().to_string();

    // Branch on the run's second segment (x=6 plane) — snapped to the route.
    let (status, branch_obj) =
        branch(port, branch_of(&host_id, [6.0, 2.0, 2.0], [6.0, 2.4, 4.0])).await;
    assert_eq!(status, 200, "{}", branch_obj);
    let branch_id = branch_obj["id"].as_str().unwrap().to_string();

    // Branch route: [snapped junction, target] with length ≈ 2.062.
    let points: serde_json::Value = branch_obj["properties"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["name"] == "route_points")
        .expect("route_points property")["value"]["value_type"]["StringValue"]
        .as_str()
        .map(|s| serde_json::from_str::<serde_json::Value>(s).unwrap())
        .unwrap();
    let pts = points.as_array().unwrap();
    assert_eq!(pts.len(), 2);
    assert_eq!(pts[0][0], 6.0, "snapped onto the x=6 segment");
    assert_eq!(pts[0][1], 2.0);
    assert_eq!(pts[1][2], 4.0);

    let all = objects(port).await;
    // Tee fitting exists under the host as a child with branch_pipe_id.
    let tee = all
        .iter()
        .find(|o| {
            o["parent_id"] == host_id.as_str()
                && o["name"].as_str().unwrap_or_default().ends_with("-tee")
        })
        .expect("tee fitting object");
    let props = tee["properties"].as_array().unwrap();
    let get_prop = |name: &str| -> Option<String> {
        props
            .iter()
            .find(|p| p["name"] == name)
            .and_then(|p| p["value"]["value_type"]["StringValue"].as_str())
            .map(str::to_string)
    };
    assert_eq!(get_prop("fitting_type").as_deref(), Some("tee"));
    assert_eq!(
        get_prop("branch_pipe_id").as_deref(),
        Some(branch_id.as_str())
    );

    // Branch pipe is parented to the host pipe.
    let bp = all
        .iter()
        .find(|o| o["id"] == branch_id.as_str())
        .expect("branch pipe object");
    assert_eq!(bp["parent_id"], host_id.as_str());

    // One undo step removes branch + tee; one redo restores both.
    let client = reqwest::Client::new();
    let before = objects(port).await.len();
    client.post(url(port, "/api/undo")).send().await.unwrap();
    assert_eq!(objects(port).await.len(), before - 2);
    client.post(url(port, "/api/redo")).send().await.unwrap();
    assert_eq!(objects(port).await.len(), before);
}

#[tokio::test]
async fn branch_validation_rejects_bad_domains() {
    let (port, _) = test_server().await;
    let host = create_run_pipe(port).await;
    let host_id = host["id"].as_str().unwrap().to_string();

    // Unknown host.
    let (status, _) = branch(port, branch_of("ghost", [0.0, 0.0, 0.0], [1.0, 0.0, 0.0])).await;
    assert_eq!(status, 404);

    // Off-route branch point (>0.1 m).
    let (status, body) = branch(port, branch_of(&host_id, [0.0, 0.5, 2.0], [1.0, 0.5, 2.0])).await;
    assert_eq!(status, 400, "{}", body);
    assert!(
        body.as_str()
            .unwrap_or_default()
            .contains("off the host route"),
        "{}",
        body
    );

    // target == branch_point.
    let (status, _) = branch(port, branch_of(&host_id, [3.0, 0.0, 2.0], [3.0, 0.0, 2.0])).await;
    assert_eq!(status, 400);

    // Missing branch fields.
    let (status, body) = branch(
        port,
        serde_json::json!({ "genrule": "pipe", "name": "x", "branch_of": host_id }),
    )
    .await;
    assert_eq!(status, 400, "{}", body);
}
