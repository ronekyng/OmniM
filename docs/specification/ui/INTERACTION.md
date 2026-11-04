---
title: "OmniM UI Interaction Contract"
status: design
last_updated: 2026-11-04
applies_to: "web/"
evidence: "n/a (design contract)"
---
# OmniM UI Interaction Contract

> **Contract scope.** This document fixes the command model, selection, picking, keyboard registry, context menus, dialogs, undo, naming, units, and viewport navigation. It is the normative source for requirements `UI-INT-01` … `UI-INT-12`.
>
> **Dual-track maintenance obligation**: `INTERACTION.zh-CN.md` is a maintained translation. Keep section numbering, tables, shortcut keys, and requirement IDs aligned in the same change.
>
> **Status.** Design contract. The picking state machine already exists in `web/src/shared/creation/stateMachine.ts`; the rest is target behavior.

## 1. Purpose

Interaction is where an engineering tool earns trust. The contract makes behavior predictable across disciplines and input devices, and makes keyboard coverage a first-class property rather than an afterthought.

## 2. Command Model

Every user action is a **command** with an ID, a label key, an icon, an optional shortcut, an availability predicate, and an execution handler.

| Field | Rule |
|---|---|
| `id` | Stable, dotted: `wall.create`, `view.fit`, `file.save` |
| `labelKey` | i18n key; no literal text |
| `group` | Ribbon group; drives palette ordering |
| `shortcut` | From the keyboard registry only |
| `enabled(ctx)` | Pure function of app state; must have a reason when false |
| `run(ctx)` | May open a picking session; must not perform hidden side effects |

**Rule:** menus, ribbon buttons, the command palette, and context menus all render from this registry. A command that exists in only one surface is a defect.

## 3. Selection Model

| Gesture | Result | Requirement |
|---|---|---|
| Left click on object | Replace selection with that object | UI-INT-01 |
| Ctrl/Cmd + click | Toggle object in selection | UI-INT-01 |
| Left click on empty space | Clear selection | UI-INT-01 |
| Drag on empty space | Box select; Ctrl/Cmd adds to selection | UI-INT-01 |
| `Esc` | Clear selection (or cancel the active tool first) | UI-INT-01 |
| `Ctrl/Cmd+A` | Select all visible objects | UI-INT-01 |
| `Shift` + tree click | Range select in the object tree | UI-INT-01 |

Rules:

1. Selection is a set of object IDs, ordered by pick time.
2. Multi-selection with mixed types shows a multi-object summary in the properties panel.
3. Hidden objects cannot be selected by viewport picking but remain selectable from the tree.
4. Selection changes are announced to assistive technology (see [A11Y](A11Y.md) §9).

## 4. Picking Session State Machine

### 4.1 States

| State | Meaning | Viewport cursor | Bottom hint |
|---|---|---|---|
| `idle` | No creation in progress | default | shortcut hint |
| `configuring` | Form open, waiting for parameters | default | "Configure and start picking" |
| `picking` | Collecting points | crosshair | "Enter to finish · Backspace to undo point · Esc to cancel" |
| `creating` | Request in flight | wait | "Creating…" |
| `done` | Success, transient | default | object name |
| `error` | Failure, form retained | default | error code + recovery |

### 4.2 Transitions

```text
idle ──activate tool──▶ configuring ──start──▶ picking
                                              │  ▲
                                   add point  │  │ backspace
                                              ▼  │
                                           picking
                                              │ enter / auto-close
                                              ▼
                                          creating
                                           │      │
                                      ok   │      │ error
                                           ▼      ▼
                                         done   error
                                           │      │
                                           └──────┴──▶ idle (cancel / retry)
```

Rules:

1. `configuring` and `picking` may be entered in either order depending on tool type, but `creating` is always terminal for the session.
2. `error` never discards the form; the user can fix parameters and retry.
3. `Esc` in `picking` removes the session; `Esc` in `error` returns to `configuring`.
4. Point removal is limited to the last point (Backspace), never arbitrary deletion.
5. Auto-close: polygon tools close when clicking the first point; linear tools require Enter.

Requirement: UI-INT-02. Verification: `web/src/__tests__/stateMachine.test.ts`.

### 4.3 Keyboard during picking

| Key | Action |
|---|---|
| `Enter` | Finish when the minimum point count is met |
| `Backspace` | Remove the last point |
| `Esc` | Cancel the session |
| `Shift` (hold) | Orthogonal constraint from the last point |
| Arrow keys | Nudge the pending point by one snap step (reserved) |

### 4.4 Preview rules

The preview must use the same geometry kernel parameters as the final object (see the creation pipeline contract). Preview colors come from the viewport palette; the ghost is never more opaque than 35%.

## 5. Keyboard Registry

### 5.1 Registry shape

```ts
type Shortcut = {
  id: string;            // command id
  keys: string;          // "ctrl+s", "shift+f10"
  scope: "global" | "viewport" | "tree" | "dialog";
  when?: (ctx: ShortcutContext) => boolean;
  allowInInput?: boolean; // default false
};
```

One module exports the full array; nothing else registers global listeners.

### 5.2 Generated documentation

[KEYBOARD_MAP](../../reference/KEYBOARD_MAP.md) mirrors the registry. Until the registry lands, `scripts/gen_keymap.py --check` validates internal consistency and en/zh parity; generation from the registry replaces the manual table once the registry exists.

**Delivered (2026-11-04, G2 · INT-03)**: the single registry lives in `web/src/app/commands.ts` (`buildWorkspaceCommands` is the only producer of the command array); `CommandPalette`/`ShortcutHelp` consume it verbatim and the `PaletteCommand` type is owned by the registry module. Window-level keyboard listeners are collapsed to an allow-list (`check_ui_contracts global_keydown` gate: the workspace dispatch in App.tsx reading the registry + the dialog focus trap); the component editor attaches its page keys to the shell container instead of `window`. `scripts/gen_keymap.py --check` keeps the en/zh KEYBOARD_MAP aligned.

Requirement: UI-INT-03.

### 5.3 Focus rules

| Context | Global shortcuts |
|---|---|
| `<input>`, `<textarea>`, `[contenteditable]` | Disabled unless `allowInInput` |
| Open dialog | Only dialog-scoped shortcuts |
| Text editing in the property grid | Disabled except `Esc` and `Enter` |
| Viewport focused | All global shortcuts |

`Esc` is the one exception: it always resolves the innermost cancellable action (dialog → picking → selection).

Requirement: UI-INT-04.

### 5.4 Baseline map

| Shortcut | Command | Scope |
|---|---|---|
| `Ctrl/Cmd+S` | `file.save` | global |
| `Ctrl/Cmd+Shift+S` | `file.saveAs` | global |
| `Ctrl/Cmd+Z` | `edit.undo` | global |
| `Ctrl/Cmd+Shift+Z` | `edit.redo` | global |
| `Ctrl/Cmd+K` | `ui.commandPalette` | global |
| `W` | `modify.translate` | viewport |
| `E` | `modify.rotate` | viewport |
| `F` | `view.fit` | viewport |
| `H` | `view.home` | viewport |
| `G` | `view.toggleGrid` | viewport |
| `O` | `view.toggleOrtho` | viewport |
| `X` | `modify.toggleSpace` | viewport |
| `Delete` | `edit.delete` | global |
| `Backspace` | `picking.undoPoint` | viewport |
| `Esc` | `ui.cancel` | global |
| `Enter` | `picking.finish` | viewport |
| `Shift+F10` | `ui.contextMenu` | global |
| `F1` | `help.shortcuts` | global |

## 6. Context Menus

| Context | Items |
|---|---|
| Viewport, empty space | Paste, Select all, Fit view, Home, Toggle grid, Section |
| Viewport, object selected | Isolate, Hide, Delete, Properties, Measure from here |
| Object tree, object | Rename, Duplicate, Delete, Hide/Show, Zoom to |
| Property row | Copy value, Reset to default, Show definition |
| Console output | Copy, Clear |

Rules:

1. Menus open at the pointer, clamped to the viewport.
2. Keyboard invocation opens at the focused element's bounds.
3. Destructive items are last, separated, and require confirmation when multi-object.
4. Menu items reflect command availability and show the shortcut.

Requirement: UI-INT-05.

**Delivered (2026-11-04, G2 · INT-05)**: the object-tree context menu opens from the keyboard via Shift+F10; `menu/menuitem` semantics, arrow/focus cycling and an Esc close; right-click and keyboard share one entry. Evidence: `web/e2e/a11y.spec.ts` (context-menu keyboard invocation).

## 7. Dialogs and Direct Manipulation

### 7.1 Dialog family

| Type | Use | Buttons |
|---|---|---|
| Confirm | Destructive or irreversible action | Cancel (default focus), Confirm (danger) |
| Prompt | Single value input | Cancel, OK |
| Form | Multi-field settings | Cancel, Apply, OK |
| Progress | Long operation | Cancel when safe |
| Error | Blocking failure | Copy details, Retry, Close |

### 7.2 Focus management

1. Opening a dialog moves focus to the first meaningful control (not the close button).
2. `Tab`/`Shift+Tab` cycle within the dialog.
3. `Esc` closes unless the operation is destructive-in-progress.
4. Closing restores focus to the invoking element.
5. Background content is `aria-hidden` and inert.

Requirement: UI-INT-06.

### 7.3 Destructive confirmation

The dialog must state: what will be deleted, how many objects and children are affected, whether it is undoable, and the recovery path if not.

| Action | Undoable | Confirmation |
|---|---|---|
| Delete object(s) | Yes, via undo | Required when children exist |
| Delete with children | Yes, via undo | Required, lists child count by type |
| Replace project via import | No | Required, offers snapshot |
| Discard unsaved changes | No | Required, three-way choice |

Requirement: UI-INT-07.

### 7.4 Double-click semantics

| Context | Meaning |
|---|---|
| Object tree row | Rename inline |
| Viewport object | Select and open Properties (never edit geometry silently) |
| Viewport empty space | No action |
| Dock splitter | Reset width |
| Tab | No action (avoid accidental close) |
| Property value | Enter edit mode |

**Delivered (2026-11-04, G2 · INT-11)**: a tree row's double-click is an in-place rename (`beginRename`, bubbles intercepted so the meaning stays unique); a dock-divier double-click resets the width (`AppLayout` wired). The remaining rows match the table (a viewport-object double-click lands on the "select and open properties" path, an empty double-click is inert, tabs have none). Evidence: `web/e2e/a11y.spec.ts` (F2 / double-click rename) and the gestures spec.

Requirement: UI-INT-11.

## 8. Undo and Redo

| Rule | Value |
|---|---|
| Coverage | Every mutating endpoint must produce an undo entry |
| Depth | At least 50 steps per session |
| Display | `Edit` menu shows the next undo/redo label |
| Non-undoable | Save, view changes, selection, panel layout |
| History panel | Bottom dock tab lists entries with label, timestamp, object count; clicking restores that state |
| Failure | A failed undo leaves state untouched and reports an error |
| Requirement | UI-INT-08 |

**Invariant:** if an operation is not undoable, the UI must say so before execution, not after.

## 9. Naming and Units

### 9.1 Auto-naming

| Rule | Value |
|---|---|
| Pattern | `<Type>-<nnn>` with three digits, per type, per project |
| Start | `001`; reuse the lowest free number only after deletion |
| Uniqueness | Names are unique per project; duplicates are rejected with a reason |
| Rename | Inline, validated on blur/Enter, undoable |
| Localization | Type prefix uses the localized type name? **No** — prefixes are stable ASCII for exchange |
| Requirement | UI-INT-09 |

### 9.2 Unit display

| Context | Unit | Precision |
|---|---|---|
| Viewport coordinates | m | 3 decimals |
| GenRule inputs | m | 3 decimals |
| Catalog parameters | mm | 0 decimals unless the definition allows fractions |
| Layer editor | mm | 0 decimals |
| Angle | ° | 1 decimal |
| Area | m² | 2 decimals |

Rules: every numeric input shows its unit as a suffix; changing the unit system converts displayed values but does not silently rewrite stored geometry. Requirement: UI-INT-10.

**Delivered (2026-11-04, G2 · INT-10)**: every numeric input in the creation panel carries a `.field-unit` suffix — world-space lengths are `m`, section thickness `mm`, elevations/offsets `m`; catalog parameters echo `{value} {unit}` (mm); the layer editor keeps its existing `mm` labels. Unit-system switching only converts the display (`unit_system` in the project settings is display-only). Evidence: `web/e2e/units.spec.ts`.

## 10. Viewport Navigation

| Input | Orbit | Pan | Zoom |
|---|---|---|---|
| Left drag (no tool) | Select or box select | — | — |
| Middle drag | Orbit | — | — |
| Shift + middle drag | — | Pan | — |
| Wheel | — | — | Zoom to cursor |
| Right drag | Orbit (configurable) | Pan (configurable) | — |
| View cube face | Snap to orthographic view | — | — |

Rules:

1. Navigation bindings are configurable in Settings and persist per user. **Current code differs:** left drag pans, middle drag dollies, right drag orbits (`ThreeViewer.tsx` pointer handlers). The table above is the target; the difference is recorded in [CODE_DEVIATIONS §7](../../status/CODE_DEVIATIONS.md).
2. `F` fits all visible objects; `H` restores the home view.
3. Selection must not be cleared by navigation.
4. Requirement: UI-INT-12.

## 11. Non-Goals

1. Free-form command-line syntax in the viewport (use the console).
2. Gesture/touch interaction.
3. Custom user macros or scripting bindings.

## 12. Open Questions

| # | Question | Owner | Needed by |
|---|---|---|---|
| Q1 | Should `Ctrl+click` on an already-selected object remove it, or enter multi-edit? | Design | G1 |
| Q2 | Is undo depth 50 sufficient, or should it be memory-bounded instead? | Frontend | G2 |
| Q3 | Should the history panel allow restoring a state without discarding later history? | Product | G3 |
