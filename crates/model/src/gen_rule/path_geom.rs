//! Shared pure path geometry — lives **below** the Genesis classification.
//!
//! Contract: [DATA_MODEL §1](../../../../docs/specification/DATA_MODEL.md) —
//! "a wall's LayerExtrusion generator may share the low-level 'constant section
//! swept along a path' primitive with LinearComponent; that primitive carries no
//! class semantics and lives below classification". Everything in this module is
//! therefore class-agnostic: it knows about points, segments, closure and
//! validity, and nothing about walls, pipes or terrain targets.
//!
//! Dispatch rule: `closed == false` → open polyline semantics (≥2 points,
//! collinear chains are legal — a straight wall is still a path);
//! `closed == true` → polygon semantics (≥3 points, degenerate area and
//! self-intersections are rejected).

use omnim_core::Point2D;

/// Minimum segment length considered non-degenerate (metres).
const EPS: f64 = 1e-9;

pub fn seg_len(a: &Point2D, b: &Point2D) -> f64 {
    ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt()
}

/// Iterate the polyline's segments; for closed paths the closing segment
/// `last → first` is included, and a duplicated closing point is tolerated.
pub fn iter_segments(points: &[Point2D], closed: bool) -> Vec<(Point2D, Point2D)> {
    let pts = if closed {
        normalize_closed(points)
    } else {
        points.to_vec()
    };
    let mut out = Vec::new();
    if pts.len() < 2 {
        return out;
    }
    for w in pts.windows(2) {
        out.push((w[0], w[1]));
    }
    if closed && pts.len() > 2 {
        out.push((pts[pts.len() - 1], pts[0]));
    }
    out
}

/// For closed inputs, drop a trailing point that duplicates the first one so
/// generators never emit a zero-length closing segment.
pub fn normalize_closed(points: &[Point2D]) -> Vec<Point2D> {
    if points.len() >= 2 && seg_len(&points[0], &points[points.len() - 1]) < EPS {
        points[..points.len() - 1].to_vec()
    } else {
        points.to_vec()
    }
}

/// Validate a path by closure. Open paths need ≥ `min` points and no
/// zero-length edges; closed paths additionally go through the polygon checks
/// (≥3 points, non-degenerate area, no self-intersection).
pub fn validate_path(points: &[Point2D], closed: bool, min: usize) -> Result<(), String> {
    let n = if closed {
        normalize_closed(points).len()
    } else {
        points.len()
    };
    if n < min {
        return Err(if closed {
            format!("closed path requires at least {} points, got {}", min, n)
        } else {
            format!("path requires at least {} points, got {}", min, n)
        });
    }
    for (a, b) in iter_segments(points, closed) {
        if seg_len(&a, &b) < EPS {
            return Err("path has a zero-length segment".into());
        }
    }
    if closed {
        crate::gen_rule::layer_extrusion::validate_boundary(&normalize_closed(points))?;
    }
    Ok(())
}

/// C3 collinear merge: drop intermediate points that are collinear (|cross
/// product| < 1e-9) with their neighbours and duplicate points. Class-agnostic
/// preprocessing shared by open paths and closed boundaries.
pub fn merge_collinear(points: &[Point2D]) -> Vec<Point2D> {
    if points.len() < 3 {
        return points.to_vec();
    }
    let mut out: Vec<Point2D> = Vec::with_capacity(points.len());
    out.push(points[0]);
    for p in &points[1..] {
        if seg_len(out.last().unwrap(), p) < EPS {
            continue;
        }
        if out.len() >= 2 {
            let a = &out[out.len() - 2];
            let b = out.last().unwrap();
            let cr = (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
            if cr.abs() < 1e-9 {
                let ab = (b.x - a.x) * (p.x - a.x) + (b.y - a.y) * (p.y - a.y);
                if ab > 0.0 {
                    *out.last_mut().unwrap() = *p;
                    continue;
                }
            }
        }
        out.push(*p);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(x: f64, y: f64) -> Point2D {
        Point2D { x, y }
    }

    #[test]
    fn iter_segments_open_and_closed() {
        let pts = vec![p(0.0, 0.0), p(1.0, 0.0), p(1.0, 1.0)];
        assert_eq!(iter_segments(&pts, false).len(), 2);
        assert_eq!(iter_segments(&pts, true).len(), 3);
    }

    #[test]
    fn closed_normalization_drops_duplicate_closing_point() {
        let pts = vec![p(0.0, 0.0), p(1.0, 0.0), p(1.0, 1.0), p(0.0, 0.0)];
        assert_eq!(normalize_closed(&pts).len(), 3);
        assert_eq!(iter_segments(&pts, true).len(), 3);
    }

    #[test]
    fn open_path_allows_collinear_chain() {
        let pts = vec![p(0.0, 0.0), p(1.0, 0.0), p(2.0, 0.0)];
        assert!(validate_path(&pts, false, 2).is_ok());
    }

    #[test]
    fn open_path_rejects_too_few_points_and_zero_length() {
        assert!(validate_path(&[p(0.0, 0.0)], false, 2).is_err());
        let dup = vec![p(0.0, 0.0), p(0.0, 0.0), p(1.0, 0.0)];
        assert!(validate_path(&dup, false, 2).is_err());
    }

    #[test]
    fn merge_collinear_removes_midpoints_and_duplicates() {
        let pts = vec![p(0.0, 0.0), p(1.0, 0.0), p(2.0, 0.0), p(2.0, 1.0)];
        let merged = merge_collinear(&pts);
        assert_eq!(merged, vec![p(0.0, 0.0), p(2.0, 0.0), p(2.0, 1.0)]);
        let dup = vec![p(0.0, 0.0), p(0.0, 0.0), p(1.0, 1.0)];
        assert_eq!(merge_collinear(&dup), vec![p(0.0, 0.0), p(1.0, 1.0)]);
    }

    #[test]
    fn closed_path_rejects_self_intersection_and_degenerate_area() {
        let bowtie = vec![p(0.0, 0.0), p(1.0, 1.0), p(1.0, 0.0), p(0.0, 1.0)];
        assert!(validate_path(&bowtie, true, 3).is_err());
        let line = vec![p(0.0, 0.0), p(1.0, 0.0), p(2.0, 0.0)];
        assert!(validate_path(&line, true, 3).is_err());
    }
}
