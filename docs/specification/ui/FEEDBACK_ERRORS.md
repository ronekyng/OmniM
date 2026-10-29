---
title: "OmniM UI Feedback and Error Contract"
status: design
last_updated: 2026-10-29
applies_to: "web/"
evidence: "n/a (design contract)"
---
# OmniM UI Feedback and Error Contract

> **Contract scope.** This document fixes panel states, long-operation feedback, the error model, error boundaries, toasts, and the diagnostics bundle. It is the normative source for requirements `UI-FBK-01` … `UI-FBK-10`.
>
> **Dual-track maintenance obligation**: `FEEDBACK_ERRORS.zh-CN.md` is a maintained translation. Keep section numbering, error-code rules, and requirement IDs aligned in the same change.
>
> **Status.** Design contract. Native dialogs are gone (`ConfirmDialog` + `notify`), and error boundaries wrap the app, the workspace and every panel slot; loading/empty/error completeness and a throwing-panel test remain open. See [CODE_DEVIATIONS §7](../../status/CODE_DEVIATIONS.md).

## 1. Purpose

A commercial application never leaves the user guessing. Every wait, every failure, and every success must be visible, attributable, and actionable.

## 2. Feedback Layers

| Layer | Latency | Mechanism | Interrupts work? |
|---|---|---|---|
| Instant | < 100 ms | Hover, pressed, focus ring | No |
| Transient | 100 ms – 1 s | Inline spinner in the control, optimistic state | No |
| Progress | > 1 s | Progress bar with phase label and cancel | No |
| Result | On completion | Toast (success/info) or banner (warning) | No |
| Blocking | Cannot continue | Dialog with recovery actions | Yes |
| Ambient | Persistent condition | Status bar slot + banner | No |

**Rule:** the same event must not produce two competing messages (for example a toast and a banner for one save).

## 3. Panel States

### 3.1 Loading

- First load of a panel: skeleton matching the final layout.
- Subsequent loads: keep stale content with a subtle top progress line (no layout shift).
- Never replace the whole workspace with a spinner.

### 3.2 Skeleton timing

| Delay | Behavior |
|---|---|
| < 400 ms | Show nothing; the result appears directly |
| 400 ms – 3 s | Skeleton |
| > 3 s | Skeleton + phase label + cancel when safe |

Requirement: UI-FBK-02.

### 3.3 Empty

Every empty state answers three questions: why is it empty, what can I do, and how do I learn more.

| Panel | Primary action |
|---|---|
| Object tree | Create object, or open sample project |
| Properties (no selection) | "Select an object in the viewport or tree" |
| Library (no results) | Clear filters |
| Grids | Add grid |
| Console | Show example command |
| Recent projects | New project |

Requirement: UI-FBK-01, UI-FBK-09.

### 3.4 Error

Panel-level errors render in place, keep the panel chrome, and offer Retry plus Details. They must never blank the shell. Requirement: UI-FBK-01.

## 4. Long Operations

| Operation | Progress | Cancel | Notes |
|---|---|---|---|
| IFC import | Parse, map, rebuild phases | Yes before commit | Rollback on cancel |
| IFC export | Serialize phase | No (fast) | Download when done |
| Rebuild all features | Per-feature count | Yes between features | Partial state is explicit |
| Catalog import | Parse, validate | Yes before commit | No partial library |
| Save as | Write phase | No | Atomic write server-side |

Rules:

1. Progress reports a phase and a count where possible, not a fake percentage.
2. Cancel must leave the model in its pre-operation state or clearly label partial results.
3. The UI stays interactive; the viewport is not blocked by an overlay.

Requirement: UI-FBK-03.

## 5. Error Model

### 5.1 Error code format

```text
OMNI-<AREA>-<NNNN>
  AREA: UI | API | NET | IO | GEO | CAT | IFC | AUTH | SYS
  NNNN: zero-padded sequence, stable forever
```

Examples: `OMNI-NET-0003` (server unreachable), `OMNI-GEO-0012` (degenerate boundary), `OMNI-IFC-0007` (unsupported entity).

Rules: codes are never reused; a retired code stays reserved; the code is the join key between UI text, the error catalogue, and telemetry.

### 5.2 Message structure

Every user-facing error has four parts:

1. **Title** — what failed, in the user's terms.
2. **Cause** — the most likely reason, one sentence.
3. **Recovery** — the next action (button or instruction).
4. **Details** — code, HTTP status, request id; expandable, copyable.

Example:

> **Couldn't save the project.** The server rejected the write (`OMNI-IO-0004`, HTTP 500). Your changes are still in memory. [Retry] [Save as…] [Copy details]

### 5.3 Severity

| Severity | Meaning | Surface | Telemetry |
|---|---|---|---|
| Blocking | Work cannot continue | Dialog | error event |
| Degraded | Feature unavailable, app usable | Banner | warn event |
| Recoverable | Operation failed, retry possible | Inline + toast (error) | error event |
| Informational | Expected rejection (validation) | Inline field error | none |
| Unexpected | Unhandled exception | Error boundary | error event + stack |

### 5.4 Catalogue

`docs/reference/ERROR_CODES.md` is the authored catalogue. `scripts/check_ui_contracts.py --only errors` fails if source code uses an `OMNI-*` code that is absent from the catalogue.

Requirement: UI-FBK-04, UI-TLM-03.

## 6. Native Dialog Ban

The following are prohibited in application code:

- `window.alert`
- `window.confirm`
- `window.prompt`

Replacements: `ConfirmDialog`, `PromptDialog`, and the inline validation pattern.

`scripts/check_ui_contracts.py --dialogs` reports every call site; `--strict --dialogs` fails the build while any remain.

Requirement: UI-FBK-05.

## 7. Error Boundaries

| Boundary | Scope | Fallback |
|---|---|---|
| Root | Whole app | Full-page error with reload and diagnostics export |
| Route | Workspace, each editor | Route shell with breadcrumb and "back to workspace" |
| Panel | Each registered panel | Panel frame with retry; other panels unaffected |
| Viewport | 3D canvas | Canvas placeholder with renderer error and reload scene |

Rules:

1. Boundaries log to the diagnostics buffer with a correlation id.
2. The fallback never hides the navigation chrome.
3. Repeated failures of the same panel within one session disable auto-retry and offer diagnostics.

Requirement: UI-FBK-06.

## 8. Toasts

### 8.1 Types

| Type | Lifetime | Dismissible | Example |
|---|---|---|---|
| success | 4 s | Yes | "Wall-003 created" |
| info | 6 s | Yes | "Layout reset" |
| warning | 8 s or until dismissed | Yes | "Snapping disabled for this tool" |
| error | Until dismissed | Yes | "Save failed — changes kept in memory" |

### 8.2 Placement

| Rule | Value |
|---|---|
| Anchor | Bottom-right of the viewport, above the status bar |
| Stack | Max 3 visible; older ones collapse into a counter |
| Width | 320–420 px |
| Safe area | Must not cover the view cube, tool strip, or section handle |
| Motion | 150 ms slide; disabled under `prefers-reduced-motion` |

Requirement: UI-FBK-07, UI-FBK-08.

### 8.3 Deduplication

Identical messages within 2 s collapse into one with a count. Errors are never collapsed by type alone — each object-level failure names its object.

## 9. Diagnostics Bundle

| Contents | Included | Excluded |
|---|---|---|
| App version and build hash | Yes | — |
| Server version and route table hash | Yes | — |
| Browser, GPU, WebGL capabilities | Yes | — |
| Recent 500 log lines | Yes | File contents |
| Recent 100 commands (ids, timestamps, result) | Yes | Geometry, names of user objects |
| Project metadata | Name and unit system only | Object data |
| Screenshot | Optional, user-confirmed | Automatic capture |

Format: a single `.zip` with `manifest.json`, `logs.txt`, `commands.json`, `environment.json`. Requirement: UI-FBK-10, UI-TLM-05.

## 10. Non-Goals

1. Automatic crash upload without consent.
2. In-app log browsing for end users beyond the diagnostics bundle.
3. Localized stack traces.

## 11. Open Questions

| # | Question | Owner | Needed by |
|---|---|---|---|
| Q1 | Should error codes be localized in their numeric part? No — confirm the rule. | Product | G1 |
| Q2 | Is a 3 s threshold for "cancel" acceptable for IFC import on large files? | Frontend | G2 |
| Q3 | Should diagnostics include the last screenshot automatically in developer builds only? | Frontend | G3 |
