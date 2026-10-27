# OmniM Unified Data Model

> **Last updated: 2026-10-27, audited against code** (`proto/omnim.proto`, `proto/omcom.proto`, `crates/model/**`, `crates/sketch/**`, `crates/ifc/**`). Blocks marked *design specification* describe the target model and are not yet in the proto/code; capabilities marked 🗓 Planned are design goals awaiting implementation.

## 1. Overview

> **Related documents**
> - Component Catalog & Series Library: [CATALOG_SERIES.md](CATALOG_SERIES.md)
> - Engineering lifecycle & change flow: [LIFECYCLE.md](LIFECYCLE.md)

OmniM's data model is designed as an **IFC superset** with native extensions for Civil Engineering, Plant Design, and Mechanical Design domains. It provides a single, unified representation for engineering objects, project lifecycle records, documents, schedules, costs, assets, work orders, operational data, and digital twin streams across the entire engineering project lifecycle.

### Design Principles

- **IFC Compatible**: Core classes align with IFC 4x3 where applicable
- **Domain Extensible**: Clean extension mechanism for non-building domains
- **Geometry First**: Rich geometry representation as first-class citizen
- **Relation Rich**: Native support for containment, connection, spatial, lifecycle, contractual, and semantic relationships
- **Lifecycle Native**: Planning, design, procurement, construction, commissioning, asset, operation & maintenance, production, and decommissioning data are first-class model concepts
- **Versioned**: Every object, lifecycle record, and change is versioned for audit
- **Unified geometry, layered semantics**: Building, plant, and civil domains share the same parametric geometry primitives. Domain semantics are mapped to geometry through GenRule (generation rules), rather than being hardcoded in separate code paths.
- **Performant**: Optimized for both storage and query performance

### Three-Layer Object Model

OmniM adopts a **unified geometry kernel + layered semantics** architecture, supporting Revit-style (semantics-driven), PDMS-style (primitive assembly + catalog specification), Civil3D-style (linear component), and AVEVA Instrumentation-style (logical objects) design paradigms on a single geometry foundation:

> **E3D database types** (`docs/research/aveva_e3d_primitives.md §1.3`): AVEVA E3D uses 8 database types that map to OmniM's storage layers:
> | E3D Database | OmniM Equivalent | Purpose |
> |-------------|------------------|---------|
> | MODEL | Project Object Store | 3D model geometry |
> | PADD/DRAW | — | 2D drawing generation |
> | ISOD | — | Isometric drawing generation |
> | SCHEMATIC | LogicalObject Store | P&ID / schematic data |
> | ENGINEERING | Property System | Engineering attributes |
> | CATALOGUE | Library CatalogStore | Component catalog |
> | LEXICON | Property System | UDA/UDET (User Defined Attributes/Types) |
> | PROPERTIES | Property System | Element properties |
>
> OmniM unifies the separate MODEL, SCHEMATIC, and ENGINEERING databases into a single object graph with the LogicalObject ↔ Object cross-view, and the CATALOGUE/LEXICON/PROPERTIES into the Library + Property System.

```
┌──────────────────────────────────────────────────────┐
│         Layer 4: Domain Semantics                      │
│  WallType / PipeSpec / Alignment / AssetRecord        │
│  - Type definitions, layer structures, specs,         │
│    constraints                                        │
│  - Digital twin data binding (IoT streams, work       │
│    orders, alerts)                                    │
└──────────────────────────┬───────────────────────────┘
                           │ instantiated via
┌──────────────────────────▼───────────────────────────┐
│   ╔══════════════════════════════════════════════╗    │
│   ║         Library (共享库 — session 级全局)      ║    │
│   ║  • CatalogStore: ComponentDef definitions    ║    │
│   ║  • ComponentStore: Component templates       ║    │
│   ║  • FeatureTreeRegistry: feature templates    ║    │
│   ║  • Default Template Registry                 ║    │
│   ║  Loaded as global static once per session    ║    │
│   ╚══════════════════════════════════════════════╝    │
│                           │ 被 Series 引用           │
│         Layer 3: Component Catalog & Spec Library     │
│  ComponentDef: parametric 3D model definitions         │
│  SeriesStore (独立模块)  ◄── Project 引用              │
│             only series-loaded components usable      │
│  Connectors: connection points with type codes         │
└──────────────────────────┬───────────────────────────┘
                           │ mapped via GenRule
┌──────────────────────────▼───────────────────────────┐
│         Layer 2: Object Generation (GenRule)          │
│  ObjectTypeDef → GenRule → [primitive list + bool ops]│
│  - 5 GenRule types cover all domains                  │
│  - VariationRule drives variable-section linear       │
│    components                                         │
└──────────────────────────┬───────────────────────────┘
                           │ outputs
┌──────────────────────────▼───────────────────────────┐
│         Layer 1: Unified Geometry Kernel              │
│  Box / Cylinder / Cone / Sphere / Extrusion           │
│  Sweep / Revolve / Loft                               │
│  + Union / Subtract / Intersect (Boolean ops)         │
│  + Transform (Origin + Orientation)                   │
│       ↓                                               │
│  BRep solid → triangle mesh → render output           │
└──────────────────────────────────────────────────────┘
```

**Core idea**: A building wall and a plant pipe are both Extrusions/Cylinders at the geometry layer; the difference lies only in GenRule complexity — a wall's GenRule expands "layer structure" into multiple Extrusions, while a pipe's GenRule passes parameters through almost directly.

### Component Classification: Surface / Linear / Point / Body

Across all four engineering domains, component geometry is conceptually classified into four categories based on dimensionality. This classification is the primary conceptual taxonomy — it describes *what kind* of geometry each component has, while the GenRule layer (see §6) describes *how* that geometry is generated:

| Class | Chinese | Typical Objects | GenRule Mapping | Description |
|-------|---------|----------------|-----------------|-------------|
| **Surface** | 面构件 | Wall, Slab, Roof, Floor, GradingSurface | LayerExtrusion (or TIN) | 2D area element with thickness/material layers; defined by a boundary (rectangle or sketch) with a layered material stack applied orthogonally as thickness. The primary geometry is the **surface** itself, not a centerline path. |
| **Linear** | 线构件 | Beam, Column, Pipe, Road, Duct, CableTray | LinearComponent | 1D path element with cross-section profile; defined by an alignment (path) with a section that varies along the path. The primary geometry is the **path**, not the surface. |
| **Point** | 点构件 | PointCloud, TerrainModel (vertices/breaklines), FeatureLine, ContourLine | PointCloudGeometry / TerrainModel (non-GenRule) | 0D point-defined element; the geometry is the point set itself — no sweep, no loft, no boolean. Terrain model TIN surfaces are constructed from points (vertices + breaklines). |
| **Body** | 体构件 | Tank, Pump, Valve, Gear, Equipment, StructuralSteel | Primitive / Assembly | 3D solid volume; defined by direct primitive operations (box, cylinder, cone, sphere) or assembly of multiple primitives. |

**Wall is a Surface component**: unlike a beam (linear) or a tank (body), a wall defines a 2D surface panel — its length and height form a rectangle (or an arbitrary shape drawn via sketch lines) with a layered material stack applied as thickness. This is generated via LayerExtrusion (§6.2): the surface boundary provides the extrusion profile, and the layer stack provides the thickness. The wall is **not** a LinearComponent path sweep — its primary geometry is the **surface**, not a centerline path. See §6.4 for the distinction.

> **Cross-platform reference**: The 4-class taxonomy is derived from the cross-platform analysis (`docs/research/comparative_analysis.md §2-4`): Revit Wall/Floor/Roof (surface), Revit Beam/Column + Civil 3D Corridor (linear), point clouds + TIN surfaces (point), E3D primitives + SolidWorks parts (body).

**Library vs Project**: The **Library** is a **session-level global singleton** holding the master Component Catalog (`CatalogStore`), Component templates (`ComponentStore`), and feature tree templates (`FeatureTreeRegistry`). It is loaded once per session and shared across all projects. The **SeriesStore** is an **independent module** — it defines spec decision trees that reference Library content by ComponentDef ID. The **Project** holds a private `SeriesStore` (project-specific series) and `external_series_refs` (references to external series). Project does **not** hold a Library reference. See [§14](#14-component-catalog-元件库-paragon) and [§15](#15-series-library-系列库--等级库--specon).

---

## 2. Core Object Hierarchy

```
OmniObject (base for all objects)
├── SpatialElement
│   ├── Site
│   ├── Building
│   ├── BuildingStorey
│   └── Space
├── Element (IFC-aligned base for all physical elements)
│   ├── BuildingElement (IFC compatible building domain)
│   │   ├── Wall, Beam, Column, Slab
│   │   ├── Door, Window, Stair, Roof
│   │   ├── MEP elements (Duct, Pipe, etc.)
│   │   └── Furniture, Fixtures, Equipment
│   ├── CivilElement (Civil Engineering extension)
│   │   ├── Road, Bridge, Tunnel, Dam
│   │   ├── Culvert, RetainingWall
│   │   ├── Alignment, Profile, CrossSection
│   │   ├── TerrainModel (TIN surface, independent type)
│   │   ├── CutVolume, FillVolume (BooleanSubtract surface-surface)
│   │   ├── GradingSurface (design terrain)
│   │   ├── Slope (LinearComponent + ExternalTarget)
│   │   ├── FeatureLine, ContourLine
│   │   ├── SiteComponent (terrain-attached fixtures)
│   │   └── EarthworkBalance (analysis result)
│   ├── PlantElement (3D Plant Design extension)
│   │   ├── Tank, Pump, Valve
│   │   ├── HeatExchanger, Reactor, Vessel
│   │   ├── Pipe, Nozzle, Flange
│   │   └── StructuralSteel, Platform, Ladder
│   └── MechanicalElement (Mechanical Design extension)
│       ├── Part, Assembly, Fastener
│       ├── Gear, Shaft, Bearing
│       ├── Motor, PumpBody, Housing
│       └── Spring, Seal, Bushing
├── System
│   ├── BuildingSystem
│   ├── CivilSystem
│   ├── PlantSystem
│   └── MechanicalSystem
└── ProjectLifecycleRecord
    (typed via free-form `record_type` and `phase` fields,
     not fixed subclasses — see §4 for details)
```
> **Site geographic properties** (from Revit IfcSite + Civil 3D survey, `docs/research/site_terrain_components.md §1.1`): Site is a spatial container with no geometry of its own. Its physical extent is defined by:
> - `name` — site name
> - `address` — geographic address
> - `latitude / longitude` — geographic coordinates (WGS84)
> - `site_rotation` — orientation relative to true north
> - `project_base_point` — survey coordinate origin (Easting, Northing, Elevation)
> - `survey_point` — real-world coordinate reference
> - `boundary` — optional boundary polygon defining the site extent
> - `geo_ref` — `GeographicReference` (coordinate system / projection / EPSG code)
>
> Spatial hierarchy follows IFC 4x3: `Site → Building → BuildingStorey → Space`. Civil and Plant elements also belong under Site, organized by system type rather than spatial storey.
>
> **Multibody Part support** (from SOLIDWORKS, `docs/research/solidworks_components.md §10`): A `Part` can contain multiple disjoint solid bodies. Operations include: Combine (Add/Subtract/Intersect) between bodies, Feature Scope (select which bodies a feature affects), Move/Copy Body, per-body materials, and Exploded Views. OmniM supports this **by design** (🗓 Planned): a Part's feature tree would generate multiple independent bodies, with the `Boolean` feature operation controlling inter-body combination. (The current Feature Engine produces a single merged mesh per component.)

> **Implementation status of this hierarchy**: The `ObjectType` enum in `proto/omnim.proto` currently implements — Spatial: Site, Building, BuildingStorey, Space; Building: Wall, Beam, Column, Slab, Door, Window, Stair, Roof, Floor, plus MEP (Duct, CableTray, AirTerminal, Ahu, Chiller, Transformer, Luminaire); Civil: Road, Bridge, Tunnel, Dam, Culvert, RetainingWall, PipeCivil, Pier, Manhole, Abutment; Plant: Tank, Pump, Valve, HeatExchanger, Reactor, PipePlant, StructuralSteel, Fitting, Compressor, Instrument; Mechanical: Part, Assembly, Fastener, Gear, Shaft, Bearing, Motor, Spring, Seal; plus Component, FeatureLine, ContourLine, Generic. The remaining branches shown above (Nozzle, Flange, Platform, Ladder, Vessel, PumpBody, Housing, Bushing; Alignment, Profile, CrossSection, TerrainModel, CutVolume, FillVolume, GradingSurface, Slope, SiteComponent, EarthworkBalance; the System classes) are conceptual design targets — 🗓 Planned to be added to the enum.

---

## 3. Base Class: OmniObject

All objects in OmniM are built on the `OmniObject` base, which carries common identity and lifecycle fields. The message below is the **design specification**; no `OmniObject` protobuf message exists yet — the implemented type is the serde-JSON `OmniObject` struct in `crates/model/src/object.rs`.

```protobuf
message OmniObject {
  // Identity
  string id = 1;                    // UUID v4
  string name = 2;                  // Human readable name
  string description = 3;           // Optional description
  ObjectType type = 4;              // Object classification
  
  // Geometry
  Geometry geometry = 5;            // Primary geometry representation
  Transform transform = 6;          // World space transform
  
  // Properties
  repeated Property properties = 7; // Dynamic property bag
  repeated PropertySet property_sets = 8; // Grouped IFC-style property sets
  
  // Classification
  string ifc_class = 10;            // Corresponding IFC class if applicable
  repeated string tags = 11;        // Flexible tagging
  string material = 12;             // Material reference
  
  // Lifecycle
  ObjectStatus status = 13;
  google.protobuf.Timestamp created_at = 14;
  string created_by = 15;
  google.protobuf.Timestamp modified_at = 16;
  string modified_by = 17;
  
  // Versioning
  string version_id = 18;
  string previous_version_id = 19;
  
  // Relationships are stored separately in the relation index

  // Catalog reference
  optional CatalogRef catalog_ref = 20;  // Link to catalog component definition

  // Administrative Division & Project Breakdown
  optional string ad_id = 21;            // Administrative Division (parcel or higher)
  repeated string pbs_ids = 22;          // PBS nodes this object belongs to (typically 1)

  // Coordinate System
  string crs_id = 23;                    // Which CRS this object's transform is expressed in (typically project_crs)
}

enum ObjectStatus {
  OBJECT_STATUS_UNSPECIFIED = 0;
  DRAFT = 1;
  PROPOSED = 2;
  APPROVED = 3;
  CONSTRUCTION = 4;
  AS_BUILT = 5;
  OPERATIONAL = 6;
  DECOMMISSIONED = 7;
}
```

> **Implemented today** (Rust struct, `crates/model/src/object.rs`): `id`, `name`, `description`, `object_type`, `status`, `properties`, `property_sets`, `geometry`, `material_id` (optional; shown above as `material`), `parent_id` (structural tree parent), `tags`, `ifc_class`, plus modeling fields `features`, `sketch_ids`, `dirty`, `catalog_ref`. The `ObjectStatus` values above match `proto/omnim.proto` exactly.
>
> **🗓 Planned fields** (not yet in the struct or proto): `transform` (per-object world transform; transforms are currently applied at API level via `PUT /api/objects/{id}/transform`), `created_at`/`created_by`, `modified_at`/`modified_by`, `version_id`, `previous_version_id` (version/audit records live in §10, itself still a design spec), `ad_id` (§18), `pbs_ids` (§19), `crs_id` (§21).

---

## 4. Project Lifecycle Model

OmniM treats project lifecycle data as first-class model entities, not as external project management metadata. The engineering project lifecycle is divided into nine phases, and lifecycle records are organized by phase. Each record can reference engineering objects, geometry, documents, users, organizations, schedules, costs, and external systems.

```
Planning → Design → Procurement → Construction → Commissioning & Acceptance → Asset → Operation & Maintenance → Production → Decommissioning
```

```protobuf
message ProjectLifecycleRecord {
  string id = 1;
  LifecyclePhase phase = 2;          // Which lifecycle phase this record belongs to
  string record_type = 3;            // Free-form type tag (e.g. "schedule", "cost", "delivery", "inspection", "work_order")
  string name = 4;
  string description = 5;
  repeated string related_object_ids = 6;
  repeated string related_document_ids = 7;
  repeated Property properties = 8;
  RecordStatus status = 9;
  google.protobuf.Timestamp planned_start = 10;
  google.protobuf.Timestamp planned_finish = 11;
  google.protobuf.Timestamp actual_start = 12;
  google.protobuf.Timestamp actual_finish = 13;
  double planned_cost = 14;
  double actual_cost = 15;
  string responsible_party = 16;
  string version_id = 17;
}

// 工程项目生命周期阶段 (9 phases, lifecycle closed loop)
// 规划 → 设计 → 采购 → 施工 → 调试验收 → 资产 → 运维 → 生产 → 退役拆除
enum LifecyclePhase {
  LIFECYCLE_PHASE_UNSPECIFIED = 0;
  PLANNING = 1;                  // 规划: site selection, feasibility, zoning, project requirements
  DESIGN = 2;                    // 设计: BIM/CAD models, drawings, specifications, calculations
  PROCUREMENT = 3;               // 采购: bill of materials, suppliers, contracts, delivery tracking
  CONSTRUCTION = 4;              // 施工: schedule, progress, quality, safety, as-built
  COMMISSIONING_ACCEPTANCE = 5;  // 调试验收: testing, inspection, acceptance, handover
  ASSET = 6;                     // 资产: asset registry, warranty, handover to owner
  OPERATION_MAINTENANCE = 7;     // 运维: operation, maintenance, work orders, energy management
  PRODUCTION = 8;                // 生产: actual production operation (plant/manufacturing)
  DECOMMISSIONING = 9;           // 退役拆除: decommissioning, demolition, disposal, end of life
}

enum RecordStatus {
  RECORD_STATUS_UNSPECIFIED = 0;
  DRAFT = 1;
  OPEN = 2;
  IN_PROGRESS = 3;
  BLOCKED = 4;
  COMPLETED = 5;
  APPROVED = 6;
  REJECTED = 7;
  CLOSED = 8;
}
```

### Lifecycle Relationships

Lifecycle records are connected to engineering objects through phase-scoped relations. A record carries a `phase` and a free-form `record_type`, and can be linked to any engineering object with a `link_role` describing how the object relates to the record (e.g. "scope", "target", "reference").

| Phase | Example Records | Example Linked Objects |
|-------|-----------------|------------------------|
| Planning | Feasibility study, site constraint, zoning requirement | Site, terrain, utility alignments |
| Design | Design package, drawing, calculation, design change | Walls, beams, columns, pipes, equipment |
| Procurement | Bill of materials, contract, delivery, supplier data | Valves, pumps, steel members, materials |
| Construction | Construction activity, schedule task, inspection, issue | Physical elements under installation |
| Commissioning & Acceptance | Test record, acceptance report, handover package | Equipment, systems, as-built elements |
| Asset | Asset registry entry, warranty record | Handover assets, equipment, components |
| Operation & Maintenance | Work order, maintenance log, energy record | Operational equipment, systems, spaces |
| Production | Production batch, throughput record, downtime log | Production lines, reactors, vessels |
| Decommissioning | Decommissioning plan, demolition record, disposal log | Retired assets, structures to be removed |

---

## 5. Geometry Representation

OmniM supports multiple geometry representations for different use cases. The geometry pipeline is:

```
ComponentDef (catalog) → Instantiated → Sketch (2D constrained) → Feature (first-class entity with cached Geometry) → BRep (precise) → Mesh (visualization)
```

The key insight: **Feature is a first-class 3D entity**, not just a parameter container. Each Feature has its own cached MeshGeometry, is marked dirty when inputs change, and participates in the rebuild pipeline. Sketch is a basic 2D entity within a Component — it provides the profile input for Feature operations.

### 5.1 Geometry Types

```protobuf
message Geometry {
  string id = 1;
  GeometryType type = 2;
  
  oneof representation {
    ParametricGeometry parametric = 3;
    BRepGeometry brep = 4;
    MeshGeometry mesh = 5;
    PointCloudGeometry point_cloud = 6;
    CurveGeometry curve = 7;
    ExternalGeometryReference external = 8;
  }
  
  BoundingBox bbox = 9;
  uint32 level_of_detail = 10;  // LOD 100-500 scale
}

enum GeometryType {
  GEOMETRY_TYPE_UNSPECIFIED = 0;
  PARAMETRIC = 1;    // Parametric primitives and operations
  BREP = 2;          // Boundary representation (precise)
  MESH = 3;          // Polygon mesh (visualization)
  POINT_CLOUD = 4;   // Scan data
  CURVE = 5;         // 2D/3D curves and lines
  EXTERNAL = 6;      // Reference to external file
}
```

> **Implementation status**: The `Geometry` message and the 6-value `GeometryType` enum above are in `proto/omnim.proto`, but the representation `oneof` currently wires only `parametric`, `mesh`, and `external`. The `brep`, `point_cloud`, and `curve` representation arms are design specification — 🗓 Planned (the BRep engine exists in `crates/model/src/brep.rs` via truck-modeling, but BRep results are tessellated to meshes rather than stored in a `brep` representation). `PointCloudGeometry` and `CurveGeometry` messages are not yet defined.

### 5.2 Parametric Geometry

```protobuf
message ParametricGeometry {
  repeated ParametricOperation operations = 1;
}

message ParametricOperation {
  oneof operation {
    Box box = 1;
    Cylinder cylinder = 2;
    Sphere sphere = 3;
    Cone cone = 4;
    Extrusion extrusion = 5;
    Revolve revolve = 6;
    Sweep sweep = 7;
    Loft loft = 8;
    Torus torus = 9;      // elbow segments (tanks / pipes)
    Dish dish = 10;       // vessel / tank heads
  }
}
```

> **Primitives now implemented** (`proto/omnim.proto`): all 10 operations above exist, including `Torus` (elbow segments) and `Dish` (vessel/tank heads) — see `docs/research/aveva_e3d_primitives.md` for the mapping. `BooleanOperation` and `Transform` are deliberately **not** `ParametricOperation` members: boolean composition lives at the Feature layer (`BooleanFeature` / `BooleanOp`), and adding boolean ops to `ParametricOperation` remains a §6.7 future enhancement.

### 5.3 Boundary Representation (BRep)

> **Design specification (🗓 Planned)**: the messages below are the target schema for a serialized BRep; they are not yet in `proto/omnim.proto`. The shipped BRep engine (`crates/model/src/brep.rs`, built on truck-modeling) constructs `Solid`/`Shell`/`Wire` in memory and tessellates them to `MeshGeometry`, but does not yet store or serialize this message form.

```protobuf
message BRepGeometry {
  repeated Vertex vertices = 1;
  repeated Edge edges = 2;
  repeated Face faces = 3;
  repeated Shell shells = 4;
  repeated Solid solids = 5;
}

message Vertex {
  Point3D point = 1;
}

message Edge {
  int32 start_vertex = 1;
  int32 end_vertex = 2;
  Curve curve = 3;  // Optional curve geometry for edge
}

message Face {
  repeated int32 edge_indices = 1;
  Surface surface = 2;  // Optional surface geometry
}
```

### 5.5 Sketch Geometry (2D Constrained Sketch)

Sketches are the foundation of parametric modeling. A sketch lives on a plane and contains 2D entities constrained by geometric constraints.

> **Implementation status**: The shipped sketch system is `omnim_sketch` (`crates/sketch/`, persisted in `Project.sketches`, surfaced via `SketchData` in `proto/omcom.proto`). Implemented entity types: `Point, Line, Circle, Arc, Rectangle, Polygon, Ellipse, Spline, Polyline` (each entity stores control points, no typed sub-messages). Implemented constraint types (19): `Horizontal, Vertical, Coincident, Parallel, Perpendicular, Length, Distance, Angle, Radius, Diameter, Tangent, Symmetric, Fixed, AtMidpoint, EqualLength, EqualRadius, PtOnLine, PtOnCircle, SameOrientation`. The workplane is carried as `origin` + `normal` fields, not a separate `SketchPlane` message. Everything below that goes beyond this set (typed sub-messages, Slot/Conic/Parabola/SketchText/BeltChain/EquationCurve, the 3D sketch constraints) is a SOLIDWORKS-derived design specification — 🗓 Planned.

```protobuf
message Sketch {
  string id = 1;
  string name = 2;
  SketchPlane plane = 3;
  repeated SketchEntity entities = 4;
  repeated Constraint constraints = 5;
  bool solved = 6;
}

message SketchPlane {
  Point3D origin = 1;
  Vector3D normal = 2;
  Vector3D ref_direction = 3;  // X-axis reference direction
}

message SketchEntity {
  string id = 1;
  oneof entity {
    SketchLine line = 2;
    SketchArc arc = 3;
    SketchCircle circle = 4;
    SketchPoint point = 5;
    SketchSpline spline = 6;
    SketchSlot slot = 7;
    SketchConic conic = 8;
    SketchParabola parabola = 9;
    SketchText text = 10;
    BeltChain belt_chain = 11;
    EquationCurve equation_curve = 12;
  }
}

message SketchLine {
  double start_x = 1; double start_y = 2;
  double end_x = 3;   double end_y = 4;
}

message SketchArc {
  double center_x = 1; double center_y = 2;
  double radius = 3;
  double start_angle = 4; double end_angle = 5;
  bool is_ccw = 6;
}

message SketchCircle {
  double center_x = 1; double center_y = 2;
  double radius = 3;
}

// Slot 3 variants (SOLIDWORKS §4.0): Straight / Arc / Conical
message SketchSlot {
  SlotType type = 1;          // STRAIGHT / ARC / CONICAL
  double center_x = 2; double center_y = 3;
  double width = 4;           // slot width
  double length = 5;          // slot length (straight)
  optional double radius = 6; // arc/conical radius
  optional double start_angle = 7;
  optional double end_angle = 8;
  bool is_ccw = 9;
}

enum SlotType {
  SLOT_STRAIGHT = 0;
  SLOT_ARC = 1;
  SLOT_CONICAL = 2;
}

// Conic driven by endpoints + Rho value (SOLIDWORKS §4.0)
message SketchConic {
  double start_x = 1; double start_y = 2;
  double end_x = 3;   double end_y = 4;
  double rho = 5;          // Rho value: <0.5 elliptical, =0.5 parabolic, >0.5 hyperbolic
}

message SketchParabola {
  double vertex_x = 1; double vertex_y = 2;
  double focus_offset_x = 3; double focus_offset_y = 4;
  double start_x = 5; double start_y = 6;
  double end_x = 7;   double end_y = 8;
}

// SketchText: 2D text on a curve/edge, extrudable or cuttable into 3D (SOLIDWORKS §4.0)
message SketchText {
  string text = 1;
  double pos_x = 2; double pos_y = 3;
  double height = 4;
  optional string font = 5;
  optional string curve_entity_id = 6;  // curve/edge to follow
  bool reverse_direction = 7;
  bool extrude = 8;   // true = extrudable/cuttable 3D text
}

// BeltChain: layout sketch for pulleys / sprocket mechanisms (SOLIDWORKS §4.0)
message BeltChain {
  repeated string pulley_entity_ids = 1;  // circles/arcs (pulleys or sprockets)
  double belt_offset = 2;                 // belt inside/outside offset
  bool tangent_lines = 3;
}

// EquationCurve: parametric / equation-driven curve (SOLIDWORKS §4.0)
message EquationCurve {
  string x_formula = 1;   // parametric x(t)
  string y_formula = 2;   // parametric y(t)
  double t_start = 3; double t_end = 4;
  int32 samples = 5;
}

message Constraint {
  string id = 1;
  ConstraintType type = 2;
  repeated string entity_ids = 3;    // constrained entities
  optional double value = 4;         // parameter value (distance, angle, radius)
  optional string reference_entity_id = 5;  // reference entity
  optional int32 point_index_a = 6;  // point index on entity a
  optional int32 point_index_b = 7;  // point index on entity b
}

enum ConstraintType {
  HORIZONTAL = 0;
  VERTICAL = 1;
  FIXED = 2;
  COINCIDENT = 3;
  PARALLEL = 4;
  PERPENDICULAR = 5;
  TANGENT = 6;
  EQUAL_LENGTH = 7;
  EQUAL_RADIUS = 8;
  COLLINEAR = 9;
  MIDPOINT = 10;
  SYMMETRIC = 11;
  DISTANCE = 12;
  ANGLE = 13;
  RADIUS = 14;
  LENGTH = 15;
  DIAMETER = 16;
  OFFSET = 17;

  // 3D sketch constraints (SOLIDWORKS §4.1)
  CORADIAL = 18;           // arcs/circles share same center and radius
  EQUAL_CURVATURE = 19;    // curvature continuity at curve junction
  EQUAL_CURVE_LENGTH = 20; // curve length equality
  MERGE_POINTS = 21;       // merge two endpoints
  PIERCE = 22;             // curve pierces plane/surface
  FIX_SLOT = 23;           // fix slot geometry
  ON_EDGE = 24;            // lie on model edge
  ON_PLANE = 25;           // lie on reference plane
  ON_SURFACE = 26;         // lie on model surface
  TANGENT_TO_FACE = 27;    // tangent to face
  TRACTION = 28;           // traction constraint (contact)
  TORSION_CONTINUITY = 29; // G3 torsion continuity
}

> **Planned sketch entities** (from SOLIDWORKS, `docs/research/solidworks_components.md §4.0`): `Slot` (3 variants: Straight/Arc/Conical), `Conic` (endpoints + Rho value — elliptical/parabolic/hyperbolic based on Rho), `Parabola`, `SketchText` (extrudable/cuttable 3D text), `BeltChain` (pulley/sprocket layout sketch), `EquationCurve` (parametric equation-driven curve). Designed as additions to the `SketchEntity` oneof above — 🗓 Planned, not yet in proto/omcom.proto or `omnim_sketch`.

> **Planned 3D sketch constraints** (from SOLIDWORKS, `docs/research/solidworks_components.md §4.1`): `Collinear` (=9 above), `Coradial`, `EqualCurvature`, `EqualCurveLength`, `MergePoints`, `Pierce`, `FixSlot`, `OnEdge`, `OnPlane`, `OnSurface`, `TangentToFace`, `Traction`, `TorsionContinuity` — essential for 3D sketch mechanical design. Designed as additions to the `ConstraintType` enum above — 🗓 Planned, not yet in proto/omcom.proto or `omnim_sketch`.

### 5.6 Component Container

Components are container nodes in the project tree that group multiple Features into a logical 3D body. A Component is the primary node users interact with in the project tree (e.g., "Beam-001", "Column-A", "Wall-S1").

A Component holds an ordered list of Features (construction history). Each Feature is a **first-class entity** with its own cached Geometry. The Component's final geometry is the result of merging all its Features in order.

```protobuf
message Component {
  string id = 1;
  string name = 2;
  repeated string feature_ids = 3;   // ordered list of Features
  repeated string sketch_ids = 4;    // sketches owned by this Component
  string parent_id = 5;              // tree parent (Component or Project)
  Transform transform = 6;           // local placement
  MeshGeometry merged_geometry = 7;  // result of merging all Feature geometries
}
```

Key relationships:
- A **Component** contains multiple **Features** (construction history)
- Each **Feature** references one or more **Sketches** for its 2D profile(s)
- Features within a Component form a dependency-ordered chain
- Each **Feature** has its own cached **Geometry** (MeshGeometry)
- The Component's `merged_geometry` is computed by the rebuild pipeline

Project tree hierarchy: `Project → Component → Feature → Sketch`

```
Component: "Beam-001"
  ├── Feature 1: Extrude (sketch: beam_profile, depth=6.0)  → geometry: mesh
  ├── Feature 2: Boolean (subtract: feature1, feature3)      → geometry: mesh
  └── Feature 3: Extrude (sketch: hole_profile, depth=0.3)   → geometry: mesh
```

### 5.7 Feature Modeling (3D Operations from Sketch)

Features are **first-class 3D entities** that live inside a Component. Each Feature:
1. References one or more **Sketches** for its 2D profile(s)
2. Has its own **cached Geometry** (MeshGeometry) computed by the rebuild pipeline
3. Is marked **dirty** when its inputs change, triggering automatic rebuild
4. Forms a dependency chain with other Features in the same Component

**Sketch is a basic entity within a Component.** A Sketch is a 2D constrained profile (lines, circles, arcs) that serves as input to a Feature. Sketches are owned by the Component and can be referenced by multiple Features.

```protobuf
message Feature {
  // Design specification. The implemented Feature (crates/model/src/feature.rs)
  // is a flat typed struct, not an operation oneof:
  string id = 1;
  string name = 2;
  string component_id = 3;               // parent Component
  FeatureType type = 4;
  repeated string dependency_ids = 5;    // parent features in same Component
  string sketch_id = 6;                  // primary source sketch (profile)
  repeated string input_sketch_ids = 7;  // additional sketches (Sweep path, Loft sections)
  repeated FeatureParameter parameters = 8;
  bool suppressed = 9;                   // temporarily disabled
  int32 order = 10;                      // position in component feature list

  // Cached geometry — computed by rebuild pipeline, not set by user
  MeshGeometry cached_geometry = 20;
  bool dirty = 21;                       // true when inputs changed, needs rebuild

  oneof operation {
    ExtrudeFeature extrude = 30;
    RevolveFeature revolve = 31;
    SweepFeature sweep = 32;
    LoftFeature loft = 33;
    ChamferFeature chamfer = 34;
    FilletFeature fillet = 35;
    BooleanFeature boolean = 36;
    PatternFeature pattern = 37;
    ShellFeature shell = 38;
  }
}

enum FeatureType {
  FEATURE_TYPE_UNSPECIFIED = 0;
  EXTRUDE = 1;          // Extrude profile along direction
  REVOLVE = 2;          // Revolve profile around axis
  SWEEP = 3;            // Sweep profile along path curve
  LOFT = 4;             // Loft between multiple cross-sections
  BOOLEAN = 5;          // Boolean operation (cut/union/intersect)
  CHAMFER = 6;          // Chamfer edge
  FILLET = 7;           // Fillet edge
  PATTERN = 8;          // Linear/circular pattern
  SHELL = 9;            // Shell (hollow out)
  HOLE = 10;            // Simple hole / Hole Wizard (7 subtypes)
  WELD = 11;            // Weld bead feature (Tekla)
  SHEET_METAL = 12;     // Sheet metal bend/unfold (SOLIDWORKS)
  BOLT_GROUP = 13;      // Bolt array/circle/XY list (Tekla)
  REFERENCE_GEOMETRY = 14; // Plane/Axis/CSys/Point/BoundingBox/GridSystem (SOLIDWORKS)
}

> **Implementation status**: the shipped `FeatureType` (Rust, `crates/model/src/feature.rs`; mirrored as `FeatureType` in `proto/omcom.proto`) implements exactly: Extrude, Revolve, Sweep, Loft, Fusion, LoftedFusion, Boolean, Chamfer, Fillet, Pattern, Shell (`Fusion`/`LoftedFusion` are implemented extras not listed above). `HOLE`, `WELD`, `SHEET_METAL`, `BOLT_GROUP`, `REFERENCE_GEOMETRY` are 🗓 Planned — the research callouts below describe their target parameterization. The implemented boolean feature uses `BooleanOp { Union, Subtract, Intersect }` on a flat struct rather than the `BooleanFeature` message above.

> **Hole Wizard subtypes** (from SOLIDWORKS, `docs/research/solidworks_components.md §3.5`): 7 hole types: Counterbore, Countersink, Hole, Straight Tap, Tapered Tap, Legacy, Slots (3 slot variants: Straight/Arc/Conical).

> **BoltGroup subtypes** (from Tekla, `docs/research/tekla_components.md §7`): BoltArray (rectangular grid), BoltCircle (circular), BoltXYList (arbitrary positions).

> **BoltGroup HoleType** (from Tekla, `docs/research/tekla_components.md §8.5`): `HoleType` variants — `STANDARD`, `OVERSIZED`, `SLOTTED` (with XY slot dimensions), `BLIND` (with depth). The `BoltHolesAttributes` object controls hole tolerances and thread preferences.

> **Weld feature parameters** (from Tekla, `docs/research/tekla_components.md §8.6`): `SizeAbove`/`SizeBelow` (weld leg size), `AngleAbove`/`AngleBelow` (groove angle), `AroundWeld` (all-around vs side weld), `ContourAbove`/`ContourBelow` (flat/convex/concave), `Shape` (continuous/intermittent/staggered), `Workshop`/`Site` (fabrication location). Weld geometry is a sweep along the weld path; `PolygonWeld` variant follows a picked polygon path.

> **Sheet Metal features** (from SOLIDWORKS, `docs/research/solidworks_components.md §12`): Base Flange, Miter Flange, Edge Flange, Swept Flange, Hem, Jog, Louver, Forming Tool, Break Corner. The Feature Engine supports unfold/fold for flat pattern generation.

> **Fillet sub-types** (from SOLIDWORKS, `docs/research/solidworks_components.md §3.3`): 8 sub-types — `Constant Size`, `Multiple Radius`, `Round Corner`, `Setback`, `Variable Size`, `Face`, `Full Round`, `Asymmetric`. Each requires a different parameter schema (e.g., Setback uses setback distances at vertices; Variable Size uses control points along edge). The current `FilletFeature` (edge_ref + radius) is sufficient for Constant Size only.

> **Pattern sub-types** (from SOLIDWORKS, `docs/research/solidworks_components.md §3.4`): Missing sub-types — `Curve-Driven`, `Fill`, `Sketch-Driven`, `Table-Driven`, `Variable`. The current `PatternFeature` (PatternType + count + spacing) supports only Linear/ Rectangular/Circular. `Curve-Driven` requires a reference curve; `Fill` fills a boundary region; `Sketch-Driven` distributes pattern points from a sketch; `Table-Driven` uses (x,y,z) coordinate table; `Variable` applies per-instance scale/rotation.

message ExtrudeFeature {
  double depth = 1;
  bool is_symmetric = 2;       // extrude in both directions
  double draft_angle = 3;      // taper angle
  ExtrudeEndCondition end_condition = 4;
}

enum ExtrudeEndCondition {
  BLIND = 0;
  THROUGH_ALL = 1;
  TO_NEXT = 2;
  TO_SURFACE = 3;
}

message RevolveFeature {
  double angle = 1;            // radians, 0 = full 360
  Point2D axis_start = 2;
  Point2D axis_end = 3;
}

message SweepFeature {
  string path_sketch_id = 1;   // references input_sketch_ids[0]
  bool is_solid = 2;
}

message LoftFeature {
  repeated string sketch_ids = 1;  // references input_sketch_ids
  bool is_solid = 2;
  double scale_start = 3;          // optional scaling at start section
  double scale_end = 4;            // optional scaling at end section
}

message ChamferFeature {
  string edge_ref = 1;
  double distance = 2;
}

message FilletFeature {
  string edge_ref = 1;
  double radius = 2;
}

message PatternFeature {
  PatternType pattern_type = 1;
  int32 count_x = 2; int32 count_y = 3; int32 count_z = 4;
  double spacing_x = 5; double spacing_y = 6; double spacing_z = 7;
}

enum PatternType {
  PATTERN_LINEAR = 0;
  PATTERN_RECTANGULAR = 1;
  PATTERN_CIRCULAR = 2;
}

message BooleanFeature {
  BooleanOperationType type = 1;
  repeated string target_feature_ids = 2;  // features to operate on (by order)
}

enum BooleanOperationType {
  BOOLEAN_UNSPECIFIED = 0;
  UNION = 1;           // merge two solids
  SUBTRACTION = 2;     // cut (hollow out, open holes)
  INTERSECTION = 3;    // keep only intersection
}
```

**Feature as First-Class Entity:**

Each Feature is a first-class entity with its own Geometry. This is the key architectural distinction from a pure parameter container:

| Aspect | Parameter Container (old) | First-Class Entity (new) |
|--------|--------------------------|-------------------------|
| Geometry | Computed on-the-fly, not stored | Cached in `cached_geometry` |
| Rebuild | Manual or not supported | Automatic on input change (`dirty` flag) |
| References | Sketch only | Sketch + other Features (dependency_ids) |
| Boolean ops | `is_cut` flag on each feature | Separate BooleanFeature with target references |

**Rebuild Pipeline:**

When a Feature's inputs change (Sketch modified, upstream Feature rebuilt), the Feature is marked `dirty`. The rebuild pipeline processes all dirty Features in order:

```
1. For each Feature in order (0..n):
   a. If Feature is dirty or any dependency is dirty:
      - Read input Sketch(es) from SketchStore
      - Read dependency Feature geometries
      - Compute new MeshGeometry based on FeatureType
      - Store in Feature.cached_geometry
      - Mark Feature as clean (dirty = false)
   b. If Feature is clean, skip

2. After all Features are rebuilt:
   - Merge all Feature geometries into Component.merged_geometry
   - Update bounding box
```

Construction History (Feature Tree):

```
Feature Tree:
  Extrude (wall_s1, depth=0.2)              → geometry: solid block
    └─ Boolean SUBTRACT (hole_s1, target=0) → geometry: block with hole
         └─ Fillet (edges, r=0.05)          → geometry: filleted block

Operations:
  - add_feature → appends to tree, marks dirty, triggers rebuild
  - edit_feature(id, params) → marks dirty, rebuilds all downstream
  - suppress_feature(id) → temporary disable, rebuilds downstream
  - rollback_to(id) → set cursor, hide later features
```

### 5.7.1 Feature Panel UI (Revit-style)

The Component Editor uses a **Feature Panel** on the left side, similar to Revit's Properties panel. This is the primary interface for managing features within a component.

**Layout (Revit-style workflow):**

```
┌──────────────────────────────────────────────────────────────┐
│ [← Back] Component Editor  [Sketch ▾] [+ Sketch]            │
├───────────┬──────────────────────────────────────────────────┤
│ Feature   │  Row1: [Select][Line][Circle][Arc]              │
│ Panel     │       [H][V][∥][⊥][L=][D=][∠][⚓]              │
│ (左边栏)   │       [Solve] [Del]                             │
│           │  Row2: [⬍Ext][↻Rev][⟿Swp][ℳLft][⊖Bool]...     │
│ ≡ Ext-1  │──────────────────────────────────────────────────│
│   sk: sk-0│  ┌────────────────────────────────────────┐     │
│   d: 1.0  │  │                                        │     │
│   [✏️][🗑️]│  │        2D Canvas / 3D Preview          │     │
│           │  │                                        │     │
│ ≡ Rev-1  │  └────────────────────────────────────────┘     │
│   sk: sk-1│──────────────────────────────────────────────────│
│   a: 2π   │  Status: Solved · Entities: 5 · 2D             │
│   [✏️][🗑️]│                                                 │
│           │                                                 │
│ [Rebuild] │                                                 │
└───────────┴──────────────────────────────────────────────────┘

Drawing Mode (after clicking a feature ribbon button):
┌──────────────────────────────────────────────────────────────┐
│ [← Back] Component Editor  [Sketch ▾]                       │
├───────────┬──────────────────────────────────────────────────┤
│ Feature   │  Row1: [Select][Line][Circle][Arc]              │
│ Panel     │       [H][V][∥][⊥][L=][D=][∠][⚓]              │
│           │       [✓ Done] [✗ Cancel]  ← replaces Solve/Del │
│           │  Row2: [⬍Ext][↻Rev][⟿Swp][...] (disabled)      │
│           │──────────────────────────────────────────────────│
│           │  ⚠ 正在绘制 Extrude 轮廓 — 完成后点击 Done      │
│           │  ┌────────────────────────────────────────┐     │
│           │  │                                        │     │
│           │  │        2D Canvas (drawing active)      │     │
│           │  │                                        │     │
│           │  └────────────────────────────────────────┘     │
│           │──────────────────────────────────────────────────│
│           │  Status: Drawing Extrude · Entities: 3 · 2D     │
└───────────┴──────────────────────────────────────────────────┘
```

**Feature Types and Their Inputs:**

| Feature Type | Category | Required Inputs | Parameters |
|---|---|---|---|
| **Extrude** | 创建 | 1个轮廓 sketch | depth, direction_x/y/z, is_symmetric, draft_angle |
| **Revolve** | 创建 | 1个轮廓 sketch + 1个轴线 sketch | angle, axis_origin_x/y/z, axis_direction_x/y/z |
| **Sweep** | 创建 | 1个轮廓 sketch + 1个路径 sketch | (无额外参数) |
| **Loft** | 创建 | 2+个截面 sketch | (无额外参数) |
| **Boolean** | 修改 | 前面2个Feature的输出 | operation (union/subtract/intersect) |
| **Chamfer** | 修改 | 前面Feature的输出 | edge_indices, distance |
| **Fillet** | 修改 | 前面Feature的输出 | edge_indices, radius |
| **Pattern** | 修改 | 前面Feature的输出 | pattern_type, count, spacing |
| **Shell** | 修改 | 前面Feature的输出 | wall_thickness |

**Feature Panel Operations (Revit-style workflow):**

1. **添加 Feature**: 点击工具栏 Feature 按钮（如 "⬍ Extrude"）→ 自动创建 Sketch → 进入 2D 绘图模式 → 用户绘制轮廓 → 点击 "✓ Done" → 弹出参数对话框（sketch 已自动关联）→ 填写参数 → 确认创建
2. **编辑 Feature**: 双击 FeaturePanel 中的 feature 卡片（或点击 ✏️）→ 打开 AddFeatureDialog（预填参数和 sketch）→ 修改参数 → 保存
3. **删除 Feature**: 点击 feature 卡片的 🗑️ 按钮 → 确认 → 删除
4. **拖拽排序**: 拖拽 feature 卡片调整顺序 → 更新 order 字段
5. **重建**: 点击 "Rebuild All" → 按新顺序重建所有 feature

**Drawing Mode 状态机:**

```
idle → [点击 Feature 按钮] → creatingFeature(type, sketchId)
  → 自动创建 Sketch → loadSketch → setTab('component') → setActiveTool('line')
  → 用户在 2D Canvas 绘制轮廓
  → [点击 "✓ Done"] → editingFeature(sketch_id=自动关联)
  → 弹出 AddFeatureDialog → 用户设置参数
  → [确认] → api.addFeature() → creatingFeature=null → editingFeature=null → idle
  → [点击 "✗ Cancel"] → creatingFeature=null → idle
```

**Component 入口:**

- **创建**: 主视图工具栏 "+ Component" 按钮 → 创建 component → 跳转到编辑器 `?component={id}`
- **编辑**: 主视图 Object Tree 右键菜单 "Edit" → 跳转到编辑器 `?component={id}`

### 5.8 Parameter System

Named parameters with expressions drive feature dimensions. Parameters can reference each other through expressions.

```protobuf
message Parameter {
  string name = 1;
  string expression = 2;       // "5000" or "width * 2" or "sin(angle) * length"
  double value = 3;            // computed value
  string unit = 4;             // mm / deg / ...
  string description = 5;
}

message FeatureParameter {
  string feature_id = 1;
  string param_name = 2;       // maps to feature field (e.g., "depth", "radius")
  string expression = 3;       // can reference other parameters
}
```

Expression engine supports:
- Arithmetic: `+`, `-`, `*`, `/`, `^`
- Functions: `sin()`, `cos()`, `tan()`, `sqrt()`, `abs()`, `min()`, `max()`
- References: `length`, `width`, `wall_1.height`
- Conditionals: `if(condition, val_true, val_false)`

```protobuf
message MeshGeometry {
  repeated Point3D vertices = 1;
  repeated Vector3D normals = 2;
  repeated Point2D uvs = 3;
  repeated FaceIndices faces = 4;
  // 🗓 Planned: Draco compression field (not yet in proto/omnim.proto)
}
```

> **Implementation status (Parameter System)**: `Property`/`PropertyValue` (§7) are fully implemented; the named `Parameter`/`FeatureParameter` messages and the full expression engine above are design specification. Today, template parameters are resolved by `resolve_param()` in `crates/model/src/feature_template.rs`, which supports direct lookup (`"length"`), simple arithmetic (`"1.5 * DN"`), and literals (`"360"`). Functions (`sin()`, `if()`, …), units, and cross-parameter references are 🗓 Planned. `MeshGeometry` above is the proto definition (the `draco_compressed` field shown in earlier drafts is not implemented).

---

## 6. Object Generation Rule System (GenRule)

> **⚠️ Phase 6 Update**: Since GenRule refactoring (Phase 6), the primary geometry generation path has shifted from GenRule to the **Default Template Registry + Component Catalog**.
>
> When a wall is created via the command system, the current flow is (commands are typed `OmniCommand` messages — `proto/omcom.proto`, serialized as protobuf/serde JSON and executed by the Executor in `crates/omcom`; the `create wall W1` *text* form is the future surface defined by [RFC-0001](../RFC/RFC-0001-omcom-text-dsl.md), currently Draft):
> 1. `default_catalog_component(Wall)` resolves to `"Concrete-Wall"` (from **Library**)
> 2. The `Concrete-Wall` `ComponentDef` is looked up from the Library's `CatalogStore` (§14)
> 3. Its linked `FeatureTreeTemplate` (from Library's `FeatureTreeRegistry`) is instantiated
> 4. **Series check**: if the project has series loaded, verify `Concrete-Wall` is allowed by at least one series
> 5. The Feature Engine rebuilds the component via BRep operations
>
> **Important: GenRule has two layers with different deprecation status:**
>
> | Layer | What it is | Status |
> |-------|-----------|--------|
> | **GenRule as Classification** | `GenRuleHint` enum, `primary_gen_rule()` mapping (Wall → LayerExtrusion, Beam → LinearComponent, etc.) | ✅ **Active** — serves as a conceptual taxonomy for how geometry is structured per domain. Used by `default_catalog_component()` (template selection) and `default_series_for_type()` (series selection), both session-level global functions in the gen_rule module. |
> | **GenRule as Generation** | `generate_geometry()` calling `build_wall()`, `build_pipe()`, etc. which generate raw mesh directly | ❌ **Deprecated** — superseded by catalog feature tree templates (`FeatureTreeTemplate` → Feature Engine → BRep). |
>
> In short: **the 5 GenRule types still describe *what kind* of geometry each object has** (LayerExtrusion for walls, Primitive for beams), but the actual *generation* is now driven by catalog data, not hardcoded functions.

The `generate_geometry()` function (old GenRule path) is **deprecated**.
The GenRule types below remain as a **conceptual classification** layer
(used by `primary_gen_rule()` for type hints) but no longer drive geometry generation.
>
> See [`default_catalog_component()`](../../crates/model/src/gen_rule/mod.rs) and
> [`create_object_from_default_template()`](../../crates/model/src/project.rs) for the current implementation.

GenRule is the classification layer connecting domain semantics to the geometry kernel. Each ObjectType has a conceptual GenRule type that describes how its geometry is structured.

> **Design specification**: the `ObjectTypeDef`, `GenRule`, `LinearComponentGen`, `AssemblyGen`, and `BooleanSubtractGen` protobuf messages in this section are the target schema — in code, GenRule exists as the `GenRuleHint` Rust enum (`crates/model/src/gen_rule/mod.rs`) with the input structs `LayerExtrusionInput`, `LinearComponentInput`, `AssemblyInput`, `BooleanSubtractInput`; none of these messages is in proto yet.

**Architecture Note (Legacy)**: The original GenRule architecture was implemented on top of the Feature Engine and Constraint Solver. When a user typed `create wall std_wall 0,0 10,0 3.6` (historical CLI-style invocation — today's commands are typed `OmniCommand` messages, see RFC-0001 for the planned text surface), the GenRule would internally:
1. Create a sketch with the wall profile
2. Apply default constraints (horizontal base, vertical height)
3. Extrude the sketch with the specified thickness
4. Return the resulting feature tree

Since Phase 6, this expansion is done via **catalog feature tree templates** instead.
The 5 GenRule types below still serve as a **conceptual taxonomy** for geometry classification.

### 6.1 ObjectTypeDef Definition

```protobuf
message ObjectTypeDef {
  ObjectType type = 1;
  GenRule generation = 2;       // Geometry generation rule
  BehaviorSet behaviors = 3;    // Interaction behaviors
  ConstraintSet constraints = 4; // Constraint rules
}
```

### 6.2 Five GenRule Types

Geometry generation across all domains (building, plant, civil, mechanical) is unified into 5 GenRule types:

```protobuf
message GenRule {
  oneof rule {
    PrimitiveGen primitive = 1;      // Direct primitive
    LayerExtrusionGen layered = 2;   // Layered extrusion
    LinearComponentGen linear = 3;   // Linear component
    AssemblyGen assembly = 4;        // Assembly composition
    BooleanSubtractGen subtract = 5; // Boolean subtraction
  }
}
```

| GenRule | Applicable Domains | Typical Objects | Description |
|---------|-------------------|-----------------|-------------|
| 1. Primitive | Plant, Mechanical | Tank, Pump, Valve, Gear | Maps type parameters directly to a single primitive (Body components) |
| 2. LayerExtrusion | Building | Wall, Slab, Floor, Roof | Expands the surface boundary + layer structure into multiple Extrusions; the primary GenRule for **Surface** components |
| 3. LinearComponent | Building, Civil, Plant | Beam, Column, Road, Bridge, Pipe, CableTray | Path + variable cross-section + external constraints; the GenRule for **Linear** components |
| 4. Assembly | Mechanical, Plant | Equipment, Assemblies | Composes multiple primitives into one object (Body components) |
| 5. BooleanSubtract | All | Openings, SNO | Performs boolean subtraction on existing geometry |

### 6.3 Domain GenRule Mapping

```
Domain        Primary GenRule       Secondary GenRule
----------    ----------------      -----------------
Building      2. LayerExtrusion     5. BooleanSubtract
Plant         1. Primitive          3. LinearComponent + 5
Civil         3. LinearComponent    1. Primitive
Mechanical    4. Assembly           1. Primitive
Digital Twin  (reuses above)        + IoT data binding
```

### 6.4 LinearComponent

LinearComponent is one of OmniM's core innovations. It unifies building structural elements (Beam, Column with fixed cross-section), Civil3D roads (alignment + cross-section + width variation rules), and PDMS pipes (diameter + path) into a single abstraction: **path + variable cross-section + external constraints**. LinearComponent is the GenRule for **Linear** components (§1).

A Beam/Column = LinearComponent with constant cross-section (path + fixed section profile).
A road = LinearComponent with variable cross-section (horizontal alignment + vertical curve + segmented cross-section + terrain target constraint).

> **Wall is a Surface component, not a LinearComponent**: Unlike beams/columns/pipes (which are path-driven), a wall defines a 2D surface panel — a length×height rectangle (or arbitrary sketch shape) extruded with a layered material stack as thickness. Walls use **LayerExtrusion** (§6.2), not LinearComponent. The surface boundary provides the extrusion profile; the layer stack provides the thickness. This distinction matters because wall geometry is defined by *surface* properties (layers, boundary sketch), whereas linear component geometry is defined by *path* + cross-section properties.

```protobuf
message LinearComponentGen {
  Alignment3D main_path = 1;
  repeated Alignment3D edge_paths = 2;
  repeated CrossSectionLayer layers = 3;
  repeated VariationRule variations = 4;
  optional TerrainTarget terrain = 5;
  // Future enhancements (see RESEARCH_INTEGRATION.md §2.3):
  repeated CrossSectionSegment segments = 6;          // multi-segment variable cross-section
  repeated ComponentSequenceItem component_sequence = 7; // component sequence mode
  repeated ExternalTarget external_targets = 8;       // external constraint solving
}

message CrossSectionLayer {
  string name = 1;
  double default_width = 2;
  double default_thickness = 3;
  string material_id = 4;
  string function = 5;
}

enum InterpolationMethod {
  INTERPOLATION_CONSTANT = 0;
  INTERPOLATION_LINEAR = 1;
  INTERPOLATION_PARABOLIC_UP = 2;   // OBM Built-Up variation
  INTERPOLATION_PARABOLIC_DOWN = 3;
  INTERPOLATION_CIRCULAR_UP = 4;
  INTERPOLATION_CIRCULAR_DOWN = 5;
}

message VariationRule {
  double start_station = 1;
  double end_station = 2;
  string target = 3;
  InterpolationMethod interpolation = 4;   // extended to 6 methods (Bentley)
  double start_value = 5;
  double end_value = 6;
}

// Multi-segment variable cross-section (OBM Beam Rows / OTD Tunnel Unit)
message CrossSectionSegment {
  double start_station = 1;
  double end_station = 2;
  CrossSectionAtStation start_section = 3;
  CrossSectionAtStation end_section = 4;   // == start_section => sweep, no loft
  InterpolationMethod interpolation = 5;
}

// Component sequence item (E3D BRANCH component)
message ComponentSequenceItem {
  string component_def_id = 1;   // catalog ComponentDef reference
  double position_along_path = 2;
  map<string, string> params = 3;
}

// External constraint (ORD End Condition / Daylight)
message ExternalTarget {
  TargetType target_type = 1;   // Surface / Offset / Elevation / Width
  ConstraintType constraint = 2; // Daylight / Offset / Elevation / Width
  string target_id = 3;
  int32 priority = 4;           // ORD End Condition Exception: multiple targets tried in priority order
  optional double max_distance = 5; // maximum daylight search distance

  // Elevation target semantics (Civil 3D): a profile or surface elevation the
  // cross-section conforms to at each station
  optional string elevation_profile_id = 10;
  optional string elevation_surface_id = 11;

  // Width target semantics (Civil 3D): condition-driven width variation
  optional string width_condition_id = 12;  // e.g. "width = 3.6m if daylight hits EG, else 3.0m"
  optional double default_width = 13;
}
```

> **Implemented input schema** (`crates/model/src/gen_rule/linear_component.rs`): `LinearComponentInput { alignment: Vec<Point3D>, cross_sections: Vec<CrossSectionAtStation>, segments, ring_segments }`; a `CrossSectionLayer` carries `name`, `shape` (`CrossSectionShape`), `default_width`, `default_thickness`, `material_id: Option<String>`, `function`. Sections are lofted along the sampled path (variable cross-section with a static `segments` count — see §6.6). The richer `LinearComponentGen` message above (edge paths, variation rules, terrain/external targets) is design specification — 🗓 Planned.

**Four-kind target family** (from Civil 3D Target Mapping, `docs/research/site_terrain_components.md §2.5`): Civil 3D models a four-kind target family that `ExternalTarget`/`TargetType` should reflect:

| Target Kind | Semantics | OmniM mapping |
|-------------|-----------|---------------|
| **Surface** | Daylight/attach to a TIN surface (daylight to terrain, deck to design surface, subgrade base) | `TerrainModel` id in `target_id` |
| **Offset** | Feature-line or surface offset as the lateral target | `FeatureLine`/surface offset id |
| **Elevation** | Profile or surface elevation the section conforms to at each station | `elevation_profile_id` / `elevation_surface_id` |
| **Width** | Condition-driven width (width varies by terrain/design condition) | `width_condition_id` + `default_width` |

The existing `TargetType` (Surface/FeatureLine/Plane) is a subset of the **Surface** and **Offset** kinds. The distinct **Elevation** and **Width** targets add: (1) a *profile/surface-elevation* constraint driving vertical alignment, and (2) *width-variation-by-condition* semantics where the effective cross-section width is selected by a condition rather than a fixed value. This matches Civil 3D where a corridor's width can switch automatically based on target-daylight success or design conditions.

**External target exception handling**: Based on Bentley ORD End Condition Exception analysis (`docs/research/bentley_openroads_components.md §6.3.1`), multiple `ExternalTarget` entries can be specified with different `priority` values. The solver tries each target in priority order; the first one that successfully daylight (i.e., intersects a valid surface within `max_distance`) is used. This enables fallback behavior: e.g., "try daylight to Existing Ground first; if no intersection within 50m, try daylight to a Design Grade plane."

Doors and windows can be expressed as cross-section variation rules (a layer's thickness becomes 0 at a certain station), rather than as separate objects performing boolean subtraction — this is a reinterpretation of building domain concepts through the LinearComponent lens.

**Explicit Wall Openings collection** (from Revit, `docs/research/revit_components.md §5.1`): Revit tracks openings as named, schedulable, independently-editable members (Door/Window Opening). As an alternative to the cross-section variation approach above, openings can be modeled as an explicit `Openings` collection on the wall — each opening is a child object with its own identity, dimensions, and schedule data, referencing a host wall and a location along the wall path.

**Variation interpolation methods**: Based on Bentley OBM Built-Up girder analysis (`docs/research/bentley_openroads_components.md §2.1.3`), the interpolation methods are extended from a simple linear to 6 methods: Constant, Linear, Parabolic Up/Down, Circular Up/Down. These cover the full range of structural cross-section variation observed across all researched platforms.

**Multi-segment variable cross-section**: Based on Bentley OBM Beam Rows + Different End Template and OTD multi-Tunnel-Unit analysis, a `CrossSectionSegment` list enables per-segment section definition. When `start_section == end_section` within a segment, the segment is a simple sweep (no loft) — matching the "segment-interpolation + segment-jump" engineering reality observed in OTD §4.1.

**Multi-layer cross-sections (cable/conduit/duct)**: Beyond the structural and corridor cross-sections above, LinearComponent also models **concentric layered cross-sections** for cables and conduits. Based on AVEVA Instrumentation cable analysis (`docs/research/e3d_instrumentation_components.md §3.5.2`), a cable is a 5-layer concentric cross-section: Conductor → Insulation → Screen/Shield → Armor → Outer Sheath. These map to `CrossSectionLayer` entries where `default_width` is the outer diameter and `default_thickness` is the radial layer thickness (concentric ordering). A future enhancement should add `center_radius` and `radial_thickness` to `CrossSectionLayer` for explicit concentric section definition.

> **Cable tray fill factor + routing** (`docs/research/e3d_instrumentation_components.md §3.1.4, §3.5.2`): Cable tray routing is driven by a **fill factor** rule (sum of cable cross-section areas / tray cross-section area), plus signal/power separation and automatic path selection. Each cable carries a `cross_section` property (mm²) used to compute tray fill. The fill factor constrains both routing feasibility and maximum cable count per tray.

**Rebar as LinearComponent** (from Tekla Structures, `docs/research/tekla_components.md §6.1`): Reinforcement rebar is a LinearComponent with a circular cross-section sweeping along a path (straight or bent). Key parameters beyond the base LinearComponent include: Size (diameter), Grade (e.g., B500B), RadiusValues (bending radii), OnPlaneOffsets / FromPlaneOffset (positioning), StartHook / EndHook (hook angles), SpliceType / SpliceOffsets (splicing), StirrupType (stirrup vs. non-stirrup), ExcludeType (distribution pattern). Rebar modifiers (RebarPropertyModifier, RebarSplitter, RebarEndDetailModifier) are modeled as post-generation feature operations in the Feature Engine.

**Level-constrained vertical extent** (from Revit, `docs/research/revit_components.md §5.1`): Building vertical elements (Wall/Floor/Roof) are constrained to levels (top/bottom) rather than absolute heights. LinearComponentGen supports a `top_level_id` / `bottom_level_id` constraint concept for building-domain parity: the vertical extent is resolved relative to named level elevations, enabling automatic height adjustment when level elevations change.

**Connector-to-Connector creation mode** (from Revit, `docs/research/revit_components.md §7`): MEP elements (Pipe/Duct/CableTray) in Revit are created by selecting a start and end Connector, not just a path. This is an additional LinearComponent creation mode: instead of specifying a geometric path, the user picks two Connectors on existing components, and the system auto-routes the path between them (respecting slope, clearance, and connection type rules). The `main_path` is then generated automatically rather than user-specified.

### 6.5 LinearComponent Component Sequence Mode (E3D BRANCH)

> **Research reference**: E3D BRANCH describes a pipeline as a sequence of discrete components (pipe segments, elbows, tees, reducers) along a path. See `docs/research/aveva_e3d_primitives.md`, `docs/research/comparative_analysis.md`, `docs/research/genrule_design_analysis.md`.

The third mode of LinearComponent — **component sequence** — is not yet implemented. Unlike the fixed-section mode (Beam/Column) or variable cross-section mode (Road), this mode places discrete components one after another along a path:

```
ComponentSequence
├── ComponentItem: PipeSegment (length=6000, DN=100)
├── ComponentItem: Elbow (angle=90°, DN=100)
├── ComponentItem: PipeSegment (length=3000, DN=100)
├── ComponentItem: Reducer (DN=100→80)
└── ComponentItem: Valve (DN=80)
```

Each `ComponentItem` references a `ComponentDef` from the catalog and specifies its position along the path. This is the E3D BRANCH pattern — a pipeline is not a continuous extrusion but a sequence of catalog components.

**Implementation plan**: Add `ComponentSequenceItem` to `LinearComponentInput` in future phases. See `docs/research/genrule_design_analysis.md §5.2` for details.

### 6.6 Adaptive Sampling & LOD Strategy

> **Research reference**: `docs/research/genrule_design_analysis.md §5.2`.

**Adaptive sampling**: The current `segments` parameter in `LinearComponentInput` is a static count. Future implementation should dynamically adjust segment density based on path curvature — denser in high-curvature regions, sparser in straight sections — to balance geometry accuracy and performance.

**Civil 3D sampling frequency nodes** (from Civil3D §4.3): Corridor sampling density should be driven by concrete triggers, not just curvature. The following sampling-frequency node types are derived from Civil 3D's corridor frequency settings:
- `AlongCurves` — denser sampling along horizontal/vertical curves (e.g., every 1-2m based on radius)
- `AlongTangents` — sparser sampling along straight sections (e.g., every 10-20m)
- `AtProfileHighLowPoints` — insert sample at every crest and sag of the vertical profile
- `AtSuperelevationCriticalPoints` — insert samples at superelevation transition stations (start/end of runoff, full super)
- `AtAlignmentGeometryPoints` — insert at every alignment PI, PC, PT, and profile VPI
- `AtStructureCrossings` — insert at every intersecting pipe, culvert, or structure location

These sampling node types should be added to the `LinearComponentGen` input as a `repeated SamplingNode sampling_nodes` field, enabling corridor engines to generate geometry at the right stations for both accuracy and performance.

**LOD (Level of Detail) strategy**: The current `lod`/`level_of_detail` field is a single tessellation tolerance. A future multi-level LOD system should support distinct levels (coarse / medium / fine) with progressive mesh switching based on view distance, enabling efficient rendering of large-scale models.

### 6.7 Additional Research Findings

> **Research reference**: `docs/research/genrule_design_analysis.md §5.3`, `docs/research/comparative_analysis.md §8.3`.

**BooleanOperation in ParametricOperation**: The existing `BooleanSubtract` GenRule module is retained as an independent module. A future enhancement should add a `BooleanOperation` type to `ParametricOperation` (e.g., `BooleanUnion`, `BooleanIntersect`, `BooleanSubtract`) so that the Feature Engine can perform boolean operations directly within the feature tree, not just at the GenRule level.

**Flowchart-style VariationRule (SAC pattern)**: The current `VariationRule` supports simple interpolation (constant/linear/smooth). Future enhancement should support Civil3D Subassembly Composer-style flowchart logic — `Sequence`, `Decision`, `Switch` nodes — enabling conditional geometry generation based on parameters, targets, and external conditions.

> **Design Specification**: The TerrainModel, LogicalObject, BooleanSubtract Enhancement, Assembly Enhancement, and Constraint System (§6.8-§6.12) are design specifications. Their protobuf messages are defined in this document but may not yet be implemented in the codebase/proto files.

### 6.8 TerrainModel (TIN Surface)

> **Research reference**: Multi-platform site/terrain analysis (`docs/research/site_terrain_components.md`), Bentley OpenRoads Terrain Model (`docs/research/bentley_openroads_components.md §6.1`). Terrain is a TIN surface, not a sweep/loft/boolean — it does not fit the 5 GenRule types.

`TerrainModel` is a distinct geometry type, separate from the 5 GenRule types. It represents an irregular triangulated network (TIN) surface.

```protobuf
message TerrainModel {
  repeated TerrainVertex vertices = 1;
  repeated TerrainTriangle triangles = 2;    // auto-generated from vertices + breaklines
  repeated TerrainBreakline break_lines = 3; // edges that triangles cannot cross
  repeated TerrainBoundary boundaries = 4;   // outer / benchmark / void boundaries
  repeated Void voids = 5;                   // holes in the terrain
  TerrainRole role = 6;                      // Existing / Design / Subgrade / Reference
  optional GeographicReference geo_ref = 7;  // coordinate system (EPSG, projection)
  map<string, string> properties = 8;        // source, accuracy, metadata
}

message TerrainVertex {
  string id = 1;
  double x = 2; double y = 3; double z = 4;
  map<string, string> properties = 5;       // e.g. "source" = "survey", "accuracy" = "0.01m"
}

message TerrainTriangle {
  string id = 1;
  uint32 v0 = 2; uint32 v1 = 3; uint32 v2 = 4;  // vertex indices
}

message TerrainBreakline {
  string id = 1;
  repeated uint32 vertex_indices = 2;        // ordered point chain
  bool enforced = 3;                         // triangles cannot cross
}

message TerrainBoundary {
  string id = 1;
  repeated uint32 vertex_indices = 2;        // closed polygon
  BoundaryType type = 3;                     // OUTER / BENCHMARK / VOID
}

enum BoundaryType {
  BOUNDARY_OUTER = 0;    // outer boundary of the terrain
  BOUNDARY_BENCHMARK = 1; // benchmark boundary (limits volume calc region)
  BOUNDARY_VOID = 2;     // hole in the terrain
}

enum TerrainRole {
  TERRAIN_EXISTING = 0;   // subtracted surface (existing ground)
  TERRAIN_DESIGN = 1;     // additive surface (design grade)
  TERRAIN_SUBGRADE = 2;   // independent volume calc (subgrade)
  TERRAIN_REFERENCE = 3;  // reference surface, no volume calc
}
```

**Role system**: Borrowed from Bentley Terrain Model's Volume Option (`docs/research/site_terrain_components.md §8.3`). The `TerrainRole` enum controls how a surface participates in volume calculations:
- `TERRAIN_EXISTING`: the "before" surface — subtracted in Cut/Fill analysis
- `TERRAIN_DESIGN`: the "after" surface — the design intent grade
- `TERRAIN_SUBGRADE`: an independent surface for subgrade volume (e.g., road base course)
- `TERRAIN_REFERENCE`: a reference surface with no volume contribution

**Creation sources**: TerrainModel can be created from:
1. Survey points (imported from `.txt` / `.csv` / total station data)
2. Contour lines (imported from DWG/DXF/LandXML)
3. Corridor output (Civil 3D / Bentley corridor → TIN surface)
4. Plane terrain (horizontal plane at a given elevation)
5. External reference (LandXML, IFC, or point cloud)
6. Manual point entry (add points interactively)

**Editing API**: TerrainModel supports the following operations:
- `add_vertex(x, y, z)` / `remove_vertex(id)` / `modify_vertex(id, z)`
- `add_breakline(indices)` / `remove_breakline(id)`
- `add_boundary(indices, type)` / `remove_boundary(id)`
- `merge(other)` — merge two TIN surfaces
- `split(plane)` — split terrain along a plane
- `smooth(iterations)` — smooth the TIN surface
- `auto_triangulate()` — regenerate triangles from vertices + breaklines

**Analysis API**: TerrainModel supports spatial queries:
- `get_elevation(x, y)` → interpolated elevation at a point
- `get_slope(x, y)` → slope gradient at a point
- `get_contour_lines(elevation_interval)` → contour curves
- `volume_analysis(other, method)` → CutVolume / FillVolume / EarthworkBalance
- `surface_area()` → total surface area
- `daylight_analysis(linear_component, slope_ratio)` → intersection curve with a LinearComponent End Condition

**Usage**: TerrainModel serves as:
1. An external constraint target for LinearComponent End Conditions (daylight solving)
2. A source for Cut/Fill volume computation (BooleanSubtract between two TIN surfaces)
3. A visualization surface for terrain, site grading, and corridor output
4. A base for derived objects: CutVolume, FillVolume, GradingSurface, EarthworkBalance

**Volume calculation methods**: Based on Bentley ORD analysis (`docs/research/bentley_openroads_components.md §6.2.1`) and Civil 3D grading (`docs/research/site_terrain_components.md §7.3`, three algorithms), supported via `volume_analysis()`:
1. **Element Component Quantities** — per-element volume of LinearComponent layers
2. **Corridor Component Quantities** — corridor-wide volume by layer
3. **End Area Volumes** — cross-section-based volume between stations (Average End Area)
4. **Terrain-to-Terrain** — BooleanSubtract between two TIN surfaces (Existing − Design)
5. **Terrain-to-Plane** — BooleanSubtract between TIN surface and a horizontal plane
6. **Quantities Report by Named Boundary** — volume within a bounded region of the terrain
7. **Composite / Conic / Prismoidal** — Civil 3D's three volume algorithms (Conic is most accurate for curved corridors)

**Derived objects** (from `docs/research/site_terrain_components.md §7.3`):
- **CutVolume** — excavation volume result (BooleanSubtract: Existing − Design)
- **FillVolume** — fill volume result (BooleanSubtract: Design − Existing)
- **GradingSurface** — a TerrainModel with `TERRAIN_DESIGN` role, produced by a Grading Group
- **Grading Group** — a `GradingGroup` binding a feature line, a grading surface, a target surface, and grading rules (see §6.8.1 below)
- **EarthworkBalance** — a logical object comparing Cut vs Fill totals for a bounded region
- **Slope** — a LinearComponent whose cross-section is a slope face swept along a boundary/edge line, with `ExternalTarget` for daylight intersection
- **SiteComponent** — a `ComponentDef` attached to a terrain surface (trees, utility poles, fire hydrants)
- **FeatureLine** — a no-thickness control/feature line (LinearElement) used as grading reference
- **ContourLine** — a curve element representing a fixed-elevation contour for display/analysis

#### 6.8.1 Grading Group (Civil 3D Grading)

> **Research reference**: `docs/research/site_terrain_components.md §2.3`. A Grading Group binds a design grade surface, an existing terrain target, and slope rules driven off a feature line, producing a graded design surface with optional benched/stepped platforms.

```protobuf
message GradingGroup {
  string id = 1;
  string name = 2;

  // The feature line driving the grading (pad, boundary, bench edge)
  string feature_line_id = 3;

  // The design grading surface being created (role=DESIGN)
  string grading_surface_id = 4;

  // The target/existing terrain surface (role=EXISTING)
  string target_surface_id = 5;

  // Grading rules applied to the feature line
  repeated GradingRule rules = 6;

  // Optional: multi-platform bench grading
  repeated BenchLevel bench_levels = 7;
}

message GradingRule {
  string name = 1;                  // e.g. "Cut 1:1.5", "Fill 1:2"
  GradingRuleType type = 2;         // CUT / FILL / DAYLIGHT
  double cut_slope = 3;             // e.g. 1.5 (1V:1.5H)
  double fill_slope = 4;            // e.g. 2.0 (1V:2H)
  optional double max_cut_depth = 5; // maximum excavation depth before benching
  optional double max_fill_height = 6; // maximum fill height before benching
  double daylight_max_distance = 7; // search limit for daylight intersection
  double min_slope_length = 8;      // minimum slope face length (design criteria)
  double max_slope_ratio = 9;       // maximum slope ratio (design criteria)
}

enum GradingRuleType {
  GRADING_RULE_DAYLIGHT = 0;   // slope to terrain intersection
  GRADING_RULE_TO_SURFACE = 1; // slope to a specified elevation surface
  GRADING_RULE_TO_ELEVATION = 2; // slope to a fixed elevation
  GRADING_RULE_TO_DISTANCE = 3; // slope projected a fixed distance
}

// Bench grading: multi-platform stepped grading
message BenchLevel {
  string name = 1;               // e.g. "Bench 1", "Catch 1"
  double elevation = 2;          // bench platform elevation
  double bench_width = 3;        // horizontal platform width
  double bench_slope = 4;        // platform cross-slope (drainage)
  optional double ditch_width = 5;  // optional ditch at bench inner edge
  optional double ditch_depth = 6;
}
```

**Grading workflow** (from `site_terrain_components.md §2.3`):
1. Define a **Feature Line** (pad edge, boundary, or bench edge) on the design surface
2. Create a **Grading Group** referencing the feature line, a target surface (existing terrain), and one or more **Grading Rules**
3. Grading rules carry design criteria: `CutSlope` (e.g. 1V:1.5H), `FillSlope` (e.g. 1V:2H), `MaxCutDepth`, `MaxFillHeight`, `min_slope_length`, `max_slope_ratio`, `bench_width`
4. The solver generates the graded surface by projecting the feature line outward according to the rules, computing daylight intersection with the target surface
5. **Bench Grading** (multi-platform/stepped): when slope exceeds max cut depth, a bench platform is inserted at `bench_levels[n].elevation` with a specified `bench_width`, creating a terraced slope profile

### 6.9 LogicalObject (Tag / Loop / LineNumber)

> **Research reference**: AVEVA Instrumentation's Loop/Tag system (`docs/research/e3d_instrumentation_components.md §4`). Loop and Tag are logical entities without geometry, yet they are core engineering objects.

`LogicalObject` is a new base type for objects that have no geometry but carry engineering data and reference physical objects. They sit alongside `Object` (physical) in the object hierarchy.

```protobuf
message LogicalObject {
  string id = 1;
  string name = 2;
  LogicalObjectType type = 3;
  repeated string member_object_ids = 4;     // references to physical objects
  map<string, string> properties = 5;        // tag, loop_number, line_number, etc.
}

enum LogicalObjectType {
  LOGICAL_TAG = 0;          // ISA-5.1 Tag (e.g., "FT-101A")
  LOGICAL_LOOP = 1;         // Control loop (e.g., Loop 101)
  LOGICAL_LINE_NUMBER = 2;  // Process line (e.g., "12\"-3A1-CS-SCH40")
  LOGICAL_SYSTEM = 3;       // Engineering system
  LOGICAL_UNIT = 4;         // Plant unit/area
  LOGICAL_CABLE = 5;        // Cable schedule entry
}
```

**Key design principles**:
1. LogicalObject has no geometry, no GenRule, no Feature tree
2. It holds references to physical objects via `member_object_ids`
3. It carries engineering properties (Tag string, Loop number, etc.)
4. It enables cross-view traceability: logical side (P&ID) ↔ physical side (3D model)

> **LineNumber structured encoding** (`docs/research/e3d_instrumentation_components.md §4.2.1`): The LineNumber LogicalObject carries a structured encoding `DN"-"Spec"-"Material"-"Schedule"-"HeatTracing"-"Insulation"` (e.g., `12"-3A1-CS-SCH40-HT1-INS`). This enables parsing, validation, and cross-view matching between P&ID logical lines and 3D physical pipe runs.

### 6.10 BooleanSubtract Enhancements

> **Research reference**: Tekla's BooleanPart/Fitting/CutPlane (`docs/research/tekla_components.md §8`). Tekla supports three Boolean modes: part-to-part cut, plane cut, and edge chamfer.

The current `BooleanSubtract` GenRule supports only **solid-to-solid** subtraction. Based on Tekla analysis, two additional modes are needed:

```protobuf
message BooleanSubtractGen {
  // Existing: solid-to-solid subtraction
  optional string base_id = 1;       // base object (the one being cut)
  optional string cutter_id = 2;     // cutting object (solid body)

  // New: plane cut (Tekla Fitting / CutPlane)
  // Cuts the base object with a half-space plane, more efficient than solid boolean
  optional PlaneCut plane_cut = 10;

  // New: edge chamfer (Tekla EdgeChamfer / SOLIDWORKS Fillet)
  optional EdgeChamfer edge_chamfer = 11;

  // New: surface-surface subtraction (Cut/Fill volume)
  // Civil 3D / Bentley earthwork: Boolean between two TIN surfaces
  optional SurfaceSubtract surface_subtract = 12;
}

// Surface-surface boolean (TIN earthwork)
message SurfaceSubtract {
  string base_terrain_id = 1;    // e.g. Existing surface
  string cutter_terrain_id = 2;  // e.g. Design surface
  VolumeMethod method = 3;       // COMPOSITE / CONIC / PRISMOIDAL / END_AREA
  VolumeResult result = 4;       // CUT / FILL / BALANCE
}

enum VolumeMethod {
  VOLUME_COMPOSITE = 0;   // Civil 3D default, most accurate
  VOLUME_CONIC = 1;       // curved corridors
  VOLUME_PRISMOIDAL = 2;  // prismoidal correction
  VOLUME_END_AREA = 3;    // average end area between stations
}

enum VolumeResult {
  VOLUME_RESULT_CUT = 0;    // excavation (Existing − Design > 0)
  VOLUME_RESULT_FILL = 1;   // fill (Design − Existing > 0)
  VOLUME_RESULT_BALANCE = 2; // net balance comparison
}

message PlaneCut {
  Plane cut_plane = 1;         // plane definition
  bool keep_positive_side = 2; // which side of the plane to keep
}

message EdgeChamfer {
  repeated uint32 edge_indices = 1;  // edges to chamfer/fillet
  double distance = 2;               // chamfer distance
  optional double radius = 3;        // fillet radius (overrides distance if set)
  ChamferType type = 4;              // CHAMFER / FILLET
}
```

> **BooleanUnion from Tekla BooleanPart** (from Tekla, `docs/research/tekla_components.md §8.1`): Tekla `BooleanPart` supports both `CUT` and `ADD` modes via `BooleanTypeEnum`. The `BooleanSubtractGen` GenRule should be generalizable to a `BooleanOperationType` enum (CUT/ADD) to support union operations. The ADD mode corresponds to a `BooleanUnion` that merges the `OperativePart` geometry into the `Father` — equivalent to OmniM's `BooleanOperationType::UNION`.

### 6.11 Assembly Enhancements

> **Research reference**: Tekla ConnectionBase (`docs/research/tekla_components.md §3`), Bentley Civil Cell (`docs/research/bentley_openroads_components.md §8.3`).

The Assembly GenRule is enhanced with conditional parameters and processual scripting capability:

```protobuf
message AssemblyGen {
  // Existing: declarative children list
  repeated SubComponent children = 1;

  // New: conditional parameters (Tekla Connection parameter switches)
  // e.g., "has_stiffeners" = true → generate stiffener child components
  map<string, bool> conditions = 10;

  // New: processual script reference (ConnectionScript)
  // References a catalog ComponentDef that implements the generation logic
  optional string script_component_id = 11;

  // New: expression-driven child positions (Bentley Civil Cell)
  // e.g., "cap.position_x = bridge_alignment.station(0.5).x"
  map<string, string> position_expressions = 12;
}

message SubComponent {
  string component_def_id = 1;
  Transform transform = 2;
  GenRuleHint gen_rule = 3;          // each child has its own GenRule
  optional string condition_ref = 4; // only generate if condition is true
}
```

> **ComponentInput order sensitivity** (from Tekla, `docs/research/tekla_components.md §3.2`): Tekla's `ComponentInput` is order-sensitive — `AddInputObject` + `AddInputPosition` produces different results from the reverse order. The `script_component_id` reference in `AssemblyGen` should define input slots (Primary, Secondaries, Positions) with explicit ordering, since the downstream ConnectionScript `Run()` method consumes them in sequence.

> **Parametric plate generation pattern** (from Tekla, `docs/research/tekla_components.md §4.1`): Gusset plates, end plates, and base plates are a common ConnectionScript pattern. The script takes a primary (beam/column) + secondaries (braces) + parameters (edge distances, angles, bolt positions) and generates a `ContourPlate` with a parameterized polygon contour. This is a canonical `AssemblyGen` use case: a `SubComponent` with `Primitive::Extrusion` whose contour vertices are expression-driven.

### 6.12 Constraint System (Mates)

> **Research reference**: SOLIDWORKS 21 Mates types (`docs/research/solidworks_components.md §4`). SOLIDWORKS supports 3 categories of mates: Standard (8), Advanced (6), Mechanical (7).

A new `Constraint` system is added alongside the existing `Connection` system in `RelationStore`. Constraints govern how Assembly child components relate to each other in terms of degrees of freedom.

```protobuf
message Constraint {
  string id = 1;
  string name = 2;
  ConstraintType type = 3;
  string entity_a_id = 4;          // first component/face/edge/vertex
  string entity_b_id = 5;          // second component/face/edge/vertex
  optional double distance = 6;     // for Distance constraint
  optional double angle = 7;        // for Angle constraint
  optional double ratio = 8;        // for Gear / LinearCoupler
  optional double min_value = 9;    // for Limit constraints
  optional double max_value = 10;
}

enum ConstraintType {
  // Standard Mates (SOLIDWORKS)
  CONSTRAINT_COINCIDENT = 0;
  CONSTRAINT_CONCENTRIC = 1;
  CONSTRAINT_PARALLEL = 2;
  CONSTRAINT_PERPENDICULAR = 3;
  CONSTRAINT_TANGENT = 4;
  CONSTRAINT_DISTANCE = 5;
  CONSTRAINT_ANGLE = 6;
  CONSTRAINT_LOCK = 7;

  // Advanced Mates
  CONSTRAINT_PROFILE_CENTER = 10;
  CONSTRAINT_SYMMETRIC = 11;
  CONSTRAINT_WIDTH = 12;
  CONSTRAINT_PATH_MATE = 13;
  CONSTRAINT_LINEAR_COUPLER = 14;
  CONSTRAINT_LIMIT = 15;

  // Mechanical Mates
  CONSTRAINT_CAM = 20;
  CONSTRAINT_SLOT = 21;
  CONSTRAINT_HINGE = 22;
  CONSTRAINT_GEAR = 23;
  CONSTRAINT_RACK_PINION = 24;
  CONSTRAINT_SCREW = 25;
  CONSTRAINT_UNIVERSAL_JOINT = 26;
}
```

---

## 7. Property System

OmniM supports both flat properties and IFC-style property sets.

> **UDA/UDET (User Defined Attributes/Types)**: E3D's LEXICON database (`docs/research/e3d_instrumentation_components.md §1.3`) provides UDA (User Defined Attributes) and UDET (User Defined Element Types). OmniM's Property System supports the equivalent via dynamic property sets: any object can carry arbitrary key-value pairs via `PropertySet` (implemented — matching `Property`/`PropertySet` in `proto/omnim.proto`), and property definitions/types per ObjectType exist in `crates/model/src/property_def.rs`. Note there is no dedicated "user-defined 900+" range in the `ObjectType` enum — the only generic value is `OBJECT_TYPE_GENERIC = 999`; a proper user-defined type mechanism is 🗓 Planned.

```protobuf
message Property {
  string name = 1;
  PropertyValue value = 2;
  string unit = 3;
  string description = 4;
}

message PropertyValue {
  oneof value_type {
    string string_value = 1;
    int64 int_value = 2;
    double double_value = 3;
    bool bool_value = 4;
    Point3D point_value = 5;
    Vector3D vector_value = 6;
    bytes binary_value = 7;
    google.protobuf.Timestamp timestamp_value = 8;
  }
}

message PropertySet {
  string name = 1;
  string description = 2;
  repeated Property properties = 3;
}
```

### Standard Property Sets (IFC aligned)
- `Pset_WallCommon`
- `Pset_BeamCommon`
- `Pset_ColumnCommon`
- `Pset_SlabCommon`
- `Pset_DoorCommon`
- `Pset_WindowCommon`
- Plus domain-specific property sets

> **Implementation status**: these names are a naming convention aligned with IFC, not predefined server-side templates — `PropertySet` instances are free-form, named per object (the IFC importer/exporter round-trips `Pset_*` names, see `crates/ifc/tests/ifc_roundtrip.rs`). `crates/model/src/property_def.rs` provides per-ObjectType property definitions, but template-defined `Pset_*` bundles are 🗓 Planned.

---

## 8. Relationship System

OmniM has a native relationship system that supports:

### 8.1 Containment (Hierarchy)

```protobuf
message Containment {
  string parent_id = 1;
  string child_id = 2;
  ContainmentType type = 3;
}

enum ContainmentType {
  CONTAINMENT_TYPE_UNSPECIFIED = 0;
  SPATIAL = 1;      // Spatial hierarchy (Site → Building → Storey)
  PHYSICAL = 2;     // Physical assembly (Assembly → Part)
  SYSTEM = 3;       // System membership (PipeSystem → Pipe segments)
  GROUP = 4;        // Logical grouping
}
```

### 8.2 Connections

```protobuf
message Connection {
  string from_object_id = 1;
  string to_object_id = 2;
  ConnectionType type = 3;
  Point3D from_point = 4;
  Point3D to_point = 5;
  string connection_detail = 6;  // e.g., "Welded", "Bolted", "Flanged"
}

enum ConnectionType {
  CONNECTION_TYPE_UNSPECIFIED = 0;
  STRUCTURAL = 1;    // Structural connection (beam to column)
  MECHANICAL = 2;    // Mechanical (pipe flange)
  ELECTRICAL = 3;    // Electrical (wire to terminal)
  HYDRAULIC = 4;     // Hydraulic connection
  SPATIAL_ADJACENT = 5;  // Objects are adjacent
}
```

### 8.3 Spatial Relationships

```protobuf
message SpatialRelation {
  string object_a_id = 1;
  string object_b_id = 2;
  SpatialRelationType type = 3;
  double distance = 4;
}

enum SpatialRelationType {
  SPATIAL_RELATION_TYPE_UNSPECIFIED = 0;
  INTERSECTS = 1;
  CONTAINS = 2;
  WITHIN = 3;
  OVERLAPS = 4;
  TOUCHES = 5;
  DISJOINT = 6;
}
```

> **Design specification (🗓 Planned)**: `SpatialRelation` is not yet in `proto/omnim.proto` — the relationship system currently implements Containment and Connection (`RelationStore`, `crates/model/src/relation.rs`). Spatial predicate queries (intersects/contains/…) would sit on the spatial index of §12, which is itself not yet implemented.

---

## 9. Material System

```protobuf
message Material {
  string id = 1;
  string name = 2;
  MaterialCategory category = 3;
  
  // Physical properties
  double density = 4;        // kg/m³
  double youngs_modulus = 5; // Pa
  double poissons_ratio = 6;
  double yield_strength = 7; // Pa
  double ultimate_strength = 8; // Pa
  double thermal_conductivity = 9; // W/(m·K)
  double specific_heat = 10; // J/(kg·K)
  
  // Visual properties
  Color color = 11;
  double roughness = 12;
  double metallic = 13;
  double transparency = 14;
  
  // Classification
  string ifc_material = 15;
  repeated string standards = 16; // e.g., "ASTM A36", "EN 10025"
}

enum MaterialCategory {
  MATERIAL_CATEGORY_UNSPECIFIED = 0;
  METAL = 1;
  CONCRETE = 2;
  WOOD = 3;
  MASONRY = 4;
  PLASTIC = 5;
  COMPOSITE = 6;
  GLASS = 7;
  INSULATION = 8;
  SOIL = 9;
  WATER = 10;
  OTHER = 99;
}

message Color {
  float r = 1;
  float g = 2;
  float b = 3;
  float a = 4;
}
```

---

## 10. Version and Audit System

> **Design specification (🗓 Planned)**: no `ObjectVersion`/`ChangeType` messages exist in `proto/omnim.proto` or code yet. There is a session-level undo/redo stack (`/api/undo`, `/api/redo`) and `change_description`-style audit hooks on commands, but per-object version records are design goals.

```protobuf
message ObjectVersion {
  string object_id = 1;
  string version_id = 2;
  string previous_version_id = 3;
  google.protobuf.Timestamp timestamp = 4;
  string author_id = 5;
  string change_description = 6;
  repeated string changed_fields = 7;
  ChangeType change_type = 8;
}

enum ChangeType {
  CHANGE_TYPE_UNSPECIFIED = 0;
  CREATED = 1;
  MODIFIED = 2;
  GEOMETRY_CHANGED = 3;
  PROPERTIES_CHANGED = 4;
  RELATIONSHIP_CHANGED = 5;
  DELETED = 6;
}
```

---

## 11. Classification Mapping

### 11.1 IFC Mapping

Core building elements map directly to IFC classes. Verified against `crates/ifc/src/lib.rs` (`export_ifc_class()` — lines 584-585):

| OmniM Class | IFC Class | Export status |
|-------------|-----------|---------------|
| Site | IfcSite | ✅ Exported / imported (`IFCSITE`) |
| Building | IfcBuilding | ✅ |
| BuildingStorey | IfcBuildingStorey | ✅ |
| Space | IfcSpace | ✅ |
| Wall | IfcWall | ✅ (`IfcWallStandardCase` from imported IFC is normalized to `IfcWall` on re-export) |
| Beam | IfcBeam | ✅ |
| Column | IfcColumn | ✅ |
| Slab | IfcSlab | ✅ |
| Door | IfcDoor | ✅ |
| Window | IfcWindow | ✅ |
| Stair | IfcStair | 🗓 Planned (currently exported as `IfcBuildingElementProxy`) |
| Roof | IfcRoof | 🗓 Planned (currently `IfcBuildingElementProxy`) |

All other object types fall back to `IfcBuildingElementProxy` on export; geometry is exported as `IfcFacetedBrep`. IFC import (`import_ifc()`) and export (`export_ifc()`) both ship in `crates/ifc` / `crates/omcom/src/ifc`.

### 11.2 Domain Extensions

Non-building domains use extended classification:

- **Civil**: Follow ISO 12006 where applicable, extend for roads/bridges
- **Plant**: Follow ISO 15926 and P&amp;ID standards
- **Mechanical**: Follow STEP AP242 and ISO 10303

---

## 12. Indexing and Query Optimization

Design goals for query performance; the current `ObjectStore` (`crates/model/src/store.rs`) is an ID→object map with parent lookup — the physical indexes below are 🗓 Planned:

### Primary Indexes
- Object ID (primary key)
- Object type
- Spatial bounding box (R-tree)
- Containment hierarchy

### Secondary Indexes
- Property value indexes
- Tag indexes
- Material indexes
- IFC class mapping
- Relationship indexes

---

## 13. Open Questions

1. How to handle different level of detail (LOD) for the same object?
2. Should parametric history be preserved for imported objects?
3. What is the optimal representation for alignments and profiles in civil?
4. How to handle material composites and layered materials?
5. Should constraint relationships be first-class citizens?

---

## 14. Component Catalog (元件库 / PARAGON)

> **Library-level resource**: The Component Catalog belongs to the **Library** (session-level global singleton), not the Project. Project accesses Library content indirectly through Series, never directly. See [Library vs Project](#1-overview).

The Component Catalog is the master library of all parametric 3D component definitions. Each component is defined using the same sketch → constraint → feature pipeline as any other parametric model, then published to the catalog for reuse.

### 14.1 Core Concepts

| Concept | PDMS Equivalent | Description |
|---------|----------------|-------------|
| **ComponentDef** | SCOM (catalog component) | A parametric 3D model definition with typed parameters, feature tree, and connectors |
| **Category** | CATE (category) | Logical grouping of components with similar geometry (e.g., "Elbow-LR", "IPE-Beam") |
| **Section** | SECT (section) | Domain grouping (e.g., "Piping", "Structural", "Equipment") |
| **Connector** | P-point (pointset) | Connection point with position, direction, bore, and type code for compatibility matching |

**In-Place Component** (from Revit, `docs/research/revit_components.md §3.1`): A Component can be flagged `in_place` (project-local, non-reusable, not promotable to catalog). Unlike a standard `ComponentDef` which lives in the Library's CatalogStore and is reusable across projects, an in-place component is defined directly in the project's Object Store, cannot be referenced by a Series, and is not available via `catalog list`. This distinction mirrors Revit's "In-Place Mass" vs "Loadable Family" paradigm. The `in_place` flag lives on the `Component` message in the project — it has no `ComponentDef` and no `CatalogRef`.

### 14.2 Data Model

```protobuf
message ComponentDef {
  string id = 1;                        // e.g., "IPE240", "Elbow-LR-ASME-B16.9"
  string name = 2;                      // Human-readable name
  string category = 3;                  // Category path (e.g., "Structural/Steel/Beam")
  string description = 4;

  // Parameters
  repeated ParamDef params = 10;

  // Geometry: the parametric model definition
  // Uses the same sketch+constraint+feature pipeline as project objects
  // but with parameterized dimensions
  string feature_tree_id = 11;          // Reference to the feature tree

  // Connectors (connection points)
  repeated ConnectorDef connectors = 12;

  // Materials
  repeated string allowed_materials = 13;
  string default_material = 14;

  // Metadata
  string ifc_class = 20;
  string standard_reference = 21;       // e.g., "ASME B16.9", "EN 10025", "GB/T 11263"
  repeated string tags = 22;
  string domain = 23;                   // building / civil / plant / mechanical
  map<string, string> properties = 24;  // Additional metadata
  bool published = 25;                  // Publish flag (proto/omnim.proto, used by PUBLISH_COMPONENT)
}

message ParamDef {
  string name = 1;
  ParamType type = 2;
  double default_value = 3;
  double min_value = 4;                 // Optional domain constraint
  double max_value = 5;
  double step = 6;                      // Discrete step (e.g., 50mm for column sizes)
  string unit = 7;                      // mm / deg / ...
  string formula = 8;                   // Expression referencing other params
  repeated string choices = 9;          // For CHOICE type: "LR", "SR", "3D"
  string description = 10;
}

enum ParamType {
  PARAM_TYPE_UNSPECIFIED = 0;
  LENGTH = 1;             // mm
  ANGLE = 2;              // degrees
  INTEGER = 3;            // count
  CHOICE = 4;             // string selection
  BOOLEAN = 5;            // true/false
  RATIO = 6;              // dimensionless
}

message ConnectorDef {
  string id = 1;                        // e.g., "P1", "P2", "NOZZLE_A"
  string name = 2;

  // Position and direction are expressed as parameter expressions
  // e.g., "0,0,0" or "length/2, 0, 0" or "PARAM[1] * 0.5, 0, 0"
  string position_x_expr = 10;
  string position_y_expr = 11;
  string position_z_expr = 12;
  string direction_x_expr = 13;
  string direction_y_expr = 14;
  string direction_z_expr = 15;

  string bore_expr = 16;                // Connection bore/diameter expression
  string connection_type_code = 17;     // e.g., "FAA" (Flange-ASME-FlatFace), "BW" (ButtWeld)
  ConnectorRole role = 18;

  // Future: cross-platform encoding mapping
  // map<string, string> cross_reference = 19;  // { "civil3d_point_code": "ETW", "e3d_ppoint": "P1" }
}

> **Cross-platform encoding mapping**: A future enhancement will add a `cross_reference` field to `ConnectorDef` and a `ConnectorMapping` registry to map between Civil3D Point Codes, OmniM ConnectorType, and E3D P-Point identifiers. See `docs/research/genrule_design_analysis.md §2.5`.

> **ComponentDef parameter cross-reference** (from SOLIDWORKS, `docs/research/solidworks_components.md §3.5`): Hole Wizard ↔ Toolbox say a `ComponentDef` parameter can be constrained by another `ComponentDef`'s dimensions. Enable `ParamDef.formula` to reference a sibling component's param — e.g., a bolt-hole diameter `ComponentDef` whose `diameter` param is driven by a `${bolt_def.size}` reference (bolt size → hole diameter). This mirrors how Hole Wizard hole dimensions are linked to Toolbox fastener sizes.

enum ConnectorRole {
  CONNECTOR_ROLE_UNSPECIFIED = 0;
  INLET = 1;
  OUTLET = 2;
  BIDIRECTIONAL = 3;
  MOUNT = 4;                            // Mounting point (structural)
  BRANCH = 5;                           // Branch connection (tee)
}
```

### 14.3 Catalog Organization

```
ComponentLib
├── Piping
│   ├── Pipe-CS (Carbon steel pipe, schedule-based)
│   ├── Elbow-LR (Long radius elbow, R=1.5D)
│   ├── Elbow-SR (Short radius elbow, R=1.0D)
│   ├── Tee-Equal (Straight tee)
│   ├── Tee-Reducing (Reducing tee)
│   ├── Reducer-Conc (Concentric reducer)
│   ├── Reducer-Ecc (Eccentric reducer)
│   ├── Flange-WN-CL150 (Weld neck, Class 150)
│   ├── Flange-WN-CL300 (Weld neck, Class 300)
│   ├── Flange-SO-CL150 (Slip-on, Class 150)
│   ├── Gasket-SpiralWound
│   ├── Gasket-Ring
│   ├── BoltSet-CS (A193-B7 / A194-2H)
│   ├── Valve-Gate
│   ├── Valve-Globe
│   ├── Valve-Ball
│   ├── Valve-Check
│   ├── Valve-Butterfly
│   └── Cap (Pipe cap)
├── Structural
│   ├── Steel-Beam-IPE (IPE80..IPE600)
│   ├── Steel-Beam-HEA (HEA100..HEA900)
│   ├── Steel-Beam-HEB (HEB100..HEB900)
│   ├── Steel-Beam-UB (UK Universal Beam)
│   ├── Steel-Column-UC (UK Universal Column)
│   ├── Steel-CHS (Circular hollow section)
│   ├── Steel-SHS (Square hollow section)
│   ├── Steel-Angle (L-section)
│   ├── Concrete-Beam-Rect (Rectangular RC beam)
│   ├── Concrete-Column-Rect (Rectangular RC column)
│   ├── Concrete-Column-Circ (Circular RC column)
│   ├── Concrete-Slab (RC slab)
│   └── Concrete-Wall (RC wall)
├── Building
│   ├── Door-Single (Single leaf door)
│   ├── Door-Double (Double leaf door)
│   ├── Door-Sliding (Sliding door)
│   ├── Door-Fire (Fire-rated door)
│   ├── Door-Overhead (Overhead/sectional door)
│   ├── Window-Fixed (Fixed window)
│   ├── Window-Casement (Casement window)
│   ├── Window-Sliding (Sliding window)
│   └── Window-Bay (Bay window)
├── Equipment
│   ├── Pump-Centrifugal
│   ├── Pump-PositiveDisplacement
│   ├── Pump-Submersible
│   ├── Tank-Vertical (Atmospheric vertical tank)
│   ├── Tank-Horizontal (Horizontal pressure vessel)
│   ├── Tank-Sphere (Spherical storage)
│   ├── HeatExchanger-FloatingHead
│   ├── HeatExchanger-FixedTube
│   └── HeatExchanger-AirCooled
├── Civil
│   ├── Road-Asphalt (Asphalt pavement)
│   ├── Road-Concrete (Concrete pavement)
│   ├── Bridge-Girder (Simple girder bridge)
│   ├── Culvert-Box (Box culvert)
│   └── RetainingWall-Gravity (Gravity retaining wall)
└── Mechanical
    ├── Part-Gear (Spur gear)
    ├── Part-Shaft (Rotating shaft)
    ├── Part-Bearing (Rolling bearing)
    └── Part-Fastener (Bolt/nut/washer)
```

---

## 15. Series Library (系列库 / 等级库 / SPECON)

> **Independent module**: SeriesStore is an **independent module** — it defines spec decision trees that reference Library content by ComponentDef ID. Series can reside in three places: (1) project-private SeriesStore (embedded in `.omnim` file), (2) external `.omnimseries` files referenced by path, (3) built-in series seeded from code. **Only components reachable through the project's series can be used.** Any `create` or `instantiate` operation must first pass the series gate — if the target `ComponentDef` is not reachable from any loaded series, it is rejected.
>
> The default series mapping (`default_series_for_type()`, a session-level global function in the gen_rule module) suggests which series to load for a given domain, but the final series selection is a project configuration decision.

The Series Library defines project-approved subsets of the Component Catalog. A Series (spec) is a **decision tree** that selects which components are available, with constrained parameter ranges, for a given engineering context.

### 15.1 Core Concepts

| Concept | PDMS Equivalent | Description |
|---------|----------------|-------------|
| **Series** | SPEC (specification) | A named spec tree defining available components for a project or system |
| **Selector** | SELE (selector) | A decision node: Question → Answer → child Selector or SpecComponent |
| **SpecComponent** | SPCO (spec component) | A leaf node referencing a ComponentDef with parameter constraints |
| **Question** | QUES (question attribute) | The decision criterion (e.g., "generic_type?", "material?", "pressure_class?") |
| **Answer** | ANSW (answer attribute) | The matching value (e.g., "ELBO", "FLAN", "VALV") or numeric range |

### 15.2 Data Model

```protobuf
message Series {
  string id = 1;                        // e.g., "CS150", "Concrete-C30", "Steel-Q355"
  string name = 2;
  string description = 3;
  string domain = 4;                    // building / civil / plant / mechanical
  string standard_reference = 5;        // e.g., "ASME B31.3", "EN 1993", "GB 50017"

  Selector root_selector = 10;          // The decision tree root
  repeated string applicable_phases = 11; // lifecycle phases where this spec applies
  map<string, string> metadata = 12;
}

message Selector {
  string id = 1;
  string question = 2;                  // The question text (e.g., "generic_type?", "element_type?")
  repeated Answer answers = 3;
}

message Answer {
  string id = 1;
  string value = 2;                     // Text match (e.g., "ELBO", "wall") or numeric range "50..200"
  bool is_numeric_range = 3;            // If true, value is a range like "50..200"
  double range_min = 4;
  double range_max = 5;

  oneof outcome {
    Selector next_selector = 10;        // Continue narrowing down
    SpecComponent component = 11;       // Leaf: select this component
  }
}

message SpecComponent {
  string id = 1;
  string component_def_id = 2;          // Reference to ComponentDef

  // Parameter constraints: override or restrict the ComponentDef's parameters
  repeated ParamConstraint param_constraints = 10;

  // Material constraint
  string material = 11;                 // Allowed material (or empty = any)

  // Connection type filter
  string connection_type_code = 12;     // Filter by connection type (e.g., "BW", "RF")

  // Design parameters
  map<string, string> design_params = 13; // Additional design-time parameters
}

message ParamConstraint {
  string param_name = 1;                // Matches ParamDef.name
  double default_value = 2;             // Override default
  double min_value = 3;                 // Restrict domain
  double max_value = 4;
  double step = 5;                      // Restrict step
  string fixed_value = 6;               // For CHOICE type: fix to one option
}
```

### 15.3 Selection Algorithm

When a designer creates an object through a series:

1. Start at `root_selector`
2. Evaluate `question` against the design context
3. Find the first `answer` whose `value` matches
4. If the match is a `Selector`, go to step 2 (narrow down)
5. If the match is a `SpecComponent`, instantiate that component
6. Apply `param_constraints` to restrict the component's parameters
7. **Series Gate**: confirm the target `ComponentDef` is reachable from at least one loaded series in the project. If not → reject.
8. Look up the `ComponentDef` from the Library's `CatalogStore`
9. Instantiate the ComponentDef with the constrained parameters

The selection engine is context-aware: the first question is typically "generic_type?" (what kind of component?), and the system provides the answer based on what the designer is trying to place (e.g., elbow, flange, valve).

**Series Gate Behavior**:
- Series loaded + component approved → allowed
- Series loaded + component not in any series → rejected
- No series loaded → open mode (all catalog components allowed)

---

## 16. Series Tree Design by Domain

### 16.1 Building Domain

#### Series: "Cast-in-Place Concrete (C30)" (现浇混凝土结构)

```
Selector: "element_type?"
├── Answer: "wall"
│   └── Selector: "function?"
│       ├── Answer: "shear" → SpecComponent: Concrete-Wall-Shear
│       │   └── constraints: thickness=200..400, step=50, rebar=0.25%..0.4%
│       ├── Answer: "retaining" → SpecComponent: Concrete-Wall-Retaining
│       │   └── constraints: thickness=300..800, step=50
│       ├── Answer: "basement" → SpecComponent: Concrete-Wall-Basement
│       │   └── constraints: thickness=250..600, step=50
│       └── Answer: "infill" → SpecComponent: Concrete-Wall-Infill
│           └── constraints: thickness=150..250, step=50
├── Answer: "column"
│   └── Selector: "shape?"
│       ├── Answer: "rectangular" → SpecComponent: Concrete-Column-Rect
│       │   └── constraints: width=200..1000, depth=200..1000, step=50
│       └── Answer: "circular" → SpecComponent: Concrete-Column-Circ
│           └── constraints: diameter=300..1200, step=50
├── Answer: "beam"
│   └── Selector: "function?"
│       ├── Answer: "main" → SpecComponent: Concrete-Beam-Rect
│       │   └── constraints: width=250..800, depth=400..1200, step=50
│       └── Answer: "secondary" → SpecComponent: Concrete-Beam-Rect
│           └── constraints: width=200..600, depth=300..800, step=50
├── Answer: "slab"
│   └── Selector: "system?"
│       ├── Answer: "one_way" → SpecComponent: Concrete-Slab-1Way
│       │   └── constraints: thickness=100..250, step=10
│       ├── Answer: "two_way" → SpecComponent: Concrete-Slab-2Way
│       │   └── constraints: thickness=120..300, step=10
│       ├── Answer: "flat" → SpecComponent: Concrete-Slab-Flat
│       │   └── constraints: thickness=150..350, step=10
│       └── Answer: "waffle" → SpecComponent: Concrete-Slab-Waffle
│           └── constraints: thickness=300..600, step=50
├── Answer: "foundation"
│   └── Selector: "type?"
│       ├── Answer: "strip" → SpecComponent: Foundation-Strip
│       │   └── constraints: width=500..2000, depth=300..600
│       ├── Answer: "pad" → SpecComponent: Foundation-Pad
│       │   └── constraints: size=800..3000, step=100
│       └── Answer: "raft" → SpecComponent: Foundation-Raft
│           └── constraints: thickness=300..1000, step=50
└── Answer: "stair"
    └── Selector: "type?"
        ├── Answer: "straight" → SpecComponent: Stair-Straight
        └── Answer: "spiral" → SpecComponent: Stair-Spiral
```

#### Series: "Steel Structure (Q355)" (钢结构)

```
Selector: "element_type?"
├── Answer: "column"
│   └── Selector: "section?"
│       ├── Answer: "HEA" → SpecComponent: Steel-Column-HEA
│       │   └── constraints: size=HEA100..HEA900, material=Q355
│       ├── Answer: "HEB" → SpecComponent: Steel-Column-HEB
│       │   └── constraints: size=HEB100..HEB900, material=Q355
│       ├── Answer: "CHS" → SpecComponent: Steel-CHS
│       │   └── constraints: diameter=100..500, material=Q355
│       └── Answer: "SHS" → SpecComponent: Steel-SHS
│           └── constraints: size=100..400, material=Q355
├── Answer: "beam"
│   └── Selector: "section?"
│       ├── Answer: "IPE" → SpecComponent: Steel-Beam-IPE
│       │   └── constraints: size=IPE80..IPE600, material=Q355
│       ├── Answer: "HEA" → SpecComponent: Steel-Beam-HEA
│       │   └── constraints: size=HEA100..HEA900, material=Q355
│       └── Answer: "UB" → SpecComponent: Steel-Beam-UB
│           └── constraints: size=UB127..UB914, material=Q355
├── Answer: "bracing"
│   └── Selector: "section?"
│       ├── Answer: "angle" → SpecComponent: Steel-Angle
│       │   └── constraints: size=L50..L200, material=Q355
│       └── Answer: "CHS" → SpecComponent: Steel-CHS
│           └── constraints: diameter=50..200, material=Q355
└── Answer: "connection"
    └── Selector: "type?"
        ├── Answer: "end_plate" → SpecComponent: SteelConn-EndPlate
        ├── Answer: "fin_plate" → SpecComponent: SteelConn-FinPlate
        └── Answer: "base_plate" → SpecComponent: SteelConn-BasePlate
```

#### Series: "Masonry" (砌体结构)

```
Selector: "element_type?"
└── Answer: "wall"
    └── Selector: "function?"
        ├── Answer: "bearing" → SpecComponent: MasonryWall-Bearing
        │   └── constraints: thickness=190,290,390, height≤6000
        ├── Answer: "non_bearing" → SpecComponent: MasonryWall-NonBearing
        │   └── constraints: thickness=100,150,190, height≤4000
        └── Answer: "cavity" → SpecComponent: MasonryWall-Cavity
            └── constraints: outer=100, inner=100, cavity=50..150
```

#### Series: "Doors & Windows" (门窗系列)

```
Selector: "element_type?"
├── Answer: "door"
│   └── Selector: "type?"
│       ├── Answer: "single" → SpecComponent: Door-Single
│       │   └── constraints: width=700..1200, height=2000..2400
│       ├── Answer: "double" → SpecComponent: Door-Double
│       │   └── constraints: width=1200..2400, height=2000..2400
│       ├── Answer: "sliding" → SpecComponent: Door-Sliding
│       │   └── constraints: width=1500..3000, height=2000..2400
│       ├── Answer: "fire" → SpecComponent: Door-Fire
│       │   └── constraints: width=700..1200, height=2000..2400,
│       │       rating=30,60,90,120
│       └── Answer: "overhead" → SpecComponent: Door-Overhead
│           └── constraints: width=2400..6000, height=2400..5000
└── Answer: "window"
    └── Selector: "type?"
        ├── Answer: "fixed" → SpecComponent: Window-Fixed
        │   └── constraints: width=600..3000, height=600..2400
        ├── Answer: "casement" → SpecComponent: Window-Casement
        │   └── constraints: width=600..1800, height=600..1800
        ├── Answer: "sliding" → SpecComponent: Window-Sliding
        │   └── constraints: width=1200..4000, height=600..2400
        └── Answer: "bay" → SpecComponent: Window-Bay
            └── constraints: width=1500..4000, height=1500..2400
```

### 16.2 Civil Domain

#### Series: "Highway (GB JTG B01)" (公路)

```
Selector: "road_class?"
├── Answer: "expressway"
│   └── Selector: "design_speed?"
│       ├── Answer: "120" → SpecComponent: Road-Expressway-120
│       │   └── constraints: lanes=2..4, lane_width=3.75, median=2..3m
│       ├── Answer: "100" → SpecComponent: Road-Expressway-100
│       │   └── constraints: lanes=2..4, lane_width=3.75, median=2m
│       └── Answer: "80" → SpecComponent: Road-Expressway-80
│           └── constraints: lanes=2..4, lane_width=3.75, median=2m
├── Answer: "arterial"
│   └── Selector: "design_speed?"
│       ├── Answer: "60" → SpecComponent: Road-Arterial-60
│       │   └── constraints: lanes=2..6, lane_width=3.5
│       └── Answer: "40" → SpecComponent: Road-Arterial-40
│           └── constraints: lanes=2..4, lane_width=3.5
├── Answer: "collector" → SpecComponent: Road-Collector
│   └── constraints: lanes=2, lane_width=3.25..3.5
└── Answer: "local" → SpecComponent: Road-Local
    └── constraints: lanes=2, lane_width=3.0..3.25
```

### 16.3 Plant Domain (PDMS-Compatible Pipe Specs)

#### Series: "CS150 (Carbon Steel, Class 150)" (碳钢150磅级)

This is the most complex series tree, matching PDMS SPECON conventions:

```
Selector: "generic_type?"
├── Answer: "PIPE" → SpecComponent: Pipe-CS
│   └── constraints: DN=15..600, schedule=40,80,STD,XS, end=BW, material=A106-B
├── Answer: "ELBO"
│   └── Selector: "radius?"
│       ├── Answer: "LR" → SpecComponent: Elbow-LR-CS
│       │   └── constraints: DN=15..600, R=1.5D, end=BW, material=A234-WPB
│       ├── Answer: "SR" → SpecComponent: Elbow-SR-CS
│       │   └── constraints: DN=15..600, R=1.0D, end=BW, material=A234-WPB
│       └── Answer: "3D" → SpecComponent: Elbow-3D-CS
│           └── constraints: DN=50..400, R=3.0D, end=BW, material=A234-WPB
├── Answer: "TEE"
│   └── Selector: "type?"
│       ├── Answer: "equal" → SpecComponent: Tee-Equal-CS
│       │   └── constraints: DN=15..600, end=BW, material=A234-WPB
│       └── Answer: "reducing" → SpecComponent: Tee-Reducing-CS
│           └── constraints: run=15..600, branch=15..run, end=BW, material=A234-WPB
├── Answer: "FLAN"
│   └── Selector: "face?"
│       ├── Answer: "RF" → SpecComponent: Flange-WN-CL150-RF
│       │   └── constraints: DN=15..600, face=RaisedFace, material=A105
│       ├── Answer: "FF" → SpecComponent: Flange-WN-CL150-FF
│       │   └── constraints: DN=15..600, face=FlatFace, material=A105
│       └── Answer: "RTJ" → SpecComponent: Flange-WN-CL150-RTJ
│           └── constraints: DN=100..600, face=RingJoint, material=A105
├── Answer: "GASK" → SpecComponent: Gasket-CS150
│   └── Selector: "type?"
│       ├── Answer: "spiral" → SpecComponent: Gasket-SpiralWound-CS150
│       │   └── constraints: material=SS304/Graphite
│       ├── Answer: "ring" → SpecComponent: Gasket-Ring-CS150
│       │   └── constraints: material=SS304
│       └── Answer: "sheet" → SpecComponent: Gasket-Sheet-CS150
│           └── constraints: material=CompressedFiber
├── Answer: "BOLT" → SpecComponent: BoltSet-CS150
│   └── constraints: material=A193-B7/A194-2H
├── Answer: "VALV"
│   └── Selector: "valve_type?"
│       ├── Answer: "gate" → SpecComponent: Valve-Gate-CS150
│       │   └── constraints: DN=15..600, end=BW or RF, body=CS, trim=13Cr
│       ├── Answer: "globe" → SpecComponent: Valve-Globe-CS150
│       │   └── constraints: DN=15..300, end=BW or RF, body=CS, trim=13Cr
│       ├── Answer: "ball" → SpecComponent: Valve-Ball-CS150
│       │   └── constraints: DN=15..400, end=BW or RF, body=CS, seat=PTFE
│       ├── Answer: "check" → SpecComponent: Valve-Check-CS150
│       │   └── constraints: DN=15..400, end=BW or RF, type=Swing
│       └── Answer: "butterfly" → SpecComponent: Valve-Butterfly-CS150
│           └── constraints: DN=50..600, end=Wafer or Lug, body=CS, disc=SS304
├── Answer: "REDU" → SpecComponent: Reducer-CS
│   └── Selector: "type?"
│       ├── Answer: "concentric" → SpecComponent: Reducer-Conc-CS
│       │   └── constraints: DN=15..600, end=BW, material=A234-WPB
│       └── Answer: "eccentric" → SpecComponent: Reducer-Ecc-CS
│           └── constraints: DN=15..600, end=BW, material=A234-WPB
└── Answer: "CAP" → SpecComponent: Cap-CS
    └── constraints: DN=15..600, end=BW, material=A234-WPB
```

#### Series: "SS300 (Stainless Steel, Class 300)" (不锈钢300磅级)

```
Same structure as CS150 but:
- material=SS304/SS316 (A312-TP304, A403-WP304, A182-F304)
- pressure_class=300
- DN range: 15..400 (SS typically smaller range)
- gasket=SpiralWound (stainless/graphite)
- Connection type codes differ (e.g., RF for stainless typically)
```

### 16.4 Equipment Domain

#### Series: "Pump Standard" (泵标准系列)

```
Selector: "pump_type?"
├── Answer: "centrifugal" → SpecComponent: Pump-Centrifugal
│   └── constraints: flow=1..1000m³/h, head=10..200m, speed=1450/2900rpm
│       casing=CI/CS/SS, impeller=Bronze/SS
├── Answer: "positive_displacement" → SpecComponent: Pump-PD
│   └── constraints: flow=0.1..100m³/h, head=50..500m, type=Gear/Screw/Diaphragm
└── Answer: "submersible" → SpecComponent: Pump-Submersible
    └── constraints: flow=1..500m³/h, head=5..150m, motor=OilFilled/WaterFilled
```

#### Series: "Tank Standard" (储罐标准系列)

```
Selector: "tank_type?"
├── Answer: "vertical" → SpecComponent: Tank-Vertical
│   └── constraints: volume=1..10000m³, D/H=0.5..2, roof=Fixed/Floating/Dome
├── Answer: "horizontal" → SpecComponent: Tank-Horizontal
│   └── constraints: volume=1..500m³, L/D=2..6, ends=Elliptical/Hemispherical
└── Answer: "sphere" → SpecComponent: Tank-Sphere
    └── constraints: volume=100..50000m³, pressure=atmospheric..2MPa
```

---

## 16a. Feature Template System & Profile Library

The Feature Template System bridges the gap between **Component Catalog definitions** and **actual 3D geometry generation**. It defines how a ComponentDef's parameters map to concrete Sketch + Feature operations that produce BRep solids.

### 16a.1 Architecture

```
ComponentDef (catalog entry)
    │ params: height, width, length, ...
    │ feature_tree_id ──────────────────────┐
    ▼                                       ▼
FeatureTreeTemplate                     FeatureTreeRegistry
    │ features: [Extrude, Revolve, ...]      (named template store)
    │ param_bindings: {comp→feat}
    ▼
instantiate_template()
    │
    ├── profile::generate(ProfileType) → Vec<Point3D>
    │       Generates closed 2D profile polygon from parameters
    │       (I-section, box, circle, annulus, rectangle, L-shape)
    │
    ├── create_profile_sketch() → Sketch
    │       Wraps profile points into a solved Sketch entity
    │
    ├── Feature::new_extrude/revolve/...()
    │       Creates Feature from sketch + resolved parameters
    │
    └── project.rebuild_component()
            Executes Feature Engine (truck-modeling BRep)
            → tessellated MeshGeometry
```

### 16a.2 Profile Library

The `profile` module (`crates/model/src/profile/`) provides parameterized 2D cross-section generators:

| Profile Type | Function | Parameters | Use Case |
|-------------|----------|------------|----------|
| `IShape` | `i_section()` | height, width, web_thickness, flange_thickness | IPE, HEA, HEB beams |
| `BoxShape` | `box_section()` | width, height, thickness | SHS, RHS hollow sections |
| `Rectangle` | `rectangle()` | width, height | Concrete beams, columns |
| `Circle` | `circle()` | radius, segments | Solid bars, pipe OD |
| `Annulus` | `annulus()` | outer_radius, inner_radius, segments | Pipe cross-section |
| `LShape` | `l_section()` | leg1, leg2, thickness | Angle sections |
| `UShape` | `u_section()` | height, width, web_thickness, flange_thickness | U/C channels |
| `TShape` | `t_section()` | height, width, web_thickness, flange_thickness | T sections |
| `PipeSchedule` | `resolve_pipe_schedule_profile()` | DN, schedule, segments | ASME schedule-based pipe |
| `ParametricRect` | params-resolved rectangle | width_param, height_param | 2-point wall/slab creation |

### 16a.3 Feature Tree Template

```rust
pub struct FeatureTreeTemplate {
    pub id: String,
    pub name: String,
    pub features: Vec<FeatureTemplateStep>,
    pub param_bindings: HashMap<String, String>,  // comp_param → feat_param
}

pub struct FeatureTemplateStep {
    pub name: String,
    pub feature_type: FeatureType,      // Extrude / Revolve / Sweep / ...
    pub profile_type: Option<ProfileType>,
    pub aux_profile_type: Option<ProfileType>, // second profile (Loft profile1 / Sweep path)
    pub depth_expr: Option<String>,      // e.g., "length", "1.5 * DN"
    pub angle_expr: Option<String>,
    pub direction: Vector3D,
    pub is_cut: bool,
    pub boolean_op: Option<BooleanOp>,
    pub pattern_counts: Option<(usize, usize, usize)>,
}
```

### 16a.4 Parameter Resolution

Parameters are resolved through `resolve_param()` which supports:
- **Direct lookup**: `"length"` → `params["length"]`
- **Simple arithmetic**: `"1.5 * DN"` → `1.5 * params["DN"]`
- **Literal**: `"360"` → `360.0`

### 16a.5 ASME B36.10M Pipe Schedule Table

The `pipe_schedule` module provides ASME B36.10M wall thickness lookup:

```
pipe_schedule::wall_thickness(DN=100, schedule="40") → 6.02mm
pipe_schedule::outer_diameter(DN=100) → 114.3mm
```

Supports DN 15–600 and schedules 5S, 10S, 40, 80, 160. It also provides stainless variants (`wall_thickness_ss()`, `resolve_pipe_schedule_profile_ss()`), ASME B16.9 elbow center-to-face dimensions, and ASME B16.5 Class 150/300 flange tables (`crates/model/src/profile/pipe_schedule.rs`).

### 16a.6 Component Instantiation Flow

```
GUI: Select Steel-Beam-IPE from project's Available Components Panel, set height=300, length=6000

Server: POST /api/catalog/instantiate
{
  "component_def_id": "Steel-Beam-IPE",
  "instance_name": "my_beam",
  "params": { "height": 300, "length": 6000 },
  "position": { "x": 0, "y": 0, "z": 0 }
}

Backend:
  1. Look up ComponentDef in catalog
  2. Resolve FeatureTreeTemplate from feature_tree_id
  3. Merge default params + overrides
  4. Create Component object with catalog metadata
  5. For each FeatureTemplateStep:
     a. profile::generate(ProfileType, resolved_params) → profile points
     b. Create Sketch from profile points
     c. Create Feature (Extrude/Revolve/etc.) with resolved depth/angle
  6. rebuild_component() → BRep extrusion → tessellated mesh
  7. Return new component ID

Response: { "component_id": "...", "name": "my_beam" }
```

---

## 17. Object → Catalog Reference

When a project object is created from a catalog component through a series, it carries a reference to its origin:

```protobuf
message CatalogRef {
  string component_def_id = 1;          // Which ComponentDef
  string series_id = 2;                 // Which Series (spec)
  string selector_path = 3;             // Path through the decision tree
  map<string, double> param_values = 4; // Instance parameter values
}
```

This is stored on `OmniObject`:

```protobuf
message OmniObject {
  // ... existing fields ...
  optional CatalogRef catalog_ref = 20;  // Reference to catalog definition
}
```

This enables:
- **Traceability**: every object knows which catalog component and spec it came from
- **Regeneration**: changing a catalog component's definition updates all instances
- **Procurement**: directly generate BOM from spec components
- **Standards enforcement**: verify that all objects in a project conform to their specs

---

## 18. Administrative Division (行政分区)

Administrative Divisions model the geographic/political hierarchy in which a project is situated. This is distinct from the spatial coordinate system — it provides the *jurisdictional* context for permits, standards compliance, land use, and reporting.

### 18.1 Hierarchy

| Level | Standard | Examples |
|-------|----------|----------|
| **World** | — | Earth |
| **Country** (国家/地区) | ISO 3166-1 (alpha-2/alpha-3/numeric) | CN, US, GB, DE, JP |
| **Province/State** (省/州/邦) | ISO 3166-2, GB/T 2260 (省级) | Guangdong, California, Bavaria |
| **Prefecture** (地级市/地区) | GB/T 2260 (地级), local equivalents | Shenzhen, Osaka-fu, LA County |
| **County** (县/区/县级市) | GB/T 2260 (县级), local | Nanshan, Manhattan, Shinjuku |
| **Township** (乡/镇/街道) | GB/T 2260 (乡级) | Yuehai Subdistrict, Ginza |
| **Community** (社区/村) | local administration | OCT Community, Soho |
| **Block** (街区) | urban planning | Block C12, Lot 47 |
| **Land Parcel** (地块) | land registry | Parcel G2024-0015 |

### 18.2 Data Model

```protobuf
message AdministrativeDivision {
  string id = 1;                         // e.g., "CN/44/03/0305"
  string name = 2;                       // e.g., "Nanshan District"
  string local_name = 3;                 // e.g., "南山区" (native script)
  DivisionLevel level = 4;               // WORLD, COUNTRY, PROVINCE, PREFECTURE, COUNTY, TOWNSHIP, COMMUNITY, BLOCK, PARCEL
  string parent_id = 5;                  // Parent division ID
  optional string code_iso_3166_1 = 6;   // "CN"
  optional string code_iso_3166_2 = 7;   // "CN-GD"
  optional string code_gb_t2260 = 8;     // "440305" (GB/T 2260 county code)
  optional GeoBoundary boundary = 9;     // Geographic boundary (Polygon/MultiPolygon)
  map<string, string> attributes = 10;   // Population, area, timezone, etc.
}

enum DivisionLevel {
  WORLD = 0;
  COUNTRY = 1;
  PROVINCE = 2;
  PREFECTURE = 3;
  COUNTY = 4;
  TOWNSHIP = 5;
  COMMUNITY = 6;
  BLOCK = 7;
  PARCEL = 8;
}

> **Design specification vs. proto**: the message schema above is the target model. `proto/omnim.proto` currently defines `AdministrativeDivision { id, level, name, native_name, code, parent_id, boundary, properties }` with an `AdministrativeLevel` enum (UNSPECIFIED = 0, WORLD = 1 … PARCEL = 9, i.e. 1-based level numbers), and `GeoBoundary { boundary_type, coordinates: repeated Point3D, crs }` rather than the WKT-bytes form. The AD store (`crates/model/src/administrative.rs`, `/api/administrative*`) is implemented. Aligning the doc schema with proto is 🗓 Planned.

message GeoBoundary {
  // WGS 84 coordinates defining the boundary polygon(s)
  bytes wkt_polygon = 1;                // Well-Known Text representation
  string srid = 2;                      // "EPSG:4326"
}
```

### 18.3 Usage

- A **Project** is linked to one or more AdministrativeDivisions (e.g., site spans Nanshan + Futian)
- Administrative divisions provide **applicable standards** (building codes, environmental regs, fire safety)
- **Reporting** groups objects by administrative boundary
- **Land parcel** is the finest level — maps to real-estate registration

### 18.4 Scope: Reference Data Only

Administrative Division data is **outside the scope of AMIR and Omcom**. This is immutable reference data sourced from government gazetteers:
- **No Omcom verbs** exist for creating, editing, or deleting AD records
- **No AMIR intents** target administrative divisions
- AD data is loaded into the system by administrators through bulk import (CSV/GeoJSON from national statistics bureaus)
- Project objects reference AD via `ad_id` as read-only context for compliance and reporting

---

## 19. Project Breakdown Structure (项目分解结构)

The Project Breakdown Structure (PBS) defines the hierarchical decomposition of a project's scope of work. Unlike Administrative Division (geographic/political) or Spatial hierarchy (physical placement), PBS is a **management and engineering decomposition** that follows domain-specific conventions.

### Scope: Project Setup Only

PBS is managed through **project initialization and configuration**, not through modeling commands:
- **No `create` / `delete` / `modify` Omcom verbs** exist for PBS nodes
- **No AMIR intents** generate PBS structures
- PBS is defined by the project team during project setup (via config files or project wizard)
- Objects are **assigned** to PBS nodes automatically or via declarative rules (e.g., "all beams in storey L2 → PBS: Tower/L2/Structural-Zone")
- The PBS tree is read during modeling but never mutated by Omcom commands

### 19.1 Supported Breakdown Conventions

| Convention | Domains | Standard |
|------------|---------|----------|
| **Program → Project → Sub-project → Unit → System → Element** | General engineering | ISO 19650, PMI PMBOK |
| **Project → Building → Storey → Zone → Element** | Building | GB/T 51301-2018, UniClass |
| **Project → Area → Unit → System → Component** | Plant | ISO 81346, PDMS |
| **Project → Section → Segment → Element** | Civil (linear) | GB/T 51253-2017 |
| **Project → System → Sub-system → Assembly → Part** | Mechanical | STEP AP242 |

### 19.2 Data Model

```protobuf
message ProjectBreakdownNode {
  string id = 1;                         // e.g., "P1/BLDG-A/L2/Z3/BEAM-045"
  string name = 2;                       // "Main Building — Level 2 — Zone 3"
  string code = 3;                       // "A2.3.045" (domain shorthand)
  BreakdownLevel level = 4;              // PROGRAM, PROJECT, SUBPROJECT, UNIT, BUILDING, STOREY, ZONE, SECTION, SEGMENT, SYSTEM, ELEMENT, PART
  string parent_id = 5;                  // Parent PBS node
  string project_id = 6;                 // Root project reference
  BreakdownConvention convention = 7;    // GENERAL, BUILDING, PLANT, CIVIL, MECHANICAL
  repeated string tags = 8;              // Functional labels (e.g., "structural", "fire-zone-A", "HVAC-Z1")
  repeated string object_ids = 9;        // OmniObject IDs belonging to this node
  map<string, string> attributes = 10;   // Phase, contractor, budget code, etc.
}

> **Design specification vs. proto**: `proto/omnim.proto` currently defines `BreakdownNode { id, level, name, code, parent_id, convention, related_object_ids, administrative_division_ids, properties }` with `BreakdownLevel` (UNSPECIFIED = 0, PROGRAM = 1 … PART = 10) — no `SECTION`/`SEGMENT` levels and no `tags`/`attributes`/`project_id` fields. The breakdown store (`crates/model/src/breakdown.rs`, `/api/breakdown*`) is implemented. Adding the missing levels/fields is 🗓 Planned.

enum BreakdownLevel {
  PROGRAM = 0;
  PROJECT = 1;
  SUBPROJECT = 2;
  UNIT = 3;
  BUILDING = 4;
  STOREY = 5;
  ZONE = 6;
  SECTION = 7;
  SEGMENT = 8;
  SYSTEM = 9;
  ELEMENT = 10;
  PART = 11;
}

enum BreakdownConvention {
  GENERAL = 0;
  BUILDING = 1;
  PLANT = 2;
  CIVIL = 3;
  MECHANICAL = 4;
}
```

### 19.3 Domain Examples

**Building:**
```
Shanghai Tower (PROJECT)
├── Tower (BUILDING)
│   ├── L1-L10 (STOREY range)
│   │   ├── Core-Zone (ZONE)
│   │   │   ├── Wall-Core-N (ELEMENT)
│   │   │   └── Wall-Core-S (ELEMENT)
│   │   └── Perimeter-Zone (ZONE)
│   │       ├── Beam-G1-01 (ELEMENT)
│   │       └── Column-C1-01 (ELEMENT)
│   └── L11-L20 (STOREY range)
```

**Plant:**
```
Refinery-Expansion (PROJECT)
├── CDU-Unit (UNIT)
│   ├── Crude-Tower-System (SYSTEM)
│   │   ├── Tower-C103 (ELEMENT)
│   │   └── Pipe-P-101..150 (ELEMENT group)
│   └── Preheating-System (SYSTEM)
│       ├── HX-E201 (ELEMENT)
│       └── Pipe-P-201..230 (ELEMENT group)
└── VDU-Unit (UNIT)
```

**Civil (Linear):**
```
HSR-Line-3 (PROJECT)
├── Section-A (SECTION: km 0–45)
│   ├── Segment-A1 (SEGMENT: km 0–12)
│   │   ├── Bridge-B1 (ELEMENT)
│   │   └── Bridge-B2 (ELEMENT)
│   └── Segment-A2 (SEGMENT: km 12–28)
│       ├── Tunnel-T1 (ELEMENT)
│       └── Embankment-E1 (ELEMENT)
└── Section-B (SECTION: km 45–98)
```

---

## 20. Relationship Between AD, PBS, and Spatial Hierarchy

These three hierarchies serve distinct purposes and coexist in the data model:

| Hierarchy | Partition Criterion | Example | Granularity |
|-----------|--------------------|---------|-------------|
| **Administrative Division** | Jurisdiction / politics | Nanshan District, Shenzhen | Parcel → World |
| **Project Breakdown** | Management / engineering | Tower → Storey → Zone → Beam | Part → Program |
| **Spatial (containment)** | Physical placement | Building → Wall → Opening | Sub-object → Scene |

### 20.1 Cross-References

```protobuf
message OmniObject {
  // ... existing fields ...
  optional string ad_id = 21;            // Administrative Division (parcel or higher)
  repeated string pbs_ids = 22;          // PBS nodes this object belongs to (typically 1)
}

message Project {
  string id = 1;
  string name = 2;
  repeated string ad_ids = 3;            // Administrative Divisions the project spans
  string root_pbs_node_id = 4;          // Root of the project breakdown tree
  ProjectCoordinateSystem coord_system = 5;  // Coordinate system configuration
  // ... other fields ...
}
```

> **Design specification vs. implemented `Project`** (`crates/model/src/project.rs`): the implemented struct holds `name`, `objects` (ObjectStore), `materials`, `lifecycle`, `relations` (Containment/Connection), `administrative`, `breakdown`, `sketches`, `series` (SeriesStore), `external_series_refs`, and `measurements` — the AD links and PBS tree are held in dedicated stores rather than `ad_ids`/`root_pbs_node_id` fields, and there is no `coord_system` field yet (🗓 Planned, §21). There is no `Project` message in proto; projects are serde JSON.

### 20.2 Key Rules

1. **A Project belongs to 1+ Administrative Divisions** (its site footprint)
2. **Every OmniObject belongs to exactly 1 PBS leaf node** (the finest PBS level that covers it)
3. **An OmniObject's spatial parent may differ from its PBS parent** (e.g., a beam is spatially inside a storey, but its PBS parent may be a structural zone)
4. **AD is immutable reference data** (sourced from government gazetteers; no Omcom/AMIR operations)
5. **PBS is project-specific setup** (defined during project initialization; no modeling commands mutate it)
6. **Spatial containment is physical** (determined by geometric placement, can be auto-detected)
7. **AMIR and Omcom operate exclusively on engineering model content** (objects, sketches, features, parameters, catalog components) — never on AD or PBS structure

### 20.3 Digital Delivery Compliance

This three-hierarchy model satisfies the structure requirements of:

- **ISO 19650**: Information container breakdown (PBS maps to "Container" hierarchy)
- **GB/T 51301-2018**: Model unit breakdown levels (项目级→功能级→构件级→零件级)
- **GB/T 51253-2017**: BIM application for building lifecycle
- **China Digital Delivery Standards**: Administrative division codes in project metadata + hierarchical PBS for model organization

---

## 21. Coordinate System Model (坐标系模型)

> **Design specification (🗓 Planned)**: the CRS messages in this section are the target model — none of `CoordinateReferenceSystem`, `ProjectCoordinateSystem`, `LatLonHeight`, or `HeightSystem` exists in `proto/omnim.proto` or `crates/model` today, and `OmniObject.crs_id` is not yet implemented (§3). The rules below describe the intended behavior; CRS configuration is still project-setup design work.

OmniM supports a **two-tier coordinate system** — every project has exactly one coordinate system:

| Tier | Purpose | Type |
|------|---------|------|
| **Global CRS** (地理坐标系) | Real geographic location of the site | `GEOGRAPHIC` or `PROJECTED` (EPSG) |
| **Project CRS** (施工坐标系) | Local engineering coordinates used for all modeling | `ENGINEERING` (custom origin + rotation) |

All modeling operations work in **Project CRS**. When viewing across projects or importing/exporting, coordinates are transformed to/from Global CRS on the fly.

A project has **exactly one coordinate system** — sub-projects or sections share the same origin and CRS.

### 21.1 Data Model

```protobuf
// ============================================================
// Coordinate Reference System
// ============================================================
message CoordinateReferenceSystem {
  string id = 1;                         // "EPSG:4490" or custom UUID
  string name = 2;                       // "CGCS2000" / "WGS84" / "Site-Origin"
  CRSType type = 3;                      // GEOGRAPHIC, PROJECTED, ENGINEERING, VERTICAL
  optional int32 epsg_code = 4;          // 4490 (CGCS2000), 4326 (WGS84), etc.
  optional string definition = 5;        // PROJ string or WKT for custom CRS
}

enum CRSType {
  GEOGRAPHIC = 0;                        // Lat/Lon (e.g., WGS84, CGCS2000)
  PROJECTED = 1;                         // Projected (e.g., UTM, Gauss-Kruger)
  ENGINEERING = 2;                       // Local engineering/ construction coordinates
  VERTICAL = 3;                          // Height system / vertical datum
}

// ============================================================
// Project Coordinate System
// ============================================================
message ProjectCoordinateSystem {
  // --- Geographic (global) reference ---
  CoordinateReferenceSystem global_crs = 1;     // e.g., CGCS2000 (EPSG:4490)

  // --- Engineering (local) CRS ---
  CoordinateReferenceSystem project_crs = 2;    // ENGINEERING type, custom

  // --- Transformation: Project CRS → Global CRS ---
  // The geographic position of the project origin (0,0,0 in project CRS)
  LatLonHeight origin_geo = 3;
  double rotation = 4;                          // Degrees clockwise from true north
  double scale_factor = 5;                      // Typically 1.0

  // --- Height system ---
  HeightSystem height_system = 6;
  optional CoordinateReferenceSystem vertical_crs = 7;  // e.g., "EPSG:5730" (1985 NHD)
}

// ============================================================
// Geographic Position
// ============================================================
message LatLonHeight {
  double latitude = 1;                   // Degrees (WGS84)
  double longitude = 2;                  // Degrees (WGS84)
  double height = 3;                     // Meters in the project's height system
}

// ============================================================
// Height Systems
// ============================================================
enum HeightSystem {
  HEIGHT_UNSPECIFIED = 0;
  NATIONAL_1985 = 1;                     // 1985 National Height Datum (China)
  ELLIPSOIDAL = 2;                       // Ellipsoidal height (h)
  MEAN_SEA_LEVEL = 3;                    // Mean Sea Level
  LOCAL_DATUM = 4;                       // Local / custom vertical datum
}
```

### 21.2 Project CRS Definition

The project CRS is defined by:

1. **Choose Global CRS** — e.g., CGCS2000 (EPSG:4490) or WGS84 (EPSG:4326)
2. **Choose Height System** — e.g., 1985 National Height Datum
3. **Set project origin** — surveyed GPS coordinates of the site's (0,0,0) point
4. **Set rotation** — azimuth of the project's X-axis from true north (input or from survey file)
5. **Scale factor** — typically 1.0; can be adjusted for high-precision surveys

Origin and rotation can be:
- **Manually entered** by the user
- **Imported** from survey files (e.g., .csv, .txt with station coordinates)
- **Picked** from a GIS map (future)

### 21.3 Coordinate Transformations

```
┌─────────────────────────────────────────────────────────┐
│                    On-the-fly Display                     │
│                                                          │
│  User enters / sees coordinates in Project CRS           │
│                                                          │
│  ┌──────────┐    ┌───────────┐    ┌──────────────────┐ │
│  │ Project  │───→│ Coordinate │───→│ Global CRS       │ │
│  │ CRS      │    │ Service    │    │ (for display/    │ │
│  │ (local)  │←───│ (PROJ)     │←───│  export/report)  │ │
│  └──────────┘    └───────────┘    └──────────────────┘ │
│                                                          │
│  Display switch (e.g., CGCS2000 ↔ WGS84):               │
│  Project CRS → Source Global CRS → Target Global CRS    │
└─────────────────────────────────────────────────────────┘
```

**Key rules:**
- All object coordinates (`Transform`) are stored in **Project CRS**
- The Coordinate Service transforms **only the origin point and rotation** for display purposes
- No per-vertex geometric precision conversion — the displayed origin/rotation is converted so the user can relate to geographic coordinates
- Switching between global CRS variants (e.g., CGCS2000 ↔ WGS84) recalculates the displayed origin coordinates via PROJ's 7-parameter transformation
- Import/export to GIS formats uses the full PROJ pipeline

### 21.4 Built-in CRS Registry

| Name | EPSG | Type | Notes |
|------|------|------|-------|
| WGS 84 | 4326 | Geographic | Global standard (GPS) |
| CGCS2000 | 4490 | Geographic | China Geodetic Coord Sys 2000 (mandatory in China since 2018) |
| Beijing 1954 | 4214 | Geographic | Legacy (deprecated) |
| Xian 1980 | 4610 | Geographic | Legacy (deprecated) |
| Web Mercator | 3857 | Projected | Web mapping |
| UTM zone N | 326xx | Projected | xx = zone number |
| 1985 National Height Datum | 5730 | Vertical | China vertical datum |

Users can also define **custom CRS** via PROJ string or WKT.

### 21.5 Omcom / AMIR Scope

Coordinate system configuration is **project setup**, not a modeling operation:
- **No Omcom verbs** for CRS creation or transformation
- **No AMIR intents** target coordinate systems
- CRS is set during project initialization (config file or project wizard)
- A single `project set-coord` Omcom command may be provided for **initial setup only**, not for modeling workflows
