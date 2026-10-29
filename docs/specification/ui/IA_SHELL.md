---
title: "OmniM UI Shell and Information Architecture"
status: design
last_updated: 2026-10-29
applies_to: "web/"
evidence: "n/a (design contract)"
---
# OmniM UI Shell and Information Architecture

> **Contract scope.** This document fixes the application shell: levels, regions, docking, persistence, status bar, responsive collapse, and the command palette. It is the normative source for requirements `UI-SHL-01` … `UI-SHL-12`.
>
> **Dual-track maintenance obligation**: `IA_SHELL.zh-CN.md` is a maintained translation. Keep section numbering, tables, and requirement IDs aligned in the same change.
>
> **Status.** Design contract. Current implementation is described in [WEB_ARCHITECTURE](../../architecture/WEB_ARCHITECTURE.md); gaps are tracked in [FRONTEND_MATURITY](../../status/FRONTEND_MATURITY.md).

## 1. Purpose and Scope

The shell is the product's first impression and its permanent operating frame. It must behave like a docked engineering workspace: stable regions, predictable resizing, persistent layout, and no modal takeover of the viewport.

Out of scope: the visual language (see [DESIGN_SYSTEM](DESIGN_SYSTEM.md)), command semantics (see [INTERACTION](INTERACTION.md)), and project lifecycle (see [DATA_INTEGRITY](DATA_INTEGRITY.md)).

## 2. Application Levels

### 2.1 Project layer

The entry surface. Lists recent projects, offers new/open/sample. It has no viewport and no modeling commands.

| Rule | Value |
|---|---|
| Route | `/projects` |
| Default landing | Yes, when no project is open |
| Exit | Opening or creating a project navigates to the workspace |
| Requirement | UI-SHL-01, UI-DAT-01 |

### 2.2 Workspace layer

The main modeling surface. Owns the seven regions, the viewport, and the project session.

| Rule | Value |
|---|---|
| Route | `/` (workspace root) |
| Title | Project name + dirty mark |
| Breadcrumb | `Project / Discipline / Active tool` |
| Requirement | UI-SHL-01, UI-SHL-10 |

### 2.3 Editor layer

Focused authoring surfaces that temporarily replace the workspace viewport but keep the shell chrome consistent.

| Editor | Route | Chrome |
|---|---|---|
| Component editor | `/component-editor` | Menu bar + editor toolbar + feature panel + 2D/3D canvas |
| Library manager | `/library-manager` | Menu bar + tree + form + preview |
| Series manager | `/series-manager` | Menu bar + decision tree + detail form |
| Requirement | UI-SHL-01, UI-SHL-10 |

**Rule:** every editor returns to the workspace through a breadcrumb link and `Esc` is *not* used for navigation (it cancels the active tool first).

## 3. Workspace Regions

### 3.1 Seven regions

```text
┌──────────────────────────────────────────────────────────────┐
│ 1 Menu bar (32 px)                                           │
├──────────────────────────────────────────────────────────────┤
│ 2 Ribbon (72 px, collapsible)                                │
├────────────┬────────────────────────────────┬────────────────┤
│ 3 Left     │ 4 Viewport                     │ 5 Right dock   │
│   dock     │                                │                │
│ 240–400 px │  flexible, minimum 480 px      │ 260–600 px     │
│            ├────────────────────────────────┤                │
│            │ 6 Bottom dock (collapsed/240)  │                │
├────────────┴────────────────────────────────┴────────────────┤
│ 7 Status bar (24 px)                                         │
└──────────────────────────────────────────────────────────────┘
```

| # | Region | Required | Default size | Resizable |
|---|---|---|---|---|
| 1 | Menu bar | Yes | 32 px | No |
| 2 | Ribbon | Yes | 72 px | Collapsible to 0 |
| 3 | Left dock | Yes | 280 px | 240–400 px |
| 4 | Viewport | Yes | flexible | Min 480×360 px |
| 5 | Right dock | Yes | 340 px | 260–600 px |
| 6 | Bottom dock | Yes | collapsed | 0 or 160 px–40% of viewport |
| 7 | Status bar | Yes | 24 px | No |

**Invariant:** at 1440×900 with all docks at default width, the viewport must be at least 720×520 px. If the computed size is smaller, the docks collapse before the viewport shrinks below its minimum.

### 3.2 Menu bar

| Menu | Contents |
|---|---|
| File | New, Open, Recent, Save, Save As, Import, Export, Close |
| Edit | Undo, Redo, Cut/Copy/Paste (reserved), Delete, Select All |
| View | Visual style, Storey view, Section, Grid, Snap, Panels, Reset layout |
| Model | Discipline switch, Create, Modify, Measure |
| Tools | Console, AI assistant, Diagnostics, Settings |
| Help | Shortcuts, Documentation, Sample projects, About |

Menu items must mirror command IDs from the keyboard registry; a menu item without a command is a defect.

### 3.3 Ribbon

The ribbon is **contextual** and **domain-scoped**. It shows only commands valid for the active discipline and active selection type.

| Zone | Content |
|---|---|
| Left | Discipline tabs: Building, Civil, Plant, Mechanical |
| Center | Command groups for the active discipline (Create, Modify, Datum, Measure, Exchange) |
| Right | Active tool indicator and cancel affordance |

Rules:

1. Switching discipline changes the command set and must not mutate the model.
2. Unavailable commands are **disabled with a reason** in the tooltip, not hidden, except commands invalid for the current selection type (hidden).
3. The active tool is always visible while a picking session is running.
4. `Esc` cancels the active tool before it closes any panel.

Requirement: UI-SHL-07.

### 3.4 Left dock

| Tab | Content | Default |
|---|---|---|
| Objects | Object tree with visibility and selection | Active |
| Grids | Datum grids and storeys | Inactive |
| Library | Available components with series filter | Inactive |

Rules: tabs keep their component mounted when hidden only if they hold transient state (object tree scroll position, library filters); heavy 3D previews must unmount.

### 3.5 Viewport

Owns the 3D scene, view cube, vertical tool strip, section handle, measurement overlay, and the shortcut hint. It must never be covered by the properties panel in the default layout.

### 3.6 Right dock

| Tab | Content | Auto-activation |
|---|---|---|
| Properties | Selected object, catalog reference, parameters | On selection |
| Create | Active creation form | On tool activation |
| Components | Catalog browser | Manual |
| Domain | Discipline quick actions | Manual |

Auto-activation must not steal focus from the viewport: the tab changes, keyboard focus stays in the viewport.

### 3.7 Bottom dock

| Tab | Content | Default |
|---|---|---|
| Console | Omcom JSON console | Active |
| AI | Assistant chat | Inactive |
| Output | Recent operations and errors | Inactive |
| History | Undo stack browser | Inactive |

The bottom dock is collapsed by default to protect viewport height.

### 3.8 Status bar

See §6.

## 4. Docking Model

### 4.1 Dock anatomy

Every dock is: header (tab strip + overflow menu + collapse button), content area, optional footer. Docks are siblings of the viewport, never overlays, except in floating mode.

### 4.2 Sizing rules

| Rule | Value |
|---|---|
| Splitter width | 4 px hit area, 1 px visual |
| Drag step | 1 px, snapped to 2 px |
| Double-click splitter | Restore default width |
| Keyboard resize | Focus splitter, `←`/`→` by 16 px, `Home` resets |
| Persistence | Width stored per dock, per level |
| Requirement | UI-SHL-03, UI-SHL-04, UI-SHL-05 |

### 4.3 Tab behavior

- Tabs are ordered by the registry `order` field, then by user drag order.
- A tab with unsaved form state shows a dot indicator.
- Closing a tab that hosts a running operation is blocked with an explanation.
- `Ctrl/Cmd+W` closes the active tab; `Ctrl/Cmd+Shift+W` closes the dock (if optional).

### 4.4 Floating mode

Floating is an **escape hatch**, not the default.

| Rule | Value |
|---|---|
| Trigger | Drag a tab out of its dock, or "Float" in the tab overflow menu |
| Window | In-app floating panel (not a browser window) |
| Bounds | Must stay within the app viewport; clamp to 32 px margin |
| Persistence | Position and size saved per panel id |
| Return | "Dock" action restores the last dock position |
| Non-goal | Multi-monitor detachable windows |
| Requirement | UI-SHL-09 |

## 5. Layout Persistence

| Rule | Value |
|---|---|
| Storage | `localStorage`, key `omnim.layout.v1` |
| Schema | `{ version, level, docks: { left, right, bottom }, ribbonCollapsed, floating: [] }` |
| Write timing | Debounced 300 ms after a change; flushed on unload |
| Migration | Unknown version → discard and use defaults, never crash |
| Reset | View → Panels → Reset layout, with confirmation |
| Corrupt data | Ignore, log a diagnostic event, restore defaults |
| Requirement | UI-SHL-06 |

**Invariant:** layout state is UI-only. It must never be written into the project file, because layout is per-user, not per-project.

## 6. Status Bar Contract

| Slot | Content | Update trigger |
|---|---|---|
| Coordinates | Cursor world position in metres, 3 decimals | Pointer move (throttled 50 ms) |
| Selection | `n selected` or object type of single selection | Selection change |
| Units | `m` or `mm` context indicator | Focus change |
| Save state | `Saved HH:MM:SS` / `Unsaved changes` / `Saving…` / `Save failed` | Project state machine |
| Frame rate | Rolling 1 s FPS, hidden unless developer mode | Render loop sample |

Rules: the status bar is read-only; clicking a slot may open the related panel but must not mutate state. Requirement: UI-SHL-08.

## 7. Responsive Collapse

| Width | Behavior |
|---|---|
| ≥1600 px | All docks at stored width, bottom dock may be expanded |
| 1280–1599 px | Right dock max 420 px; left dock max 320 px |
| 1024–1279 px | Both side docks collapse to a 44 px icon rail; bottom dock auto-collapses — **degraded, not certified** |
| <1024 px | Unsupported; show a blocking notice with the minimum requirement |

The viewport minimum (480×360) always wins over dock width. Requirement: UI-SHL-11, UI-CMP-05.

## 8. Command Palette

| Rule | Value |
|---|---|
| Shortcut | `Ctrl/Cmd+K` |
| Behavior | Fuzzy search over command registry, grouped by ribbon group |
| Row | Command name, shortcut, disabled reason |
| Execution | `Enter` runs; `Esc` closes and restores focus |
| Coverage | Every enabled ribbon command must be reachable |
| Non-goal | Free-form text commands (that is the Omcom console) |
| Requirement | UI-SHL-12 |

## 9. Non-Goals

1. Multi-window or detachable panels.
2. User-defined dock layouts beyond the four positions.
3. A freeform canvas layout for the shell.
4. Mobile or tablet layout below 1024 px.

## 10. Open Questions

| # | Question | Owner | Needed by |
|---|---|---|---|
| Q1 | Should the ribbon be collapsible to a single row at 1280–1599 px? | Design | G1 |
| Q2 | Should the bottom dock remember per-project tab selection? | Frontend | G2 |
| Q3 | Does the command palette need to expose console commands? | Product | G2 |
