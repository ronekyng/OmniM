---
title: "Assembly Container & Parts (装配容器与子件)"
status: design
last_updated: 2026-10-30
applies_to: "proto/, crates/model, crates/omcom, crates/server, web/"
evidence: "design contract for the Phase-8 assembly upgrade; implemented incrementally by the same change set"
---
# Assembly Container & Parts (装配容器与子件)

> **Why this document exists.** The 2026-10-30 UI/UX review and the code audit both
> flagged the old `Assembly` GenRule: it merged primitive meshes into a single
> object and lost all structure — no children, no BOM, no per-part selection. This
> contract replaces that user-facing flow with an **assembly container whose parts
> are ordinary objects** (`Component` instances with `catalog_ref`, or primitive
> `Part` bodies), while keeping the legacy merged-mesh path readable.

## 1. Object model

```
Assembly (ObjectType::Assembly, 401)          ← container, no geometry of its own
├── Component (ObjectType::Component, 500)    ← catalog part: catalog_ref + baked geometry
├── Component                                 ← …
└── Part (ObjectType::Part)                   ← primitive part: baked mesh, no catalog_ref
```

- **Container**: name, optional `parent_id` (site / building / storey / another
  container), `geometry = None`, properties/tags. It is the selection, grouping
  and export unit.
- **Parts**: direct children via `parent_id` + spatial `Containment`. Catalog
  parts are created through the same instantiation helper as
  `POST /api/catalog/instantiate` (params in mm, placement in m, feature-tree
  geometry, `catalog_ref`). Primitive parts are box/cylinder/sphere/cone meshes
  baked at the requested pose.
- **Placement**: v1 **bakes** the pose into each part's geometry (translation and
  Z rotation). A per-object `placement` field is future work; until then moving
  the container cascades the same transform to its direct children (§4).
- **Nesting**: containers can sit under any object; parts are leaves. Nested
  assemblies are out of scope for v1.

## 2. Command contract — `AssemblyCreate`

```
Action: ASSEMBLY_CREATE (130)
Operation: CreateAssemblyOperation {
  string name = 1;
  string parent_id = 2;          // optional container parent
  bool skip_series_gate = 3;     // same semantics as InstantiateCatalog
  repeated AssemblyPart parts = 4;
}
AssemblyPart { string name = 1; oneof source {
  ComponentPart component = 2;   // component_def_id, params, position, rotation_z(rad)
  PrimitivePart primitive = 3;   // shape, p1..p3, position, rotation_z(rad)
} }
```

Validation (both `Validator` and executor):
- `1..=64` parts; names non-empty; `component_def_id` exists in the catalog;
  primitive `shape ∈ {box, cylinder, sphere, cone}` and dimensions > 0;
- optional `parent_id` must exist;
- series gate applies unless `skip_series_gate` (same reachability rule as
  catalog instantiation).

Execution:
1. insert the container object (type `Assembly`);
2. create each part (catalog helper / primitive mesh), set `parent_id`, add
   containment edges;
3. record **one** `Command::CreateBatch { snapshots, sketches }` covering the
   container, all parts and any sketches produced by instantiation → a single
   undo step;
4. any failure removes what this operation created and returns
   `success: false` (no partial assembly).

`created_ids` = container first, then parts in request order.

## 3. Delete semantics

- `DELETE /api/objects/{id}?cascade_children=true` (UI sends it for containers)
  → `DELETE_OBJECT_SUBTREE` action: every descendant is removed, containment /
  connection edges are snapshotted, lifecycle/breakdown references detached,
  and the whole subtree is restored by **one** undo (`DeleteCascade` extended
  with `deleted_children`).
- Without the flag the existing "promote children" behaviour is unchanged, so
  deleting a Site/Building keeps its contents.
- Deleting a storey that still anchors objects is still refused.

## 4. Transform semantics

`PUT /api/objects/{id}/transform` on an assembly container with children applies
the same matrix (translation / rotation / scale, pivot semantics unchanged) to
every **direct child** geometry, and records one `Command::TransformAssembly`
covering all children (`Command::Batch` of `ModifyObject` commands) — one undo restores every part. Legacy merged-mesh
assemblies (no children) keep the leaf behaviour.

## 5. IFC export

Parts are ordinary objects and export with their own mappings (Wall → `IfcWall`,
Component → proxy, …). The container is exported by the existing fallback
(`IfcBuildingElementProxy`) and carries the Name/Description; a dedicated
`IfcElementAssembly` mapping is future work.

## 6. UI flow (AssemblyPanel)

1. Name the assembly; optional parent via the existing parent selector.
2. Add parts, each either:
   - **Catalog part** — pick a catalog component, edit its parameters (mm), set
     position (m) and Z rotation (deg); or
   - **Primitive part** — box/cylinder/sphere/cone with dimensions/offsets (m).
3. The part list shows the source badge, dimensions and pose; validation refuses
   empty names / zero-size primitives.
4. "创建 Assembly" calls `POST /api/assemblies`; parts appear under the container
   in the object tree; deleting the container removes the subtree.

`POST /api/genrule` with `type: "assembly"` remains as a **legacy** merged-mesh
path for scripted/backward-compatible callers and for the built-in equipment
types (`Stair`, `AHU`, `Chiller`, `Transformer`, `Reactor`) that reuse
`AssemblyGen` internally.

## 7. Out of scope (tracked)

- Per-part live placement field and non-destructive move of individual parts.
- BOM / quantity endpoint over `catalog_ref` children.
- Kinematic joints / constrained assemblies (ROADMAP Phase 8).
- Nested assemblies and part replacement workflows.
