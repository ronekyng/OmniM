//! `(objectType, genesis class/form)` authorization matrix — the genrule
//! dispatch gate.
//!
//! Contract: [DATA_MODEL §1 — Governance rule 4](../../../../docs/specification/DATA_MODEL.md):
//! "the genrule dispatch checks the matrix; any combination outside the matrix
//! returns 'creation method not authorized for this object type' and a 400.
//! Extended authority is granted explicitly by adding a matrix row, then
//! implemented."
//!
//! Every row must have an implementation behind it; a planned combination is
//! added as a commented placeholder with its backlog reference, never as a
//! silent pass-through.

/// Genesis class of an instance (DATA_MODEL §1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenesisClass {
    Surface,
    Linear,
    Point,
    Body,
}

impl GenesisClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            GenesisClass::Surface => "Surface",
            GenesisClass::Linear => "Linear",
            GenesisClass::Point => "Point",
            GenesisClass::Body => "Body",
        }
    }
}

/// The per-instance creation method: class + enumerated form.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreationForm {
    pub class: GenesisClass,
    pub form: &'static str,
}

/// Object kinds that accept a path-shaped input, and the forms they may use.
///
/// Rows (kind, form) → class. Keep the table sorted by kind for review.
const AUTHORIZED: &[(&str, &str, GenesisClass)] = &[
    // Surface family — LayerExtrusion (module: layer_extrusion.rs)
    ("wall", "panel", GenesisClass::Surface), // centerline + layered thickness (default)
    ("wall", "region", GenesisClass::Surface), // closed boundary: face-based wall pieces
    ("slab", "region", GenesisClass::Surface), // closed footprint (default)
    ("floor", "region", GenesisClass::Surface), // closed footprint (default)
    ("roof", "region", GenesisClass::Surface), // footprint roof (default)
    ("roof", "panel", GenesisClass::Surface), // 2-point extrusion-profile roof
    // Linear family — LinearComponent (module: linear_component.rs)
    ("road", "linear", GenesisClass::Linear), // alignment + section, segmented mesh
    // Point family — transitional guide-line tube (linear_component::build_guide_line)
    ("line", "point", GenesisClass::Point), // featureLine / contourLine guide tube
                                            // Planned, NOT authorized yet (see ROADMAP "Accepted Designs"):
                                            // ("wall", "linear") → retaining wall: needs terrain target (DATA_MODEL §6.4)
                                            // ("wall", "body")   → sculpted curtain wall: needs boolean/freeform pipeline
                                            // ("beam", "linear") → catalog-driven beams land via catalog/series, not genrule
];

/// Check `(kind, form)` against the matrix.
///
/// `Err` carries the 400 reason text required by the contract.
pub fn authorize(kind: &str, form: &str) -> Result<CreationForm, String> {
    for (k, f, class) in AUTHORIZED {
        if *k == kind && *f == form {
            return Ok(CreationForm {
                class: *class,
                form: f,
            });
        }
    }
    Err(format!(
        "creation method '{}' is not authorized for object type '{}' \
         (see DATA_MODEL §1 authorization matrix; extend gen_rule::matrix to grant it)",
        form, kind
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_forms_are_authorized() {
        assert!(authorize("wall", "panel").is_ok());
        assert!(authorize("slab", "region").is_ok());
        assert!(authorize("floor", "region").is_ok());
        assert!(authorize("roof", "region").is_ok());
        assert!(authorize("roof", "panel").is_ok());
        assert!(authorize("road", "linear").is_ok());
        assert!(authorize("line", "point").is_ok());
    }

    #[test]
    fn wall_region_is_an_explicit_grant() {
        let f = authorize("wall", "region").unwrap();
        assert_eq!(f.class, GenesisClass::Surface);
        assert_eq!(f.form, "region");
    }

    #[test]
    fn cross_class_combinations_are_rejected() {
        // slab has no open-path panel form
        assert!(authorize("slab", "panel").is_err());
        // road is not a region
        assert!(authorize("road", "region").is_err());
        // walls do not sweep DN/elbows — the §6.4 boundary note
        assert!(authorize("wall", "linear").is_err());
        // unknown kinds/forms
        assert!(authorize("pipe", "panel").is_err());
        assert!(authorize("wall", "weird").is_err());
    }

    #[test]
    fn error_message_matches_contract() {
        let err = authorize("slab", "panel").unwrap_err();
        assert!(err.contains("creation method 'panel' is not authorized for object type 'slab'"));
    }
}
