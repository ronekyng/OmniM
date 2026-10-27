# Testing Guide

[English](TESTING.md) | [简体中文](TESTING.zh-CN.md)

How to run and write tests in OmniM: Rust unit + integration tests, the server API integration suite, and web unit tests.

## Overview

| Layer | Framework | Command | Where tests live |
| --- | --- | --- | --- |
| Rust (any crate) | `cargo test` | `cargo test -p <crate>` | `#[cfg(test)] mod tests` inside `crates/*/src` |
| Rust (workspace) | `cargo test` | `cargo test` | all crates |
| Server REST API | `cargo test` | `cargo test -p omnim-server` | `crates/server/tests/api_integration.rs` |
| IFC | `cargo test` | `cargo test -p omnim-ifc` | `crates/ifc/tests/ifc_roundtrip.rs` |
| Web | Vitest | `npm run test` (from `web/`) | `web/src/__tests__/*.test.ts` |

`scripts/test-all.sh` runs the workspace `cargo test` and then a **web build** (not web tests) if Node is available — see the caveat below.

## Rust Tests

### Per-crate unit tests

Most crates keep unit tests in the same files they test:

```bash
cargo test -p omnim-omcom        # DSL parsing/execution tests in crates/omcom/src/lib.rs
cargo test -p omnim-model        # model/store/catalog/material tests
cargo test -p omnim-sketch       # constraint solver tests (slvs-backed)
cargo test -p omnim-amir         # prompt parsing etc.; provider network tests skip without AMIR_API_KEY
```

(workspace member names; `omnim-amir`'s real-provider tests check `AMIR_API_KEY` and otherwise print "Skipping real provider test" — keyless environments only exercise the mock provider.)

### Whole workspace

```bash
cargo test
```

This includes the integration suites below. First run compiles everything; later runs are incremental.

### Server API integration tests

`crates/server/tests/api_integration.rs` (~1,400 lines) is the big one. It spins up the server's axum router over a shared in-memory project via `axum` + `tower` (no network), and exercises real REST flows end-to-end: object CRUD, tree/stats, properties, transforms, materials, measurements, lifecycle, administrative hierarchy, breakdowns, omcom execution, undo/redo, sketches/solve/extrude, catalog and series management, AMIR translate/chat with the mock provider, and IFC import/export. IFC fixtures are read from repo-root `.ifc` files.

```bash
cargo test -p omnim-server --test api_integration
cargo test -p omnim-server --test api_integration amir   # filter by name
```

### IFC round-trip test

```bash
cargo test -p omnim-ifc --test ifc_roundtrip
```

Round-trips geometry through the IFC writer and re-imports it, comparing result fidelity.

### `.ifc` fixtures

Sample IFC files live in **`fixtures/ifc/`** and are used manually via the `/api/import/ifc` endpoint:

- `simple.ifc` — minimal piece
- `part.ifc`, `part40.ifc` — mechanical part samples
- `test.ifc` — general smoke test file
- `test_building.ifc` — a full-storey building sample good for the object tree/hierarchy

They live at the root (not under `crates/ifc/tests/`) because the server integration tests read them relative to the project root; keep paths in mind if you add new fixtures.

## Web Tests

Vitest, run from the `web/` directory:

```bash
cd web
npm run test          # = `vitest run`, non-watch
npm run test:watch    # or: npx vitest
```

Existing tests live in `web/src/__tests__/` (`stateMachine.test.ts` covers the creation-tool / interaction state machine used by the viewer). Vitest matches files like `__tests__/*.test.ts` by default.

Note: **`scripts/test-all.sh` does not run web unit tests** — it only runs `cargo test` and then `npm install && npm run build` for the web app. To cover everything, run it *and* `cd web && npm run test`.

## Known Flaky Tests

**`omnim-model` → `test_find_and_rebuild_catalog_instances`** (crates/model/src/project.rs) has failed once under a full parallel `cargo test --workspace` run (assertion `should find 2 wall instances`, got 1) and could not be reproduced afterwards (stress: 15× per-crate runs + 6× workspace runs, all green). Recorded here so the observation is not lost.

- **Root cause (assessed):** tests share the session-level global `static LIBRARY: RwLock<Option<Library>>` (initialized `call_once` by `setup_library()`). Cargo runs tests within a binary on multiple threads, so tests that read the global library (e.g. `create_object_from_default_template` → `instantiate_catalog_component_impl`) race with tests that write or observe it — `Once` guarantees "initialize once", not per-test isolation.
- **Debug recipe:** re-run the binary with `cargo test -p omnim-model --lib -- --test-threads=1`; if isolated passes and parallel intermittently fails, it is the shared-global race above.
- **Candidate fixes (pick one when doing the test-isolation refactor):** per-test `Library` injection (pass the library into `Project` construction instead of reading the static), or serialize global-dependent tests, or a `OnceLock` snapshot per test process.
- Server integration tests are NOT affected (separate process + port isolation per test run).

**Anti-flake practices for new tests:** don't write to global statics from tests; construct state locally like the store/`Project` tests already do.

## Adding Tests

**Rust unit tests** — add `#[cfg(test)] mod tests` inline in the module you're touching, mirroring the style already there (small setup + assert, no external fixtures).

**Server API tests** — extend `crates/server/tests/api_integration.rs`. Build a JSON body, hit the handler via the shared router's `oneshot` client pattern already used across the file, and assert on status + decoded serde JSON. Prefer one focused test per endpoint/edge case; mock-provider AMIR tests need no keys by design. Use the existing `.ifc` fixtures rather than creating oversized new ones — and if you do add fixtures, name them clearly (`*_test.ifc`) and place them in `fixtures/ifc/`.

**Sketch/model stateful tests** — construct a `Project` directly (see existing model/store tests) instead of going through the network stack.

**Web tests** — colocate under `web/src/__tests__/`, name `<feature>.test.ts`, and test store/state-machine logic rather than three.js rendering itself (rendering is hard to assert on headlessly). Keep them pure-logic: zustand stores and reducer-style functions test well with Vitest.

## Quick Sanity Run

```bash
cargo test                                   # all Rust
cd web && npm run test                       # all web
scripts/test-all.sh                          # Rust + web build (not web tests)
```
