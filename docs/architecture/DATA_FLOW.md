---
title: "OmniM — Data Flows"
status: current
last_updated: 2026-10-27
applies_to: "crates/, web/"
evidence: "audited against crates/server/src/lib.rs 2026-10-27"
---
# OmniM — Data Flows

This document describes the **end-to-end data flows that actually exist in the code today**, based on
`crates/server/src/lib.rs`, `crates/omcom/src/lib.rs`, `crates/amir/src/generator.rs`, and `crates/ifc/`.
Where a mechanism is only planned, it is explicitly labeled **planned**.

The architectural overview lives in [PLATFORM_ARCH.md](PLATFORM_ARCH.md); this page zooms in on *how data moves*.

---

## 0. The Universal Mutation Rule — TARGET, not current

The **target** (ADR-0006) is that every write to the `Project` — whether triggered by a mouse click, an AI answer, or
a REST call — becomes an `OmniCommand` and flows through:

```
OmniCommand ──▶ Validator::validate(cmd, &Project)──▶ (gather errors; abort if any)
            ──▶ Executor::execute(cmd, &mut Project) ─▶ ExecutionResult + undo/redo entry
```

**Current state matches this (2026-10-28).** Every mutating handler builds an `OmniCommand` (or delegates to the
`Executor`); direct writes went 42 → 0, every mutation persists atomically, and `scripts/check_write_paths.py`
enforces the invariant. See [WRITE_PATHS.md](WRITE_PATHS.md) for the per-endpoint matrix and the two documented
exceptions (`import_ifc` as one whole-project command, `.omnimlib` import into the global library).

---

## 1. (a) User click → REST → Executor → model mutation → mesh → three.js render

The bread-and-butter flow, e.g. "create a wall" clicked in the toolbar:

```
┌─ Browser ──────────────────────────────────────────────────────┐
│ React component (e.g. MainToolbar.tsx)                          │
│   │ user click                                                  │
│   ▼                                                             │
│ typed REST client (shared/api.ts) → POST /api/objects  (JSON)   │
│   … Zustand store / React Query invalidates affected queries     │
└──────────────────────────────┬──────────────────────────────────┘
                               │ HTTP /api/*
┌──────────────────────────────▼──────────────────────────────────┐
│ omnim-server (Axum)                                             │
│   • route handler acquires project.write() + executor.write()   │
│   • assembles OmniCommand{ action, operations:[…] }             │
│   • Executor::execute() → validates → mutates Project           │
│   • no save: POST /api/objects is not a saving handler          │
│   • returns object/JSON (or error status)                       │
└──────────────────────────────┬──────────────────────────────────┘
                               │ 200 OK
       (browser again)         │
                               ▼
      React Query refetches the object tree → mesh fetch
                               │
┌──────────────────────────────▼──────────────────────────────────┐
│ omnim-model                                                      │
│   geometry.rs / brep.rs build solids (truck kernel)              │
│   mesh_gen.rs tessellates → triangle mesh (positions/normals)    │
└──────────────────────────────┬──────────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────────┐
│ three.js renderer (web/src/app/components/ThreeViewer.tsx)       │
│   loads mesh + object transform, updates scene graph, renders    │
└─────────────────────────────────────────────────────────────────┘
```

Notes grounded in code:

- The **mesh is fetched repeatedly from the server** — geometry regeneration happens Rust-side in
  `omnim-model` (mesh_gen.rs), and the browser is a pure display client.
- There is **no push channel**: after each mutation the UI re-pulls state (queries invalidated after the
  call resolves). No WebSocket exists in the codebase.
- Every mutating endpoint now follows this path: rename, transform, reparent, connect, material assignment,
  sketch update/solve, IFC import, catalog instantiate and series edits are all commands with undo and save —
  see [WRITE_PATHS.md](WRITE_PATHS.md). IFC import is the one whole-project command (`Command::ReplaceProject`).

### Mermaid version

```mermaid
sequenceDiagram
    participant U as UI (React)
    participant S as omnim-server (Axum)
    participant O as omnim-omcom
    participant M as omnim-model
    participant R as three.js
    U->>S: POST /api/objects (JSON)
    S->>O: OmniCommand → validate → execute
    O->>M: mutate Project
    O-->>S: ExecutionResult
    Note over S,M: no save for this handler
    S-->>U: 200 JSON
    U->>M: GET /api/mesh /api/tree
    M-->>U: mesh + tree JSON
    U->>R: scene update + render
```

---

## 2. (b) Natural language → AMIR engine → OmniCommand → Validator → Executor

```mermaid
sequenceDiagram
    participant U as AI Chat (AiChat.tsx)
    participant S as omnim-server /api/ai/*
    participant A as omnim-amir
    participant L as LLM provider (OpenAI/Anthropic/Mock)
    participant O as omnim-omcom
    U->>S: POST user message
    S->>A: context + prompt intent request
    A->>L: HTTPS (reqwest) chat completion
    L-->>A: structured intent / command draft
    A->>A: generator.rs: intent → OmniCommand
    A-->>S: OmniCommand list
    S->>O: Validator::validate + Executor::execute
    O-->>S: ExecutionResult (errors fed back via feedback.rs for retry)
    S-->>U: result + human-readable summary
```

What happens in code:

1. **Context assembly** (`amir::context`): a snapshot of the relevant `Project` state (selected objects, catalog info,
   recent history) is built so the LLM can ground its answer.
2. **Provider abstraction** (`amir::lib`): provider is selected by configuration value `"openai" | "anthropic"`, with a
   `Mock` provider for offline/testing. Communication is **plain HTTPS** via `reqwest` — no SDK, no local model runtime.
3. **Intent → command** (`amir::generator.ts`): a large rule + template file maps intents onto concrete `OmniCommand`
   payloads; `prompt_parser.rs` handles looser free-form input, `templates.rs` holds response templates, and
   `feedback.rs` turns Executor errors back into retryable AI feedback.
4. **Same gate:** the generated commands are **not** applied directly to the `Project`; they enter the same
   Validator/Executor as user clicks (see §0). This is the key safety property — a hallucinated command fails loudly
   instead of corrupting state.
5. **Reverse direction** (`amir::reverse`): the engine can also describe an existing `Project` in natural language
   (documentation/summarization, not mutation).

---

## 3. (c) IFC import / export

```mermaid
sequenceDiagram
    U as Web UI (Library manager / import actions)
    S as omnim-server (import/export)
    I as omnim-ifc (hand-written STEP parser)
    M as omnim-model
    Note over I,M: EXPORT path
    U->>S: GET /api/export/ifc
    S->>M: read current Project
    M-->>S: objects/geometry
    S->>I: serialize Project → STEP text
    I-->>S: IFC 2x3 / IFC 4 subset text
    S-->>U: file download

    Note over I,M: IMPORT path
    U->>S: POST /api/import/ifc (raw IFC text)
    S->>I: parse text (hand-written, no IFC crate)
    I-->>S: parsed IFC records + mapped geometry (geometry.rs)
    S->>M: snapshot before, then insert parsed objects/materials/lifecycle
    S->>M: record Command::ReplaceProject (before, after)
    M-->>S: project replaced (single undo step, persisted)
    S-->>U: imported objects
```

Facts from the code:

- `omnim-ifc` contains a **hand-written parser and serializer** (no `ifc`-ecosystem Rust crate is used — ADR-0005).
- Coverage is **partial** — a subset of **IFC 2x3 and IFC 4** entities/attributes; unsupported entities are skipped
  rather than rejected wholesale.
- **Import is one whole-project transaction.** `import_ifc` inserts parsed objects/materials/lifecycle records
  (`mode=replace` overwrites the project) and records `Command::ReplaceProject { before, after }`; undo restores the
  pre-import project, redo re-applies, and the result persists. See [WRITE_PATHS.md](WRITE_PATHS.md).
- The request body is **raw IFC text**, not multipart — `curl --data-binary @file.ifc`, not `curl -F`.
- Sample IFC files under `fixtures/ifc/` (`simple.ifc`, `test_building.ifc`, `part.ifc`, …) exercise this flow.

---

## 4. (d) Catalog / series instantiation

The PARAGON-style **component catalog** and SPECON-style **series library** live in `omnim-model`
(`catalog.rs`, `seed_catalog.rs`, `series_store.rs`, `library.rs`). Catalog definitions are served from the
library/seed catalog; `Project` stores series (`Project.series`) and external series references, **not** a
`catalog` field.

```mermaid
flowchart TD
    A[Seed catalog / library<br/>seed_catalog.rs] -->|served to UI| U[Web UI catalog & series panels]
    U -->|POST /api/catalog/instantiate| S[omnim-server]
    S -->|InstantiateCatalog command<br/>executor + undo + save| M[omnim-model::Project]
    M --> F[feature tree / feature_template.rs<br/>official geometry path]
    F --> G[geometry.rs / brep.rs → mesh_gen.rs]
    G --> J[(mesh JSON served to three.js)]
```

Narrative:

1. **Catalog entry:** each catalog component declares a parameter schema and default parameters (PARAGON-style
   parameter-driven part definition).
2. **Series:** a series binds one catalog component to a family of dimensioned variants (SPECON-style), so a user picks
   a **series + size** rather than raw dimensions.
3. **Instantiation:** `POST /api/catalog/instantiate` runs as an `InstantiateCatalog` command through the
   Executor: the created object and the sketches generated by the template are recorded together, so undo/redo and
   save are atomic. It still creates an `ObjectType::Component` (not `Wall`/`Beam`/…), with parameters filled from
   the chosen series record — unifying that type is Wave 3 work.
4. **Generation:** the new object's feature tree (`GenRule` generator, `feature_template.rs`) drives geometry building
   (`brep.rs` via the truck kernel) and tessellation (`mesh_gen.rs`), which is then served to the three.js viewer.

**Geometry-path split (important).** The catalog feature tree → BRep/mesh pipeline is the **official geometry path**.
The toolbar's `/api/genrule` route is a **legacy/transitional pass-through**: it emits hardcoded mesh primitives and
leaves `param_values` empty. The same wall can therefore arrive as `Wall` (genrule), `Component` (catalog), or a
mapped IFC type (import), with different undo/save behavior. See [WRITE_PATHS.md](WRITE_PATHS.md) and
[CODE_DEVIATIONS.md §6](../status/CODE_DEVIATIONS.md#6-write-path-persistence--geometry-path-deviations--known-2026-10-27).

---

## 5. (e) Save / load of `.omnim` JSON files

```mermaid
flowchart LR
    subgraph Start
        A[CLI: cargo run / release binary<br/>main.rs] -->|read OMNIM_PROJECT<br/>default project.omnim| B{File exists?}
        B -->|yes| C[Project::load → deserialize JSON]
        B -->|no| D[New empty Project + seed materials]
    end
    C --> E[SharedProject: Arc RwLock Project]
    D --> E
    E --> F[omnim-server handles all /api requests]
    F -->|save_path = None for default project.omnim| X[nothing persisted]
    F -->|only 5 handlers, only when save_path is Some| G[.omnim JSON file rewritten]
```

Mechanics:

- **One file per process.** `omnim-server::ServerConfig { project, save_path, ... }`. `main.rs` sets `save_path`
  to `None` when the project path is exactly `project.omnim`; only a non-default `OMNIM_PROJECT` yields `Some`.
- **Partial saves even when enabled.** Only `update_object`, `add_feature`, `update_feature`, `remove_feature`,
  and `rebuild_component` call `Project::save(path)`. Creating objects, `/api/genrule`, `/api/omcom`, catalog
  instantiation, IFC import, and series edits are never saved. See [WRITE_PATHS.md](WRITE_PATHS.md).
- **Full-rewrite, not incremental.** Each save serializes the **entire `Project`** to JSON and rewrites the file.
  There is no journal, no WAL, no checkpointing.
- **Crash window:** a kill between mutation and successful write can leave the file stale or (theoretically) truncated;
  there is no atomic-rename pattern yet.
- **No history.** One file = one snapshot. Undo/redo lives only in in-memory `Executor` state.
- **Size risk:** for very large projects (big `seed_catalog` expansions), full serialization of a monolithic file is
  inherently O(project size) per mutation. This is the concrete motivation behind the planned database-backed
  persistence (see [PLATFORM_ARCH.md §9](PLATFORM_ARCH.md#9-future-directions-not-yet-implemented)).

---

## 6. Non-flows (things that do **not** exist)

For clarity, the following paths that people sometimes assume are **not implemented**:

- **No WebSocket / SSE / push.** The browser polls with React Query re-fetches.
- **No streaming LLM output.** AMIR's LLM responses are consumed whole, then command-validated.
- **No server-side rendering of previews** — neither rasterized images nor server-gl timers.
- **No multi-file-project model**, no per-object file persistence. There is one `.omnim`.
- **No worktree/branching** in version-model sense (repository `git` branches are unrelated to the product).
