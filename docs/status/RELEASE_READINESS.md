---
title: "OmniM Frontend Release Readiness"
status: current
last_updated: 2026-10-29
applies_to: "web/"
evidence: "gate status assessed against REQUIREMENTS_REGISTER and FRONTEND_MATURITY on 2026-10-29"
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
| UI-SHL-01 Three application levels | 🔀 | `/projects` + workspace + editors; route-matrix e2e pending |
| UI-SHL-02 Seven fixed regions | 🔀 | Regions exist but not as a stable seven-zone shell |
| UI-SHL-03 Properties docked right | 🔀 | Docked by default; width persisted (`layout.ts`); no resize e2e |
| UI-SHL-04 Left dock tabs | 🔀 | Left sidebar exists; no tab groups |
| UI-SHL-05 Bottom dock collapsed and tabbed | 🔀 | Console is always mounted, no tabs |
| UI-SHL-06 Layout persistence and reset | 🔀 | `omnim.layout.v1` (right width/mode) + Reset layout; reload e2e pending |
| UI-SHL-07 Contextual ribbon | 🔀 | Flat toolbar, not discipline-scoped |
| UI-SHL-08 Status bar readouts | 🔀 | Coordinates, selection, units, save state; FPS behind developer mode |
| UI-SHL-10 Title and breadcrumb | 🔀 | Workspace title/breadcrumb + document title; editor breadcrumbs pending |
| UI-DS-02 Dark/light switch | 📅 | No theme provider or `data-theme` |
| UI-INT-01 Five selection gestures | 🔀 | Click / Ctrl-click / box select exist; Esc clears tool first |
| UI-INT-02 Picking state machine | ✅ | `stateMachine.test.ts` |
| UI-INT-04 No shortcuts while typing | 🔀 | `App.tsx` guards input/textarea; not yet registry-wide |
| UI-INT-09 Auto-naming | ✅ | `naming.ts` + 4 unit tests; wired to create/quick-create/child/duplicate |
| UI-FBK-01 Panel three states | 🔀 | Loading mostly; empty/error partial |
| UI-FBK-02 Skeleton timing | 📅 | No skeletons |
| UI-FBK-05 No native dialogs | ✅ | `ConfirmDialog` + `notify`; `check_ui_contracts` dialogs = 0 |
| UI-FBK-06 Error boundaries | 🔀 | Application, workspace and every panel slot; throwing-panel test pending |
| UI-FBK-07 Typed toasts, errors persist | 🔀 | Typed toasts exist; errors auto-dismiss |
| UI-FBK-08 Dismissible, non-blocking feedback | 🔀 | Toasts are dismissible; placement overlaps chrome |
| UI-FBK-09 Empty states with action | 🔀 | Some panels only |
| UI-DAT-01 Project home | 🔀 | `/projects` with new/open/recent/sample; server list/new/open + allowed-root; first-run e2e pending |
| UI-DAT-02 Explicit save | 🔀 | Save button + Ctrl/Cmd+S + Save as… (`/api/project/save-as`); round-trip e2e pending |
| UI-DAT-03 Dirty state | 🔀 | `saveState.ts` + title/status indicators; state e2e pending |
| UI-DAT-06 Unsaved interception | 🔀 | `beforeunload` while dirty/failed; in-app navigation guard pending |

**G1 verdict:** 📅 **not ready.** 3 of 25 complete (`UI-INT-02`, `UI-INT-09`, `UI-FBK-05`), 20 partial, 2 absent (`UI-DS-02`, `UI-FBK-02`).

## 5. G2 Beta Readiness

| Area | Required | Status |
|---|---|---|
| Design system (12) | Tokens, themes, icons, viewport palette; 3 completeness items deferred to G3 | 📅 |
| Accessibility (8) | WCAG 2.2 AA on critical paths; motion and live-region items deferred to G3 | 📅 |
| Internationalization (7) | 100% externalized and switchable; pseudo-locale deferred to G3 | 📅 |
| Performance (8) | All budgets enforced in CI | 📅 |
| Feedback (remaining) | Progress, coded errors, toasts | 🔀 |
| Data integrity (remaining) | Autosave, recovery, metadata | 📅 |
| Interaction (remaining) | Command registry, dialogs, undo history | 🔀 |

**Deferred to G3 on 2026-10-29** (O1 gate rebalance): `UI-DS-05`, `UI-DS-07`, `UI-DS-10`, `UI-A11Y-06`, `UI-A11Y-08`, `UI-I18N-05`. Rationale: G2 is the external-pilot gate; completeness tiers belong to GA. `UI-DS-08` stays at G2 because the z-index scale is a shell prerequisite and cheap to enforce.

**G2 verdict:** 📅 **not ready.**

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
| B6 | Tokens bypassed in 731 places | G2 | Frontend | UI-DS-04 |
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
