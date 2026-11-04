---
title: "OmniM UI Performance Budget"
status: design
last_updated: 2026-11-04
applies_to: "web/"
evidence: "n/a (design contract)"
---
# OmniM UI Performance Budget

> **Contract scope.** This document fixes the reference machine, startup and interaction budgets, rendering and memory budgets, bundle limits, large-list behavior, and the regression gate. It is the normative source for requirements `UI-PRF-01` … `UI-PRF-08`.
>
> **Dual-track maintenance obligation**: `PERFORMANCE.zh-CN.md` is a maintained translation. Keep section numbering, numbers, and requirement IDs aligned in the same change.
>
> **Status.** Design contract. No budget is currently enforced; the baseline is recorded in [FRONTEND_BASELINE](../../status/FRONTEND_BASELINE.md).

## 1. Purpose and Method

Performance is a product requirement because engineering models are large and users notice latency before they notice features. A budget is only real when it has a number, a machine, and a gate.

Method:

1. Every budget is measured on the reference machine (§2) with a fixed fixture.
2. Numbers are p95 over five runs unless stated otherwise.
3. Regression is measured against the committed baseline in `web/tests/perf/baseline.json`.
4. A budget change requires a recorded decision, not a silent edit.

## 2. Reference Machine

| Component | Specification |
|---|---|
| CPU | 8-core x86-64, 3.2 GHz base |
| RAM | 16 GB |
| GPU | Integrated or entry discrete, WebGL2 capable |
| Display | 1920×1080 at 100% DPI |
| Browser | Chromium latest stable |
| Server | Same machine, local loopback |
| Fixture | `fixtures/perf/office-2000.omnim` (2000 objects) |

All numbers in this document assume this machine. Slower machines get a proportional allowance declared in the release note, never a silent pass.

## 3. Startup and Interaction Budgets

| Metric | Budget | Measurement |
|---|---|---|
| First contentful paint | ≤ 1.2 s p95 | Navigation timing |
| First meaningful paint (workspace chrome) | ≤ 2.5 s p95 | Custom mark after shell mount |
| Workspace ready (viewport interactive) | ≤ 4.0 s p95 | Mark after first rendered frame |
| Project open (2000 objects) | ≤ 3.0 s p95 | From click to interactive |
| Requirement | UI-PRF-01 |
**Fixture measurement (2026-11-04, dev-server / Chromium; `e2e/perf.spec.ts`)**: FCP 312 ms (≤1.2 s ✓); shell ready 199 ms (≤2.5 s ✓); single-select properties render 123 ms. **Open-project ~4 s (>3 s ✗) and command palette ~4 s (>120 ms ✗)** appear on the remount path of a reused session — suspected WebGL context rebuild / first re-render stall; PRF-01/02 stay 🔀, recorded in the [RELEASE_READINESS](../../status/RELEASE_READINESS.md) G2 board; optimisation backlog. Open→ready is <3 s on a first fresh session.

### 3.2 Interaction latency

| Interaction | Budget |
|---|---|
| Button/toggle feedback | ≤ 100 ms |
| Panel/tab switch | ≤ 150 ms |
| Selection highlight | ≤ 100 ms |
| Property panel render for one object | ≤ 120 ms |
| Command palette open | ≤ 120 ms |
| Context menu open | ≤ 100 ms |
| Requirement | UI-PRF-02 |

**Rule:** anything above 300 ms needs a progress indicator (see [FEEDBACK_ERRORS](FEEDBACK_ERRORS.md) §4), not a longer spinner.

## 4. Rendering Budgets

### 4.1 Frame rate

| Scene | Target | Floor |
|---|---|---|
| 500 objects, orbit | 60 fps | 45 fps |
| 2000 objects, orbit | 45 fps | 30 fps |
| 2000 objects, static | 60 fps | 30 fps |
| 5000 objects, static (storey filtered) | 30 fps | 20 fps |

Frame time is measured on the render loop, excluding GC spikes above 50 ms which are reported separately.

### 4.2 Scene scale

| Rule | Value |
|---|---|
| Repeated catalog instances | Must use instancing above 50 identical meshes |
| Off-screen objects | Culled; no draw calls for fully clipped storeys |
| Section cut | Clipping planes, not per-triangle CPU work |
| Shadow map | 2048² maximum, updated on demand |

### 4.3 Idle behavior

| Metric | Budget |
|---|---|
| Idle CPU (no input, no animation) | < 1% of one core |
| Idle GPU | No continuous redraw |
| Requirement | UI-PRF-04 |

**Rule:** the render loop must be demand-driven; an unconditional `requestAnimationFrame` loop is a defect.

Requirement: UI-PRF-03.

## 5. Memory

| Metric | Budget |
|---|---|
| Baseline after workspace ready | ≤ 350 MB |
| 2000 objects | ≤ 800 MB |
| 5000 objects | ≤ 1.5 GB |
| 8-hour session growth | ≤ 10% over baseline |
| Geometry after undo/redo churn (500 ops) | No unbounded growth |

Measurement: Chromium heap snapshot plus `performance.memory` sampling every 60 s. Requirement: UI-PRF-05.

## 6. Bundle

| Asset | Budget |
|---|---|
| Initial JS (gzip) | ≤ 500 KB |
| Initial CSS (gzip) | ≤ 60 KB |
| Three.js chunk | Lazy, loaded with the viewport route |
| Editor routes | Separate chunks |
| Fonts | ≤ 2 families, subset, `font-display: swap` |
| Requirement | UI-PRF-06 |
**Delivered (2026-11-04, G2 · PRF-06)**: ThreeViewer/component-editor/library/series routes use `lazy()`; `snapSettings`/`typeColors` are extracted three-free; the build reports index gzip 141 KB and CSS 16.6 KB; the three.js graph lives in a lazy OrbitControls chunk (603 KB lazy, not fetched upfront).

Rules: no library above 50 KB gzip without a recorded decision; icon fonts are prohibited (use SVG).

## 7. Large Lists

| List | Scale | Technique | Budget |
|---|---|---|---|
| Object tree | 5000 nodes | Virtualization | Scroll ≥ 50 fps |
| Property table | 500 rows | Virtualization or paging | Scroll ≥ 50 fps |
| Catalog browser | 1000 components | Windowed grid | Filter response ≤ 200 ms |
| Console output | 10000 lines | Ring buffer | Append ≤ 16 ms |

Requirement: UI-PRF-07.

## 8. Regression Gate

| Rule | Value |
|---|---|
| Baseline file | `web/tests/perf/baseline.json`, committed |
| Tolerance | ±10% on p95, ±15% on memory |
| Failure | CI fails the performance job and blocks merge |
| Noise control | Median of 5 runs; discard the first warm-up run |
| Environment drift | Reference machine fingerprint recorded with the baseline |
| Requirement | UI-PRF-08 |

**Exception process:** a regression above tolerance requires a written justification in the PR and a follow-up issue with an owner and date.

## 9. Measurement Tooling

| Layer | Tool |
|---|---|
| Navigation and marks | `PerformanceObserver`, custom marks |
| Frame timing | `requestAnimationFrame` deltas, 5 s windows |
| Memory | Chromium heap snapshots, `measureUserAgentSpecificMemory` where available |
| Bundle | Vite build report, size-limit check |
| CI | Playwright performance project on the reference runner |

## 10. Non-Goals

1. Sub-16 ms interaction guarantees on integrated GPUs.
2. Mobile performance targets.
3. Network transfer optimization for cloud-hosted models.
4. Rendering quality parity with offline renderers.

## 11. Open Questions

| # | Question | Owner | Needed by |
|---|---|---|---|
| Q1 | Is the reference machine available in CI, or must we use a cloud runner tier? | Frontend / Infra | G2 |
| Q2 | Should the 5000-object fixture be generated or committed as a file? | Frontend | G2 |
| Q3 | Is 500 KB initial JS realistic with three.js loaded lazily? | Frontend | G1 |
