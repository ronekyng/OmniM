---
title: "OmniM Development Roadmap"
status: current
last_updated: 2026-11-02
applies_to: "repo"
evidence: "Reconciled against code, CHANGELOG and the gate board; every status carries an evidence link"
---
# OmniM Development Roadmap

> This document answers three questions: **what exists today, what comes next, and what is blocking**.
>
> - **Dates are not commitments.** This roadmap is ordered by dependencies and gates, not by calendar; historical delivery dates live only in [CHANGELOG](CHANGELOG.md).
> - **Single source of truth.** Capability status comes from `status/` and `architecture/`; design intent comes from `specification/` and `RFC/`. Any mismatch must be recorded explicitly in [CODE_DEVIATIONS](CODE_DEVIATIONS.md).
> - **Numbers are labels, not order.** `7b`, `8` and `P8-S1` are stable labels other documents can reference. They do not require serial execution and do not encode calendar order.

## 1. How to Read This Document

### 1.1 Status Legend

| Mark | Meaning |
|---|---|
| ✅ Done | Implemented, with code or test evidence |
| 🔄 In progress | Active work, some slices delivered |
| 🔀 Partial | Substantially delivered, with named gaps |
| 📅 Planned | Not started, or intent only |

### 1.2 Three Easily Confused Terms

| Term | What it is | What it is not |
|---|---|---|
| Batch / phase | A delivery-stream label (1–12, 7b) | A calendar quarter, or a mandatory serial number |
| Gate | G0–G3, release-quality gates | A feature batch |
| Slice | One acceptance-sized delivery inside a batch (e.g. P8-S1) | A standalone phase |

## 2. Current State Snapshot

| Workstream | Batch | Status | Usable today | Named gaps | Evidence |
|---|---|---|---|---|---|
| Kernel & geometry | 1–6 | ✅ main body (batch 1 🔀) | Unified data model, typed commands with undo/redo, sketch + slvs, features + truck-BRep, catalog/series, GenRule and default templates | The nom-based Omcom text DSL ([RFC-0001](../RFC/README.md)) | `crates/model/`, `crates/omcom/`, [CAPABILITY_MATRIX](../CAPABILITY_MATRIX.md) |
| Workspace prototype | 7 | ✅ | Viewport, sketch editor, feature tree, catalog panel, properties panel, Omcom console, AI chat, Library/Series Manager | A functional prototype only, not a product-grade UI; AI command preview belongs to batch 9 | `web/src/`, [CHANGELOG](CHANGELOG.md) |
| Frontend productization | 7b | 🔄 | Project home, docked shell, save/save-as, error boundaries, shared dialogs, auto-naming | G1 not passed; G2/G3 not started; no e2e, visual or performance baseline | [RELEASE_READINESS](RELEASE_READINESS.md) |
| Domain extensions | 8 | 🔄 | Production catalog data; storey elevation/grid/storey view/openings; assembly container; pipe routing and connectors | Full building IFC/MEP/quantities, civil terrain/alignment, plant branch piping, mechanical BOM; domain frontend | [CAPABILITY_MATRIX](../CAPABILITY_MATRIX.md) |
| AI compilation | 9 | 🔀 | AMIR providers, natural language→command, multi-turn chat, reverse model description | Multi-step planning, validation explanation, confidence, human-in-the-loop preview, multi-agent, RAG | `crates/amir/` |
| Format interoperability | 11 | 🔀 | IFC 2x3 / IFC 4 subset import/export (hand-written parser/writer) | Incomplete entity coverage; STEP/DXF/DWG/GIS/glTF/OBJ/FBX unimplemented | [FORMAT_COMPAT](FORMAT_COMPAT.md) |
| Cloud & collaboration | 10 | 🔀 (subset only) | Axum REST, JSON-file persistence, undo/redo on every mutating endpoint | PostgreSQL/PostGIS, version history, WebSocket, multi-user collaboration | [API](../reference/API.md), [WRITE_PATHS](../architecture/WRITE_PATHS.md) |
| Analysis & ecosystem | 12 | 📅 | None | Structural analysis, clash detection, quantities, cost, plugin ecosystem | — |

## 3. Delivered Batches (1–7)

### 3.1 Kernel & Geometry (1–6)

- **Batch 1 🔀**: unified data model, typed `OmniCommand` plus executor, lifecycle/administrative/breakdown hierarchies and the relationship system. One gap only: the text DSL is still a draft RFC.
- **Batch 2 ✅**: sketch data model, 11 constraint types, slvs solver, sketch editor, Extrude/Revolve pipeline.
- **Batch 3 ✅**: feature engine (9 feature types), truck-BRep, parameter system, component editor.
- **Batch 4 ✅**: ComponentDef/ParamDef/ConnectorDef, Series selectors, `.omnimlib` / `.omnimseries`.
- **Batch 5 ✅**: steel sections, piping and concrete building component catalog plus the instantiation pipeline.
- **Batch 6 ✅**: GenRule refactor, Library/SeriesStore/Project split, series gate, CatalogRef, default templates.

Evidence entry points: `crates/model/`, `crates/sketch/`, `crates/omcom/`, [ARCHITECTURE_MAP](../guides/ARCHITECTURE_MAP.md).

### 3.2 Workspace Prototype (7) ✅

Project Workspace, sketch editor, feature tree, catalog panel, properties panel, Omcom console, AI chat, and the Library/Series Manager are all usable.

**Boundary note:** batch 7's ✅ means "the main flows work in a prototype", not "product-grade UI". Productization is batch 7b and cannot substitute for it.

## 4. Main Line A: Frontend Productization (7b) 🔄

### 4.1 Exit Definition

| Exit | Contains | Criterion |
|---|---|---|
| Engineering exit | G1 + G2 | Frontend engineering is closable and every requirement has evidence |
| Release exit | G3 | Compatibility, security, telemetry and external review |

> Current position: **7b's engineering finish line is G2**; G3 belongs to pre-release. G3 includes an external security review and ACR/VPAT, which are not pure code work. This position needs product sign-off (see Q2 and Q4 in §10).

### 4.2 Gate Board

| Gate | Scope | Status | Today | Exit condition |
|---|---|---|---|---|
| G0 | Contracts and requirements | 🔀 | Draft complete, signature block pending | Product/design/frontend signatures |
| G1 | Alpha | 📅 Not passed | 22 ✅ / 3 🔀 / 0 📅 of 25 | All 25 ✅ with clickable evidence |
| G2 | Beta | 📅 Not started | 0 / 37 | Design system, i18n, A11Y, performance all met |
| G3 | GA | 📅 Not started | 0 / 34 | Compatibility/security/telemetry/ACR/release handbook + external review |

Full list: [REQUIREMENTS_REGISTER](../product/REQUIREMENTS_REGISTER.md); judgement rules: [RELEASE_READINESS](RELEASE_READINESS.md).

### 4.3 Execution Order (Dependency-Ordered, Not Calendar-Ordered)

| Step | Content | Completion criterion |
|---|---|---|
| 0 Governance | G0 signatures; close O4/O6; confirm the §4.1 exit position; issue the freeze order | Decisions recorded, freeze scope listed |
| 1 Test infrastructure | Playwright, screenshot baselines, axe, performance reference machine, CI job | CI runs on PRs; baselines are comparable |
| 2 G1 close-out | Fill missing features first (theme provider, skeleton screens), then shell/panels/data integrity, then evidence item by item | 25 / 25 |
| 3 G2 close-out | Semantic tokens, i18n, accessibility, performance budgets, remaining feedback and data-integrity items | 37 / 37 |
| 4 G3 preparation | Compatibility matrix, security disclosure, telemetry dictionary, ACR/VPAT draft, release and support handbook | 34 / 34 + external review outcome |

### 4.4 Waves & Requirement Domains

| Wave | Scope | Requirement domains | Target gate |
|---|---|---|---|
| A | Shell, docking, status bar, dual themes, icon and component system | SHL, DS | G1 → G2 |
| B | Project lifecycle, dirty state, error boundaries, dialogs, auto-naming | DAT, FBK, INT | G1 |
| C | Command registry, feedback states, i18n, accessibility, undo history | INT, I18N, A11Y | G2 |
| D | On-demand rendering, instancing, virtualization, e2e, visual regression, CI gates | PRF, CMP, SEC, TLM | G2 → G3 |

## 5. Main Line B: Domain Extensions (8) 🔄

### 5.1 Delivered Slices

- Production catalog data: steel sections (EN 10365 / GB/T 706), pipe schedules (ASME B36.10), concrete templates, real material standards, BRep correctness checks.
- Building datum: storey `elevation`, grid panel, `bottom_level_id`/`top_level_id` single-step cascade, storey view and section linkage.
- Standalone opening object (`IfcOpeningElement`).
- Assembly container (`POST /api/assemblies`; container plus Component/Part children, whole-tree delete in one undo step).
- Pipe routing: N-point polyline sweep, elbow placeholders, instance connectors with bore/type compatibility checks.

Evidence: [CAPABILITY_MATRIX](../CAPABILITY_MATRIX.md), [CHANGELOG](CHANGELOG.md).

### 5.2 Not Delivered

| Item | Status | Prerequisite | Note |
|---|---|---|---|
| 8-1 Building: full IFC element coverage, MEP, quantities | 📅 | Kernel ✅ | Doors/windows are host fills (leaf + frame); frame types, sill/head constraints and schedules are still missing; the host is not re-cut |
| 8-2 Civil: terrain fitting, earthwork, alignment/profile | 📅 | Kernel ✅ | Bridges/tunnels still use start/end only |
| 8-3 Plant: branch piping, P&ID | 📅 | Pipe routing ✅ | Branches, fittings and feature-tree templates not covered |
| 8-4 Mechanical: BOM, quantities, kinematic joints | 🔀 | Assembly container ✅ | Container delivered; BOM/joints not started |
| 8-5 Frontend: domain panels, filters, domain-aware drag-to-place | 📅 | **7b G2** | Must wait for the shell and design system to settle |

### 5.3 Dependency on 7b

[ADR-0008](../architecture/DECISIONS.md) states explicitly that "the shell rewrite must precede panel polish"; [ADR-0009](../architecture/DECISIONS.md)'s panel registry exists precisely as the extension point for domain panels. Therefore:

- **8-5 does not start before 7b G2.**
- 8-5 must use `PanelRegistration`, semantic tokens, `t(key)` and the unified command registry; otherwise it is booked as new debt.
- 8-1 to 8-4 are backend/geometry work; whether they may run alongside 7b is covered in §7.2.

## 6. Other Workstreams (9–12)

- **9 AI compilation 🔀**: engine, providers, translation and chat delivered; multi-step planning, validation explanation, confidence, human-in-the-loop and RAG missing.
- **11 Format interoperability 🔀**: IFC subset only; other formats unimplemented, see the matrix in [FORMAT_COMPAT](FORMAT_COMPAT.md).
- **10 Cloud & collaboration 🔀 (subset only)**: REST + JSON files delivered; database, version history, WebSocket and collaboration not implemented.
- **12 Analysis & ecosystem 📅**: structural analysis, clash, quantities, cost and plugins not started.

## 7. Dependencies, Parallelism, and Freeze Rules

### 7.1 Dependencies

- Kernel (1–6) → 9, 11, 8-1…8-4.
- 7b (shell, design system, command registry, i18n) → 8-5 domain frontend.
- 7b → the collaboration UI of 10.
- 8-5 may not precede 7b G2; the other domain backend items have no code dependency on 7b.

### 7.2 Parallelism Rules

1. **One writing stream per file domain at a time.** `web/src/` belongs to 7b; `crates/` belongs to the backend stream.
2. Cross-stream changes land the contract first, then the code.
3. New UI must use the panel registry, semantic tokens, `t(key)` and the unified command registry; bypassing them is booked as new debt.
4. Whether 8-1 to 8-4 backend work may proceed during the 7b freeze is Q7 in §10; the default is frozen.

### 7.3 Current Freeze & Exceptions

| Frozen | Exception |
|---|---|
| New features in 8-1 to 8-4 | Regression fixes for already-delivered capabilities |
| 8-5 domain frontend | — |
| New object types, new REST routes | Security or data-loss fixes |

**Closed regression fixes (not 8-1 feature work):**

- Door geometry: doors/windows are now leaf/frame panels placed from the wall's `path_points`; no more world-AABB thickness and no more floating at the origin. See [CODE_DEVIATIONS §6.5](CODE_DEVIATIONS.md).
- Storey creation: `storey` no longer carries a box geometry and is a datum-only container; the server likewise discards container geometry.

Both ship with tests (`crates/server/tests/opening.rs`, `datum.rs`).

## 8. Current Non-Goals

1. Photorealistic rendering.
2. Offline-first architecture.
3. Mobile AR/VR and mobile apps.
4. Full CAM/CNC.
5. Desktop packaging (Tauri).
6. A WASM-shared core in the production web build.

The text DSL is not a non-goal; it is batch 1's undelivered item, waiting on [RFC-0001](../RFC/README.md).

## 9. Technical-Debt Ledger

| Item | Owner | Evidence |
|---|---|---|
| Hard-coded colours bypass semantic tokens | G2 | [CODE_DEVIATIONS](CODE_DEVIATIONS.md) 7.8 |
| UI strings not externalized (31 files) | G2 | CODE_DEVIATIONS 7.9 |
| Accessibility semantics missing | G2 | CODE_DEVIATIONS 7.10 |
| `App.tsx` whole-store subscription, unsplit `ThreeViewer`, react-query unused | G2 | [CODE_AUDIT](CODE_AUDIT_2026-10-30.zh-CN.md) P2/P3 |
| Error codes reserved but never emitted | G2 | [ERROR_CODES](../reference/ERROR_CODES.md) |
| Performance budgets not enforced in CI (TLM 0%) | G2 | [MODELER_UIUX_REAUDIT](MODELER_UIUX_REAUDIT_2026-10-30.zh-CN.md) §5 (zh-CN) |
| No e2e / visual / performance baseline | G1 prerequisite | §4.3 of this document |
| Server token middleware, `TimeoutLayer`, persist off the write lock, full-list/full-clone reads | Pre-release or backend-parallel | CODE_AUDIT P1/P2 |
| omcom single-file split, executor `Result` conversion, `project.rs` split, shared `IndexedStore` | Long term | CODE_AUDIT §5 |
| Real triangle-mesh CSG replacing the AABB approximation | Long term | CODE_AUDIT |
| clippy warning baseline not cleared; `-D warnings` not enabled | Quality baseline (before G2) | CODE_AUDIT acceptance record |
| `sketch` constraint `let _ =`, `resolve_param` unknown parameters | Long term | CODE_AUDIT |
| Tests sharing the global `LIBRARY` | Long term | [TESTING](../guides/TESTING.md) |
| `eprintln!` not replaced by structured logging | Long term | CODE_AUDIT |
| Door geometry and storey-creation regressions | Current exception fixes | §7.3 of this document |

## 10. Open Questions

| # | Question | Owner | Impact |
|---|---|---|---|
| Q1 | Is G1 a customer-visible milestone or internal only? | Product | G1 |
| Q2 | Does G3 require an external security review? | Product / legal | G3 |
| Q3 | Who signs G2 when design and engineering disagree? | Product | G2 |
| Q4 | Is 7b's engineering exit confirmed as G1+G2, with G3 at release? | Product | 7b / G2 |
| Q5 | Do storey view, section, measure and 8-5 domain panels get requirement IDs? | Product / frontend | G1 / G2 |
| Q6 | Should hard-coded English UI strings be scanned too? | Frontend | G2 |
| Q7 | May 8-1 to 8-4 backend work proceed independently during the freeze? | Tech lead | 7b / 8 |

## 11. Maintenance Rules

1. **This file holds current state only.** History goes to [CHANGELOG](CHANGELOG.md), never copied here.
2. **Every ✅ needs an evidence link; every 📅 must belong to a batch or gate.** Items without an acceptance artefact do not belong in the roadmap.
3. **No calendar dates in the body.** Only the front-matter `last_updated` is kept; historical dates are CHANGELOG's job.
4. **A status change updates three things in the same commit**: this snapshot, the gate board, and CHANGELOG.
5. **Bilingual structure must match.** `check_docs.py` validates heading counts and numbering; the English file is a maintained translation, not an independent document.
6. **Batch labels stay stable.** Renames require a mapping table so other documents do not break.
