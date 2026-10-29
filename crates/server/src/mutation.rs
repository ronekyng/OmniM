//! Single write path for mutating endpoints (ADR-0006).
//!
//! Every handler that changes the `Project` builds an `OmniCommand` and runs it
//! through `Validator` + `Executor` via [`apply`], or records aggregate-level
//! commands via the `Executor`'s snapshot API (see Slice 5, `import_ifc`).
//! Handler migrations land slice by slice; the allowlist in
//! `scripts/check_write_paths.py` tracks which endpoints still write directly.
//!
//! **Lock order (deadlock discipline):** acquire `project` first, then
//! `executor`, always in that order. `apply` owns both locks; handlers must
//! never acquire `executor` before `project`, and must prefer `apply` over
//! hand-rolled locking.
//!
//! **Persistence model:** mutations persist when `save_path` is `Some`.
//! `save_path` is configured by `main.rs` — default is the project file,
//! `OMNIM_EPHEMERAL=1` disables persistence entirely.

use std::time::{SystemTime, UNIX_EPOCH};

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use omnim_core as c;
use omnim_model::Project;
use omnim_omcom::ExecutionResult;

use crate::AppState;

/// Typed error for mutating endpoints: status + reason.
#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl ApiError {
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

impl From<StatusCode> for ApiError {
    fn from(status: StatusCode) -> Self {
        Self {
            status,
            message: status.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, self.message).into_response()
    }
}

/// Build a command envelope for a handler-generated mutation.
pub fn envelope(action: c::Action, ops: Vec<c::Operation>) -> c::OmniCommand {
    c::OmniCommand {
        version: "1.0".into(),
        id: format!(
            "mut-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ),
        action: action as i32,
        actor: None,
        target: None,
        operations: ops,
        validation: None,
        timestamp: None,
        metadata: Default::default(),
    }
}

/// The single write path: validate → execute → record undo → persist.
///
/// On validation failure nothing changes and no save happens. On success the
/// result carries the created ids; persistence failure is logged, not fatal —
/// the in-memory change and undo entry already exist, and failing the request
/// would mislead a client into retrying a mutation that took effect.
pub async fn apply(
    state: &AppState,
    action: c::Action,
    ops: Vec<c::Operation>,
) -> Result<ExecutionResult, ApiError> {
    let mut p = state.project.write().await;
    let mut exe = state.executor.write().await;
    Ok(apply_locked(
        &state.save_path,
        &mut exe,
        action,
        ops,
        &mut p,
    ))
}

/// Same as [`apply`] for handlers that already hold both write locks
/// (e.g. handlers composing several commands under one lock guard).
pub fn apply_locked(
    save_path: &Option<String>,
    exe: &mut omnim_omcom::Executor,
    action: c::Action,
    ops: Vec<c::Operation>,
    project: &mut Project,
) -> ExecutionResult {
    let result = exe.execute(&envelope(action, ops), project);
    if result.success {
        persist(save_path, project);
    }
    result
}

/// Persist the project if a save path is configured. Failures are logged and
/// non-fatal (see [`apply`]).
pub fn persist(save_path: &Option<String>, project: &Project) {
    if let Some(path) = save_path {
        if let Err(e) = project.save(path) {
            eprintln!("Auto-save failed: {}", e);
        }
    }
}

/// External id of the first object created by an [`apply`] call, if any.
/// Several handlers only ever create one object and return it.
pub fn first_created(result: &ExecutionResult) -> Option<String> {
    result.created_ids.first().cloned()
}

/// Convert an `ExecutionResult` into a handler response or an `ApiError`.
pub fn require_success(result: ExecutionResult) -> Result<ExecutionResult, ApiError> {
    if result.success {
        Ok(result)
    } else {
        Err(ApiError::bad_request(result.message))
    }
}

// ── op builders for Handler migrations ───────────────────────────────────────

/// Wrap an arbitrary `operation::Op` into an `Operation`. Used by handlers with
/// fully-specified protocol structs (sketch / feature / catalog migrations).
pub fn op(op: c::operation::Op) -> c::Operation {
    c::Operation { op: Some(op) }
}

pub fn target(id: &str) -> Option<c::Target> {
    Some(c::Target {
        target_type: Some(c::target::TargetType::Object(c::ObjectTarget {
            object_id: id.to_string(),
        })),
    })
}

#[allow(clippy::too_many_arguments)]
pub fn op_modify_object(
    id: &str,
    name: Option<String>,
    description: Option<String>,
    ifc_class: Option<String>,
    tags: Option<Vec<String>>,
    elevation: Option<f64>,
    level_id: Option<String>,
    bottom_level_id: Option<String>,
    top_level_id: Option<String>,
) -> c::Operation {
    c::Operation {
        op: Some(c::operation::Op::ModifyObject(c::ModifyObjectOperation {
            object_id: id.to_string(),
            new_name: name,
            new_description: description,
            new_geometry: None,
            new_ifc_class: ifc_class,
            new_tags: tags.map(|tags| c::TagList { values: tags }),
            new_elevation: elevation,
            new_level_id: level_id,
            new_bottom_level_id: bottom_level_id,
            new_top_level_id: top_level_id,
        })),
    }
}

pub fn op_set_property(id: &str, name: &str, value: &str) -> c::Operation {
    c::Operation {
        op: Some(c::operation::Op::SetObjectProperty(
            c::SetObjectPropertyOperation {
                object_id: id.to_string(),
                property: Some(c::Property {
                    name: name.to_string(),
                    value: Some(c::PropertyValue {
                        value_type: Some(c::property_value::ValueType::StringValue(
                            value.to_string(),
                        )),
                    }),
                    unit: String::new(),
                    description: String::new(),
                }),
            },
        )),
    }
}

pub fn op_create_material(m: c::Material) -> c::Operation {
    c::Operation {
        op: Some(c::operation::Op::CreateMaterial(
            c::CreateMaterialOperation { material: Some(m) },
        )),
    }
}

pub fn op_relationship(op: c::operation::Op) -> c::Operation {
    c::Operation { op: Some(op) }
}
