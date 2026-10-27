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

No database, no external services. The server persists to a plain JSON file.

## 5-Minute Quickstart

### 1. Build and run the server

The server binary is **`omnim`** (crate `omnim-server`, in `crates/server`). It starts an Axum HTTP server on `0.0.0.0:3000`.

```bash
cargo run -p omnim-server --bin omnim
```

On startup it:

- loads the library from `~/.omnim/library.omnimlib` if present (falls back to the built-in seeded catalog),
- loads the project from `./project.omnim` (or `$OMNIM_PROJECT`), creating a new one named "OmniM Project" if the file doesn't exist,
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
- **Try the Omcom console.** Open the command/console panel at the bottom and type a simple command, e.g.:

  ```text
  CREATE Object name:"Wall-01"
  ```

  then try `UNDO` to roll it back. Omcom is the unified modeling DSL — everything the UI or the AI does ultimately becomes an Omcom instruction. You can also hit the HTTP endpoint directly:

  ```bash
  curl -X POST localhost:3000/api/omcom -H "Content-Type: application/json" \
    -d '{"command": "UNDO"}'
  ```

- **Try the AI chat (keyless mock provider).** AMIR is the AI-instruction layer (`crates/amir`). Without any configuration, it runs with a **mock provider** that needs no API key — open the AI chat panel (`web/src/app/components/AiChat.tsx`) and type something like *"create a wall named W1"*. Real LLM usage (OpenAI / Anthropic) requires env keys — see [BUILDING.md](BUILDING.md).
- **Import an IFC file.** A few `.ifc` fixtures live under `fixtures/ifc/`: `simple.ifc`, `part.ifc`, `part40.ifc`, `test.ifc`, `test_building.ifc`. Use the UI's import action, or:

  ```bash
  curl -X POST localhost:3000/api/import/ifc -F "file=@../../fixtures/ifc/simple.ifc"
  ```

  Then check `/api/tree` or the object tree panel to see what came in.

## Where to Explore Next

- [BUILDING.md](BUILDING.md) — workspace layout, environment variables, troubleshooting.
- [ARCHITECTURE_MAP.md](ARCHITECTURE_MAP.md) — what each crate does, dependency graph, and "where do I add X?" recipes.
- [TESTING.md](TESTING.md) — Rust unit tests, server API integration tests, web Vitest, and `scripts/test-all.sh`.
- `docs/architecture/PLATFORM_ARCH.md` — the long-form platform architecture document.
- `README.md` — project vision, the three core layers (Component Catalog, Series Library, Omcom), and roadmap.

## Stopping Things

- Ctrl-C in each terminal, or:

```bash
scripts/stop-server.sh    # frees port 3000 (uses fuser/lsof)
```
