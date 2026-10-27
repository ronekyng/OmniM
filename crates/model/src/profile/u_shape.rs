use crate::profile::ProfilePoints;
use omnim_core as c;

/// Generate a U-channel (UPE/UPN) profile.
///
/// Returns a closed polygon in the XY plane (z=0) tracing the perimeter
/// of a U-shaped cross-section (channel).
pub fn u_section(height: f64, width: f64, web_thickness: f64, flange_thickness: f64) -> ProfilePoints {
    let hh = height / 2.0;
    let hw = width;
    let tw = web_thickness;
    let tf = flange_thickness;
    vec![
        c::Point3D { x: 0.0, y:  hh, z: 0.0 },
        c::Point3D { x:  hw, y:  hh, z: 0.0 },
        c::Point3D { x:  hw, y:  hh - tf, z: 0.0 },
        c::Point3D { x:  tw, y:  hh - tf, z: 0.0 },
        c::Point3D { x:  tw, y: -hh + tf, z: 0.0 },
        c::Point3D { x:  hw, y: -hh + tf, z: 0.0 },
        c::Point3D { x:  hw, y: -hh, z: 0.0 },
        c::Point3D { x: 0.0, y: -hh, z: 0.0 },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u_section_basic() {
        let pts = u_section(100.0, 50.0, 5.0, 8.0);
        assert_eq!(pts.len(), 8);
    }

    #[test]
    fn test_u_section_dimensions() {
        let pts = u_section(200.0, 80.0, 6.0, 10.0);
        assert!((pts[0].y - 100.0).abs() < 1e-10);
        assert!((pts[7].y + 100.0).abs() < 1e-10);
        assert!((pts[1].x - 80.0).abs() < 1e-10);
    }
}