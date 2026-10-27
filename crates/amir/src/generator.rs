use crate::types::*;
use std::collections::HashMap;
use omnim_core as c;

/// Omcom Generator — 将 ParsedIntent 转换为 OmniCommand 序列
pub struct OmcomGenerator;

impl OmcomGenerator {
    pub fn new() -> Self {
        Self
    }

    /// 将一个 ParsedIntent 转为一条或多条 OmniCommand
    pub fn generate(&self, intent: &ParsedIntent) -> Result<Vec<c::OmniCommand>, AmirError> {
        match &intent.intent {
            Intent::CreateObject {
                object_type,
                name,
                parameters,
            } => self.create_object(object_type, name.as_deref(), parameters),
            Intent::ModifyObject {
                target_ref,
                modifications,
            } => self.modify_object(target_ref, modifications),
            Intent::DeleteObject { target_ref } => self.delete_object(target_ref),
            Intent::MoveObject {
                target_ref,
                new_parent_ref,
            } => self.move_object(target_ref, new_parent_ref),
            Intent::SetProperty {
                target_ref,
                property_name,
                property_value,
            } => self.set_property(target_ref, property_name, property_value),
            Intent::AddContainment {
                parent_ref,
                child_ref,
            } => self.add_containment(parent_ref, child_ref),
            Intent::RemoveContainment {
                parent_ref,
                child_ref,
            } => self.remove_containment(parent_ref, child_ref),
            Intent::AddConnection {
                from_ref,
                to_ref,
                connection_type,
            } => self.add_connection(from_ref, to_ref, connection_type.as_deref()),
            Intent::CreateLifecycleRecord {
                phase,
                record_type,
                name,
                description,
            } => self.create_lifecycle_record(phase, record_type, name.as_deref(), description.as_deref()),
            Intent::ModifyLifecycleRecord {
                record_ref,
                modifications,
            } => self.modify_lifecycle_record(record_ref, modifications),
            Intent::DeleteLifecycleRecord { record_ref } => self.delete_lifecycle_record(record_ref),
            Intent::LinkRecordToObject {
                record_ref,
                object_ref,
            } => self.link_record(record_ref, object_ref),
            Intent::UpdateSchedule { record_ref } => self.update_schedule(record_ref),
            Intent::UpdateCost { record_ref } => self.update_cost(record_ref),
            Intent::UpdateProgress { record_ref } => self.update_progress(record_ref),
            Intent::CreateWorkOrder { asset_ref, title } => {
                self.create_work_order(asset_ref, title.as_deref())
            }
            Intent::DefineComponent { name, category, description } => {
                self.define_component(name, category, description.as_deref())
            }
            Intent::PublishComponent { component_ref } => {
                self.publish_component(component_ref)
            }
            Intent::CreateSeries { name, description, domain } => {
                self.create_series(name, description.as_deref(), domain.as_deref())
            }
            Intent::SelectFromSeries { series_ref, answers } => {
                self.select_from_series(series_ref, answers)
            }
            Intent::ImportFile { format } => self.import_file(format),
            Intent::ExportFile { format } => self.export_file(format),
            Intent::QueryModel { .. } => Err(AmirError::UnsupportedIntent(
                "QueryModel is handled by the Reverse Parser, not Generator".into(),
            )),
            // Undo/Redo are not Omcom action types; they are handled by the
            // Executor's dedicated undo/redo methods (POST /api/undo, /api/redo).
            // The caller checks for these action strings to route accordingly.
            Intent::Undo => {
                let cmd = make_undo_redo_command("amir-undo");
                Ok(vec![cmd])
            }
            Intent::Redo => {
                let cmd = make_undo_redo_command("amir-redo");
                Ok(vec![cmd])
            }
            Intent::CreateSketch { parent_ref, plane } => {
                self.create_sketch(parent_ref, plane)
            }
            Intent::ExtrudeSketch { sketch_ref, depth, direction } => {
                self.extrude_sketch(sketch_ref, *depth, direction.as_deref())
            }
            Intent::RevolveSketch { sketch_ref, angle } => {
                self.revolve_sketch(sketch_ref, *angle)
            }
            Intent::AddFeature { parent_ref, feature_type, parameters } => {
                self.add_feature(parent_ref, feature_type, parameters)
            }
            Intent::CreateComponent { name } => {
                self.create_component(name)
            }
            Intent::BooleanOp { target_ref, tool_ref, operation } => {
                self.boolean_op(target_ref, tool_ref, operation)
            }
            Intent::Unknown { original_text } => Err(AmirError::UnsupportedIntent(format!(
                "Cannot generate command for unknown intent: {}",
                original_text
            ))),
        }
    }

    // ---- Private methods ----

    fn create_object(
        &self,
        object_type: &str,
        name: Option<&str>,
        parameters: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<Vec<c::OmniCommand>, AmirError> {
        let obj_type = parse_object_type(object_type);
        let obj_name = name.unwrap_or(&format!("{}-1", object_type)).to_string();

        let length = parameters.get("length").and_then(|v| v.as_f64()).unwrap_or(5.0);
        let height = parameters.get("height").and_then(|v| v.as_f64()).unwrap_or(3.0);
        let width = parameters.get("width").and_then(|v| v.as_f64()).unwrap_or(0.3);
        let thickness = parameters.get("thickness").and_then(|v| v.as_f64()).unwrap_or(0.2);
        let depth = parameters.get("depth").and_then(|v| v.as_f64()).unwrap_or(width);
        let radius = parameters.get("radius").and_then(|v| v.as_f64()).unwrap_or(0.15);

        let geom = self.create_geometry(object_type, length, width, height, thickness, depth, radius);

        let op = c::Operation {
            op: Some(c::operation::Op::CreateObject(c::CreateObjectOperation {
                r#type: obj_type as i32,
                name: obj_name,
                description: String::new(),
                geometry: geom,
                properties: vec![],
                parent_id: String::new(),
            })),
        };

        Ok(vec![make_command(c::Action::CreateObject, "", None, vec![op])])
    }

    fn create_geometry(
        &self,
        object_type: &str,
        length: f64,
        width: f64,
        height: f64,
        thickness: f64,
        depth: f64,
        radius: f64,
    ) -> Option<c::ParametricGeometry> {
        match object_type.to_lowercase().as_str() {
            "wall" => Some(c::ParametricGeometry {
                operations: vec![c::ParametricOperation {
                    operation: Some(c::parametric_operation::Operation::Box(c::Box {
                        length,
                        width: thickness,
                        height,
                    })),
                }],
            }),
            "beam" => Some(c::ParametricGeometry {
                operations: vec![c::ParametricOperation {
                    operation: Some(c::parametric_operation::Operation::Box(c::Box {
                        length,
                        width,
                        height,
                    })),
                }],
            }),
            "column" => Some(c::ParametricGeometry {
                operations: vec![c::ParametricOperation {
                    operation: Some(c::parametric_operation::Operation::Box(c::Box {
                        length: width,
                        width: depth,
                        height,
                    })),
                }],
            }),
            "slab" => Some(c::ParametricGeometry {
                operations: vec![c::ParametricOperation {
                    operation: Some(c::parametric_operation::Operation::Box(c::Box {
                        length,
                        width,
                        height: thickness,
                    })),
                }],
            }),
            "pipe" => Some(c::ParametricGeometry {
                operations: vec![c::ParametricOperation {
                    operation: Some(c::parametric_operation::Operation::Cylinder(c::Cylinder {
                        radius,
                        height: length,
                        segments: 16,
                    })),
                }],
            }),
            "road" => Some(c::ParametricGeometry {
                operations: vec![c::ParametricOperation {
                    operation: Some(c::parametric_operation::Operation::Box(c::Box {
                        length,
                        width,
                        height: thickness,
                    })),
                }],
            }),
            "tank" => Some(c::ParametricGeometry {
                operations: vec![c::ParametricOperation {
                    operation: Some(c::parametric_operation::Operation::Cylinder(c::Cylinder {
                        radius,
                        height,
                        segments: 16,
                    })),
                }],
            }),
            _ => {
                // Generic — create a box
                Some(c::ParametricGeometry {
                    operations: vec![c::ParametricOperation {
                        operation: Some(c::parametric_operation::Operation::Box(c::Box {
                            length,
                            width,
                            height,
                        })),
                    }],
                })
            }
        }
    }

    fn modify_object(
        &self,
        target_ref: &str,
        modifications: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<Vec<c::OmniCommand>, AmirError> {
        let target_id = resolve_ref(target_ref);
        let new_name = modifications.get("name").and_then(|v| v.as_str()).map(|s| s.to_string());
        let new_desc = modifications.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());

        let mut cmd = make_command(c::Action::ModifyObject, "", None, vec![]);

        // Set target
        cmd.target = Some(c::Target {
            target_type: Some(c::target::TargetType::Object(c::ObjectTarget {
                object_id: target_id.clone(),
            })),
        });

        // Create modify operation
        let op = c::Operation {
            op: Some(c::operation::Op::ModifyObject(c::ModifyObjectOperation {
                object_id: target_id,
                new_name,
                new_description: new_desc,
                new_geometry: None,
            })),
        };
        cmd.operations.push(op);

        Ok(vec![cmd])
    }

    fn delete_object(&self, target_ref: &str) -> Result<Vec<c::OmniCommand>, AmirError> {
        let target_id = resolve_ref(target_ref);
        let cmd = make_command(c::Action::DeleteObject, "", None, vec![]);
        // action-level delete uses target, no operations needed
        Ok(vec![c::OmniCommand {
            target: Some(c::Target {
                target_type: Some(c::target::TargetType::Object(c::ObjectTarget {
                    object_id: target_id,
                })),
            }),
            ..cmd
        }])
    }

    fn move_object(
        &self,
        target_ref: &str,
        new_parent_ref: &str,
    ) -> Result<Vec<c::OmniCommand>, AmirError> {
        let target_id = resolve_ref(target_ref);
        let parent_id = resolve_ref(new_parent_ref);

        let op = c::Operation {
            op: Some(c::operation::Op::MoveObject(c::MoveObjectOperation {
                object_id: target_id,
                new_parent_id: parent_id,
            })),
        };

        Ok(vec![make_command(c::Action::MoveObject, "", None, vec![op])])
    }

    fn set_property(
        &self,
        target_ref: &str,
        property_name: &str,
        property_value: &serde_json::Value,
    ) -> Result<Vec<c::OmniCommand>, AmirError> {
        let target_id = resolve_ref(target_ref);
        let value = json_to_property_value(property_value);

        let op = c::Operation {
            op: Some(c::operation::Op::SetProperty(c::Property {
                name: property_name.to_string(),
                value: Some(value),
                unit: String::new(),
                description: String::new(),
            })),
        };

        let mut cmd = make_command(c::Action::SetProperty, "", None, vec![op]);
        cmd.target = Some(c::Target {
            target_type: Some(c::target::TargetType::Object(c::ObjectTarget {
                object_id: target_id,
            })),
        });

        Ok(vec![cmd])
    }

    fn add_containment(
        &self,
        parent_ref: &str,
        child_ref: &str,
    ) -> Result<Vec<c::OmniCommand>, AmirError> {
        let parent_id = resolve_ref(parent_ref);
        let child_id = resolve_ref(child_ref);

        let op = c::Operation {
            op: Some(c::operation::Op::AddContainment(c::Containment {
                parent_id,
                child_id,
                r#type: c::ContainmentType::Spatial as i32,
            })),
        };

        Ok(vec![make_command(c::Action::AddContainment, "", None, vec![op])])
    }

    fn remove_containment(
        &self,
        parent_ref: &str,
        child_ref: &str,
    ) -> Result<Vec<c::OmniCommand>, AmirError> {
        let parent_id = resolve_ref(parent_ref);
        let child_id = resolve_ref(child_ref);

        let op = c::Operation {
            op: Some(c::operation::Op::RemoveContainment(c::RemoveContainmentOperation {
                parent_id,
                child_id,
            })),
        };

        Ok(vec![make_command(c::Action::RemoveContainment, "", None, vec![op])])
    }

    fn add_connection(
        &self,
        from_ref: &str,
        to_ref: &str,
        connection_type: Option<&str>,
    ) -> Result<Vec<c::OmniCommand>, AmirError> {
        let from_id = resolve_ref(from_ref);
        let to_id = resolve_ref(to_ref);
        let conn_type = match connection_type.unwrap_or("structural").to_lowercase().as_str() {
            "structural" => c::ConnectionType::Structural,
            "mechanical" => c::ConnectionType::Mechanical,
            "electrical" => c::ConnectionType::Electrical,
            "hydraulic" => c::ConnectionType::Hydraulic,
            _ => c::ConnectionType::Structural,
        };

        let op = c::Operation {
            op: Some(c::operation::Op::AddConnection(c::Connection {
                from_object_id: from_id,
                to_object_id: to_id,
                r#type: conn_type as i32,
                from_point: None,
                to_point: None,
                connection_detail: String::new(),
            })),
        };

        Ok(vec![make_command(c::Action::AddConnection, "", None, vec![op])])
    }

    fn create_lifecycle_record(
        &self,
        phase: &str,
        record_type: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<Vec<c::OmniCommand>, AmirError> {
        let phase_enum = parse_phase(phase);
        let record_name = name.unwrap_or(record_type).to_string();

        let op = c::Operation {
            op: Some(c::operation::Op::CreateLifecycleRecord(
                c::CreateLifecycleRecordOperation {
                    phase: phase_enum as i32,
                    name: record_name,
                    description: description.unwrap_or("").to_string(),
                    record_type: record_type.to_string(),
                    related_object_ids: vec![],
                    properties: vec![],
                },
            )),
        };

        Ok(vec![make_command(c::Action::CreateLifecycleRecord, "", None, vec![op])])
    }

    fn modify_lifecycle_record(
        &self,
        record_ref: &str,
        modifications: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<Vec<c::OmniCommand>, AmirError> {
        let record_id = resolve_ref(record_ref);
        let new_name = modifications.get("name").and_then(|v| v.as_str().map(|s| s.to_string()));
        let new_description = modifications.get("description").and_then(|v| v.as_str().map(|s| s.to_string()));
        let new_status = modifications.get("status").and_then(|v| v.as_str().map(|s| s.to_string()));
        let new_status_enum = new_status.as_ref().map(|s| {
            match s.to_lowercase().as_str() {
                "draft" => c::RecordStatus::Draft as i32,
                "open" => c::RecordStatus::Open as i32,
                "in_progress" | "inprogress" => c::RecordStatus::InProgress as i32,
                "blocked" => c::RecordStatus::Blocked as i32,
                "completed" => c::RecordStatus::Completed as i32,
                "approved" => c::RecordStatus::Approved as i32,
                "rejected" => c::RecordStatus::Rejected as i32,
                "closed" => c::RecordStatus::Closed as i32,
                _ => c::RecordStatus::Open as i32,
            }
        });
        let op = c::Operation {
            op: Some(c::operation::Op::ModifyLifecycleRecord(
                c::ModifyLifecycleRecordOperation {
                    record_id,
                    name: new_name,
                    description: new_description,
                    status: new_status_enum,
                    properties: Vec::new(),
                },
            )),
        };
        Ok(vec![make_command(c::Action::ModifyLifecycleRecord, "record", None, vec![op])])
    }

    fn delete_lifecycle_record(&self, record_ref: &str) -> Result<Vec<c::OmniCommand>, AmirError> {
        let record_id = resolve_ref(record_ref);
        let op = c::Operation {
            op: Some(c::operation::Op::DeleteLifecycleRecord(
                c::DeleteLifecycleRecordOperation {
                    record_id,
                },
            )),
        };
        Ok(vec![make_command(c::Action::DeleteLifecycleRecord, "", None, vec![op])])
    }

    fn link_record(
        &self,
        record_ref: &str,
        object_ref: &str,
    ) -> Result<Vec<c::OmniCommand>, AmirError> {
        let record_id = resolve_ref(record_ref);
        let object_id = resolve_ref(object_ref);

        let op = c::Operation {
            op: Some(c::operation::Op::LinkRecordToObject(
                c::LinkRecordToObjectOperation {
                    record_id,
                    object_id,
                    link_role: String::new(),
                },
            )),
        };

        Ok(vec![make_command(c::Action::LinkRecordToObject, "", None, vec![op])])
    }

    fn update_schedule(&self, record_ref: &str) -> Result<Vec<c::OmniCommand>, AmirError> {
        let record_id = resolve_ref(record_ref);
        let op = c::Operation {
            op: Some(c::operation::Op::UpdateSchedule(
                c::UpdateScheduleOperation {
                    record_id,
                    planned_start: None,
                    planned_finish: None,
                    actual_start: None,
                    actual_finish: None,
                },
            )),
        };
        Ok(vec![make_command(c::Action::UpdateSchedule, "", None, vec![op])])
    }

    fn update_cost(&self, record_ref: &str) -> Result<Vec<c::OmniCommand>, AmirError> {
        let record_id = resolve_ref(record_ref);
        let op = c::Operation {
            op: Some(c::operation::Op::UpdateCost(c::UpdateCostOperation {
                record_id,
                planned_cost: 0.0,
                actual_cost: 0.0,
                currency: String::new(),
            })),
        };
        Ok(vec![make_command(c::Action::UpdateCost, "", None, vec![op])])
    }

    fn update_progress(&self, record_ref: &str) -> Result<Vec<c::OmniCommand>, AmirError> {
        let record_id = resolve_ref(record_ref);
        let op = c::Operation {
            op: Some(c::operation::Op::UpdateProgress(c::UpdateProgressOperation {
                record_id,
                percent_complete: 0.0,
                note: String::new(),
            })),
        };
        Ok(vec![make_command(c::Action::UpdateProgress, "", None, vec![op])])
    }

    fn create_work_order(
        &self,
        asset_ref: &str,
        title: Option<&str>,
    ) -> Result<Vec<c::OmniCommand>, AmirError> {
        let asset_id = resolve_ref(asset_ref);
        let op = c::Operation {
            op: Some(c::operation::Op::CreateWorkOrder(
                c::CreateWorkOrderOperation {
                    asset_object_id: asset_id,
                    title: title.unwrap_or("Work Order").to_string(),
                    description: String::new(),
                },
            )),
        };
        Ok(vec![make_command(c::Action::CreateWorkOrder, "", None, vec![op])])
    }

    fn define_component(
        &self,
        name: &str,
        category: &str,
        description: Option<&str>,
    ) -> Result<Vec<c::OmniCommand>, AmirError> {
        let op = c::Operation {
            op: Some(c::operation::Op::DefineComponent(c::DefineComponentOperation {
                name: name.to_string(),
                category: category.to_string(),
                description: description.unwrap_or("").to_string(),
                ifc_class: String::new(),
                standard_reference: String::new(),
                tags: Vec::new(),
                domain: String::new(),
            })),
        };
        Ok(vec![make_command(c::Action::DefineComponent, "catalog", None, vec![op])])
    }

    fn publish_component(&self, component_ref: &str) -> Result<Vec<c::OmniCommand>, AmirError> {
        let comp_id = resolve_ref(component_ref);
        let op = c::Operation {
            op: Some(c::operation::Op::PublishComponent(c::PublishComponentOperation {
                component_id: comp_id,
            })),
        };
        Ok(vec![make_command(c::Action::PublishComponent, "catalog", None, vec![op])])
    }

    fn create_series(
        &self,
        name: &str,
        description: Option<&str>,
        domain: Option<&str>,
    ) -> Result<Vec<c::OmniCommand>, AmirError> {
        let op = c::Operation {
            op: Some(c::operation::Op::CreateSeries(c::CreateSeriesOperation {
                name: name.to_string(),
                description: description.unwrap_or("").to_string(),
                domain: domain.unwrap_or("").to_string(),
                standard_reference: String::new(),
            })),
        };
        Ok(vec![make_command(c::Action::CreateSeries, "series", None, vec![op])])
    }

    fn select_from_series(
        &self,
        _series_ref: &str,
        _answers: &[String],
    ) -> Result<Vec<c::OmniCommand>, AmirError> {
        // Selection is a query operation - the server handles it via /api/series/{id}/select
        // The generator just produces a validate command as a placeholder
        Ok(vec![make_command(c::Action::Validate, "select", None, vec![])])
    }

    fn import_file(&self, _format: &str) -> Result<Vec<c::OmniCommand>, AmirError> {
        // ImportFile is an Action-level command; no supporting Operation struct in proto.
        // The server handles this via the /api/import/ifc endpoint directly.
        Ok(vec![make_command(c::Action::ImportFile, "", None, vec![])])
    }

    fn export_file(&self, _format: &str) -> Result<Vec<c::OmniCommand>, AmirError> {
        // ExportFile is an Action-level command; no supporting Operation struct in proto.
        // The server handles this via the /api/export/ifc endpoint directly.
        Ok(vec![make_command(c::Action::ExportFile, "", None, vec![])])
    }

    fn create_sketch(&self, parent_ref: &str, _plane: &str) -> Result<Vec<c::OmniCommand>, AmirError> {
        let op = c::Operation {
            op: Some(c::operation::Op::CreateSketch(c::CreateSketchOperation {
                name: format!("sketch-on-{}", parent_ref),
                origin: None,
                normal: None,
            })),
        };
        Ok(vec![make_command(c::Action::CreateObject, "sketch", None, vec![op])])
    }

    fn extrude_sketch(&self, sketch_ref: &str, depth: f64, _direction: Option<&str>) -> Result<Vec<c::OmniCommand>, AmirError> {
        let op = c::Operation {
            op: Some(c::operation::Op::AddFeature(c::AddFeatureOperation {
                component_id: String::new(),
                feature_type: c::FeatureType::FeatureExtrude as i32,
                sketch_id: sketch_ref.to_string(),
                depth,
                direction: Some(c::Vector3D { x: 0.0, y: 0.0, z: 1.0 }),
                angle: 0.0,
                axis_origin: None,
                axis_direction: None,
                is_cut: false,
                input_sketch_ids: vec![sketch_ref.to_string()],
                axis_sketch_id: String::new(),
            })),
        };
        let name = format!("Extrude-{}", sketch_ref);
        Ok(vec![make_command(c::Action::ModifyObject, &name, None, vec![op])])
    }

    fn revolve_sketch(&self, sketch_ref: &str, angle: f64) -> Result<Vec<c::OmniCommand>, AmirError> {
        let op = c::Operation {
            op: Some(c::operation::Op::AddFeature(c::AddFeatureOperation {
                component_id: String::new(),
                feature_type: c::FeatureType::FeatureRevolve as i32,
                sketch_id: sketch_ref.to_string(),
                depth: 0.0,
                direction: None,
                angle,
                axis_origin: None,
                axis_direction: None,
                is_cut: false,
                input_sketch_ids: vec![sketch_ref.to_string()],
                axis_sketch_id: String::new(),
            })),
        };
        let name = format!("Revolve-{}", sketch_ref);
        Ok(vec![make_command(c::Action::ModifyObject, &name, None, vec![op])])
    }

    fn add_feature(&self, parent_ref: &str, feature_type: &str, _parameters: &HashMap<String, serde_json::Value>) -> Result<Vec<c::OmniCommand>, AmirError> {
        let ft = match feature_type.to_lowercase().as_str() {
            "extrude" => c::FeatureType::FeatureExtrude,
            "revolve" => c::FeatureType::FeatureRevolve,
            "sweep" => c::FeatureType::FeatureSweep,
            "loft" => c::FeatureType::FeatureLoft,
            "boolean" | "cut" => c::FeatureType::FeatureBoolean,
            "chamfer" => c::FeatureType::FeatureChamfer,
            "fillet" => c::FeatureType::FeatureFillet,
            "pattern" | "array" => c::FeatureType::FeaturePattern,
            "shell" => c::FeatureType::FeatureShell,
            _ => c::FeatureType::FeatureExtrude,
        };
        let op = c::Operation {
            op: Some(c::operation::Op::AddFeature(c::AddFeatureOperation {
                component_id: parent_ref.to_string(),
                feature_type: ft as i32,
                sketch_id: String::new(),
                depth: 100.0,
                direction: None,
                angle: 360.0,
                axis_origin: None,
                axis_direction: None,
                is_cut: false,
                input_sketch_ids: Vec::new(),
                axis_sketch_id: String::new(),
            })),
        };
        let name = format!("{}-on-{}", feature_type, parent_ref);
        Ok(vec![make_command(c::Action::ModifyObject, &name, None, vec![op])])
    }

    fn create_component(&self, name: &str) -> Result<Vec<c::OmniCommand>, AmirError> {
        let op = c::Operation {
            op: Some(c::operation::Op::CreateComponent(c::CreateComponentOperation {
                name: name.to_string(),
                parent_id: String::new(),
            })),
        };
        Ok(vec![make_command(c::Action::CreateObject, name, None, vec![op])])
    }

    fn boolean_op(&self, target_ref: &str, tool_ref: &str, operation: &str) -> Result<Vec<c::OmniCommand>, AmirError> {
        let op = c::Operation {
            op: Some(c::operation::Op::AddFeature(c::AddFeatureOperation {
                component_id: String::new(),
                feature_type: c::FeatureType::FeatureBoolean as i32,
                sketch_id: String::new(),
                depth: 0.0,
                direction: None,
                angle: 0.0,
                axis_origin: None,
                axis_direction: None,
                is_cut: operation != "union" && operation != "intersect",
                input_sketch_ids: vec![target_ref.to_string(), tool_ref.to_string()],
                axis_sketch_id: String::new(),
            })),
        };
        let name = format!("Boolean-{}-{}", operation, target_ref);
        Ok(vec![make_command(c::Action::ModifyObject, &name, None, vec![op])])
    }
}

// ============================================================
// Helper functions
// ============================================================

/// Create a command specifically for undo/redo.
/// These are not standard Omcom actions; the command's id starts with
/// "amir-undo" or "amir-redo" so the caller can detect them and call
/// the Executor's dedicated undo()/redo() methods.
fn make_undo_redo_command(id: &str) -> c::OmniCommand {
    c::OmniCommand {
        version: "1.0".into(),
        id: id.into(),
        action: c::Action::Validate as i32, // placeholder action
        actor: Some(c::Actor {
            id: "amir".into(),
            r#type: c::ActorType::AiAgent as i32,
        }),
        target: None,
        operations: vec![],
        validation: None,
        timestamp: prost_wkt_now(),
        metadata: Default::default(),
    }
}

fn make_command(
    action: c::Action,
    id_suffix: &str,
    target: Option<c::Target>,
    operations: Vec<c::Operation>,
) -> c::OmniCommand {
    let id = format!("amir-{}-{}", action_as_str(action), id_suffix);

    c::OmniCommand {
        version: "1.0".into(),
        id,
        action: action as i32,
        actor: Some(c::Actor {
            id: "amir".into(),
            r#type: c::ActorType::AiAgent as i32,
        }),
        target,
        operations,
        validation: None,
        timestamp: prost_wkt_now(),
        metadata: Default::default(),
    }
}

fn action_as_str(a: c::Action) -> &'static str {
    match a {
        c::Action::CreateObject => "create",
        c::Action::ModifyObject => "modify",
        c::Action::DeleteObject => "delete",
        c::Action::MoveObject => "move",
        c::Action::SetProperty => "setprop",
        c::Action::AddContainment => "addcont",
        c::Action::RemoveContainment => "rmcont",
        c::Action::AddConnection => "addconn",
        c::Action::RemoveConnection => "rmconn",
        c::Action::CreateLifecycleRecord => "create_lr",
        c::Action::ModifyLifecycleRecord => "mod_lr",
        c::Action::DeleteLifecycleRecord => "del_lr",
        c::Action::LinkRecordToObject => "link",
        c::Action::UpdateSchedule => "sched",
        c::Action::UpdateCost => "cost",
        c::Action::UpdateProgress => "progress",
        c::Action::CreateWorkOrder => "wo",
        c::Action::ImportFile => "import",
        c::Action::ExportFile => "export",
        _ => "cmd",
    }
}

fn parse_object_type(s: &str) -> c::ObjectType {
    match s.to_lowercase().as_str() {
        "site" => c::ObjectType::Site,
        "building" => c::ObjectType::Building,
        "storey" | "floor" => c::ObjectType::BuildingStorey,
        "space" => c::ObjectType::Space,
        "wall" => c::ObjectType::Wall,
        "beam" => c::ObjectType::Beam,
        "column" => c::ObjectType::Column,
        "slab" => c::ObjectType::Slab,
        "door" => c::ObjectType::Door,
        "window" => c::ObjectType::Window,
        "stair" => c::ObjectType::Stair,
        "roof" => c::ObjectType::Roof,
        "road" => c::ObjectType::Road,
        "bridge" => c::ObjectType::Bridge,
        "tunnel" => c::ObjectType::Tunnel,
        "dam" => c::ObjectType::Dam,
        "culvert" => c::ObjectType::Culvert,
        "tank" => c::ObjectType::Tank,
        "pump" => c::ObjectType::Pump,
        "valve" => c::ObjectType::Valve,
        "heat_exchanger" => c::ObjectType::HeatExchanger,
        "reactor" => c::ObjectType::Reactor,
        "pipe" => c::ObjectType::PipePlant,
        "structural_steel" => c::ObjectType::StructuralSteel,
        "part" => c::ObjectType::Part,
        "assembly" => c::ObjectType::Assembly,
        "fastener" => c::ObjectType::Fastener,
        "gear" => c::ObjectType::Gear,
        "shaft" => c::ObjectType::Shaft,
        "bearing" => c::ObjectType::Bearing,
        "motor" => c::ObjectType::Motor,
        _ => c::ObjectType::Generic,
    }
}

fn parse_phase(s: &str) -> c::LifecyclePhase {
    match s.to_lowercase().as_str() {
        "planning" => c::LifecyclePhase::Planning,
        "design" => c::LifecyclePhase::Design,
        "procurement" => c::LifecyclePhase::Procurement,
        "construction" => c::LifecyclePhase::Construction,
        "commissioning" => c::LifecyclePhase::CommissioningAcceptance,
        "asset" => c::LifecyclePhase::Asset,
        "operation" => c::LifecyclePhase::OperationMaintenance,
        "production" => c::LifecyclePhase::Production,
        "decommissioning" => c::LifecyclePhase::Decommissioning,
        _ => c::LifecyclePhase::Unspecified,
    }
}

/// Resolve a reference string to an object ID.
/// Placeholder — will be replaced with actual project lookup.
fn resolve_ref(_ref: &str) -> String {
    if _ref == "selected" || _ref.is_empty() {
        // Will be resolved by the executor or caller
        _ref.to_string()
    } else {
        _ref.to_string()
    }
}

fn json_to_property_value(value: &serde_json::Value) -> c::PropertyValue {
    match value {
        serde_json::Value::String(s) => c::PropertyValue {
            value_type: Some(c::property_value::ValueType::StringValue(s.clone())),
        },
        serde_json::Value::Number(n) => {
            // Check integer first so whole numbers (e.g. 42) map to IntValue
            // rather than DoubleValue. Falls back to f64 for fractional values.
            if let Some(i) = n.as_i64() {
                c::PropertyValue {
                    value_type: Some(c::property_value::ValueType::IntValue(i)),
                }
            } else if let Some(f) = n.as_f64() {
                c::PropertyValue {
                    value_type: Some(c::property_value::ValueType::DoubleValue(f)),
                }
            } else {
                c::PropertyValue {
                    value_type: Some(c::property_value::ValueType::StringValue(n.to_string())),
                }
            }
        }
        serde_json::Value::Bool(b) => c::PropertyValue {
            value_type: Some(c::property_value::ValueType::BoolValue(*b)),
        },
        _ => c::PropertyValue {
            value_type: Some(c::property_value::ValueType::StringValue(value.to_string())),
        },
    }
}

fn prost_wkt_now() -> Option<c::Timestamp> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now().duration_since(UNIX_EPOCH).ok()?;
    Some(c::Timestamp {
        seconds: dur.as_secs() as i64,
        nanos: dur.subsec_nanos() as i32,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_create_object() {
        let gen = OmcomGenerator::new();
        let intent = Intent::CreateObject {
            object_type: "wall".into(),
            name: Some("Main Wall".into()),
            parameters: [("length".into(), serde_json::json!(5.0))].into(),
        };
        let parsed = ParsedIntent::new(intent, "create a wall", 0.9);
        let result = gen.generate(&parsed).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].action, c::Action::CreateObject as i32);
    }

    #[test]
    fn test_generate_delete_object() {
        let gen = OmcomGenerator::new();
        let intent = Intent::DeleteObject { target_ref: "obj-1".into() };
        let parsed = ParsedIntent::new(intent, "delete object", 0.95);
        let result = gen.generate(&parsed).unwrap();
        assert_eq!(result[0].action, c::Action::DeleteObject as i32);
    }

    #[test]
    fn test_generate_set_property() {
        let gen = OmcomGenerator::new();
        let intent = Intent::SetProperty {
            target_ref: "obj-1".into(),
            property_name: "height".into(),
            property_value: serde_json::json!(3.0),
        };
        let parsed = ParsedIntent::new(intent, "set height to 3", 0.9);
        let result = gen.generate(&parsed).unwrap();
        assert_eq!(result[0].action, c::Action::SetProperty as i32);
    }

    #[test]
    fn test_generate_undo_redo() {
        let gen = OmcomGenerator::new();
        let undo = ParsedIntent::new(Intent::Undo, "undo", 0.9);

        let result = gen.generate(&undo).unwrap();
        assert_eq!(result[0].id, "amir-undo");

        let redo = ParsedIntent::new(Intent::Redo, "redo", 0.9);
        let result = gen.generate(&redo).unwrap();
        assert_eq!(result[0].id, "amir-redo");
    }

    #[test]
    fn test_generate_import_export() {
        let gen = OmcomGenerator::new();
        let import = ParsedIntent::new(Intent::ImportFile { format: "ifc".into() }, "import", 0.8);
        let export = ParsedIntent::new(Intent::ExportFile { format: "ifc".into() }, "export", 0.8);

        let result = gen.generate(&import).unwrap();
        assert_eq!(result[0].action, c::Action::ImportFile as i32);

        let result = gen.generate(&export).unwrap();
        assert_eq!(result[0].action, c::Action::ExportFile as i32);
    }

    #[test]
    fn test_generate_unknown_returns_error() {
        let gen = OmcomGenerator::new();
        let intent = Intent::Unknown { original_text: "gibberish".into() };
        let parsed = ParsedIntent::new(intent, "gibberish", 0.1);
        let result = gen.generate(&parsed);
        assert!(result.is_err());
        match result {
            Err(AmirError::UnsupportedIntent(_)) => {} // expected
            _ => panic!("Expected UnsupportedIntent error"),
        }
    }

    #[test]
    fn test_resolve_ref() {
        assert_eq!(resolve_ref("selected"), "selected");
        assert_eq!(resolve_ref(""), "");
        assert_eq!(resolve_ref("obj-42"), "obj-42");
    }

    #[test]
    fn test_json_to_property_value() {
        let s = json_to_property_value(&serde_json::json!("hello"));
        assert!(matches!(s.value_type, Some(c::property_value::ValueType::StringValue(ref v)) if v == "hello"));

        let n = json_to_property_value(&serde_json::json!(42));
        assert!(matches!(n.value_type, Some(c::property_value::ValueType::IntValue(42))));

        let f = json_to_property_value(&serde_json::json!(3.14));
        assert!(matches!(f.value_type, Some(c::property_value::ValueType::DoubleValue(v)) if (v - 3.14).abs() < 1e-10));

        let b = json_to_property_value(&serde_json::json!(true));
        assert!(matches!(b.value_type, Some(c::property_value::ValueType::BoolValue(true))));
    }
}
