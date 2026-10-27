use crate::profile::ProfilePoints;
use omnim_core as c;

/// Generate an I-section profile (IPE/HEA-style).
///
/// Returns a closed polygon in the XY plane (z=0) tracing the perimeter
/// of an I-shaped cross-section.
///
/// # Arguments
/// * `height` - total height of the I-section
/// * `width` - flange width
/// * `web_thickness` - thickness of the web
/// * `flange_thickness` - thickness of each flange
pub fn i_section(
    height: f64,
    width: f64,
    web_thickness: f64,
    flange_thickness: f64,
) -> ProfilePoints {
    let hh = height / 2.0;
    let hw = width / 2.0;
    let tw2 = web_thickness / 2.0;
    let tf = flange_thickness;
    let y1 = hh;                    // top flange top
    let y2 = hh - tf;               // top flange bottom
    let y3 = -hh + tf;              // bottom flange top
    let y4 = -hh;                   // bottom flange bottom

    vec![
        c::Point3D { x: -hw, y: y1, z: 0.0 },
        c::Point3D { x:  hw, y: y1, z: 0.0 },
        c::Point3D { x:  hw, y: y2, z: 0.0 },
        c::Point3D { x:  tw2, y: y2, z: 0.0 },
        c::Point3D { x:  tw2, y: y3, z: 0.0 },
        c::Point3D { x:  hw, y: y3, z: 0.0 },
        c::Point3D { x:  hw, y: y4, z: 0.0 },
        c::Point3D { x: -hw, y: y4, z: 0.0 },
        c::Point3D { x: -hw, y: y3, z: 0.0 },
        c::Point3D { x: -tw2, y: y3, z: 0.0 },
        c::Point3D { x: -tw2, y: y2, z: 0.0 },
        c::Point3D { x: -hw, y: y2, z: 0.0 },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i_section_basic() {
        let pts = i_section(300.0, 150.0, 8.0, 12.0);
        assert_eq!(pts.len(), 12);
    }

    #[test]
    fn test_i_section_aspect() {
        // IPE 240: h=240, w=120, tw=6.2, tf=9.8
        let pts = i_section(240.0, 120.0, 6.2, 9.8);
        let top_y = pts[0].y;
        let bot_y = pts[6].y;
        assert!((top_y - 120.0).abs() < 1e-6);
        assert!((bot_y + 120.0).abs() < 1e-6);
    }

    #[test]
    fn test_i_section_minimal() {
        let pts = i_section(80.0, 46.0, 1.9, 3.8);
        assert_eq!(pts.len(), 12);
    }
}