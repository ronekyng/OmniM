---
title: "OmniM Frontend Requirements Register (RTM)"
status: design
last_updated: 2026-10-31
applies_to: "web/"
evidence: "n/a (product requirement)"
---
# OmniM Frontend Requirements Register (RTM)

> **Document role.** This is the single source of truth for frontend requirement traceability. Every requirement has one ID, one contract anchor, at least one verification artifact, one release gate, and one status. Nothing else in the docs set may invent an ID.
>
> **Dual-track maintenance obligation**: `REQUIREMENTS_REGISTER.zh-CN.md` is a maintained translation. IDs, gate letters, and status marks MUST be identical in both files.
>
> **Machine-checked.** `scripts/check_ui_contracts.py` verifies: ID uniqueness, presence in the PRD, a non-empty verification column, valid gate and status values, and the declared total of 96. `scripts/gen_keymap.py --check` verifies keyboard-map parity.

## 1. How to Read This Register

| Column | Meaning |
|---|---|
| ID | Stable requirement identifier, `UI-<AREA>-<nn>` |
| Requirement | Short form; the normative text is in [PRD_FRONTEND](PRD_FRONTEND.md) |
| Contract | Where the design contract lives |
| Verification | The artifact that proves it (test, scan, screenshot set, checklist) |
| Gate | Earliest gate at which the requirement must pass |
| Status | ✅ implemented · 🔀 partial · 📅 planned · 💭 proposed |

**Status honesty rule:** status reflects the code as of `last_updated`, not the intent of this document. A requirement can be `MUST` and still be 📅.

## 2. ID Allocation

| Area | Meaning | Count | Contract |
|---|---|---|---|
| SHL | Shell and layout | 12 | [IA_SHELL](../specification/ui/IA_SHELL.md) |
| DS | Design system and theming | 12 | [DESIGN_SYSTEM](../specification/ui/DESIGN_SYSTEM.md) |
| INT | Interaction and command model | 12 | [INTERACTION](../specification/ui/INTERACTION.md) |
| FBK | Feedback and errors | 10 | [FEEDBACK_ERRORS](../specification/ui/FEEDBACK_ERRORS.md) |
| DAT | Data integrity and project lifecycle | 10 | [DATA_INTEGRITY](../specification/ui/DATA_INTEGRITY.md) |
| A11Y | Accessibility | 8 | [A11Y](../specification/ui/A11Y.md) |
| I18N | Internationalization | 7 | [I18N_L10N](../specification/ui/I18N_L10N.md) |
| PRF | Performance | 8 | [PERFORMANCE](../specification/ui/PERFORMANCE.md) |
| CMP | Compatibility | 6 | [COMPATIBILITY](../specification/ui/COMPATIBILITY.md) |
| SEC | Security and privacy | 5 | [SECURITY_PRIVACY](../specification/ui/SECURITY_PRIVACY.md) |
| TLM | Telemetry and supportability | 6 | [TELEMETRY](../specification/ui/TELEMETRY.md) |
| | **Total** | **96** | |

## 3. Register: SHL — Shell and Layout

| ID | Requirement | Contract | Verification | Gate | Status |
|---|---|---|---|---|---|
| UI-SHL-01 | Three application levels (project, workspace, editor) | IA_SHELL §2 | e2e route matrix | G1 | 🔀 |
| UI-SHL-02 | Seven fixed workspace regions | IA_SHELL §3 | `web/e2e/shell.spec.ts` | G1 | ✅ |
| UI-SHL-03 | Properties docked right, 260–600 px | IA_SHELL §4 | `web/e2e/layout.spec.ts` | G1 | ✅ |
| UI-SHL-04 | Left dock tabs, 240–400 px | IA_SHELL §4 | `web/e2e/layout.spec.ts` | G1 | ✅ |
| UI-SHL-05 | Bottom dock collapsed, tabbed, ≤40% height | IA_SHELL §4 | screenshot `bottom-collapsed` | G1 | 🔀 |
| UI-SHL-06 | Layout persistence and reset | IA_SHELL §5 | `web/e2e/layout.spec.ts` | G1 | ✅ |
| UI-SHL-07 | Contextual ribbon per discipline | IA_SHELL §3.3 | command-set snapshot test | G1 | 🔀 |
| UI-SHL-08 | Status bar with five live readouts | IA_SHELL §6 | `web/e2e/shell.spec.ts` (incl. developer FPS) | G1 | ✅ |
| UI-SHL-09 | Floating panels with persisted geometry | IA_SHELL §4.4 | e2e drag/resize/reload | G2 | 🔀 |
| UI-SHL-10 | Title and breadcrumb on every page | IA_SHELL §2.2 | route matrix assertions | G1 | 🔀 |
| UI-SHL-11 | Icon-rail collapse below 1280 px | IA_SHELL §7 | 1024×768 screenshot | G2 | 📅 |
| UI-SHL-12 | Command palette | IA_SHELL §8 | e2e command coverage | G2 | 📅 |

## 4. Register: DS — Design System and Theming

| ID | Requirement | Contract | Verification | Gate | Status |
|---|---|---|---|---|---|
| UI-DS-01 | Three token layers, no primitive leakage | DESIGN_SYSTEM §2 | `check_ui_contracts.py --tokens` | G2 | 📅 |
| UI-DS-02 | Dark/light switch without reload | DESIGN_SYSTEM §3 | `theme.test.ts` + theme screenshots pending | G1 | 🔀 |
| UI-DS-03 | OS preference default, choice persisted | DESIGN_SYSTEM §3.2 | emulation test | G2 | 📅 |
| UI-DS-04 | Zero hardcoded colors in user-facing styles | DESIGN_SYSTEM §2.4 | color scan | G2 | 📅 |
| UI-DS-05 | Complete semantic state matrix | DESIGN_SYSTEM §4 | token matrix review | G3 | 📅 |
| UI-DS-06 | Single type scale, tabular numerals | DESIGN_SYSTEM §5 | style scan + screenshot | G2 | 🔀 |
| UI-DS-07 | 4 px spacing base, fixed radii and shadows | DESIGN_SYSTEM §6 | style scan | G3 | 🔀 |
| UI-DS-08 | Z-index scale | DESIGN_SYSTEM §7 | style scan | G2 | 📅 |
| UI-DS-09 | Icon sizes and stroke, no emoji icons | DESIGN_SYSTEM §8 | emoji scan | G2 | 📅 |
| UI-DS-10 | Component state matrix complete | DESIGN_SYSTEM §9 | component inventory review | G3 | 📅 |
| UI-DS-11 | High-contrast accessibility theme | DESIGN_SYSTEM §3.4 | contrast audit | G3 | 📅 |
| UI-DS-12 | Per-theme viewport token set | DESIGN_SYSTEM §10 | screenshot pair per theme | G2 | 📅 |

## 5. Register: INT — Interaction and Command Model

| ID | Requirement | Contract | Verification | Gate | Status |
|---|---|---|---|---|---|
| UI-INT-01 | Five selection gestures | INTERACTION §3 | interaction e2e | G1 | 🔀 |
| UI-INT-02 | Picking state machine contract | INTERACTION §4 | `stateMachine.test.ts` | G1 | ✅ |
| UI-INT-03 | Single shortcut registry, generated docs | INTERACTION §5 | `gen_keymap.py --check` | G2 | 📅 |
| UI-INT-04 | No global shortcuts while typing | INTERACTION §5.3 | `web/e2e/shell.spec.ts` | G1 | ✅ |
| UI-INT-05 | Contextual and keyboard-invokable menus | INTERACTION §6 | menu e2e | G2 | 🔀 |
| UI-INT-06 | One dialog family with focus management | INTERACTION §7 | focus-trap test | G2 | 📅 |
| UI-INT-07 | Destructive confirmation with impact | INTERACTION §7.3 | delete-parent e2e | G2 | 📅 |
| UI-INT-08 | Undo/redo across all mutations, depth ≥50 | INTERACTION §8 | endpoint coverage test | G2 | 🔀 |
| UI-INT-09 | Auto-naming with duplicate detection | INTERACTION §9 | naming unit tests | G1 | ✅ |
| UI-INT-10 | Consistent units with suffixes | INTERACTION §9.2 | form audit | G2 | 🔀 |
| UI-INT-11 | One meaning per double-click context | INTERACTION §7.4 | context table review | G2 | 📅 |
| UI-INT-12 | Configurable viewport navigation | INTERACTION §10 | settings e2e | G2 | 🔀 |

## 6. Register: FBK — Feedback and Errors

| ID | Requirement | Contract | Verification | Gate | Status |
|---|---|---|---|---|---|
| UI-FBK-01 | Loading/empty/error in every panel | FEEDBACK_ERRORS §3 | panel checklist | G1 | 🔀 |
| UI-FBK-02 | Skeletons, no flash under 400 ms | FEEDBACK_ERRORS §3.2 | `web/src/__tests__/delayedVisibility.test.ts` | G1 | 🔀 |
| UI-FBK-03 | Progress and cancel over 1 s | FEEDBACK_ERRORS §4 | import e2e | G2 | 📅 |
| UI-FBK-04 | Coded errors with cause and recovery | FEEDBACK_ERRORS §5 | error catalogue diff | G3 | 📅 |
| UI-FBK-05 | No native alert/confirm/prompt | FEEDBACK_ERRORS §6 | `window.*` scan | G1 | ✅ |
| UI-FBK-06 | Route- and panel-scoped error boundaries | FEEDBACK_ERRORS §7 | throwing-panel test | G1 | 🔀 |
| UI-FBK-07 | Typed toasts; errors persist | FEEDBACK_ERRORS §8 | toast e2e | G1 | 🔀 |
| UI-FBK-08 | Dismissible, non-blocking success feedback | FEEDBACK_ERRORS §8.2 | screenshot review | G1 | 🔀 |
| UI-FBK-09 | Empty states with a next action | FEEDBACK_ERRORS §3.3 | panel checklist | G1 | 🔀 |
| UI-FBK-10 | Diagnostics bundle export | FEEDBACK_ERRORS §9 | bundle inspection | G3 | 📅 |

## 7. Register: DAT — Data Integrity and Project Lifecycle

| ID | Requirement | Contract | Verification | Gate | Status |
|---|---|---|---|---|---|
| UI-DAT-01 | Project home with new/open/recent/sample | DATA_INTEGRITY §2 | first-run e2e | G1 | 🔀 |
| UI-DAT-02 | Explicit save and save-as, retry on failure | DATA_INTEGRITY §3 | save round-trip e2e | G1 | 🔀 |
| UI-DAT-03 | Dirty mark and last-saved time | DATA_INTEGRITY §3.2 | state e2e | G1 | 🔀 |
| UI-DAT-04 | Autosave ≤60 s, non-blocking | DATA_INTEGRITY §4 | autosave trace | G2 | 📅 |
| UI-DAT-05 | Crash recovery ≤5 min loss | DATA_INTEGRITY §5 | kill/restart drill | G2 | 📅 |
| UI-DAT-06 | Unsaved-changes interception | DATA_INTEGRITY §3.3 | three-branch e2e | G1 | 🔀 |
| UI-DAT-07 | Editable, persisted project metadata | DATA_INTEGRITY §2.3 | reopen assertion | G2 | 📅 |
| UI-DAT-08 | External change detection and read-only | DATA_INTEGRITY §6 | mtime conflict test | G3 | 📅 |
| UI-DAT-09 | Missing dependency report on open | DATA_INTEGRITY §2.4 | fixture project test | G3 | 📅 |
| UI-DAT-10 | Destructive-operation snapshots | DATA_INTEGRITY §7 | snapshot restore test | G3 | 📅 |

## 8. Register: A11Y — Accessibility

| ID | Requirement | Contract | Verification | Gate | Status |
|---|---|---|---|---|---|
| UI-A11Y-01 | WCAG 2.2 AA on critical paths | A11Y §2 | axe suite | G2 | 📅 |
| UI-A11Y-02 | Keyboard-complete commands and help | A11Y §3 | keyboard-only walkthrough | G2 | 📅 |
| UI-A11Y-03 | Visible, ordered, trapped focus | A11Y §4 | focus audit | G2 | 📅 |
| UI-A11Y-04 | Roles and ARIA for trees/tables/tabs | A11Y §5 | accessibility tree snapshot | G2 | 📅 |
| UI-A11Y-05 | Accessible names; viewport text alternative | A11Y §6 | axe + manual review | G2 | 📅 |
| UI-A11Y-06 | Reduced-motion support | A11Y §7 | media emulation test | G3 | 📅 |
| UI-A11Y-07 | High-contrast theme meets AA | A11Y §8 | contrast audit | G3 | 📅 |
| UI-A11Y-08 | Live announcements for results | A11Y §9 | screen-reader script | G3 | 📅 |

## 9. Register: I18N — Internationalization

| ID | Requirement | Contract | Verification | Gate | Status |
|---|---|---|---|---|---|
| UI-I18N-01 | Zero hardcoded user strings | I18N_L10N §2 | string scan | G2 | 📅 |
| UI-I18N-02 | zh-CN default, en-US fallback, live switch | I18N_L10N §3 | locale e2e | G2 | 📅 |
| UI-I18N-03 | ICU MessageFormat for plurals | I18N_L10N §4 | ICU unit tests | G2 | 📅 |
| UI-I18N-04 | Locale-aware numbers/dates/units | I18N_L10N §5 | format fixtures | G2 | 📅 |
| UI-I18N-05 | Pseudo-localization passes | I18N_L10N §6 | pseudo-locale screenshots | G3 | 📅 |
| UI-I18N-06 | Key convention and unused-key detection | I18N_L10N §7 | key lint | G3 | 📅 |
| UI-I18N-07 | RTL structural readiness | I18N_L10N §8 | logical-property scan | G3 | 📅 |

## 10. Register: PRF — Performance

| ID | Requirement | Contract | Verification | Gate | Status |
|---|---|---|---|---|---|
| UI-PRF-01 | First paint and ready budgets | PERFORMANCE §3 | perf harness | G2 | 📅 |
| UI-PRF-02 | Interaction latency budgets | PERFORMANCE §3.2 | interaction trace | G2 | 📅 |
| UI-PRF-03 | Frame-rate budgets at object counts | PERFORMANCE §4 | frame-time harness | G2 | 📅 |
| UI-PRF-04 | Idle CPU budget | PERFORMANCE §4.3 | idle sample | G2 | 📅 |
| UI-PRF-05 | Memory ceiling and soak | PERFORMANCE §5 | soak test | G3 | 📅 |
| UI-PRF-06 | Bundle budget and splitting | PERFORMANCE §6 | bundle report | G2 | 📅 |
| UI-PRF-07 | Tree scroll at 5000 nodes | PERFORMANCE §7 | scroll trace | G3 | 📅 |
| UI-PRF-08 | Regression gate ±10% | PERFORMANCE §8 | CI job | G3 | 📅 |

## 11. Register: CMP — Compatibility

| ID | Requirement | Contract | Verification | Gate | Status |
|---|---|---|---|---|---|
| UI-CMP-01 | Supported browser matrix | COMPATIBILITY §2 | matrix run | G3 | 📅 |
| UI-CMP-02 | WebGL2 detection and blocking | COMPATIBILITY §3 | forced-failure test | G3 | 📅 |
| UI-CMP-03 | DPI and zoom matrix | COMPATIBILITY §4 | screenshot matrix | G3 | 📅 |
| UI-CMP-04 | 3D mouse support or explicit exclusion | COMPATIBILITY §5 | decision record | G3 | 📅 |
| UI-CMP-05 | Usable from 1280×720 | COMPATIBILITY §4.2 | minimum-size walkthrough | G2 | 📅 |
| UI-CMP-06 | Offline/weak-network degradation | COMPATIBILITY §6 | offline banner test | G3 | 📅 |

## 12. Register: SEC — Security and Privacy

| ID | Requirement | Contract | Verification | Gate | Status |
|---|---|---|---|---|---|
| UI-SEC-01 | Local-only disclosure when unauthenticated | SECURITY_PRIVACY §2 | UI copy review | G3 | 📅 |
| UI-SEC-02 | Permission-gated commands, hidden not erroring | SECURITY_PRIVACY §3 | role matrix test | G3 | 📅 |
| UI-SEC-03 | Project open restricted to allowed roots | SECURITY_PRIVACY §4 | traversal test | G3 | 📅 |
| UI-SEC-04 | CSP and no untrusted HTML | SECURITY_PRIVACY §5 | header + code scan | G3 | 📅 |
| UI-SEC-05 | Opt-in telemetry, no PII | SECURITY_PRIVACY §6 | consent + payload audit | G3 | 📅 |

## 13. Register: TLM — Telemetry and Supportability

| ID | Requirement | Contract | Verification | Gate | Status |
|---|---|---|---|---|---|
| UI-TLM-01 | Versioned event dictionary | TELEMETRY §2 | dictionary diff | G3 | 📅 |
| UI-TLM-02 | Five-step funnel instrumented | TELEMETRY §3 | session funnel log | G3 | 📅 |
| UI-TLM-03 | Error events with code, no model content | TELEMETRY §4 | payload audit | G3 | 📅 |
| UI-TLM-04 | Performance percentiles | TELEMETRY §5 | metric sample | G3 | 📅 |
| UI-TLM-05 | Exportable local diagnostics without geometry | TELEMETRY §6 | bundle inspection | G3 | 📅 |
| UI-TLM-06 | Shortcut help, contextual help, samples, about | TELEMETRY §7 | help surface walkthrough | G3 | 📅 |

## 14. Verification Inventory

**Only ✅ rows exist today.** Rows marked 📅 are planned; a requirement whose only verification is a 📅 artifact is **not verifiable** and cannot pass its gate. Several rows in §3–§13 name planned artifacts (for example "screenshot set `shell`"); those do not count as evidence yet.

| Artifact | Type | Status | Covers |
|---|---|---|---|
| `web/src/__tests__/stateMachine.test.ts` | unit | ✅ | UI-INT-02 |
| `web/src/__tests__/genesisForm.test.ts` | unit | ✅ | UI-INT-02, UI-INT-10 |
| `web/src/__tests__/previewGeometry.test.ts` | unit | ✅ | UI-INT-02 |
| `web/src/__tests__/storeyView.test.ts` | unit | ✅ | Storey filtering (`meshVisibleOnStorey`) — no dedicated requirement yet (Q4) |
| `web/src/__tests__/datumGrids.test.ts` | unit | ✅ | Datum-grid rendering — no dedicated requirement yet (Q4) |
| `scripts/check_docs.py` | scan | ✅ | Documentation structure and bilingual parity |
| `scripts/check_ui_contracts.py` | scan | ✅ | ID/keymap/error-code structure; debt counts (`--strict` = zero-debt gate) |
| `scripts/gen_keymap.py --check` | scan | ✅ | UI-INT-03 (table consistency only; no code cross-check yet) |
| `web/tests/e2e/*.spec.ts` | e2e | 📅 | SHL, INT, FBK, DAT, I18N |
| `web/tests/visual/*.spec.ts` | visual | 📅 | DS, SHL, A11Y |
| `web/tests/a11y/*.spec.ts` | a11y | 📅 | A11Y-01/04/05/06 |
| `web/tests/perf/*.spec.ts` | perf | 📅 | PRF-01…08 |
| Screenshot set (`docs/img/ui/`) | visual | 📅 | SHL-02/03/05, DS-02/12 |

**Rule:** a new requirement must reference an existing ✅ artifact, or add a 📅 artifact with an owner and a gate. Unverified requirements block the gate.

## 15. Gate Coverage

| Gate | Requirements | Character |
|---|---|---|
| G0 | 0 (contracts only) | PRD, IA, DS, INTERACTION approved |
| G1 | 25 | Shell usable, no white screen, no native dialogs, save works |
| G2 | 37 | Themes, core i18n, critical-path a11y, performance, full feedback |
| G3 | 34 | Completeness tiers, compatibility, security, telemetry, supportability |

Counts are derived from the Gate column above; re-count on every change.

## 16. Status Summary

| Status | Count | Meaning |
|---|---|---|
| ✅ | 1 | Implemented and verified against the cited artifact |
| 🔀 | 17 | Partially implemented; the contract states the remainder |
| 📅 | 78 | Planned; no implementation |
| 💭 | 0 | Proposed only |

**Interpretation:** the frontend is at G0 with a usable prototype. The distance to G3 is 78 pure-planned requirements plus 17 partially implemented ones — not a matter of styling.

## 17. Change Log

| Date | Change | By |
|---|---|---|
| 2026-10-29 | Register created with 96 requirements across 11 areas | Frontend docs baseline |

## 18. Non-Goals

1. Tracking backend or IFC requirements.
2. Estimating effort per requirement.
3. Replacing the PRD as the normative description of a requirement.

## 19. Open Questions

| # | Question | Owner |
|---|---|---|
| Q1 | Should partial requirements be split so a `🔀` row carries its own sub-ID? | Product / Frontend |
| Q2 | Should performance requirements be duplicated in CI config as the authoritative threshold? | Frontend |
| Q3 | Who signs off status changes: product or the implementing engineer? | Product |
| Q4 | Storey view, section cut, and measurement persistence have journeys but no requirement ID. Add `UI-INT-13…15`, or treat them as viewport tools outside the register? | Product / Frontend |
| Q5 | Should the string scan detect hardcoded English as well as CJK? Today only CJK is detected (`I18N_L10N` §2). | Frontend |
