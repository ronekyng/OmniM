use std::collections::HashMap;
use omnim_core as c;
use omnim_model::Project;

// IFC entity representation
#[derive(Debug, Clone)]
struct IfcEntity {
    id: u64,
    entity_type: String,
    args: Vec<IfcValue>,
}

#[derive(Debug, Clone)]
enum IfcValue {
    Null,
    Integer(i64),
    Float(f64),
    String(String),
    Enum(String),
    Ref(u64),
    List(Vec<IfcValue>),
}

#[derive(Debug)]
pub struct IfcDocument {
    entities: HashMap<u64, IfcEntity>,
    project_name: String,
}

impl IfcDocument {
    pub fn parse(content: &str) -> Result<Self, String> {
        let mut entities = HashMap::new();
        let mut project_name = String::from("IFC Import");

        // Parse header
        let data_start = content.find("DATA;")
            .ok_or("IFC: no DATA section")?;
        let data_section = &content[data_start + 5..];

        // Parse each line
        let mut remaining = data_section;
        while let Some(line_end) = remaining.find(';') {
            let line = &remaining[..line_end].trim();
            remaining = &remaining[line_end + 1..];

            if line.starts_with("ENDSEC") || line.starts_with("END-ISO") {
                break;
            }

            if let Some(entity) = parse_entity_line(line) {
                if entity.entity_type == "IFCPROJECT" || entity.entity_type == "IFCBUILDING" {
                    if let Some(name) = extract_string_arg(&entity.args, 1) {
                        project_name = name;
                    }
                }
                entities.insert(entity.id, entity);
            }
        }

        Ok(IfcDocument { entities, project_name })
    }

    fn get(&self, id: u64) -> Option<&IfcEntity> {
        self.entities.get(&id)
    }

    fn get_string(&self, entity: &IfcEntity, index: usize) -> Option<String> {
        match entity.args.get(index)? {
            IfcValue::String(s) => Some(s.clone()),
            _ => None,
        }
    }

    fn get_ref(&self, entity: &IfcEntity, index: usize) -> Option<u64> {
        match entity.args.get(index)? {
            IfcValue::Ref(id) => Some(*id),
            _ => None,
        }
    }

    fn get_float(&self, entity: &IfcEntity, index: usize) -> Option<f64> {
        match entity.args.get(index)? {
            IfcValue::Float(f) => Some(*f),
            IfcValue::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }
}

fn parse_entity_line(line: &str) -> Option<IfcEntity> {
    let line = line.trim();
    if !line.starts_with('#') { return None; }

    // Extract #ID = TYPE(...)
    let eq_pos = line.find('=')?;
    let id_str = line[1..eq_pos].trim();
    let id: u64 = id_str.parse().ok()?;

    let rest = line[eq_pos + 1..].trim();
    let paren_pos = rest.find('(')?;
    let entity_type = rest[..paren_pos].trim().to_uppercase();
    let args_str = &rest[paren_pos + 1..];

    // Parse arguments
    let args = parse_args(args_str);
    Some(IfcEntity { id, entity_type, args })
}

fn parse_args(input: &str) -> Vec<IfcValue> {
    let mut values = Vec::new();
    let mut s = input;
    // Remove trailing )
    if s.ends_with(')') { s = &s[..s.len()-1]; }

    let tokens = tokenize(s);
    let mut pos = 0;
    while pos < tokens.len() {
        let (val, next) = parse_value(&tokens, pos);
        values.push(val);
        pos = next;
        // Skip comma
        while pos < tokens.len() && tokens[pos] == "," { pos += 1; }
    }
    values
}

fn tokenize(s: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            ' ' | '\t' | '\n' | '\r' => { i += 1; continue; }
            ',' | '(' | ')' => {
                tokens.push(chars[i].to_string());
                i += 1;
            }
            '\'' => {
                // String literal
                i += 1;
                let mut s = String::new();
                while i < chars.len() && chars[i] != '\'' {
                    if chars[i] == '\'' && i + 1 < chars.len() && chars[i+1] == '\'' {
                        s.push('\'');
                        i += 2;
                    } else {
                        s.push(chars[i]);
                        i += 1;
                    }
                }
                i += 1; // skip closing '
                tokens.push(format!("'{}'", s));
            }
            '.' => {
                // Check if previous token looks like an integer number (decimal point)
                if let Some(last) = tokens.last() {
                    if last.parse::<f64>().is_ok() || last == "$" || last == ")" || last == "," || last == "(" {
                        // Could be enum like .NOTDEFINED. or decimal .5
                        if i + 1 < chars.len() && chars[i+1].is_ascii_alphabetic() {
                            i += 1;
                            let mut s = String::new();
                            while i < chars.len() && chars[i] != '.' {
                                s.push(chars[i]);
                                i += 1;
                            }
                            if i < chars.len() { i += 1; }
                            tokens.push(format!(".{}", s));
                            continue;
                        }
                    }
                }
                // Treat as decimal point - append to previous number token
                tokens.push(".".to_string());
                i += 1;
            }
            '#' => {
                // Entity reference
                i += 1;
                let mut s = String::new();
                while i < chars.len() && chars[i].is_ascii_digit() {
                    s.push(chars[i]);
                    i += 1;
                }
                tokens.push(format!("#{}", s));
            }
            '$' => {
                tokens.push("$".to_string());
                i += 1;
            }
            _ => {
                // Number or keyword
                let mut s = String::new();
                while i < chars.len() && !" ,()#'.$\t\n\r".contains(chars[i]) {
                    s.push(chars[i]);
                    i += 1;
                }
                tokens.push(s);
            }
        }
    }
    tokens
}

fn parse_value(tokens: &[String], pos: usize) -> (IfcValue, usize) {
    if pos >= tokens.len() { return (IfcValue::Null, pos); }

    let token = &tokens[pos];

    if token == "$" {
        (IfcValue::Null, pos + 1)
    } else if token.starts_with('\'') && token.len() >= 2 {
        let s = &token[1..token.len()-1];
        (IfcValue::String(s.to_string()), pos + 1)
    } else if token.starts_with('.') && token.len() >= 3 {
        // Enum literal .WORD.
        let s = &token[1..token.len()-1];
        (IfcValue::Enum(s.to_string()), pos + 1)
    } else if token == "." {
        // Decimal point separator (handled in number parsing)
        (IfcValue::Integer(0), pos + 1)
    } else if token.starts_with('#') {
        let id: u64 = token[1..].parse().unwrap_or(0);
        (IfcValue::Ref(id), pos + 1)
    } else if token == "(" {
        // Parse list
        let mut list = Vec::new();
        let mut p = pos + 1;
        while p < tokens.len() && tokens[p] != ")" {
            let (val, next) = parse_value(tokens, p);
            list.push(val);
            p = next;
            if p < tokens.len() && tokens[p] == "," { p += 1; }
        }
        if p < tokens.len() { p += 1; } // skip )
        (IfcValue::List(list), p)
    } else {
        // Try integer first, then float, then keyword
        if let Ok(i) = token.parse::<i64>() {
            (IfcValue::Integer(i), pos + 1)
        } else if let Ok(f) = token.parse::<f64>() {
            (IfcValue::Float(f), pos + 1)
        } else {
            (IfcValue::String(token.clone()), pos + 1)
        }
    }
}

fn extract_string_arg(args: &[IfcValue], idx: usize) -> Option<String> {
    match args.get(idx)? {
        IfcValue::String(s) => Some(s.clone()),
        _ => None,
    }
}

// === IFC → OmniM Conversion ===

#[derive(Debug, Clone)]
pub struct ImportOptions {
    pub normalize_to_origin: bool,
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self { normalize_to_origin: true }
    }
}

pub fn import_ifc(content: &str) -> Result<Project, String> {
    import_ifc_with_options(content, ImportOptions::default())
}

pub fn import_ifc_with_options(content: &str, options: ImportOptions) -> Result<Project, String> {
    let doc = IfcDocument::parse(content)?;
    let mut proj = Project::new(&doc.project_name);
    let mut imported = 0u32;
    let mut entity_to_object: HashMap<u64, String> = HashMap::new();

    for entity in doc.entities.values() {
        let obj_type = match entity.entity_type.as_str() {
            "IFCWALL" | "IFCWALLSTANDARDCASE" => c::ObjectType::Wall,
            "IFCBEAM" | "IFCBEAMSTANDARDCASE" => c::ObjectType::Beam,
            "IFCCOLUMN" | "IFCCOLUMNSTANDARDCASE" => c::ObjectType::Column,
            "IFCSLAB" | "IFCSLABSTANDARDCASE" => c::ObjectType::Slab,
            "IFCDOOR" => c::ObjectType::Door,
            "IFCWINDOW" => c::ObjectType::Window,
            "IFCSTAIR" | "IFCSTAIRFLIGHT" => c::ObjectType::Stair,
            "IFCROOF" => c::ObjectType::Roof,
            "IFCBUILDING" => c::ObjectType::Building,
            "IFCBUILDINGSTOREY" => c::ObjectType::BuildingStorey,
            "IFCSITE" => c::ObjectType::Site,
            "IFCSPACE" => c::ObjectType::Space,
            "IFCROAD" => c::ObjectType::Road,
            "IFCBRIDGE" => c::ObjectType::Bridge,
            "IFCMEMBER" | "IFCMEMBERSTANDARDCASE" => c::ObjectType::Generic,
            "IFCPLATE" | "IFCPLATESTANDARDCASE" => c::ObjectType::Slab,
            "IFCFOOTING" => c::ObjectType::Generic,
            "IFCRAMP" => c::ObjectType::Generic,
            "IFCFURNISHINGELEMENT" | "IFCFURNITURE" => c::ObjectType::Generic,
            "IFCBUILDINGELEMENTPROXY" => c::ObjectType::Generic,
            _ => continue,
        };

        let name = doc.get_string(entity, 2)
            .or_else(|| doc.get_string(entity, 0))
            .unwrap_or_else(|| format!("{}_{}", entity.entity_type, entity.id));
        let obj_id = proj.create_object(obj_type, &name).id.clone();
        entity_to_object.insert(entity.id, obj_id.clone());

        {
            let obj = proj.objects.get_mut(&obj_id).unwrap();
            obj.ifc_class = entity.entity_type.clone();
            add_ifc_common_properties(obj, entity);
        }

        if let Some(geom) = extract_ifc_geometry(&doc, entity) {
            proj.objects.get_mut(&obj_id).unwrap().geometry = Some(geom);
        }

        if let Some(placement) = extract_placement(&doc, entity) {
            let obj = proj.objects.get_mut(&obj_id).unwrap();
            obj.set_property("IfcPlacementX", double_value(placement.x));
            obj.set_property("IfcPlacementY", double_value(placement.y));
            obj.set_property("IfcPlacementZ", double_value(placement.z));
            if let Some(ref mut geom) = obj.geometry {
                translate_geometry(geom, &placement);
            }
        }

        imported += 1;
    }

    apply_ifc_hierarchy(&doc, &mut proj, &entity_to_object);
    apply_ifc_property_sets(&doc, &mut proj, &entity_to_object);
    if options.normalize_to_origin {
        normalize_project_to_origin(&mut proj);
    }
    proj.objects.rebuild_children();

    eprintln!("IFC import: {} objects imported", imported);
    Ok(proj)
}

fn string_value(value: impl Into<String>) -> c::PropertyValue {
    c::PropertyValue {
        value_type: Some(c::property_value::ValueType::StringValue(value.into())),
    }
}

fn double_value(value: f64) -> c::PropertyValue {
    c::PropertyValue {
        value_type: Some(c::property_value::ValueType::DoubleValue(value)),
    }
}

fn add_ifc_common_properties(obj: &mut omnim_model::OmniObject, entity: &IfcEntity) {
    obj.set_property("IfcEntityId", string_value(format!("#{}", entity.id)));
    obj.set_property("IfcClass", string_value(entity.entity_type.clone()));
    if let Some(global_id) = get_ifc_global_id(entity) {
        obj.set_property("IfcGlobalId", string_value(global_id));
    }
    if let Some(name) = entity.args.get(2).and_then(ifc_value_to_string) {
        obj.set_property("IfcName", string_value(name));
    }
    if let Some(desc) = entity.args.get(3).and_then(ifc_value_to_string) {
        obj.description = desc.clone();
        obj.set_property("IfcDescription", string_value(desc));
    }
}

fn get_ifc_global_id(entity: &IfcEntity) -> Option<String> {
    entity.args.get(0).and_then(ifc_value_to_string)
}

fn ifc_value_to_string(value: &IfcValue) -> Option<String> {
    match value {
        IfcValue::String(s) => Some(s.clone()),
        IfcValue::Enum(s) => Some(s.clone()),
        IfcValue::Integer(i) => Some(i.to_string()),
        IfcValue::Float(f) => Some(f.to_string()),
        IfcValue::Ref(id) => Some(format!("#{}", id)),
        IfcValue::Null => None,
        IfcValue::List(values) => Some(values.iter().filter_map(ifc_value_to_string).collect::<Vec<_>>().join(",")),
    }
}

fn translate_geometry(geom: &mut c::Geometry, translation: &c::Point3D) {
    if let Some(mesh) = omnim_model::geometry::geometry_to_mesh(geom) {
        let t = c::Transform {
            matrix: vec![],
            translation: Some(translation.clone()),
            rotation: None,
            scale: None,
        };
        let transformed = omnim_model::geometry::transform_mesh(&mesh, &t);
        let bbox = omnim_model::geometry::compute_bbox(&transformed);
        *geom = c::Geometry {
            id: String::new(),
            r#type: c::GeometryType::Mesh as i32,
            representation: Some(c::geometry::Representation::Mesh(transformed)),
            bbox: Some(bbox),
            level_of_detail: 0,
        };
    }
}

fn normalize_project_to_origin(proj: &mut Project) {
    let mut min = c::Point3D { x: f64::INFINITY, y: f64::INFINITY, z: f64::INFINITY };
    let mut max = c::Point3D { x: f64::NEG_INFINITY, y: f64::NEG_INFINITY, z: f64::NEG_INFINITY };
    let mut has_bbox = false;

    for obj in proj.objects.list() {
        if let Some(bbox) = obj.bbox() {
            if let (Some(bmin), Some(bmax)) = (&bbox.min, &bbox.max) {
                min.x = min.x.min(bmin.x); min.y = min.y.min(bmin.y); min.z = min.z.min(bmin.z);
                max.x = max.x.max(bmax.x); max.y = max.y.max(bmax.y); max.z = max.z.max(bmax.z);
                has_bbox = true;
            }
        }
    }

    if !has_bbox { return; }
    let center = c::Point3D {
        x: (min.x + max.x) / 2.0,
        y: (min.y + max.y) / 2.0,
        z: (min.z + max.z) / 2.0,
    };
    let translation = c::Point3D { x: -center.x, y: -center.y, z: -center.z };

    let ids: Vec<String> = proj.objects.list().into_iter().map(|o| o.id.clone()).collect();
    for id in ids {
        if let Some(obj) = proj.objects.get_mut(&id) {
            if let Some(ref mut geom) = obj.geometry {
                translate_geometry(geom, &translation);
            }
            obj.set_property("IfcOriginOffsetX", double_value(center.x));
            obj.set_property("IfcOriginOffsetY", double_value(center.y));
            obj.set_property("IfcOriginOffsetZ", double_value(center.z));
        }
    }
}

fn apply_ifc_hierarchy(doc: &IfcDocument, proj: &mut Project, entity_to_object: &HashMap<u64, String>) {
    for rel in doc.entities.values().filter(|e| e.entity_type == "IFCRELAGGREGATES" || e.entity_type == "IFCRELCONTAINEDINSPATIALSTRUCTURE") {
        if rel.entity_type == "IFCRELAGGREGATES" {
            let parent_id = match doc.get_ref(rel, 4).and_then(|id| entity_to_object.get(&id)).cloned() {
                Some(id) => id,
                None => continue,
            };
            if let Some(IfcValue::List(children)) = rel.args.get(5) {
                for child in children {
                    if let IfcValue::Ref(child_entity_id) = child {
                        if let Some(child_id) = entity_to_object.get(child_entity_id).cloned() {
                            let _ = proj.set_parent(&child_id, &parent_id);
                        }
                    }
                }
            }
        } else {
            let parent_id = match doc.get_ref(rel, 5).and_then(|id| entity_to_object.get(&id)).cloned() {
                Some(id) => id,
                None => continue,
            };
            if let Some(IfcValue::List(children)) = rel.args.get(4) {
                for child in children {
                    if let IfcValue::Ref(child_entity_id) = child {
                        if let Some(child_id) = entity_to_object.get(child_entity_id).cloned() {
                            let _ = proj.set_parent(&child_id, &parent_id);
                        }
                    }
                }
            }
        }
    }
}

fn apply_ifc_property_sets(doc: &IfcDocument, proj: &mut Project, entity_to_object: &HashMap<u64, String>) {
    for rel in doc.entities.values().filter(|e| e.entity_type == "IFCRELDEFINESBYPROPERTIES") {
        let pset_ref = match doc.get_ref(rel, 5) {
            Some(id) => id,
            None => continue,
        };
        let pset = match doc.get(pset_ref) {
            Some(pset) if pset.entity_type == "IFCPROPERTYSET" => pset,
            _ => continue,
        };
        let pset_name = doc.get_string(pset, 2).unwrap_or_else(|| format!("Pset_{}", pset.id));
        let properties = extract_property_set(doc, pset);

        if let Some(IfcValue::List(related)) = rel.args.get(4) {
            for item in related {
                if let IfcValue::Ref(entity_id) = item {
                    if let Some(obj_id) = entity_to_object.get(entity_id) {
                        if let Some(obj) = proj.objects.get_mut(obj_id) {
                            obj.property_sets.push(c::PropertySet {
                                name: pset_name.clone(),
                                description: String::new(),
                                properties: properties.clone(),
                            });
                        }
                    }
                }
            }
        }
    }
}

fn extract_property_set(doc: &IfcDocument, pset: &IfcEntity) -> Vec<c::Property> {
    let mut props = Vec::new();
    if let Some(IfcValue::List(items)) = pset.args.get(4) {
        for item in items {
            let prop = match item {
                IfcValue::Ref(id) => doc.get(*id),
                _ => None,
            };
            if let Some(prop) = prop {
                if prop.entity_type == "IFCPROPERTYSINGLEVALUE" {
                    let name = doc.get_string(prop, 0).unwrap_or_else(|| format!("Property_{}", prop.id));
                    let value = prop.args.get(2).and_then(ifc_value_to_string).unwrap_or_default();
                    props.push(c::Property {
                        name,
                        value: Some(string_value(value)),
                        unit: String::new(),
                        description: String::new(),
                    });
                }
            }
        }
    }
    props
}

fn extract_placement(doc: &IfcDocument, entity: &IfcEntity) -> Option<c::Point3D> {
    // For IfcProduct: ObjectPlacement is at arg index 5
    let placement_ref = doc.get_ref(entity, 5)?;
    let placement = doc.get(placement_ref)?;

    // IfcLocalPlacement: args[0] = PlacementRelTo, args[1] = RelativePlacement
    let rel_placement_ref = doc.get_ref(placement, 1)?;
    let rel_placement = doc.get(rel_placement_ref)?;

    // IfcAxis2Placement3D: args[0] = Location (IfcCartesianPoint), args[1] = Axis, args[2] = RefDirection
    let location_ref = doc.get_ref(rel_placement, 0)?;
    let location = doc.get(location_ref)?;

    // IfcCartesianPoint: args[0] = list of coordinates
    if let Some(IfcValue::List(coords)) = location.args.get(0) {
        let x = coord_to_f64(coords.get(0))?;
        let y = coord_to_f64(coords.get(1))?;
        let z = coord_to_f64(coords.get(2))?;
        Some(c::Point3D { x, y, z })
    } else {
        None
    }
}

fn extract_ifc_geometry(doc: &IfcDocument, entity: &IfcEntity) -> Option<c::Geometry> {
    // IfcProduct: Representation at arg index 6
    if entity.args.len() < 7 { return None; }

    let rep_ref = match &entity.args[6] {
        IfcValue::Ref(id) => *id,
        _ => return None,
    };

    // IfcProductDefinitionShape: args[2] = Representations
    let shape = doc.get(rep_ref)?;
    let reps_list = match shape.args.get(2)? {
        IfcValue::List(items) => items,
        _ => return None,
    };

    for rep_item in reps_list {
        let rep_id = match rep_item {
            IfcValue::Ref(id) => *id,
            _ => continue,
        };
        let rep = doc.get(rep_id)?;
        // IfcShapeRepresentation: args[3] = Items
        let items_list = match rep.args.get(3)? {
            IfcValue::List(items) => items,
            _ => continue,
        };

        for item in items_list {
            let item_id = match item {
                IfcValue::Ref(id) => *id,
                _ => continue,
            };
            if let Some(geom) = convert_ifc_item(doc, item_id) {
                return Some(geom);
            }
        }
    }
    None
}

fn convert_ifc_item(doc: &IfcDocument, item_id: u64) -> Option<c::Geometry> {
    geometry::convert_ifc_item(doc, item_id)
}

pub(crate) fn coord_to_f64(val: Option<&IfcValue>) -> Option<f64> {
    match val? {
        IfcValue::Float(f) => Some(*f),
        IfcValue::Integer(i) => Some(*i as f64),
        _ => None,
    }
}

pub mod geometry;

pub fn export_ifc(project: &Project) -> String {
    let mut out = String::new();
    out.push_str("ISO-10303-21;\nHEADER;\n");
    out.push_str(&format!("FILE_DESCRIPTION(('OmniM export: {}'),'2;1');\n", escape_ifc_string(&project.name)));
    out.push_str("FILE_NAME('project.ifc','2024-01-01T00:00:00',('OmniM'),('OmniM'),'OmniM','OmniM','');\n");
    out.push_str("FILE_SCHEMA(('IFC4'));\nENDSEC;\nDATA;\n");

    let mut next_id = 1u64;
    let mut obj_to_ifc: HashMap<String, u64> = HashMap::new();

    for obj in project.objects.list() {
        let ifc_id = next_id;
        next_id += 1;
        obj_to_ifc.insert(obj.id.clone(), ifc_id);
        let class = export_ifc_class(obj);
        let global_id = obj.get_property("IfcGlobalId")
            .and_then(|p| p.value.as_ref())
            .and_then(property_value_as_string)
            .unwrap_or_else(|| compress_id(&obj.id));
        let name = escape_ifc_string(&obj.name);
        let description = if obj.description.is_empty() { "$".to_string() } else { format!("'{}'", escape_ifc_string(&obj.description)) };

        let obj_placement = if obj.geometry.is_some() {
            export_placement(&mut out, &mut next_id, obj)
        } else {
            None
        };
        let obj_shape = obj_placement.and_then(|_| {
            export_solid_geometry(&mut out, &mut next_id, obj)
        });

        let placement_str = match obj_placement { Some(id) => format!("#{}", id), None => "$".to_string() };
        let shape_str = match obj_shape { Some(id) => format!("#{}", id), None => "$".to_string() };

        out.push_str(&format!(
            "#{}={}('{}',$,{},'{}',$,{},{});
",
            ifc_id, class, global_id, description, name, placement_str, shape_str
        ));
    }

    for cont in project.relations.containments() {
        if let (Some(parent), Some(child)) = (obj_to_ifc.get(&cont.parent_id), obj_to_ifc.get(&cont.child_id)) {
            let rel_id = next_id;
            next_id += 1;
            out.push_str(&format!("#{}=IFCRELAGGREGATES('{}',$,$,$,#{},(#{}));\n", rel_id, compress_id(&format!("{}-{}", cont.parent_id, cont.child_id)), parent, child));
        }
    }

    for obj in project.objects.list() {
        if obj.property_sets.is_empty() { continue; }
        let Some(obj_ifc_id) = obj_to_ifc.get(&obj.id) else { continue; };
        for pset in &obj.property_sets {
            let mut prop_ids = Vec::new();
            for prop in &pset.properties {
                let prop_id = next_id;
                next_id += 1;
                let value = prop.value.as_ref().and_then(property_value_as_string).unwrap_or_default();
                out.push_str(&format!("#{}=IFCPROPERTYSINGLEVALUE('{}',$,IFCTEXT('{}'),$);\n", prop_id, escape_ifc_string(&prop.name), escape_ifc_string(&value)));
                prop_ids.push(format!("#{}", prop_id));
            }
            let pset_id = next_id;
            next_id += 1;
            out.push_str(&format!("#{}=IFCPROPERTYSET('{}',$,'{}',$,({}));\n", pset_id, compress_id(&format!("{}-{}", obj.id, pset.name)), escape_ifc_string(&pset.name), prop_ids.join(",")));
            let rel_id = next_id;
            next_id += 1;
            out.push_str(&format!("#{}=IFCRELDEFINESBYPROPERTIES('{}',$,$,$,(#{}),#{});\n", rel_id, compress_id(&format!("rel-{}-{}", obj.id, pset.name)), obj_ifc_id, pset_id));
        }
    }

    out.push_str("ENDSEC;\nEND-ISO-10303-21;\n");
    out
}

fn export_placement(out: &mut String, next_id: &mut u64, obj: &omnim_model::OmniObject) -> Option<u64> {
    let (px, py, pz) = (
        obj.get_property("IfcPlacementX").and_then(|p| p.value.as_ref()).and_then(property_value_as_f64).unwrap_or(0.0),
        obj.get_property("IfcPlacementY").and_then(|p| p.value.as_ref()).and_then(property_value_as_f64).unwrap_or(0.0),
        obj.get_property("IfcPlacementZ").and_then(|p| p.value.as_ref()).and_then(property_value_as_f64).unwrap_or(0.0),
    );

    let pt_id = *next_id; *next_id += 1;
    out.push_str(&format!("#{}=IFCCARTESIANPOINT(({:.6},{:.6},{:.6}));\n", pt_id, px, py, pz));

    let axis2_id = *next_id; *next_id += 1;
    out.push_str(&format!("#{}=IFCAXIS2PLACEMENT3D(#{},$,);\n", axis2_id, pt_id));

    let placement_id = *next_id; *next_id += 1;
    out.push_str(&format!("#{}=IFCLOCALPLACEMENT($,#{});\n", placement_id, axis2_id));
    Some(placement_id)
}

fn export_solid_geometry(out: &mut String, next_id: &mut u64, obj: &omnim_model::OmniObject) -> Option<u64> {
    let geom = obj.geometry.as_ref()?;
    match &geom.representation {
        Some(c::geometry::Representation::Parametric(pg)) => {
            let mut ids = Vec::new();
            for op in &pg.operations {
                if let Some(ref kind) = op.operation {
                    if let Some(id) = export_parametric_operation(out, next_id, kind) {
                        ids.push(id);
                    }
                }
            }
            if ids.is_empty() { None } else { Some(ids[0]) }
        }
        Some(c::geometry::Representation::Mesh(mesh)) => {
            export_mesh_brep(out, next_id, mesh)
        }
        _ => None,
    }
}

fn export_parametric_operation(out: &mut String, next_id: &mut u64, op: &c::parametric_operation::Operation) -> Option<u64> {
    use c::parametric_operation::Operation::*;
    match op {
        Extrusion(ext) => {
            let profile_id = export_profile(out, next_id, &ext.profile);
            let dir_id = *next_id; *next_id += 1;
            let (dx, dy, dz) = ext.direction.as_ref()
                .map(|d| (d.x, d.y, d.z))
                .unwrap_or((0.0, 0.0, 1.0));
            let len = (dx*dx + dy*dy + dz*dz).sqrt();
            let (ndx, ndy, ndz) = if len > 1e-10 { (dx/len, dy/len, dz/len) } else { (0.0, 0.0, 1.0) };
            out.push_str(&format!("#{}=IFCDIRECTION(({:.6},{:.6},{:.6}));\n", dir_id, ndx, ndy, ndz));

            let pos_id = *next_id; *next_id += 1;
            out.push_str(&format!("#{}=IFCAXIS2PLACEMENT3D(#{},$,);\n", pos_id, dir_id));

            let solid_id = *next_id; *next_id += 1;
            out.push_str(&format!("#{}=IFCEXTRUDEDAREASOLID(#{},#{},#{},{:.6});\n",
                solid_id, profile_id, pos_id, dir_id, ext.depth));
            Some(solid_id)
        }
        Revolve(rev) => {
            let profile_id = export_profile(out, next_id, &rev.profile);
            let origin = rev.axis_origin.clone().unwrap_or(c::Point3D { x: 0.0, y: 0.0, z: 0.0 });
            let dir = rev.axis_direction.clone().unwrap_or(c::Vector3D { x: 0.0, y: 0.0, z: 1.0 });
            let len = (dir.x*dir.x + dir.y*dir.y + dir.z*dir.z).sqrt();
            let (ndx, ndy, ndz) = if len > 1e-10 { (dir.x/len, dir.y/len, dir.z/len) } else { (0.0, 0.0, 1.0) };

            let axis_pt_id = *next_id; *next_id += 1;
            out.push_str(&format!("#{}=IFCCARTESIANPOINT(({:.6},{:.6},{:.6}));\n", axis_pt_id, origin.x, origin.y, origin.z));
            let axis_dir_id = *next_id; *next_id += 1;
            out.push_str(&format!("#{}=IFCDIRECTION(({:.6},{:.6},{:.6}));\n", axis_dir_id, ndx, ndy, ndz));
            let axis1_id = *next_id; *next_id += 1;
            out.push_str(&format!("#{}=IFCAXIS1PLACEMENT(#{},#{});\n", axis1_id, axis_pt_id, axis_dir_id));

            let solid_id = *next_id; *next_id += 1;
            let angle_deg = rev.angle * 180.0 / std::f64::consts::PI;
            out.push_str(&format!("#{}=IFCREVOLVEDAREASOLID(#{},#{},$,{:.6});\n",
                solid_id, profile_id, axis1_id, angle_deg));
            Some(solid_id)
        }
        _ => None,
    }
}

fn export_profile(out: &mut String, next_id: &mut u64, profile: &[c::Point2D]) -> u64 {
    let is_rect = profile.len() == 4
        && (profile[0].x - 0.0).abs() < 1e-6 && (profile[0].y - 0.0).abs() < 1e-6
        && (profile[1].x - profile[2].x).abs() < 1e-6
        && (profile[3].y - profile[2].y).abs() < 1e-6;
    let is_circle = false;

    if is_rect {
        let w = profile[1].x - profile[0].x;
        let h = profile[3].y - profile[0].y;
        let id = *next_id; *next_id += 1;
        out.push_str(&format!("#{}=IFCRECTANGLEPROFILEDEF(.AREA.,$,{:.6},{:.6});\n", id, w.abs(), h.abs()));
        id
    } else if is_circle {
        let r = ((profile[0].x - profile[1].x).powi(2) + (profile[0].y - profile[1].y).powi(2)).sqrt();
        let id = *next_id; *next_id += 1;
        out.push_str(&format!("#{}=IFCCIRCLEPROFILEDEF(.AREA.,$,{:.6});\n", id, r));
        id
    } else {
        let mut pt_ids = Vec::new();
        for p in profile {
            let pt_id = *next_id; *next_id += 1;
            out.push_str(&format!("#{}=IFCCARTESIANPOINT(({:.6},{:.6}));\n", pt_id, p.x, p.y));
            pt_ids.push(pt_id);
        }
        let polyline_id = *next_id; *next_id += 1;
        let pts_str = pt_ids.iter().map(|id| format!("#{}", id)).collect::<Vec<_>>().join(",");
        out.push_str(&format!("#{}=IFCPOLYLINE(({}));\n", polyline_id, pts_str));
        let id = *next_id; *next_id += 1;
        out.push_str(&format!("#{}=IFCARBITRARYCLOSEDPROFILEDEF(.AREA.,$,#{});\n", id, polyline_id));
        id
    }
}

fn export_mesh_brep(out: &mut String, next_id: &mut u64, mesh: &c::MeshGeometry) -> Option<u64> {
    if mesh.vertices.is_empty() || mesh.faces.is_empty() { return None; }

    let mut vert_ids = Vec::new();
    for v in &mesh.vertices {
        let id = *next_id; *next_id += 1;
        out.push_str(&format!("#{}=IFCCARTESIANPOINT(({:.6},{:.6},{:.6}));\n", id, v.x, v.y, v.z));
        vert_ids.push(id);
    }

    let mut face_ids = Vec::new();
    for face in &mesh.faces {
        let indices = &face.vertex_indices;
        let mut loop_pt_ids = Vec::new();
        for &idx in indices {
            if idx >= 0 && (idx as usize) < vert_ids.len() {
                loop_pt_ids.push(vert_ids[idx as usize]);
            }
        }
        if loop_pt_ids.len() < 3 { continue; }
        let poly_loop_id = *next_id; *next_id += 1;
        let pts_str = loop_pt_ids.iter().map(|id| format!("#{}", id)).collect::<Vec<_>>().join(",");
        out.push_str(&format!("#{}=IFCPOLYLOOP(({}));\n", poly_loop_id, pts_str));

        let face_bound_id = *next_id; *next_id += 1;
        out.push_str(&format!("#{}=IFCFACEBOUND(#{},.T);\n", face_bound_id, poly_loop_id));

        let face_id = *next_id; *next_id += 1;
        out.push_str(&format!("#{}=IFCFACE((#{}));\n", face_id, face_bound_id));
        face_ids.push(face_id);
    }

    if face_ids.is_empty() { return None; }
    let faces_str = face_ids.iter().map(|id| format!("#{}", id)).collect::<Vec<_>>().join(",");
    let closed_shell_id = *next_id; *next_id += 1;
    out.push_str(&format!("#{}=IFCCLOSEDSHELL(({}));\n", closed_shell_id, faces_str));

    let brep_id = *next_id; *next_id += 1;
    out.push_str(&format!("#{}=IFCFACETEDBREP(#{});\n", brep_id, closed_shell_id));
    Some(brep_id)
}

fn property_value_as_f64(value: &c::PropertyValue) -> Option<f64> {
    match value.value_type.as_ref()? {
        c::property_value::ValueType::DoubleValue(v) => Some(*v),
        c::property_value::ValueType::IntValue(v) => Some(*v as f64),
        _ => None,
    }
}

fn export_ifc_class(obj: &omnim_model::OmniObject) -> &'static str {
    if !obj.ifc_class.is_empty() {
        match obj.ifc_class.as_str() {
            "IFCSITE" => return "IFCSITE",
            "IFCBUILDING" => return "IFCBUILDING",
            "IFCBUILDINGSTOREY" => return "IFCBUILDINGSTOREY",
            "IFCSPACE" => return "IFCSPACE",
            "IFCWALL" | "IFCWALLSTANDARDCASE" => return "IFCWALL",
            "IFCBEAM" | "IFCBEAMSTANDARDCASE" => return "IFCBEAM",
            "IFCCOLUMN" | "IFCCOLUMNSTANDARDCASE" => return "IFCCOLUMN",
            "IFCSLAB" | "IFCSLABSTANDARDCASE" => return "IFCSLAB",
            "IFCDOOR" => return "IFCDOOR",
            "IFCWINDOW" => return "IFCWINDOW",
            _ => {}
        }
    }
    match c::ObjectType::try_from(obj.object_type).unwrap_or(c::ObjectType::Generic) {
        c::ObjectType::Site => "IFCSITE",
        c::ObjectType::Building => "IFCBUILDING",
        c::ObjectType::BuildingStorey => "IFCBUILDINGSTOREY",
        c::ObjectType::Space => "IFCSPACE",
        c::ObjectType::Wall => "IFCWALL",
        c::ObjectType::Beam => "IFCBEAM",
        c::ObjectType::Column => "IFCCOLUMN",
        c::ObjectType::Slab => "IFCSLAB",
        c::ObjectType::Door => "IFCDOOR",
        c::ObjectType::Window => "IFCWINDOW",
        _ => "IFCBUILDINGELEMENTPROXY",
    }
}

fn property_value_as_string(value: &c::PropertyValue) -> Option<String> {
    match value.value_type.as_ref()? {
        c::property_value::ValueType::StringValue(v) => Some(v.clone()),
        c::property_value::ValueType::IntValue(v) => Some(v.to_string()),
        c::property_value::ValueType::DoubleValue(v) => Some(v.to_string()),
        c::property_value::ValueType::BoolValue(v) => Some(v.to_string()),
        _ => None,
    }
}

fn escape_ifc_string(value: &str) -> String {
    value.replace('\'', "''")
}

fn compress_id(value: &str) -> String {
    let compact: String = value.chars().filter(|c| c.is_ascii_alphanumeric()).take(22).collect();
    if compact.is_empty() { "0".into() } else { compact }
}
