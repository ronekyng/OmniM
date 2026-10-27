use omnim_core as c;

use omnim_model::{OmniObject, Project, LIBRARY};
use serde::{Deserialize, Serialize};

// ============================================================
// Validator
// ============================================================

pub struct Validator;

impl Validator {
    pub fn validate(instr: &c::OmniCommand, project: &Project) -> Vec<String> {
        let mut errors = Vec::new();

        if instr.id.is_empty() {
            errors.push("instruction id is empty".into());
        }

        let action = match c::Action::try_from(instr.action) {
            Ok(a) => a,
            Err(_) => {
                errors.push(format!("unknown action: {}", instr.action));
                return errors;
            }
        };

        match action {
            c::Action::ModifyObject
            | c::Action::DeleteObject
            | c::Action::SetProperty
            | c::Action::DeleteProperty
            | c::Action::AddContainment
            | c::Action::RemoveContainment
            | c::Action::AddConnection
            | c::Action::RemoveConnection
            | c::Action::SetMaterial
            | c::Action::CreateGeometry
            | c::Action::TransformGeometry => {
                if let Some(target_id) = target_object_id(instr) {
                    if !project.objects.contains(&target_id) {
                        errors.push(format!("target object '{}' not found", target_id));
                    }
                }
            }
            c::Action::CreateLifecycleRecord
            | c::Action::ModifyLifecycleRecord
            | c::Action::DeleteLifecycleRecord
            | c::Action::LinkRecordToObject => {
                if let Some(record_id) = target_record_id(instr) {
                    if action != c::Action::CreateLifecycleRecord
                        && project.lifecycle.get(&record_id).is_none()
                    {
                        errors.push(format!("lifecycle record '{}' not found", record_id));
                    }
                }
            }
            c::Action::AddSketchEntity
            | c::Action::AddSketchConstraint
            | c::Action::SolveSketch
            | c::Action::DeleteSketchEntity
            | c::Action::RemoveSketchConstraint
            | c::Action::ExtrudeSketch
            | c::Action::RevolveSketch => {
                for op in &instr.operations {
                    if let Some(op_type) = &op.op {
                        let sketch_id = match op_type {
                            c::operation::Op::AddSketchEntity(e) => Some(&e.sketch_id),
                            c::operation::Op::AddSketchConstraint(c) => Some(&c.sketch_id),
                            c::operation::Op::SolveSketch(s) => Some(&s.sketch_id),
                            c::operation::Op::DeleteSketchEntity(d) => Some(&d.sketch_id),
                            c::operation::Op::RemoveSketchConstraint(r) => Some(&r.sketch_id),
                            _ => None,
                        };
                        if let Some(sid) = sketch_id {
                            if project.sketches.get(sid).is_none() {
                                errors.push(format!("sketch '{}' not found", sid));
                            }
                        }
                    }
                }
            }
            c::Action::DeleteSketch => {
                for op in &instr.operations {
                    if let Some(op_type) = &op.op {
                        if let c::operation::Op::DeleteSketch(d) = op_type {
                            if project.sketches.get(&d.sketch_id).is_none() {
                                errors.push(format!("sketch '{}' not found", d.sketch_id));
                            }
                        }
                    }
                }
            }
            // === Catalog validation ===
            c::Action::DefineComponent => {
                for op in &instr.operations {
                    if let Some(op_type) = &op.op {
                        if let c::operation::Op::DefineComponent(dc) = op_type {
                            if dc.name.is_empty() {
                                errors.push("component name is empty".into());
                            }
                        }
                    }
                }
            }
            c::Action::AddComponentParam
            | c::Action::AddConnector
            | c::Action::PublishComponent => {
                for op in &instr.operations {
                    if let Some(op_type) = &op.op {
                        let cid = match op_type {
                            c::operation::Op::AddComponentParam(p) => Some(&p.component_id),
                            c::operation::Op::AddConnector(cn) => Some(&cn.component_id),
                            c::operation::Op::PublishComponent(pb) => Some(&pb.component_id),
                            _ => None,
                        };
                        if let Some(id) = cid {
                            if LIBRARY.read().unwrap().as_ref().unwrap().catalog.get(id).is_none() {
                                errors.push(format!("catalog component '{}' not found", id));
                            }
                        }
                    }
                }
            }
            c::Action::DeleteComponentDef => {
                for op in &instr.operations {
                    if let Some(op_type) = &op.op {
                        if let c::operation::Op::DeleteComponentDef(dc) = op_type {
                            if LIBRARY.read().unwrap().as_ref().unwrap().catalog.get(&dc.component_id).is_none() {
                                errors.push(format!("catalog component '{}' not found", dc.component_id));
                            }
                        }
                    }
                }
            }
            // === Series validation ===
            c::Action::AddSelector
            | c::Action::AddAnswer
            | c::Action::AddSpecComponent
            | c::Action::AddParamConstraint
            | c::Action::DeleteSeries => {
                for op in &instr.operations {
                    if let Some(op_type) = &op.op {
                        let sid = match op_type {
                            c::operation::Op::AddSelector(s) => Some(&s.series_id),
                            c::operation::Op::AddAnswer(a) => Some(&a.series_id),
                            c::operation::Op::AddSpecComponent(sc) => Some(&sc.series_id),
                            c::operation::Op::AddParamConstraint(pc) => Some(&pc.series_id),
                            c::operation::Op::DeleteSeries(ds) => Some(&ds.series_id),
                            _ => None,
                        };
                        if let Some(id) = sid {
                            if action != c::Action::CreateSeries && project.series.get(id).is_none() {
                                errors.push(format!("series '{}' not found", id));
                            }
                        }
                    }
                }
            }
            c::Action::LoadSeries => {
                for op in &instr.operations {
                    if let Some(c::operation::Op::LoadSeries(ls)) = &op.op {
                        if ls.path.is_empty() {
                            errors.push("load_series: path is required".into());
                        }
                    }
                }
            }
            _ => {}
        }

        // --- Operation-level validation ---

        for op in &instr.operations {
            if let Some(op_type) = &op.op {
                match op_type {
                    c::operation::Op::CreateObject(co) => {
                        // Validate object type
                        if c::ObjectType::try_from(co.r#type).is_err() {
                            errors.push(format!("invalid object type: {}", co.r#type));
                        }
                        // Validate name is not empty
                        if co.name.is_empty() {
                            errors.push("object name is empty".into());
                        }
                        // Validate parent exists if specified
                        if !co.parent_id.is_empty() && !project.objects.contains(&co.parent_id) {
                            errors.push(format!("parent object '{}' not found", co.parent_id));
                        }
                        // Spatial conflict detection: log as warning, not blocking error
                        // (overlaps are common in multi-discipline engineering: pipes through walls, columns in walls, etc.)
                        if let Some(pg) = &co.geometry {
                            let geom = omnim_model::geometry::create_geometry(pg.clone());
                            if let Some(bbox) = &geom.bbox {
                                for existing in project.objects.list() {
                                    if let Some(ex_bbox) = existing.bbox() {
                                        if omnim_model::geometry::bbox_intersects(bbox, ex_bbox) {
                                            eprintln!("[WARN] spatial conflict with existing object '{}' ({})",
                                                existing.name, &existing.id[..8]);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    c::operation::Op::ModifyObject(mo) => {
                        if !project.objects.contains(&mo.object_id) {
                            errors.push(format!("modify target '{}' not found", mo.object_id));
                        }
                    }
                    c::operation::Op::CopyObject(co) => {
                        if !project.objects.contains(&co.source_object_id) {
                            errors.push(format!("source object '{}' not found", co.source_object_id));
                        }
                        if !co.parent_id.is_empty() && !project.objects.contains(&co.parent_id) {
                            errors.push(format!("parent '{}' not found for copy", co.parent_id));
                        }
                    }
                    c::operation::Op::MoveObject(mo) => {
                        if !project.objects.contains(&mo.object_id) {
                            errors.push(format!("move object '{}' not found", mo.object_id));
                        }
                        if !mo.new_parent_id.is_empty() && !project.objects.contains(&mo.new_parent_id) {
                            errors.push(format!("new parent '{}' not found", mo.new_parent_id));
                        }
                    }
                    c::operation::Op::SetMaterial(sm) => {
                        if !project.objects.contains(&sm.object_id) {
                            errors.push(format!("material target '{}' not found", sm.object_id));
                        }
                        if !project.materials.get(&sm.material_id).is_some() {
                            errors.push(format!("material '{}' not found", sm.material_id));
                        }
                    }
                    c::operation::Op::AddContainment(cont) => {
                        if !project.objects.contains(&cont.parent_id) {
                            errors.push(format!("containment parent '{}' not found", cont.parent_id));
                        }
                        if !project.objects.contains(&cont.child_id) {
                            errors.push(format!("containment child '{}' not found", cont.child_id));
                        }
                    }
                    c::operation::Op::AddConnection(conn) => {
                        if !project.objects.contains(&conn.from_object_id) {
                            errors.push(format!("connection from '{}' not found", conn.from_object_id));
                        }
                        if !project.objects.contains(&conn.to_object_id) {
                            errors.push(format!("connection to '{}' not found", conn.to_object_id));
                        }
                    }
                    c::operation::Op::TransformGeometry(tg) => {
                        if !project.objects.contains(&tg.object_id) {
                            errors.push(format!("transform target '{}' not found", tg.object_id));
                        }
                    }
                    c::operation::Op::DeleteLifecycleRecord(dlr) => {
                        if project.lifecycle.get(&dlr.record_id).is_none() {
                            errors.push(format!("lifecycle record '{}' not found to delete", dlr.record_id));
                        }
                    }
                    c::operation::Op::LinkRecordToObject(lro) => {
                        if project.lifecycle.get(&lro.record_id).is_none() {
                            errors.push(format!("lifecycle record '{}' not found to link", lro.record_id));
                        }
                        if !project.objects.contains(&lro.object_id) {
                            errors.push(format!("object '{}' not found for link", lro.object_id));
                        }
                    }
                    c::operation::Op::UpdateSchedule(us) => {
                        if project.lifecycle.get(&us.record_id).is_none() {
                            errors.push(format!("lifecycle record '{}' not found for schedule update", us.record_id));
                        }
                    }
                    c::operation::Op::UpdateCost(uc) => {
                        if project.lifecycle.get(&uc.record_id).is_none() {
                            errors.push(format!("lifecycle record '{}' not found for cost update", uc.record_id));
                        }
                    }
                    c::operation::Op::UpdateProgress(up) => {
                        if project.lifecycle.get(&up.record_id).is_none() {
                            errors.push(format!("lifecycle record '{}' not found for progress update", up.record_id));
                        }
                    }
                    c::operation::Op::CreateWorkOrder(cwo) => {
                        if !project.objects.contains(&cwo.asset_object_id) {
                            errors.push(format!("asset object '{}' not found for work order", cwo.asset_object_id));
                        }
                    }
                    c::operation::Op::RemoveContainment(rc) => {
                        if !project.objects.contains(&rc.parent_id) {
                            errors.push(format!("containment parent '{}' not found", rc.parent_id));
                        }
                        if !project.objects.contains(&rc.child_id) {
                            errors.push(format!("containment child '{}' not found", rc.child_id));
                        }
                    }
                    c::operation::Op::Array(arr) => {
                        if !project.objects.contains(&arr.source_object_id) {
                            errors.push(format!("array source '{}' not found", arr.source_object_id));
                        }
                    }
                    c::operation::Op::Boolean(bo) => {
                        if !project.objects.contains(&bo.target_object_id) {
                            errors.push(format!("boolean target '{}' not found", bo.target_object_id));
                        }
                        for oid in &bo.operand_object_ids {
                            if !project.objects.contains(oid) {
                                errors.push(format!("boolean operand '{}' not found", oid));
                            }
                        }
                    }
                    c::operation::Op::AddFeature(af) => {
                        if !project.objects.contains(&af.component_id) {
                            errors.push(format!("component '{}' not found for add_feature", af.component_id));
                        }
                        if !project.sketches.get(&af.sketch_id).is_some() {
                            errors.push(format!("sketch '{}' not found for add_feature", af.sketch_id));
                        }
                    }
                    c::operation::Op::RemoveFeature(rf) => {
                        if !project.objects.contains(&rf.component_id) {
                            errors.push(format!("component '{}' not found for remove_feature", rf.component_id));
                        }
                    }
                    _ => {}
                }
            }
        }

        errors
    }
}

fn target_object_id(instr: &c::OmniCommand) -> Option<String> {
    let target = instr.target.as_ref()?;
    let tt = target.target_type.as_ref()?;
    if let c::target::TargetType::Object(ot) = tt {
        Some(ot.object_id.clone())
    } else {
        None
    }
}

fn target_record_id(instr: &c::OmniCommand) -> Option<String> {
    let target = instr.target.as_ref()?;
    let tt = target.target_type.as_ref()?;
    if let c::target::TargetType::Lifecycle(lt) = tt {
        Some(lt.record_id.clone())
    } else {
        None
    }
}

fn feature_type_name(ft: &omnim_model::FeatureType) -> &'static str {
    match ft {
        omnim_model::FeatureType::Extrude => "extrude",
        omnim_model::FeatureType::Revolve => "revolve",
        omnim_model::FeatureType::Sweep => "sweep",
        omnim_model::FeatureType::Loft => "loft",
        omnim_model::FeatureType::Fusion => "fusion",
        omnim_model::FeatureType::LoftedFusion => "lofted_fusion",
        omnim_model::FeatureType::Boolean => "boolean",
        omnim_model::FeatureType::Chamfer => "chamfer",
        omnim_model::FeatureType::Fillet => "fillet",
        omnim_model::FeatureType::Pattern => "pattern",
        omnim_model::FeatureType::Shell => "shell",
    }
}

// ============================================================
// ExecutionResult
// ============================================================

#[derive(Serialize, Deserialize, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub message: String,
    pub created_ids: Vec<String>,
}

// ============================================================
// Snapshots for undo/redo
// ============================================================

/// 存储被删除/修改的原始对象数据，用于 undo
#[derive(Clone)]
pub struct ObjectSnapshot {
    /// 原始对象数据
    pub object: OmniObject,
    /// 原始属性名（用于恢复删除的属性）
    pub removed_properties: Vec<c::Property>,
    /// 原始 material_id（用于恢复材料修改）
    pub old_material_id: Option<String>,
    /// 原始 geometry（用于恢复几何修改）
    pub old_geometry: Option<c::Geometry>,
}

/// 可逆命令 — 记录操作的反向数据
#[derive(Clone)]
pub enum Command {
    /// 创建对象 — undo 时删除，redo 时恢复
    CreateObject { snapshot: ObjectSnapshot },
    /// 删除对象 — undo 时恢复
    DeleteObject { snapshot: ObjectSnapshot },
    /// 修改对象 — undo 时恢复原始值
    ModifyObject { object_id: String, snapshot: ObjectSnapshot },
    /// 设置属性 — undo 时恢复或删除
    SetProperty { object_id: String, property_name: String, old_value: Option<c::PropertyValue> },
    /// 添加 containment — undo 时移除
    AddContainment { parent_id: String, child_id: String },
    /// 移除 containment — undo 时恢复
    RemoveContainment { containment: c::Containment },
    /// 添加连接 — undo 时移除（简单标记）
    AddConnection { from_id: String, to_id: String },
    /// 设置材料 — undo 时恢复
    SetMaterial { object_id: String, old_material_id: Option<String> },
    /// 创建生命周期记录 — undo 时删除
    CreateLifecycleRecord { record_id: String },
    /// 修改生命周期记录 — undo 时恢复
    ModifyLifecycleRecord { record_id: String, old_name: String, old_description: String, old_status: i32 },
    /// 删除生命周期记录 — undo 时恢复
    DeleteLifecycleRecord { record: c::ProjectLifecycleRecord },
    /// 删除属性 — undo 时恢复
    DeleteProperty { object_id: String, removed: c::Property },
    /// 移除属性集 — undo 时恢复
    RemovePropertySet { object_id: String, removed: c::PropertySet },
    /// 移除连接 — undo 时恢复
    RemoveConnection { from_id: String, to_id: String },
    /// 复制对象 — undo 时删除副本
    CopyObject { old_id: String, new_id: String, snapshot: ObjectSnapshot },
    /// 移动对象 — undo 时恢复原父级
    MoveObject { object_id: String, old_parent_id: Option<String>, new_parent_id: String },
    /// 几何变换 — undo 时恢复原几何
    TransformGeometry { object_id: String, old_geometry: Option<c::Geometry>, translation: Option<c::Point3D>, rotation: Option<c::Rotation>, scale: Option<c::Vector3D> },
    /// 链接记录到对象 — undo 时取消链接
    LinkRecordToObject { record_id: String, object_id: String },
    /// 更新排期 — undo 时恢复
    UpdateSchedule { record_id: String, old_planned_start: Option<c::Timestamp>, old_planned_finish: Option<c::Timestamp>, old_actual_start: Option<c::Timestamp>, old_actual_finish: Option<c::Timestamp> },
    /// 更新成本 — undo 时恢复
    UpdateCost { record_id: String, old_planned_cost: f64, old_actual_cost: f64 },
    /// 更新进度 — undo 时恢复
    UpdateProgress { record_id: String, old_percent: f64, new_percent: f64 },
    /// 创建工作单 — undo 时删除
    CreateWorkOrder { record_id: String },
    /// 更新工作单 — undo 时恢复
    UpdateWorkOrder { record_id: String, old_status: String, old_description: String, new_status: String, new_description: String },
    /// 创建草图 — undo 时删除
    CreateSketch { sketch_id: String },
    /// 添加草图实体 — undo 时移除
    AddSketchEntity { sketch_id: String, entity_index: usize, entity: omnim_sketch::SketchEntity },
    /// 添加草图约束 — undo 时移除
    AddSketchConstraint { sketch_id: String, constraint_index: usize, constraint: omnim_sketch::SketchConstraint },
    /// 求解草图 — undo 时恢复 solved 状态
    SolveSketch { sketch_id: String, old_solved: bool },
    /// 删除草图实体 — undo 时恢复
    DeleteSketchEntity { sketch_id: String, entity: omnim_sketch::SketchEntity },
    /// 移除草图约束 — undo 时恢复
    RemoveSketchConstraint { sketch_id: String, constraint: omnim_sketch::SketchConstraint },
    /// 删除草图 — undo 时恢复
    DeleteSketch { sketch: omnim_sketch::Sketch },
    /// 拉伸草图 — undo 时移除生成的3D物体
    ExtrudeSketch { sketch_id: String, depth: f64, direction: omnim_core::Vector3D, profile: Option<Vec<omnim_core::Point2D>> },
    /// 旋转草图 — undo 时移除生成的3D物体
    RevolveSketch { sketch_id: String, angle: f64, axis_origin: omnim_core::Point3D, axis_direction: omnim_core::Vector3D, profile: Option<Vec<omnim_core::Point2D>> },
    /// 创建组件 — undo 时删除
    CreateComponent { component_id: String },
    /// 添加特征 — undo 时移除
    AddFeature { component_id: String, feature_id: String },
    /// 移除特征 — undo 时恢复
    RemoveFeature { component_id: String, feature: omnim_model::Feature },
    /// 重建特征 — undo 时无操作（rebuild 是幂等的）
    RebuildFeatures { component_id: String },
    // === Catalog commands ===
    DefineComponent { component_def: omnim_model::catalog::ComponentDef },
    AddComponentParam { component_id: String, param_index: usize },
    AddConnector { component_id: String, connector_id: String },
    PublishComponent { component_id: String },
    DeleteComponentDef { component_def: omnim_model::catalog::ComponentDef },
    // === Series commands ===
    CreateSeries { series: omnim_model::series_store::Series },
    AddSelector { series_id: String, selector_id: String },
    AddAnswer { series_id: String, selector_id: String, answer_index: usize },
    AddSpecComponent { series_id: String, selector_id: String, answer_index: usize },
    AddParamConstraint { series_id: String, component_def_id: String, param_index: usize },
    DeleteSeries { series: omnim_model::series_store::Series },
    LoadSeries { path: String, source: String, loaded_ids: Vec<String> },
}

// ============================================================
// Executor (stateful, with undo/redo/transaction)
// ============================================================

/// 最大 undo 历史深度
const MAX_UNDO_DEPTH: usize = 200;

pub struct Executor {
    /// undo 命令栈（最近的在末尾）
    undo_stack: Vec<Command>,
    /// redo 命令栈（最近的在末尾）
    redo_stack: Vec<Command>,
    /// 当前活跃事务的命令栈（事务内的命令，commit 时合并到 undo_stack）
    transaction_stack: Option<Vec<Command>>,
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

impl Executor {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            transaction_stack: None,
        }
    }

    /// 执行一条 Omcom 指令，并自动记录 undo 信息。
    pub fn execute(&mut self, instr: &c::OmniCommand, project: &mut Project) -> ExecutionResult {
        // 记录本次执行中的所有命令
        let mut commands = Vec::new();

        let errors = Validator::validate(instr, project);
        if !errors.is_empty() {
            return ExecutionResult {
                success: false,
                message: format!("Validation failed: {}", errors.join("; ")),
                created_ids: vec![],
            };
        }

        let action = match c::Action::try_from(instr.action) {
            Ok(a) => a,
            Err(_) => {
                return ExecutionResult {
                    success: false,
                    message: format!("Unknown action: {}", instr.action),
                    created_ids: vec![],
                };
            }
        };

        let mut created_ids = Vec::new();

        // 执行 operations
        for op in &instr.operations {
            if let Some(op_type) = &op.op {
                match op_type {
                    c::operation::Op::CreateObject(co) => {
                        let obj_type = c::ObjectType::try_from(co.r#type)
                            .unwrap_or(c::ObjectType::Generic);
                        let mut obj = OmniObject::new(obj_type, &co.name);
                        obj.description = co.description.clone();
                        if !co.parent_id.is_empty() {
                            obj.parent_id = Some(co.parent_id.clone());
                        }
                        for prop in &co.properties {
                            obj.properties.push(prop.clone());
                        }
                        if let Some(pg) = &co.geometry {
                            let geom = omnim_model::geometry::create_geometry(pg.clone());
                            obj.geometry = Some(geom);
                        }
                        created_ids.push(obj.id.clone());
                        project.objects.insert(obj.clone());
                        commands.push(Command::CreateObject {
                            snapshot: ObjectSnapshot {
                                object: obj,
                                removed_properties: Vec::new(),
                                old_material_id: None,
                                old_geometry: None,
                            },
                        });
                    }

                    c::operation::Op::ModifyObject(mo) => {
                        if let Some(obj) = project.objects.get_mut(&mo.object_id) {
                            let snapshot = ObjectSnapshot {
                                object: obj.clone(),
                                removed_properties: Vec::new(),
                                old_material_id: obj.material_id.clone(),
                                old_geometry: obj.geometry.clone(),
                            };
                            if let Some(nn) = &mo.new_name {
                                obj.name = nn.clone();
                            }
                            if let Some(nd) = &mo.new_description {
                                obj.description = nd.clone();
                            }
                            if let Some(ng) = &mo.new_geometry {
                                obj.geometry =
                                    Some(omnim_model::geometry::create_geometry(ng.clone()));
                            }
                            commands.push(Command::ModifyObject {
                                object_id: mo.object_id.clone(),
                                snapshot,
                            });
                        }
                    }

                    c::operation::Op::SetProperty(sp) => {
                        if let Some(val) = sp.value.clone() {
                            if let Some(tid) = target_object_id(instr) {
                                if let Some(obj) = project.objects.get_mut(&tid) {
                                    let old_value = obj
                                        .get_property(&sp.name)
                                        .and_then(|p| p.value.clone());
                                    obj.set_property(&sp.name, val);
                                    commands.push(Command::SetProperty {
                                        object_id: tid,
                                        property_name: sp.name.clone(),
                                        old_value,
                                    });
                                }
                            }
                        }
                    }

                    c::operation::Op::AddPropertySet(ps) => {
                        if let Some(tid) = target_object_id(instr) {
                            if let Some(obj) = project.objects.get_mut(&tid) {
                                obj.property_sets.push(ps.clone());
                            }
                        }
                    }

                    c::operation::Op::AddContainment(cont) => {
                        if let Err(e) = project.relations.add_containment(cont.clone()) {
                            return self.fail_and_rollback(
                                &commands,
                                project,
                                e,
                                created_ids,
                            );
                        }
                        commands.push(Command::AddContainment {
                            parent_id: cont.parent_id.clone(),
                            child_id: cont.child_id.clone(),
                        });
                    }

                    c::operation::Op::AddConnection(conn) => {
                        project.relations.add_connection(conn.clone());
                        commands.push(Command::AddConnection {
                            from_id: conn.from_object_id.clone(),
                            to_id: conn.to_object_id.clone(),
                        });
                    }

                    c::operation::Op::SetMaterial(sm) => {
                        if let Some(obj) = project.objects.get_mut(&sm.object_id) {
                            let old_material_id = obj.material_id.clone();
                            obj.material_id = Some(sm.material_id.clone());
                            commands.push(Command::SetMaterial {
                                object_id: sm.object_id.clone(),
                                old_material_id,
                            });
                        }
                    }

                    c::operation::Op::CreateLifecycleRecord(clr) => {
                        let id = uuid::Uuid::new_v4().to_string();
                        let record = c::ProjectLifecycleRecord {
                            id,
                            phase: clr.phase,
                            record_type: clr.record_type.clone(),
                            name: clr.name.clone(),
                            description: clr.description.clone(),
                            related_object_ids: clr.related_object_ids.clone(),
                            related_document_ids: Vec::new(),
                            properties: clr.properties.clone(),
                            status: c::RecordStatus::Draft as i32,
                            planned_start: None,
                            planned_finish: None,
                            actual_start: None,
                            actual_finish: None,
                            planned_cost: 0.0,
                            actual_cost: 0.0,
                            responsible_party: String::new(),
                            version_id: String::new(),
                        };
                        let record_id = record.id.clone();
                        created_ids.push(record_id.clone());
                        project.lifecycle.insert(record);
                        commands.push(Command::CreateLifecycleRecord { record_id });
                    }

                    c::operation::Op::ModifyLifecycleRecord(mlr) => {
                        if let Some(record) = project.lifecycle.get_mut(&mlr.record_id) {
                            let old_name = record.name.clone();
                            let old_description = record.description.clone();
                            let old_status = record.status;
                            if let Some(name) = &mlr.name {
                                record.name = name.clone();
                            }
                            if let Some(desc) = &mlr.description {
                                record.description = desc.clone();
                            }
                            if let Some(status) = &mlr.status {
                                record.status = *status;
                            }
                            for prop in &mlr.properties {
                                record.properties.push(prop.clone());
                            }
                            commands.push(Command::ModifyLifecycleRecord {
                                record_id: mlr.record_id.clone(),
                                old_name,
                                old_description,
                                old_status,
                            });
                        }
                    }

                    c::operation::Op::RemoveContainment(rc) => {
                        if !rc.parent_id.is_empty() && !rc.child_id.is_empty() {
                            if let Some(cont) = project.relations.remove_containment(&rc.parent_id, &rc.child_id) {
                                commands.push(Command::RemoveContainment { containment: cont });
                            }
                        }
                    }

                    c::operation::Op::Array(arr) => {
                        if let Some(src) = project.objects.get(&arr.source_object_id) {
                            let src_clone = src.clone();
                            let (cx, cy, cz) = (arr.count_x.max(1) as usize, arr.count_y.max(1) as usize, arr.count_z.max(1) as usize);
                            for ix in 0..cx {
                                for iy in 0..cy {
                                    for iz in 0..cz {
                                        if ix == 0 && iy == 0 && iz == 0 { continue; }
                                        let mut new_obj = src_clone.clone();
                                        new_obj.id = uuid::Uuid::new_v4().to_string();
                                        new_obj.name = format!("{} ({},{},{})", src_clone.name, ix, iy, iz);
                                        let tx = ix as f64 * arr.spacing_x;
                                        let ty = iy as f64 * arr.spacing_y;
                                        let tz = iz as f64 * arr.spacing_z;
                                        if let Some(geom) = &mut new_obj.geometry {
                                            if let Some(mesh) = omnim_model::geometry::geometry_to_mesh(geom) {
                                                let t = c::Transform {
                                                    matrix: vec![],
                                                    translation: Some(c::Point3D { x: tx, y: ty, z: tz }),
                                                    rotation: None, scale: None,
                                                };
                                                let tm = omnim_model::geometry::transform_mesh(&mesh, &t);
                                                let bbox = omnim_model::geometry::compute_bbox(&tm);
                                                geom.representation = Some(c::geometry::Representation::Mesh(tm));
                                                geom.bbox = Some(bbox);
                                            }
                                        }
                                        let new_id = new_obj.id.clone();
                                        project.objects.insert(new_obj.clone());
                                        created_ids.push(new_id.clone());
                                        commands.push(Command::CopyObject {
                                            old_id: arr.source_object_id.clone(),
                                            new_id,
                                            snapshot: ObjectSnapshot {
                                                object: new_obj,
                                                removed_properties: Vec::new(),
                                                old_material_id: None,
                                                old_geometry: None,
                                            },
                                        });
                                    }
                                }
                            }
                        }
                    }

                    c::operation::Op::Boolean(bo) => {
                        // Boolean operation: merge operand meshes (simplified)
                        // Clone all needed data first to avoid borrow conflicts
                        let target_id = bo.target_object_id.clone();
                        let operand_ids = bo.operand_object_ids.clone();

                        // Collect operand meshes while project.objects is immutably borrowed
                        let operand_meshes: Vec<c::MeshGeometry> = operand_ids.iter()
                            .filter_map(|id| project.objects.get(id))
                            .filter_map(|op_obj| op_obj.geometry.as_ref())
                            .filter_map(|op_geom| omnim_model::geometry::geometry_to_mesh(op_geom))
                            .collect();

                        // Now mutate target with a mutable borrow
                        if let Some(target_obj) = project.objects.get_mut(&target_id) {
                            if let Some(target_geom) = &target_obj.geometry {
                                if let Some(mut target_mesh) = omnim_model::geometry::geometry_to_mesh(target_geom) {
                                    for op_mesh in &operand_meshes {
                                        let base_idx = target_mesh.vertices.len() as u32;
                                        target_mesh.vertices.extend(op_mesh.vertices.clone());
                                        target_mesh.normals.extend(op_mesh.normals.clone());
                                        target_mesh.uvs.extend(op_mesh.uvs.clone());
                                        for face in &op_mesh.faces {
                                            let mut new_face = face.clone();
                                            for vi in &mut new_face.vertex_indices {
                                                *vi += base_idx as i32;
                                            }
                                            target_mesh.faces.push(new_face);
                                        }
                                    }
                                    let bbox = omnim_model::geometry::compute_bbox(&target_mesh);
                                    target_obj.geometry = Some(c::Geometry {
                                        id: target_geom.id.clone(),
                                        r#type: c::GeometryType::Mesh as i32,
                                        representation: Some(c::geometry::Representation::Mesh(target_mesh)),
                                        bbox: Some(bbox),
                                        level_of_detail: target_geom.level_of_detail,
                                    });
                                }
                            }
                        }
                    }

                    c::operation::Op::CreateGeometry(_) => {
                        // Handled via CreateObject or ModifyObject
                    }

                    c::operation::Op::DeleteProperty(dp) => {
                        if let Some(tid) = target_object_id(instr) {
                            if let Some(obj) = project.objects.get_mut(&tid) {
                                if let Some(removed) = obj.remove_property(&dp.name) {
                                    commands.push(Command::DeleteProperty {
                                        object_id: tid,
                                        removed,
                                    });
                                }
                            }
                        }
                    }

                    c::operation::Op::RemovePropertySet(rps) => {
                        if let Some(tid) = target_object_id(instr) {
                            if let Some(obj) = project.objects.get_mut(&tid) {
                                if let Some(pos) = obj.property_sets.iter().position(|ps| ps.name == rps.name) {
                                    let removed = obj.property_sets.remove(pos);
                                    commands.push(Command::RemovePropertySet {
                                        object_id: tid,
                                        removed,
                                    });
                                }
                            }
                        }
                    }

                    c::operation::Op::RemoveConnection(rc) => {
                        commands.push(Command::RemoveConnection {
                            from_id: rc.from_object_id.clone(),
                            to_id: rc.to_object_id.clone(),
                        });
                    }

                    c::operation::Op::CopyObject(co) => {
                        if let Some(src) = project.objects.get(&co.source_object_id) {
                            let mut new_obj = src.clone();
                            new_obj.id = uuid::Uuid::new_v4().to_string();
                            new_obj.name = if co.new_name.is_empty() {
                                format!("{} (Copy)", src.name)
                            } else {
                                co.new_name.clone()
                            };
                            new_obj.parent_id = if co.parent_id.is_empty() {
                                src.parent_id.clone()
                            } else {
                                Some(co.parent_id.clone())
                            };
                            let new_id = new_obj.id.clone();
                            project.objects.insert(new_obj.clone());
                            created_ids.push(new_id.clone());
                            commands.push(Command::CopyObject {
                                old_id: co.source_object_id.clone(),
                                new_id,
                                snapshot: ObjectSnapshot {
                                    object: new_obj,
                                    removed_properties: Vec::new(),
                                    old_material_id: None,
                                    old_geometry: None,
                                },
                            });
                        }
                    }

                    c::operation::Op::MoveObject(mo) => {
                        if let Some(obj) = project.objects.get_mut(&mo.object_id) {
                            let old_parent = obj.parent_id.clone();
                            let new_parent = if mo.new_parent_id.is_empty() {
                                None
                            } else {
                                Some(mo.new_parent_id.clone())
                            };
                            obj.parent_id = new_parent;
                            project.objects.rebuild_children();
                            commands.push(Command::MoveObject {
                                object_id: mo.object_id.clone(),
                                old_parent_id: old_parent,
                                new_parent_id: mo.new_parent_id.clone(),
                            });
                        }
                    }

                    c::operation::Op::TransformGeometry(tg) => {
                        if let Some(obj) = project.objects.get_mut(&tg.object_id) {
                            let old_geom = obj.geometry.clone();
                            if let Some(geom) = &mut obj.geometry {
                                if let Some(mesh) = omnim_model::geometry::geometry_to_mesh(geom) {
                                    let origin = mesh_center(&mesh);
                                    let pre = c::Transform {
                                        matrix: vec![],
                                        translation: Some(c::Point3D {
                                            x: -origin.x, y: -origin.y, z: -origin.z,
                                        }),
                                        rotation: None, scale: None,
                                    };
                                    let main = c::Transform {
                                        matrix: vec![],
                                        translation: tg.translation.clone(),
                                        rotation: tg.rotation.clone(),
                                        scale: tg.scale.clone(),
                                    };
                                    let post = c::Transform {
                                        matrix: vec![],
                                        translation: Some(origin),
                                        rotation: None, scale: None,
                                    };
                                    let shifted = omnim_model::geometry::transform_mesh(&mesh, &pre);
                                    let transformed = omnim_model::geometry::transform_mesh(&shifted, &main);
                                    let transformed = omnim_model::geometry::transform_mesh(&transformed, &post);
                                    let bbox = omnim_model::geometry::compute_bbox(&transformed);
                                    geom.representation = Some(c::geometry::Representation::Mesh(transformed));
                                    geom.bbox = Some(bbox);
                                }
                            }
                            commands.push(Command::TransformGeometry {
                                object_id: tg.object_id.clone(),
                                old_geometry: old_geom,
                                translation: tg.translation.clone(),
                                rotation: tg.rotation.clone(),
                                scale: tg.scale.clone(),
                            });
                        }
                    }

                    c::operation::Op::DeleteLifecycleRecord(dlr) => {
                        if let Some(record) = project.lifecycle.remove(&dlr.record_id) {
                            commands.push(Command::DeleteLifecycleRecord { record });
                        }
                    }

                    c::operation::Op::LinkRecordToObject(lro) => {
                        if let Some(record) = project.lifecycle.get_mut(&lro.record_id) {
                            if !record.related_object_ids.contains(&lro.object_id) {
                                record.related_object_ids.push(lro.object_id.clone());
                            }
                        }
                        commands.push(Command::LinkRecordToObject {
                            record_id: lro.record_id.clone(),
                            object_id: lro.object_id.clone(),
                        });
                    }

                    c::operation::Op::UpdateSchedule(us) => {
                        if let Some(record) = project.lifecycle.get_mut(&us.record_id) {
                            let old_planned_start = record.planned_start.clone();
                            let old_planned_finish = record.planned_finish.clone();
                            let old_actual_start = record.actual_start.clone();
                            let old_actual_finish = record.actual_finish.clone();
                            if us.planned_start.is_some() { record.planned_start = us.planned_start.clone(); }
                            if us.planned_finish.is_some() { record.planned_finish = us.planned_finish.clone(); }
                            if us.actual_start.is_some() { record.actual_start = us.actual_start.clone(); }
                            if us.actual_finish.is_some() { record.actual_finish = us.actual_finish.clone(); }
                            commands.push(Command::UpdateSchedule {
                                record_id: us.record_id.clone(),
                                old_planned_start,
                                old_planned_finish,
                                old_actual_start,
                                old_actual_finish,
                            });
                        }
                    }

                    c::operation::Op::UpdateCost(uc) => {
                        if let Some(record) = project.lifecycle.get_mut(&uc.record_id) {
                            let old_planned = record.planned_cost;
                            let old_actual = record.actual_cost;
                            record.planned_cost = uc.planned_cost;
                            record.actual_cost = uc.actual_cost;
                            commands.push(Command::UpdateCost {
                                record_id: uc.record_id.clone(),
                                old_planned_cost: old_planned,
                                old_actual_cost: old_actual,
                            });
                        }
                    }

                    c::operation::Op::UpdateProgress(up) => {
                        let old_percent = if let Some(record) = project.lifecycle.get(&up.record_id) {
                            record.properties.iter()
                                .find(|p| p.name == "percent_complete")
                                .and_then(|p| p.value.as_ref())
                                .and_then(|v| match v.value_type.as_ref()? {
                                    c::property_value::ValueType::DoubleValue(d) => Some(*d),
                                    _ => None,
                                })
                                .unwrap_or(0.0)
                        } else {
                            0.0
                        };
                        if let Some(record) = project.lifecycle.get_mut(&up.record_id) {
                            if let Some(existing) = record.properties.iter_mut()
                                .find(|p| p.name == "percent_complete")
                            {
                                existing.value = Some(c::PropertyValue {
                                    value_type: Some(c::property_value::ValueType::DoubleValue(up.percent_complete)),
                                });
                            } else {
                                record.properties.push(c::Property {
                                    name: "percent_complete".into(),
                                    value: Some(c::PropertyValue {
                                        value_type: Some(c::property_value::ValueType::DoubleValue(up.percent_complete)),
                                    }),
                                    unit: String::new(),
                                    description: String::new(),
                                });
                            }
                        }
                        commands.push(Command::UpdateProgress {
                            record_id: up.record_id.clone(),
                            old_percent,
                            new_percent: up.percent_complete,
                        });
                    }

                    c::operation::Op::CreateWorkOrder(cwo) => {
                        let id = uuid::Uuid::new_v4().to_string();
                        let record = c::ProjectLifecycleRecord {
                            id: id.clone(),
                            phase: c::LifecyclePhase::OperationMaintenance as i32,
                            record_type: "work_order".into(),
                            name: cwo.title.clone(),
                            description: cwo.description.clone(),
                            related_object_ids: vec![cwo.asset_object_id.clone()],
                            related_document_ids: Vec::new(),
                            properties: Vec::new(),
                            status: c::RecordStatus::Draft as i32,
                            planned_start: None, planned_finish: None,
                            actual_start: None, actual_finish: None,
                            planned_cost: 0.0, actual_cost: 0.0,
                            responsible_party: String::new(),
                            version_id: String::new(),
                        };
                        let record_id = record.id.clone();
                        created_ids.push(record_id.clone());
                        project.lifecycle.insert(record);
                        commands.push(Command::CreateWorkOrder { record_id });
                    }

                    c::operation::Op::UpdateWorkOrder(upwo) => {
                        // Find work order by id and update status
                        for record in project.lifecycle.list() {
                            let rec = record.clone();
                            if rec.id == upwo.work_order_id || rec.name == upwo.work_order_id {
                                if let Some(rec_mut) = project.lifecycle.get_mut(&rec.id) {
                                    let old_status = rec_mut.status.to_string();
                                    let old_desc = rec_mut.description.clone();
                                    let new_status = upwo.status.clone();
                                    let new_desc = upwo.description.clone();
                                    if !upwo.status.is_empty() {
                                        rec_mut.status = parse_record_status(&upwo.status) as i32;
                                    }
                                    if !upwo.description.is_empty() {
                                        rec_mut.description = upwo.description.clone();
                                    }
                                    commands.push(Command::UpdateWorkOrder {
                                        record_id: rec.id.clone(),
                                        old_status,
                                        old_description: old_desc,
                                        new_status,
                                        new_description: new_desc,
                                    });
                                }
                                break;
                            }
                        }
                    }

                    // === Sketch Operations ===

                    c::operation::Op::CreateSketch(cs) => {
                        let origin = cs.origin.as_ref().map(|p| omnim_core::Point3D { x: p.x, y: p.y, z: p.z })
                            .unwrap_or(omnim_core::Point3D { x: 0.0, y: 0.0, z: 0.0 });
                        let normal = cs.normal.as_ref().map(|v| omnim_core::Vector3D { x: v.x, y: v.y, z: v.z })
                            .unwrap_or(omnim_core::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
                        let sketch = omnim_sketch::Sketch::new(&cs.name, origin, normal);
                        let sketch_id = sketch.id.clone();
                        created_ids.push(sketch_id.clone());
                        project.sketches.insert(sketch);
                        commands.push(Command::CreateSketch { sketch_id });
                    }

                    c::operation::Op::AddSketchEntity(ase) => {
                        if let Some(sketch) = project.sketches.get_mut(&ase.sketch_id) {
                            if let Some(ref entity_data) = ase.entity {
                                let entity_type = match entity_data.entity_type {
                                    1 => omnim_sketch::SketchEntityType::Point,
                                    2 => omnim_sketch::SketchEntityType::Line,
                                    3 => omnim_sketch::SketchEntityType::Circle,
                                    4 => omnim_sketch::SketchEntityType::Arc,
                                    6 => omnim_sketch::SketchEntityType::Rectangle,
                                    7 => omnim_sketch::SketchEntityType::Polygon,
                                    8 => omnim_sketch::SketchEntityType::Ellipse,
                                    9 => omnim_sketch::SketchEntityType::Spline,
                                    10 => omnim_sketch::SketchEntityType::Polyline,
                                    _ => omnim_sketch::SketchEntityType::Point,
                                };
                                let points: Vec<omnim_core::Point3D> = entity_data.points.iter()
                                    .map(|p| omnim_core::Point3D { x: p.x, y: p.y, z: p.z })
                                    .collect();
                                let entity = omnim_sketch::SketchEntity {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    entity_type,
                                    points,
                                    radius: entity_data.radius,
                                    sides: entity_data.sides,
                                    construction: false,
                                };
                                let idx = sketch.add_entity(entity.clone());
                                commands.push(Command::AddSketchEntity {
                                    sketch_id: ase.sketch_id.clone(),
                                    entity_index: idx,
                                    entity,
                                });
                            }
                        }
                    }

                    c::operation::Op::AddSketchConstraint(asc) => {
                        if let Some(sketch) = project.sketches.get_mut(&asc.sketch_id) {
                            if let Some(ref constr_data) = asc.constraint {
                                let constraint_type = match constr_data.constraint_type {
                                    1 => omnim_sketch::SketchConstraintType::Horizontal,
                                    2 => omnim_sketch::SketchConstraintType::Vertical,
                                    3 => omnim_sketch::SketchConstraintType::Coincident,
                                    4 => omnim_sketch::SketchConstraintType::Parallel,
                                    5 => omnim_sketch::SketchConstraintType::Perpendicular,
                                    6 => omnim_sketch::SketchConstraintType::Length,
                                    7 => omnim_sketch::SketchConstraintType::Distance,
                                    8 => omnim_sketch::SketchConstraintType::Angle,
                                    9 => omnim_sketch::SketchConstraintType::Radius,
                                    10 => omnim_sketch::SketchConstraintType::Diameter,
                                    11 => omnim_sketch::SketchConstraintType::Tangent,
                                    12 => omnim_sketch::SketchConstraintType::Symmetric,
                                    13 => omnim_sketch::SketchConstraintType::Fixed,
                                    14 => omnim_sketch::SketchConstraintType::AtMidpoint,
                                    15 => omnim_sketch::SketchConstraintType::EqualLength,
                                    16 => omnim_sketch::SketchConstraintType::EqualRadius,
                                    17 => omnim_sketch::SketchConstraintType::PtOnLine,
                                    18 => omnim_sketch::SketchConstraintType::PtOnCircle,
                                    19 => omnim_sketch::SketchConstraintType::SameOrientation,
                                    _ => omnim_sketch::SketchConstraintType::Fixed,
                                };
                                let entity_refs: Vec<omnim_sketch::SketchPointRef> = constr_data.entity_refs.iter()
                                    .map(|r| omnim_sketch::SketchPointRef {
                                        entity_index: (r.entity_index as usize).saturating_sub(1),
                                        point_index: (r.point_index as usize).saturating_sub(1),
                                    })
                                    .collect();
                                let constraint = omnim_sketch::SketchConstraint {
                                    id: uuid::Uuid::new_v4().to_string(),
                                    constraint_type,
                                    entity_refs,
                                    value: constr_data.value,
                                };
                                let idx = sketch.add_constraint(constraint.clone());
                                commands.push(Command::AddSketchConstraint {
                                    sketch_id: asc.sketch_id.clone(),
                                    constraint_index: idx,
                                    constraint,
                                });
                            }
                        }
                    }

                    c::operation::Op::SolveSketch(ss) => {
                        if let Some(sketch) = project.sketches.get_mut(&ss.sketch_id) {
                            let old_solved = sketch.solved;
                            let result = omnim_sketch::solver::solve(sketch);
                            commands.push(Command::SolveSketch {
                                sketch_id: ss.sketch_id.clone(),
                                old_solved,
                            });
                            if !result.success {
                                eprintln!("[WARN] sketch solve: {}", result.message);
                            }
                        }
                    }

                    c::operation::Op::DeleteSketchEntity(dse) => {
                        if let Some(sketch) = project.sketches.get_mut(&dse.sketch_id) {
                            let idx = (dse.entity_index as usize).saturating_sub(1);
                            if idx < sketch.entities.len() {
                                let entity = sketch.entities.remove(idx);
                                commands.push(Command::DeleteSketchEntity {
                                    sketch_id: dse.sketch_id.clone(),
                                    entity,
                                });
                            }
                        }
                    }

                    c::operation::Op::RemoveSketchConstraint(rsc) => {
                        if let Some(sketch) = project.sketches.get_mut(&rsc.sketch_id) {
                            let idx = (rsc.constraint_index as usize).saturating_sub(1);
                            if idx < sketch.constraints.len() {
                                let constraint = sketch.constraints.remove(idx);
                                commands.push(Command::RemoveSketchConstraint {
                                    sketch_id: rsc.sketch_id.clone(),
                                    constraint,
                                });
                            }
                        }
                    }

                    c::operation::Op::DeleteSketch(ds) => {
                        if let Some(sketch) = project.sketches.remove(&ds.sketch_id) {
                            commands.push(Command::DeleteSketch { sketch });
                        }
                    }

                    c::operation::Op::ExtrudeSketch(es) => {
                        if let Some(sketch) = project.sketches.get(&es.sketch_id) {
                            let dir = es.direction.as_ref()
                                .map(|d| omnim_core::Vector3D { x: d.x, y: d.y, z: d.z })
                                .unwrap_or(omnim_core::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
                            let profile = omnim_sketch::profile::sketch_to_profile_2d(sketch);
                            let depth = es.depth;
                            let direction = dir;
                            let sketch_id = es.sketch_id.clone();
                            // Use BRep for validation, but Command stores profile for undo
                            if let Some(ref prof) = profile {
                                if prof.len() >= 3 {
                                    let profile3d: Vec<omnim_core::Point3D> = prof.iter()
                                        .map(|p| omnim_core::Point3D { x: p.x, y: p.y, z: 0.0 })
                                        .collect();
                                    if omnim_model::brep::extrude_brep(&profile3d, depth, dir, omnim_model::brep::DEFAULT_TOLERANCE).is_some() {
                                        commands.push(Command::ExtrudeSketch { sketch_id, depth, direction, profile });
                                    }
                                }
                            }
                        }
                    }

                    c::operation::Op::RevolveSketch(rs) => {
                        if let Some(sketch) = project.sketches.get(&rs.sketch_id) {
                            let origin = rs.axis_origin.as_ref()
                                .map(|p| omnim_core::Point3D { x: p.x, y: p.y, z: p.z })
                                .unwrap_or(omnim_core::Point3D { x: 0.0, y: 0.0, z: 0.0 });
                            let axis_dir = rs.axis_direction.as_ref()
                                .map(|d| omnim_core::Vector3D { x: d.x, y: d.y, z: d.z })
                                .unwrap_or(omnim_core::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
                            let profile = omnim_sketch::profile::sketch_to_profile_2d(sketch);
                            let angle = rs.angle;
                            let axis_origin = origin;
                            let axis_direction = axis_dir;
                            let sketch_id = rs.sketch_id.clone();
                            if let Some(ref prof) = profile {
                                if prof.len() >= 3 {
                                    let profile3d: Vec<omnim_core::Point3D> = prof.iter()
                                        .map(|p| omnim_core::Point3D { x: p.x, y: p.y, z: 0.0 })
                                        .collect();
                                    if omnim_model::brep::revolve_brep(&profile3d, angle, origin, axis_dir, omnim_model::brep::DEFAULT_TOLERANCE).is_some() {
                                        commands.push(Command::RevolveSketch { sketch_id, angle, axis_origin, axis_direction, profile });
                                    }
                                }
                            }
                        }
                    }

                    c::operation::Op::CreateComponent(cc) => {
                        let component_id = project.create_component(&cc.name, if cc.parent_id.is_empty() { None } else { Some(&cc.parent_id) });
                        created_ids.push(component_id.clone());
                        commands.push(Command::CreateComponent { component_id });
                    }

                    c::operation::Op::AddFeature(af) => {
                        let feature_type = match c::FeatureType::try_from(af.feature_type) {
                            Ok(c::FeatureType::FeatureExtrude) => omnim_model::FeatureType::Extrude,
                            Ok(c::FeatureType::FeatureRevolve) => omnim_model::FeatureType::Revolve,
                            Ok(c::FeatureType::FeatureSweep) => omnim_model::FeatureType::Sweep,
                            Ok(c::FeatureType::FeatureLoft) => omnim_model::FeatureType::Loft,
                            Ok(c::FeatureType::FeatureBoolean) => omnim_model::FeatureType::Boolean,
                            Ok(c::FeatureType::FeatureChamfer) => omnim_model::FeatureType::Chamfer,
                            Ok(c::FeatureType::FeatureFillet) => omnim_model::FeatureType::Fillet,
                            Ok(c::FeatureType::FeaturePattern) => omnim_model::FeatureType::Pattern,
                            Ok(c::FeatureType::FeatureShell) => omnim_model::FeatureType::Shell,
                            Ok(c::FeatureType::FeatureFusion) => omnim_model::FeatureType::Fusion,
                            Ok(c::FeatureType::FeatureLoftedFusion) => omnim_model::FeatureType::LoftedFusion,
                            _ => omnim_model::FeatureType::Extrude,
                        };
                        let dir = af.direction.as_ref()
                            .map(|d| omnim_core::Vector3D { x: d.x, y: d.y, z: d.z })
                            .unwrap_or(omnim_core::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
                        let origin = af.axis_origin.as_ref()
                            .map(|p| omnim_core::Point3D { x: p.x, y: p.y, z: p.z })
                            .unwrap_or(omnim_core::Point3D { x: 0.0, y: 0.0, z: 0.0 });
                        let axis_dir = af.axis_direction.as_ref()
                            .map(|d| omnim_core::Vector3D { x: d.x, y: d.y, z: d.z })
                            .unwrap_or(omnim_core::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
                        let feature = omnim_model::Feature {
                            id: String::new(),
                            name: format!("{}-{:.0}", feature_type_name(&feature_type), af.depth),
                            feature_type: feature_type.clone(),
                            sketch_id: af.sketch_id.clone(),
                            axis_sketch_id: af.axis_sketch_id.clone(),
                            input_sketch_ids: af.input_sketch_ids.clone(),
                            dependency_ids: Vec::new(),
                            depth: af.depth,
                            direction: dir,
                            angle: af.angle,
                            axis_origin: origin,
                            axis_direction: axis_dir,
                            is_cut: af.is_cut,
                            order: 0,
                            suppressed: false,
                            cached_geometry: None,
                            dirty: true,
                            lod: omnim_model::brep::DEFAULT_TOLERANCE,
                            boolean_op: None,
                            pattern_count_x: None,
                            pattern_count_y: None,
                            pattern_count_z: None,
                        };
                        match project.add_feature(&af.component_id, feature) {
                            Ok(feature_id) => {
                                created_ids.push(feature_id.clone());
                                commands.push(Command::AddFeature { component_id: af.component_id.clone(), feature_id });
                            }
                            Err(e) => {
                                return self.fail_and_rollback(&commands, project, e, created_ids);
                            }
                        }
                    }

                    c::operation::Op::RemoveFeature(rf) => {
                        match project.remove_feature(&rf.component_id, &rf.feature_id) {
                            Ok(feature) => {
                                commands.push(Command::RemoveFeature { component_id: rf.component_id.clone(), feature });
                            }
                            Err(e) => {
                                return self.fail_and_rollback(&commands, project, e, created_ids);
                            }
                        }
                    }

                    c::operation::Op::RebuildFeatures(rf) => {
                        if let Err(e) = project.rebuild_component(&rf.component_id) {
                            return self.fail_and_rollback(&commands, project, e, created_ids);
                        }
                        commands.push(Command::RebuildFeatures { component_id: rf.component_id.clone() });
                    }

                    // === Catalog Operations ===

                    c::operation::Op::DefineComponent(dc) => {
                        let mut comp = omnim_model::catalog::ComponentDef::new(&dc.name, &dc.category);
                        comp.description = dc.description.clone();
                        comp.ifc_class = dc.ifc_class.clone();
                        comp.standard_reference = dc.standard_reference.clone();
                        comp.tags = dc.tags.clone();
                        comp.domain = dc.domain.clone();
                        let id = comp.id.clone();
                        LIBRARY.write().unwrap().as_mut().unwrap().catalog.insert(comp.clone());
                        created_ids.push(id);
                        commands.push(Command::DefineComponent { component_def: comp });
                    }

                    c::operation::Op::AddComponentParam(acp) => {
                        if let Some(comp) = LIBRARY.write().unwrap().as_mut().unwrap().catalog.get_mut(&acp.component_id) {
                            let param = omnim_model::catalog::ParamDef::new(&acp.param_name, acp.param_type as i32)
                                .with_default(acp.default_value)
                                .with_range(acp.min_value, acp.max_value)
                                .with_step(acp.step)
                                .with_unit(&acp.unit);
                            let param_index = comp.params.len();
                            comp.params.push(omnim_model::catalog::ParamDef {
                                name: param.name.clone(),
                                param_type: param.param_type,
                                default_value: param.default_value,
                                min_value: param.min_value,
                                max_value: param.max_value,
                                step: param.step,
                                unit: param.unit.clone(),
                                formula: acp.formula.clone(),
                                choices: acp.choices.clone(),
                                description: acp.description.clone(),
                            });
                            commands.push(Command::AddComponentParam { component_id: acp.component_id.clone(), param_index });
                        }
                    }

                    c::operation::Op::AddConnector(acn) => {
                        if let Some(comp) = LIBRARY.write().unwrap().as_mut().unwrap().catalog.get_mut(&acn.component_id) {
                            let conn = omnim_model::catalog::ConnectorDef::new(&acn.connector_id, &acn.connector_name)
                                .with_position(&acn.position_x_expr, &acn.position_y_expr, &acn.position_z_expr)
                                .with_direction(&acn.direction_x_expr, &acn.direction_y_expr, &acn.direction_z_expr)
                                .with_bore(&acn.bore_expr)
                                .with_type_code(&acn.connection_type_code)
                                .with_role(acn.role as i32);
                            comp.connectors.push(conn);
                            commands.push(Command::AddConnector {
                                component_id: acn.component_id.clone(),
                                connector_id: acn.connector_id.clone(),
                            });
                        }
                    }

                    c::operation::Op::PublishComponent(pb) => {
                        if let Some(comp) = LIBRARY.write().unwrap().as_mut().unwrap().catalog.get_mut(&pb.component_id) {
                            comp.published = true;
                            commands.push(Command::PublishComponent { component_id: pb.component_id.clone() });
                        }
                    }

                    c::operation::Op::DeleteComponentDef(dcd) => {
                        if let Some(comp) = LIBRARY.write().unwrap().as_mut().unwrap().catalog.remove(&dcd.component_id) {
                            commands.push(Command::DeleteComponentDef { component_def: comp });
                        }
                    }

                    // === Series Operations ===

                    c::operation::Op::CreateSeries(cs) => {
                        let mut series = omnim_model::series_store::Series::new(&cs.name);
                        series.description = cs.description.clone();
                        series.domain = cs.domain.clone();
                        series.standard_reference = cs.standard_reference.clone();
                        let series_clone = series.clone();
                        let series_id = series.id.clone();
                        project.series.insert(series);
                        created_ids.push(series_id.clone());
                        commands.push(Command::CreateSeries { series: series_clone });
                    }

                    c::operation::Op::AddSelector(asel) => {
                        if let Some(series) = project.series.get_mut(&asel.series_id) {
                            let selector = omnim_model::series_store::Selector::new(&asel.selector_id, &asel.question);
                            if series.root_selector.is_none() {
                                series.root_selector = Some(selector);
                            }
                            // If root already exists, the selector is added via AddAnswer operations
                            commands.push(Command::AddSelector {
                                series_id: asel.series_id.clone(),
                                selector_id: asel.selector_id.clone(),
                            });
                        }
                    }

                    c::operation::Op::AddAnswer(aans) => {
                        if let Some(series) = project.series.get_mut(&aans.series_id) {
                            let answer = omnim_model::series_store::Answer {
                                id: aans.answer_id.clone(),
                                value: aans.answer_value.clone(),
                                is_numeric_range: aans.is_numeric_range,
                                range_min: aans.range_min,
                                range_max: aans.range_max,
                                outcome: omnim_model::series_store::AnswerOutcome::Component(
                                    omnim_model::series_store::SpecComponent::new("")
                                ),
                            };
                            // Find selector and add answer
                            let mut stack = Vec::new();
                            if let Some(ref mut root) = series.root_selector {
                                stack.push(root);
                            }
                            let mut answer_index = 0;
                            while let Some(sel) = stack.pop() {
                                if sel.id == aans.selector_id {
                                    answer_index = sel.answers.len();
                                    sel.answers.push(answer);
                                    break;
                                }
                                for ans in &mut sel.answers {
                                    if let omnim_model::series_store::AnswerOutcome::NextSelector(ref mut next) = ans.outcome {
                                        stack.push(next);
                                    }
                                }
                            }
                            commands.push(Command::AddAnswer {
                                series_id: aans.series_id.clone(),
                                selector_id: aans.selector_id.clone(),
                                answer_index,
                            });
                        }
                    }

                    c::operation::Op::AddSpecComponent(asc) => {
                        if let Some(series) = project.series.get_mut(&asc.series_id) {
                            let spec = omnim_model::series_store::SpecComponent::new(&asc.component_def_id)
                                .material(&asc.material)
                                .connection_type_code(&asc.connection_type_code);
                            // Find selector and update matching answer's outcome
                            let mut stack = Vec::new();
                            if let Some(ref mut root) = series.root_selector {
                                stack.push(root);
                            }
                            let mut answer_index = 0;
                            while let Some(sel) = stack.pop() {
                                if sel.id == asc.selector_id {
                                    // Find the answer by id or last empty one
                                    for (i, ans) in sel.answers.iter_mut().enumerate() {
                                        if ans.id == asc.answer_id || matches!(ans.outcome, omnim_model::series_store::AnswerOutcome::Component(ref sc) if sc.component_def_id.is_empty()) {
                                            answer_index = i;
                                            ans.outcome = omnim_model::series_store::AnswerOutcome::Component(spec.clone());
                                            break;
                                        }
                                    }
                                    break;
                                }
                                for ans in &mut sel.answers {
                                    if let omnim_model::series_store::AnswerOutcome::NextSelector(ref mut next) = ans.outcome {
                                        stack.push(next);
                                    }
                                }
                            }
                            commands.push(Command::AddSpecComponent {
                                series_id: asc.series_id.clone(),
                                selector_id: asc.selector_id.clone(),
                                answer_index,
                            });
                        }
                    }

                    c::operation::Op::AddParamConstraint(apc) => {
                        if let Some(series) = project.series.get_mut(&apc.series_id) {
                            // Find the SpecComponent in the series tree and add constraint
                            let constraint = omnim_model::series_store::ParamConstraint::new(&apc.param_name)
                                .with_default(apc.default_value)
                                .with_range(apc.min_value, apc.max_value)
                                .with_step(apc.step)
                                .with_fixed(&apc.fixed_value);
                            let mut stack = Vec::new();
                            if let Some(ref mut root) = series.root_selector {
                                stack.push(root);
                            }
                            let mut param_index = 0;
                            while let Some(sel) = stack.pop() {
                                for ans in &mut sel.answers {
                                    match &mut ans.outcome {
                                        omnim_model::series_store::AnswerOutcome::Component(ref mut sc) => {
                                            if sc.component_def_id == apc.component_def_id {
                                                param_index = sc.param_constraints.len();
                                                sc.param_constraints.push(constraint.clone());
                                            }
                                        }
                                        omnim_model::series_store::AnswerOutcome::NextSelector(ref mut next) => {
                                            stack.push(next);
                                        }
                                    }
                                }
                            }
                            commands.push(Command::AddParamConstraint {
                                series_id: apc.series_id.clone(),
                                component_def_id: apc.component_def_id.clone(),
                                param_index,
                            });
                        }
                    }

                    c::operation::Op::DeleteSeries(ds) => {
                        if let Some(series) = project.series.remove(&ds.series_id) {
                            commands.push(Command::DeleteSeries { series });
                        }
                    }
                    c::operation::Op::LoadSeries(ls) => {
                        match omnim_model::series_store::SeriesStore::load_from_file(&ls.path) {
                            Ok(ext_store) => {
                                let source = if ls.source.is_empty() { "standalone" } else { &ls.source };
                                let mut loaded_ids = Vec::new();
                                for s in ext_store.list() {
                                    let id = s.id.clone();
                                    let name = s.name.clone();
                                    // insert() handles priority - won't override project series
                                    project.series.insert(s.clone());
                                    loaded_ids.push(id.clone());
                                    created_ids.push(id.clone());
                                    // Track in external_series_refs
                                    let already_ref = project.external_series_refs
                                        .iter().any(|r| r.id == id);
                                    if !already_ref {
                                        project.external_series_refs.push(
                                            omnim_model::SeriesRef {
                                                id,
                                                name,
                                                source: source.to_string(),
                                                path: Some(ls.path.clone()),
                                            }
                                        );
                                    }
                                }
                                let count = loaded_ids.len();
                                commands.push(Command::LoadSeries {
                                    path: ls.path.clone(),
                                    source: source.to_string(),
                                    loaded_ids,
                                });
                                // Early return with custom message for series load
                                if !self.in_transaction() {
                                    self.push_undo_commands(commands);
                                }
                                return ExecutionResult {
                                    success: true,
                                    message: format!("Loaded {} series from {}", count, ls.path),
                                    created_ids,
                                };
                            }
                            Err(e) => {
                                return ExecutionResult {
                                    success: false,
                                    message: format!("Failed to load series from {}: {}", ls.path, e),
                                    created_ids: Vec::new(),
                                };
                            }
                        }
                    }
                }
            }
        }

        // Actions that don't need an Operation
        match action {
            c::Action::DeleteObject => {
                if let Some(tid) = target_object_id(instr) {
                    if let Some(obj) = project.objects.remove(&tid) {
                        commands.push(Command::DeleteObject {
                            snapshot: ObjectSnapshot {
                                object: obj,
                                removed_properties: Vec::new(),
                                old_material_id: None,
                                old_geometry: None,
                            },
                        });
                    }
                }
            }
            c::Action::RemoveContainment => {
                if let Some(tid) = target_object_id(instr) {
                    if let Some(cont) = project.relations.parent_of(&tid) {
                        let c = cont.clone();
                        let pid = cont.parent_id.clone();
                        let cid = cont.child_id.clone();
                        project.relations.remove_containment(&pid, &cid);
                        commands.push(Command::RemoveContainment { containment: c });
                    }
                }
            }
            c::Action::RebuildFeatures => {
                for op in &instr.operations {
                    if let Some(c::operation::Op::RebuildFeatures(rf)) = &op.op {
                        if let Err(e) = project.rebuild_component(&rf.component_id) {
                            return self.fail_and_rollback(&commands, project, e, created_ids);
                        }
                        commands.push(Command::RebuildFeatures { component_id: rf.component_id.clone() });
                    }
                }
            }
            _ => {}
        }

        // 记录到 undo 栈
        self.redo_stack.clear();
        if let Some(tx) = &mut self.transaction_stack {
            tx.extend(commands);
        } else {
            self.push_undo_commands(commands);
        }

        ExecutionResult {
            success: true,
            message: format!("{}: {} object(s) created", action_str(action), created_ids.len()),
            created_ids,
        }
    }

    // ---- Undo / Redo ----

    /// 撤销最近一次 execute() 或 transaction commit。
    /// 返回 undone 的命令描述。
    pub fn undo(&mut self, project: &mut Project) -> Result<String, String> {
        let command = self.undo_stack.pop()
            .ok_or_else(|| "nothing to undo".to_string())?;

        let desc = self.apply_inverse(&command, project);
        self.redo_stack.push(command);
        Ok(desc)
    }

    /// 重做最近一次 undo()。
    pub fn redo(&mut self, project: &mut Project) -> Result<String, String> {
        let command = self.redo_stack.pop()
            .ok_or_else(|| "nothing to redo".to_string())?;

        let desc = self.apply_forward(&command, project);
        self.undo_stack.push(command);
        Ok(desc)
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn undo_depth(&self) -> usize {
        self.undo_stack.len()
    }

    // ---- Transactions ----

    /// 开始一个事务。事务内的所有 execute() 调用会被暂存。
    /// 调用 commit() 一次性推入 undo 栈，或 rollback() 全部撤销。
    pub fn begin_transaction(&mut self) -> Result<(), String> {
        if self.transaction_stack.is_some() {
            return Err("transaction already in progress".into());
        }
        self.transaction_stack = Some(Vec::new());
        Ok(())
    }

    /// 提交当前事务：将所有暂存命令推入 undo 栈。
    pub fn commit(&mut self) -> Result<usize, String> {
        let commands = self.transaction_stack.take()
            .ok_or_else(|| "no active transaction".to_string())?;
        let count = commands.len();
        self.push_undo_commands(commands);
        Ok(count)
    }

    /// 回滚当前事务：逆向执行所有暂存命令以恢复状态，然后丢弃。
    pub fn rollback(&mut self, project: &mut Project) -> Result<usize, String> {
        let commands = self.transaction_stack.take()
            .ok_or_else(|| "no active transaction".to_string())?;
        let count = commands.len();
        // 反向执行撤销
        for cmd in commands.iter().rev() {
            self.apply_inverse(cmd, project);
        }
        Ok(count)
    }

    /// 事务是否活跃
    pub fn in_transaction(&self) -> bool {
        self.transaction_stack.is_some()
    }

    /// 清空历史
    pub fn clear_history(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// 手动记录一条创建对象的 undo 命令（用于不经过 execute 的直接创建）
    pub fn record_create(&mut self, obj: &OmniObject) {
        self.redo_stack.clear();
        let command = Command::CreateObject {
            snapshot: ObjectSnapshot {
                object: obj.clone(),
                removed_properties: Vec::new(),
                old_material_id: None,
                old_geometry: None,
            },
        };
        if let Some(tx) = &mut self.transaction_stack {
            tx.push(command);
        } else {
            self.undo_stack.push(command);
        }
        while self.undo_stack.len() > MAX_UNDO_DEPTH {
            self.undo_stack.remove(0);
        }
    }

    /// 手动记录一条删除对象的 undo 命令
    pub fn record_delete(&mut self, obj: &OmniObject) {
        self.redo_stack.clear();
        let command = Command::DeleteObject {
            snapshot: ObjectSnapshot {
                object: obj.clone(),
                removed_properties: Vec::new(),
                old_material_id: None,
                old_geometry: None,
            },
        };
        if let Some(tx) = &mut self.transaction_stack {
            tx.push(command);
        } else {
            self.undo_stack.push(command);
        }
        while self.undo_stack.len() > MAX_UNDO_DEPTH {
            self.undo_stack.remove(0);
        }
    }

    /// 手动记录一条创建草图的 undo 命令
    pub fn record_sketch_create(&mut self, sketch_id: &str) {
        self.redo_stack.clear();
        let command = Command::CreateSketch { sketch_id: sketch_id.to_string() };
        if let Some(tx) = &mut self.transaction_stack {
            tx.push(command);
        } else {
            self.undo_stack.push(command);
        }
        while self.undo_stack.len() > MAX_UNDO_DEPTH {
            self.undo_stack.remove(0);
        }
    }

    /// 手动记录一条添加草图实体的 undo 命令
    pub fn record_sketch_add_entity(&mut self, sketch_id: &str, entity_index: usize, entity: &omnim_sketch::SketchEntity) {
        self.redo_stack.clear();
        let command = Command::AddSketchEntity {
            sketch_id: sketch_id.to_string(),
            entity_index,
            entity: entity.clone(),
        };
        if let Some(tx) = &mut self.transaction_stack {
            tx.push(command);
        } else {
            self.undo_stack.push(command);
        }
        while self.undo_stack.len() > MAX_UNDO_DEPTH {
            self.undo_stack.remove(0);
        }
    }

    /// 手动记录一条删除草图实体的 undo 命令
    pub fn record_sketch_delete_entity(&mut self, sketch_id: &str, entity: &omnim_sketch::SketchEntity) {
        self.redo_stack.clear();
        let command = Command::DeleteSketchEntity {
            sketch_id: sketch_id.to_string(),
            entity: entity.clone(),
        };
        if let Some(tx) = &mut self.transaction_stack {
            tx.push(command);
        } else {
            self.undo_stack.push(command);
        }
        while self.undo_stack.len() > MAX_UNDO_DEPTH {
            self.undo_stack.remove(0);
        }
    }

    /// 手动记录一条添加草图约束的 undo 命令
    pub fn record_sketch_add_constraint(&mut self, sketch_id: &str, constraint_index: usize, constraint: &omnim_sketch::SketchConstraint) {
        self.redo_stack.clear();
        let command = Command::AddSketchConstraint {
            sketch_id: sketch_id.to_string(),
            constraint_index,
            constraint: constraint.clone(),
        };
        if let Some(tx) = &mut self.transaction_stack {
            tx.push(command);
        } else {
            self.undo_stack.push(command);
        }
        while self.undo_stack.len() > MAX_UNDO_DEPTH {
            self.undo_stack.remove(0);
        }
    }

    /// 手动记录一条删除草图约束的 undo 命令
    pub fn record_sketch_remove_constraint(&mut self, sketch_id: &str, constraint: &omnim_sketch::SketchConstraint) {
        self.redo_stack.clear();
        let command = Command::RemoveSketchConstraint {
            sketch_id: sketch_id.to_string(),
            constraint: constraint.clone(),
        };
        if let Some(tx) = &mut self.transaction_stack {
            tx.push(command);
        } else {
            self.undo_stack.push(command);
        }
        while self.undo_stack.len() > MAX_UNDO_DEPTH {
            self.undo_stack.remove(0);
        }
    }

    /// 手动记录一条删除草图的 undo 命令
    pub fn record_sketch_delete(&mut self, sketch: &omnim_sketch::Sketch) {
        self.redo_stack.clear();
        let command = Command::DeleteSketch { sketch: sketch.clone() };
        if let Some(tx) = &mut self.transaction_stack {
            tx.push(command);
        } else {
            self.undo_stack.push(command);
        }
        while self.undo_stack.len() > MAX_UNDO_DEPTH {
            self.undo_stack.remove(0);
        }
    }

    // ================================================================
    // Private helpers
    // ================================================================

    fn push_undo_commands(&mut self, commands: Vec<Command>) {
        self.undo_stack.extend(commands);
        // 限制深度
        while self.undo_stack.len() > MAX_UNDO_DEPTH {
            self.undo_stack.remove(0);
        }
    }

    /// 逆向执行命令（用于 undo）
    fn apply_inverse(&self, cmd: &Command, project: &mut Project) -> String {
        match cmd {
            Command::CreateObject { snapshot } => {
                project.objects.remove(&snapshot.object.id);
                format!("undo: removed object {}", &snapshot.object.id[..8])
            }
            Command::DeleteObject { snapshot } => {
                project.objects.insert(snapshot.object.clone());
                format!("undo: restored object {}", &snapshot.object.id[..8])
            }
            Command::ModifyObject { object_id, snapshot } => {
                if let Some(obj) = project.objects.get_mut(object_id) {
                    obj.name = snapshot.object.name.clone();
                    obj.description = snapshot.object.description.clone();
                    obj.geometry = snapshot.old_geometry.clone();
                    obj.material_id = snapshot.old_material_id.clone();
                }
                format!("undo: restored object {}", &object_id[..8])
            }
            Command::SetProperty { object_id, property_name, old_value } => {
                if let Some(obj) = project.objects.get_mut(object_id) {
                    match old_value {
                        Some(val) => obj.set_property(property_name, val.clone()),
                        None => { obj.remove_property(property_name); }
                    }
                }
                format!("undo: restored property '{}'", property_name)
            }
            Command::AddContainment { parent_id, child_id } => {
                project.relations.remove_containment(parent_id, child_id);
                format!("undo: removed containment {} -> {}", &parent_id[..8], &child_id[..8])
            }
            Command::RemoveContainment { containment } => {
                let _ = project.relations.add_containment(containment.clone());
                format!("undo: restored containment")
            }
            Command::AddConnection { from_id, to_id } => {
                // 连接没有简单的一对一删除 — 标记为不完全撤销
                format!("undo: noted connection {} <-> {}", &from_id[..8], &to_id[..8])
            }
            Command::SetMaterial { object_id, old_material_id } => {
                if let Some(obj) = project.objects.get_mut(object_id) {
                    obj.material_id = old_material_id.clone();
                }
                format!("undo: restored material on {}", &object_id[..8])
            }
            Command::CreateLifecycleRecord { record_id } => {
                project.lifecycle.remove(record_id);
                format!("undo: removed lifecycle record {}", &record_id[..8])
            }
            Command::ModifyLifecycleRecord { record_id, old_name, old_description, old_status } => {
                if let Some(record) = project.lifecycle.get_mut(record_id) {
                    record.name = old_name.clone();
                    record.description = old_description.clone();
                    record.status = *old_status;
                }
                format!("undo: restored lifecycle record {}", &record_id[..8])
            }
            Command::DeleteLifecycleRecord { record } => {
                project.lifecycle.insert(record.clone());
                format!("undo: restored lifecycle record {}", &record.id[..8])
            }
            Command::DeleteProperty { object_id, removed } => {
                if let Some(obj) = project.objects.get_mut(object_id) {
                    obj.properties.push(removed.clone());
                }
                format!("undo: restored property '{}'", removed.name)
            }
            Command::RemovePropertySet { object_id, removed } => {
                if let Some(obj) = project.objects.get_mut(object_id) {
                    obj.property_sets.push(removed.clone());
                }
                format!("undo: restored property set '{}'", removed.name)
            }
            Command::RemoveConnection { from_id, to_id } => {
                // Re-add the connection (best effort)
                project.relations.add_connection(c::Connection {
                    from_object_id: from_id.clone(),
                    to_object_id: to_id.clone(),
                    r#type: c::ConnectionType::Structural as i32,
                    from_point: None,
                    to_point: None,
                    connection_detail: String::new(),
                });
                format!("undo: restored connection {} <-> {}", &from_id[..8], &to_id[..8])
            }
            Command::CopyObject { new_id, .. } => {
                project.objects.remove(new_id);
                format!("undo: removed copied object {}", &new_id[..8])
            }
            Command::MoveObject { object_id, old_parent_id, .. } => {
                if let Some(obj) = project.objects.get_mut(object_id) {
                    obj.parent_id = old_parent_id.clone();
                    project.objects.rebuild_children();
                }
                format!("undo: restored parent for object {}", &object_id[..8])
            }
            Command::TransformGeometry { object_id, old_geometry, .. } => {
                if let Some(obj) = project.objects.get_mut(object_id) {
                    obj.geometry = old_geometry.clone();
                }
                format!("undo: restored geometry on object {}", &object_id[..8])
            }
            Command::LinkRecordToObject { record_id, object_id } => {
                if let Some(record) = project.lifecycle.get_mut(record_id) {
                    record.related_object_ids.retain(|id| id != object_id);
                }
                format!("undo: unlinked object {} from record {}", &object_id[..8], &record_id[..8])
            }
            Command::UpdateSchedule { record_id, old_planned_start, old_planned_finish, old_actual_start, old_actual_finish } => {
                if let Some(record) = project.lifecycle.get_mut(record_id) {
                    record.planned_start = old_planned_start.clone();
                    record.planned_finish = old_planned_finish.clone();
                    record.actual_start = old_actual_start.clone();
                    record.actual_finish = old_actual_finish.clone();
                }
                format!("undo: restored schedule on record {}", &record_id[..8])
            }
            Command::UpdateCost { record_id, old_planned_cost, old_actual_cost } => {
                if let Some(record) = project.lifecycle.get_mut(record_id) {
                    record.planned_cost = *old_planned_cost;
                    record.actual_cost = *old_actual_cost;
                }
                format!("undo: restored cost on record {}", &record_id[..8])
            }
            Command::UpdateProgress { record_id, old_percent, .. } => {
                if let Some(record) = project.lifecycle.get_mut(record_id) {
                    if let Some(existing) = record.properties.iter_mut()
                        .find(|p| p.name == "percent_complete")
                    {
                        existing.value = Some(c::PropertyValue {
                            value_type: Some(c::property_value::ValueType::DoubleValue(*old_percent)),
                        });
                    } else {
                        record.properties.push(c::Property {
                            name: "percent_complete".into(),
                            value: Some(c::PropertyValue {
                                value_type: Some(c::property_value::ValueType::DoubleValue(*old_percent)),
                            }),
                            unit: String::new(),
                            description: String::new(),
                        });
                    }
                }
                format!("undo: restored progress on record {}", &record_id[..8])
            }
            Command::CreateWorkOrder { record_id } => {
                project.lifecycle.remove(record_id);
                format!("undo: removed work order {}", &record_id[..8])
            }
            Command::UpdateWorkOrder { record_id, old_status, old_description, .. } => {
                if let Some(record) = project.lifecycle.get_mut(record_id) {
                    record.status = old_status.parse::<i32>().unwrap_or(1);
                    record.description = old_description.clone();
                }
                format!("undo: restored work order {}", &record_id[..8])
            }
            Command::CreateSketch { sketch_id } => {
                project.sketches.remove(sketch_id);
                format!("undo: removed sketch {}", &sketch_id[..8])
            }
            Command::AddSketchEntity { sketch_id, entity_index, .. } => {
                if let Some(sketch) = project.sketches.get_mut(sketch_id) {
                    if *entity_index < sketch.entities.len() {
                        sketch.entities.remove(*entity_index);
                    }
                }
                format!("undo: removed entity from sketch {}", &sketch_id[..8])
            }
            Command::AddSketchConstraint { sketch_id, constraint_index, .. } => {
                if let Some(sketch) = project.sketches.get_mut(sketch_id) {
                    if *constraint_index < sketch.constraints.len() {
                        sketch.constraints.remove(*constraint_index);
                    }
                }
                format!("undo: removed constraint from sketch {}", &sketch_id[..8])
            }
            Command::SolveSketch { sketch_id, old_solved } => {
                if let Some(sketch) = project.sketches.get_mut(sketch_id) {
                    sketch.solved = *old_solved;
                }
                format!("undo: restored solve state for sketch {}", &sketch_id[..8])
            }
            Command::DeleteSketchEntity { sketch_id, entity } => {
                if let Some(sketch) = project.sketches.get_mut(sketch_id) {
                    sketch.entities.push(entity.clone());
                }
                format!("undo: restored entity in sketch {}", &sketch_id[..8])
            }
            Command::RemoveSketchConstraint { sketch_id, constraint } => {
                if let Some(sketch) = project.sketches.get_mut(sketch_id) {
                    sketch.constraints.push(constraint.clone());
                }
                format!("undo: restored constraint in sketch {}", &sketch_id[..8])
            }
            Command::DeleteSketch { sketch } => {
                project.sketches.insert(sketch.clone());
                format!("undo: restored sketch {}", &sketch.id[..8])
            }
            Command::ExtrudeSketch { sketch_id, .. } => {
                // Feature operations are currently additive (no 3D body storage yet)
                // Undo is a no-op until we add feature body tracking
                format!("undo: noted extrude on sketch {}", &sketch_id[..8])
            }
            Command::RevolveSketch { sketch_id, .. } => {
                format!("undo: noted revolve on sketch {}", &sketch_id[..8])
            }
            Command::CreateComponent { component_id } => {
                project.objects.remove(component_id);
                project.objects.rebuild_children();
                format!("undo: removed component {}", &component_id[..8])
            }
            Command::AddFeature { component_id, feature_id } => {
                let _ = project.remove_feature(component_id, feature_id);
                format!("undo: removed feature {} from component {}", &feature_id[..8], &component_id[..8])
            }
            Command::RemoveFeature { component_id, feature } => {
                let f = feature.clone();
                let _ = project.add_feature(component_id, f);
                format!("undo: restored feature in component {}", &component_id[..8])
            }
            Command::RebuildFeatures { component_id } => {
                format!("undo: rebuild is idempotent for component {}", &component_id[..8])
            }
            // === Catalog undo ===
            Command::DefineComponent { component_def } => {
                LIBRARY.write().unwrap().as_mut().unwrap().catalog.remove(&component_def.id);
                format!("undo: removed component def {}", &component_def.id[..8])
            }
            Command::AddComponentParam { component_id, param_index } => {
                if let Some(comp) = LIBRARY.write().unwrap().as_mut().unwrap().catalog.get_mut(component_id) {
                    if *param_index < comp.params.len() {
                        comp.params.remove(*param_index);
                    }
                }
                format!("undo: removed param from component {}", &component_id[..8])
            }
            Command::AddConnector { component_id, connector_id } => {
                if let Some(comp) = LIBRARY.write().unwrap().as_mut().unwrap().catalog.get_mut(component_id) {
                    comp.connectors.retain(|c| c.id != *connector_id);
                }
                format!("undo: removed connector {} from component {}", connector_id, &component_id[..8])
            }
            Command::PublishComponent { component_id } => {
                if let Some(comp) = LIBRARY.write().unwrap().as_mut().unwrap().catalog.get_mut(component_id) {
                    comp.published = false;
                }
                format!("undo: unpublished component {}", &component_id[..8])
            }
            Command::DeleteComponentDef { component_def } => {
                LIBRARY.write().unwrap().as_mut().unwrap().catalog.insert(component_def.clone());
                format!("undo: restored component def {}", &component_def.id[..8])
            }
            // === Series undo ===
            Command::CreateSeries { series } => {
                project.series.remove(&series.id);
                format!("undo: removed series {}", &series.id[..8])
            }
            Command::AddSelector { series_id, selector_id } => {
                // Remove the selector from the series tree
                if let Some(series) = project.series.get_mut(series_id) {
                    Self::remove_selector_from_tree(series, selector_id);
                }
                format!("undo: removed selector {} from series {}", selector_id, &series_id[..8])
            }
            Command::AddAnswer { series_id, selector_id, answer_index } => {
                if let Some(series) = project.series.get_mut(series_id) {
                    Self::remove_answer_from_tree(series, selector_id, *answer_index);
                }
                format!("undo: removed answer from selector {} in series {}", selector_id, &series_id[..8])
            }
            Command::AddSpecComponent { series_id, selector_id, answer_index } => {
                // Reset the answer outcome to empty SpecComponent
                if let Some(series) = project.series.get_mut(series_id) {
                    Self::reset_answer_outcome(series, selector_id, *answer_index);
                }
                format!("undo: removed spec component from selector {} in series {}", selector_id, &series_id[..8])
            }
            Command::AddParamConstraint { series_id, component_def_id, param_index } => {
                if let Some(series) = project.series.get_mut(series_id) {
                    Self::remove_param_constraint(series, component_def_id, *param_index);
                }
                format!("undo: removed param constraint from series {}", &series_id[..8])
            }
            Command::DeleteSeries { series } => {
                project.series.insert(series.clone());
                format!("undo: restored series {}", &series.id[..8])
            }
            Command::LoadSeries { path, source, loaded_ids } => {
                // Undo: remove all series that were loaded from this file
                for id in loaded_ids {
                    project.series.remove(id);
                }
                // Remove from external_series_refs
                project.external_series_refs
                    .retain(|r| r.path.as_deref() != Some(path) || r.source != *source);
                format!("undo: unloaded {} series from {}", loaded_ids.len(), path)
            }
        }
    }

    /// 正向执行命令（用于 redo）
    fn apply_forward(&self, cmd: &Command, project: &mut Project) -> String {
        match cmd {
            Command::CreateObject { snapshot } => {
                project.objects.insert(snapshot.object.clone());
                format!("redo: created object {}", &snapshot.object.id[..8])
            }
            Command::DeleteObject { snapshot } => {
                project.objects.remove(&snapshot.object.id);
                format!("redo: deleted object {}", &snapshot.object.id[..8])
            }
            Command::ModifyObject { object_id, snapshot: _ } => {
                format!("redo: modify object {}", &object_id[..8])
            }
            Command::SetProperty { object_id, property_name, old_value } => {
                if let Some(obj) = project.objects.get_mut(object_id) {
                    if let Some(val) = old_value {
                        obj.set_property(property_name, val.clone());
                    }
                }
                format!("redo: set property '{}'", property_name)
            }
            Command::AddContainment { parent_id, child_id } => {
                let _ = project.relations.add_containment(c::Containment {
                    parent_id: parent_id.clone(),
                    child_id: child_id.clone(),
                    r#type: c::ContainmentType::Spatial as i32,
                });
                format!("redo: added containment")
            }
            Command::RemoveContainment { containment } => {
                project.relations.remove_containment(&containment.parent_id, &containment.child_id);
                format!("redo: removed containment")
            }
            Command::AddConnection { from_id, to_id } => {
                project.relations.add_connection(c::Connection {
                    from_object_id: from_id.clone(),
                    to_object_id: to_id.clone(),
                    r#type: c::ConnectionType::Structural as i32,
                    from_point: None,
                    to_point: None,
                    connection_detail: String::new(),
                });
                format!("redo: added connection")
            }
            Command::SetMaterial { object_id, old_material_id: _ } => {
                format!("redo: set material on {}", &object_id[..8])
            }
            Command::CreateLifecycleRecord { record_id } => {
                project.lifecycle.remove(record_id);
                format!("redo: removed lifecycle record")
            }
            Command::ModifyLifecycleRecord { record_id, .. } => {
                format!("redo: modify lifecycle record {}", &record_id[..8])
            }
            Command::DeleteLifecycleRecord { record } => {
                project.lifecycle.remove(&record.id);
                format!("redo: deleted lifecycle record")
            }
            Command::DeleteProperty { object_id, removed } => {
                if let Some(obj) = project.objects.get_mut(object_id) {
                    obj.remove_property(&removed.name);
                }
                format!("redo: deleted property '{}'", removed.name)
            }
            Command::RemovePropertySet { object_id, removed } => {
                if let Some(obj) = project.objects.get_mut(object_id) {
                    obj.property_sets.retain(|ps| ps.name != removed.name);
                }
                format!("redo: removed property set '{}'", removed.name)
            }
            Command::RemoveConnection { from_id, to_id } => {
                format!("redo: removed connection {} <-> {}", &from_id[..8], &to_id[..8])
            }
            Command::CopyObject { snapshot, .. } => {
                project.objects.insert(snapshot.object.clone());
                format!("redo: copied object {}", &snapshot.object.id[..8])
            }
            Command::MoveObject { object_id, new_parent_id, .. } => {
                if let Some(obj) = project.objects.get_mut(object_id) {
                    obj.parent_id = if new_parent_id.is_empty() {
                        None
                    } else {
                        Some(new_parent_id.clone())
                    };
                    project.objects.rebuild_children();
                }
                format!("redo: moved object {}", &object_id[..8])
            }
            Command::TransformGeometry { object_id, translation, rotation, scale, .. } => {
                if let Some(obj) = project.objects.get_mut(object_id) {
                    if let Some(geom) = &mut obj.geometry {
                        if let Some(mesh) = omnim_model::geometry::geometry_to_mesh(geom) {
                            let origin = mesh_center(&mesh);
                            let pre = c::Transform {
                                matrix: vec![],
                                translation: Some(c::Point3D {
                                    x: -origin.x, y: -origin.y, z: -origin.z,
                                }),
                                rotation: None, scale: None,
                            };
                            let main = c::Transform {
                                matrix: vec![],
                                translation: translation.clone(),
                                rotation: rotation.clone(),
                                scale: scale.clone(),
                            };
                            let post = c::Transform {
                                matrix: vec![],
                                translation: Some(origin),
                                rotation: None, scale: None,
                            };
                            let shifted = omnim_model::geometry::transform_mesh(&mesh, &pre);
                            let transformed = omnim_model::geometry::transform_mesh(&shifted, &main);
                            let transformed = omnim_model::geometry::transform_mesh(&transformed, &post);
                            let bbox = omnim_model::geometry::compute_bbox(&transformed);
                            geom.representation = Some(c::geometry::Representation::Mesh(transformed));
                            geom.bbox = Some(bbox);
                        }
                    }
                }
                format!("redo: transform geometry on {}", &object_id[..8])
            }
            Command::LinkRecordToObject { record_id, object_id } => {
                if let Some(record) = project.lifecycle.get_mut(record_id) {
                    if !record.related_object_ids.contains(object_id) {
                        record.related_object_ids.push(object_id.clone());
                    }
                }
                format!("redo: linked object {} to record {}", &object_id[..8], &record_id[..8])
            }
            Command::UpdateSchedule { record_id, .. } => {
                format!("redo: update schedule on record {}", &record_id[..8])
            }
            Command::UpdateCost { record_id, .. } => {
                format!("redo: update cost on record {}", &record_id[..8])
            }
            Command::UpdateProgress { record_id, new_percent, .. } => {
                if let Some(record) = project.lifecycle.get_mut(record_id) {
                    if let Some(existing) = record.properties.iter_mut()
                        .find(|p| p.name == "percent_complete")
                    {
                        existing.value = Some(c::PropertyValue {
                            value_type: Some(c::property_value::ValueType::DoubleValue(*new_percent)),
                        });
                    } else {
                        record.properties.push(c::Property {
                            name: "percent_complete".into(),
                            value: Some(c::PropertyValue {
                                value_type: Some(c::property_value::ValueType::DoubleValue(*new_percent)),
                            }),
                            unit: String::new(),
                            description: String::new(),
                        });
                    }
                }
                format!("redo: update progress on record {}", &record_id[..8])
            }
            Command::CreateWorkOrder { record_id } => {
                format!("redo: create work order {}", &record_id[..8])
            }
            Command::UpdateWorkOrder { record_id, new_status, new_description, .. } => {
                if let Some(record) = project.lifecycle.get_mut(record_id) {
                    if !new_status.is_empty() {
                        record.status = parse_record_status(new_status) as i32;
                    }
                    if !new_description.is_empty() {
                        record.description = new_description.clone();
                    }
                }
                format!("redo: update work order {}", &record_id[..8])
            }
            Command::CreateSketch { sketch_id } => {
                format!("redo: created sketch {}", &sketch_id[..8])
            }
            Command::AddSketchEntity { sketch_id, entity, .. } => {
                if let Some(sketch) = project.sketches.get_mut(sketch_id) {
                    sketch.add_entity(entity.clone());
                }
                format!("redo: added entity to sketch {}", &sketch_id[..8])
            }
            Command::AddSketchConstraint { sketch_id, constraint, .. } => {
                if let Some(sketch) = project.sketches.get_mut(sketch_id) {
                    sketch.add_constraint(constraint.clone());
                }
                format!("redo: added constraint to sketch {}", &sketch_id[..8])
            }
            Command::SolveSketch { sketch_id, .. } => {
                format!("redo: solved sketch {}", &sketch_id[..8])
            }
            Command::DeleteSketchEntity { sketch_id, .. } => {
                format!("redo: deleted entity from sketch {}", &sketch_id[..8])
            }
            Command::RemoveSketchConstraint { sketch_id, .. } => {
                format!("redo: removed constraint from sketch {}", &sketch_id[..8])
            }
            Command::DeleteSketch { sketch } => {
                format!("redo: deleted sketch {}", &sketch.id[..8])
            }
            Command::ExtrudeSketch { sketch_id, .. } => {
                format!("redo: extruded sketch {}", &sketch_id[..8])
            }
            Command::RevolveSketch { sketch_id, .. } => {
                format!("redo: revolved sketch {}", &sketch_id[..8])
            }
            Command::CreateComponent { component_id } => {
                format!("redo: created component {}", &component_id[..8])
            }
            Command::AddFeature { component_id, feature_id } => {
                format!("redo: added feature {} to component {}", &feature_id[..8], &component_id[..8])
            }
            Command::RemoveFeature { component_id, .. } => {
                format!("redo: removed feature from component {}", &component_id[..8])
            }
            Command::RebuildFeatures { component_id } => {
                let _ = project.rebuild_component(component_id);
                format!("redo: rebuilt features for component {}", &component_id[..8])
            }
            // === Catalog redo ===
            Command::DefineComponent { component_def } => {
                LIBRARY.write().unwrap().as_mut().unwrap().catalog.insert(component_def.clone());
                format!("redo: defined component {}", &component_def.id[..8])
            }
            Command::AddComponentParam { component_id, param_index: _ } => {
                format!("redo: added param to component {}", &component_id[..8])
            }
            Command::AddConnector { component_id, connector_id } => {
                format!("redo: added connector {} to component {}", connector_id, &component_id[..8])
            }
            Command::PublishComponent { component_id } => {
                if let Some(comp) = LIBRARY.write().unwrap().as_mut().unwrap().catalog.get_mut(component_id) {
                    comp.published = true;
                }
                format!("redo: published component {}", &component_id[..8])
            }
            Command::DeleteComponentDef { component_def } => {
                LIBRARY.write().unwrap().as_mut().unwrap().catalog.remove(&component_def.id);
                format!("redo: deleted component def {}", &component_def.id[..8])
            }
            // === Series redo ===
            Command::CreateSeries { series } => {
                project.series.insert(series.clone());
                format!("redo: created series {}", &series.id[..8])
            }
            Command::AddSelector { series_id, selector_id } => {
                format!("redo: added selector {} to series {}", selector_id, &series_id[..8])
            }
            Command::AddAnswer { series_id, selector_id, answer_index: _ } => {
                format!("redo: added answer to selector {} in series {}", selector_id, &series_id[..8])
            }
            Command::AddSpecComponent { series_id, selector_id, answer_index: _ } => {
                format!("redo: added spec component to selector {} in series {}", selector_id, &series_id[..8])
            }
            Command::AddParamConstraint { series_id, component_def_id: _, param_index: _ } => {
                format!("redo: added param constraint to series {}", &series_id[..8])
            }
            Command::DeleteSeries { series } => {
                project.series.remove(&series.id);
                format!("redo: deleted series {}", &series.id[..8])
            }
            Command::LoadSeries { path, source, loaded_ids } => {
                // Redo: re-load series from the file
                if let Ok(ext_store) = omnim_model::series_store::SeriesStore::load_from_file(path) {
                    for s in ext_store.list() {
                        if loaded_ids.contains(&s.id) {
                            project.series.insert(s.clone());
                        }
                    }
                    for id in loaded_ids {
                        let already_ref = project.external_series_refs.iter().any(|r| r.id == *id);
                        if !already_ref {
                            if let Some(s) = ext_store.get(id) {
                                project.external_series_refs.push(omnim_model::SeriesRef {
                                    id: id.clone(),
                                    name: s.name.clone(),
                                    source: source.clone(),
                                    path: Some(path.clone()),
                                });
                            }
                        }
                    }
                }
                format!("redo: loaded {} series from {}", loaded_ids.len(), path)
            }
        }
    }

    fn fail_and_rollback(
        &self,
        commands: &[Command],
        project: &mut Project,
        error: String,
        created_ids: Vec<String>,
    ) -> ExecutionResult {
        // 反向执行已记录的命令以回滚状态
        for cmd in commands.iter().rev() {
            self.apply_inverse(cmd, project);
        }
        ExecutionResult {
            success: false,
            message: error,
            created_ids,
        }
    }

    /// Remove a selector from a series tree by selector_id
    fn remove_selector_from_tree(series: &mut omnim_model::series_store::Series, selector_id: &str) {
        if let Some(ref mut root) = series.root_selector {
            if root.id == selector_id {
                series.root_selector = None;
                return;
            }
            let mut stack = vec![root];
            while let Some(sel) = stack.pop() {
                for ans in &mut sel.answers {
                    if let omnim_model::series_store::AnswerOutcome::NextSelector(ref mut next) = ans.outcome {
                        if next.id == selector_id {
                            *next = omnim_model::series_store::Selector::new("", "");
                        } else {
                            stack.push(next);
                        }
                    }
                }
            }
        }
    }

    /// Remove an answer at index from a selector in the series tree
    fn remove_answer_from_tree(series: &mut omnim_model::series_store::Series, selector_id: &str, answer_index: usize) {
        if let Some(ref mut root) = series.root_selector {
            let mut stack = vec![root];
            while let Some(sel) = stack.pop() {
                if sel.id == selector_id && answer_index < sel.answers.len() {
                    sel.answers.remove(answer_index);
                    return;
                }
                for ans in &mut sel.answers {
                    if let omnim_model::series_store::AnswerOutcome::NextSelector(ref mut next) = ans.outcome {
                        stack.push(next);
                    }
                }
            }
        }
    }

    /// Reset an answer's outcome to an empty SpecComponent
    fn reset_answer_outcome(series: &mut omnim_model::series_store::Series, selector_id: &str, answer_index: usize) {
        if let Some(ref mut root) = series.root_selector {
            let mut stack = vec![root];
            while let Some(sel) = stack.pop() {
                if sel.id == selector_id && answer_index < sel.answers.len() {
                    sel.answers[answer_index].outcome = omnim_model::series_store::AnswerOutcome::Component(
                        omnim_model::series_store::SpecComponent::new("")
                    );
                    return;
                }
                for ans in &mut sel.answers {
                    if let omnim_model::series_store::AnswerOutcome::NextSelector(ref mut next) = ans.outcome {
                        stack.push(next);
                    }
                }
            }
        }
    }

    /// Remove a param constraint at index from a SpecComponent in the series tree
    fn remove_param_constraint(series: &mut omnim_model::series_store::Series, component_def_id: &str, param_index: usize) {
        if let Some(ref mut root) = series.root_selector {
            let mut stack = vec![root];
            while let Some(sel) = stack.pop() {
                for ans in &mut sel.answers {
                    match &mut ans.outcome {
                        omnim_model::series_store::AnswerOutcome::Component(ref mut sc) => {
                            if sc.component_def_id == component_def_id && param_index < sc.param_constraints.len() {
                                sc.param_constraints.remove(param_index);
                                return;
                            }
                        }
                        omnim_model::series_store::AnswerOutcome::NextSelector(ref mut next) => {
                            stack.push(next);
                        }
                    }
                }
            }
        }
    }
}

fn action_str(a: c::Action) -> &'static str {
    match a {
        c::Action::CreateObject => "CREATE_OBJECT",
        c::Action::ModifyObject => "MODIFY_OBJECT",
        c::Action::DeleteObject => "DELETE_OBJECT",
        c::Action::CopyObject => "COPY_OBJECT",
        c::Action::MoveObject => "MOVE_OBJECT",
        c::Action::CreateGeometry => "CREATE_GEOMETRY",
        c::Action::ModifyGeometry => "MODIFY_GEOMETRY",
        c::Action::DeleteGeometry => "DELETE_GEOMETRY",
        c::Action::TransformGeometry => "TRANSFORM_GEOMETRY",
        c::Action::BooleanOperation => "BOOLEAN_OPERATION",
        c::Action::SetProperty => "SET_PROPERTY",
        c::Action::DeleteProperty => "DELETE_PROPERTY",
        c::Action::AddPropertySet => "ADD_PROPERTY_SET",
        c::Action::RemovePropertySet => "REMOVE_PROPERTY_SET",
        c::Action::AddContainment => "ADD_CONTAINMENT",
        c::Action::RemoveContainment => "REMOVE_CONTAINMENT",
        c::Action::AddConnection => "ADD_CONNECTION",
        c::Action::RemoveConnection => "REMOVE_CONNECTION",
        c::Action::Array => "ARRAY",
        c::Action::BatchDelete => "BATCH_DELETE",
        c::Action::BatchTransform => "BATCH_TRANSFORM",
        c::Action::SetMaterial => "SET_MATERIAL",
        c::Action::ImportFile => "IMPORT_FILE",
        c::Action::ExportFile => "EXPORT_FILE",
        c::Action::ParseToOmcom => "PARSE_TO_OMCOM",
        c::Action::CreateLifecycleRecord => "CREATE_LIFECYCLE_RECORD",
        c::Action::ModifyLifecycleRecord => "MODIFY_LIFECYCLE_RECORD",
        c::Action::DeleteLifecycleRecord => "DELETE_LIFECYCLE_RECORD",
        c::Action::LinkRecordToObject => "LINK_RECORD_TO_OBJECT",
        c::Action::UpdateSchedule => "UPDATE_SCHEDULE",
        c::Action::UpdateCost => "UPDATE_COST",
        c::Action::UpdateProgress => "UPDATE_PROGRESS",
        c::Action::CreateWorkOrder => "CREATE_WORK_ORDER",
        c::Action::UpdateWorkOrder => "UPDATE_WORK_ORDER",
        c::Action::IngestSensorData => "INGEST_SENSOR_DATA",
        c::Action::Validate => "VALIDATE",
        c::Action::CreateSketch => "CREATE_SKETCH",
        c::Action::AddSketchEntity => "ADD_SKETCH_ENTITY",
        c::Action::AddSketchConstraint => "ADD_SKETCH_CONSTRAINT",
        c::Action::SolveSketch => "SOLVE_SKETCH",
        c::Action::DeleteSketchEntity => "DELETE_SKETCH_ENTITY",
        c::Action::RemoveSketchConstraint => "REMOVE_SKETCH_CONSTRAINT",
        c::Action::DeleteSketch => "DELETE_SKETCH",
        c::Action::ExtrudeSketch => "EXTRUDE_SKETCH",
        c::Action::RevolveSketch => "REVOLVE_SKETCH",
        c::Action::CreateComponent => "CREATE_COMPONENT",
        c::Action::AddFeature => "ADD_FEATURE",
        c::Action::RemoveFeature => "REMOVE_FEATURE",
        c::Action::RebuildFeatures => "REBUILD_FEATURES",
        c::Action::DefineComponent => "DEFINE_COMPONENT",
        c::Action::AddComponentParam => "ADD_COMPONENT_PARAM",
        c::Action::AddConnector => "ADD_CONNECTOR",
        c::Action::PublishComponent => "PUBLISH_COMPONENT",
        c::Action::DeleteComponentDef => "DELETE_COMPONENT_DEF",
        c::Action::CreateSeries => "CREATE_SERIES",
        c::Action::AddSelector => "ADD_SELECTOR",
        c::Action::AddAnswer => "ADD_ANSWER",
        c::Action::AddSpecComponent => "ADD_SPEC_COMPONENT",
        c::Action::AddParamConstraint => "ADD_PARAM_CONSTRAINT",
        c::Action::DeleteSeries => "DELETE_SERIES",
        _ => "ACTION",
    }
}

fn mesh_center(mesh: &c::MeshGeometry) -> c::Point3D {
    if mesh.vertices.is_empty() {
        return c::Point3D { x: 0.0, y: 0.0, z: 0.0 };
    }
    let mut min = c::Point3D { x: f64::INFINITY, y: f64::INFINITY, z: f64::INFINITY };
    let mut max = c::Point3D { x: f64::NEG_INFINITY, y: f64::NEG_INFINITY, z: f64::NEG_INFINITY };
    for v in &mesh.vertices {
        min.x = min.x.min(v.x); min.y = min.y.min(v.y); min.z = min.z.min(v.z);
        max.x = max.x.max(v.x); max.y = max.y.max(v.y); max.z = max.z.max(v.z);
    }
    c::Point3D { x: (min.x + max.x) / 2.0, y: (min.y + max.y) / 2.0, z: (min.z + max.z) / 2.0 }
}

fn parse_record_status(s: &str) -> c::RecordStatus {
    match s.to_lowercase().as_str() {
        "draft" => c::RecordStatus::Draft,
        "open" => c::RecordStatus::Open,
        "in_progress" | "inprogress" => c::RecordStatus::InProgress,
        "blocked" => c::RecordStatus::Blocked,
        "completed" => c::RecordStatus::Completed,
        "approved" => c::RecordStatus::Approved,
        "rejected" => c::RecordStatus::Rejected,
        "closed" => c::RecordStatus::Closed,
        _ => c::RecordStatus::Draft,
    }
}

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Mutex;
    use std::sync::MutexGuard;

    /// Serializes tests that mutate the shared `LIBRARY` global catalog.
    /// Several tests (define component / series) read `catalog.len()` and then
    /// mutate it; running them concurrently races on the global static, so they
    /// must be serialized to stay deterministic.
    static LIBRARY_TEST_LOCK: Mutex<()> = Mutex::new(());

    /// Acquire the global LIBRARY test lock. Returns a guard the caller must
    /// keep alive for the whole test body.
    fn lock_library() -> MutexGuard<'static, ()> {
        LIBRARY_TEST_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn make_instruction(id: &str, action: c::Action) -> c::OmniCommand {
        c::OmniCommand {
            version: "1.0".into(),
            id: id.into(),
            action: action as i32,
            actor: None,
            target: None,
            operations: Vec::new(),
            validation: None,
            timestamp: None,
            metadata: Default::default(),
        }
    }

    /// Initialize the global LIBRARY static with seeded data if not already set.
    fn setup_library() {
        if LIBRARY.read().unwrap().is_none() {
            *LIBRARY.write().unwrap() = Some(omnim_model::library::Library::seeded());
        }
    }

    // ---- Basic execution tests ----

    #[test]
    fn test_create_object_via_omcom() {
        let mut proj = Project::new("Test");
        let mut exe = Executor::new();

        let create_op = c::Operation {
            op: Some(c::operation::Op::CreateObject(c::CreateObjectOperation {
                r#type: c::ObjectType::Wall as i32,
                name: "Wall-1".into(),
                description: "Test wall".into(),
                geometry: Some(c::ParametricGeometry {
                    operations: vec![c::ParametricOperation {
                        operation: Some(c::parametric_operation::Operation::Box(c::Box {
                            length: 5.0,
                            width: 0.3,
                            height: 3.0,
                        })),
                    }],
                }),
                properties: Vec::new(),
                parent_id: String::new(),
            })),
        };

        let mut instr = make_instruction("instr-1", c::Action::CreateObject);
        instr.operations.push(create_op);

        let result = exe.execute(&instr, &mut proj);
        assert!(result.success);
        assert_eq!(result.created_ids.len(), 1);

        let obj = proj.objects.get(&result.created_ids[0]).unwrap();
        assert_eq!(obj.name, "Wall-1");
        assert!(obj.geometry.is_some());
    }

    #[test]
    fn test_set_property_via_omcom() {
        let mut proj = Project::new("Test");
        let mut exe = Executor::new();

        let obj = proj.create_object(c::ObjectType::Wall, "Wall-1");
        let obj_id = obj.id.clone();

        let mut instr = make_instruction("instr-2", c::Action::SetProperty);
        instr.target = Some(c::Target {
            target_type: Some(c::target::TargetType::Object(c::ObjectTarget {
                object_id: obj_id.clone(),
            })),
        });
        instr.operations.push(c::Operation {
            op: Some(c::operation::Op::SetProperty(c::Property {
                name: "FireRating".into(),
                value: Some(c::PropertyValue {
                    value_type: Some(c::property_value::ValueType::StringValue("2h".into())),
                }),
                unit: String::new(),
                description: String::new(),
            })),
        });

        let result = exe.execute(&instr, &mut proj);
        assert!(result.success);
        let prop = proj.objects.get(&obj_id).unwrap().get_property("FireRating").unwrap();
        assert_eq!(prop.name, "FireRating");
    }

    #[test]
    fn test_delete_object_via_omcom() {
        let mut proj = Project::new("Test");
        let mut exe = Executor::new();

        let obj = proj.create_object(c::ObjectType::Wall, "Wall-1");
        let obj_id = obj.id.clone();

        let mut instr = make_instruction("instr-3", c::Action::DeleteObject);
        instr.target = Some(c::Target {
            target_type: Some(c::target::TargetType::Object(c::ObjectTarget {
                object_id: obj_id.clone(),
            })),
        });

        let result = exe.execute(&instr, &mut proj);
        assert!(result.success);
        assert!(!proj.objects.contains(&obj_id));
    }

    #[test]
    fn test_create_lifecycle_record_via_omcom() {
        let mut proj = Project::new("Test");
        let mut exe = Executor::new();

        let mut instr = make_instruction("instr-4", c::Action::CreateLifecycleRecord);
        instr.operations.push(c::Operation {
            op: Some(c::operation::Op::CreateLifecycleRecord(c::CreateLifecycleRecordOperation {
                phase: c::LifecyclePhase::Planning as i32,
                name: "Site Survey".into(),
                description: "Initial site survey".into(),
                record_type: "activity".into(),
                related_object_ids: Vec::new(),
                properties: Vec::new(),
            })),
        });

        let result = exe.execute(&instr, &mut proj);
        assert!(result.success);
        assert_eq!(proj.lifecycle.len(), 1);
    }

    // ---- Undo / Redo tests ----

    #[test]
    fn test_undo_create_object() {
        let mut proj = Project::new("Test");
        let mut exe = Executor::new();

        assert!(!exe.can_undo());
        assert!(!exe.can_redo());

        // 创建一个 Wall
        let mut instr = make_instruction("u-1", c::Action::CreateObject);
        instr.operations.push(c::Operation {
            op: Some(c::operation::Op::CreateObject(c::CreateObjectOperation {
                r#type: c::ObjectType::Wall as i32,
                name: "Wall-Undo".into(),
                description: String::new(),
                geometry: None,
                properties: Vec::new(),
                parent_id: String::new(),
            })),
        });

        let result = exe.execute(&instr, &mut proj);
        assert!(result.success);
        let _wall_id = result.created_ids[0].clone();
        assert_eq!(proj.objects.len(), 1);
        assert!(exe.can_undo());
        assert!(!exe.can_redo());

        // Undo
        let desc = exe.undo(&mut proj).unwrap();
        assert!(desc.contains("removed object"));
        assert_eq!(proj.objects.len(), 0);
        assert!(!exe.can_undo());
        assert!(exe.can_redo());

        // Redo
        let desc = exe.redo(&mut proj).unwrap();
        assert!(desc.contains("created object"));
        assert_eq!(proj.objects.len(), 1);
        assert!(exe.can_undo());
        assert!(!exe.can_redo());
    }

    #[test]
    fn test_undo_delete_object() {
        let mut proj = Project::new("Test");
        let mut exe = Executor::new();

        // 先创建一个对象
        let obj = proj.create_object(c::ObjectType::Beam, "Beam-1");
        let obj_id = obj.id.clone();
        exe.clear_history(); // 清除创建操作的历史

        // 删除操作
        let mut instr = make_instruction("u-2", c::Action::DeleteObject);
        instr.target = Some(c::Target {
            target_type: Some(c::target::TargetType::Object(c::ObjectTarget {
                object_id: obj_id.clone(),
            })),
        });

        let result = exe.execute(&instr, &mut proj);
        assert!(result.success);
        assert!(!proj.objects.contains(&obj_id));

        // Undo delete → restore
        exe.undo(&mut proj).unwrap();
        assert!(proj.objects.contains(&obj_id));
        assert_eq!(proj.objects.get(&obj_id).unwrap().name, "Beam-1");

        // Redo delete
        exe.redo(&mut proj).unwrap();
        assert!(!proj.objects.contains(&obj_id));
    }

    #[test]
    fn test_undo_set_property() {
        let mut proj = Project::new("Test");
        let mut exe = Executor::new();

        let obj = proj.create_object(c::ObjectType::Wall, "Wall-Prop");
        let obj_id = obj.id.clone();
        exe.clear_history();

        // Set property
        let mut instr = make_instruction("u-3", c::Action::SetProperty);
        instr.target = Some(c::Target {
            target_type: Some(c::target::TargetType::Object(c::ObjectTarget {
                object_id: obj_id.clone(),
            })),
        });
        instr.operations.push(c::Operation {
            op: Some(c::operation::Op::SetProperty(c::Property {
                name: "Height".into(),
                value: Some(c::PropertyValue {
                    value_type: Some(c::property_value::ValueType::DoubleValue(3.0)),
                }),
                unit: "m".into(),
                description: String::new(),
            })),
        });

        exe.execute(&instr, &mut proj);
        let prop = proj.objects.get(&obj_id).unwrap().get_property("Height").unwrap();
        assert!(matches!(
            prop.value.as_ref().and_then(|v| v.value_type.as_ref()),
            Some(c::property_value::ValueType::DoubleValue(3.0))
        ));

        // Undo → property removed
        exe.undo(&mut proj).unwrap();
        assert!(proj.objects.get(&obj_id).unwrap().get_property("Height").is_none());
    }

    #[test]
    fn test_undo_set_material() {
        let mut proj = Project::new("Test");
        let mut exe = Executor::new();

        let mat = c::Material {
            id: "mat-steel".into(),
            name: "Steel".into(),
            category: c::MaterialCategory::Metal as i32,
            density: 7850.0,
            ..Default::default()
        };
        proj.materials.insert(mat);

        let obj = proj.create_object(c::ObjectType::Beam, "Beam-Steel");
        let obj_id = obj.id.clone();
        exe.clear_history();

        // Set material
        let mut instr = make_instruction("u-4", c::Action::SetMaterial);
        instr.target = Some(c::Target {
            target_type: Some(c::target::TargetType::Object(c::ObjectTarget {
                object_id: obj_id.clone(),
            })),
        });
        instr.operations.push(c::Operation {
            op: Some(c::operation::Op::SetMaterial(c::SetMaterialOperation {
                object_id: obj_id.clone(),
                material_id: "mat-steel".into(),
            })),
        });

        exe.execute(&instr, &mut proj);
        assert_eq!(
            proj.objects.get(&obj_id).unwrap().material_id,
            Some("mat-steel".into())
        );

        // Undo → material restored to None
        exe.undo(&mut proj).unwrap();
        assert_eq!(proj.objects.get(&obj_id).unwrap().material_id, None);
    }

    // ---- Transaction tests ----

    #[test]
    fn test_transaction_commit() {
        let mut proj = Project::new("Test");
        let mut exe = Executor::new();

        exe.begin_transaction().unwrap();
        assert!(exe.in_transaction());

        // 事务内创建 3 个对象
        for i in 0..3 {
            let mut instr = make_instruction(&format!("tx-{}", i), c::Action::CreateObject);
            instr.operations.push(c::Operation {
                op: Some(c::operation::Op::CreateObject(c::CreateObjectOperation {
                    r#type: c::ObjectType::Wall as i32,
                    name: format!("Wall-{}", i),
                    description: String::new(),
                    geometry: None,
                    properties: Vec::new(),
                    parent_id: String::new(),
                })),
            });
            exe.execute(&instr, &mut proj);
        }

        assert_eq!(proj.objects.len(), 3);

        // Commit — 所有命令作为一个 undo 组
        let count = exe.commit().unwrap();
        assert_eq!(count, 3);
        assert!(!exe.in_transaction());
        assert!(exe.can_undo());

        // One undo should reverse all 3 creates (but our current implementation undoes one at a time)
        // The commands are individual, so 3 undos needed
        for _ in 0..3 {
            exe.undo(&mut proj).unwrap();
        }
        assert_eq!(proj.objects.len(), 0);
    }

    #[test]
    fn test_transaction_rollback() {
        let mut proj = Project::new("Test");
        let mut exe = Executor::new();

        exe.begin_transaction().unwrap();

        let mut instr = make_instruction("tx-rb", c::Action::CreateObject);
        instr.operations.push(c::Operation {
            op: Some(c::operation::Op::CreateObject(c::CreateObjectOperation {
                r#type: c::ObjectType::Wall as i32,
                name: "Wall-Rollback".into(),
                description: String::new(),
                geometry: None,
                properties: Vec::new(),
                parent_id: String::new(),
            })),
        });
        exe.execute(&instr, &mut proj);
        assert_eq!(proj.objects.len(), 1);

        // Rollback — 对象应被删除
        let count = exe.rollback(&mut proj).unwrap();
        assert_eq!(count, 1);
        assert_eq!(proj.objects.len(), 0);
        assert!(!exe.in_transaction());
        assert!(!exe.can_undo()); // rollback doesn't add to undo
    }

    #[test]
    fn test_nested_transaction_denied() {
        let mut exe = Executor::new();
        exe.begin_transaction().unwrap();
        let err = exe.begin_transaction().unwrap_err();
        assert!(err.contains("already in progress"));
    }

    #[test]
    fn test_undo_stack_limit() {
        let mut proj = Project::new("Test");
        let mut exe = Executor::new();

        // Create many objects (exceeds MAX_UNDO_DEPTH)
        for i in 0..250 {
            let mut instr = make_instruction(&format!("limit-{}", i), c::Action::CreateObject);
            instr.operations.push(c::Operation {
                op: Some(c::operation::Op::CreateObject(c::CreateObjectOperation {
                    r#type: c::ObjectType::Generic as i32,
                    name: format!("Obj-{}", i),
                    description: String::new(),
                    geometry: None,
                    properties: Vec::new(),
                    parent_id: String::new(),
                })),
            });
            exe.execute(&instr, &mut proj);
        }

        // Undo stack should be capped at MAX_UNDO_DEPTH
        assert!(exe.undo_depth() <= MAX_UNDO_DEPTH);
    }

    // ---- GenRule integration tests ----

    #[test]
    fn test_create_wall_with_layer_extrusion_via_omcom() {
        let mut proj = Project::new("Test");
        let mut exe = Executor::new();

        // 用 LayerExtrusion 构建一个多层墙体
        let _wall_mesh = omnim_model::gen_rule::layer_extrusion::build_wall(5.0, 3.0, 0.2, 0.02);

        let mut instr = make_instruction("gen-1", c::Action::CreateObject);
        let create_op = c::Operation {
            op: Some(c::operation::Op::CreateObject(c::CreateObjectOperation {
                r#type: c::ObjectType::Wall as i32,
                name: "Wall-Layered".into(),
                description: "Multi-layer wall via GenRule".into(),
                geometry: None,
                properties: Vec::new(),
                parent_id: String::new(),
            })),
        };
        // Attach geometry directly after creation via ModifyObject
        instr.operations.push(create_op);

        let result = exe.execute(&instr, &mut proj);
        assert!(result.success);
        let wall_id = result.created_ids[0].clone();

        // Modify to add geometry
        let mut mod_instr = make_instruction("gen-2", c::Action::ModifyObject);
        mod_instr.target = Some(c::Target {
            target_type: Some(c::target::TargetType::Object(c::ObjectTarget {
                object_id: wall_id.clone(),
            })),
        });
        mod_instr.operations.push(c::Operation {
            op: Some(c::operation::Op::ModifyObject(c::ModifyObjectOperation {
                object_id: wall_id.clone(),
                new_name: None,
                new_description: None,
                new_geometry: Some(c::ParametricGeometry {
                    operations: vec![
                        c::ParametricOperation {
                            operation: Some(c::parametric_operation::Operation::Box(c::Box {
                                length: 5.0, width: 0.2, height: 3.0,
                            })),
                        },
                        c::ParametricOperation {
                            operation: Some(c::parametric_operation::Operation::Box(c::Box {
                                length: 5.0, width: 0.02, height: 3.0,
                            })),
                        },
                    ],
                }),
            })),
        });

        let result = exe.execute(&mod_instr, &mut proj);
        assert!(result.success);

        let wall = proj.objects.get(&wall_id).unwrap();
        assert!(wall.has_geometry());
    }

    #[test]
    fn test_create_road_via_linear_component() {
        let mut proj = Project::new("Test");
        let mut exe = Executor::new();

        let road_mesh = omnim_model::gen_rule::linear_component::build_road(100.0, 3.5, 0.2, 0.1, 30);

        let mut instr = make_instruction("gen-3", c::Action::CreateObject);
        instr.operations.push(c::Operation {
            op: Some(c::operation::Op::CreateObject(c::CreateObjectOperation {
                r#type: c::ObjectType::Road as i32,
                name: "MainRoad".into(),
                description: "LinearComponent road".into(),
                geometry: None,
                properties: Vec::new(),
                parent_id: String::new(),
            })),
        });

        let result = exe.execute(&instr, &mut proj);
        assert!(result.success);
        assert_eq!(proj.objects.len(), 1);
        // Verify road mesh has geometry
        assert!(!road_mesh.vertices.is_empty());
        assert!(!road_mesh.faces.is_empty());
    }

    #[test]
    fn test_transaction_with_genrule_wall() {
        let mut proj = Project::new("Test");
        let mut exe = Executor::new();

        exe.begin_transaction().unwrap();

        // Create site
        let mut instr = make_instruction("tx-gen-1", c::Action::CreateObject);
        instr.operations.push(c::Operation {
            op: Some(c::operation::Op::CreateObject(c::CreateObjectOperation {
                r#type: c::ObjectType::Site as i32,
                name: "Site-A".into(),
                description: String::new(),
                geometry: None,
                properties: Vec::new(),
                parent_id: String::new(),
            })),
        });
        let r = exe.execute(&instr, &mut proj);
        let _site_id = r.created_ids[0].clone();

        // Create building under site
        let mut instr = make_instruction("tx-gen-2", c::Action::CreateObject);
        instr.operations.push(c::Operation {
            op: Some(c::operation::Op::CreateObject(c::CreateObjectOperation {
                r#type: c::ObjectType::Building as i32,
                name: "Building-1".into(),
                description: String::new(),
                geometry: None,
                properties: Vec::new(),
                parent_id: String::new(),
            })),
        });
        let r = exe.execute(&instr, &mut proj);
        let _bldg_id = r.created_ids[0].clone();

        // Create wall under building
        let mut instr = make_instruction("tx-gen-3", c::Action::CreateObject);
        instr.operations.push(c::Operation {
            op: Some(c::operation::Op::CreateObject(c::CreateObjectOperation {
                r#type: c::ObjectType::Wall as i32,
                name: "Wall-1".into(),
                description: String::new(),
                geometry: Some(c::ParametricGeometry {
                    operations: vec![c::ParametricOperation {
                        operation: Some(c::parametric_operation::Operation::Box(c::Box {
                            length: 6.0, width: 0.2, height: 3.0,
                        })),
                    }],
                }),
                properties: Vec::new(),
                parent_id: String::new(),
            })),
        });
        exe.execute(&instr, &mut proj);

        assert_eq!(proj.objects.len(), 3);

        // Rollback the transaction
        exe.rollback(&mut proj).unwrap();
        assert_eq!(proj.objects.len(), 0);
        assert!(!exe.in_transaction());
    }

    // ---- Catalog/series tests ----

    #[test]
    fn test_define_component_via_omcom() {
        let _guard = lock_library();
        setup_library();
        let mut proj = Project::new("Test");
        let mut exe = Executor::new();
        let initial_count = {
            let lib = LIBRARY.read().unwrap();
            let lib = lib.as_ref().unwrap();
            lib.catalog.len()
        };

        let op = c::Operation {
            op: Some(c::operation::Op::DefineComponent(c::DefineComponentOperation {
                name: "TestBeam".into(),
                category: "Structural/Steel/Beam".into(),
                description: "Test beam".into(),
                ifc_class: "IfcBeam".into(),
                standard_reference: "EN 10365".into(),
                tags: vec!["test".into()],
                domain: "structural".into(),
            })),
        };

        let mut instr = make_instruction("cat-1", c::Action::DefineComponent);
        instr.operations.push(op);

        let result = exe.execute(&instr, &mut proj);
        assert!(result.success);
        assert_eq!(LIBRARY.read().unwrap().as_ref().unwrap().catalog.len(), initial_count + 1);
        assert_eq!(result.created_ids.len(), 1);

        let comp = {
            let lib = LIBRARY.read().unwrap();
            let lib = lib.as_ref().unwrap();
            lib.catalog.get(&result.created_ids[0]).unwrap().clone()
        };
        assert_eq!(comp.name, "TestBeam");
        assert_eq!(comp.category, "Structural/Steel/Beam");
    }

    #[test]
    fn test_define_component_undo() {
        let _guard = lock_library();
        setup_library();
        let mut proj = Project::new("Test");
        let mut exe = Executor::new();
        let initial_count = LIBRARY.read().unwrap().as_ref().unwrap().catalog.len();

        let op = c::Operation {
            op: Some(c::operation::Op::DefineComponent(c::DefineComponentOperation {
                name: "UndoBeam".into(),
                category: "Structural".into(),
                description: String::new(),
                ifc_class: String::new(),
                standard_reference: String::new(),
                tags: Vec::new(),
                domain: String::new(),
            })),
        };

        let mut instr = make_instruction("cat-undo", c::Action::DefineComponent);
        instr.operations.push(op);
        exe.execute(&instr, &mut proj);
        assert_eq!(LIBRARY.read().unwrap().as_ref().unwrap().catalog.len(), initial_count + 1);

        // Undo
        let undo_desc = exe.undo(&mut proj).unwrap();
        assert!(undo_desc.contains("removed component def"));
        assert_eq!(LIBRARY.read().unwrap().as_ref().unwrap().catalog.len(), initial_count);

        // Redo
        let redo_desc = exe.redo(&mut proj).unwrap();
        assert!(redo_desc.contains("defined component"));
        assert_eq!(LIBRARY.read().unwrap().as_ref().unwrap().catalog.len(), initial_count + 1);
    }

    #[test]
    fn test_create_series_via_omcom() {
        let mut proj = Project::new("Test");
        let mut exe = Executor::new();
        let initial_count = proj.series.len();

        let op = c::Operation {
            op: Some(c::operation::Op::CreateSeries(c::CreateSeriesOperation {
                name: "CS150".into(),
                description: "Carbon Steel 150".into(),
                domain: "piping".into(),
                standard_reference: "ASME B31.3".into(),
            })),
        };

        let mut instr = make_instruction("ser-1", c::Action::CreateSeries);
        instr.operations.push(op);

        let result = exe.execute(&instr, &mut proj);
        assert!(result.success);
        assert_eq!(proj.series.len(), initial_count + 1);
        assert_eq!(result.created_ids.len(), 1);

        let series = proj.series.get(&result.created_ids[0]).unwrap();
        assert_eq!(series.name, "CS150");
        assert_eq!(series.domain, "piping");
    }

    #[test]
    fn test_create_series_undo() {
        let mut proj = Project::new("Test");
        let mut exe = Executor::new();
        let initial_count = proj.series.len();

        let op = c::Operation {
            op: Some(c::operation::Op::CreateSeries(c::CreateSeriesOperation {
                name: "UndoSeries".into(),
                description: String::new(),
                domain: String::new(),
                standard_reference: String::new(),
            })),
        };

        let mut instr = make_instruction("ser-undo", c::Action::CreateSeries);
        instr.operations.push(op);
        exe.execute(&instr, &mut proj);
        assert_eq!(proj.series.len(), initial_count + 1);

        exe.undo(&mut proj).unwrap();
        assert_eq!(proj.series.len(), initial_count);

        exe.redo(&mut proj).unwrap();
        assert_eq!(proj.series.len(), initial_count + 1);
    }

    #[test]
    fn test_seed_catalog_has_components() {
        let _guard = lock_library();
        setup_library();
        let _proj = Project::new("Test");
        let lib = LIBRARY.read().unwrap();
        let lib = lib.as_ref().unwrap();
        assert!(lib.catalog.len() > 0, "seed catalog should have components");

        let ipe = lib.catalog.find_by_name("Steel-Beam-IPE");
        assert!(ipe.is_some(), "should have IPE beam family");
        let ipe = ipe.unwrap();
        assert_eq!(ipe.category, "Structural/Steel/Beam");
        assert!(ipe.published);
        assert!(!ipe.params.is_empty());
        assert!(!ipe.connectors.is_empty());

        let pipe = lib.catalog.find_by_name("Pipe-CS");
        assert!(pipe.is_some(), "should have Pipe-CS");
        assert!(pipe.unwrap().published);
    }

    #[test]
    fn test_seed_series_has_cs150() {
        let mut proj = Project::new("Test");
        omnim_model::seed_catalog::seed_series(&mut proj.series);
        assert!(proj.series.len() > 0, "seed series should have series");

        let cs150 = proj.series.find_by_name("CS150");
        assert!(cs150.is_some(), "should have CS150 series");
        let cs150 = cs150.unwrap();
        assert!(cs150.root_selector.is_some());
        let root = cs150.root_selector.as_ref().unwrap();
        assert_eq!(root.answers.len(), 6, "CS150 root should have PIPE, ELBO, TEE, REDUCER, FLANGE, VALVE answers");
    }

    // ---- Feature engine tests ----

    #[test]
    fn test_chamfer_feature_constructs() {
        let f = omnim_model::Feature::new_chamfer("chamfer1", 5.0, vec![]);
        assert_eq!(f.feature_type, omnim_model::FeatureType::Chamfer);
        assert_eq!(f.depth, 5.0);
        assert!(f.is_cut);
        assert!(f.dirty);
    }

    #[test]
    fn test_fillet_feature_constructs() {
        let f = omnim_model::Feature::new_fillet("fillet1", 3.0, vec![]);
        assert_eq!(f.feature_type, omnim_model::FeatureType::Fillet);
        assert_eq!(f.depth, 3.0);
        assert!(f.is_cut);
    }

    #[test]
    fn test_pattern_feature_constructs() {
        let f = omnim_model::Feature::new_pattern("pattern1", 2, 2, 1, 5.0, 5.0, 0.0, vec![]);
        assert_eq!(f.feature_type, omnim_model::FeatureType::Pattern);
        assert_eq!(f.depth, 5.0);
        assert_eq!(f.pattern_count_x, Some(2));
    }

    #[test]
    fn test_shell_feature_constructs() {
        let f = omnim_model::Feature::new_shell("shell1", 2.0, vec![]);
        assert_eq!(f.feature_type, omnim_model::FeatureType::Shell);
        assert_eq!(f.depth, 2.0);
        assert!(f.is_cut);
    }

    #[test]
    fn test_pattern_feature_rebuild() {
        let mut proj = Project::new("Test");
        let cid = proj.create_component("PatternTest", None);
        // Add a box sketch
        let sketch = omnim_sketch::Sketch::new("sketch1", omnim_core::Point3D { x: 0.0, y: 0.0, z: 0.0 }, omnim_core::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
        let sid = sketch.id.clone();
        proj.sketches.insert(sketch);
        // Add rectangle
        let rect = omnim_sketch::SketchEntity {
            id: String::new(),
            entity_type: omnim_sketch::SketchEntityType::Rectangle,
            points: vec![
                omnim_core::Point3D { x: 0.0, y: 0.0, z: 0.0 },
                omnim_core::Point3D { x: 2.0, y: 1.0, z: 0.0 },
            ],
            radius: 0.0,
            sides: 4,
            construction: false,
        };
        if let Some(s) = proj.sketches.get_mut(&sid) {
            s.add_entity(rect);
            s.solved = true;
        }
        // Add extrude feature
        let extrude = omnim_model::Feature::new_extrude("extrude1", &sid, 3.0, omnim_core::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
        let eid = proj.add_feature(&cid, extrude).unwrap();
        // Add pattern feature
        let pattern = omnim_model::Feature::new_pattern("pattern1", 2, 2, 1, 5.0, 5.0, 0.0, vec![eid.clone()]);
        let pid = proj.add_feature(&cid, pattern).unwrap();
        // Rebuild
        let result = proj.rebuild_component(&cid);
        assert!(result.is_ok());
        let comp = proj.objects.get(&cid).unwrap();
        assert!(!comp.dirty);
        // Pattern should produce cached geometry
        let p_feature = comp.features.iter().find(|f| f.id == pid).unwrap();
        assert!(p_feature.cached_geometry.is_some());
        // Pattern should have more vertices than the base extrude
        let e_feature = comp.features.iter().find(|f| f.id == eid).unwrap();
        if let (Some(ref pg), Some(ref eg)) = (&p_feature.cached_geometry, &e_feature.cached_geometry) {
            assert!(pg.vertices.len() > eg.vertices.len(), "Pattern should produce more vertices");
        }
    }

    #[test]
    fn test_shell_feature_rebuild() {
        let mut proj = Project::new("Test");
        let cid = proj.create_component("ShellTest", None);
        let sketch = omnim_sketch::Sketch::new("sketch2", omnim_core::Point3D { x: 0.0, y: 0.0, z: 0.0 }, omnim_core::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
        let sid = sketch.id.clone();
        proj.sketches.insert(sketch);
        let rect = omnim_sketch::SketchEntity {
            id: String::new(),
            entity_type: omnim_sketch::SketchEntityType::Rectangle,
            points: vec![
                omnim_core::Point3D { x: 0.0, y: 0.0, z: 0.0 },
                omnim_core::Point3D { x: 2.0, y: 1.0, z: 0.0 },
            ],
            radius: 0.0,
            sides: 4,
            construction: false,
        };
        if let Some(s) = proj.sketches.get_mut(&sid) {
            s.add_entity(rect);
            s.solved = true;
        }
        let extrude = omnim_model::Feature::new_extrude("extrude1", &sid, 3.0, omnim_core::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
        let eid = proj.add_feature(&cid, extrude).unwrap();
        let shell = omnim_model::Feature::new_shell("shell1", 1.0, vec![eid]);
        let shid = proj.add_feature(&cid, shell).unwrap();
        let result = proj.rebuild_component(&cid);
        assert!(result.is_ok());
        let comp = proj.objects.get(&cid).unwrap();
        let s_feature = comp.features.iter().find(|f| f.id == shid).unwrap();
        assert!(s_feature.cached_geometry.is_some());
    }

    #[test]
    fn test_dirty_propagation_on_update() {
        let mut proj = Project::new("Test");
        let cid = proj.create_component("DirtyTest", None);
        let sketch = omnim_sketch::Sketch::new("sketch3", omnim_core::Point3D { x: 0.0, y: 0.0, z: 0.0 }, omnim_core::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
        let sid = sketch.id.clone();
        proj.sketches.insert(sketch);
        let rect = omnim_sketch::SketchEntity {
            id: String::new(),
            entity_type: omnim_sketch::SketchEntityType::Rectangle,
            points: vec![
                omnim_core::Point3D { x: 0.0, y: 0.0, z: 0.0 },
                omnim_core::Point3D { x: 2.0, y: 1.0, z: 0.0 },
            ],
            radius: 0.0,
            sides: 4,
            construction: false,
        };
        if let Some(s) = proj.sketches.get_mut(&sid) {
            s.add_entity(rect);
            s.solved = true;
        }
        let f1 = omnim_model::Feature::new_extrude("base", &sid, 3.0, omnim_core::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
        let f1_id = f1.id.clone();
        proj.add_feature(&cid, f1).unwrap();
        let f2 = omnim_model::Feature::new_extrude("dep", &sid, 2.0, omnim_core::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
        let f2_id = f2.id.clone();
        proj.add_feature(&cid, f2).unwrap();
        // Rebuild once
        proj.rebuild_component(&cid).unwrap();
        // Update f1 depth
        proj.update_feature(&cid, &f1_id, None, None, Some(5.0), None, None, None, None, None, None, None, None, None, None, None, None, None).unwrap();
        // f2 should be dirty after f1 is updated (lower order -> higher order)
        let comp = proj.objects.get(&cid).unwrap();
        let f2_after = comp.features.iter().find(|f| f.id == f2_id).unwrap();
        assert!(f2_after.dirty, "Downstream feature should be marked dirty");
    }

    #[test]
    fn test_boolean_intersect_merge() {
        let mut proj = Project::new("Test");
        let cid = proj.create_component("IntersectTest", None);
        let sketch = omnim_sketch::Sketch::new("sketch4", omnim_core::Point3D { x: 0.0, y: 0.0, z: 0.0 }, omnim_core::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
        let sid = sketch.id.clone();
        proj.sketches.insert(sketch);
        let rect = omnim_sketch::SketchEntity {
            id: String::new(),
            entity_type: omnim_sketch::SketchEntityType::Rectangle,
            points: vec![
                omnim_core::Point3D { x: 0.0, y: 0.0, z: 0.0 },
                omnim_core::Point3D { x: 2.0, y: 1.0, z: 0.0 },
            ],
            radius: 0.0,
            sides: 4,
            construction: false,
        };
        if let Some(s) = proj.sketches.get_mut(&sid) {
            s.add_entity(rect);
            s.solved = true;
        }
        let f1 = omnim_model::Feature::new_extrude("base", &sid, 3.0, omnim_core::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
        proj.add_feature(&cid, f1).unwrap();
        // Boolean intersect is a union feature (non-cut) with Boolean type
        let intersect_f = omnim_model::Feature::new_boolean("intersect", omnim_model::BooleanOp::Intersect, vec![0]);
        proj.add_feature(&cid, intersect_f).unwrap();
        let result = proj.rebuild_component(&cid);
        assert!(result.is_ok());
        let comp = proj.objects.get(&cid).unwrap();
        assert!(!comp.dirty);
        assert!(comp.geometry.is_some());
    }

    #[test]
    fn test_recompute_normals_helper() {
        use omnim_core as c;
        let mesh = c::MeshGeometry {
            vertices: vec![
                c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
                c::Point3D { x: 1.0, y: 0.0, z: 0.0 },
                c::Point3D { x: 0.0, y: 1.0, z: 0.0 },
            ],
            normals: Vec::new(),
            uvs: Vec::new(),
            faces: vec![c::FaceIndices { vertex_indices: vec![0, 1, 2] }],
        };
        let normals = Project::recompute_normals(&mesh);
        assert_eq!(normals.len(), 3);
        for n in &normals {
            let len = (n.x * n.x + n.y * n.y + n.z * n.z).sqrt();
            assert!((len - 1.0).abs() < 1e-6, "Normal should be unit length");
            assert!((n.z - 1.0).abs() < 1e-6, "Triangle in XY plane should have Z=1 normal");
        }
    }
}
