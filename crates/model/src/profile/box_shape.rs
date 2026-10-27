use crate::profile::ProfilePoints;
use omnim_core as c;

/// Generate a box/rectangular hollow section profile.
pub fn box_section(width: f64, height: f64, thickness: f64) -> ProfilePoints {
    let hw = width / 2.0;
    let hh = height / 2.0;
    let t = thickness;
    vec![
        c::Point3D { x: -hw, y:  hh, z: 0.0 },
        c::Point3D { x:  hw, y:  hh, z: 0.0 },
        c::Point3D { x:  hw, y:  hh - t, z: 0.0 },
        c::Point3D { x:  hw - t, y:  hh - t, z: 0.0 },
        c::Point3D { x:  hw - t, y: -hh + t, z: 0.0 },
        c::Point3D { x:  hw, y: -hh + t, z: 0.0 },
        c::Point3D { x:  hw, y: -hh, z: 0.0 },
        c::Point3D { x: -hw, y: -hh, z: 0.0 },
        c::Point3D { x: -hw, y: -hh + t, z: 0.0 },
        c::Point3D { x: -hw + t, y: -hh + t, z: 0.0 },
        c::Point3D { x: -hw + t, y:  hh - t, z: 0.0 },
        c::Point3D { x: -hw, y:  hh - t, z: 0.0 },
    ]
}

/// Generate a solid rectangular profile.
pub fn rectangle(width: f64, height: f64) -> ProfilePoints {
    let hw = width / 2.0;
    let hh = height / 2.0;
    vec![
        c::Point3D { x: -hw, y: -hh, z: 0.0 },
        c::Point3D { x:  hw, y: -hh, z: 0.0 },
        c::Point3D { x:  hw, y:  hh, z: 0.0 },
        c::Point3D { x: -hw, y:  hh, z: 0.0 },
    ]
}

/// Generate an L-shaped angle profile.
pub fn l_section(leg1: f64, leg2: f64, thickness: f64) -> ProfilePoints {
    let t = thickness;
    vec![
        c::Point3D { x: 0.0, y: 0.0, z: 0.0 },
        c::Point3D { x: leg1, y: 0.0, z: 0.0 },
        c::Point3D { x: leg1, y: t, z: 0.0 },
        c::Point3D { x: t, y: t, z: 0.0 },
        c::Point3D { x: t, y: leg2, z: 0.0 },
        c::Point3D { x: 0.0, y: leg2, z: 0.0 },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_section_basic() {
        let pts = box_section(100.0, 100.0, 5.0);
        assert_eq!(pts.len(), 12);
    }

    #[test]
    fn test_rectangle_basic() {
        let pts = rectangle(200.0, 300.0);
        assert_eq!(pts.len(), 4);
        assert!((pts[0].x + 100.0).abs() < 1e-10);
        assert!((pts[0].y + 150.0).abs() < 1e-10);
        assert!((pts[2].x - 100.0).abs() < 1e-10);
        assert!((pts[2].y - 150.0).abs() < 1e-10);
    }

    #[test]
    fn test_l_section_basic() {
        let pts = l_section(50.0, 50.0, 5.0);
        assert_eq!(pts.len(), 6);
    }

    #[test]
    fn test_box_section_chs() {
        let pts = box_section(60.0, 60.0, 3.0);
        assert_eq!(pts.len(), 12);
    }
}