---
title: "OmniM Frontend Product Requirements"
status: design
last_updated: 2026-10-29
applies_to: "web/"
evidence: "n/a (product requirement)"
---
# OmniM Frontend Product Requirements

> **Document role.** This is the product requirements document (PRD) for the OmniM frontend. It defines *what* the UI must do, in testable terms. Contracts (*how*) live in `docs/specification/ui/`; current state lives in `docs/status/`.
>
> **Dual-track maintenance obligation**: `PRD_FRONTEND.zh-CN.md` is a maintained translation. Keep section numbering, requirement IDs, priorities, and gate columns aligned in the same change.
>
> **Single source of truth.** Requirement IDs are also listed in [REQUIREMENTS_REGISTER](REQUIREMENTS_REGISTER.md). If the two disagree, the register wins and this document must be corrected.

## 1. Purpose and Scope

The frontend must move from "feature prototype" to "commercial engineering application". Scope is the browser client in `web/` plus the minimal server endpoints it requires for project lifecycle.

Out of scope: backend domain modeling, geometry kernels, IFC mapping, and AI provider internals — those are covered by their own specifications.

## 2. Requirement Conventions

| Field | Rule |
|---|---|
| ID | `UI-<AREA>-<nn>`. Areas: SHL, DS, INT, FBK, DAT, A11Y, I18N, PRF, CMP, SEC, TLM. Never reused. |
| Priority | `MUST` (release blocking) or `SHOULD` (may slip one minor release with a recorded decision). |
| Acceptance | Observable and checkable without reading source. |
| Gate | G0 contract-ready, G1 alpha, G2 beta, G3 GA. Defined in [DEFINITION_OF_DONE](../process/DEFINITION_OF_DONE.md). |
| Status | ✅ implemented · 🔀 partial · 📅 planned · 💭 proposed. Mirrors the docs legend. |

**Traceability rule:** every requirement must name at least one verification artifact (test file, scan script, screenshot set, or manual checklist). Requirements without verification cannot pass a gate.

## 3. R1 — Shell and Layout

The workspace must read as a professional engineering application at first glance: stable regions, persistent state, no modal takeover of the viewport.

| ID | Requirement | Priority | Acceptance | Gate |
|---|---|---|---|---|
| UI-SHL-01 | The application MUST separate three levels: project layer, workspace layer, editor layer | MUST | Routes exist for project home, workspace, and each editor | G1 |
| UI-SHL-02 | The workspace MUST have seven fixed regions: menu bar, ribbon, left dock, viewport, right dock, bottom dock, status bar | MUST | All seven visible at 1440×900 with no overlap | G1 |
| UI-SHL-03 | The properties panel MUST dock on the right by default, resizable 260–600 px, double-click splitter resets width | MUST | Panel never overlays the viewport in default layout | G1 |
| UI-SHL-04 | The left dock MUST use tabs (object tree, grids, component library) with 240–400 px width | MUST | Three tabs switch without remounting the viewport | G1 |
| UI-SHL-05 | The bottom dock MUST be collapsed by default, expandable to ≤40% viewport height, tabbed | MUST | Console, AI, output, and history available as tabs | G1 |
| UI-SHL-06 | Layout state MUST persist and restore; a "reset layout" action MUST exist | MUST | Restart restores widths, tabs, and collapsed state | G1 |
| UI-SHL-07 | The ribbon MUST be contextual per discipline, showing only commands valid for the current domain | MUST | Switching domain changes the visible command set | G1 |
| UI-SHL-08 | The status bar MUST always show coordinates, selection count, unit system, and save state; frame rate MUST be available in developer mode | MUST | The four readouts update without a reload; FPS appears in developer mode | G1 |
| UI-SHL-09 | Panels SHOULD support a floating mode with persisted position and size | SHOULD | Floating panel survives restart at the same position | G2 |
| UI-SHL-10 | Every page MUST expose a title and breadcrumb | MUST | Workspace, component editor, library, series pages all labeled | G1 |
| UI-SHL-11 | At widths below 1280 px the docks MUST collapse to an icon rail, preserving viewport priority | MUST | 1024×768 renders with icon rails (degraded, not certified) | G2 |
| UI-SHL-12 | A command palette SHOULD aggregate all commands as a keyboard alternative to the ribbon | SHOULD | `Ctrl/Cmd+K` opens it; every ribbon command is reachable | G2 |

## 4. R2 — Design System and Theming

The UI must be themeable, token-driven, and consistent. Hardcoded visual values in user-facing code are a defect.

| ID | Requirement | Priority | Acceptance | Gate |
|---|---|---|---|---|
| UI-DS-01 | Tokens MUST have three layers: primitive, semantic, component; components MUST NOT consume primitives directly | MUST | Lint rule or scan reports zero primitive references in components | G2 |
| UI-DS-02 | Dark and light themes MUST switch via `data-theme` without a reload | MUST | Theme toggle applies in one frame | G1 |
| UI-DS-03 | The theme MUST follow the OS preference by default and persist an explicit user choice | MUST | First run follows `prefers-color-scheme` | G2 |
| UI-DS-04 | User-facing styles MUST contain zero hardcoded colors outside token files | MUST | Scan script reports zero | G2 |
| UI-DS-05 | Semantic colors MUST cover default/hover/active/disabled/selected/focus for every surface, border, and text role | MUST | Token matrix complete in both themes | G3 |
| UI-DS-06 | Typography MUST use one scale (11/12/13/15/20/28), body text 12–13 px, tabular numerals for numeric UI | MUST | No font size outside the scale | G2 |
| UI-DS-07 | Spacing MUST be on a 4 px base (4/8/12/16/24/32); radii and shadows MUST use the defined sets | MUST | Scan detects no off-scale values | G3 |
| UI-DS-08 | A z-index scale MUST exist and be used (viewport, overlays, docks, dialogs, toasts, debug) | MUST | No ad-hoc z-index above the defined bands | G2 |
| UI-DS-09 | Icons MUST use 16/20/24 px sizes with 1.5 px stroke; emoji MUST NOT be used as icons | MUST | Emoji-as-icon scan returns zero | G2 |
| UI-DS-10 | Every interactive component MUST define default/hover/active/selected/disabled/focus/loading/error states | MUST | Component state matrix has no gaps | G3 |
| UI-DS-11 | A high-contrast theme SHOULD exist as an accessibility mapping, not a third brand theme | SHOULD | Contrast tests pass in high-contrast mode | G3 |
| UI-DS-12 | The 3D viewport MUST have its own token set per theme (background, grid, edges, selection, hover, section, measurement) | MUST | Viewport colors change with theme | G2 |

## 5. R3 — Interaction and Command Model

Interaction must be predictable, keyboard-complete, and free of browser-native dialogs.

| ID | Requirement | Priority | Acceptance | Gate |
|---|---|---|---|---|
| UI-INT-01 | Selection MUST support click, Ctrl/Cmd toggle, empty-space clear, box select, and Esc clear | MUST | All five gestures verified | G1 |
| UI-INT-02 | Picking MUST follow a state machine: idle → configuring → picking → creating → done/error; Enter completes, Backspace removes the last point, Esc cancels | MUST | State machine unit tests cover every transition | G1 |
| UI-INT-03 | Keyboard shortcuts MUST come from one registry; documentation is generated from it | MUST | No component-local global key handling outside the registry | G2 |
| UI-INT-04 | Global shortcuts MUST NOT fire while a text input or textarea has focus | MUST | Typing "w" in a field never switches tools | G1 |
| UI-INT-05 | Context menus MUST be contextual (viewport, object tree, property row) and keyboard-invokable | MUST | `Shift+F10` opens the same menu | G2 |
| UI-INT-06 | Dialogs MUST be one component family: Esc closes, Enter confirms, focus is trapped and restored | MUST | Focus returns to the invoking control | G2 |
| UI-INT-07 | Destructive actions MUST require confirmation that states impact and reversibility | MUST | Delete of a parent lists affected children | G2 |
| UI-INT-08 | Undo/redo MUST cover every mutating endpoint; the UI MUST show whether the last action is undoable | MUST | Endpoint coverage test passes; depth ≥50 | G2 |
| UI-INT-09 | New objects MUST be auto-named `Type-nnn` with rename and duplicate-name detection | MUST | Creating ten walls yields Wall-001…Wall-010 | G1 |
| UI-INT-10 | Units MUST be displayed consistently (viewport metres, catalog millimetres) with unit suffixes on inputs | MUST | No unlabeled numeric field in creation panels | G2 |
| UI-INT-11 | Double-click MUST have one documented meaning per context and MUST NOT trigger hidden destructive actions | MUST | Context table published in INTERACTION §7 | G2 |
| UI-INT-12 | Viewport navigation (orbit/pan/zoom) MUST be configurable and consistent with the view cube | MUST | Navigation settings persist | G2 |

## 6. R4 — Feedback and Errors

Every asynchronous or failing operation must explain itself and offer a next step. White screens and native alerts are defects.

| ID | Requirement | Priority | Acceptance | Gate |
|---|---|---|---|---|
| UI-FBK-01 | Every data-driven panel MUST implement loading, empty, and error states | MUST | Checklist covers all panels | G1 |
| UI-FBK-02 | Loading MUST use skeletons and MUST NOT flash for operations under 400 ms | MUST | No spinner flash on fast responses | G1 |
| UI-FBK-03 | Operations over one second MUST show progress and offer cancel where safe | MUST | Import/export/rebuild all show progress | G2 |
| UI-FBK-04 | Every user-visible error MUST include a code, a plain-language cause, and a recovery action | MUST | Error catalogue has no uncoded entries | G3 |
| UI-FBK-05 | `window.alert/confirm/prompt` MUST NOT be used | MUST | Scan returns zero | G1 |
| UI-FBK-06 | Error boundaries MUST isolate failures per route and per panel | MUST | A throwing panel does not blank the workspace | G1 |
| UI-FBK-07 | Toasts MUST be typed (success/info/warning/error); error toasts MUST NOT auto-dismiss | MUST | Error toast persists until dismissed | G1 |
| UI-FBK-08 | Success feedback MUST be dismissible and MUST NOT cover the viewport centre | MUST | Toast position respects the viewport safe area | G1 |
| UI-FBK-09 | Empty states MUST offer the next action (create, import, sample) | MUST | Every empty state has a primary action | G1 |
| UI-FBK-10 | A diagnostics bundle SHOULD be exportable (version, config, logs, recent actions, no geometry) | SHOULD | Bundle contains no model payload | G3 |

## 7. R5 — Data Integrity and Project Lifecycle

The user must always know where their model is, whether it is saved, and how to recover it.

| ID | Requirement | Priority | Acceptance | Gate |
|---|---|---|---|---|
| UI-DAT-01 | The app MUST start at a project home with new, open, recent, and sample options | MUST | No implicit project is loaded into the workspace | G1 |
| UI-DAT-02 | Explicit save (`Ctrl/Cmd+S`) and save-as MUST exist; a failed save MUST keep the dirty state and offer retry | MUST | Save failure path covered by e2e | G1 |
| UI-DAT-03 | The UI MUST show a dirty mark and the last-saved time | MUST | Mark appears on mutation and clears on save | G1 |
| UI-DAT-04 | Autosave MUST run at most every 60 s and MUST NOT block interaction | MUST | Autosave does not drop frames | G2 |
| UI-DAT-05 | On restart after a crash, the app MUST offer recovery from the latest autosave with ≤5 minutes loss | MUST | Kill-and-restart drill passes | G2 |
| UI-DAT-06 | New/open MUST intercept unsaved changes with save/discard/cancel | MUST | All three branches verified | G1 |
| UI-DAT-07 | Project metadata (name, units, description, version) MUST be editable and persisted | MUST | Values survive reopen | G2 |
| UI-DAT-08 | External modification of the project file MUST be detected; the UI MUST become read-only and explain the conflict | MUST | mtime/hash change triggers the state | G3 |
| UI-DAT-09 | Missing catalog/series dependencies on open MUST be listed explicitly | MUST | Dependency report shown before editing | G3 |
| UI-DAT-10 | Destructive operations MUST create a restorable snapshot | MUST | Snapshot restore returns the prior state | G3 |

## 8. R6 — Accessibility

| ID | Requirement | Priority | Acceptance | Gate |
|---|---|---|---|---|
| UI-A11Y-01 | Critical paths MUST meet WCAG 2.2 AA (text 4.5:1, non-text 3:1) | MUST | axe reports zero critical on critical paths | G2 |
| UI-A11Y-02 | Every command MUST be keyboard-reachable; a shortcut help surface MUST exist | MUST | Keyboard-only task completion | G2 |
| UI-A11Y-03 | Focus MUST be visible, ordered, and trapped in dialogs | MUST | No focus loss after closing a dialog | G2 |
| UI-A11Y-04 | Trees, property tables, and tabs MUST expose roles and ARIA state | MUST | Screen reader announces selection and expansion | G2 |
| UI-A11Y-05 | Icon-only buttons MUST have accessible names; the viewport MUST provide a text alternative | MUST | No unnamed button in the accessibility tree | G2 |
| UI-A11Y-06 | `prefers-reduced-motion` MUST disable non-essential animation | MUST | Toggle removes transitions | G3 |
| UI-A11Y-07 | The high-contrast theme (defined by UI-DS-11) SHOULD meet AA on all UI text | SHOULD | Contrast audit passes | G3 |
| UI-A11Y-08 | Operation results and selection changes MUST be announced via `aria-live` | MUST | Screen reader reports create/delete/save | G3 |

## 9. R7 — Internationalization

| ID | Requirement | Priority | Acceptance | Gate |
|---|---|---|---|---|
| UI-I18N-01 | 100% of user-visible strings MUST be externalized; source files MUST contain no Chinese literals outside dictionaries | MUST | String scan returns zero | G2 |
| UI-I18N-02 | Default locale is `zh-CN`, fallback `en-US`; switching applies without reload | MUST | Both locales render the full UI | G2 |
| UI-I18N-03 | Plurals and selections MUST use ICU MessageFormat | MUST | Plural cases verified in both locales | G2 |
| UI-I18N-04 | Numbers, dates, and units MUST respect the active locale; unit system MUST be switchable | MUST | 1,000.5 vs 1.000,5 render correctly | G2 |
| UI-I18N-05 | A pseudo-localization build MUST pass with no overflow or truncation | MUST | Pseudo-locale screenshot review | G3 |
| UI-I18N-06 | Keys MUST follow `area.component.key`; unused keys SHOULD be detected | SHOULD | Unused-key report in CI | G3 |
| UI-I18N-07 | RTL layout SHOULD be structurally possible (logical properties, no hardcoded left/right) | SHOULD | Scan reports no physical-direction CSS | G3 |

## 10. R8 — Performance

Budgets are defined in [PERFORMANCE](../specification/ui/PERFORMANCE.md) with a reference machine. Values below are the product-level summary.

| ID | Requirement | Priority | Acceptance | Gate |
|---|---|---|---|---|
| UI-PRF-01 | FCP ≤1.2 s P95; FMP ≤2.5 s P95; workspace ready ≤4 s | MUST | Harness reports within budget | G2 |
| UI-PRF-02 | Interaction feedback ≤100 ms; panel switch ≤150 ms | MUST | Interaction trace within budget | G2 |
| UI-PRF-03 | Viewport ≥30 fps at 2000 objects; target 60 fps at 500 | MUST | Frame-time harness | G2 |
| UI-PRF-04 | Idle CPU <1% (on-demand rendering) | MUST | 60 s idle sample | G2 |
| UI-PRF-05 | Memory ≤1.5 GB at 5000 objects and no growth over an 8 h session | MUST | Memory soak test | G3 |
| UI-PRF-06 | Initial JS ≤500 KB gzip with route-level splitting | MUST | Bundle report | G2 |
| UI-PRF-07 | Object tree scrolls ≥50 fps at 5000 nodes (virtualized) | MUST | Scroll trace | G3 |
| UI-PRF-08 | Performance regressions beyond ±10% of baseline MUST fail CI | MUST | CI gate | G3 |

## 11. R9 — Compatibility

| ID | Requirement | Priority | Acceptance | Gate |
|---|---|---|---|---|
| UI-CMP-01 | Supported browsers MUST be stated (Chromium/Firefox/Safari, latest two major versions) and tested | MUST | Matrix run passes | G3 |
| UI-CMP-02 | WebGL2 capability MUST be detected; unsupported environments MUST block with an explanation | MUST | Forced-failure screen verified | G3 |
| UI-CMP-03 | DPI 100/125/150/200% and browser zoom 80–200% MUST not break layout | MUST | Screenshot matrix passes | G3 |
| UI-CMP-04 | 3D mouse (SpaceMouse) SHOULD be supported or explicitly declared unsupported | SHOULD | Decision recorded | G3 |
| UI-CMP-05 | The workspace MUST be certified from 1280×720 upward; 1024–1279 px is degraded only | MUST | Minimum-size walkthrough at 1280×720 | G2 |
| UI-CMP-06 | Server-unreachable and weak-network states SHOULD degrade with explicit retry | SHOULD | Offline banner and retry verified | G3 |

## 12. R10 — Security and Privacy

| ID | Requirement | Priority | Acceptance | Gate |
|---|---|---|---|---|
| UI-SEC-01 | When the server has no authentication, the UI MUST state "local use only" and MUST NOT offer remote sharing | MUST | Banner and About dialog state it | G3 |
| UI-SEC-02 | Destructive and sensitive commands MUST be permission-gated (reserved), hidden rather than erroring | MUST | Disabled/hidden state per role | G3 |
| UI-SEC-03 | Project open MUST be restricted to configured root directories | MUST | Path traversal rejected | G3 |
| UI-SEC-04 | The client MUST run under a CSP and MUST NOT render untrusted HTML | MUST | CSP header present; no unsafe user-content HTML | G3 |
| UI-SEC-05 | Telemetry MUST be opt-in; PII MUST NOT be stored | MUST | Consent gate and event audit | G3 |

## 13. R11 — Telemetry and Supportability

| ID | Requirement | Priority | Acceptance | Gate |
|---|---|---|---|---|
| UI-TLM-01 | An event dictionary MUST define name, properties, trigger point, and purpose for every event | MUST | Dictionary exists and is versioned | G3 |
| UI-TLM-02 | Key funnels MUST be instrumented: first run → project → first create → save → reopen | MUST | Funnel events observed in a session | G3 |
| UI-TLM-03 | Error events MUST carry the error code and context without model content | MUST | Payload audit | G3 |
| UI-TLM-04 | Performance metrics SHOULD be reported as percentiles | SHOULD | P50/P95 present | G3 |
| UI-TLM-05 | Local diagnostic logs MUST be exportable, redacted, and MUST NOT contain user geometry (defines the redaction and support-channel contract; the export action itself is UI-FBK-10) | MUST | Bundle inspection | G3 |
| UI-TLM-06 | Help surfaces MUST include shortcut reference, contextual help, sample projects, and version/about | MUST | All four reachable | G3 |

## 14. Traceability and Change Control

- Every requirement ID appears in exactly one row of [REQUIREMENTS_REGISTER](REQUIREMENTS_REGISTER.md) with contract, verification, gate, and status.
- Changing a requirement requires: edit this document, update the register, update the contract, and note the change in [CHANGELOG](../status/CHANGELOG.md).
- Removing a requirement requires marking it `Superseded by UI-XXX-nn`; IDs are never deleted.
- A requirement may not move to an earlier gate without a recorded decision.

## 15. Non-Goals

1. Pixel-perfect parity with any specific vendor's UI.
2. Public plugin SDK or third-party panel marketplace.
3. Mobile layout below 1024 px width.
4. Cloud multi-user presence, comments, or co-editing.
5. Theming beyond dark, light, and high contrast.

## 16. Open Questions

| # | Question | Owner | Needed by |
|---|---|---|---|
| Q1 | Which locale is the shipping default for the first commercial release? | Product | G1 |
| Q2 | Is read-only reviewer mode a permission or a UI mode? | Product / Backend | G2 |
| Q3 | Does the acceptance bar for G2 include the AI console, or is it workspace-only? | Product | G2 |
| Q4 | Which tier of telemetry is acceptable for an offline-first product? | Product / Legal | G3 |
