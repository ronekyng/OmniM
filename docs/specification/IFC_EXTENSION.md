# OmniM IFC Compatibility and Extension Strategy

## 1. Overview

OmniM's data model is designed as an **IFC superset** - fully compatible with IFC while adding support for domains not covered by the standard. This document outlines the compatibility strategy and extension mechanism.

---

## 2. Core Compatibility Principles

### 2.1 Alignment with IFC Where Possible
For building domain objects, OmniM follows IFC 4x3 classification and structure:
- Same object types and hierarchy
- Compatible property set definitions
- Similar relationship semantics

### 2.2 Clean Extension Mechanism
For domains beyond IFC's scope (civil, plant, mechanical), OmniM uses:
- Standardized extension prefixes
- Isolated domain namespaces
- No modification to core IFC semantics

### 2.3 Lossless Round-Trip
IFC import → OmniM operations → IFC export should preserve:
- All IFC entity types
- All property sets and quantities
- All relationships and connections
- Geometry fidelity within 0.1mm

---

## 3. IFC Class Mapping (Building Domain)

### 3.1 Spatial Structure

| OmniM Class | IFC Class | Notes |
|-------------|-----------|-------|
| Site | IfcSite | |
| Building | IfcBuilding | |
| BuildingStorey | IfcBuildingStorey | |
| Space | IfcSpace | |

### 3.2 Building Elements

| OmniM Class | IFC Class | Notes |
|-------------|-----------|-------|
| Wall | IfcWall / IfcWallStandardCase | |
| Beam | IfcBeam | |
| Column | IfcColumn | |
| Slab | IfcSlab | Includes floor, roof, footing |
| Door | IfcDoor | |
| Window | IfcWindow | |
| Stair | IfcStair / IfcStairFlight | |
| Roof | IfcRoof | |
| Railing | IfcRailing | |
| Ramp | IfcRamp | |
| Plate | IfcPlate | |
| Member | IfcMember | |

### 3.3 MEP Elements

| OmniM Class | IFC Class | Notes |
|-------------|-----------|-------|
| Duct | IfcDuctSegment | |
| Pipe | IfcPipeSegment | |
| DuctFitting | IfcDuctFitting | |
| PipeFitting | IfcPipeFitting | |
| Valve | IfcValve | |
| FlowTerminal | IfcFlowTerminal | Air terminals, outlets |
| FlowController | IfcFlowController | Dampers, etc. |
| EnergyConversion | IfcEnergyConversionDevice | Chillers, boilers |

---

## 4. Domain Extensions

OmniM extends beyond IFC with three additional domain-specific object hierarchies.

### 4.1 Civil Engineering Extension

**Namespace Prefix**: `OMNI_CIVIL_`

| Object Class | Description | IFC Equivalent |
|--------------|-------------|----------------|
| Road | Road alignment and pavement | IfcCivilElement (proposed) |
| Pavement | Road pavement structure (multi-layer) | IfcPavement (IFC 4x3) |
| Bridge | Bridge superstructure and substructure | IfcCivilElement (proposed) |
| Tunnel | Tunnel structure and lining | IfcCivilElement (proposed) |
| Dam | Dam structure and appurtenances | None |
| Culvert | Culvert structure | None |
| RetainingWall | Retaining wall structure | None (IfcWall used) |
| Alignment | Horizontal and vertical alignment | None |
| Profile | Cross section profile | None |
| Site | Project site spatial container | IfcSite |
| TerrainModel | TIN surface terrain | IfcGeographicElement (proposed, IFC 4x3) |
| CutVolume / FillVolume | Earthwork cut/fill volume | IfcGeographicElement (quantity) |
| GradingSurface | Design grading surface | IfcGeographicElement (design) |
| Slope | Slope face along boundary | IfcGeographicElement (slope) |
| FeatureLine | Control/feature line | None (IfcAnnotation) |
| ContourLine | Fixed-elevation contour | None (IfcAnnotation) |
| SiteComponent | Terrain-attached fixture | IfcElement (e.g., IfcLightFixture) |
| EarthworkBalance | Earthwork balance result | IfcGeographicElement (quantity) |

### 4.2 Plant Design Extension

**Namespace Prefix**: `OMNI_PLANT_`

| Object Class | Description | IFC Equivalent |
|--------------|-------------|----------------|
| Tank | Storage tank | IfcTank |
| Pump | Centrifugal, positive displacement | IfcPump |
| Valve | Gate, ball, butterfly, control | IfcValve |
| HeatExchanger | Shell & tube, plate | None |
| Reactor | Chemical process reactor | None |
| Vessel | Pressure vessel | IfcVessel |
| Piping | Plant piping network | IfcPipeSegment |
| Nozzle | Equipment connection point | None |
| Flange | Pipe connection flange | None |
| StructuralSteel | Steel members and connections | IfcBeam, IfcColumn |
| Platform | Access platform | None |
| Ladder | Access ladder | None |

### 4.3 Mechanical Design Extension

**Namespace Prefix**: `OMNI_MECH_`

| Object Class | Description | IFC Equivalent |
|--------------|-------------|----------------|
| Part | Individual mechanical part | IfcElementAssembly |
| Assembly | Part assembly | IfcElementAssembly |
| Fastener | Bolt, nut, screw, washer | None |
| Gear | Spur, bevel, helical, worm | None |
| Shaft | Rotating shaft | None |
| Bearing | Ball, roller, plain bearing | None |
| Motor | Electric motor | IfcMotor |
| Spring | Compression, extension, torsion | None |
| Seal | Gasket, O-ring | None |
| Bushing | Sleeve bushing | None |
| Housing | Enclosure or casing | None |

---

## 5. Property Set Strategy

### 5.1 IFC Property Sets (Pset)

OmniM natively supports all standard IFC property sets:
- `Pset_WallCommon`, `Pset_BeamCommon`, `Pset_ColumnCommon`, etc.
- `Qto_WallBaseQuantities`, etc. for quantities

These are stored and exported exactly as IFC specification.

### 5.2 OmniM Extended Property Sets

Domain extensions use prefix-based property sets:

| Domain | Prefix | Example |
|--------|--------|---------|
| Civil | `Pset_Civil_` | `Pset_Civil_Road` |
| Plant | `Pset_Plant_` | `Pset_Plant_Pump` |
| Mechanical | `Pset_Mech_` | `Pset_Mech_Fastener` |

### 5.3 Custom Property Sets

User-defined property sets use:
- No reserved prefix
- Application-specific prefix: `Pset_<AppName>_`
- Company-specific prefix: `Pset_<Company>_`

---

## 6. IFC Export Strategy

### 6.1 Standard IFC Objects
Objects with IFC equivalents export as native IFC entities with full fidelity.

### 6.2 Extended Domain Objects
For objects without direct IFC equivalents:
1. **Map to closest IFC supertype**: e.g., `IfcCivilElement`, `IfcDiscreteAccessory`
2. **Add OmniM type to property set**: `Pset_OmniM_ObjectType = "HeatExchanger"`
3. **Add domain-specific properties** in their respective property sets
4. **Mark as extended entity** with custom property for re-identification

### 6.3 Export Configuration Options
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

---

## 7. IFC Import Strategy

### 7.1 Standard IFC Entities
- Direct mapping to OmniM object types
- Property sets imported natively
- Relationships preserved as OmniM relations

### 7.2 Extended IFC Entities
- Objects with `Pset_OmniM_ObjectType` → map to extended domain types
- Unknown property sets → preserved as custom properties
- Unknown entity types → mapped to Generic object with original IFC class stored

### 7.3 Geometry Import Pipeline
```
IFC Geometry Definition
    ↓
Parse (Extrusion, BRep, CSG, etc.)
    ↓
Convert to meters, standard orientation
    ↓
Geometry Healing (fix invalid topology)
    ↓
Tessellate to mesh (for rendering)
    ↓
Generate LOD variants
    ↓
Store as native OmniM geometry
```

---

## 8. Version Compatibility

### 8.1 Supported IFC Versions

| Version | Import | Export |
|---------|--------|--------|
| IFC 2x3 | ✅ Yes | ✅ Yes |
| IFC 4 | ✅ Yes | ✅ Yes |
| IFC 4x3 | ✅ Yes | ✅ Yes |

### 8.2 Version Conversion
OmniM supports automatic version conversion during export:
- IFC 4 → IFC 2x3: Downgrade with warning for unsupported features
- IFC 2x3 → IFC 4: Upgrade with property set mapping

---

## 9. LandXML Terrain Interoperability

> **Research ref**: `docs/research/site_terrain_components.md §9.2`. Civil 3D and Bentley natively support LandXML for terrain data exchange. OmniM supports dual-format interchange: **IFC 4x3** for spatial hierarchy (IfcSite) + **LandXML** for terrain/TIN data.

| LandXML Element | OmniM Mapping |
|-----------------|---------------|
| `Surfaces` | TerrainModel collection |
| `Surface` | TerrainModel (single surface) |
| `Definition` | TerrainVertex + TerrainTriangle |
| `Breaklines` | TerrainBreakline |
| `Boundary` | TerrainBoundary |
| `Grade` | GradingSurface |

LandXML is the terrain data interchange standard; IFC 4x3 provides the spatial container hierarchy (IfcSite → IfcBuilding → IfcBuildingStorey → IfcSpace) and geographic element mapping (IfcGeographicElement for TerrainModel).

---

## 11. Testing and Validation

### 11.1 Reference Models
OmniM maintains a test suite of reference models:
- Building Smart official test cases
- Real-world project examples
- Edge case models (complex geometry, large files)

### 11.2 Validation Checks
For each import/export:
1. Geometry validity (no non-manifold edges, closed solids)
2. Object count verification
3. Property round-trip check
4. Relationship integrity
5. File size and performance benchmarks

---

## 12. Open Questions

1. Should Civil/Plant/Mechanical extensions follow any existing standards (ISO 15926, etc.)?
2. What is the best way to represent alignments and profiles for IFC export?
3. Should parametric history be preserved in IFC or only the final geometry?
4. How to handle fastener standards and catalog data across domains?
5. What level of STEP AP242 PMI should be mapped to OmniM properties?
