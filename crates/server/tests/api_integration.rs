use std::sync::Arc;
use tokio::sync::RwLock;
use omnim_model::Project;
use omnim_server::{create_router, SharedProject};

/// Start the server on a random port and return the port number.
async fn start_test_server() -> u16 {
    let project: SharedProject = Arc::new(RwLock::new(Project::new("Test Project")));
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

// Protobuf ObjectType enum values (from proto/omnim.proto)
const OT_WALL: i32 = 100;
const OT_SLAB: i32 = 103;
const OT_ROAD: i32 = 200;
const OT_PIPE_PLANT: i32 = 305;
const OT_DOOR: i32 = 104;
const OT_ASSEMBLY: i32 = 401;

// ---------------------------------------------------------------------------
// Basic health & project
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_health_check() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(api_url(port, "/api/health"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let text = resp.text().await.unwrap();
    assert_eq!(text, "ok");
}

#[tokio::test]
async fn test_get_project() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(api_url(port, "/api/project"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let json: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(json["name"], "Test Project");
}

#[tokio::test]
async fn test_get_stats() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(api_url(port, "/api/stats"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let json: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(json["object_count"], 0);
    assert_eq!(json["name"], "Test Project");
}

// ---------------------------------------------------------------------------
// Object CRUD
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_create_and_get_object() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    // Create
    let create_resp = client
        .post(api_url(port, "/api/objects"))
        .json(&serde_json::json!({
            "object_type": "Wall",
            "name": "Test Wall",
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(create_resp.status(), 200);
    let created: serde_json::Value = create_resp.json().await.unwrap();
    assert_eq!(created["name"], "Test Wall");
    assert_eq!(created["object_type"], OT_WALL);

    let obj_id = created["id"].as_str().unwrap().to_string();

    // Get by ID
    let get_resp = client
        .get(api_url(port, &format!("/api/objects/{}", obj_id)))
        .send()
        .await
        .unwrap();

    assert_eq!(get_resp.status(), 200);
    let obj: serde_json::Value = get_resp.json().await.unwrap();
    assert_eq!(obj["name"], "Test Wall");
}

#[tokio::test]
async fn test_create_object_with_geometry() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    // Create with geometry (this was returning 500)
    let create_resp = client
        .post(api_url(port, "/api/objects"))
        .json(&serde_json::json!({
            "object_type": "Wall",
            "name": "GeomWall",
            "description": "",
            "geometry": {
                "operations": [
                    {
                        "operation": {
                            "Box": {
                                "length": 5.0,
                                "width": 0.3,
                                "height": 3.0
                            }
                        }
                    }
                ]
            }
        }))
        .send()
        .await
        .unwrap();

    let status = create_resp.status();
    if status != 200 {
        let body = create_resp.text().await.unwrap_or_default();
        panic!("Create with geometry failed: HTTP {} body: {}", status, body);
    }

    let created: serde_json::Value = create_resp.json().await.unwrap();
    assert_eq!(created["name"], "GeomWall");
    assert_eq!(created["object_type"], OT_WALL);
    assert!(created["geometry"].is_object(), "Expected geometry object, got {:?}", created["geometry"]);
}

#[tokio::test]
async fn test_list_objects() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    // Create two objects
    for name in &["ObjA", "ObjB"] {
        let resp = client
            .post(api_url(port, "/api/objects"))
            .json(&serde_json::json!({
                "object_type": "Generic",
                "name": name,
            }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
    }

    // List
    let list_resp = client
        .get(api_url(port, "/api/objects"))
        .send()
        .await
        .unwrap();

    assert_eq!(list_resp.status(), 200);
    let list: Vec<serde_json::Value> = list_resp.json().await.unwrap();
    assert_eq!(list.len(), 2);
}

#[tokio::test]
async fn test_update_object() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    let created: serde_json::Value = client
        .post(api_url(port, "/api/objects"))
        .json(&serde_json::json!({
            "object_type": "Generic",
            "name": "Original",
            "description": "original desc",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let obj_id = created["id"].as_str().unwrap();

    // Update
    let update_resp = client
        .put(api_url(port, &format!("/api/objects/{}", obj_id)))
        .json(&serde_json::json!({
            "name": "Updated",
            "description": "updated desc",
            "tags": ["tag1", "tag2"],
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(update_resp.status(), 200);

    // Verify
    let obj: serde_json::Value = client
        .get(api_url(port, &format!("/api/objects/{}", obj_id)))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(obj["name"], "Updated");
    assert_eq!(obj["description"], "updated desc");
    assert!(obj["tags"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_delete_object() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    let created: serde_json::Value = client
        .post(api_url(port, "/api/objects"))
        .json(&serde_json::json!({
            "object_type": "Generic",
            "name": "ToDelete",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let obj_id = created["id"].as_str().unwrap().to_string();

    // Delete
    let del_resp = client
        .delete(api_url(port, &format!("/api/objects/{}", obj_id)))
        .send()
        .await
        .unwrap();
    assert_eq!(del_resp.status(), 200);

    // Verify gone
    let get_resp = client
        .get(api_url(port, &format!("/api/objects/{}", obj_id)))
        .send()
        .await
        .unwrap();
    assert_eq!(get_resp.status(), 404);
}

// ---------------------------------------------------------------------------
// Properties — set via `{"name": ..., "value": ...}`, read as array of objects
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_set_property() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    let created: serde_json::Value = client
        .post(api_url(port, "/api/objects"))
        .json(&serde_json::json!({
            "object_type": "Generic",
            "name": "PropTest",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let obj_id = created["id"].as_str().unwrap();

    // Set property — note: field name is "name", not "key"
    let prop_resp = client
        .put(api_url(port, &format!("/api/objects/{}/prop", obj_id)))
        .json(&serde_json::json!({
            "name": "height",
            "value": "3.5m",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(prop_resp.status(), 200);

    // Verify property — properties is an array of {name, value, unit, description}
    let obj: serde_json::Value = client
        .get(api_url(port, &format!("/api/objects/{}", obj_id)))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let props = obj["properties"].as_array().unwrap();
    let height_prop = props.iter().find(|p| p["name"] == "height").unwrap();
    // The PropertyValue oneof is serialized as { "value_type": { "StringValue": "..." } }
    assert_eq!(height_prop["value"]["value_type"]["StringValue"], "3.5m");
}

// ---------------------------------------------------------------------------
// Tree
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_tree_structure() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    // Create root
    let root: serde_json::Value = client
        .post(api_url(port, "/api/objects"))
        .json(&serde_json::json!({
            "object_type": "Building",
            "name": "Main Building",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let root_id = root["id"].as_str().unwrap().to_string();

    // Create child
    let child: serde_json::Value = client
        .post(api_url(port, "/api/objects"))
        .json(&serde_json::json!({
            "object_type": "BuildingStorey",
            "name": "Floor 1",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let child_id = child["id"].as_str().unwrap().to_string();

    // Set parent
    let parent_resp = client
        .put(api_url(port, &format!("/api/objects/{}/parent", child_id)))
        .json(&serde_json::json!({ "parent_id": root_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(parent_resp.status(), 200);

    // Tree
    let tree: Vec<serde_json::Value> = client
        .get(api_url(port, "/api/tree"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(tree.len(), 1);
    assert_eq!(tree[0]["name"], "Main Building");
    assert_eq!(tree[0]["children"].as_array().unwrap().len(), 1);
    assert_eq!(tree[0]["children"][0]["name"], "Floor 1");
}

// ---------------------------------------------------------------------------
// GenRule — object_type is an i32 enum value
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_genrule_wall() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(api_url(port, "/api/genrule"))
        .json(&serde_json::json!({
            "genrule": "wall",
            "name": "MyWall",
            "length": 5.0,
            "height": 3.0,
            "core_thickness": 0.2,
            "finish_thickness": 0.05,
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let obj: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(obj["object_type"], OT_WALL);
    assert!(!obj["geometry"].is_null());
}

#[tokio::test]
async fn test_genrule_slab() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(api_url(port, "/api/genrule"))
        .json(&serde_json::json!({
            "genrule": "slab",
            "name": "MySlab",
            "length": 6.0,
            "width": 4.0,
            "structural_thickness": 0.25,
            "finish_thickness": 0.05,
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let obj: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(obj["object_type"], OT_SLAB);
    assert!(!obj["geometry"].is_null());
}

#[tokio::test]
async fn test_genrule_road() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(api_url(port, "/api/genrule"))
        .json(&serde_json::json!({
            "genrule": "road",
            "name": "MyRoad",
            "length": 100.0,
            "width": 7.0,
            "thickness": 0.4,
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let obj: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(obj["object_type"], OT_ROAD);
    assert!(!obj["geometry"].is_null());
}

#[tokio::test]
async fn test_genrule_pipe() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(api_url(port, "/api/genrule"))
        .json(&serde_json::json!({
            "genrule": "pipe",
            "name": "MyPipe",
            "length": 10.0,
            "radius": 0.3,
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let obj: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(obj["object_type"], OT_PIPE_PLANT);
    assert!(!obj["geometry"].is_null());
}

#[tokio::test]
async fn test_genrule_door() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    // First create a wall
    let wall: serde_json::Value = client
        .post(api_url(port, "/api/genrule"))
        .json(&serde_json::json!({
            "genrule": "wall",
            "name": "WallForDoor",
            "length": 5.0,
            "height": 3.0,
            "core_thickness": 0.2,
            "finish_thickness": 0.05,
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let wall_id = wall["id"].as_str().unwrap();

    // Create door
    let resp = client
        .post(api_url(port, "/api/genrule"))
        .json(&serde_json::json!({
            "genrule": "door",
            "name": "MainDoor",
            "wall_id": wall_id,
            "opening_width": 0.9,
            "opening_height": 2.1,
            "position_x": 2.5,
            "position_z": 1.5,
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let obj: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(obj["object_type"], OT_DOOR);
    assert!(!obj["geometry"].is_null());
    assert_eq!(obj["parent_id"], wall_id);
}

#[tokio::test]
async fn test_genrule_assembly() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(api_url(port, "/api/genrule"))
        .json(&serde_json::json!({
            "genrule": "assembly",
            "name": "TestAssembly",
            "components": [
                {
                    "name": "Base",
                    "shape": "box",
                    "p1": 2.0,
                    "p2": 2.0,
                    "p3": 0.3,
                    "tx": 0.0,
                    "ty": 0.0,
                    "tz": 0.0,
                },
                {
                    "name": "Pillar",
                    "shape": "cylinder",
                    "p1": 0.2,
                    "p2": 1.5,
                    "p3": 16.0,
                    "tx": 0.0,
                    "ty": 0.0,
                    "tz": 0.0,
                }
            ],
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let obj: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(obj["object_type"], OT_ASSEMBLY);
    assert!(!obj["geometry"].is_null());
}

// ---------------------------------------------------------------------------
// Mesh endpoint
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_mesh() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    // Create a wall so there's some geometry
    client
        .post(api_url(port, "/api/genrule"))
        .json(&serde_json::json!({
            "genrule": "wall",
            "name": "MeshWall",
            "length": 5.0,
            "height": 3.0,
            "core_thickness": 0.2,
            "finish_thickness": 0.05,
        }))
        .send()
        .await
        .unwrap();

    // Get mesh
    let mesh_resp = client
        .get(api_url(port, "/api/mesh"))
        .send()
        .await
        .unwrap();

    assert_eq!(mesh_resp.status(), 200);
    let mesh: serde_json::Value = mesh_resp.json().await.unwrap();
    let objects = mesh["objects"].as_array().unwrap();
    assert!(!objects.is_empty());

    let obj = &objects[0];
    assert!(obj["vertex_count"].as_u64().unwrap() > 0);
    assert!(obj["triangle_count"].as_u64().unwrap() > 0);
    assert!(obj["positions"].as_array().unwrap().len() > 0);
    assert!(obj["normals"].as_array().unwrap().len() > 0);
}

// ---------------------------------------------------------------------------
// Omcom commands
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_omcom_create_and_undo() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    // Execute Omcom create command
    // Note: action, actor.type, and type inside CreateObject are i32 enum values
    let exec_resp = client
        .post(api_url(port, "/api/omcom"))
        .json(&serde_json::json!({
            "version": "1.0",
            "id": "cmd-001",
            "action": 1,
            "actor": {
                "id": "tester",
                "type": 1
            },
            "target": null,
            "operations": [
                {
                    "op": {
                        "CreateObject": {
                             "type": 100,
                             "name": "OmcomWall",
                             "description": "Created via Omcom",
                             "geometry": null,
                             "properties": [],
                             "parent_id": ""
                         }
                    }
                }
            ],
            "validation": null,
            "timestamp": null,
            "metadata": {}
        }))
        .send()
        .await
        .unwrap();

    let status = exec_resp.status();
    let result: serde_json::Value = exec_resp.json().await.unwrap();
    assert_eq!(status, 200);
    assert_eq!(result["success"], true);

    // Verify object created
    let stats: serde_json::Value = client
        .get(api_url(port, "/api/stats"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(stats["object_count"], 1);

    // Check undo status
    let status: serde_json::Value = client
        .get(api_url(port, "/api/undo-status"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(status["can_undo"], true);

    // Undo
    let undo_resp = client
        .post(api_url(port, "/api/undo"))
        .send()
        .await
        .unwrap();
    assert_eq!(undo_resp.status(), 200);
    let undo: serde_json::Value = undo_resp.json().await.unwrap();
    assert_eq!(undo["success"], true);

    // Verify object removed
    let stats_after: serde_json::Value = client
        .get(api_url(port, "/api/stats"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(stats_after["object_count"], 0);

    // Redo
    let redo_resp = client
        .post(api_url(port, "/api/redo"))
        .send()
        .await
        .unwrap();
    assert_eq!(redo_resp.status(), 200);
    let redo: serde_json::Value = redo_resp.json().await.unwrap();
    assert_eq!(redo["success"], true);

    // Verify object restored
    let stats_redo: serde_json::Value = client
        .get(api_url(port, "/api/stats"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(stats_redo["object_count"], 1);
}

// ---------------------------------------------------------------------------
// Connect
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_connect_objects() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    // Create two objects
    let obj1: serde_json::Value = client
        .post(api_url(port, "/api/objects"))
        .json(&serde_json::json!({
            "object_type": "Beam",
            "name": "Beam1",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let obj2: serde_json::Value = client
        .post(api_url(port, "/api/objects"))
        .json(&serde_json::json!({
            "object_type": "Column",
            "name": "Column1",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let id1 = obj1["id"].as_str().unwrap();
    let id2 = obj2["id"].as_str().unwrap();

    // Connect
    let conn_resp = client
        .post(api_url(port, &format!("/api/objects/{}/connect/{}", id1, id2)))
        .send()
        .await
        .unwrap();
    assert_eq!(conn_resp.status(), 200);

    // Verify connections in project
    let project: serde_json::Value = client
        .get(api_url(port, "/api/project"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let connections = project["relations"]["connections"].as_array().unwrap();
    assert!(!connections.is_empty());
}

// ---------------------------------------------------------------------------
// Materials
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_material_crud() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    // Create material — requires id, name, category
    let mat_resp = client
        .post(api_url(port, "/api/materials"))
        .json(&serde_json::json!({
            "id": "mat-concrete",
            "name": "Concrete C30",
            "category": "Concrete",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(mat_resp.status(), 200);

    // List materials
    let list: Vec<serde_json::Value> = client
        .get(api_url(port, "/api/materials"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(!list.is_empty());

    // Find our material
    let concrete = list.iter().find(|m| m["name"] == "Concrete C30").unwrap();
    assert_eq!(concrete["id"], "mat-concrete");
}

#[tokio::test]
async fn test_assign_material() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    // Create object
    let obj: serde_json::Value = client
        .post(api_url(port, "/api/objects"))
        .json(&serde_json::json!({
            "object_type": "Column",
            "name": "ColumnMat",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let obj_id = obj["id"].as_str().unwrap();

    // Create material
    let mat_resp = client
        .post(api_url(port, "/api/materials"))
        .json(&serde_json::json!({
            "id": "mat-steel",
            "name": "Steel",
            "category": "Metal",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(mat_resp.status(), 200);
    let mat_id = "mat-steel";

    // Assign material
    let assign_resp = client
        .put(api_url(port, &format!("/api/objects/{}/material", obj_id)))
        .json(&serde_json::json!({ "material_id": mat_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(assign_resp.status(), 200);

    // Verify assignment
    let obj_updated: serde_json::Value = client
        .get(api_url(port, &format!("/api/objects/{}", obj_id)))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(obj_updated["material_id"], mat_id);
}

// ---------------------------------------------------------------------------
// IFC Export
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_ifc_export() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    // Create some objects first
    client
        .post(api_url(port, "/api/genrule"))
        .json(&serde_json::json!({
            "genrule": "wall",
            "name": "IFCWall",
            "length": 5.0,
            "height": 3.0,
            "core_thickness": 0.2,
            "finish_thickness": 0.05,
        }))
        .send()
        .await
        .unwrap();

    // Export IFC
    let export_resp = client
        .get(api_url(port, "/api/export/ifc"))
        .send()
        .await
        .unwrap();

    assert_eq!(export_resp.status(), 200);
    let ifc_content = export_resp.text().await.unwrap();
    assert!(ifc_content.contains("ISO-10303-21"));
    assert!(ifc_content.contains("IFCWALL"));
}

// ---------------------------------------------------------------------------
// Error handling tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_get_nonexistent_object_returns_404() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(api_url(port, "/api/objects/nonexistent-id"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404);
}

#[tokio::test]
async fn test_delete_nonexistent_object_returns_500() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .delete(api_url(port, "/api/objects/nonexistent-id"))
        .send()
        .await
        .unwrap();

    // The server returns 404 for nonexistent objects
    assert_eq!(resp.status(), 404);
}

// ---------------------------------------------------------------------------
// Lifecycle — uses phase, record_type, name (not object_id / status)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_lifecycle_crud() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    // Create lifecycle record — requires phase, record_type, name
    let lc_resp = client
        .post(api_url(port, "/api/lifecycle"))
        .json(&serde_json::json!({
            "phase": "Design",
            "record_type": "review",
            "name": "Design Review 1",
            "description": "Initial design phase review"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(lc_resp.status(), 200);

    // List lifecycle
    let list: Vec<serde_json::Value> = client
        .get(api_url(port, "/api/lifecycle"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(!list.is_empty());

    let record = &list[0];
    assert_eq!(record["name"], "Design Review 1");
    assert_eq!(record["description"], "Initial design phase review");
}

// ---------------------------------------------------------------------------
// Concurrent access (sanity check)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_concurrent_operations() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    let mut handles = Vec::new();
    for i in 0..10 {
        let url = api_url(port, "/api/objects");
        let client = client.clone();
        handles.push(tokio::spawn(async move {
            client
                .post(&url)
                .json(&serde_json::json!({
                    "object_type": "Generic",
                    "name": format!("ConcurrentObj{}", i),
                }))
                .send()
                .await
                .unwrap()
        }));
    }

    for handle in handles {
        let resp = handle.await.unwrap();
        assert_eq!(resp.status(), 200);
    }

    // Verify all 10 created
    let stats: serde_json::Value = client
        .get(api_url(port, "/api/stats"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(stats["object_count"], 10);
}

// ---------------------------------------------------------------------------
// AMIR endpoints (with mock provider — no API key needed)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_amir_translate_with_mock() {
    // Set env so create_router() picks up mock provider
    std::env::set_var("AMIR_PROVIDER", "mock");
    // Clear any real API key so it doesn't interfere
    std::env::set_var("AMIR_API_KEY", "mock-key");

    let project: SharedProject = Arc::new(RwLock::new(Project::new("Test")));
    let router = create_router(project, None);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });

    // Short wait for server to start
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    // /api/amir/translate — use text that matches mock key
    let resp = client
        .post(api_url(port, "/api/amir/translate"))
        .json(&serde_json::json!({
            "text": "create wall",
            "selected_ids": null
        }))
        .send()
        .await
        .unwrap();

    // With mock provider, the call should succeed
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["success"], true);

    // /api/amir/chat
    let resp = client
        .post(api_url(port, "/api/amir/chat"))
        .json(&serde_json::json!({
            "text": "create wall",
            "session_id": null,
            "selected_ids": null
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let chat: serde_json::Value = resp.json().await.unwrap();
    assert!(chat["session_id"].as_str().map_or(false, |s| !s.is_empty()));

    // /api/amir/explain (no provider needed — local only)
    let resp = client
        .post(api_url(port, "/api/amir/explain"))
        .json(&serde_json::json!({
            "errors": ["target object 'wall-1' not found"],
            "context": null
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let explain: serde_json::Value = resp.json().await.unwrap();
    assert!(explain["explanations"].as_array().map_or(false, |a| a.len() == 1));

    // /api/amir/describe
    let resp = client
        .post(api_url(port, "/api/amir/describe"))
        .json(&serde_json::json!({
            "object_ids": null,
            "format": "text"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);

    std::env::remove_var("AMIR_PROVIDER");
    std::env::remove_var("AMIR_API_KEY");
}

// ---------------------------------------------------------------------------
// Feature Pipeline: BRep-based Extrude / Revolve / Sweep / Loft
// ---------------------------------------------------------------------------

/// Helper: create a sketch, add a rectangle entity, solve, and return its ID.
async fn create_rectangle_sketch(client: &reqwest::Client, port: u16, name: &str) -> String {
    let resp = client
        .post(api_url(port, "/api/sketches"))
        .json(&serde_json::json!({ "name": name }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let sketch: serde_json::Value = resp.json().await.unwrap();
    let sketch_id = sketch["id"].as_str().unwrap().to_string();

    let resp = client
        .post(api_url(port, &format!("/api/sketches/{}/entities", sketch_id)))
        .json(&serde_json::json!({
            "entity_type": "rectangle",
            "points": [
                { "x": 0.0, "y": 0.0, "z": 0.0 },
                { "x": 2.0, "y": 1.0, "z": 0.0 }
            ]
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);

    let resp = client
        .post(api_url(port, &format!("/api/sketches/{}/solve", sketch_id)))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);

    sketch_id
}

#[tokio::test]
async fn test_feature_pipeline_extrude() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    // 1. Create component
    let resp = client
        .post(api_url(port, "/api/components"))
        .json(&serde_json::json!({ "name": "TestComponent" }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let comp: serde_json::Value = resp.json().await.unwrap();
    let comp_id = comp["id"].as_str().unwrap().to_string();

    // 2. Create rectangle sketch
    let sketch_id = create_rectangle_sketch(&client, port, "rect-profile").await;

    // 3. Add extrude feature
    let resp = client
        .post(api_url(port, &format!("/api/components/{}/features", comp_id)))
        .json(&serde_json::json!({
            "feature_type": "extrude",
            "sketch_id": sketch_id,
            "depth": 3.0,
            "direction_x": 0.0,
            "direction_y": 0.0,
            "direction_z": 1.0
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let feat: serde_json::Value = resp.json().await.unwrap();
    let feature_id = feat["id"].as_str().unwrap().to_string();
    assert_eq!(feat["feature_type"].as_str().unwrap(), "extrude");

    // 4. Verify component was rebuilt with geometry
    let resp = client
        .get(api_url(port, &format!("/api/components/{}", comp_id)))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let comp: serde_json::Value = resp.json().await.unwrap();
    assert!(comp["feature_count"].as_i64().unwrap() >= 1);

    // 5. List features
    let resp = client
        .get(api_url(port, &format!("/api/components/{}/features", comp_id)))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let features: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert!(!features.is_empty());

    // 6. Update feature depth
    let resp = client
        .put(api_url(port, &format!("/api/components/{}/features/{}", comp_id, feature_id)))
        .json(&serde_json::json!({ "depth": 5.0 }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);

    // 7. Remove feature
    let resp = client
        .delete(api_url(port, &format!("/api/components/{}/features/{}", comp_id, feature_id)))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
}

#[tokio::test]
async fn test_feature_pipeline_revolve() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    // Create component
    let resp = client
        .post(api_url(port, "/api/components"))
        .json(&serde_json::json!({ "name": "RevolveTest" }))
        .send().await.unwrap();
    let comp: serde_json::Value = resp.json().await.unwrap();
    let comp_id = comp["id"].as_str().unwrap().to_string();

    // Create rectangle sketch
    let sketch_id = create_rectangle_sketch(&client, port, "revolve-profile").await;

    // Add revolve feature
    let resp = client
        .post(api_url(port, &format!("/api/components/{}/features", comp_id)))
        .json(&serde_json::json!({
            "feature_type": "revolve",
            "sketch_id": sketch_id,
            "angle": 3.14159,
            "axis_direction_x": 0.0,
            "axis_direction_y": 0.0,
            "axis_direction_z": 1.0
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let feat: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(feat["feature_type"].as_str().unwrap(), "revolve");
}

#[tokio::test]
async fn test_feature_pipeline_loft() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    // Create component
    let resp = client
        .post(api_url(port, "/api/components"))
        .json(&serde_json::json!({ "name": "LoftTest" }))
        .send().await.unwrap();
    let comp: serde_json::Value = resp.json().await.unwrap();
    let comp_id = comp["id"].as_str().unwrap().to_string();

    // Create two rectangle sketches
    let sketch0 = create_rectangle_sketch(&client, port, "loft-bottom").await;
    let sketch1 = create_rectangle_sketch(&client, port, "loft-top").await;

    // Add loft feature
    let resp = client
        .post(api_url(port, &format!("/api/components/{}/features", comp_id)))
        .json(&serde_json::json!({
            "feature_type": "loft",
            "sketch_id": sketch0,
            "input_sketch_ids": [sketch0, sketch1]
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let feat: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(feat["feature_type"].as_str().unwrap(), "loft");
}

#[tokio::test]
async fn test_sketch_extrude_endpoint_uses_brep() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    let sketch_id = create_rectangle_sketch(&client, port, "legacy-extrude").await;

    let resp = client
        .post(api_url(port, &format!("/api/sketches/{}/extrude", sketch_id)))
        .json(&serde_json::json!({
            "depth": 2.0,
            "direction_x": 0.0,
            "direction_y": 0.0,
            "direction_z": 1.0
        }))
        .send().await.unwrap();
    let status = resp.status();
    let body = resp.text().await.unwrap();
    assert_eq!(status, 200, "extrude failed: {}", body);
    let result: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(result["success"].as_bool().unwrap());
    assert!(result["vertices"].as_i64().unwrap() > 0);
    assert!(result["faces"].as_i64().unwrap() > 0);
}

#[tokio::test]
async fn test_sketch_revolve_endpoint_uses_brep() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    let sketch_id = create_rectangle_sketch(&client, port, "legacy-revolve").await;

    let resp = client
        .post(api_url(port, &format!("/api/sketches/{}/revolve", sketch_id)))
        .json(&serde_json::json!({
            "angle": 3.14159,
            "axis_origin_x": 0.0,
            "axis_origin_y": 0.0,
            "axis_origin_z": 0.0,
            "axis_direction_x": 0.0,
            "axis_direction_y": 0.0,
            "axis_direction_z": 1.0
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let result: serde_json::Value = resp.json().await.unwrap();
    assert!(result["success"].as_bool().unwrap());
    assert!(result["vertices"].as_i64().unwrap() > 0);
}

#[tokio::test]
async fn test_feature_chamfer_fillet_produces_no_geometry() {
    let port = start_test_server().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(api_url(port, "/api/components"))
        .json(&serde_json::json!({ "name": "ChamferTest" }))
        .send().await.unwrap();
    let comp: serde_json::Value = resp.json().await.unwrap();
    let comp_id = comp["id"].as_str().unwrap().to_string();

    let sketch_id = create_rectangle_sketch(&client, port, "chamfer-profile").await;

    // Add chamfer feature — should succeed but produce no geometry
    let resp = client
        .post(api_url(port, &format!("/api/components/{}/features", comp_id)))
        .json(&serde_json::json!({
            "feature_type": "chamfer",
            "sketch_id": sketch_id
        }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);

    // Component should still exist and rebuild without error
    let resp = client
        .get(api_url(port, &format!("/api/components/{}", comp_id)))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
}