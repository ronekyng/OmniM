use std::fmt;

use omnim_core as c;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::feature::Feature;

#[derive(Serialize, Deserialize, Clone)]
pub struct OmniObject {
    pub id: String,
    pub name: String,
    pub description: String,
    pub object_type: i32,
    pub status: i32,
    pub properties: Vec<c::Property>,
    pub property_sets: Vec<c::PropertySet>,
    pub geometry: Option<c::Geometry>,
    pub material_id: Option<String>,
    pub parent_id: Option<String>,
    pub tags: Vec<String>,
    pub ifc_class: String,
    #[serde(default)]
    pub features: Vec<Feature>,
    #[serde(default)]
    pub sketch_ids: Vec<String>,
    #[serde(default)]
    pub dirty: bool,
    #[serde(default)]
    pub catalog_ref: Option<crate::catalog::CatalogRef>,
}

impl OmniObject {
    pub fn new(object_type: c::ObjectType, name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            description: String::new(),
            object_type: object_type as i32,
            status: c::ObjectStatus::Draft as i32,
            properties: Vec::new(),
            property_sets: Vec::new(),
            geometry: None,
            material_id: None,
            parent_id: None,
            tags: Vec::new(),
            ifc_class: String::new(),
            features: Vec::new(),
            sketch_ids: Vec::new(),
            dirty: true,
            catalog_ref: None,
        }
    }

    pub fn is_component(&self) -> bool {
        self.object_type == c::ObjectType::Component as i32
    }

    pub fn set_property(&mut self, name: &str, value: c::PropertyValue) {
        if let Some(existing) = self.properties.iter_mut().find(|p| p.name == name) {
            existing.value = Some(value);
        } else {
            self.properties.push(c::Property {
                name: name.to_string(),
                value: Some(value),
                unit: String::new(),
                description: String::new(),
            });
        }
    }

    pub fn get_property(&self, name: &str) -> Option<&c::Property> {
        self.properties.iter().find(|p| p.name == name)
    }

    pub fn remove_property(&mut self, name: &str) -> Option<c::Property> {
        self.properties.iter().position(|p| p.name == name)
            .map(|i| self.properties.remove(i))
    }

    pub fn add_tag(&mut self, tag: impl Into<String>) {
        let t = tag.into();
        if !self.tags.contains(&t) {
            self.tags.push(t);
        }
    }

    pub fn add_tags(&mut self, tags: impl IntoIterator<Item = impl Into<String>>) {
        for t in tags {
            self.add_tag(t);
        }
    }

    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.add_tags(tags);
        self
    }

    pub fn with_ifc_class(mut self, ifc_class: impl Into<String>) -> Self {
        self.ifc_class = ifc_class.into();
        self
    }

    pub fn with_properties(mut self, props: impl IntoIterator<Item = (impl Into<String>, impl Into<c::PropertyValue>)>) -> Self {
        for (name, value) in props {
            self.set_property(&name.into(), value.into());
        }
        self
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.iter().any(|t| t == tag)
    }

    pub fn object_type_name(&self) -> &'static str {
        c::ObjectType::try_from(self.object_type)
            .map(type_name)
            .unwrap_or("UNKNOWN")
    }

    pub fn status_name(&self) -> &'static str {
        c::ObjectStatus::try_from(self.status)
            .map(status_name)
            .unwrap_or("UNKNOWN")
    }

    pub fn has_geometry(&self) -> bool {
        self.geometry.is_some()
    }

    pub fn bbox(&self) -> Option<&c::BoundingBox> {
        self.geometry.as_ref().and_then(|g| g.bbox.as_ref())
    }

    /// Mark this component as needing rebuild.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Check if this component needs rebuild.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Check if any feature in this component needs rebuild.
    pub fn has_dirty_features(&self) -> bool {
        self.features.iter().any(|f| f.dirty || f.cached_geometry.is_none())
    }

    /// Mark a specific feature and all downstream features as dirty.
    /// Features with higher order that depend on this feature's output
    /// are cascaded as dirty.
    pub fn mark_feature_dirty(&mut self, feature_id: &str) {
        if let Some(idx) = self.features.iter().position(|f| f.id == feature_id) {
            let order = self.features[idx].order;
            self.features[idx].mark_dirty();
            // Cascade to all features with higher order (they depend on earlier geometry)
            for f in self.features.iter_mut().skip(idx + 1) {
                if f.order > order {
                    f.mark_dirty();
                }
            }
            self.dirty = true;
        }
    }

    /// Mark all features as dirty.
    pub fn mark_all_features_dirty(&mut self) {
        for f in &mut self.features {
            f.mark_dirty();
        }
        self.dirty = true;
    }

    /// Clear dirty flags for all features and the component itself.
    pub fn clear_dirty(&mut self) {
        for f in &mut self.features {
            f.dirty = false;
        }
        self.dirty = false;
    }
}

fn type_name(t: c::ObjectType) -> &'static str {
    use c::ObjectType::*;
    match t {
        Unspecified => "Unspecified",
        Site => "Site",
        Building => "Building",
        BuildingStorey => "BuildingStorey",
        Space => "Space",
        Wall => "Wall",
        Beam => "Beam",
        Column => "Column",
        Slab => "Slab",
        Door => "Door",
        Window => "Window",
        Stair => "Stair",
        Roof => "Roof",
        Floor => "Floor",
        Duct => "Duct",
        CableTray => "CableTray",
        AirTerminal => "AirTerminal",
        Ahu => "AHU",
        Chiller => "Chiller",
        Transformer => "Transformer",
        Luminaire => "Luminaire",
        Road => "Road",
        Bridge => "Bridge",
        Tunnel => "Tunnel",
        Dam => "Dam",
        Culvert => "Culvert",
        RetainingWall => "RetainingWall",
        PipeCivil => "PipeCivil",
        Pier => "Pier",
        Manhole => "Manhole",
        Abutment => "Abutment",
        Tank => "Tank",
        Pump => "Pump",
        Valve => "Valve",
        HeatExchanger => "HeatExchanger",
        Reactor => "Reactor",
        PipePlant => "PipePlant",
        StructuralSteel => "StructuralSteel",
        Fitting => "Fitting",
        Compressor => "Compressor",
        Instrument => "Instrument",
        Part => "Part",
        Assembly => "Assembly",
        Fastener => "Fastener",
        Gear => "Gear",
        Shaft => "Shaft",
        Bearing => "Bearing",
        Motor => "Motor",
        Spring => "Spring",
        Seal => "Seal",
        Generic => "Generic",
        Component => "Component",
        FeatureLine => "FeatureLine",
        ContourLine => "ContourLine",
    }
}

fn status_name(s: c::ObjectStatus) -> &'static str {
    use c::ObjectStatus::*;
    match s {
        Unspecified => "Unspecified",
        Draft => "Draft",
        Proposed => "Proposed",
        Approved => "Approved",
        Construction => "Construction",
        AsBuilt => "AsBuilt",
        Operational => "Operational",
        Decommissioned => "Decommissioned",
    }
}

impl fmt::Display for OmniObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {} ({}) status={}", &self.id[..8], self.name, self.object_type_name(), self.status_name())?;
        if !self.properties.is_empty() {
            write!(f, " props={}", self.properties.len())?;
        }
        if self.geometry.is_some() {
            write!(f, " geom=yes")?;
        }
        if let Some(pid) = &self.parent_id {
            write!(f, " parent={}", &pid[..8])?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_object() {
        let obj = OmniObject::new(c::ObjectType::Wall, "Wall-1");
        assert_eq!(obj.name, "Wall-1");
        assert_eq!(obj.object_type, c::ObjectType::Wall as i32);
        assert!(!obj.id.is_empty());
        assert!(obj.properties.is_empty());
        assert!(obj.features.is_empty());
    }

    #[test]
    fn test_set_and_get_property() {
        let mut obj = OmniObject::new(c::ObjectType::Wall, "Wall-1");
        let val = c::PropertyValue {
            value_type: Some(c::property_value::ValueType::StringValue("2h".into())),
        };
        obj.set_property("FireRating", val.clone());
        let prop = obj.get_property("FireRating").unwrap();
        assert_eq!(prop.name, "FireRating");
        assert_eq!(prop.value, Some(val));
    }

    #[test]
    fn test_remove_property() {
        let mut obj = OmniObject::new(c::ObjectType::Wall, "Wall-1");
        let val = c::PropertyValue {
            value_type: Some(c::property_value::ValueType::StringValue("2h".into())),
        };
        obj.set_property("FireRating", val);
        let removed = obj.remove_property("FireRating").unwrap();
        assert_eq!(removed.name, "FireRating");
        assert!(obj.get_property("FireRating").is_none());
    }

    #[test]
    fn test_has_dirty_features() {
        let mut obj = OmniObject::new(c::ObjectType::Component, "Comp-1");
        assert!(!obj.has_dirty_features());
        let f = crate::Feature::new_extrude("ext1", "sketch-1", 1.0, c::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
        obj.features.push(f);
        assert!(obj.has_dirty_features());
    }

    #[test]
    fn test_object_type_name() {
        let obj = OmniObject::new(c::ObjectType::Wall, "W1");
        assert_eq!(obj.object_type_name(), "Wall");
        let obj2 = OmniObject::new(c::ObjectType::Beam, "B1");
        assert_eq!(obj2.object_type_name(), "Beam");
        let obj3 = OmniObject::new(c::ObjectType::PipePlant, "P1");
        assert_eq!(obj3.object_type_name(), "PipePlant");
    }

    #[test]
    fn test_display() {
        let obj = OmniObject::new(c::ObjectType::Wall, "TestWall");
        let s = format!("{}", obj);
        assert!(s.contains("TestWall"));
        assert!(s.contains("Wall"));
    }
}
