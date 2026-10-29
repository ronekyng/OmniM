---
title: "Getting Started"
status: current
last_updated: 2026-10-29
applies_to: "repo"
evidence: "commands and paths verified in the repo 2026-10-27"
---
# Getting Started

[English](GETTING_STARTED.md) | [简体中文](GETTING_STARTED.zh-CN.md)

This guide gets you from a fresh clone to an interactive session with OmniM in roughly five minutes, and then shows you where to poke around next.

## Prerequisites

| Tool | Requirement | Notes |
| --- | --- | --- |
| Rust toolchain | Any recent stable (`rustc` 1.8x+ works; the workspace uses **edition 2021**) | Install via [rustup](https://rustup.rs). Run `rustc --version` to check. |
| Node.js | **≥ 20** recommended; CI machines here run Node 24 | `scripts/dev-web.sh` prefers `nvm use 24` if nvm is available, but any recent LTS is fine. There is no `engines` field pinned in `web/package.json`. |
| npm | Ships with Node | Only needed for the web UI. |
| Optional: `protoc` | Not required as a separate install | `crates/core/build.rs` uses `prost-build`, which bundles its own protobuf codegen. |

No database, no external services. State lives in memory and can be written to a plain JSON file — but **saving is off by default** (see the warning below).

> ⚠️ **Persistence warning — the default launch does NOT save mutations.**
> ⚠️ **Persistence warning — updated 2026-10-28.**
> Persistence now defaults **on**: the server writes `$OMNIM_PROJECT` (or `project.omnim` in the server's working directory) after every executor-path mutation, and anywhere in between you can force a save with `POST /api/project/save`.
> Escape hatch: `OMNIM_EPHEMERAL=1` (or `true`/`yes`) runs the server fully in memory.
> Coverage: every mutating endpoint now saves and is undoable (objects, properties, materials, sketches, components, catalog definitions, series, lifecycle, measurements, administrative/breakdown, catalog instantiation, IFC import). The two documented exceptions are `.omnimlib` import into the global library and the `section_handler` cache lock. See [WRITE_PATHS.md](../architecture/WRITE_PATHS.md).

## 5-Minute Quickstart

### 1. Build and run the server

The server binary is **`omnim`** (crate `omnim-server`, in `crates/server`). It starts an Axum HTTP server on `0.0.0.0:3000`.

```bash
cargo run -p omnim-server --bin omnim
```

On startup it:

- loads the library from `~/.omnim/library.omnimlib` if present (falls back to the built-in seeded catalog),
- loads the project from `project.omnim` in the server's working directory (the crate dir under `cargo run`), or from `$OMNIM_PROJECT`, creating a new one named "OmniM Project" if the file doesn't exist (mutations now persist by default — see the warning above),
- serves REST APIs under `/api/*`.

> Note: the literal `serve` argument sometimes typed after the binary name (`omnim serve`) is currently ignored — running plain `omnim` does the same thing.

### 2. Run the web UI (second terminal)

```bash
cd web
npm install
npm run dev
```

The Vite dev server listens on **http://localhost:5173** and proxies `/api/*` to `http://localhost:3000`, so the UI talks to the server you started in step 1 with no CORS setup.

Open <http://localhost:5173> in a browser. (There is also `scripts/dev-server.sh` and `scripts/dev-web.sh` that wrap these two steps — see [BUILDING.md](BUILDING.md) for what they do.)

### 3. First interactions

Once the app loads, try these in any order:

- **Create an object via the UI.** Use the main toolbar / create menu to add a component or an object. It appears in the object tree (left panel) and, if it has geometry, in the 3D viewer. Click a tree item to see its properties in the properties panel.
- **Try the Omcom console.** Open the command/console panel at the bottom. The console accepts:
  - a **JSON `OmniCommand`** envelope, or
  - the shortcut **`CREATE_OBJECT <type> <name>`**, e.g.:

    ```text
    CREATE_OBJECT WALL Wall-01
    ```

  - the shortcuts `UNDO` and `REDO`.

  A free-form text DSL (`CREATE Object …`) is **not** implemented — it remains a draft ([RFC-0001](../RFC/README.md)). The Omcom console is a thin client over the typed command executor, not a separate modeling language.

  You can also hit the HTTP endpoints directly. To execute a command, post a full `OmniCommand` envelope to `/api/omcom`:

  ```bash
  curl -X POST localhost:3000/api/omcom -H "Content-Type: application/json" \
    -d '{"version":"1.0","id":"demo-1","action":1,"operations":[{"create_object":{"type":"WALL","name":"Wall-01","description":"","geometry":null,"properties":[],"parent_id":""}}]}'
  ```

  Undo has its own endpoint — `/api/omcom` does **not** accept `{"command":"UNDO"}`:

  ```bash
  curl -X POST localhost:3000/api/undo
  ```

- **Try the AI chat (keyless mock provider).** AMIR is the AI-instruction layer (`crates/amir`). Without any configuration, it runs with a **mock provider** that needs no API key — open the AI chat panel (`web/src/app/components/AiChat.tsx`) and type something like *"create a wall named W1"*. Real LLM usage (OpenAI / Anthropic) requires env keys — see [BUILDING.md](BUILDING.md).
- **Try path-based creation (Path).** Pick a tool, then: an **L-shaped wall** — click 3 corners, Enter → layered path with mitered corners; a **slab** — click 3+ polygon corners and return to the first point (or double-click) to close → layered footprint; a **feature line** — click N points → polyline guide tube. Backspace removes the last picked point, Escape cancels. The echo hint above the viewport tracks your mode and point count.
- **Import an IFC file.** A few `.ifc` fixtures live under `fixtures/ifc/`: `simple.ifc`, `part.ifc`, `part40.ifc`, `test.ifc`, `test_building.ifc`. Use the UI's import action, or (from the repository root):

  ```bash
  curl -X POST localhost:3000/api/import/ifc \
    --data-binary @fixtures/ifc/simple.ifc \
    -H 'Content-Type: text/plain'
  ```

  The endpoint takes **raw IFC text**, not a multipart upload, so `curl -F` does not work. Then check `/api/tree` or the object tree panel to see what came in. Import writes objects directly into the in-memory project and does not save the file (see the persistence warning above).

## Where to Explore Next

- [BUILDING.md](BUILDING.md) — workspace layout, environment variables, troubleshooting.
- [ARCHITECTURE_MAP.md](ARCHITECTURE_MAP.md) — what each crate does, dependency graph, and "where do I add X?" recipes.
- [TESTING.md](TESTING.md) — Rust unit tests, server API integration tests, web Vitest, and `scripts/test-all.sh`.
- `docs/architecture/PLATFORM_ARCH.md` — the long-form platform architecture document.
- `README.md` — project vision, the three core layers (Component Catalog, Series Library, Omcom), and roadmap.
- **Frontend productization** — the UI is a working prototype, not yet a commercial shell. Read [FRONTEND_MATURITY](../status/FRONTEND_MATURITY.md) for the honest score, [PRD_FRONTEND](../product/PRD_FRONTEND.md) for the 96 requirements, and [KEYBOARD_MAP](../reference/KEYBOARD_MAP.md) for what works today. Today the app opens the workspace directly and loads `$OMNIM_PROJECT`; a project home is contractual but not implemented.

## Stopping Things

- Ctrl-C in each terminal, or:

```bash
scripts/stop-server.sh    # frees port 3000 (uses fuser/lsof)
```
