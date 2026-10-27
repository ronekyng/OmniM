# OmniM Development Roadmap

**Last updated: 2026-10-27**

> ### Current Phase: Phase 8 — Domain Extensions & Production-Grade Components
>
> Completed: Phases 1–7 (Phase 1 partially — see note below). Phase 9 partially delivered in advance (AMIR AI engine).
>
> ### Honest-status note
>
> Earlier revisions of this document marked the Omcom **text DSL / nom parser** as part of the completed Phase 1. That is **not accurate**: what is shipped today is the unified data model, the typed `OmniCommand` schema, and a full typed command executor. The Omcom console in the Web UI executes **JSON commands plus a handful of ad-hoc text shortcuts** — there is **no nom-based text grammar parser**. The text DSL is a draft RFC: [RFC-0001: Omcom Text DSL](../RFC/README.md). Phase 1 is therefore marked **🔀 Partial**.

## Status legend

| Mark | Meaning |
|------|---------|
| ✅ Done | Implemented and verified in the codebase |
| 🔄 In progress | Active work, some deliverables landed |
| 🔀 Partial | Substantially delivered, with specific named gaps |
| 📅 Planned | Not started |

---

## Phase 1: Core Data Model & Omcom DSL (2026 Q1-Q2) — 🔀 Partial

**Goal**: Establish the foundational object model, the typed command/command-executor layer, and the Omcom text DSL.

### Delivered ✅

1. **Unified Data Model** — `OmniObject`, core hierarchy (SpatialElement / Element / System), Property + PropertySet system, material definitions, ObjectType enum covering all 4 domains. Protobuf/serde types are used as the internal schema (`.proto` files act purely as the serde-JSON type declaration source; **no protobuf runtime/wire encoding is used**).
   *Evidence: `crates/model/` (incl. `crates/model/src/proto/`), `crates/core/`.*
2. **Typed command layer** — `OmniCommand` message types + a full typed command executor with complete undo/redo.
   *Evidence: `crates/omcom/src/lib.rs`, executor in `crates/server/src/lib.rs`.*
3. **Project lifecycle model**, **administrative division hierarchy** (World→…→Parcel, GeoBoundary), **project breakdown structure** (Program→…→Part), and the **relationship system** (containment / connection / spatial).
   *Evidence: `crates/model/src/`.*

### Not delivered 📅

- **Omcom text DSL with nom parser** (`crates/omcom-parser`, EBNF grammar, text→OmniCommand mapping, usage-hint errors). The interactive "console" is JSON + a few ad-hoc shortcuts only.
  *Design proposal: [RFC-0001: Omcom Text DSL](../RFC/README.md) (Draft).*

### Exit criteria (revised)
- ✅ Data model + typed executor handle create/command flows for all object types
- ✅ Undo/redo of every executed command
- 📅 Text grammar parses all `create wall/beam/…` commands (blocked on RFC-0001 acceptance)

---

## Phase 2: Constraint Solver & Sketch System (2026 Q2-Q3) — ✅ Done

- **Sketch data model**: SketchPlane, SketchEntity (line/arc/circle/spline), 11 constraint types, SketchStore in Project.
- **Constraint solver**: slvs-based solver integrated (crate: `slvs`); sketch→solve→solved-sketch pipeline, DOF analysis, over/under-constrained detection and reporting.
- **Omcom (JSON) sketch commands**: `sketch new/add/constr/solve`, `component create`, `feature add/remove`, `EXTRUDE_SKETCH (107)` / `REVOLVE_SKETCH (108)` — all with executor undo/redo.
- **Sketch Editor frontend**: `/sketch-editor` page, 2D canvas with grid/axes/entities, Line/Circle/Arc tools, constraint toolbar (11 kinds), Solve button, Extrude/Revolve dialogs, round-trip navigation to the main view.

### Evidence
`crates/sketch/`, solve pipeline in `crates/server/src/lib.rs` (`/api/sketches/*`), `web/src/app/sketch-editor/`.

---

## Phase 3: Feature Engine & BRep Kernel (2026 Q3-Q4) — ✅ Done

- **Feature as first-class entity**: cached MeshGeometry, dirty-flag automatic rebuild, feature dependency ordering, rollback & suppression.
- **9 feature types**: Extrude, Revolve, Sweep, Loft, Boolean (union/subtract/intersect), Chamfer, Fillet, Pattern, Shell.
- **truck-BRep kernel** (`truck-modeling` / `truck-polymesh`): extrude/revolve/sweep/loft from sketches, booleans on BRep solids, tessellation to triangle meshes for rendering.
- **Parameter system**: named parameter store, meval expression engine, feature parameter references.
- **Component Editor UI (Revit-style)**: Feature Panel with drag-to-reorder, Feature Ribbon (9 type icons), click-feature→auto-sketch→draw→Done→dialog workflow, AddFeatureDialog, Feature Card double-click editing, `+ Component` button, object-tree "Edit" context menu, Rebuild All button.

### Evidence
`crates/model/` (BRep→mesh; the legacy `crates/renderer/` demo crate was removed 2026-10-27), feature engine in `crates/server/src/lib.rs` + `crates/model/`, `web/src/app/components/component-editor/`.`

---

## Phase 4: Component Library & Series Library (2027 Q1) — ✅ Done

PARAGON-style **component catalog** + SPECON-style **series library**, delivered ahead of the original schedule:

- **ComponentDef** with ParamDef (Length/Angle/Integer/Choice + defaults/domains/formulas), ConnectorDef (position/direction expressions, bore, type codes), Category hierarchy.
- **Series**: selector decision trees (Question→Answer→Selector/SpecComponent), SpecComponent parameter constraints, numeric range matching (e.g. DN=50..200).
- **Selection engine** and **connector type-code compatibility** ("FAA", "BW", …).
- Catalog/series stored in project or external `.omnimlib` / `.omnimseries` JSON files.

### Evidence
`crates/model/src/catalog.rs`, `crates/model/src/series_store.rs`.

---

## Phase 5: Catalog Component Modeling (2027 Q2) — ✅ Done

- **Structural**: IPE/HEA/HEB/CHS/SHS/Angle families, parameterized profile sketch → extrude.
- **Piping**: straight pipe, long-radius elbow, tee, reducers, weld-neck flange, valves — with P-point connectors and type codes.
- **Building**: concrete beam/column/wall, door, window templates.
- **Series definitions**: CS150, SS300, Concrete-C30, Steel-Q355, Highway-GB.
- **Infrastructure**: profile section library (I/box/annulus/rect/L), ASME B36.10M wall-thickness table, FeatureTemplate + FeatureTreeRegistry, instantiation pipeline (ComponentDef→Feature→Sketch→BRep→Mesh), REST endpoints `POST /api/catalog/instantiate`, `POST /api/catalog/preview`.

### Evidence
Built-in catalog seed data + `instantiate_from_series()` in `crates/model/`; handlers in `crates/server/src/lib.rs`.

---

## Phase 6: GenRule Refactoring & Default Templates (2027 Q3) — ✅ Done

- **Library ↔ SeriesStore ↔ Project split**: Library is a session-level global static (`static LIBRARY: RwLock<Library>`); SeriesStore is an independent module referenced by Project; Project holds objects + private SeriesStore + `external_series_refs`, no Library field.
- **Series gate** `check_series_gate()`: reject components not reachable from any loaded series; open mode when no series loaded.
- **GenRule → catalog**: all GenRule generators (LayerExtrusion, LinearComponent, Assembly, BooleanSubtract, Primitive) route through feature trees referencing catalog ComponentDefs. `POST /api/genrule` uses `create_object_from_default_template()` / `instantiate_from_series()`.
- **CatalogRef** on every instantiated object (`component_def_id`, `series_id`, `selector_path`, `param_values`).
- **Default template registry**: `create wall`→Concrete-Wall, `create beam`→Concrete-Beam-Rect, etc.; type-parameter validation against catalog domains.

### Phase 6b (Library & SeriesStore infrastructure) — ✅ Done
`.omnimlib` / `.omnimseries` JSON file load/save, series resolution priority (project-private > external file > built-in seed), session auto-load from `~/.omnim/library.omnimlib`.

### Evidence
`crates/model/src/gen_rule/mod.rs`, `crates/model/src/library.rs`, `crates/model/src/series_store.rs`.

---

## Phase 7: User Interface (2027 Q4) — ✅ Done (one noted gap)

- **Project Workspace**: 3D viewport (orbit/pan/zoom, selection, section cuts, measure tools, wireframe/shaded/x-ray/edge-only view modes), sketch editor, feature tree, Available Components Panel (series-filtered, parameter sliders, drag-to-place), Property Panel (catalog ref, material), Omcom Console (syntax highlighting, history, tab completion, validation errors), AI chat (natural language).
- **Library Manager** at `/library-manager`: ComponentDef tree, parameter definition editor, feature-tree template assignment, 3D preview, `.omnimlib` import/export.
- **SeriesStore Manager** at `/series-manager`: decision-tree editor, built-in seed viewer, external `.omnimseries` load/save, drag-to-map ComponentDefs, parameter constraint editor.
- **UI architecture**: Zustand store, React Query data layer, PanelRegistry panel slots, resizable AppLayout, CSS Modules, `ThreeViewerContext`.

### Known gap 🔀
Interactive 3D creation supports 1–2 point picking for `PICKING_TYPES` only. N-point polygon/polyline picking (needed for slab/roof/floor boundaries, featureLine, contourLine) is **not implemented** — see [CODE_DEVIATIONS.md](CODE_DEVIATIONS.md) §3/§5.

(Earlier versions listed "command preview before application" here; the AI command preview/accept workflow was **not** delivered and remains a Phase-9 item that itself only partially exists today.)

---

## Phase 8: Domain Extensions & Production-Grade Components (2028 Q1-Q2) — 🔄 In progress

**Current status**: production-grade catalog data largely landed (steel sections per EN 10365/GB/T 706, pipe schedules per ASME B36.10, concrete templates, connector definitions with flange ratings); material assignments mapped to real standards; components validated through BRep correctness checks. Domain-specific work remains:

1. 📅 Building: complete IFC element coverage, MEP systems, quantity take-off
2. 📅 Civil: terrain fitting & earthwork, alignment/profile editing
3. 📅 Plant: pipe routing with branches, P&ID integration
4. 📅 Mechanical: parts/assemblies, kinematic joints
5. 📅 Frontend: domain-specific panels, filters, domain-aware drag-to-place

Partial exit criteria already met (steel IPE/HEA/HEB series, CS150/SS304 DN coverage, BRep validation).

---

## Phase 9: AMIR AI Compilation (2028 Q3) — 🔄 Partial (delivered in advance, gaps remain)

**Delivered** ✅: AMIR AI engine with OpenAI / Anthropic / Mock providers; NL→OmniCommand translation; multi-turn chat; reverse "describe model" path. Integrated as the AI chat panel in the Project Workspace.

**Not yet delivered** 📅: structured multi-step planning for complex operations, natural-language validation error explanation with auto-correction suggestions, confidence scoring, human-in-the-loop accept/reject command preview, per-domain multi-agent system, RAG for standards lookup.

### Evidence
`crates/amir/` (providers, translate, chat, describe), `web/` AI chat panel.

---

## Phase 10: Cloud Server & Collaboration (2029) — 📅 Planned

**Reality today** ✅ (subset of this phase, done early): an **Axum REST server** (~100 routes) with **JSON-file (`.omnim`) persistence**, full undo/redo state history, and catalog/series file endpoints.

**Not implemented** 📅: PostgreSQL + PostGIS storage, version history / audit trail (currently only JSON file save), WebSocket real-time notifications, geometry streaming, client cache layers, multi-user collaboration/Conflict resolution, presence, collaboration UI. There is **no WebSocket anywhere in the server**.

---

## Phase 11: IFC/STEP Format Compatibility (2029) — 🔀 Partial

- **IFC 2x3 / IFC 4**: import + export ✅ via a **hand-written parser/writer** (`crates/ifc/`) — **partial entity coverage**. Details in [FORMAT_COMPAT.md](FORMAT_COMPAT.md).
- **STEP / DXF / DWG / GIS / glTF / OBJ / FBX**: ❌ not implemented (code contains only IFC + JSON paths). Plan in [FORMAT_COMPAT.md](FORMAT_COMPAT.md).

---

## Phase 12: Analysis & Ecosystem (2029+) — 📅 Planned

Not started: structural analysis, clash detection, cost estimation, plugin ecosystem.

## Technical-Debt Backlog

Unplanned-widow items recorded during development; pick up opportunistically, no scheduled phase:

- **Test isolation refactor（测试隔离重构）** — model tests share the session-level global `static LIBRARY` (see [TESTING guide → Known Flaky Tests](../guides/TESTING.md)). One intermittent parallel-test failure has been observed (`test_find_and_rebuild_catalog_instances`); not reproducible under stress, but the shared-global dependency is a real risk as the test surface grows. Candidate fixes listed in the guide.
- **Logging standardization（日志规范化）** — replace ad-hoc `eprintln!` in production paths with a structured logging crate (`tracing` or `log`). Current residual prints are error-path/startup probes only.

---

## Non-Goals (Current Scope)

1. **Photorealistic rendering**
2. **Offline-first architecture**
3. **Mobile AR/VR / mobile apps** (no mobile app is planned or built)
4. **Full CAM/CNC capability**
5. Also explicit non-goals right now: desktop packaging (Tauri), WASM-shared core in the production web build, nom text DSL execution (pending RFC-0001).

---

## Milestone Summary

| Milestone | Target | Key capability | Status |
|-----------|--------|----------------|--------|
| M1 | 2026 Q1–Q2 | Core data model + typed command executor | ✅ (text DSL → RFC-0001) |
| M2 | 2026 Q3 | Constraint solver + sketch system | ✅ |
| M3 | 2026 Q4 | Feature engine + truck-BRep kernel | ✅ |
| M4 | 2027 Q1 | Component catalog + series library (PARAGON/SPECON) | ✅ |
| M5 | 2027 Q2 | Catalog component modeling (20+ components) | ✅ |
| M6 / M6b | 2027 Q3–Q4 | GenRule refactor, default templates, Library/SeriesStore infra | ✅ |
| M7 | 2027 Q4 | UI: Workspace + Library Manager + SeriesStore Manager | ✅ |
| M8 | 2028 Q1–Q2 | Domain extensions + production-grade components | 🔄 |
| M9 | 2028 Q3 | AMIR AI compilation | 🔀 Partial |
| M10 | 2029 | Cloud storage + collaboration | 📅 (REST+JSON-file server exists) |
| M11 | 2029 | Format compatibility | 🔀 Partial (IFC only) |
| M12 | 2029+ | Analysis + ecosystem | 📅 |
