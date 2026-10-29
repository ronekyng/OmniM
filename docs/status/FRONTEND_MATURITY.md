---
title: "OmniM Frontend Maturity"
status: current
last_updated: 2026-10-29
applies_to: "web/"
evidence: "scored against web/src on 2026-10-29; requirement counts derived from REQUIREMENTS_REGISTER"
---
# OmniM Frontend Maturity

> **Document role.** This is the truth layer for frontend maturity. It scores the current implementation against ISO/IEC 25010 and tracks requirement coverage. Targets and contracts live in `docs/product/` and `docs/specification/ui/`.
>
> **Dual-track maintenance obligation**: `FRONTEND_MATURITY.zh-CN.md` is a maintained translation. Scores, counts, and requirement IDs must be identical in both files.
>
> **Scoring rule.** Scores describe code that exists today, not the plan. A score of 3 requires an implemented capability with a verification artifact.

## 1. Purpose and Method

Commercial readiness is not one number. ISO/IEC 25010 gives nine quality characteristics; each is scored 1–5 with evidence and a gate at which it must reach the target.

| Score | Meaning |
|---|---|
| 1 | Absent — no implementation, no policy |
| 2 | Prototype — works in demos, breaks under normal use |
| 3 | Usable — works for the main flow, known gaps documented |
| 4 | Commercial — measured, gated, and supported |
| 5 | Best-in-class — benchmarked and continuously improved |

The overall score is the arithmetic mean, reported to one decimal. It is **not** an average of intentions.

## 2. ISO/IEC 25010 Scorecard

| # | Characteristic | Score | Target (G3) | Evidence |
|---|---|---|---|---|
| 1 | Functional suitability | **3.0** | 4 | Creation, catalog, sketch, console, measure, section work; project lifecycle missing (UI-DAT-01/02) |
| 2 | Performance efficiency | **2.0** | 4 | Continuous render loop, full scene reload per mutation, no budgets enforced ([WEB_ARCHITECTURE](../architecture/WEB_ARCHITECTURE.md) §5) |
| 3 | Compatibility | **1.0** | 3 | No browser matrix, no WebGL2 gating, no DPI policy ([COMPATIBILITY](../specification/ui/COMPATIBILITY.md)) |
| 4 | Usability | **2.5** | 4 | Docked properties, title/breadcrumb, status bar and auto-naming landed; project home and mixed zh/en remain ([WEB_ARCHITECTURE](../architecture/WEB_ARCHITECTURE.md) §8) |
| 5 | Reliability | **2.0** | 4 | Panel/route error boundaries, explicit save + save-as, dirty interception; no autosave or crash recovery yet ([DATA_INTEGRITY](../specification/ui/DATA_INTEGRITY.md)) |
| 6 | Security | **1.0** | 3 | No auth disclosure, no path restriction, no role model ([SECURITY_PRIVACY](../specification/ui/SECURITY_PRIVACY.md)) |
| 7 | Maintainability | **2.5** | 3 | Pure modules for layout/save/naming with unit tests; still a 4,110-line stylesheet, a 1,823-line `ThreeViewer.tsx`, and no lint/format |
| 8 | Portability | **2.0** | 3 | Browser-only, no packaging decision, no minimum-size policy |
| 9 | Supportability | **1.0** | 3 | No error codes, no diagnostics bundle, no telemetry or help surfaces ([TELEMETRY](../specification/ui/TELEMETRY.md)) |
| | **Overall** | **1.9** | **3.4** | |

**Reading:** the frontend is a functional prototype with a strong geometry/viewport core and a weak product shell. The lowest scores cluster exactly where "feels like a demo" comes from: reliability, security, supportability, compatibility.

## 3. Requirement Coverage

Counts are derived from [REQUIREMENTS_REGISTER](../product/REQUIREMENTS_REGISTER.md) §3–§13.

| Area | Total | ✅ | 🔀 | 📅 | Coverage |
|---|---|---|---|---|---|
| SHL Shell and layout | 12 | 0 | 9 | 3 | 38% |
| DS Design system | 12 | 0 | 2 | 10 | 8% |
| INT Interaction | 12 | 2 | 6 | 4 | 42% |
| FBK Feedback and errors | 10 | 1 | 5 | 4 | 35% |
| DAT Data integrity | 10 | 0 | 3 | 7 | 15% |
| A11Y Accessibility | 8 | 0 | 0 | 8 | 0% |
| I18N Internationalization | 7 | 0 | 0 | 7 | 0% |
| PRF Performance | 8 | 0 | 0 | 8 | 0% |
| CMP Compatibility | 6 | 0 | 0 | 6 | 0% |
| SEC Security and privacy | 5 | 0 | 0 | 5 | 0% |
| TLM Telemetry and support | 6 | 0 | 0 | 6 | 0% |
| **Total** | **96** | **3** | **25** | **68** | **16%** |

Coverage counts `✅` as complete and `🔀` as half. The result (16%) is the honest distance from "commercial", independent of code quality.

## 4. Evidence Index

| Evidence | Type | Confirms |
|---|---|---|
| `web/package.json` | file | Stack and scripts |
| `web/src/app/App.tsx` | file | Routes, global shortcuts, toolbar wiring |
| `web/src/app/panels/PanelRegistry.tsx` | file | Panel registration and visibility |
| `web/src/app/store/appStore.ts` | file | State ownership, floating default |
| `web/src/app/components/ThreeViewer.tsx` | file | Rendering, picking, section, measure |
| `web/src/shared/api.ts` | file | REST surface used by the client |
| `web/src/__tests__/*.test.ts` | 8 files / 91 tests | Unit-level verification |
| `wc -l web/src/**` | 21,130 lines | Size and concentration |
| Color scan | 731 hex + 61 rgb/hsl across all CSS | Design-system debt |
| CJK literal scan | 854 runs across 31 files | i18n debt |
| Icon scan | 63 emoji occurrences, 25 used as panel icons | Icon-system debt |
| Dialog scan | 0 native dialogs; shared `ConfirmDialog` + `notify` | Feedback debt closed |
| A11y scan | 1 `aria-*` (plus dialog roles), 0 table/tree roles | Accessibility debt |
| Save-call scan | `saveProject` / `saveProjectAs` wired to toolbar + Ctrl/Cmd+S | Project lifecycle debt reduced |

## 5. Trend and Targets

| Gate | Required overall | Meaning |
|---|---|---|
| G0 | — | Contracts drafted and reviewed |
| G1 | ≥ 2.2 | Usable prototype: shell, save, no white screen |
| G2 | ≥ 3.0 | Commercial beta: themes, i18n, a11y, performance |
| G3 | ≥ 3.4 | GA: compatibility, security, supportability |

Historical entries are appended when a wave closes, with the date and the changed characteristics.

| Date | Overall | Change |
|---|---|---|
| 2026-10-29 | 1.7 | Baseline measurement; no frontend productization work yet |
| 2026-10-29 | 1.9 | G1 shell/reliability slice: docked properties, layout persistence, title/status bars, error boundaries, shared dialogs, explicit save/save-as, auto-naming |

## 6. Gate Readiness Snapshot

| Gate | Status | Blocking items |
|---|---|---|
| G0 | 🔀 Contracts drafted | Review and sign-off of PRD, IA_SHELL, DESIGN_SYSTEM, INTERACTION |
| G1 | 📅 Not ready | UI-SHL-01/02/04/05/07, UI-DAT-01, UI-FBK-01/02/08/09, UI-INT-01/04, UI-DS-02 |
| G2 | 📅 Not ready | DS/A11Y/I18N at pilot scope, PRF; remaining FBK and DAT |
| G3 | 📅 Not ready | Completeness tiers (DS-05/07/10, A11Y-06/08, I18N-05); all CMP, SEC, TLM; DAT-08/09/10 |

## 7. Non-Goals

1. A single "readiness percentage" for marketing use.
2. Scoring backend crates in this document.
3. Replacing usability testing with scores.

## 8. Open Questions

| # | Question | Owner |
|---|---|---|
| Q1 | Should partial scores be weighted by user-journey impact? | Product |
| Q2 | Who approves a score change, and with what evidence? | Product / Frontend |
| Q3 | Should maintainability be scored per module rather than globally? | Frontend |
