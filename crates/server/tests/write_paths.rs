//! Endpoint-level undo coverage for migrated write handlers (Wave 1, Slice 2).
//!
//! Pattern per case: set up state → mutate → assert the change took →
//! `POST /api/undo` → assert the previous state is restored.

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

async fn test_server() -> (u16, SharedProject) {
    init_library();
    let project: SharedProject = Arc::new(RwLock::new(Project::new("Undo Test")));
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

async fn create_wall(port: u16, name: &str) -> String {
    let client = reqwest::Client::new();
    let resp = client
        .post(api_url(port, "/api/objects"))
        .json(&serde_json::json!({ "object_type": "wall", "name": name }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "create wall failed: {}",
        resp.text().await.unwrap()
    );
    let obj: serde_json::Value = resp.json().await.unwrap();
    obj["id"].as_str().unwrap().to_string()
}

async fn undo_n(port: u16, times: usize) {
    let client = reqwest::Client::new();
    for _ in 0..times {
        let resp = client
            .post(api_url(port, "/api/undo"))
            .send()
            .await
            .unwrap();
        assert!(
            resp.status().is_success(),
            "undo failed: {}",
            resp.text().await.unwrap()
        );
    }
}

async fn undo(port: u16) {
    undo_n(port, 1).await;
}

async fn redo(port: u16) {
    let client = reqwest::Client::new();
    let resp = client
        .post(api_url(port, "/api/redo"))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "redo failed: {}",
        resp.text().await.unwrap()
    );
}

async fn get_object(port: u16, id: &str) -> serde_json::Value {
    let client = reqwest::Client::new();
    let resp = client
        .get(api_url(port, &format!("/api/objects/{}", id)))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "get object failed");
    resp.json().await.unwrap()
}

#[tokio::test]
async fn rename_is_undoable() {
    let (port, _) = test_server().await;
    let id = create_wall(port, "W-01").await;
    let client = reqwest::Client::new();

    let resp = client
        .put(api_url(port, &format!("/api/objects/{}", id)))
        .json(&serde_json::json!({ "name": "W-02" }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "rename failed: {}",
        resp.text().await.unwrap()
    );
    assert_eq!(get_object(port, &id).await["name"], "W-02");

    undo(port).await;
    assert_eq!(get_object(port, &id).await["name"], "W-01");
}

#[tokio::test]
async fn property_set_is_undoable() {
    let (port, _) = test_server().await;
    let id = create_wall(port, "W-01").await;
    let client = reqwest::Client::new();

    let resp = client
        .put(api_url(port, &format!("/api/objects/{}/prop", id)))
        .json(&serde_json::json!({ "name": "FireRating", "value": "120" }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "set prop failed: {}",
        resp.text().await.unwrap()
    );
    let find_prop = |v: &serde_json::Value| -> Option<serde_json::Value> {
        v["properties"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["name"] == "FireRating")
            .map(|p| p["value"]["value_type"]["StringValue"].clone())
    };
    assert_eq!(find_prop(&get_object(port, &id).await).unwrap(), "120");

    undo(port).await;
    // Old value was absent, so undo removes the property.
    assert_eq!(find_prop(&get_object(port, &id).await), None);
}

#[tokio::test]
async fn connect_is_undoable() {
    let (port, project) = test_server().await;
    let a = create_wall(port, "A").await;
    let b = create_wall(port, "B").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(api_url(port, &format!("/api/objects/{}/connect/{}", a, b)))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "connect failed: {}",
        resp.text().await.unwrap()
    );

    let count_conns = |project: SharedProject| {
        let a = a.clone();
        let b = b.clone();
        async move {
            let p = project.read().await;
            let js = serde_json::to_value(&*p).unwrap();
            js["relations"]["connections"]
                .as_array()
                .map(|conns| {
                    conns.iter().any(|c| {
                        c["from_object_id"].as_str().unwrap_or("") == a
                            && c["to_object_id"].as_str().unwrap_or("") == b
                    })
                })
                .unwrap_or(false)
        }
    };
    assert!(
        count_conns(project.clone()).await,
        "connection was not added"
    );
    undo(port).await;
    assert!(
        !count_conns(project.clone()).await,
        "connect was not undone"
    );
}

#[tokio::test]
async fn set_parent_is_undoable() {
    let (port, _) = test_server().await;
    let child = create_wall(port, "Child").await;
    let parent = create_wall(port, "Parent").await;
    let client = reqwest::Client::new();

    let resp = client
        .put(api_url(port, &format!("/api/objects/{}/parent", child)))
        .json(&serde_json::json!({ "parent_id": parent }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "set parent failed: {}",
        resp.text().await.unwrap()
    );
    assert_eq!(
        get_object(port, &child).await["parent_id"],
        get_object(port, &parent).await["id"]
    );

    undo(port).await; // removes the containment (last command first)
    undo(port).await; // restores the parent_id
    assert_eq!(
        get_object(port, &child).await["parent_id"],
        serde_json::Value::Null
    );
}

#[tokio::test]
async fn material_create_is_undoable() {
    let (port, _) = test_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(api_url(port, "/api/materials"))
        .json(&serde_json::json!({ "id": "mat-brass", "name": "Brass", "category": "metal" }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "create material failed: {}",
        resp.text().await.unwrap()
    );

    let mats: serde_json::Value = client
        .get(api_url(port, "/api/materials"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(mats
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m["id"] == "mat-brass"));

    undo(port).await;
    let mats: serde_json::Value = client
        .get(api_url(port, "/api/materials"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        !mats
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["id"] == "mat-brass"),
        "material was not removed by undo"
    );
}

#[tokio::test]
async fn material_assign_is_undoable() {
    let (port, _) = test_server().await;
    let id = create_wall(port, "W-01").await;
    let client = reqwest::Client::new();

    // Seeded default material from the built-in catalogue.
    let mats: serde_json::Value = client
        .get(api_url(port, "/api/materials"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let mat_id = mats[0]["id"].as_str().unwrap().to_string();

    let resp = client
        .put(api_url(port, &format!("/api/objects/{}/material", id)))
        .json(&serde_json::json!({ "material_id": mat_id }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "assign material failed: {}",
        resp.text().await.unwrap()
    );
    assert_eq!(get_object(port, &id).await["material_id"], mat_id);

    undo(port).await;
    assert_eq!(
        get_object(port, &id).await["material_id"],
        serde_json::Value::Null
    );
}

#[tokio::test]
async fn transform_is_undoable() {
    let (port, _) = test_server().await;
    let client = reqwest::Client::new();

    // Create a wall with real mesh geometry via genrule.
    let resp = client
        .post(api_url(port, "/api/genrule"))
        .json(&serde_json::json!({
            "genrule": "wall",
            "name": "TW",
            "length": 2.0,
            "height": 1.0,
            "core_thickness": 0.15,
            "finish_thickness": 0.02,
            "start_x": 0.0, "start_y": 0.0, "start_z": 0.0,
            "end_x": 2.0, "end_y": 0.0, "end_z": 0.0
        }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "genrule wall failed: {}",
        resp.text().await.unwrap()
    );
    let obj: serde_json::Value = resp.json().await.unwrap();
    let id = obj["id"].as_str().unwrap().to_string();

    let before = get_object(port, &id).await;
    let resp = client
        .put(api_url(port, &format!("/api/objects/{}/transform", id)))
        .json(&serde_json::json!({
            "translation": { "x": 0.0, "y": 0.0, "z": 5.0 }
        }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "transform failed: {}",
        resp.text().await.unwrap()
    );
    let after = get_object(port, &id).await;
    assert_ne!(
        before["geometry"]["bbox"], after["geometry"]["bbox"],
        "transform had no effect on bbox"
    );

    undo(port).await;
    let restored = get_object(port, &id).await;
    assert_eq!(
        before["geometry"]["bbox"], restored["geometry"]["bbox"],
        "transform was not undone"
    );
}

#[tokio::test]
async fn transform_rejects_object_without_mesh() {
    let (port, _) = test_server().await;
    let id = create_wall(port, "NoMesh").await;
    let client = reqwest::Client::new();

    let resp = client
        .put(api_url(port, &format!("/api/objects/{}/transform", id)))
        .json(&serde_json::json!({ "translation": { "x": 1.0, "y": 0.0, "z": 0.0 } }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        400,
        "transform should reject objects without mesh geometry"
    );
}

// ── Wave 1 Slice 3: sketch / component / catalog ─────────────────────────────

async fn create_sketch(port: u16, name: &str) -> String {
    let client = reqwest::Client::new();
    let resp = client
        .post(api_url(port, "/api/sketches"))
        .json(&serde_json::json!({ "name": name }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "create sketch failed: {}",
        resp.text().await.unwrap()
    );
    resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn get_sketch(port: u16, id: &str) -> serde_json::Value {
    let client = reqwest::Client::new();
    client
        .get(api_url(port, &format!("/api/sketches/{}", id)))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

#[tokio::test]
async fn sketch_rename_is_undoable() {
    let (port, _) = test_server().await;
    let id = create_sketch(port, "S-01").await;
    let client = reqwest::Client::new();

    let resp = client
        .put(api_url(port, &format!("/api/sketches/{}", id)))
        .json(&serde_json::json!({ "name": "S-02" }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "rename sketch failed: {}",
        resp.text().await.unwrap()
    );
    assert_eq!(get_sketch(port, &id).await["name"], "S-02");

    undo(port).await;
    assert_eq!(get_sketch(port, &id).await["name"], "S-01");
}

#[tokio::test]
async fn sketch_entity_update_is_undoable() {
    let (port, _) = test_server().await;
    let id = create_sketch(port, "S").await;
    let client = reqwest::Client::new();

    let resp = client
        .post(api_url(port, &format!("/api/sketches/{}/entities", id)))
        .json(&serde_json::json!({
            "entity_type": "rectangle",
            "points": [
                { "x": 0.0, "y": 0.0, "z": 0.0 },
                { "x": 2.0, "y": 1.0, "z": 0.0 }
            ]
        }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "add entity failed: {}",
        resp.text().await.unwrap()
    );

    let before = get_sketch(port, &id).await["entities"][0]["points"].clone();
    let resp = client
        .put(api_url(port, &format!("/api/sketches/{}/entities/0", id)))
        .json(&serde_json::json!({
            "points": [
                { "x": 0.0, "y": 0.0, "z": 0.0 },
                { "x": 4.0, "y": 2.0, "z": 0.0 }
            ]
        }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "update entity failed: {}",
        resp.text().await.unwrap()
    );
    assert_ne!(get_sketch(port, &id).await["entities"][0]["points"], before);

    undo(port).await;
    assert_eq!(get_sketch(port, &id).await["entities"][0]["points"], before);
}

#[tokio::test]
async fn component_delete_is_undoable() {
    let (port, _) = test_server().await;
    let client = reqwest::Client::new();
    let resp = client
        .post(api_url(port, "/api/components"))
        .json(&serde_json::json!({ "name": "C-del" }))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let id: String = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = client
        .delete(api_url(port, &format!("/api/components/{}", id)))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "delete component failed: {}",
        resp.text().await.unwrap()
    );
    assert_eq!(
        client
            .get(api_url(port, &format!("/api/components/{}", id)))
            .send()
            .await
            .unwrap()
            .status(),
        404
    );

    undo(port).await;
    assert_eq!(
        client
            .get(api_url(port, &format!("/api/components/{}", id)))
            .send()
            .await
            .unwrap()
            .status(),
        200
    );
}

#[tokio::test]
async fn feature_add_update_remove_are_undoable() {
    let (port, _) = test_server().await;
    let client = reqwest::Client::new();
    let comp_id: String = client
        .post(api_url(port, "/api/components"))
        .json(&serde_json::json!({ "name": "C-feat" }))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    let sketch_id = create_sketch(port, "profile").await;

    // Add
    let feat: serde_json::Value = client
        .post(api_url(
            port,
            &format!("/api/components/{}/features", comp_id),
        ))
        .json(&serde_json::json!({
            "feature_type": "extrude",
            "sketch_id": sketch_id,
            "depth": 3.0
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let fid = feat["id"].as_str().unwrap().to_string();

    let features = |port: u16, comp: String| async move {
        let client = reqwest::Client::new();
        client
            .get(api_url(port, &format!("/api/components/{}/features", comp)))
            .send()
            .await
            .unwrap()
            .json::<Vec<serde_json::Value>>()
            .await
            .unwrap()
    };
    assert_eq!(features(port, comp_id.clone()).await.len(), 1);
    // add_feature executes [AddFeature, RebuildFeatures] → two undo entries.
    undo_n(port, 2).await;
    assert!(
        features(port, comp_id.clone()).await.is_empty(),
        "feature add was not undone"
    );

    // Re-add for update/remove coverage
    let feat: serde_json::Value = client
        .post(api_url(
            port,
            &format!("/api/components/{}/features", comp_id),
        ))
        .json(&serde_json::json!({
            "feature_type": "extrude",
            "sketch_id": sketch_id,
            "depth": 3.0
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let fid2 = feat["id"].as_str().unwrap().to_string();

    let resp = client
        .put(api_url(
            port,
            &format!("/api/components/{}/features/{}", comp_id, fid2),
        ))
        .json(&serde_json::json!({ "depth": 5.0 }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "update feature failed: {}",
        resp.text().await.unwrap()
    );
    let depth = features(port, comp_id.clone()).await[0]["depth"]
        .as_f64()
        .unwrap();
    assert_eq!(depth, 5.0);
    undo_n(port, 2).await;
    let depth = features(port, comp_id.clone()).await[0]["depth"]
        .as_f64()
        .unwrap();
    assert_eq!(depth, 3.0, "feature update was not undone");

    let resp = client
        .delete(api_url(
            port,
            &format!("/api/components/{}/features/{}", comp_id, fid2),
        ))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "remove feature failed: {}",
        resp.text().await.unwrap()
    );
    assert!(features(port, comp_id.clone()).await.is_empty());
    undo_n(port, 2).await;
    assert_eq!(
        features(port, comp_id.clone()).await.len(),
        1,
        "feature remove was not undone"
    );
    let _ = fid;
}

async fn create_def(port: u16, name: &str) -> String {
    let client = reqwest::Client::new();
    let resp = client
        .post(api_url(port, "/api/catalog"))
        .json(&serde_json::json!({ "name": name, "category": "metal" }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "create def failed: {}",
        resp.text().await.unwrap()
    );
    resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn get_def(port: u16, id: &str) -> Option<serde_json::Value> {
    let client = reqwest::Client::new();
    let resp = client
        .get(api_url(port, &format!("/api/catalog/{}", id)))
        .send()
        .await
        .unwrap();
    if resp.status().is_success() {
        Some(resp.json().await.unwrap())
    } else {
        None
    }
}

#[tokio::test]
async fn catalog_def_create_update_delete_are_undoable() {
    let (port, _) = test_server().await;
    let client = reqwest::Client::new();
    let id = create_def(port, "Def-undo").await;

    undo(port).await;
    assert!(
        get_def(port, &id).await.is_none(),
        "def create was not undone"
    );

    let id = create_def(port, "Def-undo2").await;
    let resp = client
        .put(api_url(port, &format!("/api/catalog/{}", id)))
        .json(&serde_json::json!({ "name": "Def-renamed" }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "update def failed: {}",
        resp.text().await.unwrap()
    );
    assert_eq!(get_def(port, &id).await.unwrap()["name"], "Def-renamed");
    undo(port).await;
    assert_eq!(get_def(port, &id).await.unwrap()["name"], "Def-undo2");

    let resp = client
        .delete(api_url(port, &format!("/api/catalog/{}", id)))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "delete def failed: {}",
        resp.text().await.unwrap()
    );
    assert!(get_def(port, &id).await.is_none());
    undo(port).await;
    assert!(
        get_def(port, &id).await.is_some(),
        "def delete was not undone"
    );
}

#[tokio::test]
async fn catalog_param_add_update_delete_are_undoable() {
    let (port, _) = test_server().await;
    let client = reqwest::Client::new();
    let id = create_def(port, "Def-params").await;

    let resp = client
        .post(api_url(port, &format!("/api/catalog/{}/params", id)))
        .json(&serde_json::json!({ "name": "width", "default_value": 1.0, "min_value": 0.0, "max_value": 10.0 }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "add param failed: {}",
        resp.text().await.unwrap()
    );
    assert_eq!(
        get_def(port, &id).await.unwrap()["params"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    undo(port).await;
    assert_eq!(
        get_def(port, &id).await.unwrap()["params"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    let resp = client
        .post(api_url(port, &format!("/api/catalog/{}/params", id)))
        .json(&serde_json::json!({ "name": "width", "default_value": 1.0, "min_value": 0.0, "max_value": 10.0 }))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());

    let resp = client
        .put(api_url(port, &format!("/api/catalog/{}/params/0", id)))
        .json(&serde_json::json!({ "default_value": 4.0 }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "update param failed: {}",
        resp.text().await.unwrap()
    );
    assert_eq!(
        get_def(port, &id).await.unwrap()["params"][0]["default_value"]
            .as_f64()
            .unwrap(),
        4.0
    );
    undo(port).await;
    assert_eq!(
        get_def(port, &id).await.unwrap()["params"][0]["default_value"]
            .as_f64()
            .unwrap(),
        1.0
    );

    let resp = client
        .delete(api_url(port, &format!("/api/catalog/{}/params/0", id)))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "delete param failed: {}",
        resp.text().await.unwrap()
    );
    assert_eq!(
        get_def(port, &id).await.unwrap()["params"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    undo(port).await;
    assert_eq!(
        get_def(port, &id).await.unwrap()["params"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[tokio::test]
async fn catalog_connector_and_feature_tree_are_undoable() {
    let (port, _) = test_server().await;
    let client = reqwest::Client::new();
    let id = create_def(port, "Def-conn").await;

    let resp = client
        .post(api_url(port, &format!("/api/catalog/{}/connectors", id)))
        .json(&serde_json::json!({ "connector_id": "c1", "connector_name": "End" }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "add connector failed: {}",
        resp.text().await.unwrap()
    );
    assert_eq!(
        get_def(port, &id).await.unwrap()["connectors"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    undo(port).await;
    assert_eq!(
        get_def(port, &id).await.unwrap()["connectors"]
            .as_array()
            .unwrap()
            .len(),
        0
    );

    let resp = client
        .put(api_url(port, &format!("/api/catalog/{}/feature-tree", id)))
        .json(&serde_json::json!({ "feature_tree_id": "tree-x" }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "assign tree failed: {}",
        resp.text().await.unwrap()
    );
    assert_eq!(
        get_def(port, &id).await.unwrap()["feature_tree_id"],
        "tree-x"
    );
    undo(port).await;
    assert_ne!(
        get_def(port, &id).await.unwrap()["feature_tree_id"],
        "tree-x"
    );
}

// ── Wave 1 Slice 4: series / lifecycle / measurement / admin / breakdown ─────

async fn create_series(port: u16, name: &str) -> String {
    let client = reqwest::Client::new();
    let resp = client
        .post(api_url(port, "/api/series"))
        .json(&serde_json::json!({
            "name": name,
            "description": "undo coverage",
            "domain": "structural",
            "standard_reference": "STD-1"
        }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "create series failed: {}",
        resp.text().await.unwrap()
    );
    resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string()
}

async fn get_series(port: u16, id: &str) -> Option<serde_json::Value> {
    let client = reqwest::Client::new();
    let resp = client
        .get(api_url(port, &format!("/api/series/{}", id)))
        .send()
        .await
        .unwrap();
    if resp.status().is_success() {
        Some(resp.json().await.unwrap())
    } else {
        None
    }
}

async fn add_selector(port: u16, series_id: &str, selector_id: &str) {
    let resp = reqwest::Client::new()
        .post(api_url(
            port,
            &format!("/api/series/{}/selectors", series_id),
        ))
        .json(&serde_json::json!({ "selector_id": selector_id, "question": "Which element?" }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        201,
        "add selector failed: {}",
        resp.text().await.unwrap()
    );
}

async fn add_answer(port: u16, series_id: &str, selector_id: &str, answer_id: &str) {
    let resp = reqwest::Client::new()
        .post(api_url(port, &format!("/api/series/{}/answers", series_id)))
        .json(&serde_json::json!({
            "selector_id": selector_id,
            "answer_id": answer_id,
            "answer_value": "beam"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        201,
        "add answer failed: {}",
        resp.text().await.unwrap()
    );
}

async fn add_spec(
    port: u16,
    series_id: &str,
    selector_id: &str,
    answer_id: &str,
    component_def_id: &str,
) {
    let resp = reqwest::Client::new()
        .post(api_url(port, &format!("/api/series/{}/spec", series_id)))
        .json(&serde_json::json!({
            "selector_id": selector_id,
            "answer_id": answer_id,
            "component_def_id": component_def_id,
            "material": "steel",
            "connection_type_code": "bolted"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        201,
        "add spec failed: {}",
        resp.text().await.unwrap()
    );
}

fn constraints(series: &serde_json::Value) -> Vec<serde_json::Value> {
    series["root_selector"]["answers"][0]["outcome"]["Component"]["param_constraints"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

async fn build_series_with_spec(port: u16, name: &str, component_def_id: &str) -> String {
    let id = create_series(port, name).await;
    add_selector(port, &id, "sel-1").await;
    add_answer(port, &id, "sel-1", "a-1").await;
    add_spec(port, &id, "sel-1", "a-1", component_def_id).await;
    id
}

#[tokio::test]
async fn series_create_update_delete_are_undoable() {
    let (port, _) = test_server().await;
    let client = reqwest::Client::new();

    let id = create_series(port, "Ser-create").await;
    assert!(get_series(port, &id).await.is_some());
    undo(port).await;
    assert!(
        get_series(port, &id).await.is_none(),
        "series create was not undone"
    );

    let id = create_series(port, "Ser-update").await;
    let resp = client
        .put(api_url(port, &format!("/api/series/{}", id)))
        .json(&serde_json::json!({ "name": "Ser-renamed", "description": "after" }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "update series failed: {}",
        resp.text().await.unwrap()
    );
    assert_eq!(get_series(port, &id).await.unwrap()["name"], "Ser-renamed");
    undo(port).await;
    let restored = get_series(port, &id).await.unwrap();
    assert_eq!(
        restored["name"], "Ser-update",
        "series update was not undone"
    );
    assert_eq!(restored["description"], "undo coverage");

    let resp = client
        .delete(api_url(port, &format!("/api/series/{}", id)))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        204,
        "delete series failed: {}",
        resp.text().await.unwrap()
    );
    assert!(get_series(port, &id).await.is_none());
    undo(port).await;
    assert!(
        get_series(port, &id).await.is_some(),
        "series delete was not undone"
    );
}

#[tokio::test]
async fn series_selector_answer_spec_are_undoable() {
    let (port, _) = test_server().await;

    let id = create_series(port, "Ser-tree").await;
    add_selector(port, &id, "sel-1").await;
    assert_eq!(
        get_series(port, &id).await.unwrap()["root_selector"]["id"],
        "sel-1"
    );
    undo(port).await;
    assert_eq!(
        get_series(port, &id).await.unwrap()["root_selector"],
        serde_json::Value::Null,
        "selector add was not undone"
    );

    add_selector(port, &id, "sel-1").await;
    add_answer(port, &id, "sel-1", "a-1").await;
    assert_eq!(
        get_series(port, &id).await.unwrap()["root_selector"]["answers"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    undo(port).await;
    assert_eq!(
        get_series(port, &id).await.unwrap()["root_selector"]["answers"]
            .as_array()
            .unwrap()
            .len(),
        0,
        "answer add was not undone"
    );

    add_answer(port, &id, "sel-1", "a-1").await;
    add_spec(port, &id, "sel-1", "a-1", "CD-spec").await;
    assert_eq!(
        get_series(port, &id).await.unwrap()["root_selector"]["answers"][0]["outcome"]["Component"]
            ["component_def_id"],
        "CD-spec"
    );
    undo(port).await;
    assert_eq!(
        get_series(port, &id).await.unwrap()["root_selector"]["answers"][0]["outcome"]["Component"]
            ["component_def_id"],
        "",
        "spec component add was not undone"
    );
}

#[tokio::test]
async fn series_param_constraint_add_update_delete_are_undoable() {
    let (port, _) = test_server().await;
    let client = reqwest::Client::new();
    let id = build_series_with_spec(port, "Ser-constraints", "CD-param").await;

    let add_body = serde_json::json!({
        "component_def_id": "CD-param",
        "param_name": "width",
        "default_value": 1.0,
        "min_value": 0.0,
        "max_value": 10.0,
        "step": 0.5
    });
    let resp = client
        .post(api_url(port, &format!("/api/series/{}/constraints", id)))
        .json(&add_body)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        201,
        "add constraint failed: {}",
        resp.text().await.unwrap()
    );
    let series = get_series(port, &id).await.unwrap();
    assert_eq!(constraints(&series).len(), 1);
    assert_eq!(constraints(&series)[0]["param_name"], "width");
    undo(port).await;
    assert_eq!(
        constraints(&get_series(port, &id).await.unwrap()).len(),
        0,
        "constraint add was not undone"
    );

    // Re-add, then update it.
    let resp = client
        .post(api_url(port, &format!("/api/series/{}/constraints", id)))
        .json(&add_body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);

    let resp = client
        .put(api_url(port, &format!("/api/series/{}/constraints/0", id)))
        .json(&serde_json::json!({ "component_def_id": "CD-param", "default_value": 4.0 }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "update constraint failed: {}",
        resp.text().await.unwrap()
    );
    assert_eq!(
        constraints(&get_series(port, &id).await.unwrap())[0]["default_value"]
            .as_f64()
            .unwrap(),
        4.0
    );
    undo(port).await;
    assert_eq!(
        constraints(&get_series(port, &id).await.unwrap())[0]["default_value"]
            .as_f64()
            .unwrap(),
        1.0,
        "constraint update was not undone"
    );

    let resp = client
        .delete(api_url(port, &format!("/api/series/{}/constraints/0", id)))
        .json(&serde_json::json!({ "component_def_id": "CD-param" }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        204,
        "delete constraint failed: {}",
        resp.text().await.unwrap()
    );
    assert_eq!(constraints(&get_series(port, &id).await.unwrap()).len(), 0);
    undo(port).await;
    assert_eq!(
        constraints(&get_series(port, &id).await.unwrap()).len(),
        1,
        "constraint delete was not undone"
    );
}

#[tokio::test]
async fn lifecycle_create_and_link_are_undoable() {
    let (port, _) = test_server().await;
    let client = reqwest::Client::new();
    let wall = create_wall(port, "LC-Wall").await;

    let resp = client
        .post(api_url(port, "/api/lifecycle"))
        .json(&serde_json::json!({
            "phase": "Design",
            "record_type": "review",
            "name": "LC-undo",
            "description": "undo coverage"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "create lifecycle failed: {}",
        resp.text().await.unwrap()
    );

    let lifecycle = |port: u16| async move {
        reqwest::Client::new()
            .get(api_url(port, "/api/lifecycle"))
            .send()
            .await
            .unwrap()
            .json::<Vec<serde_json::Value>>()
            .await
            .unwrap()
    };
    let record = lifecycle(port)
        .await
        .into_iter()
        .find(|r| r["name"] == "LC-undo")
        .expect("lifecycle record missing");
    let record_id = record["id"].as_str().unwrap().to_string();
    assert_eq!(record["phase"], 2); // DESIGN

    let resp = client
        .post(api_url(
            port,
            &format!("/api/lifecycle/{}/link/{}", record_id, wall),
        ))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "link record failed: {}",
        resp.text().await.unwrap()
    );
    let linked = lifecycle(port)
        .await
        .into_iter()
        .find(|r| r["id"] == record_id)
        .unwrap();
    assert!(linked["related_object_ids"]
        .as_array()
        .unwrap()
        .iter()
        .any(|id| id == &wall));

    undo(port).await;
    let unlinked = lifecycle(port)
        .await
        .into_iter()
        .find(|r| r["id"] == record_id)
        .unwrap();
    assert!(
        !unlinked["related_object_ids"]
            .as_array()
            .unwrap()
            .iter()
            .any(|id| id == &wall),
        "record link was not undone"
    );

    undo(port).await;
    assert!(
        !lifecycle(port).await.iter().any(|r| r["id"] == record_id),
        "lifecycle create was not undone"
    );
}

#[tokio::test]
async fn measurement_create_delete_are_undoable() {
    let (port, _) = test_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(api_url(port, "/api/measurements"))
        .json(&serde_json::json!({
            "type": "distance",
            "points": [
                { "x": 0.0, "y": 0.0, "z": 0.0 },
                { "x": 3.0, "y": 4.0, "z": 0.0 }
            ],
            "value": 5.0,
            "label": "M-undo",
            "color": "#44ff88"
        }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "create measurement failed: {}",
        resp.text().await.unwrap()
    );
    let id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let measurements = |port: u16| async move {
        reqwest::Client::new()
            .get(api_url(port, "/api/measurements"))
            .send()
            .await
            .unwrap()
            .json::<Vec<serde_json::Value>>()
            .await
            .unwrap()
    };
    assert!(measurements(port).await.iter().any(|m| m["id"] == id));
    undo(port).await;
    assert!(
        !measurements(port).await.iter().any(|m| m["id"] == id),
        "measurement create was not undone"
    );

    let resp = client
        .post(api_url(port, "/api/measurements"))
        .json(&serde_json::json!({
            "type": "angular",
            "points": [],
            "value": 90.0,
            "label": "M-delete"
        }))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = client
        .delete(api_url(port, &format!("/api/measurements/{}", id)))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "delete measurement failed: {}",
        resp.text().await.unwrap()
    );
    assert!(!measurements(port).await.iter().any(|m| m["id"] == id));
    undo(port).await;
    assert!(
        measurements(port).await.iter().any(|m| m["id"] == id),
        "measurement delete was not undone"
    );
}

#[tokio::test]
async fn administrative_create_delete_are_undoable() {
    let (port, _) = test_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(api_url(port, "/api/administrative"))
        .json(&serde_json::json!({
            "level": "Country",
            "name": "ADM-create",
            "code": "ADM-1",
            "native_name": "Country One"
        }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "create admin failed: {}",
        resp.text().await.unwrap()
    );
    let created = resp.json::<serde_json::Value>().await.unwrap();
    let id = created["id"].as_str().unwrap().to_string();
    assert_eq!(created["level"], "Country");
    assert_eq!(created["native_name"], "Country One");

    let get_admin = |port: u16, id: String| async move {
        reqwest::Client::new()
            .get(api_url(port, &format!("/api/administrative/{}", id)))
            .send()
            .await
            .unwrap()
            .status()
    };
    assert_eq!(get_admin(port, id.clone()).await, 200);
    undo(port).await;
    assert_eq!(
        get_admin(port, id.clone()).await,
        404,
        "admin create was not undone"
    );

    let resp = client
        .post(api_url(port, "/api/administrative"))
        .json(&serde_json::json!({
            "level": "Province",
            "name": "ADM-delete",
            "code": "ADM-2"
        }))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = client
        .delete(api_url(port, &format!("/api/administrative/{}", id)))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "delete admin failed: {}",
        resp.text().await.unwrap()
    );
    assert_eq!(get_admin(port, id.clone()).await, 404);
    undo(port).await;
    assert_eq!(
        get_admin(port, id.clone()).await,
        200,
        "admin delete was not undone"
    );
}

#[tokio::test]
async fn breakdown_create_delete_and_links_are_undoable() {
    let (port, _) = test_server().await;
    let client = reqwest::Client::new();
    let wall = create_wall(port, "BD-Wall").await;

    let resp = client
        .post(api_url(port, "/api/breakdown"))
        .json(&serde_json::json!({
            "level": "Building",
            "name": "BD-create",
            "convention": "Building",
            "code": "BD-1"
        }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "create breakdown failed: {}",
        resp.text().await.unwrap()
    );
    let created = resp.json::<serde_json::Value>().await.unwrap();
    let id = created["id"].as_str().unwrap().to_string();
    assert_eq!(created["level"], "Building");
    assert_eq!(created["code"], "BD-1");

    let get_node = |port: u16, id: String| async move {
        let resp = reqwest::Client::new()
            .get(api_url(port, &format!("/api/breakdown/{}", id)))
            .send()
            .await
            .unwrap();
        if resp.status().is_success() {
            Some(resp.json::<serde_json::Value>().await.unwrap())
        } else {
            None
        }
    };
    assert!(get_node(port, id.clone()).await.is_some());
    undo(port).await;
    assert!(
        get_node(port, id.clone()).await.is_none(),
        "breakdown create was not undone"
    );

    let resp = client
        .post(api_url(port, "/api/breakdown"))
        .json(&serde_json::json!({
            "level": "Storey",
            "name": "BD-delete",
            "convention": "Building"
        }))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    let id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = client
        .delete(api_url(port, &format!("/api/breakdown/{}", id)))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "delete breakdown failed: {}",
        resp.text().await.unwrap()
    );
    assert!(get_node(port, id.clone()).await.is_none());
    undo(port).await;
    assert!(
        get_node(port, id.clone()).await.is_some(),
        "breakdown delete was not undone"
    );

    let resp = client
        .post(api_url(port, &format!("/api/breakdown/{}/link-obj", id)))
        .json(&serde_json::json!({ "object_id": wall }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "link object failed: {}",
        resp.text().await.unwrap()
    );
    let node = get_node(port, id.clone()).await.unwrap();
    assert!(node["related_object_ids"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o == &wall));
    undo(port).await;
    let node = get_node(port, id.clone()).await.unwrap();
    assert!(
        !node["related_object_ids"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o == &wall),
        "breakdown object link was not undone"
    );

    let resp = client
        .post(api_url(port, &format!("/api/breakdown/{}/link-admin", id)))
        .json(&serde_json::json!({ "division_id": "adm-node-1" }))
        .send()
        .await
        .unwrap();
    assert!(
        resp.status().is_success(),
        "link admin failed: {}",
        resp.text().await.unwrap()
    );
    let node = get_node(port, id.clone()).await.unwrap();
    assert!(node["administrative_division_ids"]
        .as_array()
        .unwrap()
        .iter()
        .any(|d| d == "adm-node-1"));
    undo(port).await;
    let node = get_node(port, id.clone()).await.unwrap();
    assert!(
        !node["administrative_division_ids"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d == "adm-node-1"),
        "breakdown admin link was not undone"
    );
}

/// Redo arms for the Slice 4 commands restore the post-mutation aggregate
/// state (snapshot based, not a no-op).
#[tokio::test]
async fn slice4_commands_redo_after_undo() {
    let (port, _) = test_server().await;
    let client = reqwest::Client::new();
    let wall = create_wall(port, "Redo-Wall").await;

    // Series: create + update.
    let id = create_series(port, "Ser-redo").await;
    undo(port).await;
    assert!(get_series(port, &id).await.is_none());
    redo(port).await;
    assert!(
        get_series(port, &id).await.is_some(),
        "series create redo failed"
    );

    let resp = client
        .put(api_url(port, &format!("/api/series/{}", id)))
        .json(&serde_json::json!({ "name": "Ser-redo-2" }))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    undo(port).await;
    assert_eq!(get_series(port, &id).await.unwrap()["name"], "Ser-redo");
    redo(port).await;
    assert_eq!(
        get_series(port, &id).await.unwrap()["name"],
        "Ser-redo-2",
        "series update redo failed"
    );

    // Lifecycle: create + link.
    let resp = client
        .post(api_url(port, "/api/lifecycle"))
        .json(&serde_json::json!({
            "phase": "Construction",
            "record_type": "activity",
            "name": "LC-redo"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let records: Vec<serde_json::Value> = client
        .get(api_url(port, "/api/lifecycle"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let record_id = records.iter().find(|r| r["name"] == "LC-redo").unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    undo(port).await;
    redo(port).await;
    let records: Vec<serde_json::Value> = client
        .get(api_url(port, "/api/lifecycle"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        records.iter().any(|r| r["id"] == record_id),
        "lifecycle create redo failed"
    );

    let resp = client
        .post(api_url(
            port,
            &format!("/api/lifecycle/{}/link/{}", record_id, wall),
        ))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    undo(port).await;
    redo(port).await;
    let records: Vec<serde_json::Value> = client
        .get(api_url(port, "/api/lifecycle"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let linked = records.iter().find(|r| r["id"] == record_id).unwrap();
    assert!(
        linked["related_object_ids"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o == &wall),
        "lifecycle link redo failed"
    );

    // Measurement: create redo.
    let resp = client
        .post(api_url(port, "/api/measurements"))
        .json(&serde_json::json!({
            "type": "distance",
            "points": [],
            "value": 2.0
        }))
        .send()
        .await
        .unwrap();
    let measurement_id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    undo(port).await;
    redo(port).await;
    let measurements: Vec<serde_json::Value> = client
        .get(api_url(port, "/api/measurements"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        measurements.iter().any(|m| m["id"] == measurement_id),
        "measurement create redo failed"
    );

    // Administrative division: create + delete redo.
    let resp = client
        .post(api_url(port, "/api/administrative"))
        .json(&serde_json::json!({ "level": "Country", "name": "ADM-redo", "code": "ADM-R" }))
        .send()
        .await
        .unwrap();
    let division_id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    undo(port).await;
    redo(port).await;
    assert_eq!(
        client
            .get(api_url(
                port,
                &format!("/api/administrative/{}", division_id)
            ))
            .send()
            .await
            .unwrap()
            .status(),
        200,
        "admin create redo failed"
    );
    let resp = client
        .delete(api_url(
            port,
            &format!("/api/administrative/{}", division_id),
        ))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    undo(port).await;
    redo(port).await;
    assert_eq!(
        client
            .get(api_url(
                port,
                &format!("/api/administrative/{}", division_id)
            ))
            .send()
            .await
            .unwrap()
            .status(),
        404,
        "admin delete redo failed"
    );

    // Breakdown: create + object link redo.
    let resp = client
        .post(api_url(port, "/api/breakdown"))
        .json(&serde_json::json!({
            "level": "Storey",
            "name": "BD-redo",
            "convention": "Building"
        }))
        .send()
        .await
        .unwrap();
    let node_id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_str()
        .unwrap()
        .to_string();
    undo(port).await;
    redo(port).await;
    let resp = client
        .get(api_url(port, &format!("/api/breakdown/{}", node_id)))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "breakdown create redo failed");

    let resp = client
        .post(api_url(
            port,
            &format!("/api/breakdown/{}/link-obj", node_id),
        ))
        .json(&serde_json::json!({ "object_id": wall }))
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success());
    undo(port).await;
    redo(port).await;
    let node: serde_json::Value = client
        .get(api_url(port, &format!("/api/breakdown/{}", node_id)))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(
        node["related_object_ids"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o == &wall),
        "breakdown link redo failed"
    );
}
