---
title: "OmniM UI Design System"
status: design
last_updated: 2026-10-29
applies_to: "web/"
evidence: "n/a (design contract)"
---
# OmniM UI Design System

> **Contract scope.** This document fixes tokens, themes, typography, spacing, z-index, iconography, component states, and viewport palette. It is the normative source for requirements `UI-DS-01` … `UI-DS-12`.
>
> **Dual-track maintenance obligation**: `DESIGN_SYSTEM.zh-CN.md` is a maintained translation. Keep section numbering, tables, token names, and requirement IDs aligned in the same change.
>
> **Status.** Design contract. The current stylesheet predates these tokens; migration status is in [FRONTEND_MATURITY](../../status/FRONTEND_MATURITY.md).

## 1. Purpose and Principles

1. **Token-first.** Every visual value is named, themed, and reviewable.
2. **Two themes, one structure.** Dark and light differ only in semantic values.
3. **Engineering density.** 12–13 px body text, 24–32 px row heights, no decorative whitespace.
4. **State completeness.** A component without all states is not shippable.
5. **No exceptions by inline style.** One-off values are a design-system defect.

## 2. Token Architecture

### 2.1 Three layers

| Layer | Prefix | Example | Consumed by |
|---|---|---|---|
| Primitive | `--omni-color-*`, `--omni-space-*` | `--omni-color-blue-500: #4a90d9` | Semantic layer only |
| Semantic | `--surface-*`, `--text-*`, `--border-*`, `--accent-*` | `--surface-panel` | Components |
| Component | `--btn-*`, `--tree-*`, `--viewport-*` | `--btn-bg-hover` | One component family |

**Rule:** components MUST consume semantic or component tokens. A primitive reference inside a component file is a defect detected by `scripts/check_ui_contracts.py --tokens`.

### 2.2 Naming

```text
--<layer>-<role>-<variant>-<state>
  role:    surface | text | border | accent | status | shadow
  variant: panel | elevated | sunken | muted | strong | subtle
  state:   default | hover | active | disabled | selected | focus
```

Names use semantic roles, never visual descriptions (`--surface-panel`, not `--gray-800-panel`).

### 2.3 Themes and `data-theme`

Themes are applied on `<html data-theme="dark|light|hc">`. All semantic tokens are redefined per theme in one place:

```css
:root, [data-theme="dark"] { color-scheme: dark;  --surface-app: #16181d; }
[data-theme="light"]      { color-scheme: light; --surface-app: #f4f6f9; }
[data-theme="hc"]         { color-scheme: dark;  --surface-app: #000000; }
```

No component may branch on the theme in JavaScript except for the 3D scene (which reads the viewport palette in §10).

### 2.4 Enforcement

| Check | Rule | Gate |
|---|---|---|
| Hardcoded colors | Zero hex/rgb/hsl outside token files; `--strict --tokens` enforces it | UI-DS-04 |
| Token layer | Zero primitive references in component styles | UI-DS-01 (scan 📅 until component tokens exist) |
| Off-scale values | Font sizes, spacing, radii, shadows from the defined sets | UI-DS-06, UI-DS-07 |
| Z-index | Only from the scale in §7 | UI-DS-08 |

## 3. Theming

### 3.1 Dark theme

The primary engineering theme. Neutral, low-chroma surfaces; one blue accent; status colors reserved for meaning.

| Role | Value |
|---|---|
| App background | `#16181d` |
| Panel surface | `#1b1e24` |
| Elevated surface | `#22262e` |
| Sunken surface | `#121418` |
| Border default | `#2e333c` |
| Border strong | `#3d444f` |
| Text primary | `#e6e8eb` |
| Text secondary | `#b3b9c4` |
| Text muted | `#8b93a1` |
| Text disabled | `#5a616e` |
| Accent | `#4a90d9` |
| Accent hover | `#5ba0e9` |
| Accent active | `#3d7fc4` |
| Focus ring | `#7fc4ff` |

### 3.2 Light theme and OS preference

The light theme is a **working theme**, not a print theme: contrast-stable on a bright office monitor.

| Role | Value |
|---|---|
| App background | `#f4f6f9` |
| Panel surface | `#ffffff` |
| Elevated surface | `#ffffff` |
| Sunken surface | `#e9edf2` |
| Border default | `#d5dae2` |
| Border strong | `#b8c0cb` |
| Text primary | `#1f2329` |
| Text secondary | `#4a5260` |
| Text muted | `#6b7482` |
| Text disabled | `#a2aab6` |
| Accent | `#1a6fd4` |
| Accent hover | `#1560b8` |
| Accent active | `#11509b` |
| Focus ring | `#1a6fd4` |

Rules:

1. First run follows `prefers-color-scheme`.
2. An explicit choice is stored in `localStorage` (`omnim.theme`) and wins over the OS.
3. Theme switching applies in one frame; no reload, no scene rebuild beyond palette updates.
4. Requirement: UI-DS-02, UI-DS-03.

### 3.3 Semantic token map

| Semantic token | Dark | Light |
|---|---|---|
| `--surface-app` | `#16181d` | `#f4f6f9` |
| `--surface-panel` | `#1b1e24` | `#ffffff` |
| `--surface-elevated` | `#22262e` | `#ffffff` |
| `--surface-sunken` | `#121418` | `#e9edf2` |
| `--border-default` | `#2e333c` | `#d5dae2` |
| `--text-primary` | `#e6e8eb` | `#1f2329` |
| `--text-muted` | `#8b93a1` | `#6b7482` |
| `--accent-default` | `#4a90d9` | `#1a6fd4` |
| `--status-danger` | `#e05252` | `#c62828` |
| `--status-warning` | `#e0a030` | `#a86400` |
| `--status-success` | `#4caf72` | `#2e7d32` |
| `--status-info` | `#4a90d9` | `#1565c0` |

### 3.4 High contrast

A third mapping (`data-theme="hc"`) for accessibility, not a brand theme: pure black background, pure white text, saturated accents, 3:1 minimum non-text contrast, no shadows, 2 px focus ring.

Requirement: UI-DS-11, UI-A11Y-07.

## 4. Semantic State Matrix

Every interactive role must define six states. Missing states block component registration.

| Role | Default | Hover | Active | Selected | Disabled | Focus |
|---|---|---|---|---|---|---|
| Surface | `--surface-panel` | `--surface-elevated` | `--surface-sunken` | `--accent-subtle` | `--surface-panel` @60% | ring |
| Border | `--border-default` | `--border-strong` | `--accent-default` | `--accent-default` | `--border-default` @50% | ring |
| Text | `--text-primary` | `--text-primary` | `--text-primary` | `--accent-default` | `--text-disabled` | — |
| Accent | `--accent-default` | `--accent-hover` | `--accent-active` | `--accent-active` | `--accent-default` @40% | ring |
| Danger | `--status-danger` | lighten 8% | darken 8% | `--status-danger` | 40% | ring |

Disabled elements MUST also set `aria-disabled="true"` and MUST NOT receive pointer events that mutate state.

## 5. Typography

| Token | Size | Line height | Use |
|---|---|---|---|
| `--text-xs` | 11 px | 16 px | Dense metadata, status bar |
| `--text-sm` | 12 px | 18 px | Panel labels, table cells |
| `--text-md` | 13 px | 20 px | Body, inputs, menus |
| `--text-lg` | 15 px | 22 px | Section headers |
| `--text-xl` | 20 px | 28 px | Page titles |
| `--text-2xl` | 28 px | 36 px | Project home headline |

Rules:

1. Body copy is 12–13 px; 10 px is prohibited.
2. Numeric UI (coordinates, dimensions, frame rate) uses `font-variant-numeric: tabular-nums`.
3. Mono is reserved for the console, IDs, and code; never for labels.
4. Font stack: `Inter, "Segoe UI", system-ui, -apple-system, sans-serif`; CJK fallback `"Noto Sans SC", "PingFang SC", "Microsoft YaHei"`.
5. Requirement: UI-DS-06.

## 6. Spacing, Radius, Shadows

| Scale | Values |
|---|---|
| Spacing | 4, 8, 12, 16, 24, 32 px (`--space-1…6`) |
| Row height | 24 px compact, 28 px default, 32 px comfortable |
| Radius | 2, 4, 6, 8 px (`--radius-1…4`) |
| Shadow | sm `0 1px 2px rgba(0,0,0,.30)`, md `0 6px 16px rgba(0,0,0,.35)`, lg `0 16px 40px rgba(0,0,0,.45)` |

Shadows are for elevated surfaces only (popover, dialog, floating panel). Docked panels use borders, never shadows. Requirement: UI-DS-07.

## 7. Z-Index Scale

| Band | Value | Layer |
|---|---|---|
| 0 | 0 | Viewport canvas |
| 1 | 10–19 | Viewport overlays (view cube, tool strip, hints) |
| 2 | 20–29 | Docks and splitters |
| 3 | 30–39 | Popovers, menus, tooltips anchored to chrome |
| 4 | 40–49 | Dialog backdrop and dialog |
| 5 | 60–69 | Toasts and live-region announcements |
| 6 | 90–99 | Debug overlays (never in production) |

No component may use a z-index outside these bands. Requirement: UI-DS-08.

## 8. Iconography

| Rule | Value |
|---|---|
| Sizes | 16 px (inline), 20 px (toolbar), 24 px (ribbon) |
| Stroke | 1.5 px, round cap and join |
| Color | `currentColor` only |
| Grid | 16 px artboard, 1 px padding, 2 px corner radius |
| Prohibited | Emoji, raster images, mixed stroke weights |
| Accessibility | Icon-only buttons require an accessible name |

Icon set lives in `web/src/app/components/icons/` and is exported from one index. Requirement: UI-DS-09, UI-A11Y-05.

## 9. Component Contract

A component is registered only when it declares all of the following:

| Item | Requirement |
|---|---|
| States | default, hover, active, selected, disabled, focus, loading, error |
| Sizes | sm / md (lg only for dialogs and page actions) |
| Tokens | Semantic tokens only |
| Keyboard | Focusable, operable by Enter/Space where applicable |
| A11y | Role, accessible name, `aria-disabled` when disabled |
| Documentation | One row in the component inventory |

Minimum component inventory for G2: Button, IconButton, SplitButton, Select, Combobox, NumberField, TextField, Checkbox, Radio, Switch, Slider, Tabs, Tree, Table, Panel, Tooltip, Popover, ContextMenu, Dialog, ConfirmDialog, Toast, Skeleton, EmptyState, ProgressBar, Banner, StatusBar, Badge.

Requirement: UI-DS-10.

## 10. Viewport Palette

The 3D scene must read correctly in both themes. Scene colors are tokens, not literals in renderer code.

| Scene role | Dark | Light |
|---|---|---|
| Background | `#232830` | `#eef1f5` |
| Grid major | `#4a5566` | `#b6bfca` |
| Grid minor | `#333b47` | `#d3d9e2` |
| Edge | `#11141a` | `#8a929e` |
| Selection | `#4fc3f7` | `#1565c0` |
| Hover | `#7fd4ff` | `#1a6fd4` |
| Section plane | `#e0a030` | `#a86400` |
| Measurement | `#4caf72` | `#2e7d32` |
| Ghost preview | `#4fc3f7` @35% | `#1565c0` @30% |

Rules:

1. Renderer reads a `ViewerPalette` object derived from the active theme; no theme conditionals inside render code.
2. Selection must remain distinguishable from hover at 3:1 against the background.
3. Requirement: UI-DS-12, UI-A11Y-01.

## 11. Governance

| Rule | Value |
|---|---|
| Token changes | Require a design review entry and a version bump of `tokens.css` |
| New component | Requires the full state matrix and an inventory row |
| Deprecation | Mark deprecated for one minor version before removal |
| Exceptions | Inline style requires a comment with an expiry date and an owner |
| Automated checks | `check_ui_contracts.py`, visual regression, contrast audit |

## 12. Non-Goals

1. User-authored themes or brand skins.
2. Per-project theming.
3. Print stylesheets.
4. Component library publication outside this repository.

## 13. Open Questions

| # | Question | Owner | Needed by |
|---|---|---|---|
| Q1 | Should high contrast be a theme or an OS-level `forced-colors` adaptation? | Design / A11y | G3 |
| Q2 | Is 15 px enough for section headers, or should it be 16 px? | Design | G2 |
| Q3 | Do we need a compact density mode for 1080p laptops? | Product | G2 |
