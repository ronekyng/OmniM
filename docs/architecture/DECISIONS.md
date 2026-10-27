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
| [ADR-0005](#adr-0005-hand-written-ifc-parser) | Hand-written IFC parser rather than an ifc crate | Accepted |
| [ADR-0006](#adr-0006-unified-executorvalidator-for-all-mutation-paths) | Unified Executor/Validator for all mutation paths | Accepted |

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
On every accepted mutation (`register` handler), server auto-saves the file again; there is no incremental
journal, no WAL, no snapshot rotation. Public API surface: `Arc<RwLock<Project>>` guards state (see
`server/src/lib.rs`, `model/src/project.rs`).

**Consequences**
- (+) Trivial to inspect, diff, copy, and hand-edit files; zero infra.
- (+) State is always readable, even across version mismatches (serde defaults fill in gaps).
- (−) Whole-file rewrite per mutation is O(project size); a crash mid-write can truncate the file — no
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
entities are **skipped**, not fail-whole. Imports are converted into `OmniCommand`s so nothing bypasses the
mutation pipeline (see ADR-0006).

**Consequences**
- (+) Zero external IFC dependency; precise control over which subset is produced/consumed.
- (+) Errors are legible to us; mappings match our object model directly.
- (−) Ongoing cost: IFC coverage grows only as fast as we hand-code it.
- (−) Robustness against non-standard IFC producers is limited (parser is tested against real files in the repo
  `fixtures/ifc/`, e.g. `simple.ifc`, `test_building.ifc`, `part.ifc`).
- Revisit if/when schema coverage requirements outgrow the hand-rolled subset.

---

### ADR-0006. Unified Executor/Validator for all mutation paths

**Status:** Accepted

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
