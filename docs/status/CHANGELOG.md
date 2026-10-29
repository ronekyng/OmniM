---
title: "Changelog"
status: history
last_updated: 2026-10-28
applies_to: "repo"
evidence: "audited against code 2026-10-27"
---
# Changelog

All notable changes to the OmniM project are documented in this file. Substantive phase summaries only — this is not a commit log.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project does not currently use semantic versioning; entries are organized by roadmap phase.

**Last updated: 2026-10-28**

## Keep a Changelog legend

- **Added** — new capabilities
- **Changed** — changes to existing behavior
- **Fixed** — bug fixes / deviations resolved
- **Not implemented (known gaps)** — planned but absent capabilities, stated explicitly for honesty

---

## [2026-10-28] — Path contract & genesis-form authorization (Wave 2)

### Added
- **`path { points, closed }` contract** on `/api/genrule` (`layered`, `road`, `line`); legacy `path[]`/`boundary[]`/`alignment[]` aliases normalize with their historical closure; ambiguous double-spelling returns 400.
- **`(objectType, form)` authorization matrix** (`gen_rule::matrix`, DATA_MODEL §1 governance 4): authorized rows wall×panel/region, slab/floor/roof×region, roof×panel (2-point profile), road×linear, line×point; everything else returns 400 with the contract reason. Created objects record `genesis_class`/`genesis_form` properties.
- **Shared pure-geometry layer** (`gen_rule::path_geom`): `iter_segments` / `merge_collinear` / `validate_path` dispatch by open/closed — below classification, no class semantics (§6.4 boundary note is now a module-header contract on both engines).
- **Frontend**: `CreationForm.form` field (wall panel/region dropdown, roof sub-modes feed it), single `form → pickMode` lookup, `withGenesisForm` helper; unified `addPathPreview` (region → outline+fill, panel → layered ghost with miter, linear/point → guide outline).

### Tests
- `genrule_contract.rs` (9 seed cases: contract/alias/region/profile/matrix-400/closed-rejections/ambiguity/bowtie), `path_geom` + `matrix` unit tests, web `genesisForm.test.ts` (10 cases). Full suite: Rust 385, web 69.

## [2026-10-28] — Single write path & persistence (Wave 1)

### Changed
- **Single write path restored** (audit P0-2): every mutating REST handler now builds an `OmniCommand` and runs it through `Validator` + `Executor`, or delegates to `Executor` while holding the lock. Direct `Project` writes went **42 → 0**; new ops cover materials, sketch/component/catalog detail, series, lifecycle, measurements, administrative/breakdown, catalog instantiation and `load_series`.
- **Persistence defaults on** (audit P0-3): mutations save to `$OMNIM_PROJECT` (default `project.omnim`); `OMNIM_EPHEMERAL=1` disables writes. `Project::save` is atomic (`{path}.tmp` → rename, parent dir auto-created). New route `POST /api/project/save` (79 deduplicated routes).
- **Undo/redo coverage is complete**: endpoint-level tests in `crates/server/tests/write_paths.rs` and `crates/server/tests/instantiate_import.rs` cover objects/properties/materials, sketches, components, catalog definitions, series, lifecycle, measurements, administrative/breakdown, instantiation and IFC import. Fixed pre-existing broken undo/redo for connection delete, lifecycle records and record links.
- **IFC import is one transaction**: `Command::ReplaceProject { before, after }` — undo restores the pre-import project, redo re-applies. **Catalog preview is read-only** (built on a throwaway project), and catalog instantiation records the object plus generated sketches as one command.
- **CI guards**: `scripts/check_write_paths.py` (0 direct writes required) and `scripts/check_docs.py` run in CI; `scripts/test-all.sh` now runs both plus `cargo test` and web unit tests.

### Fixed
- Documented exceptions: `section_handler` (cache-only write lock), `import_ifc` (single whole-project command), `POST /api/catalog/import` (session-global `LIBRARY`, not project state). See [WRITE_PATHS](../architecture/WRITE_PATHS.md).

## [2026-10-28] — Git branching & workflow policy

### Added
- Branching policy: `main` (release baseline) + `develop` (staging, pushed to `origin/develop`); short-lived `<type>/<phase>-<slug>` branches; docs-first split (`docs/*` before `feat/*` for new specifications). See [BRANCHING](../guides/BRANCHING.md).
- Commit convention (Conventional Commits, scopes from crate/area names) and merge policy (squash into `develop`, merge-commit `release/*` → `main`, immediate `main` → `develop` back-merge after every release/hotfix).
- Phase-style annotated tags (`phase-7` baseline, `phase-8`, `phase-8.1`, `phase-8-rc.1`); the project still does not use SemVer.
- Tooling: `.github/workflows/ci.yml` (fmt/clippy/test/web build/docs check), PR base/head guard (`scripts/ci/branch_guard.py`), PR template, `.gitattributes`, local hooks (`.githooks/commit-msg`, `.githooks/pre-push`), commit template (`.gitmessage`), one-shot setup (`scripts/setup-git.sh`).

## [2026-10-27] — Documentation reorganization & dead-code cleanup

### Added
- Reorganized `docs/` into `architecture/`, `guides/`, `specification/`, `reference/`, `status/`, `RFC/` (bilingual en/zh per file), with a docs portal (`docs/README.md`).
- New reference: full REST API reference (`reference/API.md`, **78** deduplicated routes audited against `crates/server/src/lib.rs`).
- New specifications: Component Catalog & Series Library (`specification/CATALOG_SERIES.md`), Data Flow, Architecture Decision Records (6 seed ADRs).
- New guides: Getting Started, Building, Architecture Map, Testing.
- RFC-0001: Omcom Text DSL (Draft) — captures the former "text DSL" vision, now correctly framed as unimplemented.
- End-to-end CHANGELOG (this file).
- N-point picking geometry: preserved picked elevation in layered wall/slab generation (`base_z`), roof extrusion-profile builder, road polyline alignment (`Road.alignment`), bend-corner mitering, boundary validation (area/self-intersection/collinearity) with 400 errors, collinear-point merging; +9 kernel tests.
- Web interaction polish: Backspace removes the last picked point during picking, count-aware four-mode picking hints, polygon loop-close highlight segment; road creation submits picked alignment; +6 state-machine tests.
- Picking preview overhaul: whole-path layered wall ghost preview (per-segment boxes + bend miter matching the server kernel, correct mm→m scale — fixes a 20× thickness error), hover endpoint resolved through the same snap pipeline as clicks, lit translucent materials with no depth-write, invalid polygon footprints (self-intersecting/degenerate) preview in red; preview lifecycle split into confirmed markers/chain vs per-frame hover groups; +10 preview-geometry tests.

### Changed
- `PLATFORM_ARCH.md` rewritten from a cloud-first aspirational vision to an honest description of the current implementation (REST + JSON-file persistence, no WebSocket/DB/WASM/mobile).
- `FORMAT_COMPAT.md` corrected: IFC import/export is partial (hand-written parser); DXF/DWG/STEP-CAD/GIS/glTF are not implemented.
- `ROADMAP.md` statuses corrected (Phase 1 text-DSL claim restated as RFC-0001; evidence lines added per phase).
- `DATA_MODEL.md` audited against code: ~25 stale/false claims fixed (e.g. implemented Torus/Dish primitives, real FeatureType enum, prototype-vs-proto divergences marked 🗓 Planned).
- `OMCOM_REFERENCE.md` rewritten: was a reference for a fictional text DSL; now documents the real typed OmniCommand system (proto/omcom.proto + crates/omcom Validator/Executor).

### Removed
- `crates/renderer` — orphan wgpu/winit demo crate (no dependents); removed from the workspace. Rendering is three.js in the browser; a native viewer would be re-proposed via RFC.
- Unused dependencies: `prost-types` (core), `tower` (server), `omnim-omcom` (amir).
- Unimplemented Action enum values (~62/80/81/82) annotated as reserved in `proto/omcom.proto` (values kept, ids not reused).
- Dead frontend components: `CatalogBrowser.tsx`, `SeriesBrowser.tsx`, `OmcomConsole.tsx`, `ViewCube.tsx` (superseded by OmniPanel / ViewCube3D).
- Repo junk: stray web helper scripts (`fix_delim*.sh`, `test.sh`, committed `web/dist/`), stale `scripts/update_roadmap.py`, debug `eprintln!`/`console.log`.
- Repo-root test fixtures relocated to `fixtures/ifc/` (docs updated); `logo.png` moved to `docs/img/` and embedded in both READMEs.

### Fixed
- Compiler warnings reduced to zero across the workspace (unused vars/imports, needless `mut`).
- All intradocumentation links verified (0 broken).
- Audit remediation (docs/status/DOC_AUDIT_2026-10-27): quickstart commands corrected (real console syntax, `/api/undo`, raw-text IFC import) plus an explicit persistence warning; README vision/current-state separation; IFC_EXTENSION rewritten to match the partial-IFC reality; ADR-0006 marked "Accepted (target) — Partially implemented" with a deviation note; new `architecture/WRITE_PATHS.md`; route count corrected to 78; CODE_DEVIATIONS §6 records the audit P0s; Chinese DATA_MODEL aligned (+371 lines: §5.6, §5.7.1, §16.4, §16a, §17, renumbering, fence fixes); DOMAINS/GLOSSARY carry proto-status marks; new `CAPABILITY_MATRIX` and `MODELER_HANDBOOK`; front matter on all 71 docs; `scripts/check_docs.py` (links/fences/numbering/bilingual parity) added. Code-level P0s (single write path, save policy, geometry convergence, series enforcement) are recorded in the ROADMAP technical-debt backlog.

### Not implemented (known gaps)
- Unchanged: REST-only (no WebSocket/real-time collaboration), JSON-file persistence (no DB/version history), no WASM shared core, no desktop/mobile packaging (see `PLATFORM_ARCH` future-directions section).

---

## [Phase 8 — In Progress] (2028 Q1-Q2 target, started)

### Added
- Production-grade catalog data: steel sections (IPE/HEA/HEB/CHS/SHS per EN 10365, GB/T 706), pipe schedules (CS150, SS300 / ASME B36.10), concrete wall/beam/column/slab templates, connector definitions with flange ratings.
- Material assignments mapped to real standards (EN 10025, GB/T 1591, ASTM A36); BRep correctness validation of component templates.

### Not implemented (known gaps)
- Domain operations (MEP, earthwork/alignment, pipe routing, joints), quantity take-off, domain-specific UI panels.

### Fixed
- See [CODE_DEVIATIONS.md §1/§2](CODE_DEVIATIONS.md): 4-class component taxonomy and mm-vs-metre catalog scaling — **both RESOLVED 2026-08-11**.

---

## [Phase 7 — User Interface] (2027 Q4) — Completed

### Added
- Project Workspace: three.js viewer (orbit/pan/zoom, selection, section cuts, measurement, wireframe/shaded/x-ray/edge view modes), feature tree, Available Components Panel (series-filtered, sliders, drag-to-place), property panel, Omcom Console (syntax highlighting, history, tab completion), AI chat panel.
- Library Manager (`/library-manager`): ComponentDef tree, parameter editor, feature-template assignment, 3D preview, `.omnimlib` import/export.
- SeriesStore Manager (`/series-manager`): decision-tree editor, seed viewer, `.omnimseries` load/save, drag-to-map, constraint editor.
- UI architecture: Zustand, React Query, PanelRegistry, resizable AppLayout, CSS Modules.

### Not implemented (known gaps)
- N-point polygon/polyline in-scene picking (see CODE_DEVIATIONS §3/§5); AI command preview/accept workflow (deferred to Phase 9, partial).

---

## [Phase 6 / 6b — GenRule Refactor & Library Infrastructure] (2027 Q3-Q4) — Completed

### Added
- Library ↔ SeriesStore ↔ Project three-way split; Library as session-level global static; project-private SeriesStore + `external_series_refs`.
- Series gate (`check_series_gate`), GenRule → catalog/series instantiation, `CatalogRef` on instances, default-template registry, parameter validation against catalog domains.
- `.omnimlib` / `.omnimseries` external file formats, series resolution priority, session auto-load.

### Changed
- `generate_geometry()` deprecated in favor of the feature engine; `POST /api/genrule` now routes through the catalog/series system.

---

## [Phase 5 — Catalog Component Modeling] (2027 Q2) — Completed

### Added
- 20+ parameterized catalog components across structural (IPE/HEA/HEB/CHS/SHS/Angle), piping (pipe, elbow, tee, reducers, flange, valves), and building (concrete/door/window) families; series definitions CS150, SS300, Concrete-C30, Steel-Q355, Highway-GB.
- Profile section library, ASME B36.10M wall-thickness table, FeatureTemplate + FeatureTreeRegistry, instantiation pipeline, REST `/api/catalog/instantiate` and `/api/catalog/preview`.

---

## [Phase 4 — Component Catalog & Series Library] (2027 Q1) — Completed

### Added
- PARAGON-style component catalog (ComponentDef/ParamDef/ConnectorDef, Category hierarchy) and SPECON-style series library (selector decision trees, SpecComponent constraints, numeric-range matching), selection engine, connector type-code compatibility, external file storage.

---

## [Phase 3 — Feature Engine & BRep Kernel] (2026 Q3-Q4) — Completed

### Added
- Feature as first-class entity (cached MeshGeometry, dirty tracking, rebuild pipeline, rollback/suppression); 9 feature types (Extrude, Revolve, Sweep, Loft, Boolean, Chamfer, Fillet, Pattern, Shell).
- truck-BRep integration (solid extrude/revolve, sweep along path, loft, booleans on BRep solids), tessellation to meshes, parameter system with meval expressions.
- Revit-style Component Editor UI (feature panel/ribbon, auto-sketch workflow, drag-reorder, Rebuild All).

---

## [Phase 2 — Constraint Solver & Sketch] (2026 Q2-Q3) — Completed

### Added
- Sketch data model (SketchPlane, entities, 11 constraint types, SketchStore); slvs-based constraint solver with DOF analysis and over/under-constraint reporting; sketch/feature/component commands with undo/redo; standalone Sketch Editor web page; `EXTRUDE_SKETCH` / `REVOLVE_SKETCH` to 3D meshes.

---

## [Phase 1 — Core Data Model & Command Layer — Partial] (2026 Q1-Q2)

### Added
- Unified data model (OmniObject hierarchy, properties/property sets, materials, ObjectType across 4 domains), project lifecycle & breakdown structures, administrative-division hierarchy, relationship system.
- `OmniCommand` typed schema used purely as serde-JSON types (no protobuf wire runtime) and a complete typed command executor with full undo/redo.

### Not implemented (known gaps)
- **Omcom text DSL / nom parser** — the Web UI console executes JSON commands plus a few ad-hoc shortcuts. Text DSL is a draft RFC: [RFC-0001](../RFC/README.md).

---

## Baseline (current runtime)

- Axum REST server (**78** deduplicated routes) with JSON-file (`.omnim`) persistence; **no WebSocket/real-time collaboration, no database/version history/audit trail**.
- AMIR AI engine (OpenAI/Anthropic/Mock providers): NL→OmniCommand translation, chat, reverse model description.
- Hand-written IFC (2x3/4) import/export with **partial entity coverage**; no STEP/DXF/GIS/glTF support.
- Web UI: three.js-based workspace, sketch editor, component editor, series/library managers, AI chat, Omcom console.
