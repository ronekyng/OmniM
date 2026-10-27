pub mod i_shape;
pub mod box_shape;
pub mod circular;
pub mod pipe_schedule;
pub mod u_shape;
pub mod t_shape;

use omnim_core as c;
use serde::{Deserialize, Serialize};

pub type ProfilePoints = Vec<c::Point3D>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ProfileType {
    IShape { height: f64, width: f64, web_thickness: f64, flange_thickness: f64 },
    BoxShape { width: f64, height: f64, thickness: f64 },
    Circle { radius: f64, segments: usize },
    Annulus { outer_radius: f64, inner_radius: f64, segments: usize },
    Rectangle { width: f64, height: f64 },
    LShape { leg1: f64, leg2: f64, thickness: f64 },
    UShape { height: f64, width: f64, web_thickness: f64, flange_thickness: f64 },
    TShape { height: f64, width: f64, web_thickness: f64, flange_thickness: f64 },
    PipeSchedule { segments: usize },
    /// Rectangle whose width/height are resolved from component params at
    /// instantiation time (enables 2-point wall/slab creation).
    ParametricRect { width_param: String, height_param: String },
}

pub fn resolve_pipe_schedule_profile(dn: f64, schedule: f64, segments: usize) -> ProfilePoints {
    let schedule_str = if schedule.abs() < 1e-6 {
        "40"
    } else if (schedule - 5.0).abs() < 1e-6 {
        "5S"
    } else if (schedule - 10.0).abs() < 1e-6 {
        "10S"
    } else {
        // Use schedule as-is (will match "40", "80", "160" etc.)
        // Mapped via a static lookup; we round to the nearest integer.
        let s_int = schedule.round() as i32;
        if pipe_schedule::wall_thickness(dn, &s_int.to_string()).is_some() {
            // schedule string is just the number
            return resolve_pipe_schedule_inner(dn, s_int.to_string(), segments);
        } else {
            "40"
        }
    };
    resolve_pipe_schedule_inner(dn, schedule_str.to_string(), segments)
}

/// Resolve a stainless-steel pipe schedule profile (ASME B36.19M).
/// Takes the DN and schedule (5S/10S/40S/80S) and returns the annulus profile.
pub fn resolve_pipe_schedule_profile_ss(dn: f64, schedule: f64, segments: usize) -> ProfilePoints {
    let schedule_str = if schedule.abs() < 1e-6 {
        "40S".to_string()
    } else if (schedule - 5.0).abs() < 1e-6 {
        "5S".to_string()
    } else if (schedule - 10.0).abs() < 1e-6 {
        "10S".to_string()
    } else {
        let s_int = schedule.round() as i32;
        let s_str = s_int.to_string();
        if pipe_schedule::wall_thickness_ss(dn, &s_str).is_some() {
            s_str
        } else {
            let mapped = format!("{}S", s_int);
            if pipe_schedule::wall_thickness_ss(dn, &mapped).is_some() {
                mapped
            } else {
                "40S".to_string()
            }
        }
    };
    resolve_pipe_schedule_inner_ss(dn, schedule_str, segments)
}

fn resolve_pipe_schedule_inner_ss(dn: f64, schedule: String, segments: usize) -> ProfilePoints {
    let od = pipe_schedule::outer_diameter(dn);
    let wall = pipe_schedule::wall_thickness_ss(dn, &schedule).unwrap_or(od * 0.05);
    let outer_radius = od / 2.0;
    let inner_radius = (od - 2.0 * wall).max(0.0) / 2.0;
    circular::annulus(outer_radius, inner_radius, segments)
}

fn resolve_pipe_schedule_inner(dn: f64, schedule: String, segments: usize) -> ProfilePoints {
    let od = pipe_schedule::outer_diameter(dn);
    let wall = pipe_schedule::wall_thickness(dn, &schedule).unwrap_or(od * 0.05);
    let outer_radius = od / 2.0;
    let inner_radius = (od - 2.0 * wall).max(0.0) / 2.0;
    circular::annulus(outer_radius, inner_radius, segments)
}

pub fn generate(profile_type: &ProfileType) -> ProfilePoints {
    match profile_type {
        ProfileType::IShape { height, width, web_thickness, flange_thickness } => {
            i_shape::i_section(*height, *width, *web_thickness, *flange_thickness)
        }
        ProfileType::BoxShape { width, height, thickness } => {
            box_shape::box_section(*width, *height, *thickness)
        }
        ProfileType::Circle { radius, segments } => {
            circular::circle(*radius, *segments)
        }
        ProfileType::Annulus { outer_radius, inner_radius, segments } => {
            circular::annulus(*outer_radius, *inner_radius, *segments)
        }
        ProfileType::Rectangle { width, height } => {
            box_shape::rectangle(*width, *height)
        }
        ProfileType::LShape { leg1, leg2, thickness } => {
            box_shape::l_section(*leg1, *leg2, *thickness)
        }
        ProfileType::UShape { height, width, web_thickness, flange_thickness } => {
            u_shape::u_section(*height, *width, *web_thickness, *flange_thickness)
        }
        ProfileType::TShape { height, width, web_thickness, flange_thickness } => {
            t_shape::t_section(*height, *width, *web_thickness, *flange_thickness)
        }
        ProfileType::PipeSchedule { segments } => {
            circular::circle(50.0, *segments)
        }
        ProfileType::ParametricRect { .. } => {
            // generate() has no params, so return a safe default.
            // The actual dimensions are resolved in instantiate_template.
            box_shape::rectangle(5000.0, 200.0)
        }
    }
}

pub fn extrude_profile(
    profile: &ProfilePoints,
    depth: f64,
    direction: c::Vector3D,
    tolerance: f64,
) -> Option<c::MeshGeometry> {
    crate::brep::extrude_brep(profile, depth, direction, tolerance)
}

pub fn revolve_profile(
    profile: &ProfilePoints,
    angle: f64,
    axis_origin: c::Point3D,
    axis_direction: c::Vector3D,
    tolerance: f64,
) -> Option<c::MeshGeometry> {
    crate::brep::revolve_brep(profile, angle, axis_origin, axis_direction, tolerance)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_pipe_schedule_dn100_sch40() {
        let pts = resolve_pipe_schedule_profile(100.0, 40.0, 32);
        assert!(!pts.is_empty());
        let outer_pts: Vec<_> = pts.iter().take(32).collect();
        for p in &outer_pts {
            let r = (p.x * p.x + p.y * p.y).sqrt();
            assert!((r - 57.15).abs() < 0.1, "outer pt at radius {}", r);
        }
        let inner_pts: Vec<_> = pts.iter().skip(34).take(28).collect();
        for p in &inner_pts {
            let r = (p.x * p.x + p.y * p.y).sqrt();
            assert!((r - 51.13).abs() < 0.1, "inner pt at radius {}", r);
        }
    }

    #[test]
    fn test_resolve_pipe_schedule_default_sch40() {
        let pts = resolve_pipe_schedule_profile(100.0, 0.0, 32);
        assert!(!pts.is_empty());
        let outer_pts: Vec<_> = pts.iter().take(32).collect();
        for p in &outer_pts {
            let r = (p.x * p.x + p.y * p.y).sqrt();
            assert!((r - 57.15).abs() < 0.1);
        }
    }
}