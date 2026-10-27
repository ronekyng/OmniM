use omnim_core as c;

const IFC_WITH_HIERARCHY_AND_PROPS: &str = r#"ISO-10303-21;
HEADER;
FILE_DESCRIPTION(('Test IFC'),'2;1');
FILE_NAME('test.ifc','2024-01-01T00:00:00',('Test'),('Test'),'OmniM','OmniM','');
FILE_SCHEMA(('IFC4'));
ENDSEC;
DATA;
#1=IFCSITE('site-guid',$,'Site A','Site description',$,$,$,$,.ELEMENT.,$,$,$,$,$);
#2=IFCBUILDING('building-guid',$,'Building A','Building description',$,$,$,$,.ELEMENT.,$,$,$);
#3=IFCBUILDINGSTOREY('storey-guid',$,'Level 1','Storey description',$,$,$,$,$,$);
#4=IFCWALL('wall-guid',$,'Wall A','Wall description',$,$,$,$);
#10=IFCRELAGGREGATES('rel-site-building',$,$,$,#1,(#2));
#11=IFCRELAGGREGATES('rel-building-storey',$,$,$,#2,(#3));
#12=IFCRELCONTAINEDINSPATIALSTRUCTURE('rel-storey-wall',$,$,$,(#4),#3);
#20=IFCPROPERTYSINGLEVALUE('FireRating',$,IFCTEXT('2h'),$);
#21=IFCPROPERTYSET('pset-guid',$,'Pset_WallCommon',$,(#20));
#22=IFCRELDEFINESBYPROPERTIES('rel-pset',$,$,$,(#4),#21);
ENDSEC;
END-ISO-10303-21;"#;

#[test]
fn imports_ifc_hierarchy_and_properties() {
    let project = omnim_ifc::import_ifc(IFC_WITH_HIERARCHY_AND_PROPS).unwrap();

    let sites = project.objects.find_by_type(c::ObjectType::Site);
    let buildings = project.objects.find_by_type(c::ObjectType::Building);
    let storeys = project.objects.find_by_type(c::ObjectType::BuildingStorey);
    let walls = project.objects.find_by_type(c::ObjectType::Wall);

    assert_eq!(sites.len(), 1);
    assert_eq!(buildings.len(), 1);
    assert_eq!(storeys.len(), 1);
    assert_eq!(walls.len(), 1);

    let site_id = sites[0].id.clone();
    let building_id = buildings[0].id.clone();
    let storey_id = storeys[0].id.clone();
    let wall = walls[0];

    assert_eq!(buildings[0].parent_id.as_deref(), Some(site_id.as_str()));
    assert_eq!(storeys[0].parent_id.as_deref(), Some(building_id.as_str()));
    assert_eq!(wall.parent_id.as_deref(), Some(storey_id.as_str()));
    assert_eq!(wall.ifc_class, "IFCWALL");
    assert_eq!(wall.get_property("IfcGlobalId").is_some(), true);
    assert_eq!(wall.property_sets.len(), 1);
    assert_eq!(wall.property_sets[0].name, "Pset_WallCommon");
    assert_eq!(wall.property_sets[0].properties[0].name, "FireRating");
}

#[test]
fn exports_project_to_ifc_with_hierarchy_and_property_sets() {
    let project = omnim_ifc::import_ifc(IFC_WITH_HIERARCHY_AND_PROPS).unwrap();
    let exported = omnim_ifc::export_ifc(&project);

    assert!(exported.contains("ISO-10303-21"));
    assert!(exported.contains("IFCWALL"));
    assert!(exported.contains("IFCRELAGGREGATES"));
    assert!(exported.contains("IFCPROPERTYSET"));
    assert!(exported.contains("Pset_WallCommon"));
}
