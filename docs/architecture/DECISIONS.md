---
title: "OmniM — Architecture Decision Records"
status: current
last_updated: 2026-11-02
applies_to: "crates/, web/"
evidence: "audited against crates/server/src/lib.rs 2026-10-27"
---
# OmniM — Architecture Decision Records

This is the index of Architecture Decision Records (ADRs) for OmniM.

## How we record decisions

We use a **brief MADR-style format**. Each ADR is a numbered, immutable record placed in this directory
(`doc/architecture/DECISIONS.md` contains the index; individual ADRs live inline in this file for now —
when one grows beyond a page, split it into `adr/NNNN-name.md`).

Format per record:

- **Number & title** (stable, never reused)
- **Status** — one of: Proposed / Accepted / Superseded by ADR-XXXX / Deprecated
- **Context** — what forces a decision (constraints, observed facts, code-reality today)
- **Decision** — the single, verifiable choice actually made in the source tree or protocol
- **Consequences** — what becomes better, what becomes worse, what must be re-decided later

Rules:

1. ADRs describe **what exists**, not what is wished for. A plan-blocked item is better expressed as an
   [RFC](../RFC/) until code embodies the decision.
2. Superseded ADRs are *never deleted* — they are marked Superseded and keep their content intact.
3. Each ADR names at least one file or symbol in the code that embodies it, so a reader can verify
   the decision is real.

---

## ADR Index

| ADR | Title | Status |
|---|---|---|
| [ADR-0001](#adr-0001-protobuf-as-type-schema-json-wire-no-grpc) | Protobuf as type schema (JSON wire, no gRPC) | Accepted |
| [ADR-0002](#adr-0002-slvs-library-for-2d-sketch-constraints) | slvs library for 2D sketch constraints | Accepted |
| [ADR-0003](#adr-0003-truck-kernel-for-brep-modeling) | truck kernel for BRep modeling | Accepted |
| [ADR-0004](#adr-0004-json-file-persistence-single-project-rwlock) | JSON-file persistence (single Project, RwLock) | Accepted |
| [ADR-0005](#adr-0005-hand-written-ifc-parser) | Hand-written IFC parser rather than an ifc crate | Accepted (one implementation note) |
| [ADR-0006](#adr-0006-unified-executorvalidator-for-all-mutation-paths-target) | Unified Executor/Validator for all mutation paths | Accepted — Implemented (2026-10-28) |
| [ADR-0007](#adr-0007-dual-theme-token-layers) | Dual-theme token layers (primitive → semantic → component) | Implemented (2026-11-02) |
| [ADR-0008](#adr-0008-docked-shell-over-floating-panels) | Docked shell over floating panels | Accepted (design) |
| [ADR-0009](#adr-0009-extendable-panel-registry) | Extendable panel registry with tab metadata | Accepted (design) |
| [ADR-0010](#adr-0010-key-based-i18n-with-icu) | Key-based i18n with ICU formatting | Accepted (design) |
| [ADR-0011](#adr-0011-single-command-registry) | Single command registry as the source of shortcuts and menus | Accepted (design) |
| [ADR-0012](#adr-0012-docs-as-code-for-ui-contracts) | Docs-as-code for UI contracts | Accepted (design) |

---

### ADR-0001. Protobuf as type schema (JSON wire, no gRPC)

**Status:** Accepted

**Context**
OmniM's data model must be shareable between a Rust engine and a JS browser client, ideally without runtime
schema disagreement. A traditional answer is gRPC over protobuf binary encoding; that requires codegen per client
language, an HTTP/2 server stack, human-unfriendly wire payloads during debugging, and complicates the current
no-build-tool-simple dev loop (Vite + Vite proxy + Axum).

**Decision**
Use `.proto` files as the **single source of type truth** (`proto/omnim.proto`, `proto/omcom.proto`), generate
Rust types with `prost` into `omnim-omcom`/`omnim-core`, and mirror them as **TypeScript types** in
`web/src/shared/types.ts`. On the wire, all HTTP payloads are **serde-JSON**, not protobuf bytes. **No gRPC is
used anywhere.** Serde-derived domain types that don't map cleanly to protobuf enums stay Rust+serde side (in
`omnim-model`/`omnim-sketch`), never in proto.

**Consequences**
- (+) One central, human-readable schema location; easy to diff and review.
- (+) Payload inspection with curl / browser DevTools is trivial.
- (−) Payloads are larger than binary protobuf.
- (−) No automatic schema validation at wire level; drift is only caught by hand-kept TS types — a known cost.
- (−) No proto-level enum exhaustiveness or field-number safety guarantees are enforcement-checked at runtime.
- If a cloud-era binary transport is needed, the type layer is already in place (planned evolution).

---

### ADR-0002. slvs library for 2D sketch constraints

**Status:** Accepted

**Context**
The sketch editor needs real geometric constraint solving (coincidence, distance, parallelism, radius on lines,
circles, arcs, workplanes). Writing a bespoke solver is a long-term algorithmic burden; the Rust ecosystem
offers `slvs`, idiomatic bindings to the proven SolveSpace geometry kernel.

**Decision**
Use the `slvs = "0.6.0"` crate (`omnim-sketch/src/solver.rs`) to drive 2D constraint solving. Sketch entities are
modeled as `slvs` handles (`EntityHandle<Point>` / `LineSegment` / `Circle` / `ArcOfCircle` / `Workplane`) plus
OmniM's own wrapper data in `lib.rs`/`profile.rs`. OmniM does not implement its own constraint math.

**Consequences**
- (+) Production-quality 2D constraint solving without owning the algorithms.
- (+) Clear mapping between OmniM sketch concepts and SolveSpace primitives.
- (−) Conceptual leak: solver entity kinds are imported into OmniM's type vocabulary.
- (−) Bound to the solver's capability and semantics; unsupported constraint types require solver changes upstream.
- (−) `slvs` handles are not trivially serializable in a `.omnim` JSON — sketches must be persisted at the
  OmniM-schema level and rehydrated into handles on load.

---

### ADR-0003. truck kernel for BRep modeling

**Status:** Accepted

**Context**
Engineering objects need solid modeling (extrusions, profiles, booleans, shells/solids/wires) plus tessellation
for display. The available open-source Rust options are `truck` (B-rep) and a handful of smaller / incomplete
alternatives; OCCT bindings exist but add C++/FFI complexity that conflicts with the pure-Rust crate layout.

**Decision**
Adopt the **truck** family — `truck-modeling`, `truck-polymesh`, `truck-topology`, `truck-base` (`wgpu`-friendly)
— for all solid/BRep work in `omnim-model/src/brep.rs` and mesh generation in `mesh_gen.rs`. OmniM does not
reimplement CSG or NURBS math.

**Consequences**
- (+) Real BRep semantics (Shell/Solid/Wire) without binding to a large C++ kernel.
- (+) `truck-polymesh` fits the mesh-to-three.js delivery format naturally.
- (−) Faithful to truck's feature scope; some CAD-quality operations still immature or missing upstream.
- (−) Mixed `truck_modeling` versions are pulled in transitively; version-bumping must be done together.

---

### ADR-0004. JSON-file persistence (single Project, RwLock)

**Status:** Accepted

**Context**
Current deployment target is a single-user local server. The simplest correct persistence is a full representation
of the application state synced to one file. Requirements like multi-user, database-backed version history, or
branching, are explicitly **planned** but not implemented (see [PLATFORM_ARCH.md §9](PLATFORM_ARCH.md#9-future-directions-not-yet-implemented)).

**Decision**
Persist the **entire `Project`** as a single `.omnim` JSON file. At startup, the file is loaded from `OMNIM_PROJECT` (default `project.omnim`, resolved against the server's CWD).
Saving is **on by default** (2026-10-28): `save_path = Some(project_file)` unless `OMNIM_EPHEMERAL=1`. **Every accepted mutation auto-saves** through `mutation::persist`, and `Project::save` is atomic (`{path}.tmp` → rename). `POST /api/project/save` flushes on demand. There is no incremental journal, no WAL, no snapshot rotation. Public API surface: `Arc<RwLock<Project>>` guards state (see `server/src/lib.rs`, `model/src/project.rs`). See [WRITE_PATHS.md](WRITE_PATHS.md).

**Consequences**
- (+) Trivial to inspect, diff, copy, and hand-edit files; zero infra.
- (+) State is always readable, even across version mismatches (serde defaults fill in gaps).
- (−) Whole-file rewrite per save is O(project size); tmp+rename prevents truncation, but large projects can still block a request — no
  atomic-rename pattern is currently used.
- (−) No multi-process safety: two servers sharing the file corrupt it.
- (−) No real history. Undo/redo lives only in-memory in `Executor` state.
- **These limits bound OmniM to desktop/local deployment today.** Cloud/DB support is planned, not present.

---

### ADR-0005. Hand-written IFC parser rather than an ifc crate

**Status:** Accepted

**Context**
OmniM must ingest and emit IFC/STEP files (IFC 2x3 / IFC 4 subsets). Ecosystem crates for IFC exist but either
target only read paths, pin heavy dependencies, or scope their domain to specific schemas; they also make
custom partial-coverage semantics awkward (OmniM only needs the entities it can consume/produce).

**Decision**
Write the **parser and serializer by hand** in `omnim-ifc` (no IFC crate dependency). The parser maps a chosen
subset of IFC 2x3/4 records and geometry, controlled explicitly in `lib.rs`/`geometry.rs`. Non-supported
entities are **skipped**, not fail-whole. The ADR's original intent was that imports be converted into `OmniCommand`s
so nothing bypasses the mutation pipeline (see ADR-0006); **the implementation does not do this today** (see the
implementation note below).

**Consequences**
- (+) Zero external IFC dependency; precise control over which subset is produced/consumed.
- (+) Errors are legible to us; mappings match our object model directly.
- (−) Ongoing cost: IFC coverage grows only as fast as we hand-code it.
- (−) Robustness against non-standard IFC producers is limited (parser is tested against real files in the repo
  `fixtures/ifc/`, e.g. `simple.ifc`, `test_building.ifc`, `part.ifc`).
- Revisit if/when schema coverage requirements outgrow the hand-rolled subset.

> **Implementation note (updated 2026-10-28).** `POST /api/import/ifc` still parses IFC text and inserts the
> resulting objects, materials, and lifecycle records directly (`mode=replace` overwrites the project), but it now
> records `Command::ReplaceProject { before, after }` on the Executor: the import is **one undoable transaction**
> and persists. It remains a bulk exception rather than a per-entity command stream. See
> [WRITE_PATHS.md](WRITE_PATHS.md).

---

### ADR-0006. Unified Executor/Validator for all mutation paths

**Status:** Accepted — Implemented (2026-10-28)

**Context**
Mutations originate from *several* sources: REST handlers, console (Omcom console in web UI), and the AI intent
engine (AMIR's generated `OmniCommand`s). A naive design would let each source touch `Project` directly, causing
inconsistent undo semantics, duplicated validation logic, and an easy path for an AI hallucination to silently
mutate things it shouldn't.

**Decision**
There is exactly **one** mutation pipeline: build `OmniCommand` → `Validator::validate(&cmd, &Project)`
→ `Executor::execute(&cmd, &mut Project)` → record undo/redo history. **Every** origin (`omnim-server` handlers,
`omnim-omcom` console path, `omnim-amir` generator output) constructs `OmniCommand`s and routes them through the
same Validator/Executor. No path writes to `Project` sidewise (except read-only helpers).

**Consequences**
- (+) Uniform undo/redo semantics regardless of mutation origin.
- (+) AI-generated mutations get the same validation as human ones: **an hallucinated or unsafe command fails
  loudly** and returns errors to the AI (feedback loop in `amir::feedback`) instead of corrupting state.
- (+) One place to add business rules or audit hooks later.
- (−) Every mutation costs one command serialization + validation round — acceptable at the scale of one
  in-memory project.
- (−) Server-side handler code is more verbose (building command structs), a tradeoff accepted for correctness.

**Deviation (2026-10-27) — status is target, not current.**

A static scan of `crates/server/src/lib.rs` (Appendix A of [DOC_AUDIT_2026-10-27](../status/DOC_AUDIT_2026-10-27.zh-CN.md)) found:

- **55** handlers hold `project.write()`;
- only **13** of them mention the executor;
- **~42** mutate `Project` directly, with no undo entry; and
- only **5** call `Project::save(...)` — and none of the executor-path handlers do. With the default
  `project.omnim` path, `save_path` is `None`, so the default launch persists nothing.

Representative direct writes: `update_object`, `transform_object`, `set_parent`, `connect`, `create_material` /
`assign_material`, `update_sketch` / `solve_sketch`, `update_sketch_entity`, `import_ifc`,
`instantiate_from_catalog`, and all series edit handlers. The full matrix is in [WRITE_PATHS.md](WRITE_PATHS.md);
the deviations are tracked in [CODE_DEVIATIONS.md §6](../status/CODE_DEVIATIONS.md#6-write-path-persistence--geometry-path-deviations--known-2026-10-27).
Restoring this invariant is a roadmap item, not something this ADR can assert today.

**Implementation (2026-10-28) — deviation closed.**

Wave 1 restored the invariant. Every mutating REST handler now builds an `OmniCommand` and calls
`mutation::apply` (or, for a handful of lock-owning handlers, delegates to `Executor::execute` / `record_*`).
`mutation::persist` saves atomically after each successful mutation. `scripts/check_write_paths.py` reports
**0** non-allowlisted `project.write()` calls and runs in CI.

Two documented exceptions remain, neither of which writes `Project` outside the pipeline:
`import_ifc` records a single `Command::ReplaceProject { before, after }` (whole-project transaction), and
`POST /api/catalog/import` mutates the session-global `LIBRARY` (`.omnimlib` bulk import) rather than `Project`.
See [WRITE_PATHS.md](WRITE_PATHS.md) for the current matrix.


### ADR-0007. Dual-theme token layers

**Status**: Implemented (2026-11-02): tokens consolidated in `globals.css`, zero literal colors in components.

**Context**: `styles.css` contains 712 hardcoded colors and the token block is duplicated in `globals.css`.
The product needs dark and light themes without a second visual language.

**Decision**: Three layers (primitive → semantic → component). Themes are redefinitions of the semantic
layer under `[data-theme]` on `<html>`. Components may only consume semantic or component tokens. The 3D
scene reads a `ViewerPalette` derived from the same theme.

**Consequences**: A one-time migration of ~4,000 CSS lines; a scan (`check_ui_contracts.py --tokens`) keeps
literals out; high contrast becomes a third semantic mapping rather than a third design.

**Alternatives**: CSS-in-JS theming (adds a large dependency); single dark theme (rejected: projectors and print).

### ADR-0008. Docked shell over floating panels

**Status**: Accepted (design) — not implemented.

**Context**: The properties panel defaults to a centered floating overlay (`appStore.ts:89`), so selecting an
object covers the viewport. Layout is local component state and is lost on reload.

**Decision**: Seven fixed regions with docks as siblings of the viewport, resizable and collapsible. Floating
mode remains as an escape hatch. Layout persists per user in `localStorage`, never in the project file.

**Consequences**: The shell rewrite must land before panel polish; a `layoutStore` replaces ad-hoc state.

**Alternatives**: Restyle the floating panel (does not fix occlusion); detachable browser windows (out of scope).

### ADR-0009. Extendable panel registry

**Status**: Accepted (design) — partially implemented.

**Context**: `PanelRegistry` already registers panels by position and order, but has no tabs, icons, default
widths, or persistence hooks.

**Decision**: Extend `PanelRegistration` with `icon`, `tabGroup`, `defaultWidth`, `minWidth`, `collapsible`,
and `keepMounted`, keeping existing registrations backward compatible.

**Consequences**: Panel authors declare intent once; the shell renders tabs, splitters, and restore logic.

**Alternatives**: Hard-code panels in the shell (rejected: blocks domain panels); a plugin API (premature).

### ADR-0010. Key-based i18n with ICU

**Status**: Accepted (design) — not implemented.

**Context**: 858 CJK runs across 31 files are inline; there is no locale switch and no fallback.

**Decision**: Externalize every user-visible string into per-locale catalogues accessed by `t(key)`. Use ICU
MessageFormat for plurals, numbers, dates, and units. Default `zh-CN`, fallback `en-US`.

**Consequences**: A large mechanical migration; a scan blocks new literals; pseudo-localization becomes a
release check; object-name prefixes stay ASCII by contract.

**Alternatives**: Runtime translation (unreliable); a third-party i18n library (acceptable, but the catalogue
contract matters more than the library).

### ADR-0011. Single command registry

**Status**: Accepted (design) — not implemented.

**Context**: Shortcuts are handled in `App.tsx` with a `switch`, while the console, sketch editor, tree, and AI
panel each implement their own key handlers. Menus, ribbon, and context menus duplicate availability logic.

**Decision**: One registry defines commands (`id`, `labelKey`, `group`, `shortcut`, `enabled`, `run`). Menus,
ribbon, palette, and context menus render from it. KEYBOARD_MAP mirrors it and is checked in CI.

**Consequences**: Documentation cannot drift silently; conflicts are detected at load; the sketch editor must
migrate onto the same registry.

**Alternatives**: Per-component listeners (current state, drift-prone); a third-party hotkey library (still
needs the command model).

### ADR-0012. Docs-as-code for UI contracts

**Status**: Accepted (design) — partially implemented.

**Context**: Documentation previously described features that did not exist (see the 2026-10-27 audit) and
referenced files that had been deleted.

**Decision**: Every UI requirement has an ID, a contract anchor, a verification artifact, a gate, and a status.
`check_docs.py` enforces structure and bilingual parity; `check_ui_contracts.py` enforces ID and debt rules;
`gen_keymap.py --check` enforces shortcut-table consistency.

**Consequences**: CI becomes the arbiter of documentation truth; adding a requirement has a fixed cost; debt
counts are visible and must trend down.

**Alternatives**: Prose-only documentation with review discipline (already failed once).
