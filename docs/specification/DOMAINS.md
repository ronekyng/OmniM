# OmniM Domain Coverage

## 1. Overview

OmniM is a unified engineering information platform that supports four major engineering domains without prioritizing one over another:

1. Building Engineering
2. Civil Engineering
3. 3D Plant Design
4. Mechanical Design

All domains share the same core data model, geometry model, property system, relationship system, AMIR instruction layer, and engineering project lifecycle model. OmniM is not only a model container; it also manages requirements, design, procurement, construction, commissioning, asset, operation, maintenance, production, and decommissioning data for every domain.

---

## 2. Building Engineering

Building Engineering covers architectural, structural, MEP, and facility-related information.

### Core Objects

| Category | Objects |
|----------|---------|
| Spatial | Site, Building, BuildingStorey, Space, Zone |
| Architectural | Wall, Door, Window, Roof, Ceiling, CurtainWall, Stair, Ramp |
| Structural | Beam, Column, Slab, Footing, Pile, Brace, Truss, Rebar, Connection |
| MEP | Duct, Pipe, CableTray (straight→LayerExtrusion, bend→LinearComponent), Valve, Damper, Pump, Fan, AirTerminal |
| Equipment | Chiller, Boiler, AHU, Transformer, Switchboard, Fixture |

### Key Workflows
- BIM authoring and coordination
- Quantity take-off
- Clash detection
- Code compliance checking
- Facility handover and asset management

### Standards Alignment
- IFC 2x3 / IFC 4 / IFC 4x3
- ISO 19650 information management
- COBie asset handover

---

## 3. Civil Engineering

Civil Engineering covers infrastructure, transportation, land development, utilities, and large-scale site works.

### Core Objects

| Category | Objects |
|----------|---------|
| Alignment | HorizontalAlignment, VerticalAlignment, Profile, Station |
| Road | Road, Lane, Shoulder, PavementLayer, Median, Curb |
| Bridge | Bridge, Pier, Abutment, Girder, Deck, Bearing, ExpansionJoint |
| Tunnel | Tunnel, Lining, Portal, Shaft, Segment, Ring, CrossPassage |
| Hydraulic | Dam, Culvert, Channel, Drainage, RetentionBasin |
| Utility | WaterPipe, SewerPipe, StormPipe, Manhole, UtilityPole, DuctBank |
| Terrain/Site | TerrainModel, GradingSurface, Slope, FeatureLine, ContourLine, SiteComponent |
| Earthwork | CutVolume, FillVolume, EarthworkBalance, RetainingWall |

> **Bentley bridge/tunnel component details** (`docs/research/bentley_openroads_components.md`):
> - **Bearing types** (§2.3.1): Elastomeric Pad, Pot, Disc, Mechanical
> - **Expansion Joint types** (§2.4): Strip Seal, Modular, Finger Plate, Compression Seal
> - **Deck Breakbacks** (§2.2.2): bridge decks can be non-continuous within a span via breakbacks, accommodating expansion joints or skew-segment splits. Breakbacks are modeled as intentional gap positions in the Deck's LinearComponent profile sequence.
> - **Pier Config** (§3.1.1): Pier Type (Single Column / Multi Column / Wall / Hammer Head); Cap Shape (Tapered / Rectangle / Inverted T / Variable); Column Shape (Round / Rectangular / Custom)
> - **TBM Tunnel Unit Ring (UTR)** (§4.4.2): Taper, Number of Dowels, First Ring Position, Excluded Ring Positions, Ring Name Prefix
> - **TBM UTR curve-fitting mechanism** (§4.4): segmental lining fits horizontal curves by **ring taper** — each universal ring has a max/min width difference (the *taper*). Successive rings are rotated relative to one another by the **dowel pitch angle** (derived from the Number of Dowels, e.g. 360°/N) to steer the lining axis along the design curve. Ring sequencing obeys **Inherited** (follow-the-template/first-ring) and **Excluded Ring Positions** (positions that may not combine with the first ring) rules. This is the mechanical principle enabling curved TBM advance — modeled in OmniM as a ring-sequence `Assembly` whose per-ring taper direction is a sequence `VariationRule`.
> - **Tunnel Template Catalog** (§4): Conventional/TBM tunnels, Portal, Shafts, Rooms, Excavation, Reinforcement, Auxiliaries, Materials
>
> **Civil3D Pipe Network Rules** (`docs/research/civil3d_components.md §6.1`): Pipe networks (gravity and pressure) are governed by engineering rules: Minimum Cover, Maximum/Minimum Slope, Minimum Slope, Maximum Depth, Minimum Pipe Length. These are enforced by the Series gate as SpecComponent constraints on the pipe's ComponentDef parameters.
>
> **Pipe Network Parts List** (from Civil3D §6.1): A pipe network uses a network-scoped **Parts List** — a curated set of pipes + structures specific to that network (e.g., which pipe sizes, fittings, and structure types are available for a given sanitary/storm system). This is conceptually distinct from the project-wide Series decision tree: the Parts List is bound to a single network and governs which parts can be placed/extended within it, whereas a Series is a project-level spec applied across all components. In OmniM, the network-scoped Parts List maps to a network-local catalog/constraint set layered on top of the Series gate.
>
> **Bentley Civil Model data container** (`docs/research/bentley_openroads_components.md §1.3`): Aggregates Linear Elements, Point Elements, 3D Linear, Terrain Models, Corridors, Superelevation, Civil Objects, Civil Cells, and Reference Models. OmniM's equivalent is the Project Object Store with ObjectType categorization and Relation Manager for cross-object linkages.

### Key Workflows
- Corridor and alignment modeling
- Terrain and grading
- Earthwork calculation
- Utility coordination
- Infrastructure asset management

### Standards Alignment
- IFC 4x3 infrastructure extensions
- LandXML
- GeoJSON / Shapefile / GML
- National roadway and bridge standards

---

## 4. 3D Plant Design

3D Plant Design covers process plants, industrial facilities, equipment layouts, piping, instrumentation, and structural supports.

### Core Objects

| Category | Objects |
|----------|---------|
| Equipment | Tank, Vessel, Reactor, HeatExchanger, Pump, Compressor, Filter, Support |
| Piping | Pipe, PipeSegment, Elbow, Tee, Reducer, Flange, Nozzle, Valve, ControlValve |
| Instrumentation | Instrument, Sensor, Transmitter, ControlValve, JunctionBox, Cable, Conduit |
| Structure | StructuralSteel, Platform, Stair, Ladder, Handrail, Support |
| Process | ProcessLine, System, Unit, Area, PipingSpec, EquipmentSpec |
| Electrical | CableTray, Cable, Conduit, Panel, MotorControlCenter |

### Key Workflows
- Equipment layout
- Piping routing
- Pipe specification and isometrics
- Clash detection
- P&ID linkage
- Construction work packages

> **AVEVA Instrumentation/Electrical module structure** (`docs/research/e3d_instrumentation_components.md §1.4-§1.5`): AVEVA Instrumentation consists of 4 modules — Instrument Engineer (index/spec), Wiring Manager (termination/loop), Designer (hookup), and Browser/Calculation tools. AVEVA Electrical consists of 3 modules — Engineer, Wiring Manager, Designer. OmniM unifies these into a single LogicalObject + Assembly model: Instrument Engineer ↔ LogicalObject Manager, Wiring Manager ↔ Relation Manager, Designer ↔ Assembly GenRule.
>
> **MCC hierarchical assembly** (`docs/research/e3d_instrumentation_components.md §3.4.2`): MotorControlCenter (MCC) is a **3-level nested Assembly** — `MCC → Cubicle → Compartment → Unit` (breaker + contactor + overload relay + control transformer + terminal block) — plus a shared **Busbar System** and **Cable Alley**. It carries 3 wiring classes (Power / Control / Auxiliary, per IEC 61439) and start modes (DOL / Star-Delta / VFD).
>
> **Panel Schedule parameter set** (`docs/research/e3d_instrumentation_components.md §3.3`): `Panel` carries a schedule — main breaker rating/type; per-branch breaker ratings (MCB / MCCB / ELCB / RCCB); way count; spare ways; per-phase load + total + demand factor + phase balance; feeding method; and IP / NEMA / Ex ratings.
>
> **Support as Assembly** (`docs/research/e3d_instrumentation_components.md §5.2.3`): `Support` (支吊架) should be an **Assembly** of catalog standard parts — hanger rod, spring can, U-bolt/clevis, shoe, guide, anchor, baseplate — not a single primitive.

### Standards Alignment
- ISO 15926
- P&ID standards
- ASME piping codes
- IFC equipment and flow segments where applicable

---

## 5. Mechanical Design

Mechanical Design covers parts, assemblies, mechanical systems, manufacturing-ready geometry, and product structure.

### Core Objects

| Category | Objects |
|----------|---------|
| Product Structure | Assembly, SubAssembly, Part, Configuration, Variant |
| Machine Elements | Gear, Shaft, Bearing, Coupling, Spring, Seal, Bushing |
| Fasteners | Bolt, Nut, Washer, Screw, Rivet, Pin, ThreadedInsert |
| Motion | Joint, Constraint, Axis, KinematicPair, Actuator |
| Manufacturing | Hole, Pocket, Slot, Chamfer, Fillet, Thread, PMI |
| Electrical Mechanical | Motor, Encoder, Sensor, Housing, Connector |

### Key Workflows
- Assembly modeling
- Precise BRep geometry editing
- Product structure management
- GD&T / PMI handling
- STEP import/export
- Bill of materials generation

### Standards Alignment
- STEP AP203 / AP214 / AP242
- ISO 10303
- ISO GPS / GD&T standards
- Manufacturer catalog standards

---

## 6. Cross-Domain Object Relationships

OmniM supports relationships across domain boundaries.

### Examples

| Relationship | Example |
|--------------|---------|
| Building ↔ Civil | Building connected to site utilities and roads |
| Building ↔ Plant | Industrial building contains process equipment |
| Plant ↔ Mechanical | Pump assembly represented as mechanical product structure |
| Civil ↔ Mechanical | Bridge bearing modeled as mechanical component |
| Plant ↔ Building MEP | HVAC equipment shared between building and process systems |

---

## 7. Shared Concepts Across Domains

All domains share these core concepts and a unified geometry pipeline:

| Concept | Description |
|---------|-------------|
| Unified Geometry Kernel | Box/Cylinder/Cone/Sphere/Extrusion/Sweep/Revolve/Loft + Boolean ops (shared across all domains) |
| Default Template Registry | Maps each ObjectType to a catalog ComponentDef, enabling unified `create <type> <name>` across domains |
| Component Catalog | Parametric 3D component definitions with feature trees, connectors, and typed params |
| Series Library | Spec decision trees for project-approved component selection |
| Object | Any physical, spatial, system, or logical engineering entity |
| Geometry | Parametric, BRep, mesh, curve, or point cloud representation |
| Property | Typed metadata, parameters, calculations, and specifications |
| Relation | Containment, connection, spatial, semantic, or system relation |
| Material | Physical and visual material definition |
| Version | Change history and lifecycle state |
| Lifecycle Record | Requirement, design package, procurement package, construction activity, commissioning record, asset, work order, issue, or sensor stream |
| AMIR | AI intermediate instruction representation for both model operations and lifecycle operations |

### 7.1 Domain GenRule Classification (Legacy)

> **Note**: Since Phase 6, the primary geometry generation uses **catalog default templates** (see [DATA_MODEL.md §6](DATA_MODEL.md#6-object-generation-rule-system-genrule)). The GenRule types below are a **conceptual classification** only — they describe how geometry is *structured* per domain, but the actual generation is via `default_catalog_component()` mapping to ComponentDef feature trees.
>
> The 4-class component taxonomy (Surface/Linear/Point/Body, see [Data Model §1](DATA_MODEL.md#1-overview)) provides the higher-level conceptual classification: walls and slabs are **Surface** components (LayerExtrusion), beams and pipes are **Linear** components (LinearComponent), tanks and pumps are **Body** components (Primitive/Assembly), and point clouds/terrain are **Point** components (non-GenRule).

The 5 GenRule types provide a conceptual taxonomy for how objects in each domain structure their geometry:

```
Domain        Primary GenRule            Typical Objects      Secondary GenRule
──────        ────────────────────       ──────────────       ───────────────────
Building      ② LayerExtrusion           Wall, Slab           ⑤ BooleanSubtract
              (walls)                                                      (door/window openings)
              ③ LinearComponent          Beam, Column
              (structural elements)        (fixed section)
              ③ LinearComponent          Rebar
              (reinforcement)              (circular section + path + array)
              ④ Assembly                 Connection
              (steel connections)          (Tekla-style processual components)
Plant         ① Primitive                Tank, Pump           ③ LinearComponent
               (PDMS primitives)          Valve, Pipe             (pipe rack/piping)
                                         Instrument, Sensor       (cable, conduit)
                                                               ⑤ BooleanSubtract
               ④ Assembly                 Transmitter,                     (SNO cutouts)
               (equipment assemblies)     ControlValve, JB,                 (panel cutouts)
                                         Panel, MCC, Support
               (CableTray nuance: straight=LayerExtrusion, bend/tee=LinearComponent/Assembly)
Civil         ③ LinearComponent          Road, Bridge         ① Primitive
               (corridor modeling)        Tunnel, Culvert, Dam    (standalone elements)
                                         Pier: Cap=LinearComponent, Column=LinearComponent/Primitive,
                                         Footing=Primitive, Pile=Primitive
                                         Abutment: Stem=LinearComponent, Wingwall=LinearComponent
                                         (angled path), ApproachSlab=LayerExtrusion
                                         Deck: multi-layer LinearComponent (curved path)
                                         Tunnel Lining: multilinear LinearComponent

Mechanical    ④ Assembly                Equipment assembly   ① Primitive
              (part composition)         Gear, Shaft             (base parts)
```

### 7.2 LinearComponent: Cross-Domain Unified Path-Based Modeling

LinearComponent is one of OmniM's core innovations. It unifies building structural elements (Beam, Column with fixed cross-section + path), Civil3D roads (alignment + cross-section + width variation rules), and PDMS pipes (diameter + path) into a single abstraction: **path + variable cross-section + external constraints**.

Building structural beams/columns = LinearComponent with constant cross-section (path + fixed section profile).
Road = LinearComponent with variable cross-section (horizontal alignment + vertical curve + segmented cross-section + terrain target constraint).

> **Wall is not a LinearComponent**: Wall is a **Surface** component (see [Data Model §1](DATA_MODEL.md#1-overview)), whose geometry is defined by a surface boundary (rectangle or sketch profile) with a layered material stack extruded orthogonally as thickness. Walls use **LayerExtrusion** GenRule, not LinearComponent.

---

### 7.3 Component Creation Workflows

This section describes the **correct user-facing creation workflow** for each component type, derived from cross-platform research (Revit, Civil3D, Bentley, E3D, Tekla, SolidWorks). The 4-class taxonomy (Surface/Linear/Point/Body, see §1) determines the picking mode and parameter requirements.

#### Surface Types (面构件)

| Type | Picking Mode | Clicks | Workflow | GenRule |
|------|-------------|--------|----------|---------|
| **Wall** | Polyline path | 2+ | Draw wall centerline in plan view (2+ points) → assign WallType (layer stack) → set height → done | LayerExtrusion |
| **Slab** | Closed boundary polygon | N | Draw closed boundary in plan view (N points, auto-close last→first) → assign FloorType (layer stack) → done | LayerExtrusion |
| **Roof** | Extrusion profile + extent (or footprint) | 2 (extrusion) or N (footprint) | **Extrusion mode**: draw profile sketch + pick start/end extent. **Footprint mode**: draw closed boundary polygon → assign RoofType → done | LayerExtrusion |
| **Floor** | Closed boundary polygon | N | Same as slab: draw closed boundary → assign FloorType → done | LayerExtrusion |

> **Code gap**: All surface types currently use 2-point picking (same as wall path), but slab/roof/floor need N-point closed polygon. See [CODE_DEVIATIONS.md](../status/CODE_DEVIATIONS.md) §3.1.

#### Linear Types (线构件)

| Type | Picking Mode | Clicks | Workflow | GenRule |
|------|-------------|--------|----------|---------|
| **Beam** | Start/end points | 2 | Pick start point → pick end point → assign cross-section profile → done | LinearComponent |
| **Column** | Location point (or base/top) | 1–2 | **Revit-style**: pick location point → assign section → set height. **Tekla-style**: pick base point → pick top point → done | LinearComponent |
| **Pipe** | Start/end (connector-to-connector) | 2 | Pick start connector or point → pick end connector or point → assign pipe spec (diameter, schedule) → done | LinearComponent |
| **Duct** | Start/end (connector-to-connector) | 2 | Same as pipe: start → end → assign duct spec → done | LinearComponent |
| **Road** | Alignment (multi-point corridor) | N (complex) | **Full workflow**: draw horizontal alignment → assign vertical profile → assign assembly (cross-section) → set targets (terrain, width) → done. **Simplified**: 2-point path with default cross-section | LinearComponent |
| **Bridge** | Alignment + substructure | N (complex) | Draw alignment → assign pier/abutment positions → assign deck cross-section → assign bearing types → done | LinearComponent + Assembly |
| **Tunnel** | Alignment + template | N (complex) | Draw alignment → assign tunnel template (lining, ring config) → done | LinearComponent |

> **Code gap**: Road/bridge/tunnel use simplified 2-point creation. Full corridor workflow not implemented. See [CODE_DEVIATIONS.md](../status/CODE_DEVIATIONS.md) §3.2.

#### Point Types (点构件)

| Type | Picking Mode | Clicks | Workflow | GenRule |
|------|-------------|--------|----------|---------|
| **PointCloud** | Placement point | 1 | Pick placement point → reference point cloud file → done | PointCloudGeometry |
| **TerrainModel** | Placement point | 1 | Pick placement point → provide source data (points file, contours, TIN) → done | TerrainModel |
| **FeatureLine** | Polyline | N | Draw 3D polyline (N points, defining breakline) → assign elevation/style → done | CurveGeometry |
| **ContourLine** | Polyline | N | Draw 2D polyline (N points, constant elevation) → assign elevation → done | CurveGeometry |

> **Code gap**: FeatureLine/ContourLine use 1-click placement, but need N-point polyline picking. See [CODE_DEVIATIONS.md](../status/CODE_DEVIATIONS.md) §3.3.

#### Body Types (体构件)

| Type | Picking Mode | Clicks | Workflow | GenRule |
|------|-------------|--------|----------|---------|
| **Tank** | Placement point | 1 | Pick placement point → select catalog component (tank type, dimensions) → done | Primitive / Assembly |
| **Pump** | Placement point | 1 | Pick placement point → select catalog component → done | Primitive / Assembly |
| **Valve** | Placement point (inline) | 1 | Pick placement point on pipe → select valve type → done | Primitive / Assembly |
| **Gear** | Placement point | 1 | Pick placement point → select catalog component → done | Primitive / Assembly |
| **Equipment** | Placement point | 1 | Pick placement point → select catalog component → done | Assembly |
| **Part** | Placement point | 1 | Pick placement point → select catalog component → done | Primitive |
| **Assembly** | Placement point | 1 | Pick placement point → select catalog component → done | Assembly |

> All body types are correctly implemented with 1-click picking. See [CODE_DEVIATIONS.md](../status/CODE_DEVIATIONS.md) §3.4.

---

## 8. Domain Extension Rules

New domain-specific object types must define:

1. Object class name and namespace
2. Parent class in OmniM hierarchy
3. IFC / STEP / GIS / other standard mapping if applicable
4. Required property sets
5. Geometry constraints
6. Relationship rules
7. Lifecycle behavior
8. Import/export mapping rules

---

## 9. Open Questions

1. Should domain-specific validation rules live in plugins or core?
2. How much ISO 15926 semantic mapping should be included for plant design?
3. Should mechanical constraints be represented as first-class relationships?
4. How to reconcile BIM LOD and mechanical CAD precision requirements?
5. Should domain classification codes be pluggable by country or industry?
