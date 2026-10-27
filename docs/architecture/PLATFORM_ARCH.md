# OmniM — Current Architecture

> **Scope note:** This document describes the **actual, implemented architecture** of OmniM as it exists in the codebase today.
> The design-phase, cloud-first vision (multi-tenancy, shared cloud workspaces, enterprise lifecycle services) lives in
> [`docs/specification/`](../specification/) and [`docs/RFC/`](../RFC/). Where this document says something is
> "planned" or "not yet implemented", it does **not** exist in the code yet.

---

## 1. Overview & Design Goals

OmniM is an **AI-native engineering lifecycle platform**: a local-first desktop-class application architecture delivered
today as a **web SPA + a Rust REST server**, with the engineering domain (BIM/PLM/CAD) modeled in Rust workspace crates.

The platform lets a user:

- Build and edit a project tree of engineering objects (with properties, materials, tags, relations, measurements).
- Author 2D sketches with a real constraint solver, and generate profiles from them.
- Model solids with a BRep kernel and generate triangle meshes for display.
- Instantiate components from a PARAGON-style parameter-driven **component catalog** and SPECON-style **series library**.
- Import/export partially covered **IFC 2x3 / IFC 4** subsets via STEP files.
- Drive the model **in natural language** through an AI intent engine that generates structured commands.

Concrete design goals visible in the implementation:

1. **Protocol-first types, JSON on the wire.** Protobuf (`.proto` files) defines the message schemas, but the runtime uses
   **serde-JSON only** — there is no gRPC, no runtime protobuf encoding (see §6).
2. **One engine, many frontends.** The engine is a set of pure Rust crates; the browser UI is one consumer of it, and a
   native wgpu viewer is another (standalone) one.
3. **Every mutation goes through one gate.** All mutating paths — REST handlers, AI-generated commands — produce an
   `OmniCommand`, which is validated and executed by the **same `Validator` + `Executor`** with full undo/redo.
4. **No hidden infrastructure.** No database, no WebSocket, no message broker. The entire application state is one
   `Project` value held in memory and persisted as a single JSON file.

---

## 2. Layer Diagram (as implemented)

```
┌────────────────────────────────────────────────────────────────────┐
│                        Web SPA  (web/, Vite build)                  │
│  React 19 + TypeScript · react-router v7 · Zustand · React Query     │
│  three.js viewer · sketch editor · catalog/series/library managers  │
│  AI chat · Omcom console        ── typed REST client (shared/api.ts) │
└───────────────────────────────────┬────────────────────────────────┘
                        HTTP/JSON   │  /api/*   (Vite dev proxy → :3000)
┌───────────────────────────────────▼────────────────────────────────┐
│                     omnim-server  (Axum)                            │
│  ~100 REST routes · AppState = Arc<RwLock<Project>>                 │
│  OmniCommand construction · auto-save to JSON file on mutation      │
│  static file serving for web/ build                                  │
└───────┬───────────────────┬────────────────────┬───────────────────┘
        │                   │                    │
┌───────▼───────┐   ┌───────▼────────┐   ┌───────▼────────┐
│ omnim-omcom   │   │  omnim-amir    │   │   omnim-ifc    │
│ Validator +   │   │  AI intent     │   │  hand-written  │
│ Executor,     │   │  engine        │   │  STEP/IFC      │
│ undo/redo     │   │  (LLM HTTP)    │   │  parser        │
└───────┬───────┘   └───────┬────────┘   └───────┬────────┘
        │                   │                    │
        └───────────────────▼────────────────────┘
                            │
┌───────────────────────────▼────────────────────────────────────────┐
│   omnim-model (Project aggregate, catalog/series, feature tree,     │
│   truck BRep kernel, mesh generation, lifecycle/breakdown, etc.)    │
│─── omnim-sketch (slvs constraint solver, entities, profiles)        │
│─── omnim-core  (prost-generated proto types, serde-JSON)            │
└────────────────────────────────────────────────────────────────────┘
        │
        ▼
┌────────────────────────────────────────────────────────────────────┐
│  Persistence: a single .omnim JSON file                             │
│  (default project.omnim; overridable with OMNIM_PROJECT)            │
│  No database. No version history store.                            │
└────────────────────────────────────────────────────────────────────┘

(native wgpu/winit viewer: removed 2026-10-27 as an orphan crate;
 fetches /api/mesh — standalone, NOT used by the web frontend)
```

---

## 3. Crate Architecture & Responsibilities

The workspace defines all crates in the root `Cargo.toml`. Dependency DAG:

```
omnim-core
    ▲
omnim-sketch
    ▲
omnim-model ──▲── si si
    ▲             │
    ├── omnim-omcom ──┐
    ├── omnim-ifc  ───┼──► omnim-server
    └── omnim-amir ───┘
            (sketch also → server for sketch endpoints)

omnim-renderer: removed 2026-10-27 (was depending on omnim-core + omnim-model only)
                (standalone native viewer, not wired into server/web)
```

| Crate | Size (approx. LOC in `src/`) | Responsibility |
|---|---|---|
| `omnim-core` | very small | Prost-generated Rust types from `proto/omnim.proto` and `proto/omcom.proto`. Types are `(de)serialized` with Serde **JSON only**; no gRPC, no runtime protobuf. Thin foundation shared by everything. |
| `omnim-sketch` | ~1,350 | 2D sketch domain: sketch entities (points, lines, circles, arcs on workplanes), profiles, and constraint solving backed by the `slvs` (SolveSpace) library (`solver.rs`). |
| `omnim-model` | **largest crate (~7,700)** | The `Project` aggregate and entire engineering domain: object tree, properties (`property_def.rs`), feature tree + `GenRule` generators (`feature.rs`, `feature_template.rs`), PARAGON-style component catalog (`catalog.rs`, `seed_catalog.rs`), SPECON-style series library (`series_store.rs`, `library.rs`), truck-based BRep modeling (`brep.rs`), triangle-mesh generation (`mesh_gen.rs`), materials, tags, relations, measurements, lifecycle records, administrative (documents/versioning records), breakdown (product-breakdown structures), Java-influenced component store, and **JSON `.omnim` file persistence** (`project.rs`). |
| `omnim-omcom` | ~3,650 | The command layer. `Validator::validate(&OmniCommand, &Project)` → list of errors; `Executor::execute(&OmniCommand, &mut Project)` → `ExecutionResult`, with a **full undo/redo stack and transaction grouping**. Every legitimate mutation of the project is an `OmniCommand`. |
| `omnim-ifc` | ~1,590 | **Hand-written** STEP/IFC file parser (no IFC crate dependency), import and export, partially covering **IFC 2x3 and IFC 4**. Includes IFC geometry mapping. |
| `omnim-amir` | ~2,300 | AI intent engine ("AMIR"): natural-language → intent → `OmniCommand` generation (`generator.rs`, ~1,018 LOC), prompt parsing, context building, **reverse generation** (describe an existing model in NL, `reverse.rs`), templates, feedback. LLM providers via reqwest HTTP: **OpenAI / Anthropic / Mock**. |
| `omnim-server` | ~3,700 | Axum HTTP server. ~100 REST routes under `/api/*`, `AppState` = `Arc<RwLock<Project>>` + `Arc<RwLock<Executor>>` + save path. Builds `OmniCommand`s for requests, auto-saves the JSON project file after mutations, serves the built `web/` static directory. Binds `0.0.0.0:3000` by default (`OMNIM_HOST`/`OMNIM_PORT` overridable). **No WebSocket, no database.** |
*`omnim-renderer` (a standalone native wgpu/winit viewer) was removed from the workspace on 2026-10-27 as dead weight — it was depended on by nothing; a native viewer would be re-proposed via RFC.*

> Note: `omnim-core` provides the manifest types (shared enums, `Version`, etc.); the concrete `Project` and its
> persistence live entirely in `omnim-model`.

### Sizing caveat

Line counts above are indicative (as of the last survey of `crates/*/src/*.rs`); the model crate dominates at roughly
a third of total engine code, confirming that the domain model — not infrastructure — is the project's center of mass.

---

## 4. Key Data Flow: HTTP request → mutation → JSON save

One representative mutation path (e.g. renaming an object via `PUT /api/objects/:id`):

```
Browser (React)  ──PUT /api/objects/{id}──▶  Axum route handler
                                               │  acquires state.project.write()
                                               │  acquires state.executor.write()
                                               │  builds OmniCommand{action, operations...}
                                               ▼
                                       omnim-omcom Executor::execute()
                                               │  validates via Validator first
                                               │  mutates omnim-model::Project
                                               │  records undo/redo entry
                                               ▼
                                     Project updated in memory
                                               ▼
                                if state.save_path is Some:
                                   Project::save(path)  → .omnim JSON file rewritten
                                               ▼
                                  200/4xx JSON response to browser
```

Conventions in the handler code:

- Read routes take `state.project.read().await`; mutating routes take both `project.write()` and `executor.write()`.
- Most mutating routes do **not** construct raw operation objects; instead they call the Executor with an
  `OmniCommand` assembled inline (action enum + operations list from `omcom.proto`), keeping the undo stack
  consistent across every write.
- Auto-save failure is logged to stderr and does not fail the request (`save_path` is `None` when the project
  file is the default, meaning the running server may deliberately not persist).

Full end-to-end flows (click→render, NL→command, IFC in/out, catalog instantiation, save/load) are documented in
[DATA_FLOW.md](DATA_FLOW.md).

---

## 5. Rendering Strategy

OmniM deliberately has **two independent renderers**, and they do not share code:

| Renderer | Technology | Where used | Data source |
|---|---|---|---|
| Browser 3D/2D viewers | `three.js` (+ view-cube helper) in `web/src/app/components/ThreeViewer.tsx` (~1,750 LOC), 2D sketch canvas in `web/src/sketch` | The main web SPA | Fetches triangle meshes from REST endpoints (e.g. `/api/mesh`, per-object mesh endpoints) and renders client-side |
| (Native viewer | removed 2026-10-27: `omnim-renderer` was deleted as an orphan crate; re-introduction would go through an RFC) | | |

The key property of both: **the server exposes geometry over REST; rendering is fully client-side and server-code-agnostic.**
Neither renderer consumes server-side rasterization; there is no server-side GPU pipeline.

---

## 6. Protobuf as Type Schema (JSON wire format)

Almost unique to the design: `.proto` files are the **single source of truth for message schemas**, but nothing on the
wire uses protobuf binary encoding.

- `proto/omnim.proto` and `proto/omcom.proto` define all domain/command types.
- `omnim-core` runs `prost` to generate Rust structs with reflexive Serde derives.
- Client (TS) mirrors these in `web/src/shared/types.ts`, and the REST layer always exchanges them as **JSON over HTTP**.
- Additional serde-derived domain types live directly in `omnim-model`/`omnim-sketch` (not in proto), especially
  types rich in Rust-native data that maps awkwardly to protobuf enums.

Consequences: schema evolution and cross-language contracts stay centralized in one `.proto` location, while avoiding the
operational need for gRPC framing, codegen for every client language, and binary inspection tooling — at the cost of
larger payloads (JSON vs. binary protobuf) and losing proto3's cross-language binary round-trip guarantees.

When a true multi-user/cloud backend arrives (see §8), a real gRPC or binary transport layer is the natural evolution —
the types are already in place.

---

## 7. Persistence Model

**Current: single JSON file, single in-memory project.**

- The constructor `omnim_server::ServerConfig { project, save_path, ... }` decides whether auto-save is on.
- Mutations call `Project::save(path)`, rewriting the whole `.omnim` file; loads read the same file on startup.
- `.omnim` files (`project.omnim` at the repo root, `fixtures/my-project.omnim`) are example dumps.

**Limits of this model** (these are real, not planned removal):

- **Single-process, single-project.** There is exactly one `Project`, so two browser tabs share the same in-memory state.
- **No multi-writer safety.** `Arc<RwLock<Project>>` serializes writes correctly *within one process*, but two server
  instances cannot share a file; the file is not transactional and corrupts on crash mid-write.
- **No history.** The undo/redo stack is in-memory only; the JSON file stores only the current state.
- **No concurrent branch/merge.** Nothing like Git for models exists.

These limits constrain OmniM to a **local/desktop-class** deployment today.

---

## 8. Security / Concurrency Model

- **Concurrency:** a single `tokio::sync::RwLock<Project>` guards all state. Interior writes (e.g. building an
  `OmniCommand` inside a read-lock-held section) are avoided; mutating routes acquire `write()` along with
  `executor.write()` to stamp undo history. This is correct but entirely single-process — it is not a distributed
  lock and gives no cross-server coordination.
- **Security:** the Axum layer performs **no authentication or authorization** — routes accept any request that can
  reach the port. This is intentional for a local dev/demo server and is listed as a known, planned gap (see §9).
- **Trust boundaries:** because LLM-generated commands flow through the **same** `Validator` as user-driven commands,
  the AI path cannot bypass domain validation — an important structural safety property (see ADR-0006).
- **CORS/egress:** default bind is `0.0.0.0:3000` (fine for a personal machine, exposes a LAN surface today);
  `omnim-amir` makes outbound HTTPS calls LLM providers via `reqwest`.

---

## 9. Future Directions (NOT YET IMPLEMENTED)

Everything below is **design intent only**; none of it exists in code today. It is tracked in `docs/specification/`
and `docs/RFC/`:

| Direction | Status | Where it would live |
|---|---|---|
| Cloud / multi-user workspaces, real-time collaborative editing (WebSocket/CRTD-class synchronization) | **planned** | `docs/specification/` |
| Database-backed persistence with version history / snapshot stores (replacing/beside the JSON file) | **planned** | `docs/specification/` |
| WebAssembly build of the engine crates, so the browser SPA could run the same Rust domain logic in-process (no server round-trips) | **planned** | `docs/RFC/` |
| Desktop packaging (e.g. Tauri shell) and mobile apps | **planned** | `docs/RFC/` |
| Service-boundary evolution: server-per-workspace, gRPC between services | **planned** | `docs/RFC/` |

None of these should be assumed when reading code under `crates/` or `web/` today.
