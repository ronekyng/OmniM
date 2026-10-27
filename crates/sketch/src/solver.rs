use crate::{
    Sketch, SketchConstraint, SketchConstraintType, SketchEntity, SketchEntityType,
    SketchPointRef,
};
use omnim_core as c;

type SlvsPoint = slvs::entity::EntityHandle<slvs::entity::Point>;
type SlvsLine = slvs::entity::EntityHandle<slvs::entity::LineSegment>;
type SlvsCircle = slvs::entity::EntityHandle<slvs::entity::Circle>;
type SlvsArc = slvs::entity::EntityHandle<slvs::entity::ArcOfCircle>;
type SlvsWorkplane = slvs::entity::EntityHandle<slvs::entity::Workplane>;
type SlvsNormal = slvs::entity::EntityHandle<slvs::entity::Normal>;

pub fn to_slvs_distance(a: &c::Point3D, b: &c::Point3D) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let dz = a.z - b.z;
    (dx * dx + dy * dy + dz * dz).sqrt()
}

fn normal_to_quaternion(nx: f64, ny: f64, nz: f64) -> [f64; 4] {
    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len < 1e-12 {
        return slvs::utils::make_quaternion([1.0, 0.0, 0.0], [0.0, 1.0, 0.0]);
    }
    let (nx, ny, nz) = (nx / len, ny / len, nz / len);

    let u = if nx.abs() < 0.9 {
        let t = [1.0, 0.0, 0.0];
        let u = cross([nx, ny, nz], t);
        normalize(u)
    } else {
        let t = [0.0, 1.0, 0.0];
        let u = cross([nx, ny, nz], t);
        normalize(u)
    };
    let v = normalize(cross([nx, ny, nz], u));
    slvs::utils::make_quaternion(u, v)
}

fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn normalize(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        return [0.0, 0.0, 1.0];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

#[derive(Debug, Clone)]
pub struct SolveResult {
    pub success: bool,
    pub rank: i32,
    pub dof: i32,
    pub failed_constraints: Vec<usize>,
    pub message: String,
}

/// Tracks all slvs handles created for a single SketchEntity.
#[derive(Debug, Default, Clone)]
struct BuiltEntity {
    points: Vec<SlvsPoint>,
    line: Option<SlvsLine>,
    circle: Option<SlvsCircle>,
    arc: Option<SlvsArc>,
    normal: Option<SlvsNormal>,
    _distance: Option<slvs::entity::EntityHandle<slvs::entity::Distance>>,
}

pub fn solve(sketch: &mut Sketch) -> SolveResult {
    let mut system = slvs::System::new();
    let group = system.add_group();

    let workplane = if !sketch.is_3d() {
        let origin_pt = system.sketch(slvs::entity::Point::new_in_3d(
            group,
            [sketch.origin.x, sketch.origin.y, sketch.origin.z],
        )).unwrap();
        let quat = normal_to_quaternion(
            sketch.normal.x, sketch.normal.y, sketch.normal.z,
        );
        let normal = system.sketch(slvs::entity::Normal::new_in_3d(group, quat)).unwrap();
        Some(system.sketch(slvs::entity::Workplane::new(group, origin_pt, normal)).unwrap())
    } else {
        None
    };

    let mut built_entities: Vec<BuiltEntity> = Vec::new();

    for entity in &sketch.entities {
        let built = build_entity(&mut system, group, entity, workplane);
        built_entities.push(built);
    }

    for constraint in &sketch.constraints {
        build_constraint(&mut system, group, constraint, &built_entities, workplane);
    }

    let result = system.solve(&group);

    let success = matches!(result, slvs::system::SolveResult::Ok { .. });

    for (entity_idx, built) in built_entities.iter().enumerate() {
        for (pt_idx, pt_handle) in built.points.iter().enumerate() {
            if let Ok(pt_data) = system.entity_data(pt_handle) {
                match pt_data {
                    slvs::entity::Point::In3d { coords, .. } => {
                        sketch.set_solved_point(
                            entity_idx,
                            pt_idx,
                            c::Point3D { x: coords[0], y: coords[1], z: coords[2] },
                        );
                    }
                    slvs::entity::Point::OnWorkplane { coords, .. } => {
                        sketch.set_solved_point(
                            entity_idx,
                            pt_idx,
                            c::Point3D { x: coords[0], y: coords[1], z: 0.0 },
                        );
                    }
                }
            }
        }
    }

    sketch.solved = success;

    let (dof, failed) = match &result {
        slvs::system::SolveResult::Ok { dof, .. } => (*dof, Vec::new()),
        slvs::system::SolveResult::Fail { dof, failed_constraints, .. } => {
            (*dof, (0..failed_constraints.len()).collect())
        }
    };

    SolveResult {
        success,
        rank: 0,
        dof,
        failed_constraints: failed,
        message: if success {
            "Sketch solved successfully".to_string()
        } else {
            format!("Solve failed: dof={}", dof)
        },
    }
}

fn make_point(
    system: &mut slvs::System,
    group: slvs::group::Group,
    pt: c::Point3D,
    workplane: Option<SlvsWorkplane>,
) -> SlvsPoint {
    match workplane {
        Some(wp) => system.sketch(slvs::entity::Point::new_on_workplane(
            group, wp, [pt.x, pt.y],
        )).unwrap(),
        None => system.sketch(slvs::entity::Point::new_in_3d(
            group, [pt.x, pt.y, pt.z],
        )).unwrap(),
    }
}

fn build_entity(
    system: &mut slvs::System,
    group: slvs::group::Group,
    entity: &SketchEntity,
    workplane: Option<SlvsWorkplane>,
) -> BuiltEntity {
    match entity.entity_type {
        SketchEntityType::Point => {
            let pt = entity.points.first().cloned().unwrap_or(c::Point3D { x: 0.0, y: 0.0, z: 0.0 });
            let h = make_point(system, group, pt, workplane);
            BuiltEntity { points: vec![h], ..Default::default() }
        }
        SketchEntityType::Line => {
            let start = entity.points.get(0).cloned().unwrap_or(c::Point3D { x: 0.0, y: 0.0, z: 0.0 });
            let end = entity.points.get(1).cloned().unwrap_or(c::Point3D { x: 1.0, y: 0.0, z: 0.0 });
            let h0 = make_point(system, group, start, workplane);
            let h1 = make_point(system, group, end, workplane);
            let line = system.sketch(slvs::entity::LineSegment::new(group, h0, h1)).unwrap();
            BuiltEntity { points: vec![h0, h1], line: Some(line), ..Default::default() }
        }
        SketchEntityType::Circle => {
            let center = entity.points.first().cloned().unwrap_or(c::Point3D { x: 0.0, y: 0.0, z: 0.0 });
            let hc = make_point(system, group, center, workplane);
            let normal_entity = match workplane {
                Some(wp) => slvs::entity::Normal::new_on_workplane(group, wp),
                None => {
                    let q = normal_to_quaternion(0.0, 0.0, 1.0);
                    slvs::entity::Normal::new_in_3d(group, q)
                }
            };
            let hn = system.sketch(normal_entity).unwrap();
            let radius = entity.radius.max(0.001);
            let hr = system.sketch(slvs::entity::Distance::new(group, radius)).unwrap();
            let circle = system.sketch(slvs::entity::Circle::new(group, hn, hc, hr)).unwrap();
            BuiltEntity { points: vec![hc], circle: Some(circle), normal: Some(hn), _distance: Some(hr), ..Default::default() }
        }
        SketchEntityType::Arc => {
            let wp = match workplane {
                Some(w) => w,
                None => {
                    let origin_pt = system.sketch(slvs::entity::Point::new_in_3d(group, [0.0, 0.0, 0.0])).unwrap();
                    let normal = system.sketch(slvs::entity::Normal::new_in_3d(group, [1.0, 0.0, 0.0, 0.0])).unwrap();
                    system.sketch(slvs::entity::Workplane::new(group, origin_pt, normal)).unwrap()
                }
            };
            let center = entity.points.get(0).cloned().unwrap_or(c::Point3D { x: 0.0, y: 0.0, z: 0.0 });
            let start = entity.points.get(1).cloned().unwrap_or(c::Point3D { x: 1.0, y: 0.0, z: 0.0 });
            let end = entity.points.get(2).cloned().unwrap_or(c::Point3D { x: 0.0, y: 1.0, z: 0.0 });
            let hc = make_point(system, group, center, workplane);
            let hs = make_point(system, group, start, workplane);
            let he = make_point(system, group, end, workplane);
            let arc = system.sketch(slvs::entity::ArcOfCircle::new(group, wp, hc, hs, he)).unwrap();
            BuiltEntity { points: vec![hc, hs, he], arc: Some(arc), ..Default::default() }
        }
        SketchEntityType::Rectangle | SketchEntityType::Polygon | SketchEntityType::Ellipse | SketchEntityType::Spline | SketchEntityType::Polyline => {
            let mut handles = Vec::new();
            for pt in &entity.points {
                let h = make_point(system, group, *pt, workplane);
                handles.push(h);
            }
            if handles.len() >= 2 {
                let mut prev = handles[0];
                for &h in &handles[1..] {
                    let _line = system.sketch(slvs::entity::LineSegment::new(group, prev, h)).unwrap();
                    prev = h;
                }
            }
            BuiltEntity { points: handles, ..Default::default() }
        }
    }
}

fn build_constraint(
    system: &mut slvs::System,
    group: slvs::group::Group,
    constraint: &SketchConstraint,
    built_entities: &[BuiltEntity],
    workplane: Option<SlvsWorkplane>,
) {
    let resolve_point = |r#ref: &SketchPointRef| -> Option<SlvsPoint> {
        built_entities.get(r#ref.entity_index)?
            .points.get(r#ref.point_index)
            .copied()
    };

    let resolve_line = |r#ref: &SketchPointRef| -> Option<SlvsLine> {
        built_entities.get(r#ref.entity_index)?.line
    };

    let resolve_circle = |r#ref: &SketchPointRef| -> Option<SlvsCircle> {
        built_entities.get(r#ref.entity_index)?.circle
    };

    let resolve_arc = |r#ref: &SketchPointRef| -> Option<SlvsArc> {
        built_entities.get(r#ref.entity_index)?.arc
    };

    match constraint.constraint_type {
        SketchConstraintType::Fixed => {
            if let Some(r#ref) = constraint.entity_refs.first() {
                if let Some(pt) = resolve_point(r#ref) {
                    let _ = system.constrain(slvs::constraint::WhereDragged::new(group, pt, None));
                }
            }
        }
        SketchConstraintType::Coincident => {
            if constraint.entity_refs.len() >= 2 {
                if let (Some(pt_a), Some(pt_b)) = (
                    resolve_point(&constraint.entity_refs[0]),
                    resolve_point(&constraint.entity_refs[1]),
                ) {
                    let _ = system.constrain(slvs::constraint::PointsCoincident::new(group, pt_a, pt_b, None));
                }
            }
        }
        SketchConstraintType::Distance | SketchConstraintType::Length => {
            if constraint.entity_refs.len() >= 2 {
                if let (Some(pt_a), Some(pt_b)) = (
                    resolve_point(&constraint.entity_refs[0]),
                    resolve_point(&constraint.entity_refs[1]),
                ) {
                    let _ = system.constrain(slvs::constraint::PtPtDistance::new(
                        group, pt_a, pt_b, constraint.value, None,
                    ));
                }
            }
        }
        SketchConstraintType::Horizontal => {
            if let (Some(wp), Some(r#ref_a), Some(r#ref_b)) = (
                workplane,
                constraint.entity_refs.get(0),
                constraint.entity_refs.get(1),
            ) {
                if let (Some(pt_a), Some(pt_b)) = (resolve_point(r#ref_a), resolve_point(r#ref_b)) {
                    let _ = system.constrain(slvs::constraint::Horizontal::from_points(group, wp, pt_a, pt_b));
                }
            }
        }
        SketchConstraintType::Vertical => {
            if let (Some(wp), Some(r#ref_a), Some(r#ref_b)) = (
                workplane,
                constraint.entity_refs.get(0),
                constraint.entity_refs.get(1),
            ) {
                if let (Some(pt_a), Some(pt_b)) = (resolve_point(r#ref_a), resolve_point(r#ref_b)) {
                    let _ = system.constrain(slvs::constraint::Vertical::from_points(group, wp, pt_a, pt_b));
                }
            }
        }
        SketchConstraintType::Parallel => {
            if constraint.entity_refs.len() >= 2 {
                if let (Some(line_a), Some(line_b)) = (
                    resolve_line(&constraint.entity_refs[0]),
                    resolve_line(&constraint.entity_refs[1]),
                ) {
                    let _ = system.constrain(slvs::constraint::Parallel::new(group, line_a, line_b, None));
                }
            }
        }
        SketchConstraintType::Perpendicular => {
            if constraint.entity_refs.len() >= 2 {
                if let (Some(line_a), Some(line_b)) = (
                    resolve_line(&constraint.entity_refs[0]),
                    resolve_line(&constraint.entity_refs[1]),
                ) {
                    let _ = system.constrain(slvs::constraint::Perpendicular::new(group, line_a, line_b, None));
                }
            }
        }
        SketchConstraintType::Angle => {
            if constraint.entity_refs.len() >= 2 {
                if let (Some(line_a), Some(line_b)) = (
                    resolve_line(&constraint.entity_refs[0]),
                    resolve_line(&constraint.entity_refs[1]),
                ) {
                    let _ = system.constrain(slvs::constraint::Angle::new(
                        group, line_a, line_b, constraint.value, workplane, false,
                    ));
                }
            }
        }
        SketchConstraintType::Diameter => {
            if let Some(r#ref) = constraint.entity_refs.first() {
                if let Some(circle) = resolve_circle(r#ref) {
                    let _ = system.constrain(slvs::constraint::Diameter::new(group, circle, constraint.value));
                } else if let Some(arc) = resolve_arc(r#ref) {
                    let _ = system.constrain(slvs::constraint::Diameter::new(group, arc, constraint.value));
                }
            }
        }
        SketchConstraintType::Radius => {
            if let Some(r#ref) = constraint.entity_refs.first() {
                let diameter = constraint.value * 2.0;
                if let Some(circle) = resolve_circle(r#ref) {
                    let _ = system.constrain(slvs::constraint::Diameter::new(group, circle, diameter));
                } else if let Some(arc) = resolve_arc(r#ref) {
                    let _ = system.constrain(slvs::constraint::Diameter::new(group, arc, diameter));
                }
            }
        }
        SketchConstraintType::Tangent => {
            if constraint.entity_refs.len() >= 2 {
                if let Some(wp) = workplane {
                    if let (Some(arc_a), Some(arc_b)) = (
                        resolve_arc(&constraint.entity_refs[0]),
                        resolve_arc(&constraint.entity_refs[1]),
                    ) {
                        let _ = system.constrain(slvs::constraint::CurveCurveTangent::new(
                            group, wp, arc_a, arc_b, false, false,
                        ));
                    }
                }
            }
        }
        SketchConstraintType::AtMidpoint => {
            if constraint.entity_refs.len() >= 2 {
                if let (Some(pt), Some(line)) = (
                    resolve_point(&constraint.entity_refs[0]),
                    resolve_line(&constraint.entity_refs[1]),
                ) {
                    let _ = system.constrain(slvs::constraint::AtMidpoint::new(group, pt, line, None));
                }
            }
        }
        SketchConstraintType::EqualLength => {
            if constraint.entity_refs.len() >= 2 {
                if let (Some(line_a), Some(line_b)) = (
                    resolve_line(&constraint.entity_refs[0]),
                    resolve_line(&constraint.entity_refs[1]),
                ) {
                    let _ = system.constrain(slvs::constraint::EqualLengthLines::new(group, line_a, line_b, None));
                }
            }
        }
        SketchConstraintType::EqualRadius => {
            if constraint.entity_refs.len() >= 2 {
                if let (Some(circ_a), Some(circ_b)) = (
                    resolve_circle(&constraint.entity_refs[0]),
                    resolve_circle(&constraint.entity_refs[1]),
                ) {
                    let _ = system.constrain(slvs::constraint::EqualRadius::new(group, circ_a, circ_b));
                } else if let (Some(arc_a), Some(arc_b)) = (
                    resolve_arc(&constraint.entity_refs[0]),
                    resolve_arc(&constraint.entity_refs[1]),
                ) {
                    let _ = system.constrain(slvs::constraint::EqualRadius::new(group, arc_a, arc_b));
                }
            }
        }
        SketchConstraintType::PtOnLine => {
            if constraint.entity_refs.len() >= 2 {
                if let (Some(pt), Some(line)) = (
                    resolve_point(&constraint.entity_refs[0]),
                    resolve_line(&constraint.entity_refs[1]),
                ) {
                    let _ = system.constrain(slvs::constraint::PtOnLine::new(group, pt, line, None));
                }
            }
        }
        SketchConstraintType::PtOnCircle => {
            if constraint.entity_refs.len() >= 2 {
                if let (Some(pt), Some(circle)) = (
                    resolve_point(&constraint.entity_refs[0]),
                    resolve_circle(&constraint.entity_refs[1]),
                ) {
                    let _ = system.constrain(slvs::constraint::PtOnCircle::new(group, pt, circle));
                } else if let (Some(pt), Some(arc)) = (
                    resolve_point(&constraint.entity_refs[0]),
                    resolve_arc(&constraint.entity_refs[1]),
                ) {
                    let _ = system.constrain(slvs::constraint::PtOnCircle::new(group, pt, arc));
                }
            }
        }
        SketchConstraintType::Symmetric => {
            if constraint.entity_refs.len() >= 2 {
                if let (Some(wp), Some(pt_a), Some(pt_b)) = (
                    workplane,
                    resolve_point(&constraint.entity_refs[0]),
                    resolve_point(&constraint.entity_refs[1]),
                ) {
                    let _ = system.constrain(slvs::constraint::Symmetric::new(group, pt_a, pt_b, wp, None));
                }
            }
        }
        SketchConstraintType::SameOrientation => {
            if constraint.entity_refs.len() >= 2 {
                let normal_a = built_entities.get(constraint.entity_refs[0].entity_index)
                    .and_then(|e| e.normal);
                let normal_b = built_entities.get(constraint.entity_refs[1].entity_index)
                    .and_then(|e| e.normal);
                if let (Some(na), Some(nb)) = (normal_a, normal_b) {
                    let _ = system.constrain(slvs::constraint::SameOrientation::new(group, na, nb));
                }
            }
        }
    }
}

pub fn solve_sketch_3d(sketch: &mut Sketch) -> SolveResult {
    solve(sketch)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pt(x: f64, y: f64, z: f64) -> c::Point3D {
        c::Point3D { x, y, z }
    }

    fn origin() -> c::Point3D {
        pt(0.0, 0.0, 0.0)
    }

    fn normal_2d() -> c::Vector3D {
        c::Vector3D { x: 0.0, y: 0.0, z: 1.0 }
    }

    fn fix_point(sketch: &mut Sketch, entity_index: usize, point_index: usize) {
        sketch.add_constraint(SketchConstraint {
            id: uuid::Uuid::new_v4().to_string(),
            constraint_type: SketchConstraintType::Fixed,
            entity_refs: vec![SketchPointRef { entity_index, point_index }],
            value: 0.0,
        });
    }

    #[test]
    fn test_to_slvs_distance() {
        let a = pt(0.0, 0.0, 0.0);
        let b = pt(3.0, 4.0, 0.0);
        let d = to_slvs_distance(&a, &b);
        assert!((d - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_solve_empty_sketch() {
        let mut sketch = Sketch::new("empty", origin(), normal_2d());
        let result = solve(&mut sketch);
        assert!(result.success);
    }

    #[test]
    fn test_solve_single_point_fixed() {
        let mut sketch = Sketch::new("s", origin(), normal_2d());
        sketch.add_entity(SketchEntity::point(1.0, 2.0, 0.0));
        fix_point(&mut sketch, 0, 0);
        let result = solve(&mut sketch);
        assert!(result.success);
        assert!(sketch.solved);
        let p = sketch.get_solved_point(0, 0).unwrap();
        assert!((p.x - 1.0).abs() < 1e-6);
        assert!((p.y - 2.0).abs() < 1e-6);
    }

    #[test]
    fn test_solve_two_points_coincident() {
        let mut sketch = Sketch::new("s", origin(), normal_2d());
        sketch.add_entity(SketchEntity::point(0.0, 0.0, 0.0));
        sketch.add_entity(SketchEntity::point(5.0, 5.0, 0.0));
        fix_point(&mut sketch, 0, 0);
        sketch.add_constraint(SketchConstraint {
            id: uuid::Uuid::new_v4().to_string(),
            constraint_type: SketchConstraintType::Coincident,
            entity_refs: vec![
                SketchPointRef { entity_index: 0, point_index: 0 },
                SketchPointRef { entity_index: 1, point_index: 0 },
            ],
            value: 0.0,
        });
        let result = solve(&mut sketch);
        assert!(result.success);
        let p0 = sketch.get_solved_point(0, 0).unwrap();
        let p1 = sketch.get_solved_point(1, 0).unwrap();
        assert!((p0.x - p1.x).abs() < 1e-6);
        assert!((p0.y - p1.y).abs() < 1e-6);
    }

    #[test]
    fn test_solve_line_with_distance() {
        let mut sketch = Sketch::new("s", origin(), normal_2d());
        let _ = sketch.add_entity(SketchEntity::line(pt(0.0, 0.0, 0.0), pt(5.0, 0.0, 0.0)));
        fix_point(&mut sketch, 0, 0);
        sketch.add_constraint(SketchConstraint {
            id: uuid::Uuid::new_v4().to_string(),
            constraint_type: SketchConstraintType::Distance,
            entity_refs: vec![
                SketchPointRef { entity_index: 0, point_index: 0 },
                SketchPointRef { entity_index: 0, point_index: 1 },
            ],
            value: 10.0,
        });
        let result = solve(&mut sketch);
        assert!(result.success);
        let p0 = sketch.get_solved_point(0, 0).unwrap();
        assert!((p0.x).abs() < 1e-6);
        assert!((p0.y).abs() < 1e-6);
        let p1 = sketch.get_solved_point(0, 1).unwrap();
        let dist = to_slvs_distance(p0, p1);
        assert!((dist - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_solve_circle() {
        let mut sketch = Sketch::new("s", origin(), normal_2d());
        let _ = sketch.add_entity(SketchEntity::circle(pt(0.0, 0.0, 0.0), 5.0));
        fix_point(&mut sketch, 0, 0);
        let result = solve(&mut sketch);
        assert!(result.success);
    }

    #[test]
    fn test_normal_to_quaternion() {
        let q = normal_to_quaternion(0.0, 0.0, 1.0);
        let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
        assert!((len - 1.0).abs() < 1e-10);

        let q = normal_to_quaternion(1.0, 0.0, 0.0);
        let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
        assert!((len - 1.0).abs() < 1e-10);

        let q = normal_to_quaternion(0.0, 1.0, 0.0);
        let len = (q[0] * q[0] + q[1] * q[1] + q[2] * q[2] + q[3] * q[3]).sqrt();
        assert!((len - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_solve_horizontal_line() {
        let mut sketch = Sketch::new("s", origin(), normal_2d());
        let _ = sketch.add_entity(SketchEntity::line(pt(0.0, 0.0, 0.0), pt(3.0, 5.0, 0.0)));
        fix_point(&mut sketch, 0, 0);
        sketch.add_constraint(SketchConstraint {
            id: uuid::Uuid::new_v4().to_string(),
            constraint_type: SketchConstraintType::Horizontal,
            entity_refs: vec![
                SketchPointRef { entity_index: 0, point_index: 0 },
                SketchPointRef { entity_index: 0, point_index: 1 },
            ],
            value: 0.0,
        });
        let result = solve(&mut sketch);
        assert!(result.success);
        let p0 = sketch.get_solved_point(0, 0).unwrap();
        let p1 = sketch.get_solved_point(0, 1).unwrap();
        assert!((p0.y - p1.y).abs() < 1e-3);
    }

    #[test]
    fn test_solve_vertical_line() {
        let mut sketch = Sketch::new("s", origin(), normal_2d());
        let _ = sketch.add_entity(SketchEntity::line(pt(0.0, 0.0, 0.0), pt(5.0, 3.0, 0.0)));
        fix_point(&mut sketch, 0, 0);
        sketch.add_constraint(SketchConstraint {
            id: uuid::Uuid::new_v4().to_string(),
            constraint_type: SketchConstraintType::Vertical,
            entity_refs: vec![
                SketchPointRef { entity_index: 0, point_index: 0 },
                SketchPointRef { entity_index: 0, point_index: 1 },
            ],
            value: 0.0,
        });
        let result = solve(&mut sketch);
        assert!(result.success);
        let p0 = sketch.get_solved_point(0, 0).unwrap();
        let p1 = sketch.get_solved_point(0, 1).unwrap();
        assert!((p0.x - p1.x).abs() < 1e-3);
    }

    #[test]
    fn test_solve_parallel_lines() {
        let mut sketch = Sketch::new("s", origin(), normal_2d());
        let _ = sketch.add_entity(SketchEntity::line(pt(0.0, 0.0, 0.0), pt(5.0, 0.0, 0.0)));
        let _ = sketch.add_entity(SketchEntity::line(pt(0.0, 5.0, 0.0), pt(5.0, 3.0, 0.0)));
        fix_point(&mut sketch, 0, 0);
        fix_point(&mut sketch, 1, 0);
        sketch.add_constraint(SketchConstraint {
            id: uuid::Uuid::new_v4().to_string(),
            constraint_type: SketchConstraintType::Parallel,
            entity_refs: vec![
                SketchPointRef { entity_index: 0, point_index: 0 },
                SketchPointRef { entity_index: 1, point_index: 0 },
            ],
            value: 0.0,
        });
        let result = solve(&mut sketch);
        assert!(result.success);
    }

    #[test]
    fn test_solve_perpendicular_lines() {
        let mut sketch = Sketch::new("s", origin(), normal_2d());
        let _ = sketch.add_entity(SketchEntity::line(pt(0.0, 0.0, 0.0), pt(5.0, 0.0, 0.0)));
        let _ = sketch.add_entity(SketchEntity::line(pt(0.0, 0.0, 0.0), pt(0.0, 5.0, 0.0)));
        fix_point(&mut sketch, 0, 0);
        sketch.add_constraint(SketchConstraint {
            id: uuid::Uuid::new_v4().to_string(),
            constraint_type: SketchConstraintType::Perpendicular,
            entity_refs: vec![
                SketchPointRef { entity_index: 0, point_index: 0 },
                SketchPointRef { entity_index: 1, point_index: 0 },
            ],
            value: 0.0,
        });
        let result = solve(&mut sketch);
        assert!(result.success);
    }

    #[test]
    fn test_solve_diameter() {
        let mut sketch = Sketch::new("s", origin(), normal_2d());
        let _ = sketch.add_entity(SketchEntity::circle(pt(0.0, 0.0, 0.0), 5.0));
        fix_point(&mut sketch, 0, 0);
        sketch.add_constraint(SketchConstraint {
            id: uuid::Uuid::new_v4().to_string(),
            constraint_type: SketchConstraintType::Diameter,
            entity_refs: vec![SketchPointRef { entity_index: 0, point_index: 0 }],
            value: 20.0,
        });
        let result = solve(&mut sketch);
        assert!(result.success);
    }

    #[test]
    fn test_solve_radius() {
        let mut sketch = Sketch::new("s", origin(), normal_2d());
        let _ = sketch.add_entity(SketchEntity::circle(pt(0.0, 0.0, 0.0), 5.0));
        fix_point(&mut sketch, 0, 0);
        sketch.add_constraint(SketchConstraint {
            id: uuid::Uuid::new_v4().to_string(),
            constraint_type: SketchConstraintType::Radius,
            entity_refs: vec![SketchPointRef { entity_index: 0, point_index: 0 }],
            value: 7.5,
        });
        let result = solve(&mut sketch);
        assert!(result.success);
    }
}
