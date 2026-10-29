---
title: "OmniM Frontend Baseline (2026-10-29)"
status: current
last_updated: 2026-10-29
applies_to: "web/"
evidence: "static measurements on 2026-10-29; runtime and screenshot measurements pending instrumentation"
---
# OmniM Frontend Baseline (2026-10-29)

> **Document role.** This is the "before" snapshot used to judge every later improvement. It records static measurements now and reserves slots for runtime and visual measurements to be filled by the measurement tooling.
>
> **Dual-track maintenance obligation**: `FRONTEND_BASELINE.zh-CN.md` is a maintained translation. Numbers must match exactly.
>
> **Honesty rule.** Measured values and pending values are labeled differently. Nothing here is estimated without saying so.

## 1. Purpose and Scope

A commercial improvement program needs a reproducible starting point. This document fixes:

1. The static code-shape baseline.
2. The contract-debt baseline (counts that must reach zero).
3. The runtime baseline slot, pending instrumentation.
4. The visual baseline slot, pending screenshot capture.

## 2. Environment

| Item | Value |
|---|---|
| Date | 2026-10-29 |
| Commit | `88ee367` (`develop`) |
| Node | as pinned by `web/package-lock.json` |
| Browser | Not yet captured |
| OS | Linux x86-64 |
| Reference machine | Defined in [PERFORMANCE](../specification/ui/PERFORMANCE.md) §2; not yet provisioned |

## 3. Code Shape (measured)

| Metric | Value | Source |
|---|---|---|
| Frontend lines (ts/tsx/css) | 20,086 | `wc -l` |
| `ThreeViewer.tsx` | 1,794 | `wc -l` |
| `styles.css` | 3,985 | `wc -l` |
| `ComponentEditorPage.tsx` | 1,359 | `wc -l` |
| `SketchCanvas2D.tsx` | 1,038 | `wc -l` |
| `OmniPanel.tsx` | 937 | `wc -l` |
| `api.ts` | 371 | `wc -l` |
| CSS Modules | 7 | `find -name '*.module.css'` |
| Unit test files | 5 | `web/src/__tests__` |
| Unit tests passing | 79 | `npm test` |
| Type check | Passing | `tsc --noEmit` |

## 4. Contract-Debt Counts (measured)

These are the numbers the design-system and feedback requirements must drive to zero.

| Debt | Count | Target | Requirement |
|---|---|---|---|
| Hardcoded colors in `styles.css` | 712 hex (731 hex + 61 rgb/hsl across all CSS) | 0 | UI-DS-04 |
| `var(--*)` usages | 221 in `styles.css` (331 across all CSS) | — | — |
| Duplicate token blocks | 2 files (`globals.css`, `styles.css`) | 1 | UI-DS-01 |
| Emoji used as icons | 63 total, 25 as panel icons | 0 | UI-DS-09 |
| Chinese literals in source | 858 runs across 31 ts/tsx files | 0 | UI-I18N-01 |
| `window.confirm/alert/prompt` | 5 | 0 | UI-FBK-05 |
| Error boundaries | 0 | ≥4 (root/route/panel/viewport) | UI-FBK-06 |
| `aria-*` attributes | 1 | — | UI-A11Y-04 |
| `role` attributes | 0 | — | UI-A11Y-04 |
| `Date.now()` call sites (most build object names) | 52 | 0 | UI-INT-09 |
| `InstancedMesh` / LOD / virtualization | 0 | Required | UI-PRF-03, UI-PRF-07 |
| Client calls to `POST /api/project/save` | 0 | Required | UI-DAT-02 |
| Playwright / visual-regression configs | 0 | Required | UI-PRF-08, UI-DS |
| ESLint / Prettier configs for `web/` | 0 | Required | — |
| Layout persistence | None | Required | UI-SHL-06 |
| Theme provider | None | Required | UI-DS-02 |
| i18n provider | None | Required | UI-I18N-02 |

## 5. Runtime Baseline (pending)

The following must be measured by `web/tests/perf/` once the harness exists. They are **not** estimated here.

| Metric | Value | Status |
|---|---|---|
| First contentful paint | — | 📅 pending |
| Workspace ready | — | 📅 pending |
| Project open (2000 objects) | — | 📅 pending |
| Frame rate at 500 / 2000 / 5000 objects | — | 📅 pending |
| Idle CPU | — | 📅 pending |
| Memory at 2000 / 5000 objects | — | 📅 pending |
| Initial JS (gzip) | — | 📅 pending |
| Object-tree scroll at 5000 nodes | — | 📅 pending |

## 6. Visual Baseline (pending)

| Screenshot set | Status |
|---|---|
| Workspace, empty project, dark | 📅 pending |
| Workspace, 2000-object fixture, dark | 📅 pending |
| Workspace, light theme | 📅 pending |
| Properties panel with selection | 📅 pending |
| Creation panel and picking state | 📅 pending |
| Component editor, sketch editor | 📅 pending |
| Library manager, series manager | 📅 pending |
| Console and AI tabs | 📅 pending |
| Error and empty states | 📅 pending |
| 1280×720 and 1920×1080 | 📅 pending |

Capture procedure: fixed viewport, fixed fixture, deterministic camera, no cursor, both themes, PNG at 1× DPI.

## 7. What This Baseline Already Proves

1. The viewport and geometry stack is substantial (1,794-line renderer, shadows, tone mapping, sectioning, picking).
2. The product shell is thin: no project lifecycle, no theme, no i18n, no error boundary, no save UI.
3. The styling system is nominal: tokens exist but are bypassed 712 times.
4. The verification surface is unit-only: 79 tests, zero e2e, zero visual, zero performance.

## 8. Non-Goals

1. Comparing against competitor benchmarks.
2. Setting targets (targets live in `specification/ui/`).
3. Tracking per-file debt beyond the counts above.

## 9. Open Questions

| # | Question | Owner |
|---|---|---|
| Q1 | Should screenshots be committed or generated in CI only? | Frontend |
| Q2 | Which fixture project represents "realistic scale" for measurements? | Product |
| Q3 | Should the baseline be regenerated at every gate or only on demand? | Frontend |
