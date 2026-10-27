# OmniM Format Compatibility Matrix

**Last updated: 2026-10-27**

> **Honest-status note**: Earlier revisions of this document claimed "full" IFC/STEP/DXF/GIS/glTF support. That was aspirational, not factual. This document now reflects **what the code actually does**. Verified against `crates/ifc/src/lib.rs` and `crates/server/src/lib.rs` (the only import/export code paths in the repository).

## 1. Support Matrix (actual)

| Format | Import | Export | Status | Where in code |
|--------|--------|--------|--------|---------------|
| **IFC 2x3 / IFC 4** | ✅ Partial | ✅ Partial | Implemented — hand-written parser/writer, partial entity coverage | `crates/ifc/src/lib.rs`, REST: `/api/import/ifc`, `/api/export/ifc` |
| **JSON `.omnim`** (native) | ✅ | ✅ | Implemented — primary persistence format | `crates/server/src/lib.rs`, `crates/model/` |
| `.omnimlib` / `.omnimseries` | ✅ | ✅ | Implemented — catalog/series JSON files | `crates/model/src/library.rs`, `series_store.rs` |
| **STEP (AP203/AP242)** | ❌ | ❌ | Planned | — |
| **DXF / DWG** | ❌ | ❌ | Planned | — |
| **GIS (Shapefile / GeoJSON / GML / KML / GeoTIFF / LAS)** | ❌ | ❌ | Planned | — |
| **glTF / GLB** | ❌ | ❌ | Planned | — |
| **OBJ / FBX / STL** | ❌ | ❌ | Planned | — |
| **Navisworks / Revit / ArchiCAD / Tekla native** | ❌ | ❌ | Not planned (proprietary; use their IFC export) | — |

There are exactly two import/export routes in the server: IFC and native JSON. No other format code exists.

## 2. IFC Support (implemented)

### Implementation notes

- **Hand-written** STEP Physical File (SPF) text parser — no IFC library dependency.
- Target schema: **IFC 2x3 / IFC 4** (same code path; no IFC 4x3-specific entities).
- **Import** (`import_ifc_with_options`): line-by-line entity parsing, entity resolvable via `doc.get(ref)`, optional origin normalization, hierarchy + property-set reconstruction.
- **Export** (`export_ifc`): placement-based entities and solid/parametric geometry (extruded/revolved profiles via `IFCEXTRUDEDAREASOLID`/`IFCREVOLVEDAREASOLID`, mesh via faceted BRep), with IfcGlobalId/name/description properties and compressed GlobalIds.

### Entity types that actually load on import

Objects are created only when the entity type appears in the match table in `import_ifc_with_options` (`crates/ifc/src/lib.rs`). Currently:

| IFC entity | Maps to |
|------------|---------|
| IfcWall / IfcWallStandardCase | Wall |
| IfcBeam / IfcBeamStandardCase | Beam |
| IfcColumn / IfcColumnStandardCase | Column |
| IfcSlab / IfcSlabStandardCase / IfcPlate / IfcPlateStandardCase | Slab |
| IfcDoor | Door |
| IfcWindow | Window |
| IfcStair / IfcStairFlight | Stair |
| IfcRoof | Roof |
| IfcRoad | Road |
| IfcBridge | Bridge |
| IfcBuilding | Building |
| IfcBuildingStorey | BuildingStorey |
| IfcSite | Site |
| IfcSpace | Space |
| IfcMember / IfcMemberStandardCase, IfcFooting, IfcRamp, IfcFurnishingElement / IfcFurniture, IfcBuildingElementProxy | Generic |

Structural/semantic entities consumed indirectly:

- **IfcRelAggregates**, **IfcRelContainedInSpatialStructure** → parent/child containment
- **IfcRelDefinesByProperties** + **IfcPropertySet** + **IfcPropertySingleValue** → native PropertySets
- **IfcLocalPlacement** / **IfcAxis2Placement3D** / **IfcCartesianPoint** → placement extraction and geometry translation
- **IfcProductDefinitionShape** / **IfcShapeRepresentation** → geometry items

*Not handled:* all other entities (IfcSystem grouping, IfcZone, IfcMaterial / material layers, IfcRelAssociatesMaterial, quantities, classifications, advanced BRep/CSG/swept/constituent geometries, mapped items, etc.) are parsed into the raw entity table but not translated.

### Geometry coverage

Converters live in `crates/ifc/src/geometry.rs`; coverage is limited — primarily extruded-area-solids and simple items. Full IFC geometry (advanced BRep, CSG, swept composites, mapped items with transforms) is **not** supported.

## 3. Formats not implemented — planned order

Phase 11 remains the owning roadmap phase ([ROADMAP.md](ROADMAP.md)). Suggested order:

1. **IFC coverage widening** (within existing phase): more geometry types (mapped items, refined profiles), material/quantity/classification mapping, IFC 4x3 entities → move IFC to ✅ Full for 2x3/4.
2. **glTF/GLB export** — cheap win for three.js-based visualization; meshes already exist server-side.
3. **DXF import** (basic 2D entities via an open-source crate) — feeds sketch workflows.
4. **STEP AP242** — requires a proper CAD kernel boundary; evaluate existing crates before committing.
5. **GIS (GeoJSON first, then Shapefile)** — deferred until Phase 8 civil-domain work.
6. OBJ/FBX/STL — only on demand; not currently scheduled.

## 4. Proprietary BIM formats

| Format | Support | Notes |
|--------|---------|-------|
| Revit (.rvt) | ❌ Not planned | Direct access requires an Autodesk SDK/license — use Revit's IFC export instead |
| ArchiCAD (.pln), Tekla (.tpl) | ❌ Not planned | Same — use IFC export |

## 5. Removed content

Sections describing full geometry pipelines, round-trip fidelity targets, and import performance targets for formats that are not implemented have been **removed** until code exists to justify them. They will be restored, measured from real benchmarks, as each format lands.
