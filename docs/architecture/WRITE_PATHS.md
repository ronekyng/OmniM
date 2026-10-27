---
title: "OmniM Write Paths — Static-Scan Snapshot (2026-10-27)"
status: current
last_updated: 2026-10-27
applies_to: "crates/, web/"
evidence: "audited against crates/server/src/lib.rs 2026-10-27"
---
# OmniM Write Paths — Static-Scan Snapshot (2026-10-27)

**Snapshot date:** 2026-10-27
**Basis:** static scan of `crates/server/src/lib.rs` for `async fn` handler bodies containing `.project.write()`, `executor`, and `.save(`, cross-checked against the route table. Figures reproduce Appendix A of the [2026-10-27 documentation audit](../status/DOC_AUDIT_2026-10-27.zh-CN.md).

> **Decision context.** [ADR-0006](DECISIONS.md) remains the **accepted target architecture**: every mutation should build an `OmniCommand`, pass `Validator` + `Executor`, and land on the undo stack. The current code does **not** satisfy it. This page records the gap, not a new decision.
>
> **Geometry path.** The official geometry path is the **feature tree / BRep** pipeline (`/api/catalog/instantiate`, `/api/components/*`). The `/api/genrule` pass-through that emits hardcoded mesh primitives is **legacy/transitional**, even though it is still reachable from the toolbar.

| Measure | Count |
|---|---|
| Routes under `/api/*` (deduplicated) | **78** |
| Handlers holding `project.write()` | **55** |
| Of those, handlers where `executor` appears | **13** |
| Of those, handlers calling `.save(` | **5** |
| Approximate direct-write handlers (no executor) | **~42** |

> **Method caveat.** This is a **static text scan**, not runtime tracing. It can over-count read-only handlers that acquire a write lock (e.g. `section_handler`) and under-count paths that save or record undo indirectly through helpers. The 55/13/5 magnitudes are the evidence; individual rows are illustrative. Re-scan after code changes.

---

## (a) Executor + Validator + undo path

These handlers construct an `OmniCommand` and run it through `Validator` + `Executor`, so the change is undoable. **None of them saves the project file.**

| Endpoint | Handler | Undo | Save |
|---|---|---|---|
| `POST /api/objects` | `create_object` | ✅ | ❌ |
| `DELETE /api/objects/{id}` | `delete_object` | ✅ | ❌ |
| `POST /api/genrule` | `genrule_create` | ✅ | ❌ |
| `POST /api/omcom` | `execute_omcom` | ✅ | ❌ |
| `POST /api/undo` | `undo` | history op | ❌ |
| `POST /api/redo` | `redo` | history op | ❌ |
| `POST /api/sketches`, `DELETE /api/sketches/{id}` | `create_sketch`, `delete_sketch` | ✅ | ❌ |
| `POST /api/sketches/{id}/entities`, `POST …/constraints`, `DELETE …/entities/{idx}`, `DELETE …/constraints/{idx}` | `add_sketch_entity`, `add_sketch_constraint`, `delete_sketch_entity`, `remove_sketch_constraint` | ✅ | ❌ |

Notes:

- `POST /api/omcom` accepts a full `OmniCommand` envelope. It does **not** accept `{"command":"UNDO"}`; undo is `POST /api/undo`.
- `POST /api/genrule` still produces hardcoded mesh primitives (`build_wall` / `build_slab` / `build_road` / `build_pipe` with empty `param_values`) rather than driving the catalog feature tree. It is the legacy/transitional path.

## (b) Direct `Project` writes — no undo

These handlers take `state.project.write()` and mutate the aggregate directly. They do **not** create an undo entry.

| Endpoint | Handler | Undo | Save |
|---|---|---|---|
| `PUT /api/objects/{id}` (rename/description/tags) | `update_object` | ❌ | ✅ |
| `PUT /api/objects/{id}/prop` | `set_property` | ❌ | ❌ |
| `PUT /api/objects/{id}/properties` | `update_properties` | ❌ | ❌ |
| `PUT /api/objects/{id}/transform` | `transform_object` | ❌ | ❌ |
| `PUT /api/objects/{id}/parent` | `set_parent` | ❌ | ❌ |
| `POST /api/objects/{from}/connect/{to}` | `connect` | ❌ | ❌ |
| `POST /api/materials` | `create_material` | ❌ | ❌ |
| `PUT /api/objects/{id}/material` | `assign_material` | ❌ | ❌ |
| `POST /api/measurements`, `DELETE /api/measurements/{id}` | `create_measurement`, `delete_measurement` | ❌ | ❌ |
| `POST /api/lifecycle`, `POST /api/lifecycle/{id}/link/{obj_id}` | `create_lifecycle`, `link_record` | ❌ | ❌ |
| `POST /api/administrative`, `DELETE /api/administrative/{id}` | `create_administrative`, `delete_administrative` | ❌ | ❌ |
| `POST /api/breakdown`, `DELETE /api/breakdown/{id}`, `POST …/link-obj`, `POST …/link-admin` | `create_breakdown`, `delete_breakdown`, `link_breakdown_object`, `link_breakdown_admin` | ❌ | ❌ |
| `PUT /api/sketches/{id}`, `POST /api/sketches/{id}/solve`, `PUT /api/sketches/{id}/entities/{idx}` | `update_sketch`, `solve_sketch`, `update_sketch_entity` | ❌ | ❌ |
| `POST /api/sketches/{id}/extrude`, `POST /api/sketches/{id}/revolve` | `extrude_sketch_handler`, `revolve_sketch_handler` | ❌ | ❌ |
| `POST /api/components`, `DELETE /api/components/{id}` | `create_component_handler`, `delete_component` | ❌ | ❌ |
| `POST /api/components/{id}/features` | `add_feature_handler` | ❌ | ✅ |
| `PUT /api/components/{id}/features/{fid}` | `update_feature_handler` | ❌ | ✅ |
| `DELETE /api/components/{id}/features/{fid}` | `remove_feature_handler` | ❌ | ✅ |
| `POST /api/components/{id}/rebuild` | `rebuild_component_handler` | ❌ | ✅ |
| `POST /api/import/ifc` | `import_ifc` | ❌ | ❌ |
| `POST /api/catalog/instantiate` | `instantiate_from_catalog` | ❌ | ❌ |
| `POST /api/series`, `PUT /api/series/{id}`, `DELETE /api/series/{id}`, `POST …/selectors`, `POST …/answers`, `POST …/spec`, `POST …/constraints`, `PUT …/constraints/{idx}`, `DELETE …/constraints/{idx}` | `create_series_handler`, `update_series_handler`, `delete_series_handler`, `add_selector_handler`, `add_answer_handler`, `add_spec_component_handler`, `add_param_constraint_handler`, `update_param_constraint_handler`, `delete_param_constraint_handler` | ❌ | ❌ |

`POST /api/import/ifc` deserves emphasis: it parses raw IFC text and inserts the resulting objects/materials/lifecycle records straight into `Project` (with `mode=replace` it overwrites the whole project). It does not translate to `OmniCommand`, so an import cannot be undone and is not saved. See the implementation note in [ADR-0005](DECISIONS.md#adr-0005-hand-written-ifc-parser-rather-than-an-ifc-crate).

## (c) Save behavior

A handler persists only when **both** conditions hold:

1. `state.save_path` is `Some` — which happens only when `OMNIM_PROJECT` is set to a value other than the literal `project.omnim` (`crates/server/src/main.rs`).
2. The handler is one of the five that call `Project::save(...)`:
   - `update_object`
   - `add_feature_handler`
   - `update_feature_handler`
   - `remove_feature_handler`
   - `rebuild_component_handler`

Everything else in groups (a) and (b) stays in memory. With the default launch there is no persistence at all. `scripts/dev-server.sh` forwards `OMNIM_PROJECT`, but its default (`project.omnim`) still maps to `save_path = None`; export a non-default path to enable saving.

| User action | Undo | Saved by default |
|---|---|---|
| Create object / genrule / `/api/omcom` | ✅ | ❌ |
| Rename object | ❌ | ✅ (only with non-default `OMNIM_PROJECT`) |
| Transform, reparent, connect, assign material | ❌ | ❌ |
| Feature add/update/remove/rebuild | ❌ | ✅ (only with non-default `OMNIM_PROJECT`) |
| IFC import | ❌ | ❌ |
| Catalog instantiate | ❌ | ❌ |
| Series edits | ❌ | ❌ |

---

## Restoring the single write path

The target remains ADR-0006: one command pipeline for every mutation, with undo and a defined persistence policy. This is a tracked roadmap item — link to [ROADMAP.md](../status/ROADMAP.md) rather than restating a plan here. Until it lands, these deviations are recorded in [CODE_DEVIATIONS.md §6](../status/CODE_DEVIATIONS.md#6-write-path-persistence--geometry-path-deviations--known-2026-10-27).
