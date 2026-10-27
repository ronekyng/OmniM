# Building OmniM

[English](BUILDING.md) | [简体中文](BUILDING.zh-CN.md)

How the workspace is organized, how to build and run it, and what to do when things go wrong.

## Workspace Layout

A single Cargo workspace (`Cargo.toml`, resolver 2) with eight crates under `crates/`, plus a TypeScript web app under `web/` and protobuf definitions in `proto/`. See [ARCHITECTURE_MAP.md](ARCHITECTURE_MAP.md) for crate responsibilities.

```
OmniM/
├── Cargo.toml              # workspace manifest (edition 2021)
├── proto/                  # omnim.proto + omcom.proto (prost, serde derived types)
├── crates/
│   ├── core/               # omnim-core — shared enums/domain primitives + proto codegen (build.rs)
│   ├── model/              # omnim-model — unified data model, catalog, library, persistence
│   ├── sketch/             # omnim-sketch — sketch entities + constraint solver
│   ├── omcom/              # the Omcom DSL interpreter (does the actual model mutation)
│   ├── server/             # omnim-server — Axum REST API; binary name `omnim`
│   ├── ifc/                # omnim-ifc — IFC import/export
│   └── amir/               # AMIR AI instruction layer (providers: mock/openai/anthropic)
├── web/                    # React 19 + Vite + three.js frontend
├── scripts/                # dev/test helper scripts (see below)
└── *.ifc                   # IFC test fixtures
```

## Build Commands

```bash
cargo build                          # debug build of everything
cargo build --release                # optimized build
cargo build -p omnim-server          # server crate only (binary omnim in target/debug)
cargo check                          # fast type-check without codegen/linking

cd web && npm install && npm run build   # production web build → web/dist/
```

First build is the slow one (wgpu, truck-* kernels, reqwest with rustls). Subsequent `cargo check` runs are quick.

The web dev server (`npm run dev`) serves at :5173 and proxies `/api` to `http://localhost:3000` — see `web/vite.config.ts`. Config for production (`npm run build`) uses a relative `./` base.

## Run & Debug

### Server

```bash
cargo run -p omnim-server --bin omnim
```

Environment variables (read in `crates/server/src/main.rs` and `crates/amir/src/lib.rs`):

| Variable | Default | Purpose |
| --- | --- | --- |
| `OMNIM_PROJECT` | `project.omnim` | Project file to load (created if missing). Non-default paths are saved back to on mutation. On `cargo run`, the CWD is the **crate dir** (`crates/server/`), so use `../project.omnim` or an absolute path to point at files in the repo root. |
| `OMNIM_HOST` | `0.0.0.0` | Bind host. |
| `OMNIM_PORT` | `3000` | Bind port. |
| `AMIR_PROVIDER` | `mock` | LLM provider: `mock` (no key needed), `openai`, or `anthropic`. |
| `AMIR_API_KEY` | — | API key for the chosen provider. |
| `AMIR_MODEL` | provider default | Model name override. |
| `AMIR_BASE_URL` | provider default | Custom/proxy base URL (e.g. self-hosted OpenAI-compatible API). |
| `AMIR_TEMPERATURE` | `0.1` | Sampling temperature. |
| `AMIR_MAX_TOKENS` | `1024` | Max tokens per reply. |

Provider details (exact env fallbacks) live in `crates/amir/src/llm/openai.rs` and `anthropic.rs`: if no `AMIR_API_KEY` is set, the OpenAI provider falls back to `OPENAI_API_KEY` and Anthropic to `ANTHROPIC_API_KEY`. Without a usable key, calls fail — stick with `AMIR_PROVIDER=mock` for keyless development.

Data files:

- **Library**: `~/.omnim/library.omnimlib` if present, otherwise the built-in seeded catalog (`crates/model/src/seed_catalog.rs`). If that file exists but fails to parse, the server prints a warning and falls back to seeded defaults.
- **Project**: `$OMNIM_PROJECT`, created fresh if unreadable.

Debug notes: the literal `serve` argument typed after the binary is ignored. Logging is plain `eprintln!` in places; a cold start regenerates proto types via `crates/core/build.rs` (`OUT_DIR`).

### Web

```bash
cd web
npm run dev        # :5173, proxies /api → :3000
npm run build      # dist/
npm run preview    # serve the built bundle
npm run test       # Vitest, one-shot; `npm run test:watch` for watch mode
```

Sample env wrapper:

```bash
OMNIM_PORT=3100 cargo run -p omnim-server --bin omnim &
cd web && npm run dev   # change the proxy target in web/vite.config.ts to :3100
```

## Scripts

Scripts live in `scripts/` and all resolve paths relative to the repo root (`cd "$(dirname "$0")/.."`).

| Script | What it does | Caveats |
| --- | --- | --- |
| `dev-server.sh` | Kills anything on `:3000` (fuser/lsof), `cargo build`, then starts `./target/debug/omnim` with `OMNIM_PROJECT`. | Prints both `localhost` and a **WSL2 eth0 IP** line when it detects one — the WSL URL is only meaningful under WSL2. Port is `OMNIM_PORT`-aware. |
| `dev-web.sh` | Sources nvm (`use 24`), `npm install` if `node_modules` missing, then `npm run dev`. | Exits with instructions if `node` is unavailable on Linux. Serves at :5173. |
| `stop-server.sh` | Kills the process on `OMNIM_PORT` (default 3000) and reports port status via `ss`. | None; safe to repeat. |
| `test-all.sh` | Runs `cargo test` for the whole workspace, then (if node exists) `npm install && npm run build` in `web/`. | **Does not run web unit tests** — just the Vite build. Run `npm run test` in `web/` separately; see [TESTING.md](TESTING.md). |

## Troubleshooting

- **Port 3000 already in use.** Either run `scripts/stop-server.sh`, or pick another port: `OMNIM_PORT=3100 cargo run -p omnim-server --bin omnim` (then point the Vite proxy at that port). Old server instances of this very project are the usual culprit — `dev-server.sh` kills them for you.
- **AI chat doesn't call a real LLM.** With no env configured, `AmirConfig::from_env()` defaults `AMIR_PROVIDER` to `mock`, which produces canned responses. To use a real provider, export `AMIR_PROVIDER=openai|anthropic` plus `AMIR_API_KEY` (or `OPENAI_API_KEY` / `ANTHROPIC_API_KEY`). This mock fallback is by design, so keyless demos work offline.
- **Project data didn't persist / reappear.** Remember `OMNIM_PROJECT` defaults to a relative `project.omnim`, resolved against the server's CWD (the crate dir under `cargo run`). Set an absolute path if you wonder where the file went. `fixtures/my-project.omnim` is a sample you can point at.
- **Bad library file at `~/.omnim/library.omnimlib`.** A parse failure logs a warning and silently uses the seeded catalog; delete or fix the file to stop the warning.
- **WSL2 networking.** `dev-server.sh` prints a WSL2 eth0 IP; from Windows use that URL instead of localhost if localhost forwarding doesn't work.
- **Stale `scripts/update_roadmap.py`.** It's a one-off, left in place; not part of any build/test flow. Ignore it.
- **web build fails on Node.** `dev-web.sh` and the README assume nvm; `nvm install --lts` then retry.
