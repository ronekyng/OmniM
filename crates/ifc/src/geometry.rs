//! IFC 几何实体解析模块
//!
//! 处理 IFC 文件中各种几何表示类型的解析和转换：
//! - IFCEXTRUDEDAREASOLID (拉伸体)
//! - IFCREVOLVEDAREASOLID (旋转体)
//! - IFCMAPPEDITEM (复用几何)
//! - IFCFACETEDBREP (面片边界表示)
//! - IFCBOOLEANRESULT / IFCBOOLEANCLIPPINGRESULT (布尔运算)
//! - IFCSWEPTDISKAREASOLID (扫掠圆盘)
//! - 各种剖面类型 (I/L/T/U/Z/Trapezoid/Ellipse/Derived/Composite)

use std::collections::HashMap;
use omnim_core as c;

use crate::{IfcDocument, IfcEntity, IfcValue};

// ============================================================
// 剖面提取 (Profile Extraction)
// ============================================================

/// 主入口：根据 profile entity 提取 2D 轮廓点集
pub fn extract_profile(doc: &IfcDocument, profile_id: u64) -> Option<Vec<c::Point2D>> {
    let profile = doc.get(profile_id)?;
    let profile_type = profile.entity_type.as_str();

    // 按类型分发
    let result = match profile_type {
        "IFCRECTANGLEPROFILEDEF" => extract_rect_profile(doc, profile),
        "IFCCIRCLEPROFILEDEF" => extract_circle_profile(doc, profile),
        "IFCARBITRARYCLOSEDPROFILEDEF" => extract_arbitrary_closed_profile(doc, profile),
        "IFCARBITRARYPROFILEDEFWITHVOIDS" => extract_arbitrary_profile_with_voids(doc, profile),
        "IFCISHAPEPROFILEDEF" => extract_i_profile(doc, profile),
        "IFCLSHAPEPROFILEDEF" => extract_l_profile(doc, profile),
        "IFCTSHAPEPROFILEDEF" => extract_t_profile(doc, profile),
        "IFCUSHAPEPROFILEDEF" => extract_u_profile(doc, profile),
        "IFCZSHAPEPROFILEDEF" => extract_z_profile(doc, profile),
        "IFCTRAPEZIUMPROFILEDEF" => extract_trapezium_profile(doc, profile),
        "IFCELLIPSEPROFILEDEF" => extract_ellipse_profile(doc, profile),
        "IFCDERIVEDPROFILEDEF" => extract_derived_profile(doc, profile),
        "IFCCOMPOSITEPROFILEDEF" => extract_composite_profile(doc, profile),
        _ => None,
    };

    // 根据 ProfilePosition (arg[3] or arg[4]) 将轮廓移到正确位置
    if let Some(pts) = result {
        apply_profile_position(doc, profile, profile_type, &pts)
    } else {
        None
    }
}

/// 应用剖面定位 (IfcAxis2Placement2D / IfcAxis2Placement3D)
fn apply_profile_position(
    doc: &IfcDocument,
    profile: &IfcEntity,
    profile_type: &str,
    pts: &[c::Point2D],
) -> Option<Vec<c::Point2D>> {
    // ProfileDef 的 Position 参数在不同子类型中的位置不同：
    // RectangleProfileDef: arg[3]
    // CircleProfileDef: arg[2]
    // ArbitraryClosedProfileDef: arg[3]
    // 结构剖面 (I/L/T 等): arg[4]
    let pos_idx = match profile_type {
        "IFCRECTANGLEPROFILEDEF" => 3,
        "IFCCIRCLEPROFILEDEF" => 2,
        "IFCARBITRARYCLOSEDPROFILEDEF" => 3,
        "IFCARBITRARYPROFILEDEFWITHVOIDS" => 4,
        // I/L/T/U/Z/Trapezium/Ellipse/Derived/Composite
        _ => 4,
    };

    let pos_ref = match profile.args.get(pos_idx) {
        Some(IfcValue::Ref(id)) => *id,
        _ => return Some(pts.to_vec()),
    };

    let pos_entity = doc.get(pos_ref)?;

    // IfcAxis2Placement2D: arg[0] = Location (CartesianPoint), arg[1] = RefDirection (optional)
    let location_ref = match pos_entity.args.get(0) {
        Some(IfcValue::Ref(id)) => *id,
        _ => return Some(pts.to_vec()),
    };
    let location = doc.get(location_ref)?;
    let (ox, oy) = extract_point_2d(doc, location)?;

    // Parse RefDirection (optional) for rotation
    let (cos_a, sin_a) = match pos_entity.args.get(1) {
        Some(IfcValue::Ref(dir_id)) => {
            let dir_entity = doc.get(*dir_id)?;
            extract_direction_2d(doc, dir_entity).unwrap_or((1.0, 0.0))
        }
        _ => (1.0, 0.0), // identity
    };

    let rotated: Vec<c::Point2D> = pts
        .iter()
        .map(|p| c::Point2D {
            x: ox + p.x * cos_a - p.y * sin_a,
            y: oy + p.x * sin_a + p.y * cos_a,
        })
        .collect();
    Some(rotated)
}

/// 提取 2D 方向向量，返回 (cos, sin)
fn extract_direction_2d(_doc: &IfcDocument, entity: &IfcEntity) -> Option<(f64, f64)> {
    let dir_list = match entity.args.get(0) {
        Some(IfcValue::List(items)) => items,
        _ => return None,
    };
    let dx = crate::coord_to_f64(dir_list.get(0))?;
    let dy = crate::coord_to_f64(dir_list.get(1))?;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-10 {
        return Some((1.0, 0.0));
    }
    Some((dx / len, dy / len))
}

/// 从 IfcCartesianPoint 提取 2D 坐标
fn extract_point_2d(_doc: &IfcDocument, entity: &IfcEntity) -> Option<(f64, f64)> {
    let coords = match entity.args.get(0) {
        Some(IfcValue::List(items)) => items,
        _ => return None,
    };
    let x = crate::coord_to_f64(coords.get(0))?;
    let y = crate::coord_to_f64(coords.get(1))?;
    Some((x, y))
}

// ---- 具体剖面类型实现 ----

/// IFC矩形剖面
fn extract_rect_profile(doc: &IfcDocument, profile: &IfcEntity) -> Option<Vec<c::Point2D>> {
    let width = doc.get_float(profile, 1)?;
    let height = doc.get_float(profile, 2)?;
    Some(vec![
        c::Point2D { x: -width / 2.0, y: -height / 2.0 },
        c::Point2D { x: width / 2.0, y: -height / 2.0 },
        c::Point2D { x: width / 2.0, y: height / 2.0 },
        c::Point2D { x: -width / 2.0, y: height / 2.0 },
    ])
}

/// IFC圆形剖面
fn extract_circle_profile(doc: &IfcDocument, profile: &IfcEntity) -> Option<Vec<c::Point2D>> {
    let radius = doc.get_float(profile, 1)?;
    let segs = 32;
    let mut pts = Vec::new();
    for i in 0..segs {
        let a = 2.0 * std::f64::consts::PI * i as f64 / segs as f64;
        pts.push(c::Point2D { x: radius * a.cos(), y: radius * a.sin() });
    }
    Some(pts)
}

/// 任意闭合轮廓 (IfcPolyline)
pub(crate) fn extract_polyline_2d(doc: &IfcDocument, curve_entity: &IfcEntity) -> Option<Vec<c::Point2D>> {
    let points_list = match curve_entity.args.get(0) {
        Some(IfcValue::List(points)) => points,
        _ => return None,
    };
    let mut pts = Vec::new();
    for p in points_list {
        if let IfcValue::Ref(pt_id) = p {
            if let Some(pt_entity) = doc.get(*pt_id) {
                if let Some(IfcValue::List(coords)) = pt_entity.args.get(0) {
                    let x = crate::coord_to_f64(coords.get(0))?;
                    let y = crate::coord_to_f64(coords.get(1))?;
                    pts.push(c::Point2D { x, y });
                }
            }
        }
    }
    if pts.len() >= 3 { Some(pts) } else { None }
}

/// 任意闭合轮廓
fn extract_arbitrary_closed_profile(doc: &IfcDocument, profile: &IfcEntity) -> Option<Vec<c::Point2D>> {
    let curve_ref = doc.get_ref(profile, 2)?;
    let curve = doc.get(curve_ref)?;
    if curve.entity_type == "IFCPOLYLINE" {
        extract_polyline_2d(doc, curve)
    } else {
        // 可以扩展支持 IFCTRIMMEDCURVE 等
        None
    }
}

/// 带空洞的任意剖面 (仅取外轮廓)
fn extract_arbitrary_profile_with_voids(doc: &IfcDocument, profile: &IfcEntity) -> Option<Vec<c::Point2D>> {
    let outer_curve_ref = doc.get_ref(profile, 2)?;
    let curve = doc.get(outer_curve_ref)?;
    if curve.entity_type == "IFCPOLYLINE" {
        extract_polyline_2d(doc, curve)
    } else {
        None
    }
}

/// IFC工字钢剖面 (IFCISHAPEPROFILEDEF)
///
/// 参数顺序: OverallWidth, OverallDepth, WebThickness, FlangeThickness, FilletRadius (opt)
fn extract_i_profile(doc: &IfcDocument, profile: &IfcEntity) -> Option<Vec<c::Point2D>> {
    let w = doc.get_float(profile, 1)?;  // OverallWidth
    let h = doc.get_float(profile, 2)?;  // OverallDepth
    let tw = doc.get_float(profile, 3)?; // WebThickness
    let tf = doc.get_float(profile, 4)?; // FlangeThickness

    let hw = h / 2.0;
    let ww = w / 2.0;
    let tw2 = tw / 2.0;

    // I-beam profile (counterclockwise, starting from bottom-left of top flange)
    let pts = vec![
        // Top flange (left to right, top to bottom)
        c::Point2D { x: -ww, y: hw },
        c::Point2D { x: ww, y: hw },
        c::Point2D { x: ww, y: hw - tf },
        // Right web (top to bottom)
        c::Point2D { x: tw2, y: hw - tf },
        c::Point2D { x: tw2, y: -hw + tf },
        // Bottom flange (right to left, top to bottom)
        c::Point2D { x: ww, y: -hw + tf },
        c::Point2D { x: ww, y: -hw },
        c::Point2D { x: -ww, y: -hw },
        c::Point2D { x: -ww, y: -hw + tf },
        // Left web (bottom to top)
        c::Point2D { x: -tw2, y: -hw + tf },
        c::Point2D { x: -tw2, y: hw - tf },
        c::Point2D { x: -ww, y: hw - tf },
    ];
    Some(pts)
}

/// IFC角钢剖面 (IFCLSHAPEPROFILEDEF)
///
/// 参数顺序: Depth, Width, Thickness, FilletRadius (opt), EdgeRadius (opt), LegSlope (opt)
fn extract_l_profile(doc: &IfcDocument, profile: &IfcEntity) -> Option<Vec<c::Point2D>> {
    let depth = doc.get_float(profile, 1)?;  // vertical leg
    let width = doc.get_float(profile, 2)?;  // horizontal leg
    let thick = doc.get_float(profile, 3)?;  // thickness

    // L-angle profile (counterclockwise, standing)
    let pts = vec![
        c::Point2D { x: 0.0, y: 0.0 },
        c::Point2D { x: width, y: 0.0 },
        c::Point2D { x: width, y: thick },
        c::Point2D { x: thick, y: thick },
        c::Point2D { x: thick, y: depth },
        c::Point2D { x: 0.0, y: depth },
    ];
    Some(pts)
}

/// IFCT型钢剖面 (IFCTSHAPEPROFILEDEF)
///
/// 参数顺序: Depth, Width, WebThickness, FlangeThickness, FilletRadius (opt), FlangeEdgeRadius (opt), WebEdgeRadius (opt)
fn extract_t_profile(doc: &IfcDocument, profile: &IfcEntity) -> Option<Vec<c::Point2D>> {
    let depth = doc.get_float(profile, 1)?;   // Depth (web height + flange)
    let width = doc.get_float(profile, 2)?;   // Flange width
    let tw = doc.get_float(profile, 3)?;      // WebThickness
    let tf = doc.get_float(profile, 4)?;      // FlangeThickness

    let ww = width / 2.0;
    let tw2 = tw / 2.0;

    // T-profile (counterclockwise, flange on top, web below)
    let pts = vec![
        c::Point2D { x: -ww, y: 0.0 },
        c::Point2D { x: ww, y: 0.0 },
        c::Point2D { x: ww, y: tf },
        c::Point2D { x: tw2, y: tf },
        c::Point2D { x: tw2, y: depth },
        c::Point2D { x: -tw2, y: depth },
        c::Point2D { x: -tw2, y: tf },
        c::Point2D { x: -ww, y: tf },
    ];
    Some(pts)
}

/// IFC槽钢剖面 (IFCUSHAPEPROFILEDEF)
///
/// 参数顺序: Depth, FlangeWidth, WebThickness, FlangeThickness, FilletRadius (opt), EdgeRadius (opt), FlangeSlope (opt)
fn extract_u_profile(doc: &IfcDocument, profile: &IfcEntity) -> Option<Vec<c::Point2D>> {
    let depth = doc.get_float(profile, 1)?;      // Overall depth
    let fw = doc.get_float(profile, 2)?;          // Flange width
    let tw = doc.get_float(profile, 3)?;          // Web thickness
    let tf = doc.get_float(profile, 4)?;          // Flange thickness

    let hw = depth / 2.0;

    // U-channel profile (counterclockwise, opening to the right)
    let pts = vec![
        c::Point2D { x: 0.0, y: hw },
        c::Point2D { x: fw, y: hw },
        c::Point2D { x: fw, y: hw - tf },
        c::Point2D { x: tw, y: hw - tf },
        c::Point2D { x: tw, y: -hw + tf },
        c::Point2D { x: fw, y: -hw + tf },
        c::Point2D { x: fw, y: -hw },
        c::Point2D { x: 0.0, y: -hw },
    ];
    Some(pts)
}

/// IFCZ型钢剖面 (IFCZSHAPEPROFILEDEF)
///
/// 参数顺序: Depth, Width, WebThickness, FlangeThickness, FilletRadius (opt), EdgeRadius (opt)
fn extract_z_profile(doc: &IfcDocument, profile: &IfcEntity) -> Option<Vec<c::Point2D>> {
    let depth = doc.get_float(profile, 1)?;  // Overall depth
    let width = doc.get_float(profile, 2)?;  // Flange width
    let tw = doc.get_float(profile, 3)?;    // Web thickness
    let tf = doc.get_float(profile, 4)?;    // Flange thickness

    let hw = depth / 2.0;

    // Z-profile (counterclockwise, center at origin)
    let pts = vec![
        c::Point2D { x: -width, y: hw },
        c::Point2D { x: 0.0, y: hw },
        c::Point2D { x: 0.0, y: hw - tf },
        c::Point2D { x: -tw, y: hw - tf },
        c::Point2D { x: -tw, y: -hw + tf },
        c::Point2D { x: 0.0, y: -hw + tf },
        c::Point2D { x: 0.0, y: -hw },
        c::Point2D { x: width, y: -hw },
        c::Point2D { x: width, y: -hw + tf },
        c::Point2D { x: tw, y: -hw + tf },
        c::Point2D { x: tw, y: hw - tf },
        c::Point2D { x: -width, y: hw - tf },
    ];
    Some(pts)
}

/// IFC梯形剖面 (IFCTRAPEZIUMPROFILEDEF)
///
/// 参数顺序: BottomXDim, TopXDim, YDim, TopXOffset (default 0)
fn extract_trapezium_profile(doc: &IfcDocument, profile: &IfcEntity) -> Option<Vec<c::Point2D>> {
    let bx = doc.get_float(profile, 1)?;    // Bottom width
    let tx = doc.get_float(profile, 2)?;    // Top width
    let y = doc.get_float(profile, 3)?;     // Height
    let off = doc.get_float(profile, 4).unwrap_or(0.0); // Top offset

    let bx2 = bx / 2.0;
    let tx2 = tx / 2.0;
    let hy = y / 2.0;

    Some(vec![
        c::Point2D { x: -bx2, y: -hy },
        c::Point2D { x: bx2, y: -hy },
        c::Point2D { x: tx2 + off, y: hy },
        c::Point2D { x: -tx2 + off, y: hy },
    ])
}

/// IFC椭圆剖面 (IFCELLIPSEPROFILEDEF)
///
/// 参数顺序: SemiAxis1, SemiAxis2
fn extract_ellipse_profile(doc: &IfcDocument, profile: &IfcEntity) -> Option<Vec<c::Point2D>> {
    let a = doc.get_float(profile, 1)?; // SemiAxis1 (X radius)
    let b = doc.get_float(profile, 2)?; // SemiAxis2 (Y radius)
    let segs = 32;
    let mut pts = Vec::new();
    for i in 0..segs {
        let ang = 2.0 * std::f64::consts::PI * i as f64 / segs as f64;
        pts.push(c::Point2D { x: a * ang.cos(), y: b * ang.sin() });
    }
    Some(pts)
}

/// IFC派生剖面 (IFCDERIVEDPROFILEDEF)
///
/// 参数顺序: ParentProfile (Ref), Operator (enum), Transformation (2D transform)
fn extract_derived_profile(doc: &IfcDocument, profile: &IfcEntity) -> Option<Vec<c::Point2D>> {
    // arg[2] = ParentProfile (entity ref)
    let parent_ref = match profile.args.get(2) {
        Some(IfcValue::Ref(id)) => *id,
        _ => return None,
    };
    // arg[3] = Operator (enum: DIFFERENCE, INTERSECTION, UNION, ADD)
    let _operator = match profile.args.get(3) {
        Some(IfcValue::Enum(op)) => op.as_str(),
        _ => "",
    };

    // First, try to extract the parent profile points
    let parent_pts = extract_profile(doc, parent_ref)?;

    // arg[4] = IfcCartesianTransformationOperator2D (optional transformation)
    match profile.args.get(4) {
        Some(IfcValue::Ref(trans_ref)) => {
            apply_cartesian_transform_2d(doc, *trans_ref, &parent_pts)
        }
        _ => Some(parent_pts),
    }
}

/// 应用 IfcCartesianTransformationOperator2D
fn apply_cartesian_transform_2d(
    doc: &IfcDocument,
    trans_ref: u64,
    pts: &[c::Point2D],
) -> Option<Vec<c::Point2D>> {
    let trans = doc.get(trans_ref)?;
    // arg[0] = Axis1 (optional direction)
    // arg[1] = Axis2 (optional direction)
    // arg[2] = LocalOrigin (optional point)
    // arg[3] = Scale (optional double)

    let (ox, oy) = match trans.args.get(2) {
        Some(IfcValue::Ref(loc_ref)) => {
            let loc = doc.get(*loc_ref)?;
            extract_point_2d(doc, loc)?
        }
        _ => (0.0, 0.0),
    };

    let scale = match trans.args.get(3) {
        Some(IfcValue::Float(s)) => *s,
        Some(IfcValue::Integer(i)) => *i as f64,
        _ => 1.0,
    };

    // Simplified: just translate and scale
    let transformed: Vec<c::Point2D> = pts
        .iter()
        .map(|p| c::Point2D {
            x: ox + p.x * scale,
            y: oy + p.y * scale,
        })
        .collect();
    Some(transformed)
}

/// IFC复合剖面 (IFCCOMPOSITEPROFILEDEF)
fn extract_composite_profile(doc: &IfcDocument, profile: &IfcEntity) -> Option<Vec<c::Point2D>> {
    // arg[2] = Profiles (list of refs to IfcProfileDef)
    let profile_refs = match profile.args.get(2) {
        Some(IfcValue::List(items)) => items,
        _ => return None,
    };

    // For composite profiles, just use the first non-empty one
    for item in profile_refs {
        if let IfcValue::Ref(pid) = item {
            if let Some(pts) = extract_profile(doc, *pid) {
                return Some(pts);
            }
        }
    }
    None
}

// ============================================================
// 几何实体转换 (Geometry Entity Conversion)
// ============================================================

/// 主入口：将 IFC 几何表示项 (Representation Item) 转换为 OmniM Geometry
pub fn convert_ifc_item(doc: &IfcDocument, item_id: u64) -> Option<c::Geometry> {
    let item = doc.get(item_id)?;
    match item.entity_type.as_str() {
        "IFCEXTRUDEDAREASOLID" | "IFCEXTRUDEDAREASOLIDTAPERED" => {
            convert_extruded_area_solid(doc, item)
        }
        "IFCREVOLVEDAREASOLID" => convert_revolved_area_solid(doc, item),
        "IFCMAPPEDITEM" => convert_mapped_item(doc, item),
        "IFCFACETEDBREP" => convert_faceted_brep(doc, item),
        "IFCBOOLEANRESULT" | "IFCBOOLEANCLIPPINGRESULT" => convert_boolean_result(doc, item),
        "IFCSWEPTDISKAREASOLID" => convert_swept_disk_solid(doc, item),
        "IFCCSGSOLID" => convert_csg_solid(doc, item),
        _ => None,
    }
}

// ---- 拉伸体 (ExtrudedAreaSolid) ----

fn convert_extruded_area_solid(doc: &IfcDocument, item: &IfcEntity) -> Option<c::Geometry> {
    // Args: SweptArea, Position, ExtrudedDirection, Depth (and DepthAsOverride for tapered)
    let swept_area_ref = doc.get_ref(item, 0)?;
    let depth = doc.get_float(item, 3)?;

    // Direction
    let dir_entity_id = doc.get_ref(item, 2)?;
    let dir_entity = doc.get(dir_entity_id)?;
    let dir = match dir_entity.args.get(0) {
        Some(IfcValue::List(coords)) => {
            let dx = crate::coord_to_f64(coords.get(0))?;
            let dy = crate::coord_to_f64(coords.get(1))?;
            let dz = crate::coord_to_f64(coords.get(2))?;
            c::Vector3D { x: dx, y: dy, z: dz }
        }
        _ => return None,
    };

    // Position (IfcAxis2Placement3D) - apply to profile before extrusion
    let profile_pts = extract_profile_with_position(doc, item, swept_area_ref)?;

    let extrusion = c::Extrusion {
        profile: profile_pts,
        depth,
        direction: Some(dir),
    };

    let pg = c::ParametricGeometry {
        operations: vec![c::ParametricOperation {
            operation: Some(c::parametric_operation::Operation::Extrusion(extrusion)),
        }],
    };
    Some(omnim_model::geometry::create_geometry(pg))
}

/// 提取剖面并应用 IfcSweptAreaSolid 的 Position (IfcAxis2Placement3D)
fn extract_profile_with_position(
    doc: &IfcDocument,
    item: &IfcEntity,
    profile_id: u64,
) -> Option<Vec<c::Point2D>> {
    let profile_pts = extract_profile(doc, profile_id)?;

    // Position is at arg[1] in IfcSweptAreaSolid subclasses
    let pos_ref = doc.get_ref(item, 1)?;
    let pos_entity = doc.get(pos_ref)?;

    // IfcAxis2Placement3D: arg[0] = Location, arg[1] = Axis (Z dir), arg[2] = RefDirection (X dir)
    let location_ref = match pos_entity.args.get(0) {
        Some(IfcValue::Ref(id)) => *id,
        _ => return Some(profile_pts),
    };
    let location = doc.get(location_ref)?;
    let (ox, oy, _oz) = extract_point_3d(doc, location)?;

    // For 2D profile on XY plane, just apply translation (the 3D rotation is handled by extrusion direction)
    // Only translate if the position is non-zero
    if ox.abs() < 1e-10 && oy.abs() < 1e-10 {
        Some(profile_pts)
    } else {
        Some(profile_pts.iter().map(|p| c::Point2D { x: p.x + ox, y: p.y + oy }).collect())
    }
}

fn extract_point_3d(_doc: &IfcDocument, entity: &IfcEntity) -> Option<(f64, f64, f64)> {
    let coords = match entity.args.get(0) {
        Some(IfcValue::List(items)) => items,
        _ => return None,
    };
    let x = crate::coord_to_f64(coords.get(0))?;
    let y = crate::coord_to_f64(coords.get(1))?;
    let z = crate::coord_to_f64(coords.get(2))?;
    Some((x, y, z))
}

// ---- 旋转体 (RevolvedAreaSolid) ----

/// IFCREVOLVEDAREASOLID → Revolve 参数化几何
///
/// 参数: SweptArea, Position, Axis (IfcAxis1Placement), Angle
fn convert_revolved_area_solid(doc: &IfcDocument, item: &IfcEntity) -> Option<c::Geometry> {
    let swept_area_ref = doc.get_ref(item, 0)?;
    let angle = doc.get_float(item, 3)?;

    // Axis (IfcAxis1Placement): arg[0] = Location, arg[1] = Axis (direction)
    let axis_ref = doc.get_ref(item, 2)?;
    let axis_entity = doc.get(axis_ref)?;

    let axis_loc_ref = match axis_entity.args.get(0) {
        Some(IfcValue::Ref(id)) => *id,
        _ => return None,
    };
    let axis_loc = doc.get(axis_loc_ref)?;
    let (ax, ay, az) = extract_point_3d(doc, axis_loc)?;

    // Axis direction (optional, defaults to (0,0,1))
    let (adx, ady, adz) = match axis_entity.args.get(1) {
        Some(IfcValue::Ref(dir_ref)) => {
            let dir = doc.get(*dir_ref)?;
            match dir.args.get(0) {
                Some(IfcValue::List(coords)) => {
                    let dx = crate::coord_to_f64(coords.get(0))?;
                    let dy = crate::coord_to_f64(coords.get(1))?;
                    let dz = crate::coord_to_f64(coords.get(2))?;
                    (dx, dy, dz)
                }
                _ => (0.0, 0.0, 1.0),
            }
        }
        _ => (0.0, 0.0, 1.0),
    };

    // Extract profile
    let profile_pts = extract_profile(doc, swept_area_ref)?;

    // Get position location offset and apply to profile
    let pos_ref = doc.get_ref(item, 1)?;
    let pos_entity = doc.get(pos_ref)?;
    let (ox, oy, _oz) = match pos_entity.args.get(0) {
        Some(IfcValue::Ref(id)) => {
            let loc = doc.get(*id)?;
            extract_point_3d(doc, loc)?
        }
        _ => (0.0, 0.0, 0.0),
    };

    let positioned_pts: Vec<c::Point2D> = profile_pts
        .iter()
        .map(|p| c::Point2D { x: p.x + ox, y: p.y + oy })
        .collect();

    let revolve = c::Revolve {
        profile: positioned_pts,
        angle,
        axis_origin: Some(c::Point3D { x: ax, y: ay, z: az }),
        axis_direction: Some(c::Vector3D { x: adx, y: ady, z: adz }),
    };

    let pg = c::ParametricGeometry {
        operations: vec![c::ParametricOperation {
            operation: Some(c::parametric_operation::Operation::Revolve(revolve)),
        }],
    };
    Some(omnim_model::geometry::create_geometry(pg))
}

// ---- 映射项 (MappedItem) ----

/// IFCMAPPEDITEM → 几何复用/实例化
///
/// 参数: MappingSource (IfcRepresentationMap), MappingTarget (CartesianTransformationOperator)
///
/// IfcRepresentationMap: arg[0] = RepresentedDefinition (MappedRepresentation type),
///                       arg[1] = MappingOrigin (Axis2Placement)
/// IfcMappedItem: arg[0] = MappingSource, arg[1] = MappingTarget
fn convert_mapped_item(doc: &IfcDocument, item: &IfcEntity) -> Option<c::Geometry> {
    // Get MappingSource
    let map_source_ref = doc.get_ref(item, 0)?;
    let map_source = doc.get(map_source_ref)?;

    // Get RepresentedDefinition (the actual representation to map)
    let rep_def_ref = doc.get_ref(map_source, 0)?;
    let rep_def = doc.get(rep_def_ref)?;

    // It could be a direct shape representation or a type product representation
    // Try to find geometry items in the representation definition
    match rep_def.entity_type.as_str() {
        "IFCREPRESENTATIONMAP" => {
            // Skip - we'll handle below
        }
        _ => {}
    }

    // For now, just try to convert the first item in the represented definition
    // This handles the simple case of a Type Product (e.g., IFCWALLTYPE) with a mapped representation
    // Try to convert each representation item
    if let Some(IfcValue::List(items)) = rep_def.args.get(3) {
        for item_ref in items {
            if let IfcValue::Ref(child_id) = item_ref {
                if let Some(geom) = convert_ifc_item(doc, *child_id) {
                    // Apply mapping transformation
                    return apply_mapping_transform(doc, item, geom);
                }
            }
        }
    }

    // Fallback: try to find the representation in the represented definition
    if let Some(IfcValue::List(items)) = rep_def.args.get(3) {
        for item_ref in items {
            if let IfcValue::Ref(child_id) = item_ref {
                if let Some(geom) = convert_ifc_item(doc, *child_id) {
                    return apply_mapping_transform(doc, item, geom);
                }
            }
        }
    }

    None
}

/// 应用 IFCMAPPEDITEM 的 MappingTarget 变换
fn apply_mapping_transform(doc: &IfcDocument, item: &IfcEntity, geom: c::Geometry) -> Option<c::Geometry> {
    let target_ref = doc.get_ref(item, 1)?;
    let target = doc.get(target_ref)?;

    match target.entity_type.as_str() {
        "IFCCARTESIANTRANSFORMATIONOPERATOR3D" |
        "IFCCARTESIANTRANSFORMATIONOPERATOR3DNONUNIFORM" => {
            // arg[0] = Axis1 (X direction, optional)
            // arg[1] = Axis2 (Y direction, optional)
            // arg[2] = LocalOrigin
            // arg[3] = Scale
            // arg[4] = Scale2 (non-uniform, optional)
            let (ox, oy, oz) = match target.args.get(2) {
                Some(IfcValue::Ref(loc_ref)) => {
                    let loc = doc.get(*loc_ref)?;
                    extract_point_3d(doc, loc)?
                }
                _ => return Some(geom),
            };

            let scale = match target.args.get(3) {
                Some(IfcValue::Float(s)) => *s,
                Some(IfcValue::Integer(i)) => *i as f64,
                _ => 1.0,
            };

            // Apply translation + scale to the geometry mesh
            if let Some(mesh) = omnim_model::geometry::geometry_to_mesh(&geom) {
                let t = c::Transform {
                    matrix: vec![],
                    translation: Some(c::Point3D { x: ox, y: oy, z: oz }),
                    rotation: None,
                    scale: Some(c::Vector3D { x: scale, y: scale, z: scale }),
                };
                let transformed = omnim_model::geometry::transform_mesh(&mesh, &t);
                let bbox = omnim_model::geometry::compute_bbox(&transformed);
                Some(c::Geometry {
                    id: String::new(),
                    r#type: c::GeometryType::Mesh as i32,
                    representation: Some(c::geometry::Representation::Mesh(transformed)),
                    bbox: Some(bbox),
                    level_of_detail: 0,
                })
            } else {
                Some(geom)
            }
        }
        "IFCCARTESIANTRANSFORMATIONOPERATOR2D" => {
            // 2D transform — just return the geometry as-is
            Some(geom)
        }
        _ => Some(geom),
    }
}

// ---- 面片边界表示 (FacetedBrep) ----

/// IFCFACETEDBREP → MeshGeometry
///
/// 结构: IfcFacetedBrep → IfcClosedShell → [IfcFace → [IfcFaceBound → IfcPolyLoop → points]]
fn convert_faceted_brep(doc: &IfcDocument, item: &IfcEntity) -> Option<c::Geometry> {
    // arg[0] = Outer (IfcClosedShell)
    let shell_ref = doc.get_ref(item, 0)?;
    let shell = doc.get(shell_ref)?;

    // Shell is an IfcConnectedFaceSet — arg[0] = CfsFaces (list)
    let faces_list = match shell.args.get(0) {
        Some(IfcValue::List(faces)) => faces,
        _ => return None,
    };

    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut faces = Vec::new();
    let mut vertex_map: HashMap<(i64, i64, i64), i32> = HashMap::new();

    fn key_from_pt(p: &c::Point3D) -> (i64, i64, i64) {
        (
            (p.x * 1e6).round() as i64,
            (p.y * 1e6).round() as i64,
            (p.z * 1e6).round() as i64,
        )
    }

    fn add_vertex(v: &mut Vec<c::Point3D>, vm: &mut HashMap<(i64, i64, i64), i32>, p: c::Point3D) -> i32 {
        let k = key_from_pt(&p);
        if let Some(&idx) = vm.get(&k) {
            idx
        } else {
            let idx = v.len() as i32;
            v.push(p);
            vm.insert(k, idx);
            idx
        }
    }

    // Process each IfcFace
    for face_item in faces_list {
        let face_id = match face_item {
            IfcValue::Ref(id) => *id,
            _ => continue,
        };
        let face = doc.get(face_id)?;
        if face.entity_type != "IFCFACE" {
            continue;
        }

        // arg[0] = Bounds (list of IfcFaceBound)
        let bounds_list = match face.args.get(0) {
            Some(IfcValue::List(bounds)) => bounds,
            _ => continue,
        };

        for bound_item in bounds_list {
            let bound_id = match bound_item {
                IfcValue::Ref(id) => *id,
                _ => continue,
            };
            let bound = doc.get(bound_id)?;

            // arg[0] = Bound (IfcPolyLoop / IfcEdgeLoop)
            let loop_ref = match bound.args.get(0) {
                Some(IfcValue::Ref(id)) => *id,
                _ => continue,
            };
            let loop_entity = doc.get(loop_ref)?;

            let face_pts = match loop_entity.entity_type.as_str() {
                "IFCPOLYLOOP" => extract_poly_loop(doc, loop_entity),
                _ => continue,
            };

            if face_pts.len() < 3 {
                continue;
            }

            // Triangulate: fan triangulation
            let base = face_pts.len();
            let idxs: Vec<i32> = face_pts
                .iter()
                .map(|p| add_vertex(&mut vertices, &mut vertex_map, p.clone()))
                .collect();

            for i in 1..(base as i32 - 1) {
                faces.push(c::FaceIndices {
                    vertex_indices: vec![idxs[0], idxs[i as usize + 1], idxs[i as usize]],
                });
            }
        }
    }

    if vertices.is_empty() {
        return None;
    }

    // Compute face normals
    for _ in 0..vertices.len() {
        normals.push(c::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
    }

    let mesh = c::MeshGeometry { vertices, normals, uvs: vec![], faces };
    let bbox = omnim_model::geometry::compute_bbox(&mesh);
    Some(c::Geometry {
        id: String::new(),
        r#type: c::GeometryType::Mesh as i32,
        representation: Some(c::geometry::Representation::Mesh(mesh)),
        bbox: Some(bbox),
        level_of_detail: 0,
    })
}

/// 从 IfcPolyLoop 提取 3D 点序列
fn extract_poly_loop(doc: &IfcDocument, loop_entity: &IfcEntity) -> Vec<c::Point3D> {
    let points_list = match loop_entity.args.get(0) {
        Some(IfcValue::List(items)) => items,
        _ => return vec![],
    };
    let mut pts = Vec::new();
    for item in points_list {
        if let IfcValue::Ref(pt_id) = item {
            if let Some(pt_entity) = doc.get(*pt_id) {
                if let Some(IfcValue::List(coords)) = pt_entity.args.get(0) {
                    if let (Some(x), Some(y), Some(z)) = (
                        crate::coord_to_f64(coords.get(0)),
                        crate::coord_to_f64(coords.get(1)),
                        crate::coord_to_f64(coords.get(2)),
                    ) {
                        pts.push(c::Point3D { x, y, z });
                    }
                }
            }
        }
    }
    pts
}

// ---- 布尔运算结果 (BooleanResult) ----

/// IFCBOOLEANRESULT / IFCBOOLEANCLIPPINGRESULT → Mesh boolean operations
///
/// 参数: Operator (DIFFERENCE/UNION/INTERSECTION), FirstOperand, SecondOperand
///
/// 实现策略:
/// - UNION: 简单合并两个 mesh
/// - DIFFERENCE/CLIPPING: 使用 boolean_subtract 引擎进行网格差集
/// - INTERSECTION: 使用 AABB 筛选重叠区域的三角形
fn convert_boolean_result(doc: &IfcDocument, item: &IfcEntity) -> Option<c::Geometry> {
    use omnim_model::gen_rule::boolean_subtract;
    use omnim_model::gen_rule::BooleanSubtractInput;

    // arg[1] = Operator (enum: DIFFERENCE, UNION, INTERSECTION)
    let operator = match item.args.get(1) {
        Some(IfcValue::Enum(op)) => op.as_str(),
        _ => return None,
    };

    // arg[2] = FirstOperand
    let first_ref = match item.args.get(2) {
        Some(IfcValue::Ref(id)) => *id,
        _ => return None,
    };

    // arg[3] = SecondOperand
    let second_ref = match item.args.get(3) {
        Some(IfcValue::Ref(id)) => *id,
        _ => return None,
    };

    let first_geom = convert_ifc_item(doc, first_ref)?;
    let second_geom = convert_ifc_item(doc, second_ref)?;

    let first_mesh = omnim_model::geometry::geometry_to_mesh(&first_geom)?;
    let second_mesh = omnim_model::geometry::geometry_to_mesh(&second_geom)?;

    match operator {
        "UNION" => {
            // Simple mesh merge
            let merged = omnim_model::geometry::merge_meshes(&[first_mesh, second_mesh]);
            let bbox = omnim_model::geometry::compute_bbox(&merged);
            Some(c::Geometry {
                id: String::new(),
                r#type: c::GeometryType::Mesh as i32,
                representation: Some(c::geometry::Representation::Mesh(merged)),
                bbox: Some(bbox),
                level_of_detail: 0,
            })
        }
        "DIFFERENCE" | "CLIPPING" => {
            // 使用 boolean_subtract 引擎进行真实差集运算
            let result = boolean_subtract::generate(&BooleanSubtractInput {
                host_mesh: first_mesh,
                subtract_mesh: second_mesh,
            });
            let bbox = omnim_model::geometry::compute_bbox(&result);
            Some(c::Geometry {
                id: String::new(),
                r#type: c::GeometryType::Mesh as i32,
                representation: Some(c::geometry::Representation::Mesh(result)),
                bbox: Some(bbox),
                level_of_detail: 0,
            })
        }
        "INTERSECTION" => {
            // 保留 first_mesh 中落在 second_mesh AABB 内的三角形
            let bbox2 = omnim_model::geometry::compute_bbox(&second_mesh);
            let min2 = bbox2.min.as_ref()?;
            let max2 = bbox2.max.as_ref()?;
            let tol = 1e-6;

            let mut vertices: Vec<c::Point3D> = Vec::new();
            let mut normals: Vec<c::Vector3D> = Vec::new();
            let mut faces: Vec<c::FaceIndices> = Vec::new();

            for face in &first_mesh.faces {
                if face.vertex_indices.len() < 3 {
                    continue;
                }
                // 计算三角形质心
                let mut cx = 0.0_f64; let mut cy = 0.0_f64; let mut cz = 0.0_f64;
                let n = face.vertex_indices.len() as f64;
                for vi in &face.vertex_indices {
                    let idx = *vi as usize;
                    if idx < first_mesh.vertices.len() {
                        cx += first_mesh.vertices[idx].x;
                        cy += first_mesh.vertices[idx].y;
                        cz += first_mesh.vertices[idx].z;
                    }
                }
                cx /= n; cy /= n; cz /= n;

                // 质心在 second_mesh AABB 内则保留
                if cx >= min2.x - tol && cx <= max2.x + tol
                    && cy >= min2.y - tol && cy <= max2.y + tol
                    && cz >= min2.z - tol && cz <= max2.z + tol
                {
                    let base = vertices.len() as i32;
                    for vi in &face.vertex_indices {
                        let idx = *vi as usize;
                        if idx < first_mesh.vertices.len() {
                            vertices.push(first_mesh.vertices[idx].clone());
                            if idx < first_mesh.normals.len() {
                                normals.push(first_mesh.normals[idx].clone());
                            } else {
                                normals.push(c::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
                            }
                        }
                    }
                    faces.push(c::FaceIndices {
                        vertex_indices: (0..face.vertex_indices.len() as i32).map(|i| base + i).collect(),
                    });
                }
            }

            let result = c::MeshGeometry { vertices, normals, uvs: vec![], faces };
            let bbox = omnim_model::geometry::compute_bbox(&result);
            Some(c::Geometry {
                id: String::new(),
                r#type: c::GeometryType::Mesh as i32,
                representation: Some(c::geometry::Representation::Mesh(result)),
                bbox: Some(bbox),
                level_of_detail: 0,
            })
        }
        _ => Some(first_geom),
    }
}

// ---- 扫掠圆盘 (SweptDiskSolid) ----

/// IFCSWEPTDISKAREASOLID → 沿路径扫掠的圆盘
///
/// 参数: Directrix (curve), Radius, InnerRadius (opt), StartParam (opt), EndParam (opt)
fn convert_swept_disk_solid(doc: &IfcDocument, item: &IfcEntity) -> Option<c::Geometry> {
    let radius = doc.get_float(item, 1)?;
    let _inner_radius = doc.get_float(item, 2).unwrap_or(0.0);

    // Directrix curve
    let dir_ref = doc.get_ref(item, 0)?;
    let curve = doc.get(dir_ref)?;

    let path_pts = match curve.entity_type.as_str() {
        "IFCPOLYLINE" | "IFCTRIMMEDCURVE" => extract_curve_points(doc, curve),
        _ => extract_curve_points(doc, curve),
    };

    if path_pts.len() < 2 {
        return None;
    }

    // Generate a simple tube: approximate the path with a series of disks
    let segs = 16;
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut faces = Vec::new();

    for seg_idx in 0..path_pts.len() {
        let p = &path_pts[seg_idx];
        // Determine the direction at this point
        let dir = if seg_idx < path_pts.len() - 1 {
            c::Vector3D {
                x: path_pts[seg_idx + 1].x - p.x,
                y: path_pts[seg_idx + 1].y - p.y,
                z: path_pts[seg_idx + 1].z - p.z,
            }
        } else {
            c::Vector3D {
                x: p.x - path_pts[seg_idx - 1].x,
                y: p.y - path_pts[seg_idx - 1].y,
                z: p.z - path_pts[seg_idx - 1].z,
            }
        };

        let len = (dir.x * dir.x + dir.y * dir.y + dir.z * dir.z).sqrt();
        if len < 1e-10 {
            continue;
        }

        let (dx, dy, dz) = (dir.x / len, dir.y / len, dir.z / len);

        // Build a local coordinate system
        let up = if dz.abs() < 0.9 {
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 }
        } else {
            c::Vector3D { x: 1.0, y: 0.0, z: 0.0 }
        };

        let right = c::Vector3D {
            x: dy * up.z - dz * up.y,
            y: dz * up.x - dx * up.z,
            z: dx * up.y - dy * up.x,
        };
        let rlen = (right.x * right.x + right.y * right.y + right.z * right.z).sqrt();
        if rlen < 1e-10 {
            continue;
        }
        let (rx, ry, rz) = (right.x / rlen, right.y / rlen, right.z / rlen);

        let fwd = c::Vector3D {
            x: ry * dz - rz * dy,
            y: rz * dx - rx * dz,
            z: rx * dy - ry * dx,
        };

        // Generate ring vertices around the path point
        for i in 0..segs {
            let a = 2.0 * std::f64::consts::PI * i as f64 / segs as f64;
            let (ca, sa) = (a.cos(), a.sin());
            vertices.push(c::Point3D {
                x: p.x + radius * (ca * rx + sa * fwd.x),
                y: p.y + radius * (ca * ry + sa * fwd.y),
                z: p.z + radius * (ca * rz + sa * fwd.z),
            });
            normals.push(c::Vector3D {
                x: ca * rx + sa * fwd.x,
                y: ca * ry + sa * fwd.y,
                z: ca * rz + sa * fwd.z,
            });
        }
    }

    // Generate connecting quads
    let rings = path_pts.len();
    for ring_idx in 0..(rings - 1) {
        for i in 0..segs {
            let nxt = (i + 1) % segs;
            let a = (ring_idx * segs + i) as i32;
            let b = (ring_idx * segs + nxt) as i32;
            let c = ((ring_idx + 1) * segs + i) as i32;
            let d = ((ring_idx + 1) * segs + nxt) as i32;
            faces.push(c::FaceIndices {
                vertex_indices: vec![a, c, b],
            });
            faces.push(c::FaceIndices {
                vertex_indices: vec![b, c, d],
            });
        }
    }

    if vertices.is_empty() {
        return None;
    }

    let mesh = c::MeshGeometry { vertices, normals, uvs: vec![], faces };
    let bbox = omnim_model::geometry::compute_bbox(&mesh);
    Some(c::Geometry {
        id: String::new(),
        r#type: c::GeometryType::Mesh as i32,
        representation: Some(c::geometry::Representation::Mesh(mesh)),
        bbox: Some(bbox),
        level_of_detail: 0,
    })
}

/// 从 IFCPOLYLINE / IFCTRIMMEDCURVE 提取路径点
fn extract_curve_points(doc: &IfcDocument, curve: &IfcEntity) -> Vec<c::Point3D> {
    match curve.entity_type.as_str() {
        "IFCPOLYLINE" => {
            let points_list = match curve.args.get(0) {
                Some(IfcValue::List(items)) => items,
                _ => return vec![],
            };
            let mut pts = Vec::new();
            for item in points_list {
                if let IfcValue::Ref(pt_id) = item {
                    if let Some(pt_entity) = doc.get(*pt_id) {
                        if let Some(IfcValue::List(coords)) = pt_entity.args.get(0) {
                            if let (Some(x), Some(y)) = (
                                crate::coord_to_f64(coords.get(0)),
                                crate::coord_to_f64(coords.get(1)),
                            ) {
                                let z = crate::coord_to_f64(coords.get(2)).unwrap_or(0.0);
                                pts.push(c::Point3D { x, y, z });
                            }
                        }
                    }
                }
            }
            pts
        }
        _ => vec![],
    }
}

// ---- CSG 实体 (CSGSolid) ----

/// IFCCSGSOLID → CSG 树
///
/// IFCCSGSOLID 包含一个 IfcCsgTreeRoot (通常是 IfcBooleanResult)
/// 递归处理即可
fn convert_csg_solid(doc: &IfcDocument, item: &IfcEntity) -> Option<c::Geometry> {
    // arg[0] = TreeRootExpression (IfcBooleanResult)
    let root_ref = doc.get_ref(item, 0)?;
    convert_ifc_item(doc, root_ref)
}

// ============================================================
// 局部放置解析 (LocalPlacement)
// ============================================================

/// 增强的放置解析：支持完整变换矩阵 + 层级链
#[allow(dead_code)]
pub(crate) fn extract_placement_enhanced(doc: &IfcDocument, entity: &IfcEntity) -> Option<c::Transform> {
    let placement_ref = doc.get_ref(entity, 5)?;
    build_placement_chain(doc, placement_ref)
}

/// 递归构建放置链 (跟踪 PlacementRelTo)
fn build_placement_chain(doc: &IfcDocument, placement_id: u64) -> Option<c::Transform> {
    let placement = doc.get(placement_id)?;
    if placement.entity_type != "IFCLOCALPLACEMENT" {
        return None;
    }

    // Get relative placement (IfcAxis2Placement3D)
    let rel_placement_ref = doc.get_ref(placement, 1)?;
    let rel_placement = doc.get(rel_placement_ref)?;

    let local_transform = extract_axis2_placement_3d(doc, rel_placement)?;

    // Check for parent placement (PlacementRelTo)
    let parent_transform = match placement.args.get(0) {
        Some(IfcValue::Ref(parent_id)) => build_placement_chain(doc, *parent_id),
        _ => None,
    };

    match parent_transform {
        Some(parent) => {
            // Combine parent transform with local transform
            // For now, just add translations (full matrix composition is complex without a matrix library)
            let combined = c::Transform {
                matrix: vec![],
                translation: Some(c::Point3D {
                    x: local_transform.translation.as_ref().map(|t| t.x).unwrap_or(0.0)
                        + parent.translation.as_ref().map(|t| t.x).unwrap_or(0.0),
                    y: local_transform.translation.as_ref().map(|t| t.y).unwrap_or(0.0)
                        + parent.translation.as_ref().map(|t| t.y).unwrap_or(0.0),
                    z: local_transform.translation.as_ref().map(|t| t.z).unwrap_or(0.0)
                        + parent.translation.as_ref().map(|t| t.z).unwrap_or(0.0),
                }),
                rotation: local_transform.rotation.or(parent.rotation),
                scale: None,
            };
            Some(combined)
        }
        None => Some(local_transform),
    }
}

/// 从 IfcAxis2Placement3D 提取变换 (位置 + 旋转)
///
/// 根据 IFC 规范:
/// - Axis (arg[1]) = 局部 Z 轴方向
/// - RefDirection (arg[2]) = 局部 X 轴方向 (将正交化为 X 轴)
/// - Y 轴 = Z × X (叉积)
/// 最后将旋转矩阵 [X, Y, Z] 转换为四元数
fn extract_axis2_placement_3d(doc: &IfcDocument, placement: &IfcEntity) -> Option<c::Transform> {
    // arg[0] = Location (IfcCartesianPoint)
    // arg[1] = Axis (Z direction, optional)
    // arg[2] = RefDirection (X direction, optional)

    let location_ref = doc.get_ref(placement, 0)?;
    let location = doc.get(location_ref)?;
    let (lx, ly, lz) = extract_point_3d(doc, location)?;

    // Parse axis direction (Z axis)
    let (azx, azy, azz) = match placement.args.get(1) {
        Some(IfcValue::Ref(dir_ref)) => {
            let dir = doc.get(*dir_ref)?;
            extract_direction_3d(doc, dir).unwrap_or((0.0, 0.0, 1.0))
        }
        _ => (0.0, 0.0, 1.0),
    };

    // Parse ref direction (X axis)
    let (arx, ary, arz) = match placement.args.get(2) {
        Some(IfcValue::Ref(dir_ref)) => {
            let dir = doc.get(*dir_ref)?;
            extract_direction_3d(doc, dir).unwrap_or((1.0, 0.0, 0.0))
        }
        _ => (1.0, 0.0, 0.0),
    };

    // Orthogonalize X against Z: remove Z-projection from X
    let dot_xz = arx * azx + ary * azy + arz * azz;
    let (ox, oy, oz) = (arx - dot_xz * azx, ary - dot_xz * azy, arz - dot_xz * azz);
    let len = (ox * ox + oy * oy + oz * oz).sqrt();
    let (xx, xy, xz) = if len > 1e-10 {
        (ox / len, oy / len, oz / len)
    } else {
        // If X is parallel to Z, pick an arbitrary orthogonal vector
        if azx.abs() < 0.9 {
            (1.0, 0.0, 0.0)
        } else {
            (0.0, 1.0, 0.0)
        }
    };

    // Y = Z × X
    let (yx, yy, yz) = (
        azy * xz - azz * xy,
        azz * xx - azx * xz,
        azx * xy - azy * xx,
    );

    // Build rotation quaternion from the matrix [X Y Z]
    // Matrix columns: X=(xx,xy,xz), Y=(yx,yy,yz), Z=(azx,azy,azz)
    let trace = xx + yy + azz;
    let (qw, qx, qy, qz) = if trace > 0.0 {
        let s = 0.5 / (trace + 1.0).sqrt();
        (0.25 / s, (yz - azy) * s, (azx - xz) * s, (xy - yx) * s)
    } else if xx > yy && xx > azz {
        let s = 2.0 * (1.0 + xx - yy - azz).sqrt();
        ((yz - azy) / s, 0.25 * s, (xy + yx) / s, (azx + xz) / s)
    } else if yy > azz {
        let s = 2.0 * (1.0 + yy - xx - azz).sqrt();
        ((azx - xz) / s, (xy + yx) / s, 0.25 * s, (yz + azy) / s)
    } else {
        let s = 2.0 * (1.0 + azz - xx - yy).sqrt();
        ((xy - yx) / s, (azx + xz) / s, (yz + azy) / s, 0.25 * s)
    };

    Some(c::Transform {
        matrix: vec![],
        translation: Some(c::Point3D { x: lx, y: ly, z: lz }),
        rotation: Some(c::Rotation { x: qx, y: qy, z: qz, w: qw }),
        scale: None,
    })
}

fn extract_direction_3d(_doc: &IfcDocument, entity: &IfcEntity) -> Option<(f64, f64, f64)> {
    let dir_list = match entity.args.get(0) {
        Some(IfcValue::List(items)) => items,
        _ => return None,
    };
    let x = crate::coord_to_f64(dir_list.get(0))?;
    let y = crate::coord_to_f64(dir_list.get(1))?;
    let z = crate::coord_to_f64(dir_list.get(2))?;
    let len = (x * x + y * y + z * z).sqrt();
    if len < 1e-10 {
        return Some((0.0, 0.0, 1.0));
    }
    Some((x / len, y / len, z / len))
}
