---
title: "RFC-0002: UI Shell and Design System"
status: design
last_updated: 2026-10-29
applies_to: "n/a (proposal)"
evidence: "n/a (proposal)"
---
# RFC-0002: UI Shell and Design System

- **Status**: Draft
- **Related**: [IA_SHELL](../specification/ui/IA_SHELL.md) · [DESIGN_SYSTEM](../specification/ui/DESIGN_SYSTEM.md) · [INTERACTION](../specification/ui/INTERACTION.md) · [PRD_FRONTEND](../product/PRD_FRONTEND.md)

## Summary

Replace the current ad-hoc frontend shell with a docked, token-driven, dual-theme workspace, and treat the UI as a versioned contract rather than a collection of panels. The proposal covers four coupled decisions: shell architecture, theme model, component governance, and requirement traceability.

## Motivation

The frontend executes real modeling operations but does not read as commercial software. Measured evidence (2026-10-29):

- The properties panel defaults to a centered floating overlay (`appStore.ts:89`), so selection covers the viewport.
- 712 hardcoded colors in the main stylesheet (731 across all CSS) bypass a token system that is declared twice.
- 858 CJK runs across 31 files with no i18n layer; 63 emoji occurrences, 25 of them used as icons.
- Five native browser dialogs; zero error boundaries; no save UI and zero client calls to the save endpoint.
- Accessibility is effectively absent (1 `aria-*`, 0 `role`).

A styling pass cannot fix this: the shell, the theme model, and the release process all have to change together.

## Detailed Design

### 1. Shell

Seven fixed regions (menu bar, ribbon, left dock, viewport, right dock, bottom dock, status bar). Docks are siblings of the viewport, resizable, tabbed, collapsible, and persisted per user. Floating mode exists as an escape hatch only.

### 2. Theming

Three-layer tokens (primitive → semantic → component) with `data-theme` on the root element. Dark and light are the product themes; high contrast is an accessibility mapping. The 3D scene reads a viewport palette derived from the same theme; render code contains no theme conditionals.

### 3. Components

A component is shippable only with a complete state matrix (default, hover, active, selected, disabled, focus, loading, error), semantic tokens only, keyboard operability, and an accessibility contract. Emoji are prohibited as icons.

### 4. Requirements and traceability

96 requirements in 11 areas, each with one ID, one contract anchor, at least one verification artifact, one gate, and one status. The register is the single source of truth; contracts and status documents reference IDs rather than restating requirements.

### 5. Project lifecycle

The application gains a project layer (`/projects`), explicit save and save-as, dirty tracking, autosave to a recovery file, crash recovery, external-change detection, and dependency reporting. This requires new server endpoints and a mutable `save_path`; the minimal contract is specified in [DATA_INTEGRITY](../specification/ui/DATA_INTEGRITY.md) §8.

## Compatibility

| Area | Impact |
|---|---|
| Existing routes | Workspace route stays `/`; editors unchanged; `/projects` is added |
| Existing panels | Registration API extended with icon/width/tab metadata, backward compatible |
| Existing shortcuts | Keep current bindings; add save, palette, help |
| Project files | No schema change required for the UI work |
| Server API | Four to six new endpoints; existing endpoints unchanged |
| Documentation | Existing docs gain references, not rewrites |

## Alternatives

| Alternative | Why rejected |
|---|---|
| Keep the floating panel and restyle it | Does not solve viewport occlusion or layout persistence |
| Single theme (dark only) | Rejected by product; light is required for office projectors and printing |
| CSS-in-JS component library (e.g. MUI) | Adds a large dependency and fights the engineering density requirement; tokens plus owned primitives stay smaller |
| Command palette instead of a ribbon | Keyboard-first users benefit, but discoverability for occasional users drops; the proposal keeps the ribbon and adds the palette |
| Rebuild the renderer first | The renderer is the strongest part; productization bottlenecks are shell, lifecycle, and feedback |

## Non-Goals

1. A public component library or plugin SDK.
2. Mobile or touch-first layout.
3. Replacing three.js or the geometry kernel.
4. Cloud collaboration UI.

## Open Questions

| # | Question | Owner |
|---|---|---|
| Q1 | Is desktop packaging in scope, and does it change the shell (native menu, multi-window)? | Product |
| Q2 | Do we adopt `forced-colors` instead of a custom high-contrast theme? | Design / A11y |
| Q3 | Should the ribbon collapse to a command bar at 1280–1599 px? | Design |
| Q4 | Is `save_path` mutability acceptable server-side, or should the client restart the server per project? | Backend |
