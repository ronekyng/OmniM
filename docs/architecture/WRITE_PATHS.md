---
title: "OmniM Write Paths — Current State (2026-10-28)"
status: current
last_updated: 2026-10-28
applies_to: "crates/server, crates/omcom, web/"
evidence: "verified by scripts/check_write_paths.py and cargo test --workspace 2026-10-28"
---
# OmniM Write Paths — Current State (2026-10-28)

[中文](WRITE_PATHS.zh-CN.md)

**Status: the single write path is restored (Wave 1, 2026-10-28).** Every mutating
endpoint builds an `OmniCommand`, runs it through `Validator` + `Executor`, lands
on the undo/redo stack, and persists atomically. The counts below are reproducible
with `python3 scripts/check_write_paths.py`; endpoint-level undo coverage lives in
`crates/server/tests/write_paths.rs` and `crates/server/tests/instantiate_import.rs`.

> **Decision context.** [ADR-0006](DECISIONS.md) is now implemented, not just the
> target: handlers call `mutation::apply` (or hold the lock and delegate to the
> `Executor`) instead of mutating `Project` in place.
>
> **Geometry path.** The official path is the feature tree pipeline. As of Wave 3
> (2026-10-28) the genrule **layered** family is converged: path-driven wall/slab/
> floor/roof creation instantiates a `PathSweep` feature tree and produces the
> **domain object type** (`Wall`, not `Component`) with `catalog_ref`. Still
> transitional: the rect parameter quick-forms (`wall`/`slab` variants without a
> path), roof 2-point profiles, and the catalog's extrude-based pipe/road templates.

| Measure | Value |
|---|---|
| Routes under `/api/*` (deduplicated paths) | **79** |
| Handlers scanned in `crates/server/src/lib.rs` | **105** |
| Handlers calling `mutation::apply` | **55** |
| Handlers delegating to the `Executor` while holding the lock | **7** (`create_object`, `delete_object`, `execute_omcom`, `genrule_create`, `undo`, `redo`, `import_ifc`) |
| Non-allowlisted direct `project.write()` | **0** |
| Direct `Project::save()` outside `save_project` | **0** |
| Default persistence | **on** — `$OMNIM_PROJECT` (or `project.omnim`); `OMNIM_EPHEMERAL=1` disables |

## How a mutation flows

```
handler → mutation::apply(action, ops)
        → Validator::validate        (rejects invalid state, nothing changes)
        → Executor::execute          (mutates Project, records undo commands)
        → mutation::persist          (atomic write when save_path is Some)
```

Lock order is fixed: `project` first, then `executor`. `mutation.rs` is the only
place that acquires both; handlers never hand-roll that pair.

## Migrated endpoint families

All of these are undoable and persist by default.

| Family | Endpoints | Undo | Save | Slice |
|---|---|---|---|---|
| Objects / properties / materials | create/delete/update object, set property, update properties, transform, reparent, connect, create/assign material | ✅ | ✅ | 1–2 |
| Sketches | create/delete sketch, add/update/delete entity, add/remove constraint, solve, extend, extrude/revolve | ✅ | ✅ | 1–3 |
| Components | create/delete instance, add/update/remove feature, rebuild | ✅ | ✅ | 3 |
| Catalog definitions | create/update/delete def, params add/update/delete, connector add, feature-tree assign, publish | ✅ | ✅ | 3 |
| Series | create/update/delete series, selectors, answers, spec components, param constraints add/update/delete, save/load | ✅ | ✅ | 4 |
| Lifecycle | create record, link record to object | ✅ | ✅ | 4 |
| Measurements | create/delete | ✅ | ✅ | 4 |
| Grid datum (P8-S1) | `POST/GET /api/grids`, `PUT/DELETE /api/grids/{id}` | ✅ | ✅ | P8-S1 |
| Administrative / breakdown | create/delete division, create/delete node, link object, link division | ✅ | ✅ | 4 |
| Catalog instantiation | `/api/catalog/instantiate` (object + generated sketches as one command) | ✅ | ✅ | 5 |
| IFC import | `/api/import/ifc` (whole-project transaction, merge or replace) | ✅ | ✅ | 5 |
| Explicit save | `POST /api/project/save` | — | ✅ | 1 |

## Intentional exceptions

| Path | Why it is not a normal per-entity command | Undo | Saved |
|---|---|---|---|
| `section_handler` | Takes the project write lock only to refresh feature caches before reading section lines; no user-visible mutation | — | — |
| `import_ifc` | Bulk import replaces whole aggregates; recorded as `Command::ReplaceProject { before, after }` — one undo step, deliberate memory trade-off | ✅ (single step) | ✅ |
| `POST /api/catalog/import` (`import_catalog_handler`) | Imports an `.omnimlib` into the session-global `LIBRARY`, not into `Project`; there is no project-level undo entry | ❌ | ❌ (library file, not project) |
| `preview_catalog_component` | Builds the instance on a throwaway `Project`; no write lock and no residue in the live model | — | — |
| `/api/genrule` geometry | Command + undo + save are correct, but geometry is still hard-coded mesh rather than feature-tree driven (Wave 3) | ✅ | ✅ |

## Persistence

- **Default on**: `main.rs` sets `save_path = Some(project_file)` unless
  `OMNIM_EPHEMERAL=1|true|yes`, which disables all file writes.
- **Atomic**: `Project::save` writes `{path}.tmp` then renames over the target;
  a crash cannot leave a truncated project file.
- **Failure policy**: a save failure is logged (`Auto-save failed: ...`) and does
  not fail the request — the in-memory mutation and its undo entry already exist.
- **Explicit save**: `POST /api/project/save` flushes on demand and returns the
  path; it responds 400 when persistence is disabled.
- **Object datum fields (P8-S1)**: `elevation`/`level_id` ride the normal
  create/update object commands; the `ModifyObject` undo snapshot covers
  **every** modifiable field (closing a Wave 1 gap where `ifc_class`/`tags`
  edits were not restored) and redo replays the after-image. `POST /api/objects`
  returns **400 with the validator's reason** (previously a bare 500).
- **Level-driven extent cascade (P8-S2)**: changing a storey's `elevation`
  re-resolves every object constrained to it and stores the affected before/after
  images inside the same `Command::ModifyObject` (`cascade`) — **one request, one
  undo step**, redo symmetric. `DELETE /api/objects/{id}` rejects deleting a
  storey still referenced by `level_id`/`bottom_level_id`/`top_level_id` (400).
  The `layered` genrule validates the level pair **before** instantiating, so a
  rejected pair cannot leave an orphan object.

## Verification

```bash
python3 scripts/check_write_paths.py   # 0 direct writes, exit 0
cargo test --workspace                 # includes endpoint-level undo tests
python3 scripts/check_docs.py          # docs link/fence/numbering checks
```

## History

The pre-fix audit snapshot (55 handlers holding the write lock, 13 mentioning the
executor, 5 saving, ~42 direct writes) is preserved in the
[2026-10-27 audit](../status/DOC_AUDIT_2026-10-27.zh-CN.md) and
[CODE_DEVIATIONS §6](../status/CODE_DEVIATIONS.md#6-write-path-persistence--geometry-path-deviations--known-2026-10-27).
