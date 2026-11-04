---
title: "OmniM Frontend Release Readiness"
status: current
last_updated: 2026-11-04
applies_to: "web/"
evidence: "gate status assessed against REQUIREMENTS_REGISTER and FRONTEND_MATURITY on 2026-10-30; includes the modeler UI/UX re-audit (G1 experience tier) scope note"
---
# OmniM Frontend Release Readiness

> **Document role.** This is the gate board. It states, per gate, what must be true, what is true today, and what blocks the next gate. It does not define the requirements themselves; those live in `docs/product/`.
>
> **Dual-track maintenance obligation**: `RELEASE_READINESS.zh-CN.md` is a maintained translation. Gate counts and status marks must match.
>
> **Update rule.** A gate item changes status only with a linked evidence artifact. Verbal sign-off is not evidence.

## 1. Purpose

A commercial release is a decision, not a date. This board makes the decision auditable: every gate has exit criteria, evidence, and a named owner.

## 2. Gate Model

| Gate | Name | Purpose | Exit owner |
|---|---|---|---|
| G0 | Contract ready | Design and requirements agreed before code | Product + Design |
| G1 | Alpha | Main workflow usable; no data-loss or white-screen class defects | Frontend lead |
| G2 | Beta | Product-quality UI: themes, i18n, a11y, performance, feedback | Product |
| G3 | GA | Supportable release: compatibility, security, telemetry, help | Product + Support |

Each gate has a hard rule: **a gate cannot be skipped because the previous one is "almost done"**.

## 3. G0 Contract Readiness

| Item | Required | Status | Evidence |
|---|---|---|---|
| Product vision and non-goals | Yes | 🔀 | [VISION_SCOPE](../product/VISION_SCOPE.md) drafted, review pending |
| Personas and journeys | Yes | 🔀 | [PERSONAS_JOURNEYS](../product/PERSONAS_JOURNEYS.md) drafted |
| PRD with 96 requirements | Yes | 🔀 | [PRD_FRONTEND](../product/PRD_FRONTEND.md) drafted |
| Requirement register (RTM) | Yes | 🔀 | [REQUIREMENTS_REGISTER](../product/REQUIREMENTS_REGISTER.md) drafted |
| Shell and IA contract | Yes | 🔀 | [IA_SHELL](../specification/ui/IA_SHELL.md) drafted |
| Design system contract | Yes | 🔀 | [DESIGN_SYSTEM](../specification/ui/DESIGN_SYSTEM.md) drafted |
| Interaction contract | Yes | 🔀 | [INTERACTION](../specification/ui/INTERACTION.md) drafted |
| Feedback/error contract | Yes | 🔀 | [FEEDBACK_ERRORS](../specification/ui/FEEDBACK_ERRORS.md) drafted |
| Data integrity contract | Yes | 🔀 | [DATA_INTEGRITY](../specification/ui/DATA_INTEGRITY.md) drafted |
| Non-functional contracts | Yes | 🔀 | A11Y, I18N, PERFORMANCE, COMPATIBILITY, SECURITY, TELEMETRY drafted |
| Document conventions and checks | Yes | ✅ | `check_ui_contracts.py` (0 failures) and `gen_keymap.py --check` run in the CI docs job |

**G0 verdict:** 🔀 **draft complete, review and sign-off pending.** Contract checks are automated and green; the remaining condition is the O1 decision recorded in the [G0 review](../process/G0_REVIEW_2026-10-29.md).

## 4. G1 Alpha Readiness

Complete list of the 25 G1 requirements (generated from the register, not a hand-picked subset).

| Requirement | Status | Evidence / blocker |
|---|---|---|
| UI-SHL-01 Three application levels | ✅ | `editors.spec.ts` covers the project/workspace/three-editor route matrix |
| UI-SHL-02 Seven fixed regions | ✅ | `web/e2e/shell.spec.ts` asserts left/title/toolbar/viewport/right/bottom/status |
| UI-SHL-03 Properties docked right | ✅ | Docked 260–600 px; `layout.spec.ts` resize + reload persistence |
| UI-SHL-04 Left dock tabs | ✅ | Objects / Grids / Catalog tabs; `layout.spec.ts` asserts 240–400 px |
| UI-SHL-05 Bottom dock collapsed and tabbed | ✅ | Collapsed by default, Console/AI tabs; `bottomDock.spec.ts` asserts ≤40% height |
| UI-SHL-06 Layout persistence and reset | ✅ | `layout.spec.ts`: resize → reload restores → Reset layout back to 340 px |
| UI-SHL-07 Contextual ribbon | ✅ | Four disciplines swap the command row only (`ribbon.spec.ts`) + current-tool indicator + group label |
| UI-SHL-08 Status bar readouts | ✅ | `web/e2e/shell.spec.ts` asserts coordinates/selection/units/save; FPS under developer mode |
| UI-SHL-10 Title and breadcrumb | ✅ | Workspace breadcrumb + editor-layer `PageBreadcrumb`; `editors.spec.ts` covers Series/Library back links |
| UI-DS-02 Dark/light switch | ✅ | `data-theme` + `localStorage` + OS default; `theme.spec.ts` covers reload-free switch, persistence and WCAG AA contrast in both themes |
| UI-INT-01 Five selection gestures | 🔀 | Click / Ctrl-click / box select exist; Esc clears tool first |
| UI-INT-02 Picking state machine | ✅ | `stateMachine.test.ts` |
| UI-INT-04 No shortcuts while typing | ✅ | `web/e2e/shell.spec.ts` presses Delete in the name input and the object survives; registry unification is UI-INT-03 |
| UI-INT-09 Auto-naming | ✅ | `naming.ts` + 4 unit tests; wired to create/quick-create/child/duplicate |
| UI-FBK-01 Panel three states | ✅ | Properties/catalog/domain panels + console empty + AI empty/loading/error (`routeCrash.spec.ts`) |
| UI-FBK-02 Skeleton timing | ✅ | 400 ms threshold unit test; properties/catalog/domain panels wired (console/AI use inline loading) |
| UI-FBK-05 No native dialogs | ✅ | `ConfirmDialog` + `notify`; `check_ui_contracts` dialogs = 0 |
| UI-FBK-06 Error boundaries | ✅ | Panel- and route-level crash tests landed (`feedback.spec.ts`, `routeCrash.spec.ts`) |
| UI-FBK-07 Typed toasts, errors persist | ✅ | Errors persist, success auto-dismisses (4 s); `feedback.spec.ts` asserts persistence and the ≤3 overflow count |
| UI-FBK-08 Dismissible, non-blocking feedback | ✅ | 320–420 px, above the status bar, ≤3 with overflow counter; `feedback.spec.ts` asserts placement and dismissal |
| UI-FBK-09 Empty states with action | ✅ | Object tree / domain / properties / catalog empty states carry guided next actions (`home.spec.ts` + panel checklist) |
| UI-DAT-01 Project home | ✅ | `home.spec.ts` covers sample → workspace → Save → recent list → new project |
| UI-DAT-02 Explicit save | ✅ | `data.spec.ts` covers Save / Save-as / a 500 escalation with retry |
| UI-DAT-03 Dirty state | ✅ | `data.spec.ts` asserts dirty copy and clean after Save |
| UI-DAT-06 Unsaved interception | 🔀 | `data.spec.ts` covers dirty→Stay, dirty→Leave, clean→leave; refresh/new-project and beforeunload branches pending |

**G1 verdict:** 📅 **not ready.** 23 of 25 complete (`UI-SHL-01/02/03/04/05/06/07/08/10`, `UI-DS-02`, `UI-INT-02/04/09`, `UI-FBK-01/02/05/06/07/08/09`, `UI-DAT-01/02/03`), 2 partial (`UI-INT-01`, `UI-DAT-06`), 0 absent.

> **Acceptance record (2026-10-30, scope note).** The modeler UI/UX re-audit ([MODELER_UIUX_REAUDIT_2026-10-30](MODELER_UIUX_REAUDIT_2026-10-30.zh-CN.md)) formally releases the **G1-tier UI/UX dimension**: both P0-UX findings (false save alarm, unit trap) are closed. This does not change the per-requirement gate status above — the 25 rows still follow `REQUIREMENTS_REGISTER` verification evidence; an experience acceptance does not upgrade a gate.

## 5. G2 Beta Readiness

| Area | Required | Status |
|---|---|---|
| Design system (12) | Tokens, themes, icons, viewport palette; 3 completeness items deferred to G3 | 🔀 5/12 (DS-02/03/04/08/09 done) |
| Accessibility (8) | WCAG 2.2 AA on critical paths; motion and live-region items deferred to G3 | 🔀 4/8 (A11Y-01/02/03/04 done; screen-reader review follows A11Y-05, aria-label localization follows I18N-01) |
| Internationalization (7) | 100% externalized and switchable; pseudo-locale deferred to G3 | 📅 |
| Performance (8) | All budgets enforced in CI | 📅 |
| Feedback (remaining) | Progress, coded errors, toasts | 🔀 |
| Data integrity (remaining) | Autosave, recovery, metadata | 📅 |
| Interaction (remaining) | Command registry (undo history, dialogs and focus management are closed) | 🔀 |

**Deferred to G3 on 2026-10-29** (O1 gate rebalance): `UI-DS-05`, `UI-DS-07`, `UI-DS-10`, `UI-A11Y-06`, `UI-A11Y-08`, `UI-I18N-05`. Rationale: G2 is the external-pilot gate; completeness tiers belong to GA. `UI-DS-08` stays at G2 because the z-index scale is a shell prerequisite and cheap to enforce.

**G2 verdict:** 🔀 **in progress.** 31 of 37 complete (`UI-SHL-12`, `UI-SHL-09/11`, `UI-DS-01/03/04/06/08/12`, `UI-INT-03/05/06/07/08/10/11/12`, `UI-A11Y-01/02/03/04`, `UI-I18N-02/03/04`, `UI-DAT-04/05/07`, `UI-FBK-03`, `UI-CMP-05`, `UI-PRF-06`); `UI-I18N-01`/`UI-PRF-01..04` partial (title-bar batch externalised; perf fixture built — open-to-ready ~4 s over budget, FPS/idle recorded under B9 software raster), the rest not started.

## 6. G3 GA Readiness

| Area | Required | Status |
|---|---|---|
| Completeness tiers | DS-05/07/10, A11Y-06/08, I18N-05 (moved from G2) | 📅 |
| Compatibility (6) | Browser/GPU/DPI matrix | 📅 |
| Security (5) | Disclosure, path roots, CSP, consent | 📅 |
| Telemetry and support (6) | Event dictionary, diagnostics, help | 📅 |
| Data integrity (final) | Conflict handling, dependencies, snapshots | 📅 |
| Conformance artifact | ACR/VPAT draft | 📅 |
| Release notes and migration | Template and process in [RELEASE_CHECKLIST](../process/RELEASE_CHECKLIST.md) | 📅 |
| Support runbook | Templates in [RELEASE_CHECKLIST](../process/RELEASE_CHECKLIST.md) §4–§5 | 📅 |

**G3 verdict:** 📅 **not ready.**

## 7. Blocking Issues

| # | Issue | Gate | Owner | Requirement |
|---|---|---|---|---|
| B7 | No i18n layer | G2 | Frontend | UI-I18N-01 |
| B8 | No accessibility semantics | G2 | Frontend | UI-A11Y-04 |
| B9 | Continuous render loop; no instancing | G2 | Frontend | UI-PRF-03/04 |
| B10 | No compatibility or security disclosure policy | G3 | Product | UI-CMP-01, UI-SEC-01 |

## 8. Evidence Log

| Date | Gate | Event | Evidence |
|---|---|---|---|
| 2026-10-29 | G0 | Contracts drafted (product + ui) | This document set |
| 2026-10-29 | — | Baseline measured | [FRONTEND_BASELINE](FRONTEND_BASELINE.md) |
| 2026-10-29 | G1 (partial) | Shell/reliability slice: docked properties, layout persistence, title/status bars, error boundaries, shared dialogs, explicit save/save-as, auto-naming | `web/src/__tests__/{layout,saveState,naming}.test.ts`; Rust `persistence::save_as_retargets_the_session_and_rejects_bad_paths` |
| 2026-10-29 | G1 (partial) | Project home: session gating, new/open/recent/sample, allowed-root enforcement | `crates/server/tests/projects.rs`; `web/src/__tests__/recentProjects.test.ts` |
| 2026-11-02 | G2 (partial) | Semantic token migration (UI-DS-04): single token source in `globals.css`, zero literal colours in components; unit tests for references and light overrides | `scripts/check_ui_contracts.py --only tokens`; `web/src/__tests__/designTokens.test.ts` |
| 2026-11-02 | G1/G2 (DS-02/03) | Dual-theme switch: reload-free, persisted, OS default, AA contrast in both themes | `web/e2e/theme.spec.ts` |
| 2026-11-02 | G2 (INT-06/07) | Dialog focus contract, inert background, focus restore; delete impact counted by child type | `web/e2e/dialogs.spec.ts`; `web/src/__tests__/deleteImpact.test.ts` |
| 2026-11-02 | G2 (DS-08) | Z-index collapsed onto the §7 scale tokens; component styles contain no magic numbers | `check_ui_contracts.py --only zindex`; `web/src/__tests__/designTokens.test.ts` |
| 2026-11-02 | G2 (DS-09) | Emoji/pictographic icons removed; single monochrome 16×16 SVG set (currentColor, 1.5 stroke) | `check_ui_contracts.py --only emoji` (0); `web/src/app/components/Icons.tsx` |
| 2026-11-02 | G2 (A11Y-04) | Object tree/dock tabs/bottom tabs/toolbar/menu/property table/toast/viewport semantics and keyboard behaviour | `web/e2e/a11y.spec.ts` |
| 2026-11-02 | G2 (A11Y-03) | `:focus-visible` 2 px ring, F6 region cycling, focus restore after deletion, route h1 and error fallback focus | `web/e2e/focus.spec.ts` |
| 2026-11-02 | G2 (A11Y-01) | axe audit of critical paths: launch, open, create, select, failed save, error fallback — 0 violations | `web/e2e/axe.spec.ts`; `npm run test:a11y` |
| 2026-11-02 | G2 (INT-08) | Undo/redo depth 60, cap 200, redo replay and grid update endpoint | `crates/server/tests/write_paths.rs`; `crates/server/tests/datum.rs` |
| 2026-11-02 | G2 (SHL-12/A11Y-02 partial) | Command palette, F1 shortcut help, keyboard-only J1 (create storey + save) | `web/e2e/commands.spec.ts` |
| 2026-11-03 | G2 (A11Y-02 close) | F2 inline rename, property-row arrows and Esc; tree double-click now renames | `web/e2e/a11y.spec.ts` |

## 9. Non-Goals

1. A public release calendar.
2. Marketing readiness.
3. Backend or IFC release gates (separate boards).

## 10. Open Questions

| # | Question | Owner | Needed by |
|---|---|---|---|
| Q1 | Is G1 a customer-visible milestone or internal only? | Product | G1 |
| Q2 | Does G3 require an external security review? | Product / Legal | G3 |
| Q3 | Which person signs G2 when design and engineering disagree? | Product | G2 |
