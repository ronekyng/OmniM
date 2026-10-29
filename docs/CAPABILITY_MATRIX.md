---
title: "OmniM Capability Matrix"
status: current
last_updated: 2026-10-27
applies_to: "web/, crates/server, crates/model, crates/ifc"
evidence: "audited against docs/reference/API.md, docs/specification/DOMAINS.md, docs/status/FORMAT_COMPAT.md, docs/status/CODE_DEVIATIONS.md and crates/ifc/src/lib.rs 2026-10-27"
---
# OmniM Capability Matrix

A modeler-facing, row-by-row snapshot of what actually works today. It is deliberately conservative: most rows are partial or planned. Use it to choose a workflow, then follow the linked document for the exact caveat.

> **How to read this matrix**
> - **Object type** is the semantic role (and IFC mapping), **not** necessarily the `object_type` of the instance you receive. Catalog placement (`/api/catalog/instantiate`) creates `ObjectType::Component` with a `catalog_ref`; the GenRule **layered** family now creates the **domain type** (`Wall`, `Slab`, …) with a `PathSweep` feature tree and `catalog_ref` (Wave 3, 2026-10-28). Still hard-coded mesh: GenRule rect quick-forms and roof 2-point profiles. Feature-tree geometry is the official path.
> - **Genesis** is the per-instance creation class (Surface / Linear / Point / Body) from [DATA_MODEL §1](specification/DATA_MODEL.md#1-overview), not a property inferred from the name. The `(objectType, class/form)` authorization matrix is the design contract; server-side enforcement is still pending ([ROADMAP](status/ROADMAP.md)).
> - **Proto status** follows the column in [DOMAINS](specification/DOMAINS.md): ✅ in `proto/omnim.proto`, 🔀 in proto but simplified, 🗓 planned-only (104 of 155 listed types are planned-only).
> - **Measure** and **Section** are viewport tools. Measurements are stored in `Project.measurements`, undoable and persisted. Section is a view clip; component section lines exist only for `Component` objects.
> - **Create via UI?** ❌ with Proto ✅ means API-only. 🗓 means neither UI nor API creation exists.

| Mark | Meaning |
|---|---|
| ✅ | Implemented and verified in the cited docs/code |
| 🔀 | Partial — the cell states what works and what does not |
| ❌ | Not implemented; no code path today |
| — | Not applicable to this object or workflow |
| 🗓 | Planned-only: design listed, no ObjectType/UI/API |
| ✅* | Works, but the geometry is flattened to a Mesh and the parametric history is no longer authoritative |

## 1. Building

| Object type | Genesis | Create via UI | Parameters / series | Move / transform | Host / openings | Connect | Measure | Section | Clash | Quantity | Drawing | IFC in / out | Proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **Wall** | Surface (panel; region/Body/Linear by matrix) | ✅ Create panel or 建筑 → 墙体; Path open, ≥2 picks, Enter | ✅ LayerEditor layers in mm; series optional; vertical extent via `bottom/top_level_id` — create-panel selectors (P8-S3), single-step cascade (P8-S2) | ✅* | ✅ hosts door/window | 🔀 API only | ✅ view | ✅ view | ❌ | ❌ | ❌ | ✅ / ✅ (IfcWall) | ✅ |
| **Slab** | Surface (region) | ✅ 建筑 → 楼板; Path closed, ≥3 picks | ✅ LayerEditor layers in mm; series optional | ✅* | — | — | ✅ view | ✅ view | ❌ | ❌ | ❌ | ✅ / ✅ (IfcSlab) | ✅ |
| **Roof** | Surface (footprint region or 2-point profile) | ✅ 建筑 → 屋顶; 轮廓 or 坡屋面 modes | ✅ LayerEditor layers in mm | ✅* | — | — | ✅ view | ✅ view | ❌ | ❌ | ❌ | ✅ / 🔀 (exports proxy) | ✅ |
| **Floor** | Surface (region) | ❌ no menu entry; `/api/genrule` `mode:"floor"` only | ✅ layers in mm via API | ✅* | — | — | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **Door** | — (hosted opening) | ✅ Create panel; requires an existing Wall | 🔀 opening width/height + offset only | ✅* | ✅ `wall_id` required | — | ✅ view | ✅ view | ❌ | ❌ | ❌ | ✅ / ✅ (IfcDoor) | ✅ |
| **Window** | — (hosted opening) | ✅ Create panel; requires an existing Wall | 🔀 opening width/height + sill only | ✅* | ✅ `wall_id` required | — | ✅ view | ✅ view | ❌ | ❌ | ❌ | ✅ / ✅ (IfcWindow) | ✅ |
| **Beam** | Linear | ✅ catalog path (creates Component); dialog falls back to box | ✅ catalog params in mm; series gate 🔀 | ✅* | — | 🔀 API only | ✅ view | ✅ view | ❌ | ❌ | ❌ | ✅ / ✅ (IfcBeam) | ✅ |
| **Column** | Linear | ✅ catalog path (creates Component); dialog falls back to box | ✅ catalog params in mm; series gate 🔀 | ✅* | — | 🔀 API only | ✅ view | ✅ view | ❌ | ❌ | ❌ | ✅ / ✅ (IfcColumn) | ✅ |
| **Stair** | Body (primitive placeholder) | 🔀 no stair generator; dialog creates a primitive box | 🔀 primitive dimensions | ✅* | — | — | ✅ view | ✅ view | ❌ | ❌ | ❌ | ✅ / 🔀 (exports proxy) | ✅ |
| **Duct** | Linear | 🔀 quick action/catalog only (Component); dialog creates a box | ✅ catalog params in mm | ✅* | — | 🔀 API only | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **CableTray** | Linear | 🔀 quick action/catalog only (Component) | ✅ catalog params in mm | ✅* | — | 🔀 API only | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **Ceiling, CurtainWall, Ramp** | — (planned) | 🗓 | — | — | — | — | — | — | ❌ | ❌ | ❌ | ❌ / ❌ | 🗓 |
| **Footing, Pile, Brace, Truss, Rebar, Connection** | — (planned) | 🗓 | — | — | — | — | — | — | ❌ | ❌ | ❌ | 🔀 (Footing→Generic) / 🔀 | 🗓 |

## 2. Civil

| Object type | Genesis | Create via UI | Parameters / series | Move / transform | Host / openings | Connect | Measure | Section | Clash | Quantity | Drawing | IFC in / out | Proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **Road** | Linear | ✅ 土木 → 道路; Path open, N picks, per-segment mesh | ✅ width/thickness in m; series is metadata only | ✅* | — | — | ✅ view | ✅ view | ❌ | ❌ | ❌ | ✅ / 🔀 (exports proxy) | 🔀 |
| **Bridge** | Linear | 🔀 桥梁 creates a 2-point primitive placeholder | 🔀 primitive dimensions only | ✅* | — | — | ✅ view | ✅ view | ❌ | ❌ | ❌ | ✅ / 🔀 (exports proxy) | 🔀 |
| **Tunnel** | Linear | 🔀 隧道 creates a 2-point primitive placeholder | 🔀 primitive dimensions only | ✅* | — | — | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | 🔀 |
| **FeatureLine** | Point (transitional thin tube) | ✅ 土木 → 特征线; Path open, N picks | 🔀 fixed Ø0.05 m; no style/elevation model | ✅* | — | — | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **ContourLine** | Point (transitional thin tube) | ✅ 土木 → 等高线; Path open, N picks | 🔀 fixed Ø0.05 m; no constant-elevation rule | ✅* | — | — | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **Manhole** | Body | ✅ 土木 quick action/catalog (Component) | ✅ catalog params in mm | ✅* | — | 🔀 API only | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **RetainingWall** | Linear (alignment + section by design) | ❌ API only; no retaining-wall generator | — | ✅* | — | — | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **Dam** | Linear | ❌ API only | — | ✅* | — | — | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **Culvert** | Linear | ❌ API only | — | ✅* | — | — | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **Pier, Abutment** | Body / Assembly | ❌ API only | — | ✅* | — | — | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **TerrainModel, PointCloud, GradingSurface, Slope** | Point | 🗓 | — | — | — | — | — | — | ❌ | ❌ | ❌ | ❌ / ❌ | 🗓 |
| **Alignment, Profile, Station, Lane, Shoulder, PavementLayer, Median, Curb** | — (planned) | 🗓 | — | — | — | — | — | — | ❌ | ❌ | ❌ | ❌ / ❌ | 🗓 |
| **CutVolume, FillVolume, EarthworkBalance** | — (planned) | 🗓 | — | — | — | — | — | — | ❌ | ❌ | ❌ | ❌ / ❌ | 🗓 |

> FeatureLine and ContourLine have proto IDs 501/502 and a working toolbar path, even though DOMAINS groups the Terrain/Site block as planned-only. Treat the row as "implemented thin guide tube, not native TIN/breakline geometry".

## 3. Plant

| Object type | Genesis | Create via UI | Parameters / series | Move / transform | Host / openings | Connect | Measure | Section | Clash | Quantity | Drawing | IFC in / out | Proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **Pipe** | Linear | ✅ 工厂 → 管道 or Create; 2 picks; catalog path creates Component | ✅ DN/length params in mm or radius/length in m; series gate 🔀 | ✅* | — | 🔀 API only | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **Tank** | Body | ✅ 工厂 quick action/catalog (Component) | ✅ catalog params in mm; series gate 🔀 | ✅* | — | 🔀 API only | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **Pump** | Body | ✅ 工厂 quick action/catalog (Component) | ✅ catalog params in mm; series gate 🔀 | ✅* | — | 🔀 API only | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **Valve** | Body | ✅ 工厂 quick action/catalog (Component) | ✅ catalog params in mm; series gate 🔀 | ✅* | — | 🔀 API only | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **HeatExchanger** | Body | ✅ 工厂 quick action/catalog (Component) | ✅ catalog params in mm | ✅* | — | 🔀 API only | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **Reactor** | Body | ❌ API only | — | ✅* | — | 🔀 API only | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **Compressor** | Body | ✅ 工厂 quick action/catalog (Component) | ✅ catalog params in mm | ✅* | — | 🔀 API only | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **Instrument** | Body | ❌ API only | — | ✅* | — | 🔀 API only | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **StructuralSteel** | Linear | ❌ API only | — | ✅* | — | 🔀 API only | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **Fitting** | Body | ❌ API only | — | ✅* | — | 🔀 API only | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **PipeSegment, Elbow, Tee, Reducer, Flange, Nozzle** | — (planned) | 🗓 | — | — | — | — | — | — | ❌ | ❌ | ❌ | ❌ / ❌ | 🗓 |
| **ProcessLine, System, Unit, Area, PipingSpec, EquipmentSpec, ControlValve, Support** | — (planned) | 🗓 | — | — | — | — | — | — | ❌ | ❌ | ❌ | ❌ / ❌ | 🗓 |

## 4. Mechanical

| Object type | Genesis | Create via UI | Parameters / series | Move / transform | Host / openings | Connect | Measure | Section | Clash | Quantity | Drawing | IFC in / out | Proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **Part** | Body | ✅ Create panel primitive, or catalog path (Component) | 🔀 primitive dims or catalog params in mm | ✅* | — | — | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **Assembly** | Body | ✅ Assy toolbar → AssemblyPanel (GenRule assembly) | 🔀 child primitive dims + offsets (m) | ✅* | — | — | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **Gear** | Body | ✅ 机械 quick action/catalog (Component) | ✅ catalog params in mm | ✅* | — | — | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **Shaft, Bearing, Spring, Seal, Motor** | Linear / Body | ❌ API only | — | ✅* | — | — | ✅ view | ✅ view | ❌ | ❌ | ❌ | ❌ / 🔀 (exports proxy) | ✅ |
| **Fasteners (Bolt, Nut, Washer, Screw, Rivet, Pin)** | — (planned) | 🗓 | — | — | — | — | — | — | ❌ | ❌ | ❌ | ❌ / ❌ | 🗓 |
| **Joint, Constraint, Axis, KinematicPair, Actuator** | — (planned) | 🗓 | — | — | — | — | — | — | ❌ | ❌ | ❌ | ❌ / ❌ | 🗓 |
| **Hole, Pocket, Slot, Chamfer, Fillet, Thread, PMI** | — (planned object types) | 🗓 | — | — | — | — | — | — | ❌ | ❌ | ❌ | ❌ / ❌ | 🗓 |

> Generic `chamfer` and `fillet` **features** exist in the component feature tree (`/api/components/{id}/features`); the manufacturing object types above are separate planned concepts.

## 5. Spatial

| Object type | Genesis | Create via UI | Parameters / series | Move / transform | Host / openings | Connect | Measure | Section | Clash | Quantity | Drawing | IFC in / out | Proto |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **Site** | — (container) | ✅ Create panel object-type list | 🔀 name/IFC class only | — | — | — | — | — | ❌ | ❌ | ❌ | ✅ / ✅ (IfcSite) | ✅ |
| **Building** | — (container) | ✅ Create panel object-type list | 🔀 name/IFC class only | — | — | — | — | — | ❌ | ❌ | ❌ | ✅ / ✅ (IfcBuilding) | ✅ |
| **BuildingStorey** | — (container) | ✅ Create panel ("storey"); elevation input (P8-S1) | ✅ datum `elevation` (m, IfcBuildingStorey.Elevation); 🔀 `level_id` attachment is API-only (`POST/PUT /api/objects`); vertical extent via `bottom_level_id`/`top_level_id` ✅ for path-driven objects (single-step cascade, P8-S2), 🔀 plain geometry gets the datum only | — | — | — | — | — | ❌ | ❌ | ❌ | ✅ / ✅ (IfcBuildingStorey, native Elevation attribute, P8-S3) | ✅ |
| **轴网 Grid (datum)** | — (datum annotation, not an object) | ❌ API only (`/api/grids`) | ✅ label/axis/coordinate CRUD, per-family unique labels, undoable | — | — | — | — | — | ❌ | ❌ | ❌ | ❌ / 🗓 (IfcGrid planned) | ✅ |
| **Space** | — (container; primitive box) | ✅ Create panel | 🔀 primitive dimensions | — | — | — | — | — | ❌ | ❌ | ❌ | ✅ / ✅ (IfcSpace) | ✅ |
| **Zone** | — (planned) | 🗓 | — | — | — | — | — | — | ❌ | ❌ | ❌ | ❌ / ❌ | 🗓 |

## 6. Coverage Notes and Verification

- **Domain totals** ([DOMAINS](specification/DOMAINS.md), [COMPONENT_INVENTORY](research/COMPONENT_INVENTORY.md)): **155 listed / 51 in proto / 104 planned-only**. Building 37 (22 in proto), Civil 41 (10), Plant 39 (11), Mechanical 38 (8).
- **Server surface**: **83 deduplicated REST routes** ([API](reference/API.md); 2026-10-29 P8-S1 +4 grid-datum endpoints); **no WebSocket, no database**. One in-memory `Project`, optional JSON file.
- **Persistence**: on by default (2026-10-28) — `$OMNIM_PROJECT`/`project.omnim`, escape hatch `OMNIM_EPHEMERAL=1`. All mutating endpoints save, including transform/parent/connect, materials, measurements, series/catalog edits, instantiation and IFC import.
- **Undo**: every mutating endpoint is undoable, including transform, properties, parent/connect, material, catalog instantiation, IFC import and series edits ([WRITE_PATHS](architecture/WRITE_PATHS.md)).
- **IFC**: IFC 2x3 / IFC 4 subset only, hand-written parser/writer; no material, quantity or classification mapping. Stair and Roof import with their type but export as `IfcBuildingElementProxy`; all non-building types export as proxy ([FORMAT_COMPAT](status/FORMAT_COMPAT.md)).
- **Clash, quantity take-off and drawing generation do not exist** at any layer — no UI, no REST route, no crate. They are roadmap items ([ROADMAP](status/ROADMAP.md)).
- **Series gate**: reachability only. `min/max/fixed` constraints are not enforced and `SpecComponent.material` is not applied; `connect()` validates only that both objects exist ([CODE_DEVIATIONS §6.4](status/CODE_DEVIATIONS.md)).
- Companion task guide: [MODELER_HANDBOOK](MODELER_HANDBOOK.md). Source-of-truth hierarchy: `status/` and `architecture/` describe what is built; `specification/` and `RFC/` describe design intent.
