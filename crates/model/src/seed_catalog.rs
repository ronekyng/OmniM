use crate::catalog::{ComponentDef, ConnectorDef, ParamDef};
use crate::series_store::{Answer, AnswerOutcome, Selector, Series, SeriesStore, SpecComponent};
use crate::feature_template::{FeatureTemplateStep, FeatureTreeTemplate, FeatureTreeRegistry};
use crate::profile::ProfileType;
use omnim_core as c;

/// Populate the catalog store with built-in structural and piping component families.
pub fn seed_catalog(catalog: &mut crate::catalog::CatalogStore) {
    seed_structural_beam_family(catalog);
    seed_structural_column_family(catalog);
    seed_heb_hem_family(catalog);
    seed_channel_family(catalog);
    seed_t_section_family(catalog);
    seed_piping_components(catalog);
    seed_concrete_components(catalog);
    seed_hollow_section_family(catalog);
    seed_angle_section_family(catalog);
    seed_building_components(catalog);
    seed_road_components(catalog);
    seed_equipment_stubs(catalog);
    seed_rebar_components(catalog);
    seed_domain_extension_components(catalog);
}

/// Register feature tree templates for all catalog components.
/// Returns a map of component name → tree ID for later linking.
pub fn seed_feature_trees(registry: &mut FeatureTreeRegistry) -> Vec<(String, String)> {
    let mut links = Vec::new();

    let ipe_id = register_ipe_beam_template(registry);
    links.push(("Steel-Beam-IPE".into(), ipe_id.clone()));
    links.push(("IPE240".into(), ipe_id));

    let hea_id = register_hea_column_template(registry);
    links.push(("Steel-Column-HEA".into(), hea_id));

    let heb_id = register_heb_column_template(registry);
    links.push(("Steel-Column-HEB".into(), heb_id));

    let hem_id = register_hem_column_template(registry);
    links.push(("Steel-Column-HEM".into(), hem_id));

    let upe_id = register_upe_channel_template(registry);
    links.push(("Steel-Channel-UPE".into(), upe_id));

    let upn_id = register_upn_channel_template(registry);
    links.push(("Steel-Channel-UPN".into(), upn_id));

    let t_id = register_t_section_template(registry);
    links.push(("Steel-T-Section".into(), t_id));

    let pipe_id = register_pipe_template(registry);
    links.push(("Pipe-CS".into(), pipe_id));

    let elbow_id = register_elbow_template(registry);
    links.push(("Elbow-LR-CS".into(), elbow_id));

    let cb_id = register_concrete_beam_template(registry);
    links.push(("Concrete-Beam-Rect".into(), cb_id));

    let cc_id = register_concrete_column_template(registry);
    links.push(("Concrete-Column-Rect".into(), cc_id));

    let ccc_id = register_concrete_circular_column_template(registry);
    links.push(("Concrete-Column-Circular".into(), ccc_id));

    let blk_id = register_concrete_block_template(registry);
    links.push(("Concrete-Footing-Isolated".into(), blk_id.clone()));
    links.push(("Concrete-Footing-Raft".into(), blk_id));

    let strip_id = register_concrete_strip_template(registry);
    links.push(("Concrete-Footing-Strip".into(), strip_id));

    let chs_id = register_chs_template(registry);
    links.push(("Steel-CHS".into(), chs_id));

    let shs_id = register_shs_template(registry);
    links.push(("Steel-SHS".into(), shs_id));

    let angle_id = register_angle_template(registry);
    links.push(("Steel-Angle".into(), angle_id));

    let wall_id = register_concrete_wall_template(registry);
    links.push(("Concrete-Wall".into(), wall_id));

    let slab_id = register_concrete_slab_template(registry);
    links.push(("Concrete-Slab".into(), slab_id));

    let door_id = register_door_template(registry);
    links.push(("Door-Single".into(), door_id));

    let window_id = register_window_template(registry);
    links.push(("Window-Fixed".into(), window_id));

    let road_id = register_road_template(registry);
    links.push(("Road-Asphalt".into(), road_id.clone()));
    links.push(("Road-Concrete".into(), road_id));

    let tank_id = register_tank_template(registry);
    links.push(("VerticalTank".into(), tank_id));

    let tee_id = register_equal_tee_template(registry);
    links.push(("EqualTee-CS".into(), tee_id));

    let reducer_id = register_reducer_template(registry);
    links.push(("Reducer-Conc-CS".into(), reducer_id));

    let flange_id = register_flange_template(registry);
    links.push(("Flange-WN-CS".into(), flange_id));

    let valve_id = register_gate_valve_template(registry);
    links.push(("GateValve-CS".into(), valve_id));

    let pump_id = register_centrifugal_pump_template(registry);
    links.push(("CentrifugalPump".into(), pump_id));

    let rebar_id = register_rebar_cage_template(registry);
    links.push(("Rebar-Cage".into(), rebar_id));

    let duct_id = register_duct_rect_template(registry);
    links.push(("Duct-Rect".into(), duct_id));

    let tray_id = register_cable_tray_template(registry);
    links.push(("CableTray".into(), tray_id));

    let hex_id = register_heat_exchanger_template(registry);
    links.push(("HeatExchanger".into(), hex_id));

    let compressor_id = register_compressor_template(registry);
    links.push(("Compressor".into(), compressor_id));

    let manhole_id = register_manhole_template(registry);
    links.push(("Manhole".into(), manhole_id));

    let gear_id = register_gear_template(registry);
    links.push(("Gear".into(), gear_id));

    links
}

/// Link feature tree templates to component definitions in the catalog.
pub fn link_templates(catalog: &mut crate::catalog::CatalogStore, links: &[(String, String)]) {
    for (comp_name, tree_id) in links {
        if let Some(comp) = catalog.find_by_name_mut(comp_name) {
            comp.feature_tree_id = tree_id.clone();
        }
    }
}

// ============================================================
// Feature Tree Templates
// ============================================================

fn register_ipe_beam_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("ipe-beam-extrude")
        .bind("height", "height")
        .bind("width", "width")
        .bind("web_thickness", "web_thickness")
        .bind("flange_thickness", "flange_thickness")
        .bind("length", "length")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::IShape { height: 240.0, width: 120.0, web_thickness: 6.2, flange_thickness: 9.8 },
            "length",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_hea_column_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("hea-column-extrude")
        .bind("height", "height")
        .bind("width", "width")
        .bind("column_height", "column_height")
        .bind("web_thickness", "web_thickness")
        .bind("flange_thickness", "flange_thickness")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::IShape { height: 300.0, width: 300.0, web_thickness: 8.0, flange_thickness: 14.0 },
            "column_height",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_pipe_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("pipe-extrude")
        .bind("DN", "DN")
        .bind("schedule", "schedule")
        .bind("length", "length")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::PipeSchedule { segments: 32 },
            "length",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_elbow_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("elbow-sweep")
        .bind("DN", "DN")
        .bind("angle", "angle")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::Circle { radius: 50.0, segments: 32 },
            "DN",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_concrete_beam_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("concrete-beam-extrude")
        .bind("width", "width")
        .bind("depth", "depth")
        .bind("length", "length")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::Rectangle { width: 300.0, height: 500.0 },
            "length",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_concrete_column_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("concrete-column-extrude")
        .bind("width", "width")
        .bind("depth", "depth")
        .bind("column_height", "column_height")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::Rectangle { width: 300.0, height: 300.0 },
            "column_height",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_concrete_circular_column_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("concrete-circular-column-extrude")
        .bind("diameter", "diameter")
        .bind("column_height", "column_height")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::Circle { radius: 200.0, segments: 32 },
            "column_height",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_concrete_block_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("concrete-block-extrude")
        .bind("width", "width")
        .bind("depth", "depth")
        .bind("thickness", "thickness")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::Rectangle { width: 1500.0, height: 1500.0 },
            "thickness",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_concrete_strip_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("concrete-strip-extrude")
        .bind("length", "length")
        .bind("width", "width")
        .bind("thickness", "thickness")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::Rectangle { width: 600.0, height: 300.0 },
            "length",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_chs_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("chs-extrude")
        .bind("diameter", "diameter")
        .bind("thickness", "thickness")
        .bind("length", "length")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::Annulus { outer_radius: 57.15, inner_radius: 52.15, segments: 32 },
            "length",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_shs_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("shs-extrude")
        .bind("width", "width")
        .bind("thickness", "thickness")
        .bind("length", "length")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::BoxShape { width: 100.0, height: 100.0, thickness: 5.0 },
            "length",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_angle_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("angle-extrude")
        .bind("leg1", "leg1")
        .bind("leg2", "leg2")
        .bind("thickness", "thickness")
        .bind("length", "length")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::LShape { leg1: 50.0, leg2: 50.0, thickness: 5.0 },
            "length",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_heb_column_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("heb-column-extrude")
        .bind("height", "height")
        .bind("width", "width")
        .bind("web_thickness", "web_thickness")
        .bind("flange_thickness", "flange_thickness")
        .bind("column_height", "column_height")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::IShape { height: 300.0, width: 300.0, web_thickness: 11.0, flange_thickness: 19.0 },
            "column_height",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_hem_column_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("hem-column-extrude")
        .bind("height", "height")
        .bind("width", "width")
        .bind("web_thickness", "web_thickness")
        .bind("flange_thickness", "flange_thickness")
        .bind("column_height", "column_height")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::IShape { height: 340.0, width: 340.0, web_thickness: 21.0, flange_thickness: 35.0 },
            "column_height",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_upe_channel_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("upe-channel-extrude")
        .bind("height", "height")
        .bind("width", "width")
        .bind("web_thickness", "web_thickness")
        .bind("flange_thickness", "flange_thickness")
        .bind("length", "length")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::UShape { height: 300.0, width: 100.0, web_thickness: 10.0, flange_thickness: 16.0 },
            "length",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_upn_channel_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("upn-channel-extrude")
        .bind("height", "height")
        .bind("width", "width")
        .bind("web_thickness", "web_thickness")
        .bind("flange_thickness", "flange_thickness")
        .bind("length", "length")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::UShape { height: 300.0, width: 100.0, web_thickness: 10.0, flange_thickness: 16.0 },
            "length",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_t_section_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("t-section-extrude")
        .bind("height", "height")
        .bind("width", "width")
        .bind("web_thickness", "web_thickness")
        .bind("flange_thickness", "flange_thickness")
        .bind("length", "length")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::TShape { height: 100.0, width: 100.0, web_thickness: 10.0, flange_thickness: 14.0 },
            "length",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_concrete_wall_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("concrete-wall-extrude")
        .bind("length", "length")
        .bind("thickness", "thickness")
        .bind("height", "height")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::ParametricRect { width_param: "length".into(), height_param: "thickness".into() },
            "height",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_concrete_slab_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("concrete-slab-extrude")
        .bind("length", "length")
        .bind("width", "width")
        .bind("thickness", "thickness")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::Rectangle { width: 6000.0, height: 4000.0 },
            "thickness",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_door_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("door-extrude")
        .bind("width", "width")
        .bind("height", "height")
        .bind("frame_depth", "frame_depth")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::Rectangle { width: 900.0, height: 2100.0 },
            "frame_depth",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_window_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("window-extrude")
        .bind("width", "width")
        .bind("height", "height")
        .bind("frame_depth", "frame_depth")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::Rectangle { width: 1200.0, height: 1500.0 },
            "frame_depth",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_road_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("road-extrude")
        .bind("length", "length")
        .bind("width", "width")
        .bind("thickness", "thickness")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::Rectangle { width: 3500.0, height: 150.0 },
            "length",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_tank_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("tank-extrude")
        .bind("diameter", "diameter")
        .bind("height", "height")
        .bind("wall_thickness", "wall_thickness")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::Annulus { outer_radius: 1500.0, inner_radius: 1492.0, segments: 48 },
            "height",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

// ============================================================
// Piping Fitting & Equipment Templates (domain-extension batch)
// ============================================================

fn register_equal_tee_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("equal-tee-extrude")
        .bind("DN", "DN")
        .bind("schedule", "schedule")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::TShape { height: 200.0, width: 200.0, web_thickness: 20.0, flange_thickness: 20.0 },
            "DN",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_reducer_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("reducer-loft")
        .bind("DN_large", "DN_large")
        .bind("DN_small", "DN_small")
        .bind("length", "length")
        .step(FeatureTemplateStep::loft(
            "main",
            ProfileType::Circle { radius: 75.0, segments: 32 },
            ProfileType::Circle { radius: 50.0, segments: 32 },
        ));
    registry.register(template)
}

fn register_flange_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("flange-revolve")
        .bind("DN", "DN")
        .bind("pressure_class", "pressure_class")
        .step(FeatureTemplateStep::revolve(
            "main",
            ProfileType::LShape { leg1: 100.0, leg2: 20.0, thickness: 15.0 },
            "360",
        ));
    registry.register(template)
}

fn register_gate_valve_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("gate-valve-extrude")
        .bind("DN", "DN")
        .bind("pressure_class", "pressure_class")
        .step(FeatureTemplateStep::extrude(
            "body",
            ProfileType::Rectangle { width: 200.0, height: 300.0 },
            "DN",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_centrifugal_pump_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("centrifugal-pump-extrude")
        .bind("DN", "DN")
        .bind("height", "height")
        .bind("width", "width")
        .step(FeatureTemplateStep::extrude(
            "body",
            ProfileType::Rectangle { width: 400.0, height: 500.0 },
            "height",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_rebar_cage_template(registry: &mut FeatureTreeRegistry) -> String {
    // The cage body is represented as an annular ring (main-bar circle)
    // extruded along the cage length. Default ring radius 200 mm with
    // 16 mm bars gives outer 208 / inner 192.
    let template = FeatureTreeTemplate::new("rebar-cage-extrude")
        .bind("bar_ring_radius", "bar_ring_radius")
        .bind("diameter", "diameter")
        .bind("length", "length")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::Annulus { outer_radius: 208.0, inner_radius: 192.0, segments: 24 },
            "length",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

// ── MEP / civil / plant / mechanical domain extension templates ──

fn register_duct_rect_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("duct-rect-extrude")
        .bind("width", "width")
        .bind("height", "height")
        .bind("length", "length")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::Rectangle { width: 500.0, height: 250.0 },
            "length",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_cable_tray_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("cable-tray-extrude")
        .bind("width", "width")
        .bind("height", "height")
        .bind("thickness", "thickness")
        .bind("length", "length")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::UShape { height: 100.0, width: 300.0, web_thickness: 3.0, flange_thickness: 3.0 },
            "length",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_heat_exchanger_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("heat-exchanger-extrude")
        .bind("shell_diameter", "shell_diameter")
        .bind("shell_length", "shell_length")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::Circle { radius: 250.0, segments: 32 },
            "shell_length",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_compressor_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("compressor-extrude")
        .bind("length", "length")
        .bind("width", "width")
        .bind("height", "height")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::Rectangle { width: 1200.0, height: 800.0 },
            "length",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_manhole_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("manhole-extrude")
        .bind("diameter", "diameter")
        .bind("depth", "depth")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::Circle { radius: 600.0, segments: 32 },
            "depth",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

fn register_gear_template(registry: &mut FeatureTreeRegistry) -> String {
    let template = FeatureTreeTemplate::new("gear-extrude")
        .bind("module", "module")
        .bind("teeth", "teeth")
        .bind("width", "width")
        .step(FeatureTemplateStep::extrude(
            "main",
            ProfileType::Circle { radius: 100.0, segments: 48 },
            "width",
            c::Vector3D { x: 0.0, y: 0.0, z: 1.0 },
        ));
    registry.register(template)
}

/// Populate the series store with built-in spec decision trees.
pub fn seed_series(series_store: &mut SeriesStore) {
    seed_structural_series(series_store);
    seed_cs150_series(series_store);
    seed_cs300_series(series_store);
    seed_highway_gb_series(series_store);
}

// ============================================================
// Structural: IPE Beam Family
// ============================================================

fn seed_structural_beam_family(catalog: &mut crate::catalog::CatalogStore) {
    let mut comp = ComponentDef::new("Steel-Beam-IPE", "Structural/Steel/Beam");
    comp.description = "European I-beam (IPE) per EN 10365".into();
    comp.ifc_class = "IfcBeam".into();
    comp.standard_reference = "EN 10365".into();
    comp.domain = "structural".into();
    comp.tags = vec!["steel".into(), "beam".into(), "ipe".into()];
    comp.default_material = "S355".into();
    comp.allowed_materials = vec!["S235".into(), "S275".into(), "S355".into(), "S460".into()];

    comp.params.push(ParamDef::new("height", 2).with_default(240.0).with_range(80.0, 600.0).with_step(10.0).with_unit("mm"));
    comp.params.push(ParamDef::new("width", 2).with_default(120.0).with_range(46.0, 215.0).with_step(1.0).with_unit("mm"));
    comp.params.push(ParamDef::new("web_thickness", 2).with_default(6.2).with_range(1.9, 12.0).with_step(0.1).with_unit("mm"));
    comp.params.push(ParamDef::new("flange_thickness", 2).with_default(9.8).with_range(3.8, 19.0).with_step(0.1).with_unit("mm"));
    comp.params.push(ParamDef::new("length", 2).with_default(6000.0).with_range(1000.0, 12000.0).with_step(100.0).with_unit("mm"));

    comp.connectors.push(ConnectorDef::new("start", "Start End")
        .with_position("0", "0", "0")
        .with_direction("0", "0", "1")
        .with_role(1));

    comp.connectors.push(ConnectorDef::new("end", "End End")
        .with_position("0", "0", "length")
        .with_direction("0", "0", "1")
        .with_role(2));

    comp.published = true;
    catalog.insert(comp);

    // IPE240 quick-define
    let mut ipe240 = ComponentDef::new("IPE240", "Structural/Steel/Beam");
    ipe240.description = "IPE 240 — height=240mm, width=120mm".into();
    ipe240.ifc_class = "IfcBeam".into();
    ipe240.domain = "structural".into();
    ipe240.tags = vec!["steel".into(), "ipe240".into()];
    ipe240.default_material = "S355".into();
    ipe240.params.push(ParamDef::new("height", 2).with_default(240.0).with_unit("mm"));
    ipe240.params.push(ParamDef::new("width", 2).with_default(120.0).with_unit("mm"));
    ipe240.params.push(ParamDef::new("length", 2).with_default(6000.0).with_range(1000.0, 12000.0).with_step(100.0).with_unit("mm"));
    ipe240.published = true;
    catalog.insert(ipe240);
}

// ============================================================
// Structural: HEA Column Family
// ============================================================

fn seed_structural_column_family(catalog: &mut crate::catalog::CatalogStore) {
    let mut comp = ComponentDef::new("Steel-Column-HEA", "Structural/Steel/Column");
    comp.description = "Wide-flange column (HEA) per EN 10365".into();
    comp.ifc_class = "IfcColumn".into();
    comp.standard_reference = "EN 10365".into();
    comp.domain = "structural".into();
    comp.tags = vec!["steel".into(), "column".into(), "hea".into()];
    comp.default_material = "S355".into();

    comp.params.push(ParamDef::new("height", 2).with_default(300.0).with_range(100.0, 1100.0).with_step(10.0).with_unit("mm"));
    comp.params.push(ParamDef::new("width", 2).with_default(300.0).with_range(96.0, 400.0).with_step(1.0).with_unit("mm"));
    comp.params.push(ParamDef::new("web_thickness", 2).with_default(8.0).with_range(4.0, 20.0).with_step(0.5).with_unit("mm"));
    comp.params.push(ParamDef::new("flange_thickness", 2).with_default(14.0).with_range(6.0, 30.0).with_step(0.5).with_unit("mm"));
    comp.params.push(ParamDef::new("column_height", 2).with_default(4200.0).with_range(1000.0, 10000.0).with_step(100.0).with_unit("mm"));

    comp.connectors.push(ConnectorDef::new("base", "Base")
        .with_position("0", "0", "0").with_direction("0", "0", "-1").with_role(1));
    comp.connectors.push(ConnectorDef::new("top", "Top")
        .with_position("0", "0", "column_height").with_direction("0", "0", "1").with_role(2));

    comp.published = true;
    catalog.insert(comp);
}

fn seed_heb_hem_family(catalog: &mut crate::catalog::CatalogStore) {
    let mut heb = ComponentDef::new("Steel-Column-HEB", "Structural/Steel/Column");
    heb.description = "Wide-flange column (HEB) per EN 10365".into();
    heb.ifc_class = "IfcColumn".into();
    heb.standard_reference = "EN 10365".into();
    heb.domain = "structural".into();
    heb.tags = vec!["steel".into(), "column".into(), "heb".into()];
    heb.default_material = "S355".into();
    heb.allowed_materials = vec!["S235".into(), "S275".into(), "S355".into(), "S460".into()];

    heb.params.push(ParamDef::new("height", 2).with_default(300.0).with_range(100.0, 1100.0).with_step(10.0).with_unit("mm"));
    heb.params.push(ParamDef::new("width", 2).with_default(300.0).with_range(100.0, 400.0).with_step(1.0).with_unit("mm"));
    heb.params.push(ParamDef::new("web_thickness", 2).with_default(11.0).with_range(4.0, 25.0).with_step(0.5).with_unit("mm"));
    heb.params.push(ParamDef::new("flange_thickness", 2).with_default(19.0).with_range(7.0, 40.0).with_step(0.5).with_unit("mm"));
    heb.params.push(ParamDef::new("column_height", 2).with_default(4200.0).with_range(1000.0, 10000.0).with_step(100.0).with_unit("mm"));

    heb.connectors.push(ConnectorDef::new("base", "Base")
        .with_position("0", "0", "0").with_direction("0", "0", "-1").with_role(1));
    heb.connectors.push(ConnectorDef::new("top", "Top")
        .with_position("0", "0", "column_height").with_direction("0", "0", "1").with_role(2));

    heb.published = true;
    catalog.insert(heb);

    let mut hem = ComponentDef::new("Steel-Column-HEM", "Structural/Steel/Column");
    hem.description = "Heavy wide-flange column (HEM) per EN 10365".into();
    hem.ifc_class = "IfcColumn".into();
    hem.standard_reference = "EN 10365".into();
    hem.domain = "structural".into();
    hem.tags = vec!["steel".into(), "column".into(), "hem".into()];
    hem.default_material = "S355".into();
    hem.allowed_materials = vec!["S235".into(), "S275".into(), "S355".into(), "S460".into()];

    hem.params.push(ParamDef::new("height", 2).with_default(340.0).with_range(120.0, 1100.0).with_step(10.0).with_unit("mm"));
    hem.params.push(ParamDef::new("width", 2).with_default(340.0).with_range(120.0, 400.0).with_step(1.0).with_unit("mm"));
    hem.params.push(ParamDef::new("web_thickness", 2).with_default(21.0).with_range(6.0, 30.0).with_step(0.5).with_unit("mm"));
    hem.params.push(ParamDef::new("flange_thickness", 2).with_default(35.0).with_range(9.0, 45.0).with_step(0.5).with_unit("mm"));
    hem.params.push(ParamDef::new("column_height", 2).with_default(4200.0).with_range(1000.0, 10000.0).with_step(100.0).with_unit("mm"));

    hem.connectors.push(ConnectorDef::new("base", "Base")
        .with_position("0", "0", "0").with_direction("0", "0", "-1").with_role(1));
    hem.connectors.push(ConnectorDef::new("top", "Top")
        .with_position("0", "0", "column_height").with_direction("0", "0", "1").with_role(2));

    hem.published = true;
    catalog.insert(hem);
}

fn seed_channel_family(catalog: &mut crate::catalog::CatalogStore) {
    let mut upe = ComponentDef::new("Steel-Channel-UPE", "Structural/Steel/Channel");
    upe.description = "European U-channel (UPE) with parallel flanges per EN 10365".into();
    upe.ifc_class = "IfcBeam".into();
    upe.standard_reference = "EN 10365".into();
    upe.domain = "structural".into();
    upe.tags = vec!["steel".into(), "channel".into(), "upe".into()];
    upe.default_material = "S355".into();
    upe.allowed_materials = vec!["S235".into(), "S275".into(), "S355".into()];

    upe.params.push(ParamDef::new("height", 2).with_default(300.0).with_range(80.0, 400.0).with_step(10.0).with_unit("mm"));
    upe.params.push(ParamDef::new("width", 2).with_default(100.0).with_range(46.0, 115.0).with_step(1.0).with_unit("mm"));
    upe.params.push(ParamDef::new("web_thickness", 2).with_default(10.0).with_range(3.0, 14.0).with_step(0.5).with_unit("mm"));
    upe.params.push(ParamDef::new("flange_thickness", 2).with_default(16.0).with_range(5.0, 19.0).with_step(0.5).with_unit("mm"));
    upe.params.push(ParamDef::new("length", 2).with_default(6000.0).with_range(1000.0, 12000.0).with_step(100.0).with_unit("mm"));

    upe.connectors.push(ConnectorDef::new("start", "Start End")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(1));
    upe.connectors.push(ConnectorDef::new("end", "End End")
        .with_position("0", "0", "length").with_direction("0", "0", "1").with_role(2));

    upe.published = true;
    catalog.insert(upe);

    let mut upn = ComponentDef::new("Steel-Channel-UPN", "Structural/Steel/Channel");
    upn.description = "European U-channel (UPN) with sloping flanges per EN 10365".into();
    upn.ifc_class = "IfcBeam".into();
    upn.standard_reference = "EN 10365".into();
    upn.domain = "structural".into();
    upn.tags = vec!["steel".into(), "channel".into(), "upn".into()];
    upn.default_material = "S355".into();
    upn.allowed_materials = vec!["S235".into(), "S275".into(), "S355".into()];

    upn.params.push(ParamDef::new("height", 2).with_default(300.0).with_range(80.0, 400.0).with_step(10.0).with_unit("mm"));
    upn.params.push(ParamDef::new("width", 2).with_default(100.0).with_range(45.0, 110.0).with_step(1.0).with_unit("mm"));
    upn.params.push(ParamDef::new("web_thickness", 2).with_default(10.0).with_range(3.0, 14.0).with_step(0.5).with_unit("mm"));
    upn.params.push(ParamDef::new("flange_thickness", 2).with_default(16.0).with_range(5.0, 19.0).with_step(0.5).with_unit("mm"));
    upn.params.push(ParamDef::new("length", 2).with_default(6000.0).with_range(1000.0, 12000.0).with_step(100.0).with_unit("mm"));

    upn.connectors.push(ConnectorDef::new("start", "Start End")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(1));
    upn.connectors.push(ConnectorDef::new("end", "End End")
        .with_position("0", "0", "length").with_direction("0", "0", "1").with_role(2));

    upn.published = true;
    catalog.insert(upn);
}

fn seed_t_section_family(catalog: &mut crate::catalog::CatalogStore) {
    let mut t_sec = ComponentDef::new("Steel-T-Section", "Structural/Steel/Tee");
    t_sec.description = "T-section (tee bar) per EN 10055".into();
    t_sec.ifc_class = "IfcBeam".into();
    t_sec.standard_reference = "EN 10055".into();
    t_sec.domain = "structural".into();
    t_sec.tags = vec!["steel".into(), "tee".into(), "T".into()];
    t_sec.default_material = "S355".into();
    t_sec.allowed_materials = vec!["S235".into(), "S275".into(), "S355".into()];

    t_sec.params.push(ParamDef::new("height", 2).with_default(100.0).with_range(50.0, 200.0).with_step(5.0).with_unit("mm"));
    t_sec.params.push(ParamDef::new("width", 2).with_default(100.0).with_range(50.0, 200.0).with_step(5.0).with_unit("mm"));
    t_sec.params.push(ParamDef::new("web_thickness", 2).with_default(10.0).with_range(4.0, 20.0).with_step(0.5).with_unit("mm"));
    t_sec.params.push(ParamDef::new("flange_thickness", 2).with_default(14.0).with_range(5.0, 25.0).with_step(0.5).with_unit("mm"));
    t_sec.params.push(ParamDef::new("length", 2).with_default(6000.0).with_range(1000.0, 12000.0).with_step(100.0).with_unit("mm"));

    t_sec.connectors.push(ConnectorDef::new("start", "Start End")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(1));
    t_sec.connectors.push(ConnectorDef::new("end", "End End")
        .with_position("0", "0", "length").with_direction("0", "0", "1").with_role(2));

    t_sec.published = true;
    catalog.insert(t_sec);
}

// ============================================================
// Piping: Basic Pipe + Elbow + Tee + Reducer + Flange + Valve
// ============================================================

fn seed_piping_components(catalog: &mut crate::catalog::CatalogStore) {
    let mut pipe = ComponentDef::new("Pipe-CS", "Piping/Pipe");
    pipe.description = "Carbon steel pipe per ASME B36.10M".into();
    pipe.ifc_class = "IfcPipeSegment".into();
    pipe.standard_reference = "ASME B36.10M".into();
    pipe.domain = "piping".into();
    pipe.tags = vec!["pipe".into(), "carbon-steel".into()];
    pipe.default_material = "A106-B".into();

    pipe.params.push(ParamDef::new("DN", 3).with_default(100.0).with_range(15.0, 600.0).with_step(1.0).with_unit("mm"));
    pipe.params.push(ParamDef::new("schedule", 4).with_default(0.0).with_unit("").with_choices(vec!["5S".into(), "10S".into(), "40".into(), "80".into(), "160".into()]));
    pipe.params.push(ParamDef::new("length", 2).with_default(6000.0).with_range(100.0, 12000.0).with_step(100.0).with_unit("mm"));

    pipe.connectors.push(ConnectorDef::new("end1", "End 1")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(0));
    pipe.connectors.push(ConnectorDef::new("end2", "End 2")
        .with_position("0", "0", "length").with_direction("0", "0", "1").with_role(0));

    pipe.published = true;
    catalog.insert(pipe);

    // Elbow
    let mut elbow = ComponentDef::new("Elbow-LR-CS", "Piping/Fitting/Elbow");
    elbow.description = "Long-radius elbow, 1.5D, ASME B16.9".into();
    elbow.ifc_class = "IfcPipeFitting".into();
    elbow.standard_reference = "ASME B16.9".into();
    elbow.domain = "piping".into();
    elbow.tags = vec!["elbow".into(), "long-radius".into()];

    elbow.params.push(ParamDef::new("DN", 3).with_default(100.0).with_range(15.0, 600.0).with_step(1.0).with_unit("mm"));
    elbow.params.push(ParamDef::new("schedule", 4).with_default(0.0).with_choices(vec!["40".into(), "80".into()]));
    elbow.params.push(ParamDef::new("angle", 2).with_default(90.0).with_range(1.0, 180.0).with_step(1.0).with_unit("deg"));

    elbow.connectors.push(ConnectorDef::new("end1", "End 1")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(0));
    elbow.connectors.push(ConnectorDef::new("end2", "End 2")
        .with_position("1.5*DN", "0", "0").with_direction("1", "0", "0").with_role(0));

    elbow.published = true;
    catalog.insert(elbow);

    // Equal Tee
    let mut tee = ComponentDef::new("EqualTee-CS", "Piping/Fitting/Tee");
    tee.description = "Equal tee, ASME B16.9".into();
    tee.ifc_class = "IfcPipeFitting".into();
    tee.standard_reference = "ASME B16.9".into();
    tee.domain = "piping".into();
    tee.tags = vec!["tee".into(), "equal".into()];

    tee.params.push(ParamDef::new("DN", 3).with_default(100.0).with_range(15.0, 600.0).with_step(1.0).with_unit("mm"));
    tee.params.push(ParamDef::new("schedule", 4).with_default(0.0).with_choices(vec!["40".into(), "80".into()]));

    tee.connectors.push(ConnectorDef::new("run1", "Run End 1")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(0));
    tee.connectors.push(ConnectorDef::new("run2", "Run End 2")
        .with_position("200", "0", "0").with_direction("1", "0", "0").with_role(0));
    tee.connectors.push(ConnectorDef::new("branch", "Branch End")
        .with_position("100", "100", "0").with_direction("0", "1", "0").with_role(0));

    tee.published = true;
    catalog.insert(tee);

    // Concentric Reducer
    let mut reducer = ComponentDef::new("Reducer-Conc-CS", "Piping/Fitting/Reducer");
    reducer.description = "Concentric reducer, ASME B16.9".into();
    reducer.ifc_class = "IfcPipeFitting".into();
    reducer.standard_reference = "ASME B16.9".into();
    reducer.domain = "piping".into();
    reducer.tags = vec!["reducer".into(), "concentric".into()];

    reducer.params.push(ParamDef::new("DN_large", 3).with_default(150.0).with_range(15.0, 600.0).with_step(1.0).with_unit("mm"));
    reducer.params.push(ParamDef::new("DN_small", 3).with_default(100.0).with_range(15.0, 600.0).with_step(1.0).with_unit("mm"));
    reducer.params.push(ParamDef::new("length", 2).with_default(200.0).with_range(50.0, 600.0).with_step(10.0).with_unit("mm"));

    reducer.connectors.push(ConnectorDef::new("large_end", "Large End")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(0));
    reducer.connectors.push(ConnectorDef::new("small_end", "Small End")
        .with_position("0", "0", "length").with_direction("0", "0", "1").with_role(0));

    reducer.published = true;
    catalog.insert(reducer);

    // Weld Neck Flange
    let mut flange = ComponentDef::new("Flange-WN-CS", "Piping/Flange");
    flange.description = "Weld neck flange, ASME B16.5 Class 150".into();
    flange.ifc_class = "IfcFlowTerminal".into();
    flange.standard_reference = "ASME B16.5".into();
    flange.domain = "piping".into();
    flange.tags = vec!["flange".into(), "weld-neck".into()];

    flange.params.push(ParamDef::new("DN", 3).with_default(100.0).with_range(15.0, 600.0).with_step(1.0).with_unit("mm"));
    flange.params.push(ParamDef::new("pressure_class", 4).with_default(0.0).with_choices(vec!["150".into(), "300".into(), "600".into(), "900".into()]));

    flange.connectors.push(ConnectorDef::new("pipe_end", "Pipe End")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(0));
    flange.connectors.push(ConnectorDef::new("face", "Flange Face")
        .with_position("0", "0", "50").with_direction("0", "0", "1").with_role(0));

    flange.published = true;
    catalog.insert(flange);

    // Gate Valve
    let mut valve = ComponentDef::new("GateValve-CS", "Piping/Valve");
    valve.description = "Gate valve, rising stem, ASME B16.34".into();
    valve.ifc_class = "IfcValve".into();
    valve.standard_reference = "ASME B16.34".into();
    valve.domain = "piping".into();
    valve.tags = vec!["valve".into(), "gate".into()];

    valve.params.push(ParamDef::new("DN", 3).with_default(100.0).with_range(15.0, 600.0).with_step(1.0).with_unit("mm"));
    valve.params.push(ParamDef::new("pressure_class", 4).with_default(0.0).with_choices(vec!["150".into(), "300".into(), "600".into()]));

    valve.connectors.push(ConnectorDef::new("run_in", "Run In")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(0));
    valve.connectors.push(ConnectorDef::new("run_out", "Run Out")
        .with_position("300", "0", "0").with_direction("1", "0", "0").with_role(0));
    valve.connectors.push(ConnectorDef::new("stem_top", "Stem Top")
        .with_position("150", "200", "0").with_direction("0", "1", "0").with_role(2));

    valve.published = true;
    catalog.insert(valve);
}

// ============================================================
// Concrete Components
// ============================================================

fn seed_concrete_components(catalog: &mut crate::catalog::CatalogStore) {
    let mut beam = ComponentDef::new("Concrete-Beam-Rect", "Building/Concrete/Beam");
    beam.description = "Rectangular concrete beam".into();
    beam.ifc_class = "IfcBeam".into();
    beam.domain = "building".into();
    beam.tags = vec!["concrete".into(), "beam".into()];
    beam.default_material = "C30/37".into();
    beam.allowed_materials = vec!["C20/25".into(), "C25/30".into(), "C30/37".into(), "C35/45".into()];

    beam.params.push(ParamDef::new("width", 2).with_default(300.0).with_range(150.0, 600.0).with_step(50.0).with_unit("mm"));
    beam.params.push(ParamDef::new("depth", 2).with_default(500.0).with_range(200.0, 1000.0).with_step(50.0).with_unit("mm"));
    beam.params.push(ParamDef::new("length", 2).with_default(6000.0).with_range(1000.0, 12000.0).with_step(100.0).with_unit("mm"));

    beam.connectors.push(ConnectorDef::new("start", "Start")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(1));
    beam.connectors.push(ConnectorDef::new("end", "End")
        .with_position("0", "0", "length").with_direction("0", "0", "1").with_role(2));

    beam.published = true;
    catalog.insert(beam);

    let mut column = ComponentDef::new("Concrete-Column-Rect", "Building/Concrete/Column");
    column.description = "Rectangular concrete column".into();
    column.ifc_class = "IfcColumn".into();
    column.domain = "building".into();
    column.tags = vec!["concrete".into(), "column".into()];
    column.default_material = "C30/37".into();

    column.params.push(ParamDef::new("width", 2).with_default(300.0).with_range(150.0, 600.0).with_step(50.0).with_unit("mm"));
    column.params.push(ParamDef::new("depth", 2).with_default(300.0).with_range(150.0, 600.0).with_step(50.0).with_unit("mm"));
    column.params.push(ParamDef::new("column_height", 2).with_default(4200.0).with_range(1000.0, 10000.0).with_step(100.0).with_unit("mm"));

    column.connectors.push(ConnectorDef::new("base", "Base")
        .with_position("0", "0", "0").with_direction("0", "0", "-1").with_role(1));
    column.connectors.push(ConnectorDef::new("top", "Top")
        .with_position("0", "0", "column_height").with_direction("0", "0", "1").with_role(2));

    column.published = true;
    catalog.insert(column);

    // Circular concrete column
    let mut col_circ = ComponentDef::new("Concrete-Column-Circular", "Building/Concrete/Column");
    col_circ.description = "Circular concrete column".into();
    col_circ.ifc_class = "IfcColumn".into();
    col_circ.domain = "building".into();
    col_circ.tags = vec!["concrete".into(), "column".into(), "circular".into()];
    col_circ.default_material = "C30/37".into();
    col_circ.allowed_materials = vec!["C20/25".into(), "C25/30".into(), "C30/37".into(), "C35/45".into()];

    col_circ.params.push(ParamDef::new("diameter", 2).with_default(400.0).with_range(200.0, 1200.0).with_step(50.0).with_unit("mm"));
    col_circ.params.push(ParamDef::new("column_height", 2).with_default(4200.0).with_range(1000.0, 10000.0).with_step(100.0).with_unit("mm"));

    col_circ.connectors.push(ConnectorDef::new("base", "Base")
        .with_position("0", "0", "0").with_direction("0", "0", "-1").with_role(1));
    col_circ.connectors.push(ConnectorDef::new("top", "Top")
        .with_position("0", "0", "column_height").with_direction("0", "0", "1").with_role(2));

    col_circ.published = true;
    catalog.insert(col_circ);

    // Isolated footing
    let mut footing = ComponentDef::new("Concrete-Footing-Isolated", "Building/Concrete/Foundation");
    footing.description = "Isolated rectangular spread footing".into();
    footing.ifc_class = "IfcFooting".into();
    footing.domain = "building".into();
    footing.tags = vec!["concrete".into(), "footing".into(), "isolated".into()];
    footing.default_material = "C25/30".into();
    footing.allowed_materials = vec!["C20/25".into(), "C25/30".into(), "C30/37".into()];

    footing.params.push(ParamDef::new("width", 2).with_default(1500.0).with_range(800.0, 4000.0).with_step(100.0).with_unit("mm"));
    footing.params.push(ParamDef::new("depth", 2).with_default(1500.0).with_range(800.0, 4000.0).with_step(100.0).with_unit("mm"));
    footing.params.push(ParamDef::new("thickness", 2).with_default(400.0).with_range(200.0, 1000.0).with_step(50.0).with_unit("mm"));

    footing.connectors.push(ConnectorDef::new("top", "Top")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(1));
    footing.connectors.push(ConnectorDef::new("bottom", "Bottom")
        .with_position("0", "0", "thickness").with_direction("0", "0", "-1").with_role(2));

    footing.published = true;
    catalog.insert(footing);

    // Strip footing
    let mut strip = ComponentDef::new("Concrete-Footing-Strip", "Building/Concrete/Foundation");
    strip.description = "Continuous strip footing under wall".into();
    strip.ifc_class = "IfcFooting".into();
    strip.domain = "building".into();
    strip.tags = vec!["concrete".into(), "footing".into(), "strip".into()];
    strip.default_material = "C25/30".into();
    strip.allowed_materials = vec!["C20/25".into(), "C25/30".into(), "C30/37".into()];

    strip.params.push(ParamDef::new("length", 2).with_default(5000.0).with_range(1000.0, 50000.0).with_step(100.0).with_unit("mm"));
    strip.params.push(ParamDef::new("width", 2).with_default(600.0).with_range(300.0, 2000.0).with_step(50.0).with_unit("mm"));
    strip.params.push(ParamDef::new("thickness", 2).with_default(300.0).with_range(150.0, 800.0).with_step(50.0).with_unit("mm"));

    strip.connectors.push(ConnectorDef::new("start", "Start")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(1));
    strip.connectors.push(ConnectorDef::new("end", "End")
        .with_position("length", "0", "0").with_direction("1", "0", "0").with_role(2));

    strip.published = true;
    catalog.insert(strip);

    // Raft foundation
    let mut raft = ComponentDef::new("Concrete-Footing-Raft", "Building/Concrete/Foundation");
    raft.description = "Raft (mat) foundation slab".into();
    raft.ifc_class = "IfcFooting".into();
    raft.domain = "building".into();
    raft.tags = vec!["concrete".into(), "foundation".into(), "raft".into()];
    raft.default_material = "C30/37".into();
    raft.allowed_materials = vec!["C25/30".into(), "C30/37".into(), "C35/45".into()];

    raft.params.push(ParamDef::new("length", 2).with_default(10000.0).with_range(3000.0, 50000.0).with_step(500.0).with_unit("mm"));
    raft.params.push(ParamDef::new("width", 2).with_default(8000.0).with_range(3000.0, 40000.0).with_step(500.0).with_unit("mm"));
    raft.params.push(ParamDef::new("thickness", 2).with_default(600.0).with_range(300.0, 1500.0).with_step(50.0).with_unit("mm"));

    raft.connectors.push(ConnectorDef::new("top", "Top")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(1));

    raft.published = true;
    catalog.insert(raft);
}

// ============================================================
// Hollow Sections: CHS / SHS
// ============================================================

fn seed_hollow_section_family(catalog: &mut crate::catalog::CatalogStore) {
    let mut chs = ComponentDef::new("Steel-CHS", "Structural/Steel/HollowSection");
    chs.description = "Circular Hollow Section per EN 10210".into();
    chs.ifc_class = "IfcBeam".into();
    chs.standard_reference = "EN 10210".into();
    chs.domain = "structural".into();
    chs.tags = vec!["steel".into(), "chs".into(), "hollow".into()];
    chs.default_material = "S355".into();
    chs.allowed_materials = vec!["S235".into(), "S275".into(), "S355".into()];

    chs.params.push(ParamDef::new("diameter", 2).with_default(114.3).with_range(21.3, 508.0).with_step(1.0).with_unit("mm"));
    chs.params.push(ParamDef::new("thickness", 2).with_default(5.0).with_range(2.0, 20.0).with_step(0.5).with_unit("mm"));
    chs.params.push(ParamDef::new("length", 2).with_default(6000.0).with_range(1000.0, 12000.0).with_step(100.0).with_unit("mm"));

    chs.connectors.push(ConnectorDef::new("start", "Start")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(1));
    chs.connectors.push(ConnectorDef::new("end", "End")
        .with_position("0", "0", "length").with_direction("0", "0", "1").with_role(2));

    chs.published = true;
    catalog.insert(chs);

    let mut shs = ComponentDef::new("Steel-SHS", "Structural/Steel/HollowSection");
    shs.description = "Square Hollow Section per EN 10210".into();
    shs.ifc_class = "IfcBeam".into();
    shs.standard_reference = "EN 10210".into();
    shs.domain = "structural".into();
    shs.tags = vec!["steel".into(), "shs".into(), "hollow".into()];
    shs.default_material = "S355".into();
    shs.allowed_materials = vec!["S235".into(), "S275".into(), "S355".into()];

    shs.params.push(ParamDef::new("width", 2).with_default(100.0).with_range(40.0, 400.0).with_step(5.0).with_unit("mm"));
    shs.params.push(ParamDef::new("thickness", 2).with_default(5.0).with_range(2.0, 16.0).with_step(0.5).with_unit("mm"));
    shs.params.push(ParamDef::new("length", 2).with_default(6000.0).with_range(1000.0, 12000.0).with_step(100.0).with_unit("mm"));

    shs.connectors.push(ConnectorDef::new("start", "Start")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(1));
    shs.connectors.push(ConnectorDef::new("end", "End")
        .with_position("0", "0", "length").with_direction("0", "0", "1").with_role(2));

    shs.published = true;
    catalog.insert(shs);
}

// ============================================================
// Angle Section
// ============================================================

fn seed_angle_section_family(catalog: &mut crate::catalog::CatalogStore) {
    let mut angle = ComponentDef::new("Steel-Angle", "Structural/Steel/Angle");
    angle.description = "Equal/unequal angle section per EN 10056".into();
    angle.ifc_class = "IfcBeam".into();
    angle.standard_reference = "EN 10056".into();
    angle.domain = "structural".into();
    angle.tags = vec!["steel".into(), "angle".into(), "L".into()];
    angle.default_material = "S355".into();
    angle.allowed_materials = vec!["S235".into(), "S275".into(), "S355".into()];

    angle.params.push(ParamDef::new("leg1", 2).with_default(50.0).with_range(20.0, 200.0).with_step(5.0).with_unit("mm"));
    angle.params.push(ParamDef::new("leg2", 2).with_default(50.0).with_range(20.0, 200.0).with_step(5.0).with_unit("mm"));
    angle.params.push(ParamDef::new("thickness", 2).with_default(5.0).with_range(3.0, 20.0).with_step(0.5).with_unit("mm"));
    angle.params.push(ParamDef::new("length", 2).with_default(6000.0).with_range(1000.0, 12000.0).with_step(100.0).with_unit("mm"));

    angle.connectors.push(ConnectorDef::new("start", "Start")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(1));
    angle.connectors.push(ConnectorDef::new("end", "End")
        .with_position("0", "0", "length").with_direction("0", "0", "1").with_role(2));

    angle.published = true;
    catalog.insert(angle);
}

// ============================================================
// Building: Wall, Slab, Door, Window
// ============================================================

fn seed_building_components(catalog: &mut crate::catalog::CatalogStore) {
    let mut wall = ComponentDef::new("Concrete-Wall", "Building/Concrete/Wall");
    wall.description = "Rectangular concrete wall".into();
    wall.ifc_class = "IfcWall".into();
    wall.domain = "building".into();
    wall.tags = vec!["concrete".into(), "wall".into()];
    wall.default_material = "C30/37".into();
    wall.allowed_materials = vec!["C20/25".into(), "C25/30".into(), "C30/37".into(), "C35/45".into()];

    wall.params.push(ParamDef::new("length", 2).with_default(5000.0).with_range(500.0, 30000.0).with_step(100.0).with_unit("mm"));
    wall.params.push(ParamDef::new("thickness", 2).with_default(200.0).with_range(100.0, 500.0).with_step(10.0).with_unit("mm"));
    wall.params.push(ParamDef::new("height", 2).with_default(3000.0).with_range(1000.0, 6000.0).with_step(100.0).with_unit("mm"));

    wall.connectors.push(ConnectorDef::new("start", "Start")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(1));
    wall.connectors.push(ConnectorDef::new("end", "End")
        .with_position("length", "0", "0").with_direction("1", "0", "0").with_role(2));

    wall.published = true;
    catalog.insert(wall);

    let mut slab = ComponentDef::new("Concrete-Slab", "Building/Concrete/Slab");
    slab.description = "Rectangular concrete slab".into();
    slab.ifc_class = "IfcSlab".into();
    slab.domain = "building".into();
    slab.tags = vec!["concrete".into(), "slab".into()];
    slab.default_material = "C30/37".into();
    slab.allowed_materials = vec!["C20/25".into(), "C25/30".into(), "C30/37".into(), "C35/45".into()];

    slab.params.push(ParamDef::new("length", 2).with_default(6000.0).with_range(500.0, 30000.0).with_step(100.0).with_unit("mm"));
    slab.params.push(ParamDef::new("width", 2).with_default(4000.0).with_range(500.0, 30000.0).with_step(100.0).with_unit("mm"));
    slab.params.push(ParamDef::new("thickness", 2).with_default(150.0).with_range(80.0, 500.0).with_step(10.0).with_unit("mm"));

    slab.published = true;
    catalog.insert(slab);

    let mut door = ComponentDef::new("Door-Single", "Building/Door");
    door.description = "Single-leaf door".into();
    door.ifc_class = "IfcDoor".into();
    door.domain = "building".into();
    door.tags = vec!["door".into(), "single".into()];
    door.default_material = "Timber".into();

    door.params.push(ParamDef::new("width", 2).with_default(900.0).with_range(600.0, 1200.0).with_step(50.0).with_unit("mm"));
    door.params.push(ParamDef::new("height", 2).with_default(2100.0).with_range(1800.0, 2400.0).with_step(50.0).with_unit("mm"));
    door.params.push(ParamDef::new("frame_depth", 2).with_default(100.0).with_range(50.0, 200.0).with_step(10.0).with_unit("mm"));

    door.published = true;
    catalog.insert(door);

    let mut window = ComponentDef::new("Window-Fixed", "Building/Window");
    window.description = "Fixed window".into();
    window.ifc_class = "IfcWindow".into();
    window.domain = "building".into();
    window.tags = vec!["window".into(), "fixed".into()];
    window.default_material = "Glass".into();

    window.params.push(ParamDef::new("width", 2).with_default(1200.0).with_range(400.0, 3000.0).with_step(50.0).with_unit("mm"));
    window.params.push(ParamDef::new("height", 2).with_default(1500.0).with_range(400.0, 3000.0).with_step(50.0).with_unit("mm"));
    window.params.push(ParamDef::new("frame_depth", 2).with_default(80.0).with_range(50.0, 200.0).with_step(10.0).with_unit("mm"));

    window.published = true;
    catalog.insert(window);
}

// ============================================================
// Road Components
// ============================================================

fn seed_road_components(catalog: &mut crate::catalog::CatalogStore) {
    let mut road = ComponentDef::new("Road-Asphalt", "Civil/Road");
    road.description = "Asphalt road segment per GB JTG B01".into();
    road.ifc_class = "IfcRoad".into();
    road.standard_reference = "GB JTG B01".into();
    road.domain = "civil".into();
    road.tags = vec!["road".into(), "asphalt".into()];
    road.default_material = "Asphalt".into();

    road.params.push(ParamDef::new("length", 2).with_default(10000.0).with_range(1000.0, 100000.0).with_step(100.0).with_unit("mm"));
    road.params.push(ParamDef::new("width", 2).with_default(3500.0).with_range(2500.0, 15000.0).with_step(100.0).with_unit("mm"));
    road.params.push(ParamDef::new("thickness", 2).with_default(150.0).with_range(50.0, 500.0).with_step(10.0).with_unit("mm"));

    road.published = true;
    catalog.insert(road);

    let mut road_conc = ComponentDef::new("Road-Concrete", "Civil/Road");
    road_conc.description = "Concrete road segment per GB JTG B01".into();
    road_conc.ifc_class = "IfcRoad".into();
    road_conc.standard_reference = "GB JTG B01".into();
    road_conc.domain = "civil".into();
    road_conc.tags = vec!["road".into(), "concrete".into()];
    road_conc.default_material = "C40/50".into();

    road_conc.params.push(ParamDef::new("length", 2).with_default(10000.0).with_range(1000.0, 100000.0).with_step(100.0).with_unit("mm"));
    road_conc.params.push(ParamDef::new("width", 2).with_default(3500.0).with_range(2500.0, 15000.0).with_step(100.0).with_unit("mm"));
    road_conc.params.push(ParamDef::new("thickness", 2).with_default(240.0).with_range(100.0, 500.0).with_step(10.0).with_unit("mm"));

    road_conc.published = true;
    catalog.insert(road_conc);
}

// ============================================================
// Equipment Stubs
// ============================================================

fn seed_equipment_stubs(catalog: &mut crate::catalog::CatalogStore) {
    let mut tank = ComponentDef::new("VerticalTank", "Plant/Equipment/Tank");
    tank.description = "Vertical cylindrical tank".into();
    tank.ifc_class = "IfcTank".into();
    tank.domain = "plant".into();
    tank.tags = vec!["tank".into(), "vertical".into()];
    tank.default_material = "A516-70".into();

    tank.params.push(ParamDef::new("diameter", 2).with_default(3000.0).with_range(500.0, 20000.0).with_step(100.0).with_unit("mm"));
    tank.params.push(ParamDef::new("height", 2).with_default(5000.0).with_range(1000.0, 30000.0).with_step(100.0).with_unit("mm"));
    tank.params.push(ParamDef::new("wall_thickness", 2).with_default(8.0).with_range(3.0, 50.0).with_step(0.5).with_unit("mm"));

    tank.published = true;
    catalog.insert(tank);

    let mut pump = ComponentDef::new("CentrifugalPump", "Plant/Equipment/Pump");
    pump.description = "Centrifugal pump stub".into();
    pump.ifc_class = "IfcPump".into();
    pump.domain = "plant".into();
    pump.tags = vec!["pump".into(), "centrifugal".into()];
    pump.default_material = "CastIron".into();

    pump.params.push(ParamDef::new("DN", 3).with_default(100.0).with_range(25.0, 600.0).with_step(1.0).with_unit("mm"));
    pump.params.push(ParamDef::new("height", 2).with_default(600.0).with_range(200.0, 2000.0).with_step(50.0).with_unit("mm"));
    pump.params.push(ParamDef::new("width", 2).with_default(400.0).with_range(200.0, 1500.0).with_step(50.0).with_unit("mm"));

    pump.connectors.push(ConnectorDef::new("suction", "Suction")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(0));
    pump.connectors.push(ConnectorDef::new("discharge", "Discharge")
        .with_position("0", "0", "height").with_direction("0", "0", "1").with_role(0));

    pump.published = true;
    catalog.insert(pump);
}

// ============================================================
// Rebar / Reinforcement
// ============================================================

fn seed_rebar_components(catalog: &mut crate::catalog::CatalogStore) {
    let mut cage = ComponentDef::new("Rebar-Cage", "Building/Concrete/Reinforcement");
    cage.description = "Cylindrical rebar cage for concrete columns, piles and beams".into();
    cage.ifc_class = "IfcReinforcingBar".into();
    cage.domain = "building".into();
    cage.tags = vec!["rebar".into(), "cage".into(), "reinforcement".into(), "concrete".into()];
    cage.default_material = "B500B".into();
    cage.allowed_materials = vec!["B400".into(), "B500B".into(), "HRB400".into()];

    cage.params.push(ParamDef::new("bar_ring_radius", 2).with_default(200.0).with_range(50.0, 1000.0).with_step(5.0).with_unit("mm"));
    cage.params.push(ParamDef::new("main_bars", 1).with_default(8.0).with_range(4.0, 24.0).with_step(1.0).with_unit("个"));
    cage.params.push(ParamDef::new("length", 2).with_default(3000.0).with_range(500.0, 12000.0).with_step(100.0).with_unit("mm"));
    cage.params.push(ParamDef::new("diameter", 2).with_default(16.0).with_range(6.0, 40.0).with_step(2.0).with_unit("mm"));
    cage.params.push(ParamDef::new("stirrup_spacing", 2).with_default(200.0).with_range(50.0, 500.0).with_step(10.0).with_unit("mm"));
    cage.params.push(ParamDef::new("stirrup_radius", 2).with_default(192.0).with_range(40.0, 990.0).with_step(5.0).with_unit("mm"));

    cage.connectors.push(ConnectorDef::new("base", "Base")
        .with_position("0", "0", "0").with_direction("0", "0", "-1").with_role(1));
    cage.connectors.push(ConnectorDef::new("top", "Top")
        .with_position("0", "0", "length").with_direction("0", "0", "1").with_role(2));

    cage.published = true;
    catalog.insert(cage);
}

// ============================================================
// Domain Extension Components (added during the domain-extension batches)
// MEP (building), civil, plant, mechanical
// ============================================================

fn seed_domain_extension_components(catalog: &mut crate::catalog::CatalogStore) {
    // ── MEP (building domain) ──
    let mut duct = ComponentDef::new("Duct-Rect", "Building/MEP/Duct");
    duct.description = "Rectangular HVAC duct segment".into();
    duct.ifc_class = "IfcDuctSegment".into();
    duct.domain = "building".into();
    duct.tags = vec!["mep".into(), "duct".into(), "hvac".into()];
    duct.default_material = "GalvanizedSteel".into();
    duct.allowed_materials = vec!["GalvanizedSteel".into(), "StainlessSteel".into(), "Aluminum".into()];

    duct.params.push(ParamDef::new("width", 2).with_default(500.0).with_range(100.0, 3000.0).with_step(50.0).with_unit("mm"));
    duct.params.push(ParamDef::new("height", 2).with_default(250.0).with_range(100.0, 1500.0).with_step(50.0).with_unit("mm"));
    duct.params.push(ParamDef::new("length", 2).with_default(3000.0).with_range(500.0, 12000.0).with_step(100.0).with_unit("mm"));

    duct.connectors.push(ConnectorDef::new("start", "Start")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(1));
    duct.connectors.push(ConnectorDef::new("end", "End")
        .with_position("0", "0", "length").with_direction("0", "0", "1").with_role(2));

    duct.published = true;
    catalog.insert(duct);

    let mut tray = ComponentDef::new("CableTray", "Building/MEP/CableTray");
    tray.description = "Metal cable tray / ladder segment".into();
    tray.ifc_class = "IfcCableTraySegment".into();
    tray.domain = "building".into();
    tray.tags = vec!["mep".into(), "cable".into(), "tray".into()];
    tray.default_material = "GalvanizedSteel".into();
    tray.allowed_materials = vec!["GalvanizedSteel".into(), "StainlessSteel".into()];

    tray.params.push(ParamDef::new("width", 2).with_default(300.0).with_range(50.0, 1200.0).with_step(50.0).with_unit("mm"));
    tray.params.push(ParamDef::new("height", 2).with_default(100.0).with_range(25.0, 300.0).with_step(25.0).with_unit("mm"));
    tray.params.push(ParamDef::new("thickness", 2).with_default(3.0).with_range(1.0, 6.0).with_step(0.5).with_unit("mm"));
    tray.params.push(ParamDef::new("length", 2).with_default(3000.0).with_range(500.0, 6000.0).with_step(100.0).with_unit("mm"));

    tray.connectors.push(ConnectorDef::new("start", "Start")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(1));
    tray.connectors.push(ConnectorDef::new("end", "End")
        .with_position("0", "0", "length").with_direction("0", "0", "1").with_role(2));

    tray.published = true;
    catalog.insert(tray);

    // ── Plant ──
    let mut hex = ComponentDef::new("HeatExchanger", "Plant/Equipment/HeatExchanger");
    hex.description = "Shell & tube heat exchanger stub".into();
    hex.ifc_class = "IfcHeatExchanger".into();
    hex.domain = "plant".into();
    hex.tags = vec!["heat".into(), "exchanger".into(), "shell-tube".into()];
    hex.default_material = "A516-70".into();
    hex.allowed_materials = vec!["A516-70".into(), "SA240-316L".into()];

    hex.params.push(ParamDef::new("shell_diameter", 2).with_default(500.0).with_range(100.0, 3000.0).with_step(50.0).with_unit("mm"));
    hex.params.push(ParamDef::new("shell_length", 2).with_default(6000.0).with_range(1000.0, 15000.0).with_step(100.0).with_unit("mm"));

    hex.connectors.push(ConnectorDef::new("shell_in", "Shell Inlet")
        .with_position("0", "0", "0").with_direction("0", "0", "1").with_role(0));
    hex.connectors.push(ConnectorDef::new("shell_out", "Shell Outlet")
        .with_position("0", "0", "shell_length").with_direction("0", "0", "1").with_role(0));

    hex.published = true;
    catalog.insert(hex);

    let mut compressor = ComponentDef::new("Compressor", "Plant/Equipment/Compressor");
    compressor.description = "Process compressor stub".into();
    compressor.ifc_class = "IfcCompressor".into();
    compressor.domain = "plant".into();
    compressor.tags = vec!["compressor".into(), "process".into()];
    compressor.default_material = "CastIron".into();

    compressor.params.push(ParamDef::new("length", 2).with_default(2000.0).with_range(500.0, 8000.0).with_step(100.0).with_unit("mm"));
    compressor.params.push(ParamDef::new("width", 2).with_default(1200.0).with_range(300.0, 4000.0).with_step(100.0).with_unit("mm"));
    compressor.params.push(ParamDef::new("height", 2).with_default(1200.0).with_range(300.0, 4000.0).with_step(100.0).with_unit("mm"));

    compressor.published = true;
    catalog.insert(compressor);

    // ── Civil ──
    let mut manhole = ComponentDef::new("Manhole", "Civil/Utility/Manhole");
    manhole.description = "Cylindrical manhole / access chamber".into();
    manhole.ifc_class = "IfcManhole".into();
    manhole.domain = "civil".into();
    manhole.tags = vec!["utility".into(), "manhole".into(), "drainage".into()];
    manhole.default_material = "C30/37".into();
    manhole.allowed_materials = vec!["C25/30".into(), "C30/37".into()];

    manhole.params.push(ParamDef::new("diameter", 2).with_default(1200.0).with_range(500.0, 3000.0).with_step(100.0).with_unit("mm"));
    manhole.params.push(ParamDef::new("depth", 2).with_default(2500.0).with_range(500.0, 8000.0).with_step(100.0).with_unit("mm"));

    manhole.published = true;
    catalog.insert(manhole);

    // ── Mechanical ──
    let mut gear = ComponentDef::new("Gear", "Mechanical/Part/Gear");
    gear.description = "Spur gear blank (stub)".into();
    gear.ifc_class = "IfcMechanicalFastener".into();
    gear.domain = "mechanical".into();
    gear.tags = vec!["gear".into(), "mechanical".into(), "spur".into()];
    gear.default_material = "S355".into();
    gear.allowed_materials = vec!["S355".into(), "EN8".into(), "20CrMnTi".into()];

    gear.params.push(ParamDef::new("module", 2).with_default(4.0).with_range(0.5, 50.0).with_step(0.5).with_unit("mm"));
    gear.params.push(ParamDef::new("teeth", 1).with_default(20.0).with_range(6.0, 200.0).with_step(1.0).with_unit("个"));
    gear.params.push(ParamDef::new("width", 2).with_default(40.0).with_range(10.0, 500.0).with_step(5.0).with_unit("mm"));

    gear.published = true;
    catalog.insert(gear);
}

// ============================================================
// Series: Steel Structural Selection
// ============================================================

fn seed_structural_series(series_store: &mut SeriesStore) {
    let mut series = Series::new("Steel-Q355");
    series.description = "Structural steel S355 grade selection".into();
    series.domain = "structural".into();
    series.standard_reference = "EN 1993-1-1".into();

    let mut root = Selector::new("elem_sel", "What structural element?");

    let beam_spec = SpecComponent::new("Steel-Beam-IPE")
        .material("S355");
    root.answers.push(Answer {
        id: "beam".into(),
        value: "beam".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(beam_spec),
    });

    let col_spec = SpecComponent::new("Steel-Column-HEA")
        .material("S355");
    root.answers.push(Answer {
        id: "col".into(),
        value: "column".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(col_spec),
    });

    series.root_selector = Some(root);
    series_store.insert(series);

    // Concrete-C30 series
    let mut concrete_series = Series::new("Concrete-C30");
    concrete_series.description = "Concrete C30/37 structural elements".into();
    concrete_series.domain = "building".into();
    concrete_series.standard_reference = "EN 1992-1-1".into();

    let mut concrete_root = Selector::new("concrete_elem", "What concrete element?");

    let conc_beam = SpecComponent::new("Concrete-Beam-Rect")
        .material("C30/37");
    concrete_root.answers.push(Answer {
        id: "cbeam".into(),
        value: "beam".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(conc_beam),
    });

    let conc_col = SpecComponent::new("Concrete-Column-Rect")
        .material("C30/37");
    concrete_root.answers.push(Answer {
        id: "ccol".into(),
        value: "column".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(conc_col),
    });

    let conc_wall = SpecComponent::new("Concrete-Wall")
        .material("C30/37");
    concrete_root.answers.push(Answer {
        id: "cwall".into(),
        value: "wall".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(conc_wall),
    });

    let conc_slab = SpecComponent::new("Concrete-Slab")
        .material("C30/37");
    concrete_root.answers.push(Answer {
        id: "cslab".into(),
        value: "slab".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(conc_slab),
    });

    let door_spec = SpecComponent::new("Door-Single")
        .material("C30/37");
    concrete_root.answers.push(Answer {
        id: "cdoor".into(),
        value: "door".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(door_spec),
    });

    let window_spec = SpecComponent::new("Window-Fixed")
        .material("C30/37");
    concrete_root.answers.push(Answer {
        id: "cwindow".into(),
        value: "window".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(window_spec),
    });

    concrete_series.root_selector = Some(concrete_root);
    series_store.insert(concrete_series);
}

// ============================================================
// Series: CS150 Carbon Steel Piping
// ============================================================

fn seed_cs150_series(series_store: &mut SeriesStore) {
    let mut series = Series::new("CS150");
    series.description = "Carbon Steel, ASME Class 150 piping spec".into();
    series.domain = "piping".into();
    series.standard_reference = "ASME B31.3".into();

    let mut root = Selector::new("gen_type", "What generic type?");

    let pipe_spec = SpecComponent::new("Pipe-CS")
        .material("A106-B");
    root.answers.push(Answer {
        id: "pipe".into(),
        value: "PIPE".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(pipe_spec),
    });

    let elbow_spec = SpecComponent::new("Elbow-LR-CS")
        .material("A234-WPB");
    root.answers.push(Answer {
        id: "elbo".into(),
        value: "ELBO".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(elbow_spec),
    });

    // Tee
    let tee_spec = SpecComponent::new("EqualTee-CS")
        .material("A234-WPB");
    root.answers.push(Answer {
        id: "tee".into(),
        value: "TEE".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(tee_spec),
    });

    // Reducer
    let reducer_spec = SpecComponent::new("Reducer-Conc-CS")
        .material("A234-WPB");
    root.answers.push(Answer {
        id: "red".into(),
        value: "REDUCER".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(reducer_spec),
    });

    // Flange
    let flange_spec = SpecComponent::new("Flange-WN-CS")
        .material("A105");
    root.answers.push(Answer {
        id: "flg".into(),
        value: "FLANGE".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(flange_spec),
    });

    // Valve
    let valve_spec = SpecComponent::new("GateValve-CS")
        .material("A216-WCB");
    root.answers.push(Answer {
        id: "valve".into(),
        value: "VALVE".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(valve_spec),
    });

    series.root_selector = Some(root);
    series_store.insert(series);

    // SS300 series
    let mut ss300 = Series::new("SS300");
    ss300.description = "Stainless Steel, ASME Class 300 piping spec".into();
    ss300.domain = "piping".into();
    ss300.standard_reference = "ASME B31.3".into();

    let mut ss_root = Selector::new("ss_gen_type", "What generic type?");

    let ss_pipe = SpecComponent::new("Pipe-CS")
        .material("A312-TP304");
    ss_root.answers.push(Answer {
        id: "ss_pipe".into(),
        value: "PIPE".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(ss_pipe),
    });

    let ss_elbow = SpecComponent::new("Elbow-LR-CS")
        .material("A403-WP304");
    ss_root.answers.push(Answer {
        id: "ss_elbo".into(),
        value: "ELBO".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(ss_elbow),
    });

    let ss_flange = SpecComponent::new("Flange-WN-CS")
        .material("A182-F304");
    ss_root.answers.push(Answer {
        id: "ss_flg".into(),
        value: "FLANGE".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(ss_flange),
    });

    ss300.root_selector = Some(ss_root);
    series_store.insert(ss300);
}

// ============================================================
// Series: CS300 Carbon Steel Piping (ASME Class 300)
// ============================================================

fn seed_cs300_series(series_store: &mut SeriesStore) {
    let mut series = Series::new("CS300");
    series.description = "Carbon Steel, ASME Class 300 piping spec".into();
    series.domain = "piping".into();
    series.standard_reference = "ASME B31.3".into();

    let mut root = Selector::new("cs300_gen_type", "What generic type?");

    let pipe_spec = SpecComponent::new("Pipe-CS")
        .material("A106-B");
    root.answers.push(Answer {
        id: "cs300_pipe".into(),
        value: "PIPE".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(pipe_spec),
    });

    let elbow_spec = SpecComponent::new("Elbow-LR-CS")
        .material("A234-WPB");
    root.answers.push(Answer {
        id: "cs300_elbo".into(),
        value: "ELBO".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(elbow_spec),
    });

    let tee_spec = SpecComponent::new("EqualTee-CS")
        .material("A234-WPB");
    root.answers.push(Answer {
        id: "cs300_tee".into(),
        value: "TEE".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(tee_spec),
    });

    let reducer_spec = SpecComponent::new("Reducer-Conc-CS")
        .material("A234-WPB");
    root.answers.push(Answer {
        id: "cs300_red".into(),
        value: "REDUCER".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(reducer_spec),
    });

    let flange_spec = SpecComponent::new("Flange-WN-CS")
        .material("A105");
    root.answers.push(Answer {
        id: "cs300_flg".into(),
        value: "FLANGE".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(flange_spec),
    });

    let valve_spec = SpecComponent::new("GateValve-CS")
        .material("A216-WCB");
    root.answers.push(Answer {
        id: "cs300_valve".into(),
        value: "VALVE".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(valve_spec),
    });

    series.root_selector = Some(root);
    series_store.insert(series);
}

// ============================================================
// Series: Highway-GB (Civil Road per GB JTG B01)
// ============================================================

fn seed_highway_gb_series(series_store: &mut SeriesStore) {
    let mut series = Series::new("Highway-GB");
    series.description = "Highway road selection per GB JTG B01".into();
    series.domain = "civil".into();
    series.standard_reference = "GB JTG B01".into();

    let mut root = Selector::new("pavement_type", "What pavement type?");

    let asphalt_spec = SpecComponent::new("Road-Asphalt")
        .material("Asphalt");
    root.answers.push(Answer {
        id: "asphalt".into(),
        value: "ASPHALT".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(asphalt_spec),
    });

    let concrete_spec = SpecComponent::new("Road-Concrete")
        .material("C40/50");
    root.answers.push(Answer {
        id: "concrete".into(),
        value: "CONCRETE".into(),
        is_numeric_range: false,
        range_min: 0.0,
        range_max: 0.0,
        outcome: AnswerOutcome::Component(concrete_spec),
    });

    series.root_selector = Some(root);
    series_store.insert(series);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Once;

    static BREP_TEST_INIT: Once = Once::new();

    fn brep_setup() {
        BREP_TEST_INIT.call_once(|| {
            *crate::project::LIBRARY.write().unwrap() = Some(crate::library::Library::seeded());
        });
    }

    #[test]
    fn test_seed_catalog_has_structural_beam() {
        let mut catalog = crate::catalog::CatalogStore::new();
        seed_catalog(&mut catalog);
        let ipe = catalog.find_by_name("Steel-Beam-IPE");
        assert!(ipe.is_some(), "Should have Steel-Beam-IPE");
        let ipe = ipe.unwrap();
        assert_eq!(ipe.category, "Structural/Steel/Beam");
        assert!(ipe.published);
        assert!(!ipe.params.is_empty());
        assert!(!ipe.connectors.is_empty());
    }

    #[test]
    fn test_seed_catalog_has_pipe_cs() {
        let mut catalog = crate::catalog::CatalogStore::new();
        seed_catalog(&mut catalog);
        let pipe = catalog.find_by_name("Pipe-CS");
        assert!(pipe.is_some());
        let pipe = pipe.unwrap();
        assert_eq!(pipe.domain, "piping");
        assert!(pipe.published);
        assert!(pipe.params.iter().any(|p| p.name == "DN"));
        assert!(pipe.params.iter().any(|p| p.name == "schedule"));
    }

    #[test]
    fn test_seed_catalog_has_concrete_beam() {
        let mut catalog = crate::catalog::CatalogStore::new();
        seed_catalog(&mut catalog);
        let b = catalog.find_by_name("Concrete-Beam-Rect");
        assert!(b.is_some());
        assert_eq!(b.unwrap().domain, "building");
    }

    #[test]
    fn test_seed_catalog_has_piping_fittings() {
        let mut catalog = crate::catalog::CatalogStore::new();
        seed_catalog(&mut catalog);
        assert!(catalog.find_by_name("EqualTee-CS").is_some());
        assert!(catalog.find_by_name("Reducer-Conc-CS").is_some());
        assert!(catalog.find_by_name("Flange-WN-CS").is_some());
        assert!(catalog.find_by_name("GateValve-CS").is_some());
    }

    #[test]
    fn test_seed_catalog_has_rebar_cage() {
        let mut catalog = crate::catalog::CatalogStore::new();
        seed_catalog(&mut catalog);
        let cage = catalog.find_by_name("Rebar-Cage");
        assert!(cage.is_some(), "Should have Rebar-Cage");
        let cage = cage.unwrap();
        assert_eq!(cage.domain, "building");
        assert!(cage.published);
        assert!(cage.params.iter().any(|p| p.name == "bar_ring_radius"));
        assert!(cage.params.iter().any(|p| p.name == "main_bars"));
        assert!(cage.params.iter().any(|p| p.name == "length"));
        assert!(cage.params.iter().any(|p| p.name == "diameter"));
        assert!(cage.params.iter().any(|p| p.name == "stirrup_spacing"));
        assert!(cage.params.iter().any(|p| p.name == "stirrup_radius"));
    }

    #[test]
    fn test_seed_catalog_domain_extensions() {
        let mut catalog = crate::catalog::CatalogStore::new();
        seed_catalog(&mut catalog);

        let duct = catalog.find_by_name("Duct-Rect").expect("Duct-Rect");
        assert_eq!(duct.domain, "building");
        assert!(duct.params.iter().any(|p| p.name == "width"));
        assert!(duct.params.iter().any(|p| p.name == "length"));

        let tray = catalog.find_by_name("CableTray").expect("CableTray");
        assert_eq!(tray.domain, "building");

        let hex = catalog.find_by_name("HeatExchanger").expect("HeatExchanger");
        assert_eq!(hex.domain, "plant");

        let compressor = catalog.find_by_name("Compressor").expect("Compressor");
        assert_eq!(compressor.domain, "plant");

        let manhole = catalog.find_by_name("Manhole").expect("Manhole");
        assert_eq!(manhole.domain, "civil");

        let gear = catalog.find_by_name("Gear").expect("Gear");
        assert_eq!(gear.domain, "mechanical");
    }

    #[test]
    fn test_domain_extension_default_templates() {
        use crate::gen_rule::default_catalog_component;
        assert_eq!(default_catalog_component(c::ObjectType::Duct), Some("Duct-Rect"));
        assert_eq!(default_catalog_component(c::ObjectType::CableTray), Some("CableTray"));
        assert_eq!(default_catalog_component(c::ObjectType::HeatExchanger), Some("HeatExchanger"));
        assert_eq!(default_catalog_component(c::ObjectType::Compressor), Some("Compressor"));
        assert_eq!(default_catalog_component(c::ObjectType::Manhole), Some("Manhole"));
        assert_eq!(default_catalog_component(c::ObjectType::Gear), Some("Gear"));
    }

    #[test]
    fn test_seed_feature_trees_registered() {
        let mut registry = FeatureTreeRegistry::new();
        seed_feature_trees(&mut registry);
        assert!(!registry.is_empty(), "should have registered feature trees");
    }

    #[test]
    fn test_seed_series_has_steel_q355() {
        let mut series_store = SeriesStore::new();
        seed_series(&mut series_store);
        let s = series_store.find_by_name("Steel-Q355");
        assert!(s.is_some());
        let s = s.unwrap();
        assert_eq!(s.domain, "structural");
        assert!(s.root_selector.is_some());
        let root = s.root_selector.as_ref().unwrap();
        assert_eq!(root.answers.len(), 2);
    }

    #[test]
    fn test_seed_series_has_cs150() {
        let mut series_store = SeriesStore::new();
        seed_series(&mut series_store);
        let cs150 = series_store.find_by_name("CS150");
        assert!(cs150.is_some());
        let cs150 = cs150.unwrap();
        assert_eq!(cs150.domain, "piping");
        assert_eq!(cs150.standard_reference, "ASME B31.3");
        let root = cs150.root_selector.as_ref().unwrap();
        assert!(root.answers.iter().any(|a| a.value == "PIPE"));
        assert!(root.answers.iter().any(|a| a.value == "ELBO"));
    }

    #[test]
    fn test_seed_series_has_concrete_c30() {
        let mut series_store = SeriesStore::new();
        seed_series(&mut series_store);
        let s = series_store.find_by_name("Concrete-C30");
        assert!(s.is_some());
        let root = s.unwrap().root_selector.as_ref().unwrap();
        // beam, column, wall, slab, door, window
        assert_eq!(root.answers.len(), 6);
        assert!(root.answers.iter().any(|a| a.value == "wall"));
        assert!(root.answers.iter().any(|a| a.value == "slab"));
        assert!(root.answers.iter().any(|a| a.value == "door"));
        assert!(root.answers.iter().any(|a| a.value == "window"));
    }

    #[test]
    fn test_seed_series_has_ss300() {
        let mut series_store = SeriesStore::new();
        seed_series(&mut series_store);
        let s = series_store.find_by_name("SS300");
        assert!(s.is_some());
    }

    #[test]
    fn test_seed_catalog_has_hollow_sections() {
        let mut catalog = crate::catalog::CatalogStore::new();
        seed_catalog(&mut catalog);
        assert!(catalog.find_by_name("Steel-CHS").is_some(), "Should have Steel-CHS");
        assert!(catalog.find_by_name("Steel-SHS").is_some(), "Should have Steel-SHS");
        let chs = catalog.find_by_name("Steel-CHS").unwrap();
        assert!(chs.params.iter().any(|p| p.name == "diameter"));
        assert!(chs.params.iter().any(|p| p.name == "thickness"));
    }

    #[test]
    fn test_seed_catalog_has_angle() {
        let mut catalog = crate::catalog::CatalogStore::new();
        seed_catalog(&mut catalog);
        let a = catalog.find_by_name("Steel-Angle");
        assert!(a.is_some(), "Should have Steel-Angle");
        let a = a.unwrap();
        assert!(a.params.iter().any(|p| p.name == "leg1"));
        assert!(a.params.iter().any(|p| p.name == "leg2"));
    }

    #[test]
    fn test_seed_catalog_has_wall_slab() {
        let mut catalog = crate::catalog::CatalogStore::new();
        seed_catalog(&mut catalog);
        let w = catalog.find_by_name("Concrete-Wall");
        assert!(w.is_some(), "Should have Concrete-Wall");
        assert_eq!(w.unwrap().ifc_class, "IfcWall");
        assert!(catalog.find_by_name("Concrete-Slab").is_some(), "Should have Concrete-Slab");
    }

    #[test]
    fn test_seed_catalog_has_door_window() {
        let mut catalog = crate::catalog::CatalogStore::new();
        seed_catalog(&mut catalog);
        assert!(catalog.find_by_name("Door-Single").is_some(), "Should have Door-Single");
        assert!(catalog.find_by_name("Window-Fixed").is_some(), "Should have Window-Fixed");
    }

    #[test]
    fn test_seed_catalog_has_road() {
        let mut catalog = crate::catalog::CatalogStore::new();
        seed_catalog(&mut catalog);
        assert!(catalog.find_by_name("Road-Asphalt").is_some());
        assert!(catalog.find_by_name("Road-Concrete").is_some());
    }

    #[test]
    fn test_seed_catalog_has_equipment_stubs() {
        let mut catalog = crate::catalog::CatalogStore::new();
        seed_catalog(&mut catalog);
        assert!(catalog.find_by_name("VerticalTank").is_some());
        assert!(catalog.find_by_name("CentrifugalPump").is_some());
    }

    #[test]
    fn test_seed_series_has_highway_gb() {
        let mut series_store = SeriesStore::new();
        seed_series(&mut series_store);
        let s = series_store.find_by_name("Highway-GB");
        assert!(s.is_some(), "Should have Highway-GB series");
        let s = s.unwrap();
        assert_eq!(s.domain, "civil");
        assert_eq!(s.standard_reference, "GB JTG B01");
        let root = s.root_selector.as_ref().unwrap();
        assert!(root.answers.iter().any(|a| a.value == "ASPHALT"));
        assert!(root.answers.iter().any(|a| a.value == "CONCRETE"));
    }

    #[test]
    fn test_all_catalog_components_have_feature_trees() {
        let mut catalog = crate::catalog::CatalogStore::new();
        let mut registry = FeatureTreeRegistry::new();
        let links = seed_feature_trees(&mut registry);
        seed_catalog(&mut catalog);
        link_templates(&mut catalog, &links);

        for comp in catalog.list() {
            if !comp.feature_tree_id.is_empty() {
                assert!(
                    registry.get(&comp.feature_tree_id).is_some(),
                    "component '{}' references unknown feature tree '{}'",
                    comp.name, comp.feature_tree_id
                );
            }
        }
    }

    /// Rebuild every seeded catalog component that has a feature tree (BRep regression guard)
    /// tree template and assert its generated BRep mesh is valid (non-empty
    /// vertices, normals, and faces).
    #[test]
    fn test_all_components_brep_valid() {
        brep_setup();
        let mut proj = crate::project::Project::new("BRepValidation");

        let mut catalog = crate::catalog::CatalogStore::new();
        seed_catalog(&mut catalog);
        let mut registry = FeatureTreeRegistry::new();
        let links = seed_feature_trees(&mut registry);
        link_templates(&mut catalog, &links);

        let mut validated = 0usize;
        let mut skipped = Vec::new();
        let mut failures = Vec::new();

        for comp in catalog.list() {
            if comp.feature_tree_id.is_empty() {
                skipped.push(comp.name.clone());
                continue;
            }

            // Use component default parameter values.
            let mut params: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
            for p in &comp.params {
                params.insert(p.name.clone(), p.default_value);
            }

            let instance_name = format!("{}_brep", comp.name);
            let cid = match proj.instantiate_catalog_component_skip_gate(
                &comp.name,
                &instance_name,
                &params,
                None,
                0.0,
            ) {
                Ok(id) => id,
                Err(e) => {
                    failures.push(format!("{}: instantiate failed: {}", comp.name, e));
                    continue;
                }
            };

            if let Err(e) = proj.rebuild_component(&cid) {
                failures.push(format!("{}: rebuild failed: {}", comp.name, e));
                continue;
            }

            let obj = proj.objects.get(&cid).unwrap();
            let mut valid = false;
            let mut vertex_count = 0usize;
            let mut face_count = 0usize;
            for f in &obj.features {
                if let Some(ref geom) = f.cached_geometry {
                    vertex_count += geom.vertices.len();
                    face_count += geom.faces.len();
                    if !geom.vertices.is_empty() && !geom.faces.is_empty() {
                        valid = true;
                    }
                }
            }

            if !valid {
                failures.push(format!("{}: no valid geometry (v={}, f={})", comp.name, vertex_count, face_count));
            } else {
                validated += 1;
            }
        }

        assert!(
            failures.is_empty(),
            "Seeded catalog BRep validation failures ({}):\n{}",
            failures.len(),
            failures.join("\n")
        );
        assert!(
            validated >= 10,
            "expected at least 10 components validated, got {}",
            validated
        );
    }

    /// Verify every series decision path resolves to a component (series regression guard)
    /// present in the catalog (all series answers are reachable).
    #[test]
    fn test_all_series_paths_resolvable() {
        brep_setup();
        let mut catalog = crate::catalog::CatalogStore::new();
        seed_catalog(&mut catalog);

        let mut series_store = SeriesStore::new();
        seed_series(&mut series_store);

        let mut missing = Vec::new();
        let mut checked = 0usize;
        for series in series_store.list() {
            let mut stack: Vec<&Selector> = Vec::new();
            if let Some(root) = &series.root_selector {
                stack.push(root);
            }
            while let Some(sel) = stack.pop() {
                for a in &sel.answers {
                    match &a.outcome {
                        AnswerOutcome::Component(spec) => {
                            checked += 1;
                            if catalog.find_by_name(&spec.component_def_id).is_none() {
                                missing.push(format!(
                                    "{} -> {} references missing component '{}'",
                                    series.name, spec.component_def_id, spec.component_def_id
                                ));
                            }
                        }
                        AnswerOutcome::NextSelector(next) => {
                            stack.push(next);
                        }
                    }
                }
            }
        }

        assert!(
            missing.is_empty(),
            "Seeded series path failures ({}):\n{}",
            missing.len(),
            missing.join("\n")
        );
        assert!(checked > 0, "expected at least one series answer to be checked");
    }
}