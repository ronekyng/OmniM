---
title: "OmniM Frontend Definition of Done"
status: design
last_updated: 2026-10-29
applies_to: "web/"
evidence: "n/a (process contract)"
---
# OmniM Frontend Definition of Done

> **Document role.** This is the process layer: what "done" means for a feature, a panel, a document, and a release. It turns the gates in [RELEASE_READINESS](../status/RELEASE_READINESS.md) into checklists an engineer can execute.
>
> **Dual-track maintenance obligation**: `DEFINITION_OF_DONE.zh-CN.md` is a maintained translation. Checklist items and gate definitions must match.
>
> **Rule.** "Done" is binary. An unchecked MUST item means not done, regardless of demo quality.

## 1. Purpose

Teams ship demos when "done" means "it worked once on my machine". This document removes that ambiguity for the frontend.

## 2. Functional DoD

A feature is functionally done when:

- [ ] The requirement ID is registered and linked to a contract.
- [ ] Happy path works end to end against the real server, not a mock.
- [ ] Validation rejects invalid input with a specific, localized message.
- [ ] Cancellation leaves no partial state.
- [ ] Undo restores the prior state, or the UI states that the action is not undoable.
- [ ] Empty, loading, and error states are implemented.
- [ ] Permissions/availability rules are applied server-side and reflected in the UI.
- [ ] The feature is keyboard operable.

## 3. UI DoD

A UI surface is done when:

- [ ] It uses semantic tokens only; no hardcoded color, spacing, radius, or shadow.
- [ ] It works in dark and light themes, and in high contrast for accessibility scope.
- [ ] All strings come from the i18n catalogue.
- [ ] Every interactive element has default/hover/active/selected/disabled/focus states.
- [ ] Focus order is logical and visible; dialogs trap and restore focus.
- [ ] Icon-only controls have accessible names.
- [ ] The layout holds at 1280×720 and 1920×1080, at 100% and 150% DPI.
- [ ] No native browser dialog is used.
- [ ] Screenshots exist for both themes.

## 4. Documentation DoD

A document is done when:

- [ ] Front matter has `title`, `status`, `last_updated`, `applies_to`, `evidence`.
- [ ] The English and Chinese files have identical heading counts and numbering.
- [ ] Every relative link resolves (`scripts/check_docs.py`).
- [ ] Requirement IDs match [REQUIREMENTS_REGISTER](../product/REQUIREMENTS_REGISTER.md).
- [ ] Claims about current code cite a file or a measured count.
- [ ] Non-goals and open questions are present.
- [ ] A run of `check_docs.py` passes.

## 5. Test DoD

A change is test-complete when:

- [ ] A unit test covers pure logic (state machine, naming, units, formatting).
- [ ] An e2e test covers the user-visible path.
- [ ] A visual regression entry exists for layout-affecting changes.
- [ ] Accessibility checks pass on the changed surface.
- [ ] Performance did not regress beyond the tolerance in [PERFORMANCE](../specification/ui/PERFORMANCE.md) §8.
- [ ] The new code path appears in the requirement register's verification column.

## 6. Release DoD

A release candidate is done when:

- [ ] All **MUST** requirements assigned to the target gate are `✅`; SHOULD slips follow §8.
- [ ] [RELEASE_READINESS](../status/RELEASE_READINESS.md) shows no blocking issue for that gate.
- [ ] `check_docs.py`, `check_ui_contracts.py`, type check, unit, e2e, visual, a11y, and performance jobs pass.
- [ ] The maturity score meets the gate target.
- [ ] CHANGELOG has a user-facing entry.
- [ ] Known issues are documented with workarounds.
- [ ] Diagnostics export works and contains no model data.
- [ ] Rollback or recovery procedure is documented.

## 7. Gate Definitions

### 7.1 G0 — Contract ready

Requirements, contracts, and non-goals are reviewed and signed off. No production code is required, but no code may start before this gate unless it is a spike explicitly marked throwaway.

### 7.2 G1 — Alpha

The main journey (open/create → model → save → reopen) works. There is no white screen, no native dialog, no silent data loss, and no timestamp naming. Internal users can complete a real task.

### 7.3 G2 — Beta

The product looks and behaves like commercial software: themes, localized, accessible on critical paths, performant within budget, and explicit about every failure. External pilot users can work without hand-holding.

### 7.4 G3 — GA

The product is supportable: compatibility declared, security disclosed, telemetry consented, diagnostics exportable, help available, and conformance artifacts written.

## 8. Exception Process

| Situation | Process |
|---|---|
| MUST requirement cannot be met | Written decision with scope, owner, expiry, and user impact; the requirement stays `MUST` and the gate slips |
| SHOULD requirement slips | Recorded in the release note; may slip one minor version |
| Contract change | Update contract, register, and changelog in one PR |
| Emergency fix bypassing a checklist item | Post-merge issue with a date, approved by the gate owner |

Exceptions never change a requirement's priority silently.

## 9. Non-Goals

1. A single universal checklist across backend and frontend.
2. Mandatory pair programming or review counts.
3. Process metrics for their own sake.

## 10. Open Questions

| # | Question | Owner | Needed by |
|---|---|---|---|
| Q1 | Should G1 require an external pilot user, or is internal use enough? | Product | G1 |
| Q2 | Are visual regression diffs reviewed by design or by the implementer? | Design | G2 |
| Q3 | Who owns the ACR/VPAT artifact? | Product / A11y | G3 |
