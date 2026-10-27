use crate::{Sketch, SketchEntityType};
use omnim_core as c;

/// Extract a closed 2D profile from a solved sketch.
///
/// Takes the solved point positions and converts them into a list of 2D points
/// representing the sketch profile. Only supports line-based profiles for now.
///
/// Returns `None` if the sketch is not solved, has no entities, or the profile
/// cannot be extracted (e.g. contains only circles/arcs with no line segments).
pub fn sketch_to_profile_2d(sketch: &Sketch) -> Option<Vec<c::Point2D>> {
    if !sketch.solved || sketch.entities.is_empty() {
        return None;
    }

    let mut profile_points: Vec<c::Point2D> = Vec::new();

    for entity in &sketch.entities {
        match entity.entity_type {
            SketchEntityType::Line => {
                // Line has 2 points: start (index 0) and end (index 1)
                if let Some(start) = get_solved_point(sketch, &profile_points, entity, 0) {
                    profile_points.push(start);
                }
                if let Some(end) = get_solved_point(sketch, &profile_points, entity, 1) {
                    profile_points.push(end);
                }
            }
            SketchEntityType::Point => {
                // Single point entities are ignored for profile extraction
            }
            SketchEntityType::Circle | SketchEntityType::Arc => {
                // Tessellate circles/arcs into line segments
                if let Some(tessellated) = tessellate_curve(sketch, entity) {
                    for pt in tessellated {
                        profile_points.push(pt);
                    }
                }
            }
            SketchEntityType::Rectangle => {
                if entity.points.len() >= 2 {
                    let min_pt = &entity.points[0];
                    let max_pt = &entity.points[1];
                    let (x0, y0) = (min_pt.x.min(max_pt.x), min_pt.y.min(max_pt.y));
                    let (x1, y1) = (min_pt.x.max(max_pt.x), min_pt.y.max(max_pt.y));
                    profile_points.push(c::Point2D { x: x0, y: y0 });
                    profile_points.push(c::Point2D { x: x1, y: y0 });
                    profile_points.push(c::Point2D { x: x1, y: y1 });
                    profile_points.push(c::Point2D { x: x0, y: y1 });
                }
            }
            SketchEntityType::Polygon => {
                for pt in &entity.points {
                    profile_points.push(c::Point2D { x: pt.x, y: pt.y });
                }
            }
            SketchEntityType::Ellipse | SketchEntityType::Spline | SketchEntityType::Polyline => {
                for pt in &entity.points {
                    profile_points.push(c::Point2D { x: pt.x, y: pt.y });
                }
            }
        }
    }

    if profile_points.is_empty() {
        return None;
    }

    // Remove consecutive duplicate points (end of one entity = start of next)
    let deduped = deduplicate_consecutive(profile_points);

    // Verify closedness: first and last points should be close
    if deduped.len() >= 3 {
        let first = &deduped[0];
        let last = &deduped[deduped.len() - 1];
        let dx = first.x - last.x;
        let dy = first.y - last.y;
        if (dx * dx + dy * dy).sqrt() < 1e-6 {
            // Closed: remove the duplicate last point
            Some(deduped[..deduped.len() - 1].to_vec())
        } else {
            // Not closed — still return the profile (extrude will close it)
            Some(deduped)
        }
    } else {
        Some(deduped)
    }
}

/// Get a solved point from the sketch, projected to 2D.
fn get_solved_point(
    sketch: &Sketch,
    _existing: &[c::Point2D],
    entity: &crate::SketchEntity,
    point_index: usize,
) -> Option<c::Point2D> {
    let entity_idx = sketch.entities.iter().position(|e| e.id == entity.id)?;
    let pt3d = sketch.get_solved_point(entity_idx, point_index)?;
    Some(c::Point2D { x: pt3d.x, y: pt3d.y })
}

/// Tessellate a circle or arc into line segments.
fn tessellate_curve(sketch: &Sketch, entity: &crate::SketchEntity) -> Option<Vec<c::Point2D>> {
    let entity_idx = sketch.entities.iter().position(|e| e.id == entity.id)?;

    match entity.entity_type {
        SketchEntityType::Circle => {
            let center = sketch.get_solved_point(entity_idx, 0)?;
            let radius = entity.radius.max(0.001);
            let segments = 32;
            let mut pts = Vec::with_capacity(segments);
            for i in 0..segments {
                let angle = 2.0 * std::f64::consts::PI * i as f64 / segments as f64;
                pts.push(c::Point2D {
                    x: center.x + radius * angle.cos(),
                    y: center.y + radius * angle.sin(),
                });
            }
            Some(pts)
        }
        SketchEntityType::Arc => {
            let center = sketch.get_solved_point(entity_idx, 0)?;
            let start = sketch.get_solved_point(entity_idx, 1)?;
            let end = sketch.get_solved_point(entity_idx, 2)?;

            let radius = ((start.x - center.x).powi(2) + (start.y - center.y).powi(2)).sqrt();
            if radius < 1e-10 {
                return None;
            }

            let start_angle = (start.y - center.y).atan2(start.x - center.x);
            let end_angle = (end.y - center.y).atan2(end.x - center.x);

            let mut total_angle = end_angle - start_angle;
            if total_angle <= 0.0 {
                total_angle += 2.0 * std::f64::consts::PI;
            }

            let segments = 16;
            let mut pts = Vec::with_capacity(segments + 1);
            for i in 0..=segments {
                let angle = start_angle + total_angle * i as f64 / segments as f64;
                pts.push(c::Point2D {
                    x: center.x + radius * angle.cos(),
                    y: center.y + radius * angle.sin(),
                });
            }
            Some(pts)
        }
        _ => None,
    }
}

/// Remove consecutive duplicate points.
fn deduplicate_consecutive(points: Vec<c::Point2D>) -> Vec<c::Point2D> {
    if points.is_empty() {
        return points;
    }
    let mut result = vec![points[0]];
    for pt in &points[1..] {
        let last = result.last().unwrap();
        let dx = pt.x - last.x;
        let dy = pt.y - last.y;
        if (dx * dx + dy * dy).sqrt() > 1e-6 {
            result.push(*pt);
        }
    }
    result
}

/// Convert a 2D profile to a3D profile with a given z coordinate.
pub fn profile_to_3d(profile2d: &[c::Point2D], z: f64) -> Vec<c::Point3D> {
    profile2d.iter().map(|p| c::Point3D { x: p.x, y: p.y, z }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SketchEntity, Sketch};

    fn origin() -> c::Point3D {
        c::Point3D { x: 0.0, y: 0.0, z: 0.0 }
    }

    fn normal_2d() -> c::Vector3D {
        c::Vector3D { x: 0.0, y: 0.0, z: 1.0 }
    }

    #[test]
    fn test_profile_empty_sketch() {
        let sketch = Sketch::new("empty", origin(), normal_2d());
        assert!(sketch_to_profile_2d(&sketch).is_none());
    }

    #[test]
    fn test_profile_unsolved_sketch() {
        let mut sketch = Sketch::new("s", origin(), normal_2d());
        sketch.add_entity(SketchEntity::line(
            c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
            c::Point3D { x: 1.0, y: 0.0, z: 0.0 },
        ));
        assert!(sketch_to_profile_2d(&sketch).is_none());
    }

    #[test]
    fn test_profile_triangle() {
        let mut sketch = Sketch::new("tri", origin(), normal_2d());
        // Triangle: (0,0) → (1,0) → (0,1) → (0,0)
        sketch.add_entity(SketchEntity::line(
            c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
            c::Point3D { x: 1.0, y: 0.0, z: 0.0 },
        ));
        sketch.add_entity(SketchEntity::line(
            c::Point3D { x: 1.0, y: 0.0, z: 0.0 },
            c::Point3D { x: 0.0, y: 1.0, z: 0.0 },
        ));
        sketch.add_entity(SketchEntity::line(
            c::Point3D { x: 0.0, y: 1.0, z: 0.0 },
            c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
        ));

        // Mark as solved (manually set solved_points)
        sketch.solved = true;
        sketch.set_solved_point(0, 0, c::Point3D { x: 0.0, y: 0.0, z: 0.0 });
        sketch.set_solved_point(0, 1, c::Point3D { x: 1.0, y: 0.0, z: 0.0 });
        sketch.set_solved_point(1, 0, c::Point3D { x: 1.0, y: 0.0, z: 0.0 });
        sketch.set_solved_point(1, 1, c::Point3D { x: 0.0, y: 1.0, z: 0.0 });
        sketch.set_solved_point(2, 0, c::Point3D { x: 0.0, y: 1.0, z: 0.0 });
        sketch.set_solved_point(2, 1, c::Point3D { x: 0.0, y: 0.0, z: 0.0 });

        let profile = sketch_to_profile_2d(&sketch).unwrap();
        assert_eq!(profile.len(), 3);
        // Check points (deduped: closed last point removed)
        assert!((profile[0].x - 0.0).abs() < 1e-6);
        assert!((profile[0].y - 0.0).abs() < 1e-6);
        assert!((profile[1].x - 1.0).abs() < 1e-6);
        assert!((profile[1].y - 0.0).abs() < 1e-6);
        assert!((profile[2].x - 0.0).abs() < 1e-6);
        assert!((profile[2].y - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_profile_square() {
        let mut sketch = Sketch::new("sq", origin(), normal_2d());
        sketch.add_entity(SketchEntity::line(
            c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
            c::Point3D { x: 2.0, y: 0.0, z: 0.0 },
        ));
        sketch.add_entity(SketchEntity::line(
            c::Point3D { x: 2.0, y: 0.0, z: 0.0 },
            c::Point3D { x: 2.0, y: 2.0, z: 0.0 },
        ));
        sketch.add_entity(SketchEntity::line(
            c::Point3D { x: 2.0, y: 2.0, z: 0.0 },
            c::Point3D { x: 0.0, y: 2.0, z: 0.0 },
        ));
        sketch.add_entity(SketchEntity::line(
            c::Point3D { x: 0.0, y: 2.0, z: 0.0 },
            c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
        ));

        sketch.solved = true;
        sketch.set_solved_point(0, 0, c::Point3D { x: 0.0, y: 0.0, z: 0.0 });
        sketch.set_solved_point(0, 1, c::Point3D { x: 2.0, y: 0.0, z: 0.0 });
        sketch.set_solved_point(1, 0, c::Point3D { x: 2.0, y: 0.0, z: 0.0 });
        sketch.set_solved_point(1, 1, c::Point3D { x: 2.0, y: 2.0, z: 0.0 });
        sketch.set_solved_point(2, 0, c::Point3D { x: 2.0, y: 2.0, z: 0.0 });
        sketch.set_solved_point(2, 1, c::Point3D { x: 0.0, y: 2.0, z: 0.0 });
        sketch.set_solved_point(3, 0, c::Point3D { x: 0.0, y: 2.0, z: 0.0 });
        sketch.set_solved_point(3, 1, c::Point3D { x: 0.0, y: 0.0, z: 0.0 });

        let profile = sketch_to_profile_2d(&sketch).unwrap();
        assert_eq!(profile.len(), 4);
    }

    #[test]
    fn test_profile_to_3d() {
        let pts2d = vec![
            c::Point2D { x: 0.0, y: 0.0 },
            c::Point2D { x: 1.0, y: 0.0 },
            c::Point2D { x: 0.0, y: 1.0 },
        ];
        let pts3d = profile_to_3d(&pts2d, 5.0);
        assert_eq!(pts3d.len(), 3);
        assert!((pts3d[0].z - 5.0).abs() < 1e-10);
        assert!((pts3d[1].x - 1.0).abs() < 1e-10);
    }
}
