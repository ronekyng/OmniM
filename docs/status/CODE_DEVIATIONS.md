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

### 3.1 SURFACE TYPES (面构件)

| Type | Correct workflow (from research) | Code behavior | Deviation |
|------|-------------------------------|--------------|-----------|
| **Wall** | Polyline path (2+ points in plan) + height + WallType layers | `neededClicksForType` returns 2, points collected, but `MainToolbar.tsx` only uses 1st point (`px, py, pz`) in `else` branch (line 196) | ❌ Polyline path not implemented; 2nd click is wasted |
| **Slab** | Closed boundary (N-point polygon in plan) + FloorType layers | Same as wall — 2 clicks, only 1st point used | ❌ Needs N-point closed polygon + typed floor definition |
| **Roof** | Extrusion profile + extent (2 points) OR footprint sketch (N points) | Same as wall — 2 clicks, only 1st point used | ❌ Needs profile/extent mode or footprint polygon |
| **Floor** | Closed boundary (N-point polygon in plan) + FloorType layers | Same as wall — 2 clicks, only 1st point used | ❌ Needs N-point closed polygon + typed floor definition |

**Root cause**: `neededClicksForType` (`web/src/shared/types/creation.ts:168`) lumps all SURFACE_TYPES together (returns 2). `applyPointsToForm` (line 115) fills `startX/Y/Z` / `endX/Y/Z` but `MainToolbar.tsx` ignores them for non-LINEAR types (line 196 uses single position). Picking hint (`Picking.ts:10`) only shows "click point 1 / 2" — no support for N-point polygon creation.

### 3.2 LINEAR TYPES (线构件)

| Type | Correct workflow (from research) | Code behavior | Deviation |
|------|-------------------------------|--------------|-----------|
| **Beam** | 2 points (start/end) | 2 clicks, start/end used ✅ | ✅ Matches |
| **Column** | 1 point (location) in Revit; 2 points (base/top) in Tekla | 2 clicks, treated as linear | ⚠️ Minor — both valid; single-point mode would be more intuitive for Revit-style |
| **Pipe** | 2 points (connector-to-connector) | 2 clicks ✅ | ✅ Matches |
| **Duct** | 2 points (connector-to-connector) | 2 clicks ✅ | ✅ Matches |
| **Road** | Alignment + profile + assembly (multi-step corridor workflow) | 2 clicks, simplified path | ⚠️ Expected — full corridor workflow not implemented yet (see ROADMAP.md) |
| **Bridge** | Multi-step subassembly corridor / assembly | 2 clicks, simplified path | ⚠️ Same as road |
| **Tunnel** | Multi-step corridor assembly | 2 clicks, simplified path | ⚠️ Same as road |

### 3.3 POINT TYPES (点构件)

| Type | Correct workflow (from research) | Code behavior | Deviation |
|------|-------------------------------|--------------|-----------|
| **PointCloud** | 1 click (placement) | 1 click ✅ | ✅ Matches |
| **TerrainModel** | 1 click (placement) | 1 click ✅ | ✅ Matches |
| **FeatureLine** | N-point polyline drawing | 1 click | ❌ Needs N-point polyline picking mode |
| **ContourLine** | N-point polyline drawing | 1 click | ❌ Same as featureLine |

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

## 5. Picking Infrastructure Gaps

### `web/src/app/components/viewer/three/Picking.ts`
- `pickingHint` only supports `needed === 2` or `needed === 1` — no N-point polygon hint
- `pickGroundPoint` picks a single ground-plane point — no multi-point polygon mode

### `web/src/shared/types/creation.ts`
- `neededClicksForType` returns 2 for all SURFACE_TYPES — should differentiate wall (2+ path) vs slab/roof/floor (N-point closed polygon)
- `applyPointsToForm` only handles 1 or 2 points — no N-point polygon serialization
- `CreationForm` lacks a `boundary_points` field for closed polygon storage

### `web/src/app/components/ThreeViewer.tsx`
- Preview line (line 834-860) only draws a dashed line for 1-point → cursor — no polygon preview for N-point picking
- `addPreviewLine` draws sequential segments — correct for N-point, but no "close polygon" visual feedback

### `web/src/app/components/MainToolbar.tsx`
- `SURFACE_TYPES` objects fall to `else` branch (line 196) using single position — ignores 2nd point entirely
- No `boundary_points` property written for slab/roof/floor
- Layer properties (`height`, `core_thickness`, `finish_thickness`) are hardcoded for wall/slab only (line 237) — roof and floor missing