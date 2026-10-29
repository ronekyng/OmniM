---
title: "OmniM Frontend Personas & Journeys"
status: design
last_updated: 2026-10-29
applies_to: "web/"
evidence: "n/a (product requirement)"
---
# OmniM Frontend Personas & Journeys

> **Document role.** This document turns the positioning in [VISION_SCOPE](VISION_SCOPE.md) into concrete user journeys. Every journey step names the feedback the user expects; every expectation maps to a requirement ID in [PRD_FRONTEND](PRD_FRONTEND.md).
>
> **Dual-track maintenance obligation**: `PERSONAS_JOURNEYS.zh-CN.md` is a maintained translation. Keep section numbering and requirement IDs aligned in the same change.
>
> **Honesty rule.** Journeys describe the **target** experience. Where the current build differs, the difference is recorded in [CODE_DEVIATIONS §7](../status/CODE_DEVIATIONS.md).

## 1. Personas

### 1.1 P1 — Discipline Modeler

| Attribute | Value |
|---|---|
| Context | Sits at a desktop workstation, one project at a time, long sessions |
| Skill | Knows the discipline; moderate software tolerance, low tolerance for data loss |
| Primary tasks | Create walls/slabs/equipment, place catalog components, adjust properties, save |
| Pain today | Creation is buried in toolbar popovers; object names are timestamps; no explicit save control |
| Success signal | "I trust this file. I can close it and come back." |

### 1.2 P2 — Discipline Lead / Reviewer

| Attribute | Value |
|---|---|
| Context | Opens a colleague's model, reads rather than edits |
| Skill | High discipline knowledge, low patience for hunting through panels |
| Primary tasks | Select and inspect, read property sets, measure distance/area, cut sections, verify storey |
| Pain today | The properties panel opens as a centered floating overlay; measurement results are transient |
| Success signal | "I can answer a model question in under a minute without the console." |

### 1.3 P3 — Catalog & Standards Administrator

| Attribute | Value |
|---|---|
| Context | Maintains component definitions, series decision trees, units, and language packs |
| Skill | Deep standards knowledge; comfortable with forms and tables |
| Primary tasks | Edit `ComponentDef` parameters and connectors, edit series trees, import/export `.omnimlib`/`.omnimseries` |
| Pain today | Manager pages are visually unrelated to the workspace; no validation summary |
| Success signal | "A wrong definition is caught before it reaches the model." |

## 2. Journey J1 — First Run to First Saved Model

| Step | User action | Expected feedback | Requirement |
|---|---|---|---|
| J1.1 | Launch the application | A project home, not an unknown preloaded model | UI-DAT-01 |
| J1.2 | Choose **New project** or a sample | Name, unit system, and location requested; template preview | UI-DAT-01, UI-DAT-07 |
| J1.3 | Enter the workspace | Shell loads with empty-state guidance in the viewport | UI-SHL-02, UI-FBK-09 |
| J1.4 | Pick **Wall** from the ribbon | Ribbon shows the active tool; left panel shows wall parameters | UI-SHL-07 |
| J1.5 | Pick two points | Preview follows the cursor; snapping indicator visible | UI-INT-02 |
| J1.6 | Press Enter | Object created with a readable name (`Wall-001`), selected | UI-INT-09 |
| J1.7 | Press Ctrl+S | Status bar shows saving, then "saved at HH:MM:SS"; dirty mark clears | UI-DAT-02, UI-DAT-03, UI-SHL-08 |
| J1.8 | Reopen the file | Identical object count and geometry | UI-DAT-02 |

**Failure paths:** server save fails → stays dirty with a retry action (UI-DAT-02); creation rejected → inline reason in the panel, not a browser alert (UI-FBK-04).

## 3. Journey J2 — Catalog Component Placement

| Step | User action | Expected feedback | Requirement |
|---|---|---|---|
| J2.1 | Open the component library | Tab in the right dock, not a modal takeover | UI-SHL-02 |
| J2.2 | Filter by discipline and series | Count and empty state update immediately | UI-FBK-01 |
| J2.3 | Answer the series selector questions | Remaining valid components narrow; invalid options disabled with reason | UI-FBK-04 |
| J2.4 | Adjust parameters | Numeric fields show units and bounds | UI-INT-10 |
| J2.5 | Place in viewport | Ghost preview uses real catalog mesh | UI-INT-02 |
| J2.6 | Confirm | Object appears with `catalog_ref`; properties show series provenance | UI-SHL-03 |

## 4. Journey J3 — Interruption, Failure and Recovery

| Step | User action | Expected feedback | Requirement |
|---|---|---|---|
| J3.1 | Import a large IFC file | Progress with cancel; UI stays responsive | UI-FBK-03 |
| J3.2 | Import fails halfway | Clear error code, cause, and recovery action; no partial silent state | UI-FBK-04, UI-FBK-10 |
| J3.3 | Browser or server crashes | On restart, a recovery prompt offers the last autosave | UI-DAT-05 |
| J3.4 | User declines recovery | Explicit confirmation before discarding | UI-INT-07, UI-DAT-06 |
| J3.5 | External process modifies the project file | UI switches to read-only and explains the conflict | UI-DAT-08 |
| J3.6 | User deletes a parent object | Impact preview lists affected children before deletion | UI-INT-07, UI-DAT-10 |

## 5. Journey J4 — Review, Measure and Section

| Step | User action | Expected feedback | Requirement |
|---|---|---|---|
| J4.1 | Select an object | Right dock shows properties immediately; viewport highlights | UI-SHL-03 |
| J4.2 | Switch to the storey view | Only the selected storey is visible; control shows the active storey | — (gap, see Q4) |
| J4.3 | Enable a section cut | Section plane handle is grabbable; clipping is immediate | — (gap, see Q4) |
| J4.4 | Measure a distance | Measurement persists in the project and is listed | — (gap, see Q4) |
| J4.5 | Read a disabled parameter | Tooltip explains why it is disabled | UI-FBK-04 |

## 6. Journey J5 — Reopen and Handover

| Step | User action | Expected feedback | Requirement |
|---|---|---|---|
| J5.1 | Open the project home | Recent files with path, timestamp, and size | UI-DAT-01 |
| J5.2 | Open a file created by someone else | Missing catalog/series dependencies are listed explicitly | UI-DAT-09 |
| J5.3 | Review unsaved changes from a previous session | Recovery or discard choice is explicit | UI-DAT-05 |
| J5.4 | Export for exchange | Format chosen from a dialog that states known limits | UI-FBK-04 |
| J5.5 | Hand over | Project metadata and unit system are visible and correct | UI-DAT-07 |

## 7. Journey-to-Requirement Map

| Journey | Primary requirement groups |
|---|---|
| J1 | R1 Shell, R3 Interaction, R5 Data integrity, R4 Feedback |
| J2 | R3 Interaction, R4 Feedback, R7 Internationalization |
| J3 | R4 Feedback, R5 Data integrity |
| J4 | R1 Shell, R3 Interaction, R6 Accessibility |
| J5 | R5 Data integrity, R8 Performance, R9 Compatibility |

## 8. Non-Goals

1. Screen-by-screen UI specifications (they live in `specification/ui/`).
2. Marketing personas or buyer journeys.
3. Quantitative research design (sampling, statistics).

## 9. Open Questions

| # | Question | Owner |
|---|---|---|
| Q1 | Does the reviewer persona need a read-only mode that hides editing commands entirely? | Product |
| Q2 | Are measurements part of the saved project or a session artifact for reviewers? | Product / Data model |
| Q3 | Should handover include a generated dependency report (catalog/series/IFC versions)? | Product |
| Q4 | Storey view, section cut, and measurement persistence have journeys but no requirement IDs. Add them as `UI-INT-13…15`, or treat them as viewport tools outside the register? | Product / Frontend |
