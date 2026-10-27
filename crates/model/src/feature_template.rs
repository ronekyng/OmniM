use crate::profile::{self, ProfilePoints, ProfileType};
use omnim_core as c;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A registry of named feature tree templates.
///
/// Each template defines how to create a set of features + sketches
/// from a ComponentDef's parameter values.
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct FeatureTreeRegistry {
    templates: HashMap<String, FeatureTreeTemplate>,
}

impl FeatureTreeRegistry {
    pub fn new() -> Self {
        Self { templates: HashMap::new() }
    }

    pub fn register(&mut self, template: FeatureTreeTemplate) -> String {
        let id = template.id.clone();
        self.templates.insert(id.clone(), template);
        id
    }

    pub fn get(&self, id: &str) -> Option<&FeatureTreeTemplate> {
        self.templates.get(id)
    }

    pub fn len(&self) -> usize {
        self.templates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.templates.is_empty()
    }
}

/// A template for creating features from component definition parameters.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FeatureTreeTemplate {
    pub id: String,
    pub name: String,
    pub features: Vec<FeatureTemplateStep>,
    /// Maps ComponentDef param names to feature-level param expressions.
    pub param_bindings: HashMap<String, String>,
}

impl FeatureTreeTemplate {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            features: Vec::new(),
            param_bindings: HashMap::new(),
        }
    }

    /// Bind a ComponentDef parameter name to a feature parameter expression.
    /// The expression can be a simple parameter name (e.g., "length") or
    /// a formula (e.g., "1.5 * DN").
    pub fn bind(mut self, comp_param: &str, expr: &str) -> Self {
        self.param_bindings.insert(comp_param.to_string(), expr.to_string());
        self
    }

    /// Add a feature step to this template.
    pub fn step(mut self, step: FeatureTemplateStep) -> Self {
        self.features.push(step);
        self
    }
}

/// A single step in a feature tree template.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FeatureTemplateStep {
    pub name: String,
    pub feature_type: crate::FeatureType,
    /// The type of profile to generate (if profile-based).
    pub profile_type: Option<ProfileType>,
    /// A second profile (for Loft's profile1 / sweep's path).
    pub aux_profile_type: Option<ProfileType>,
    /// Expression for depth (extrude distance).
    pub depth_expr: Option<String>,
    /// Expression for revolve angle (degrees).
    pub angle_expr: Option<String>,
    /// Extrusion direction.
    pub direction: c::Vector3D,
    /// Whether this is a cut operation.
    pub is_cut: bool,
    /// For boolean operations.
    pub boolean_op: Option<crate::BooleanOp>,
    /// Pattern counts (for Pattern feature).
    pub pattern_counts: Option<(usize, usize, usize)>,
}

impl FeatureTemplateStep {
    pub fn extrude(
        name: &str,
        profile_type: ProfileType,
        depth_expr: &str,
        direction: c::Vector3D,
    ) -> Self {
        Self {
            name: name.to_string(),
            feature_type: crate::FeatureType::Extrude,
            profile_type: Some(profile_type),
            aux_profile_type: None,
            depth_expr: Some(depth_expr.to_string()),
            angle_expr: None,
            direction,
            is_cut: false,
            boolean_op: None,
            pattern_counts: None,
        }
    }

    pub fn revolve(
        name: &str,
        profile_type: ProfileType,
        angle_expr: &str,
    ) -> Self {
        Self {
            name: name.to_string(),
            feature_type: crate::FeatureType::Revolve,
            profile_type: Some(profile_type),
            aux_profile_type: None,
            depth_expr: None,
            angle_expr: Some(angle_expr.to_string()),
            direction: c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
            is_cut: false,
            boolean_op: None,
            pattern_counts: None,
        }
    }

    pub fn sweep(
        name: &str,
        section: ProfileType,
        path: ProfileType,
    ) -> Self {
        Self {
            name: name.to_string(),
            feature_type: crate::FeatureType::Sweep,
            profile_type: Some(section),
            aux_profile_type: Some(path),
            depth_expr: None,
            angle_expr: None,
            direction: c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
            is_cut: false,
            boolean_op: None,
            pattern_counts: None,
        }
    }

    pub fn loft(
        name: &str,
        profile0: ProfileType,
        profile1: ProfileType,
    ) -> Self {
        Self {
            name: name.to_string(),
            feature_type: crate::FeatureType::Loft,
            profile_type: Some(profile0),
            aux_profile_type: Some(profile1),
            depth_expr: None,
            angle_expr: None,
            direction: c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
            is_cut: false,
            boolean_op: None,
            pattern_counts: None,
        }
    }

    pub fn boolean(name: &str, op: crate::BooleanOp) -> Self {
        Self {
            name: name.to_string(),
            feature_type: crate::FeatureType::Boolean,
            profile_type: None,
            aux_profile_type: None,
            depth_expr: None,
            angle_expr: None,
            direction: c::Vector3D { x: 0.0, y: 0.0, z: 0.0 },
            is_cut: op == crate::BooleanOp::Subtract,
            boolean_op: Some(op),
            pattern_counts: None,
        }
    }

    pub fn cut(mut self) -> Self {
        self.is_cut = true;
        self
    }

    pub fn pattern(mut self, cx: usize, cy: usize, cz: usize) -> Self {
        self.pattern_counts = Some((cx, cy, cz));
        self
    }
}

/// Instantiate a feature tree template into actual sketch and feature objects,
/// adding them to the given project.
///
/// Returns: (created_sketch_ids, created_feature_ids)
pub fn instantiate_template(
    project: &mut crate::project::Project,
    component_id: &str,
    template: &FeatureTreeTemplate,
    params: &HashMap<String, f64>,
) -> Result<(Vec<String>, Vec<String>), String> {
    let mut sketch_ids = Vec::new();
    let mut feature_ids = Vec::new();

    for step in &template.features {
        // 1. Generate profile sketch if needed
        let sketch_id = if let Some(ref pt) = step.profile_type {
            let profile = match pt {
                ProfileType::PipeSchedule { segments } => {
                    let dn = params.get("DN").copied().unwrap_or(100.0);
                    let schedule = params.get("schedule").copied().unwrap_or(0.0);
                    profile::resolve_pipe_schedule_profile(dn, schedule, *segments)
                }
                ProfileType::ParametricRect { width_param, height_param } => {
                    let width = params.get(width_param).copied().unwrap_or(5000.0);
                    let height = params.get(height_param).copied().unwrap_or(200.0);
                    profile::generate(&ProfileType::Rectangle { width, height })
                }
                _ => profile::generate(pt),
            };
            let sketch = create_profile_sketch(project, &step.name, &profile)?;
            let sid = sketch.id.clone();
            // Link sketch to component
            project.sketches.insert(sketch);
            sid
        } else {
            String::new()
        };

        if !sketch_id.is_empty() {
            sketch_ids.push(sketch_id.clone());
        }

        // 2. Create feature
        let resolve = |expr: &str| -> f64 {
            resolve_param(expr, params)
        };

        let feature = match step.feature_type {
            crate::FeatureType::Extrude => {
                let depth = step.depth_expr.as_ref().map(|e| resolve(e)).unwrap_or(1.0);
                let mut f = crate::Feature::new_extrude(
                    &step.name,
                    &sketch_id,
                    depth,
                    step.direction,
                );
                f.is_cut = step.is_cut;
                f
            }
            crate::FeatureType::Revolve => {
                let angle_deg = step.angle_expr.as_ref().map(|e| resolve(e)).unwrap_or(360.0);
                let angle_rad = angle_deg * std::f64::consts::PI / 180.0;
                let mut f = crate::Feature::new_revolve(
                    &step.name,
                    &sketch_id,
                    "",
                    angle_rad,
                );
                f.is_cut = step.is_cut;
                f
            }
            crate::FeatureType::Boolean => {
                let op = step.boolean_op.unwrap_or(crate::BooleanOp::Union);
                crate::Feature::new_boolean(&step.name, op, vec![])
            }
            crate::FeatureType::Pattern => {
                let (cx, cy, cz) = step.pattern_counts.unwrap_or((1, 1, 1));
                let sp = step.depth_expr.as_ref().map(|e| resolve(e)).unwrap_or(100.0);
                crate::Feature::new_pattern(
                    &step.name, cx, cy, cz,
                    sp, sp, 0.0, vec![],
                )
            }
            crate::FeatureType::Sweep => {
                let path_sketch_id = if let Some(ref pt) = step.aux_profile_type {
                    let profile = profile::generate(pt);
                    let sketch = create_profile_sketch(project, &format!("{}_path", step.name), &profile)?;
                    let sid = sketch.id.clone();
                    project.sketches.insert(sketch);
                    sketch_ids.push(sid.clone());
                    sid
                } else {
                    return Err("sweep step requires a path profile".into());
                };
                let mut f = crate::Feature::new_sweep(&step.name, &sketch_id, &path_sketch_id);
                f.is_cut = step.is_cut;
                f
            }
            crate::FeatureType::Loft => {
                let profile1_id = if let Some(ref pt) = step.aux_profile_type {
                    let profile = profile::generate(pt);
                    let sketch = create_profile_sketch(project, &format!("{}_end", step.name), &profile)?;
                    let sid = sketch.id.clone();
                    project.sketches.insert(sketch);
                    sketch_ids.push(sid.clone());
                    sid
                } else {
                    return Err("loft step requires a second profile".into());
                };
                let mut f = crate::Feature::new_loft(&step.name, vec![sketch_id, profile1_id]);
                f.is_cut = step.is_cut;
                f
            }
            _ => return Err(format!("unsupported feature type: {:?}", step.feature_type)),
        };

        let fid = project.add_feature(component_id, feature)?;
        feature_ids.push(fid);
    }

    Ok((sketch_ids, feature_ids))
}

fn create_profile_sketch(
    _project: &mut crate::project::Project,
    name: &str,
    profile: &ProfilePoints,
) -> Result<omnim_sketch::Sketch, String> {
    let mut sketch = omnim_sketch::Sketch::new(
        name,
        c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
        c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
    );

    for i in 0..profile.len() {
        let j = (i + 1) % profile.len();
        let start = profile[i];
        let end = profile[j];

        // For very close points (circular tessellation), skip
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let dz = end.z - start.z;
        let len_sq = dx * dx + dy * dy + dz * dz;
        if len_sq < 1e-12 {
            continue;
        }

        let entity = omnim_sketch::SketchEntity::line(start, end);
        sketch.add_entity(entity);
    }

    if sketch.entities.is_empty() {
        return Err("empty profile".to_string());
    }

    // Mark as solved (profile points are already determined)
    sketch.solved = true;
    let points: Vec<(usize, usize, c::Point3D)> = sketch.entities.iter().enumerate()
        .flat_map(|(ei, entity)| {
            entity.points.iter().enumerate()
                .map(move |(pi, pt)| (ei, pi, *pt))
        })
        .collect();
    for (ei, pi, pt) in points {
        sketch.set_solved_point(ei, pi, pt);
    }

    Ok(sketch)
}

/// Resolve a parameter expression.
///
/// Supports:
/// - Simple parameter names like "length" → params["length"]
/// - Arithmetic expressions with +, -, *, /
fn resolve_param(expr: &str, params: &HashMap<String, f64>) -> f64 {
    let expr = expr.trim();

    // Try direct parameter lookup first
    if let Some(&val) = params.get(expr) {
        return val;
    }

    // Try simple arithmetic: "1.5 * DN"
    if let Some(pos) = expr.find('*') {
        let lhs = expr[..pos].trim();
        let rhs = expr[pos + 1..].trim();
        let lv = resolve_param(lhs, params);
        let rv = resolve_param(rhs, params);
        return lv * rv;
    }
    if let Some(pos) = expr.find('/') {
        let lhs = expr[..pos].trim();
        let rhs = expr[pos + 1..].trim();
        let lv = resolve_param(lhs, params);
        let rv = resolve_param(rhs, params);
        if rv.abs() > 1e-12 {
            return lv / rv;
        }
    }
    if let Some(pos) = expr.find('+') {
        let lhs = expr[..pos].trim();
        let rhs = expr[pos + 1..].trim();
        let lv = resolve_param(lhs, params);
        let rv = resolve_param(rhs, params);
        return lv + rv;
    }
    if let Some(pos) = expr.find('-') {
        // Check for negative numbers
        if pos > 0 {
            let lhs = expr[..pos].trim();
            let rhs = expr[pos + 1..].trim();
            let lv = resolve_param(lhs, params);
            let rv = resolve_param(rhs, params);
            return lv - rv;
        }
    }

    // Try parsing as a literal number
    if let Ok(v) = expr.parse::<f64>() {
        return v;
    }

    // Fallback
    0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_resolve_param_direct() {
        let mut p = HashMap::new();
        p.insert("length".into(), 6000.0);
        let v = resolve_param("length", &p);
        assert!((v - 6000.0).abs() < 1e-10);
    }

    #[test]
    fn test_resolve_param_arithmetic() {
        let mut p = HashMap::new();
        p.insert("DN".into(), 100.0);
        let v = resolve_param("1.5 * DN", &p);
        assert!((v - 150.0).abs() < 1e-10);
    }

    #[test]
    fn test_resolve_param_literal() {
        let p = HashMap::new();
        let v = resolve_param("360", &p);
        assert!((v - 360.0).abs() < 1e-10);
    }

    #[test]
    fn test_create_profile_sketch_triangle() {
        let mut project = crate::project::Project::new("test");
        let profile = vec![
            c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
            c::Point3D { x: 100.0, y: 0.0, z: 0.0 },
            c::Point3D { x: 0.0, y: 100.0, z: 0.0 },
        ];
        let sketch = create_profile_sketch(&mut project, "tri", &profile).unwrap();
        assert!(sketch.solved);
        assert_eq!(sketch.entities.len(), 3);
    }

    #[test]
    fn test_instantiate_extrude_template() {
        let mut project = crate::project::Project::new("test");
        let cid = crate::object::OmniObject::new(c::ObjectType::Component, "beam");
        let cid_clone = cid.id.clone();
        project.objects.insert(cid);

        let template = FeatureTreeTemplate::new("ipe-beam")
            .bind("length", "length")
            .step(FeatureTemplateStep::extrude(
                "main",
                ProfileType::IShape {
                    height: 240.0,
                    width: 120.0,
                    web_thickness: 6.2,
                    flange_thickness: 9.8,
                },
                "length",
                c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
            ));

        let mut params = HashMap::new();
        params.insert("length".into(), 6000.0);

        let result = instantiate_template(&mut project, &cid_clone, &template, &params);
        assert!(result.is_ok(), "instantiate failed: {:?}", result.err());
        let (sketch_ids, feature_ids) = result.unwrap();
        assert_eq!(sketch_ids.len(), 1);
        assert_eq!(feature_ids.len(), 1);

        // Verify rebuild
        let rebuild = project.rebuild_component(&cid_clone);
        assert!(rebuild.is_ok(), "rebuild failed: {:?}", rebuild.err());
        let obj = project.objects.get(&cid_clone).unwrap();
        assert!(obj.features[0].cached_geometry.is_some());
    }

    #[test]
    fn test_instantiate_sweep_template() {
        let mut project = crate::project::Project::new("test");
        let cid = crate::object::OmniObject::new(c::ObjectType::Component, "elbow");
        let cid_clone = cid.id.clone();
        project.objects.insert(cid);

        let template = FeatureTreeTemplate::new("circ-sweep")
            .step(FeatureTemplateStep::sweep(
                "main",
                ProfileType::Circle { radius: 50.0, segments: 16 },
                ProfileType::UShape { height: 100.0, width: 50.0, web_thickness: 5.0, flange_thickness: 8.0 },
            ));

        let params = HashMap::new();
        let result = instantiate_template(&mut project, &cid_clone, &template, &params);
        assert!(result.is_ok(), "instantiate failed: {:?}", result.err());
        let (sketch_ids, _feature_ids) = result.unwrap();
        assert_eq!(sketch_ids.len(), 2);
    }

    #[test]
    fn test_instantiate_loft_template() {
        let mut project = crate::project::Project::new("test");
        let cid = crate::object::OmniObject::new(c::ObjectType::Component, "reducer");
        let cid_clone = cid.id.clone();
        project.objects.insert(cid);

        let template = FeatureTreeTemplate::new("rect-loft")
            .step(FeatureTemplateStep::loft(
                "main",
                ProfileType::Circle { radius: 50.0, segments: 16 },
                ProfileType::Rectangle { width: 100.0, height: 100.0 },
            ));

        let params = HashMap::new();
        let result = instantiate_template(&mut project, &cid_clone, &template, &params);
        assert!(result.is_ok(), "instantiate failed: {:?}", result.err());
        let (sketch_ids, feature_ids) = result.unwrap();
        assert_eq!(sketch_ids.len(), 2);
        assert_eq!(feature_ids.len(), 1);
    }
}