---
title: "Code-Level Deviations from Documented Model"
status: current
last_updated: 2026-10-27
applies_to: "repo"
evidence: "audited against code 2026-10-27"
---
# Code-Level Deviations from Documented Model

This document lists places where the codebase deviates from the documented conceptual model. Items marked **RESOLVED** have been fixed on the date indicated; remaining items are known gaps to be fixed in a future code development phase.

## 1. 4-Class Component Taxonomy (Surface / Linear / Point / Body) — **RESOLVED** (2026-08-11)

**Documented** (see `DATA_MODEL.md §1`, `GLOSSARY.md §17`, `DOMAINS.md §7.1`, `PLATFORM_ARCH.md §3.1`):

| Class | Objects | GenRule |
|-------|---------|---------|
| **Surface** (面构件) | Wall, Slab, Roof, Floor | LayerExtrusion |
| **Linear** (线构件) | Beam, Column, Pipe, Road, Duct, CableTray | LinearComponent |
| **Point** (点构件) | PointCloud, TerrainModel, FeatureLine, ContourLine | non-GenRule |
| **Body** (体构件) | Tank, Pump, Valve, Gear, Equipment | Primitive / Assembly |

**Code status**: The 4-class taxonomy is **now implemented** as `SURFACE_TYPES` / `LINEAR_TYPES` / `POINT_TYPES` / `BODY_TYPES` in both `MainToolbar.tsx` and `ObjectPropertiesPanel.tsx`.

---

## 2. Catalog Units: Millimetres vs. Metre Viewport — **RESOLVED** (2026-08-11)

**Documented**: The 3D viewport, grid, camera, and legacy GenRule geometry all work in **metres** (e.g. a legacy wall is 5×0.2×3 m). 

**Code status**: Catalog `ComponentDef` parameters are authored in **millimetres** (`unit: "mm"`, e.g. `length=5000`), and the feature-tree templates generate geometry directly from those mm values (profile dimensions hard-coded in mm, extrusion depth resolved from mm params). Before the fix, `/api/mesh` and `/api/catalog/preview` returned these raw mm coordinates, so a 5000 mm wall rendered as a **5000 m** wall in the metre-based viewport — enormous and lost in the scene fog (`THREE.Fog` near=100/far=600).

**Fix**: The render path now scales catalog-component meshes by `1/1000`:
- `crates/model/src/geometry.rs` — new `scale_mesh(mesh, factor)` helper
- `crates/server/src/lib.rs` `get_mesh` — meshes of objects with a `catalog_ref` are scaled to metres (legacy GenRule objects, which already generate metre geometry, are untouched)
- `crates/server/src/lib.rs` `preview_catalog_component` — preview mesh scaled to metres

The stored model remains in mm (consistent with `ComponentDef` params); only the display/render path converts to metres.

---

## 3. Creation Workflow — Full Deviation Matrix

### 3.1 SURFACE TYPES (面构件) — pipeline RESOLVED 2026-10-27; kernel residue tracked

| Type | Correct workflow (from research) | Code behavior (current) | Deviation |
|------|-------------------------------|------------------------|-----------|
| **Wall** | Polyline path (2+ points in plan) + height + WallType layers | LAYERED path: `neededClicksForType`=2 (min 2, N allowed), points → `boundary_points`; MainToolbar submits `layered` genrule with `path[]` → server `build_layered_path` builds per-segment layered geometry | ⚠️ pipeline works; residue: picked-point z dropped (`to_p2`), bend corners not mitered (overlap/gap) |
| **Slab** | Closed boundary (N-point polygon in plan) + FloorType layers | polygon mode (min 3), close via first point or double-click/Enter → `boundary[]` → `build_layered_slab_path` (per-layer lift with bottom closure) | ✅ pipeline matches; residue: no polygon self-intersection/area validation |
| **Roof** | Extrusion profile + extent (2 points) OR footprint sketch (N points) | footprint mode ✅ same as slab; extrusion mode (2 points) → `build_layered_roof_profile` (genuine profile, width honored, base_z applied) | ✅ Matches |
| **Floor** | Closed boundary (N-point polygon in plan) + FloorType layers | Same as slab (polygon → `boundary[]`), works | ✅ Matches |

**Root cause (historical)** ~~was~~: `neededClicksForType` collapsed all SURFACE_TYPES to 2 clicks and `MainToolbar.tsx` ignored points beyond the first. Fixed by the pick-mode state machine (`defaultPickModeForType` per type, `boundary_points`/`is_closed` on `CreationForm`, `closeToPoint` close detection) and deviated forms now consuming `path[]`/`boundary[]`.

### 3.2 LINEAR TYPES (线构件)

| Type | Correct workflow (from research) | Code behavior | Deviation |
|------|-------------------------------|--------------|-----------|
| **Beam** | 2 points (start/end) | 2 clicks, start/end used ✅ | ✅ Matches |
| **Column** | 1 point (location) in Revit; 2 points (base/top) in Tekla | 2 clicks, treated as linear | ⚠️ Minor — both valid; single-point mode would be more intuitive for Revit-style |
| **Pipe** | 2 points (connector-to-connector) | 2 clicks ✅ | ✅ Matches |
| **Duct** | 2 points (connector-to-connector) | 2 clicks ✅ | ✅ Matches |
| **Road** | Alignment + profile + assembly (multi-step corridor workflow) | polyline points collected but geometry is a straight road from start/end only (middle points folded into `length` param) | ⚠️ Expected — full corridor workflow not implemented yet (see ROADMAP.md); alignment consumption planned as the next kernel item |
| **Bridge** | Multi-step subassembly corridor / assembly | same as road | ⚠️ Same as road |
| **Tunnel** | Multi-step corridor assembly | same as road | ⚠️ Same as road |

### 3.3 POINT TYPES (点构件)

| Type | Correct workflow (from research) | Code behavior | Deviation |
|------|-------------------------------|--------------|-----------|
| **PointCloud** | 1 click (placement) | 1 click ✅ | ✅ Matches |
| **TerrainModel** | 1 click (placement) | 1 click ✅ | ✅ Matches |
| **FeatureLine** | N-point polyline drawing | polyline mode (min 2, N allowed) → `line` genrule `alignment[]` → thin guide tube along the polyline | ✅ Matches |
| **ContourLine** | N-point polyline drawing | same as featureLine | ✅ Matches |

### 3.4 BODY TYPES (体构件)

| Type | Correct workflow (from research) | Code behavior | Deviation |
|------|-------------------------------|--------------|-----------|
| **Tank** | 1 point (placement) | 1 click ✅ | ✅ Matches |
| **Pump** | 1 point (placement, connector-based) | 1 click ✅ | ✅ Matches |
| **Valve** | 1 point (placement, inline) | 1 click ✅ | ✅ Matches |
| **Gear** | 1 point (placement) | 1 click ✅ | ✅ Matches |
| **Equipment** | 1 point (placement) | 1 click ✅ | ✅ Matches |
| **Part** | 1 point (placement) | 1 click ✅ | ✅ Matches |
| **Assembly** | 1 point (placement) | 1 click ✅ | ✅ Matches |

---

## 4. Rust Backend (crates/) — Correct

The Rust backend (`crates/model/src/gen_rule/mod.rs`) correctly maps:
- Wall → `LayerExtrusion` (line 76-77)
- Beam/Column → `LinearComponent` (line 80)
- Road/Bridge/Pipe/Duct → `LinearComponent` (lines 83-90)

No deviation found in the Rust backend.

---

## 5. Picking Infrastructure Gaps — **largely RESOLVED** (2026-10-27)

**Status**: The end-to-end N-point picking pipeline is implemented and verified; the claims in §3.1/§3.3 above ("1st point only", "not implemented") predate it and are corrected in place by the row edits in this section. Remaining truths:

### `web/src/app/components/viewer/three/Picking.ts`
- ~~`pickingHint` only supports `needed === 2` or `needed === 1` — no N-point polygon hint~~ **RESOLVED**: `pickingHint` handles polyline/polygon completion states (incl. Chinese close-loop hint).
- ~~`pickGroundPoint` picks a single ground-plane point — no multi-point polygon mode~~ **RESOLVED**: pick mode state machine (`PickMode` polyline/polygon/segment/place) drives multi-point collection; snapping (`snapPoint`) applies on every pick.

### `web/src/shared/types/creation.ts`
- ~~`neededClicksForType` returns 2 for all SURFACE_TYPES~~ **RESOLVED**: `defaultPickModeForType` differentiates wall→polyline(≥2), slab/floor/roof→polygon(≥3), beam/pipe/duct→segment, others→place.
- `neededClicksForType` still collapses to `2|3|1` (span filter derived in `applyPointsToForm`); kept as a convenience count — acceptable.
- ~~`applyPointsToForm` only handles 1 or 2 points~~ **RESOLVED**: serializes N points into `CreationForm.boundary_points` + `is_closed`.
- ~~`CreationForm` lacks a `boundary_points` field~~ **RESOLVED**.

### `web/src/app/components/ThreeViewer.tsx`
- ~~Preview only drew a dashed cursor line; whole polyline redrawn on every click inside the same marker group (conflicting lifecycles)~~ **RESOLVED (2026-10-27)**: split into confirmed visuals (point markers + chain/fill, synced from `creationPoints` by effect) and a per-frame hover group (dashed line, whole-path layered ghost, polygon loop + fill), rebuilt on every pointer move; hover endpoint snapped like a click.

### `web/src/app/components/viewer/three/Markers.ts`
- ~~Preview showed only the last segment as a hit grey slab; layer thickness scaled 20× too large (`unit=0.02`); unlit `MeshBasicMaterial` ghost; no bend miter~~ **RESOLVED (2026-10-27 preview overhaul)**: `addLayeredWallPreview` builds the whole-path per-segment ghost with the server mm→m scale (`0.001`), bend miter identical to the kernel (`e=(t/2)·tan(φ/2)`, capped), lit translucent standard material (`opacity 0.55`, no depth-write); hover endpoint passes through the same `snapPoint` pipeline as clicks. Invalid polygon footprints preview red (`polygonPreviewError`, mirrors server `validate_boundary`).

### `web/src/app/components/MainToolbar.tsx`
- ~~SURFACE_TYPES objects fall to `else` branch using single position~~ **RESOLVED**: LAYERED_TYPES submit `layered` genrule with `path[]` (wall polyline) / `boundary[]` (slab/roof/floor footprint); LINE_GENRULE_TYPES submit `line` with N-point alignment.
- ~~No `boundary_points` written for slab/roof/floor~~ **RESOLVED** (form carries boundary_points; genruleCreate consumes it).
- Residual geometry gaps (kernel): ~~picked-point z dropped in `to_p2`~~ (now `base_z` preserved); ~~roof extrusion falls back to rotated slab rectangle~~ (now `build_layered_roof_profile`); ~~road alignment not consumed~~ (now `Road.alignment` polyline); ~~corners not mitered~~ (now mitered for 90°/45°). Remaining: bridge/tunnel start/end only.

### Screens/tests
- `stateMachine.test.ts` covers idle/picking/configuring flows incl. multi-point accumulation; polygon-close and span-filter cases to be added with the residue items.

---

## 6. Write Path, Persistence & Geometry-Path Deviations — **Known** (2026-10-27)

**Status: Known — 2026-10-27.** Source: static scan of `crates/server/src/lib.rs` in [DOC_AUDIT_2026-10-27](DOC_AUDIT_2026-10-27.zh-CN.md) Appendix A, plus a spot-check of the code. The per-endpoint matrix is in [WRITE_PATHS.md](../architecture/WRITE_PATHS.md). None of these items is resolved. For a modeler-facing, per-object view of the same gaps, see [CAPABILITY_MATRIX.md](../CAPABILITY_MATRIX.md).

### 6.1 Single write path is violated (audit P0-2)

**Documented** (ADR-0006, PLATFORM_ARCH §1, DATA_FLOW §0): every mutation goes through `Validator` + `Executor` and lands on the undo stack.

**Code status:** of **55** handlers holding `project.write()`, only **13** mention the executor; roughly **42** mutate `Project` directly with no undo entry. Examples: `update_object`, `transform_object`, `set_parent`, `connect`, `create_material`/`assign_material`, `update_sketch`/`solve_sketch`/`update_sketch_entity`, `import_ifc`, `instantiate_from_catalog`, and all series edit handlers. Renaming, assigning material, connecting pipes, editing series, importing IFC, and placing catalog components cannot be undone.

### 6.2 Auto-save claim is false; default launch saves nothing (audit P0-3)

**Documented**: mutations are persisted automatically.

**Code status:** only **5** handlers call `Project::save(...)`: `update_object`, `add_feature`, `update_feature`, `remove_feature`, `rebuild_component`. `POST /api/objects`, `/api/genrule`, `/api/omcom`, `/api/catalog/instantiate`, `/api/import/ifc`, and all `/api/series/*` edits never save. In addition, `main.rs` sets `save_path = None` when the project path is the default `project.omnim`, so the default launch persists nothing at all. `scripts/dev-server.sh` forwards `OMNIM_PROJECT` but defaults it to `project.omnim`, so the default script launch does not persist either.

### 6.3 Geometry path is split (audit P0-4)

**Documented**: catalog/series drive geometry generation.

**Code status:** three parallel paths produce different object types and geometry sources:

- `/api/genrule` (toolbar): hardcoded `build_wall`/`build_slab`/`build_road`/`build_pipe` mesh, `param_values` empty, object types `Wall`/`Slab`/`Road`/`Pipe`.
- `/api/catalog/instantiate` (catalog panel): feature tree → BRep/mesh (official path), but creates `ObjectType::Component` (not `Wall`/`Beam`/…), with no undo and no save.
- `/api/import/ifc`: imported mesh, mapped types, no undo and no save.

The official geometry path is the feature tree/BRep pipeline; the genrule pass-through is legacy/transitional. See [DATA_FLOW.md §4](../architecture/DATA_FLOW.md#4-d-catalog--series-instantiation).

### 6.4 Series gate, materials, connections are weaker than documented (audit P0-5)

**Documented**: series gates enforce approved components, materials, and connection compatibility.

**Code status:**

- `check_series_gate()` only checks whether a component is reachable from a series (`crates/model/src/project.rs:88-103`); it does not validate material, parameter ranges, or connection rating.
- `instantiate_from_series()` only fills `ParamConstraint.default_value`; it does not enforce `min/max/fixed`, and it does **not apply** `SpecComponent.material` (`crates/model/src/project.rs:368-386`).
- `Project::connect()` only checks that both objects exist; it does not validate connector points, type codes, bore, or pressure rating, and does not write `from_point`/`to_point` (`crates/model/src/project.rs:105-121`).
- `ConnectorDef` has only `direction`; there is no `up`/axis, so flange bolt-hole or valve-stem orientation cannot be determined (`proto/omnim.proto:506-518`).
