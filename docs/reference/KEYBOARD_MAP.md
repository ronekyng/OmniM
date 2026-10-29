---
title: "OmniM Keyboard Map"
status: design
last_updated: 2026-10-29
applies_to: "web/"
evidence: "verified against web/src/app/App.tsx, sketch/ComponentEditorPage.tsx, OmniPanel.tsx, ObjectTree.tsx, AiChat.tsx on 2026-10-29"
---
# OmniM Keyboard Map

> **Document role.** The user-facing shortcut reference. The **command registry is the source of truth** ([INTERACTION](../specification/ui/INTERACTION.md) §5); this document mirrors it and is checked by `scripts/gen_keymap.py --check` once the registry lands.
>
> **Dual-track maintenance obligation**: `KEYBOARD_MAP.zh-CN.md` is a maintained translation. Keys, commands, and scopes must match row for row.
>
> **Honesty rule.** Bindings marked ✅ exist in the code today; 📅 are contractual and not yet implemented.

## 1. Source of Truth

| Layer | Artifact |
|---|---|
| Command definitions | Command registry (planned) |
| Shortcut bindings | `web/src/shared/commands/keymap.ts` (planned) |
| This document | Mirrors the registry; never hand-edited after generation |
| Conflict resolution | Registry load order fails the build on duplicates within a scope |

## 2. Global Shortcuts

| Shortcut | Command | Scope | Status |
|---|---|---|---|
| `Ctrl/Cmd+S` | Save project | global | 📅 |
| `Ctrl/Cmd+Shift+S` | Save as | global | 📅 |
| `Ctrl/Cmd+Z` | Undo | global | 🔀 |
| `Ctrl/Cmd+Shift+Z` / `Ctrl+Y` | Redo | global | 🔀 |
| `Ctrl/Cmd+K` | Command palette | global | 📅 |
| `Ctrl/Cmd+A` | Select all visible | global | 📅 |
| `Delete` | Delete selection | global | ✅ |
| `Esc` | Cancel innermost action | global | ✅ |
| `Shift+F10` | Context menu | global | 📅 |
| `F1` | Shortcut help | global | 📅 |
| `F6` | Cycle regions | global | 📅 |
| `?` | Contextual help for focused control | global | 📅 |

## 3. Viewport Shortcuts

| Shortcut | Command | Status |
|---|---|---|
| `W` | Translate mode | ✅ |
| `E` | Rotate mode | ✅ |
| `F` | Fit visible | ✅ |
| `H` | Home view | ✅ |
| `G` | Toggle grid | ✅ |
| `O` | Toggle orthographic/perspective | ✅ |
| `X` | Toggle local/world space | ✅ |
| `Ctrl/Cmd+Click` | Toggle selection | ✅ |
| `Ctrl/Cmd+Drag` | Box select | 🔀 |

## 4. Picking Session

| Shortcut | Action | Status |
|---|---|---|
| `Enter` | Finish picking | ✅ |
| `Backspace` | Remove last point | ✅ |
| `Esc` | Cancel session | ✅ |
| `Shift` (hold) | Orthogonal constraint | 📅 |

**Target rule:** while picking, `Delete` is ignored; `Backspace` always means "undo one point". **Not enforced today:** `Delete` still deletes the selection because `App.tsx` only special-cases `Backspace` (see [CODE_DEVIATIONS §7](../status/CODE_DEVIATIONS.md) row 7.11).

## 5. Dialog and Focus

| Shortcut | Action | Status |
|---|---|---|
| `Tab` / `Shift+Tab` | Move focus within dialog | 📅 |
| `Enter` | Confirm (safe default) | 📅 |
| `Esc` | Close / cancel | 🔀 |
| `Space` | Activate focused control | 📅 |

## 6. Console and Editors

| Shortcut | Action | Status |
|---|---|---|
| `Ctrl/Cmd+Enter` | Execute console command | ✅ |
| `Enter` | Insert a newline in the console (execution is `Ctrl/Cmd+Enter`) | ✅ |
| `Tab` | JSON completion in console | ✅ |
| `↑` / `↓` | Command history | ✅ |
| `Esc` | Close completion / clear focus | ✅ |
| `Enter` (AI panel) | Send message | ✅ |
| `Shift+Enter` (AI panel) | New line | ✅ |
| `Ctrl/Cmd+Z` (sketch editor) | Undo sketch edit | ✅ |
| `Ctrl/Cmd+Y` (sketch editor) | Redo sketch edit | ✅ |
| `Enter` (sketch polyline/spline) | Finish entity | ✅ |
| `Esc` (sketch editor) | Cancel current tool | ✅ |
| `Delete` / `Backspace` (sketch) | Delete selected entity | ✅ |
| `/` (object tree) | Focus search | ✅ |

## 7. Conflict Rules

| Situation | Resolution |
|---|---|
| Same key in two scopes | The narrower scope wins (dialog > tree > viewport > global) |
| Same key in one scope | Registration fails; the build fails |
| Text input focused | Global shortcuts disabled unless `allowInInput` |
| `Esc` | Always resolves the innermost cancellable action |
| OS-reserved combinations | Not bound; documented as unavailable |

## 8. Change Procedure

1. Add or change the binding in the registry.
2. Regenerate this document (`gen_keymap.py`).
3. If the binding is user-visible, add a line to CHANGELOG.
4. If the binding changes muscle memory, note it in the release notes.

## 9. Non-Goals

1. User-defined shortcut remapping in this release.
2. Per-discipline binding sets.
3. Gamepad or 3D mouse bindings.

## 10. Open Questions

| # | Question | Owner | Needed by |
|---|---|---|---|
| Q1 | Should `Ctrl+W` close the active dock tab, or stay reserved for the browser? | Frontend | G2 |
| Q2 | Do we need a Vim-style mode for console-only users? | Product | G3 |
| Q3 | Should sketch editor bindings merge into the global registry? | Frontend | G2 |
