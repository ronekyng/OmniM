---
title: "OmniM IFC Compatibility and Extension Strategy"
status: design
last_updated: 2026-10-27
applies_to: "proto/, crates/model"
evidence: "n/a (design contract)"
---
# OmniM IFC Compatibility and Extension Strategy

> **Last updated: 2026-10-27.** Status marks follow [FORMAT_COMPAT.md](../status/FORMAT_COMPAT.md): ✅ current / 🔀 partial / 🗓 planned. Code is the source of truth; claims below describe the hand-written parser/writer in `crates/ifc/src/lib.rs`.

**Relationship to FORMAT_COMPAT**: this document is the *strategy and class-mapping* companion to [FORMAT_COMPAT.md](../status/FORMAT_COMPAT.md), which is the authoritative implementation matrix. Where the two differ, FORMAT_COMPAT wins. In particular: only an IFC 2x3/IFC 4 subset is implemented, there is no IFC 4x3-specific entity support, and no lossless round-trip or LandXML capability is claimed today.

## 1. Overview

OmniM's data model is designed as an **IFC superset** — compatible with an IFC subset while adding domain concepts not covered by the standard. This document outlines the extension namespace strategy and the per-class mapping status.

**Current reality (2026-10-27)**: IFC import/export is a partial, hand-written STEP Physical File parser/writer. It covers a subset of IFC 2x3/IFC 4 entities and simple geometry. It does **not** implement IFC 4x3, material/quantity/classification mapping, advanced BRep/CSG/mapped-item geometry, or lossless round-trip. The extension namespaces and property-set strategies below remain design targets (🗓).

---

## 2. Core Compatibility Principles

### 2.1 Alignment with IFC Where Possible — 🔀 Partial

For building-domain objects, OmniM aligns with an **IFC 2x3/IFC 4 subset**:

- A subset of object types and hierarchy is mapped (see §3).
- ✅ Property sets: single-value `IfcPropertySet` values are reconstructed on import.
- 🔀 Relationship semantics: containment relationships are reconstructed; connection semantics are not.
- 🗓 IFC 4x3 classification/structure: not targeted and not supported.

### 2.2 Clean Extension Mechanism — 🗓 Planned

For domains beyond IFC's scope (civil, plant, mechanical), the design uses:

- Standardized extension prefixes (`OMNI_CIVIL_`, `OMNI_PLANT_`, `OMNI_MECH_`).
- Isolated domain namespaces.
- No modification to core IFC semantics.

The namespace scheme is a design convention only; no extension entity is currently written as a distinct IFC entity. Extended objects fall back to `IfcBuildingElementProxy` on export (🔀) and are not round-tripped as typed domain objects (🗓).

### 2.3 Lossless Round-Trip — 🗓 Planned (not claimed)

IFC import → OmniM operations → IFC export is **not** lossless today. The former target ("all entities, all property sets/quantities, all relationships, geometry within 0.1 mm") is retained only as a future goal:

- 🗓 All IFC entity types — not supported; only the match table in §3.
- 🗓 All property sets and quantities — only single-value property sets are read; quantities are not translated.
- 🗓 All relationships and connections — only aggregates/containment are reconstructed.
- 🗓 Geometry fidelity within 0.1 mm — no such guarantee; geometry is tessellated and simplified.

---

## 3. IFC Class Mapping (Building Domain)

Status legend: ✅ current (typed import and/or export) · 🔀 partial (import only, export falls back to Proxy, or vice versa) · 🗓 planned (no typed mapping in code).

### 3.1 Spatial Structure

| OmniM Class | IFC Class | Status | Notes |
|-------------|-----------|--------|-------|
| Site | IfcSite | ✅ | Imported/exported as a spatial container |
| Building | IfcBuilding | ✅ | |
| BuildingStorey | IfcBuildingStorey | ✅ | |
| Space | IfcSpace | ✅ | |

### 3.2 Building Elements

| OmniM Class | IFC Class | Status | Notes |
|-------------|-----------|--------|-------|
| Wall | IfcWall / IfcWallStandardCase | ✅ | |
| Beam | IfcBeam / IfcBeamStandardCase | ✅ | |
| Column | IfcColumn / IfcColumnStandardCase | ✅ | |
| Slab | IfcSlab / IfcPlate | ✅ | Floor, roof and footing also import into Slab |
| Door | IfcDoor | ✅ | |
| Window | IfcWindow | ✅ | |
| Stair | IfcStair / IfcStairFlight | 🔀 | Imports as Stair; **exports as `IfcBuildingElementProxy`** (DATA_MODEL §10.1) |
| Roof | IfcRoof | 🔀 | Imports as Roof; **exports as `IfcBuildingElementProxy`** (DATA_MODEL §10.1) |
| Railing | IfcRailing | 🗓 | Not in the import match table; export falls back to Proxy |
| Ramp | IfcRamp | 🔀 | Imports as Generic; no typed Ramp round-trip |
| Plate | IfcPlate | ✅ | Imported as Slab |
| Member | IfcMember / IfcMemberStandardCase | 🔀 | Imports as Generic |

### 3.3 MEP Elements

| OmniM Class | IFC Class | Status | Notes |
|-------------|-----------|--------|-------|
| Duct | IfcDuctSegment | 🗓 | No typed import mapping; export falls back to `IfcBuildingElementProxy` |
| Pipe | IfcPipeSegment | 🗓 | Same |
| DuctFitting | IfcDuctFitting | 🗓 | Same |
| PipeFitting | IfcPipeFitting | 🗓 | Same |
| Valve | IfcValve | 🗓 | Same |
| FlowTerminal | IfcFlowTerminal | 🗓 | Air terminals, outlets |
| FlowController | IfcFlowController | 🗓 | Dampers, etc. |
| EnergyConversion | IfcEnergyConversionDevice | 🗓 | Chillers, boilers |

---

## 4. Domain Extensions

OmniM's design extends beyond IFC with three additional domain-specific object hierarchies. All typed extension entities are 🗓 planned; only the import-name matches and Proxy fallback noted below exist today.

### 4.1 Civil Engineering Extension

**Namespace Prefix**: `OMNI_CIVIL_` (🗓 design convention)

| Object Class | Description | IFC Equivalent | Status |
|--------------|-------------|----------------|--------|
| Road | Road alignment and pavement | IfcCivilElement (proposed) | 🔀 Imports when `IfcRoad` appears; export is Proxy; no 4x3 conformance |
| Pavement | Road pavement structure (multi-layer) | IfcPavement (IFC 4x3) | 🗓 |
| Bridge | Bridge superstructure and substructure | IfcCivilElement (proposed) | 🔀 Imports when `IfcBridge` appears; export is Proxy |
| Tunnel | Tunnel structure and lining | IfcCivilElement (proposed) | 🗓 |
| Dam | Dam structure and appurtenances | None | 🗓 |
| Culvert | Culvert structure | None | 🗓 |
| RetainingWall | Retaining wall structure | None (IfcWall used) | 🗓 (imports only as Wall when written as IfcWall) |
| Alignment | Horizontal and vertical alignment | None | 🗓 |
| Profile | Cross section profile | None | 🗓 |
| Site | Project site spatial container | IfcSite | ✅ (same spatial container as §3.1) |
| TerrainModel | TIN surface terrain | IfcGeographicElement (proposed, IFC 4x3) | 🗓 |
| CutVolume / FillVolume | Earthwork cut/fill volume | IfcGeographicElement (quantity) | 🗓 |
| GradingSurface | Design grading surface | IfcGeographicElement (design) | 🗓 |
| Slope | Slope face along boundary | IfcGeographicElement (slope) | 🗓 |
| FeatureLine | Control/feature line | None (IfcAnnotation) | 🗓 |
| ContourLine | Fixed-elevation contour | None (IfcAnnotation) | 🗓 |
| SiteComponent | Terrain-attached fixture | IfcElement (e.g., IfcLightFixture) | 🗓 |
| EarthworkBalance | Earthwork balance result | IfcGeographicElement (quantity) | 🗓 |

### 4.2 Plant Design Extension

**Namespace Prefix**: `OMNI_PLANT_` (🗓 design convention)

| Object Class | Description | IFC Equivalent | Status |
|--------------|-------------|----------------|--------|
| Tank | Storage tank | IfcTank | 🗓 (export falls back to Proxy) |
| Pump | Centrifugal, positive displacement | IfcPump | 🗓 |
| Valve | Gate, ball, butterfly, control | IfcValve | 🗓 |
| HeatExchanger | Shell & tube, plate | None | 🗓 |
| Reactor | Chemical process reactor | None | 🗓 |
| Vessel | Pressure vessel | IfcVessel | 🗓 |
| Piping | Plant piping network | IfcPipeSegment | 🗓 |
| Nozzle | Equipment connection point | None | 🗓 |
| Flange | Pipe connection flange | None | 🗓 |
| StructuralSteel | Steel members and connections | IfcBeam, IfcColumn | 🗓 |
| Platform | Access platform | None | 🗓 |
| Ladder | Access ladder | None | 🗓 |

### 4.3 Mechanical Design Extension

**Namespace Prefix**: `OMNI_MECH_` (🗓 design convention)

| Object Class | Description | IFC Equivalent | Status |
|--------------|-------------|----------------|--------|
| Part | Individual mechanical part | IfcElementAssembly | 🗓 |
| Assembly | Part assembly | IfcElementAssembly | 🗓 |
| Fastener | Bolt, nut, screw, washer | None | 🗓 |
| Gear | Spur, bevel, helical, worm | None | 🗓 |
| Shaft | Rotating shaft | None | 🗓 |
| Bearing | Ball, roller, plain bearing | None | 🗓 |
| Motor | Electric motor | IfcMotor | 🗓 |
| Spring | Compression, extension, torsion | None | 🗓 |
| Seal | Gasket, O-ring | None | 🗓 |
| Bushing | Sleeve bushing | None | 🗓 |
| Housing | Enclosure or casing | None | 🗓 |

---

## 5. Property Set Strategy

### 5.1 IFC Property Sets (Pset) — 🔀 Partial

- ✅ `IfcRelDefinesByProperties` + `IfcPropertySet` + `IfcPropertySingleValue` are read on import and reconstructed as native property sets.
- ✅ Single-value properties are written on export for supported entities (IfcGlobalId/name/description plus simple property values).
- 🗓 Full standard property-set coverage (`Pset_WallCommon`, `Pset_BeamCommon`, `Pset_ColumnCommon`, …) is not guaranteed.
- 🗓 `Qto_*` quantities are not translated.
- 🗓 Material associations (`IfcMaterial`, `IfcRelAssociatesMaterial`), material layers, and classification references are not translated.

### 5.2 OmniM Extended Property Sets — 🗓 Planned

Domain extensions are designed to use prefix-based property sets:

| Domain | Prefix | Example |
|--------|--------|---------|
| Civil | `Pset_Civil_` | `Pset_Civil_Road` |
| Plant | `Pset_Plant_` | `Pset_Plant_Pump` |
| Mechanical | `Pset_Mech_` | `Pset_Mech_Fastener` |

### 5.3 Custom Property Sets — 🗓 Planned

User-defined property sets are designed to use:

- No reserved prefix.
- Application-specific prefix: `Pset_<AppName>_`.
- Company-specific prefix: `Pset_<Company>_`.

Round-trip preservation of unknown property sets is not implemented.

---

## 6. IFC Export Strategy

### 6.1 Standard IFC Objects — ✅ Current (subset) / 🔀 Partial

Objects with an implemented IFC equivalent export as native IFC entities (Wall, Beam, Column, Slab, Door, Window, Site, Building, BuildingStorey, Space). Geometry is written as `IFCEXTRUDEDAREASOLID` / `IFCREVOLVEDAREASOLID` or faceted BRep where available.

- 🔀 Stair and Roof export as `IfcBuildingElementProxy` (see DATA_MODEL §10.1), not `IfcStair` / `IfcRoof`.
- 🔀 All other object types fall back to `IfcBuildingElementProxy`; no typed domain entity is emitted.
- 🗓 Full-fidelity export of all properties and relationships is not implemented.

### 6.2 Extended Domain Objects — 🔀 Proxy fallback / 🗓 design

For objects without direct IFC equivalents, the target strategy is:

1. **Map to closest IFC supertype**: e.g., `IfcCivilElement`, `IfcDiscreteAccessory`.
2. **Add OmniM type to property set**: `Pset_OmniM_ObjectType = "HeatExchanger"`.
3. **Add domain-specific properties** in their respective property sets.
4. **Mark as extended entity** with a custom property for re-identification.

Current code performs only the Proxy fallback (🔀). Steps 2–4 are 🗓 planned; `Pset_OmniM_ObjectType` is not written today.

### 6.3 Export Configuration Options — 🗓 Planned

```
Export Profile:
  ├── Full (all objects, all properties)
  ├── Geometry Only (no properties)
  ├── Spatial Only (site/building/storey hierarchy)
  └── Coordination View (for clash detection)

LOD Filter:
  ├── LOD 100 (Concept)
  ├── LOD 200 (Design)
  ├── LOD 300 (Detailed)
  ├── LOD 400 (Construction)
  └── LOD 500 (As-built)
```

No export-profile or LOD-filter option is exposed by the current server.

---

## 7. IFC Import Strategy

### 7.1 Standard IFC Entities — ✅ Current (subset)

- ✅ A fixed match table maps entity types to OmniM object types (see FORMAT_COMPAT §2 and §3 above).
- ✅ Property sets with single values are imported natively.
- 🔀 Only `IfcRelAggregates` and `IfcRelContainedInSpatialStructure` are reconstructed as containment; other relationships are not.

### 7.2 Extended IFC Entities — 🗓 Planned

- 🗓 Objects with `Pset_OmniM_ObjectType` → map to extended domain types.
- 🔀 Unknown property sets are not round-tripped as custom properties.
- 🔀 Unknown entity types are parsed into the raw entity table but **not** converted to objects; there is no Generic fallback object created for them today.

### 7.3 Geometry Import Pipeline — 🔀 Partial

```
IFC Geometry Definition
    ↓
Parse (extruded-area solids, revolved solids, faceted BRep)
    ↓
Convert to meters, standard orientation
    ↓
Tessellate to mesh (for rendering)
    ↓
Store as native OmniM geometry
```

- ✅ The current pipeline parses extruded/revolved profiles and faceted BRep, normalizes placement/origin, and tessellates to `MeshGeometry`.
- 🗓 Geometry healing (fixing invalid topology) is not implemented.
- 🗓 LOD variant generation is not implemented.
- 🗓 Advanced BRep, CSG, swept solids, constituent geometries, and mapped items with transforms are not supported.

---

## 8. Version Compatibility

### 8.1 Supported IFC Versions

| Version | Import | Export | Status |
|---------|--------|--------|--------|
| IFC 2x3 | ✅ Subset | ✅ Subset | 🔀 Partial — same code path as IFC 4 |
| IFC 4 | ✅ Subset | ✅ Subset | 🔀 Partial |
| IFC 4x3 | ❌ | ❌ | 🗓 Not supported — no IFC 4x3-specific entities |

**No IFC 4x3 support is claimed.** `IfcRoad` / `IfcBridge` names may be recognized opportunistically by the hand-written parser, but that is not schema-level 4x3 conformance.

### 8.2 Version Conversion — 🗓 Planned

Automatic version conversion during export is not implemented. The former design (IFC 4 → 2x3 downgrade with warnings; 2x3 → 4 upgrade with property-set mapping) remains a target.

---

## 9. LandXML Terrain Interoperability — 🗓 Planned Only

> **Research ref**: `docs/research/site_terrain_components.md §9.2`. Civil 3D and Bentley natively support LandXML for terrain data exchange.

LandXML is **not implemented**. There is no LandXML reader/writer in the repository, and no dual-format interchange exists today. The mapping below is a design target only (🗓).

| LandXML Element | OmniM Mapping (planned) |
|-----------------|-------------------------|
| `Surfaces` | TerrainModel collection |
| `Surface` | TerrainModel (single surface) |
| `Definition` | TerrainVertex + TerrainTriangle |
| `Breaklines` | TerrainBreakline |
| `Boundary` | TerrainBoundary |
| `Grade` | GradingSurface |

The former claim that "IFC 4x3 provides the spatial container hierarchy and geographic element mapping for TerrainModel" is also 🗓 planned, because IFC 4x3 is not supported.

---

## 10. Testing and Validation

### 10.1 Reference Models — 🗓 Planned

A maintained suite of BuildingSMART reference models, real-world project examples, and edge-case models is a design target; no such suite is part of the current test coverage.

### 10.2 Validation Checks — 🔀 Partial

For each import/export, the current code performs a subset of these checks:

1. 🔀 Geometry validity is limited to what the tessellation/parser accepts; no global non-manifold/closed-solid validation is run.
2. ✅ Imported object counts are reported (`ImportIfcResponse.objects`).
3. 🗓 Property round-trip checks are not automated.
4. 🔀 Relationship integrity is limited to containment reconstruction.
5. 🗓 File-size and performance benchmarks are not maintained.

---

## 11. Open Questions

1. Should Civil/Plant/Mechanical extensions follow any existing standards (ISO 15926, etc.)?
2. What is the best way to represent alignments and profiles for IFC export?
3. Should parametric history be preserved in IFC or only the final geometry?
4. How to handle fastener standards and catalog data across domains?
5. What level of STEP AP242 PMI should be mapped to OmniM properties?
