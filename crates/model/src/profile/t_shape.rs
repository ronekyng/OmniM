use crate::profile::ProfilePoints;
use omnim_core as c;

/// Generate a T-section profile.
///
/// Returns a closed polygon in the XY plane (z=0) tracing the perimeter
/// of a T-shaped cross-section (typically for T-bars).
pub fn t_section(height: f64, width: f64, web_thickness: f64, flange_thickness: f64) -> ProfilePoints {
    let hh = height / 2.0;
    let hw = width / 2.0;
    let tw2 = web_thickness / 2.0;
    let tf = flange_thickness;
    let y1 = hh;
    let y2 = hh - tf;
    let y3 = -hh;
    vec![
        c::Point3D { x: -hw, y: y1, z: 0.0 },
        c::Point3D { x:  hw, y: y1, z: 0.0 },
        c::Point3D { x:  hw, y: y2, z: 0.0 },
        c::Point3D { x:  tw2, y: y2, z: 0.0 },
        c::Point3D { x:  tw2, y: y3, z: 0.0 },
        c::Point3D { x: -tw2, y: y3, z: 0.0 },
        c::Point3D { x: -tw2, y: y2, z: 0.0 },
        c::Point3D { x: -hw, y: y2, z: 0.0 },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_t_section_basic() {
        let pts = t_section(100.0, 80.0, 6.0, 8.0);
        assert_eq!(pts.len(), 8);
    }

    #[test]
    fn test_t_section_dimensions() {
        let pts = t_section(200.0, 100.0, 8.0, 12.0);
        assert!((pts[0].y - 100.0).abs() < 1e-10);
        assert!((pts[5].y + 100.0).abs() < 1e-10);
    }
}