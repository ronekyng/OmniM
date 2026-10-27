//! Default material library seeding.
//!
//! Populates the project's MaterialStore with a curated set of common
//! construction and engineering materials so the frontend material browser
//! (MaterialBrowser) has real data to display, and LayerEditor can assign
//! materials to layers by ID.
//!
//! IDs are stable, human-readable keys (e.g. "C30/37", "S355", "XPS-100")
//! so they can be referenced from catalog `allowed_materials` /
//! `default_material` and from object material assignments.

use omnim_core as c;
use crate::material::MaterialStore;

fn mat(
    id: &str,
    name: &str,
    category: c::MaterialCategory,
    density: f64,
    youngs: f64,
    poisson: f64,
    yield_s: f64,
    ultimate: f64,
    k: f64,
    color: (f32, f32, f32),
    ifc_material: &str,
) -> c::Material {
    c::Material {
        id: id.into(),
        name: name.into(),
        category: category as i32,
        density,
        youngs_modulus: youngs,
        poissons_ratio: poisson,
        yield_strength: yield_s,
        ultimate_strength: ultimate,
        thermal_conductivity: k,
        specific_heat: 840.0,
        color: Some(c::Color {
            r: color.0,
            g: color.1,
            b: color.2,
            a: 1.0,
        }),
        roughness: 0.7,
        metallic: 0.0,
        transparency: 0.0,
        ifc_material: ifc_material.into(),
        standards: Vec::new(),
    }
}

/// Populate the material store with built-in default materials.
pub fn seed_default_materials(store: &mut MaterialStore) {
    // ── Concrete (SI: N/m² = Pa) ──
    let concretes = [
        ("C20/25", "C20/25 混凝土", 2200.0, 2.3e10, 0.2, 0.0, 2.5e6, 1.4, (0.72, 0.70, 0.66)),
        ("C25/30", "C25/30 混凝土", 2300.0, 2.6e10, 0.2, 0.0, 2.8e6, 1.5, (0.70, 0.68, 0.64)),
        ("C30/37", "C30/37 混凝土", 2400.0, 3.0e10, 0.2, 0.0, 3.0e6, 1.6, (0.68, 0.66, 0.62)),
        ("C35/45", "C35/45 混凝土", 2400.0, 3.3e10, 0.2, 0.0, 3.5e6, 1.7, (0.65, 0.63, 0.59)),
        ("C40/50", "C40/50 混凝土", 2450.0, 3.5e10, 0.2, 0.0, 4.0e6, 1.8, (0.62, 0.60, 0.56)),
    ];
    for (id, name, d, e, nu, _y, u, k, col) in concretes {
        store.insert(mat(
            id, name, c::MaterialCategory::Concrete, d, e, nu, 0.0, u, k, col, "IFCCONCRETE",
        ));
    }

    // ── Steel ──
    let steels = [
        ("S235", "碳钢 S235", 7850.0, 2.1e11, 0.3, 2.35e8, 3.6e8, 50.0, (0.55, 0.58, 0.62)),
        ("S275", "碳钢 S275", 7850.0, 2.1e11, 0.3, 2.75e8, 4.1e8, 50.0, (0.52, 0.55, 0.59)),
        ("S355", "碳钢 S355", 7850.0, 2.1e11, 0.3, 3.55e8, 4.9e8, 50.0, (0.50, 0.53, 0.57)),
        ("S460", "高强钢 S460", 7850.0, 2.1e11, 0.3, 4.6e8, 5.5e8, 50.0, (0.48, 0.51, 0.55)),
        ("A106-B", "无缝碳钢管 A106 B", 7850.0, 2.0e11, 0.3, 2.4e8, 4.1e8, 54.0, (0.56, 0.55, 0.52)),
        ("A516-70", "压力容器钢板 A516-70", 7850.0, 2.0e11, 0.3, 2.6e8, 4.8e8, 50.0, (0.53, 0.52, 0.50)),
    ];
    for (id, name, d, e, nu, y, u, k, col) in steels {
        store.insert(mat(
            id, name, c::MaterialCategory::Metal, d, e, nu, y, u, k, col, "IFCMETAL",
        ));
    }

    // ── Timber ──
    store.insert(mat(
        "Timber",
        "木材",
        c::MaterialCategory::Wood,
        500.0,
        1.1e10,
        0.3,
        2.0e7,
        4.0e7,
        0.13,
        (0.68, 0.55, 0.35),
        "IFCWOOD",
    ));

    // ── Masonry / Mortar ──
    store.insert(mat(
        "Mortar",
        "水泥砂浆",
        c::MaterialCategory::Masonry,
        2000.0,
        2.0e10,
        0.2,
        0.0,
        1.0e6,
        1.0,
        (0.80, 0.79, 0.75),
        "IFCMASONRYUNIT",
    ));
    store.insert(mat(
        "Brick",
        "红砖",
        c::MaterialCategory::Masonry,
        1900.0,
        1.5e10,
        0.2,
        0.0,
        1.2e6,
        0.7,
        (0.75, 0.42, 0.30),
        "IFCMASONRYUNIT",
    ));
    store.insert(mat(
        "Block",
        "加气混凝土砌块",
        c::MaterialCategory::Masonry,
        650.0,
        2.0e9,
        0.2,
        0.0,
        0.5e6,
        0.16,
        (0.78, 0.78, 0.76),
        "IFCMASONRYUNIT",
    ));

    // ── Insulation ──
    store.insert(mat(
        "XPS-100",
        "挤塑聚苯板 XPS",
        c::MaterialCategory::Insulation,
        35.0,
        1.0e8,
        0.3,
        0.0,
        0.0,
        0.030,
        (0.92, 0.90, 0.62),
        "IFCINSULATION",
    ));
    store.insert(mat(
        "MW-100",
        "岩棉板 MW",
        c::MaterialCategory::Insulation,
        120.0,
        1.0e8,
        0.3,
        0.0,
        0.0,
        0.040,
        (0.82, 0.76, 0.60),
        "IFCINSULATION",
    ));
    store.insert(mat(
        "EPS",
        "模塑聚苯乙烯泡沫 EPS",
        c::MaterialCategory::Insulation,
        20.0,
        1.0e8,
        0.3,
        0.0,
        0.0,
        0.038,
        (0.93, 0.93, 0.92),
        "IFCINSULATION",
    ));

    // ── Glass ──
    store.insert(mat(
        "Glass",
        "玻璃",
        c::MaterialCategory::Glass,
        2500.0,
        7.0e10,
        0.22,
        0.0,
        4.5e7,
        1.0,
        (0.60, 0.75, 0.80),
        "IFCGLAZING",
    ));

    // ── Asphalt ──
    store.insert(mat(
        "Asphalt",
        "沥青混凝土",
        c::MaterialCategory::Soil,
        2400.0,
        1.2e10,
        0.35,
        0.0,
        0.0,
        1.2,
        (0.30, 0.30, 0.31),
        "IFCCONCRETE",
    ));

    // ── Water ──
    store.insert(mat(
        "Water",
        "水",
        c::MaterialCategory::Water,
        1000.0,
        0.0,
        0.5,
        0.0,
        0.0,
        0.6,
        (0.35, 0.55, 0.80),
        "IFCFLUID",
    ));
}

/// Number of default materials seeded by [`seed_default_materials`].
pub fn seed_default_materials_len() -> usize {
    21
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_materials() {
        let mut store = MaterialStore::new();
        seed_default_materials(&mut store);
        assert!(store.len() >= 20, "expected at least 20 seeded materials, got {}", store.len());
        assert!(store.get("C30/37").is_some());
        assert!(store.get("S355").is_some());
        assert!(store.get("XPS-100").is_some());
        let concretes = store.find_by_category(c::MaterialCategory::Concrete);
        assert!(!concretes.is_empty());
    }
}
