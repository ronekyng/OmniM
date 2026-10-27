use crate::profile::ProfilePoints;
use omnim_core as c;

/// Generate a circular profile tessellated into `segments` line segments.
pub fn circle(radius: f64, segments: usize) -> ProfilePoints {
    let n = segments.max(8);
    (0..n)
        .map(|i| {
            let angle: f64 = 2.0 * std::f64::consts::PI * i as f64 / n as f64;
            c::Point3D {
                x: radius * angle.cos(),
                y: radius * angle.sin(),
                z: 0.0,
            }
        })
        .collect()
}

/// Generate an annular profile (outer circle minus inner circle).
///
/// Produces a closed polygon that traces the outer circle then the inner
/// circle in reverse direction, creating a "figure-8" shape suitable for
/// extrusion as a hollow tube.
pub fn annulus(outer_radius: f64, inner_radius: f64, segments: usize) -> ProfilePoints {
    let n = segments.max(8);
    // Half the segments for outer, half for inner (to keep vertex count reasonable)
    let n_outer = n;
    let n_inner = (n as f64 * (inner_radius / outer_radius).max(0.25)) as usize;

    // Create bridge points between outer and inner loops
    let bridge_angle: f64 = 0.0;
    let bridge_outer = c::Point3D {
        x: outer_radius * bridge_angle.cos(),
        y: outer_radius * bridge_angle.sin(),
        z: 0.0,
    };
    let bridge_inner = c::Point3D {
        x: inner_radius * bridge_angle.cos(),
        y: inner_radius * bridge_angle.sin(),
        z: 0.0,
    };

    let mut pts = Vec::new();

    // Outer circle (clockwise)
    for i in 0..n_outer {
        let angle = 2.0 * std::f64::consts::PI * i as f64 / n_outer as f64;
        pts.push(c::Point3D {
            x: outer_radius * angle.cos(),
            y: outer_radius * angle.sin(),
            z: 0.0,
        });
    }

    // Bridge to inner circle
    pts.push(bridge_outer);
    pts.push(bridge_inner);

    // Inner circle (counter-clockwise to form the hole)
    for i in 0..n_inner {
        let angle = 2.0 * std::f64::consts::PI * (n_inner - i) as f64 / n_inner as f64;
        pts.push(c::Point3D {
            x: inner_radius * angle.cos(),
            y: inner_radius * angle.sin(),
            z: 0.0,
        });
    }

    // Bridge back to outer circle
    pts.push(bridge_inner);
    pts.push(bridge_outer);

    pts
}

/// Generate a simple circle profile for solid pipe/bar.
pub fn solid_circle(radius: f64, segments: usize) -> ProfilePoints {
    circle(radius, segments)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circle_basic() {
        let pts = circle(50.0, 32);
        assert_eq!(pts.len(), 32);
    }

    #[test]
    fn test_circle_radius() {
        let pts = circle(100.0, 64);
        for p in &pts {
            let r = (p.x.powi(2) + p.y.powi(2)).sqrt();
            assert!((r - 100.0).abs() < 0.1);
        }
    }

    #[test]
    fn test_annulus_basic() {
        let pts = annulus(50.0, 40.0, 32);
        assert!(pts.len() > 32);
        let d = ((pts[0].x - pts[pts.len() - 1].x).powi(2)
            + (pts[0].y - pts[pts.len() - 1].y).powi(2)).sqrt();
        assert!(d < 1e-10);
    }

    #[test]
    fn test_solid_circle() {
        let pts = solid_circle(25.0, 16);
        assert_eq!(pts.len(), 16);
    }
}