---
title: "OmniM Frontend Developer Guide"
status: current
last_updated: 2026-10-29
applies_to: "web/"
evidence: "audited against web/src structure on 2026-10-29; recipes reference target contracts"
---
# OmniM Frontend Developer Guide

> **Document role.** The hands-on guide for changing the frontend: where things live, and the five recipes used most often. It complements [ARCHITECTURE_MAP](ARCHITECTURE_MAP.md), which maps crates.
>
> **Dual-track maintenance obligation**: `FRONTEND_GUIDE.zh-CN.md` is a maintained translation. Section numbering and code paths must match.
>
> **Honesty rule.** Recipes marked 📅 describe the target structure and will not work until the referenced contract is implemented.

## 1. Purpose

The frontend has 20,000 lines, three editor surfaces, and one 1,794-line renderer. This guide shortens the path from "I need to change X" to "here is the file and the check".

## 2. Where Things Live

| Concern | Location |
|---|---|
| Routes | `web/src/app/App.tsx` |
| Shell | `web/src/app/layout/AppLayout.tsx` (📅 replaced by `app/shell/`) |
| Panel registration | `web/src/app/panels/PanelRegistry.tsx`, `defaultPanels.ts` |
| Session/UI state | `web/src/app/store/appStore.ts` |
| 3D scene and picking | `web/src/app/components/ThreeViewer.tsx`, `viewer/three/` |
| Creation forms | `web/src/app/components/create/` |
| Console | `web/src/app/components/OmniPanel.tsx` |
| Sketch editor | `web/src/sketch/` |
| Library / series pages | `web/src/library/`, `web/src/series/` |
| REST client | `web/src/shared/api.ts` |
| Types | `web/src/shared/types.ts`, `shared/types/creation.ts` |
| i18n (📅) | `web/src/shared/i18n/` |
| Tokens (📅) | `web/src/styles/tokens.css` |
| Unit tests | `web/src/__tests__/` |

## 3. Add a Panel

1. Define the component in `web/src/app/panels/` or `components/`.
2. Register it in `defaultPanels.ts` with `id`, `position`, `title`, `order`, `visible()`.
3. If it needs a tab icon or default width, extend `PanelTypes.ts` (📅 contract: [IA_SHELL](../specification/ui/IA_SHELL.md) §4).
4. Add an empty and error state ([FEEDBACK_ERRORS](../specification/ui/FEEDBACK_ERRORS.md) §3).
5. Add a screenshot to the visual baseline and a row to the requirement register.

**Rule:** a panel that fetches data must not mount its own global provider; data access goes through `shared/api.ts`.

## 4. Add a Command

1. Add the command to the registry with `id`, `labelKey`, `group`, `shortcut?`, `enabled(ctx)`, `run(ctx)` (📅).
2. Wire it into the ribbon group and the menu.
3. Regenerate [KEYBOARD_MAP](../reference/KEYBOARD_MAP.md) if a shortcut changed.
4. Add the command to the palette coverage test.
5. Requirement: `UI-INT-03` — a shortcut not in the registry is a defect.

## 5. Add a Dialog

1. Use the shared dialog family (`ConfirmDialog`, `PromptDialog`, form shell) (📅).
2. Provide `aria-labelledby`, `aria-describedby`, and an initial focus target.
3. `Esc` closes, `Enter` confirms, focus returns to the invoker.
4. Destructive dialogs state impact and reversibility ([INTERACTION](../specification/ui/INTERACTION.md) §7.3).
5. Never use `window.confirm`; `check_ui_contracts.py --strict --dialogs` blocks it.

## 6. Add a Token

1. Add a primitive only if no existing primitive fits.
2. Add a semantic token per theme in `tokens.css` (📅).
3. Map component tokens if the value is component-specific.
4. Verify contrast in both themes.
5. Never hardcode the value in a component; `check_ui_contracts.py --strict --tokens` blocks it.

## 7. Add a String

1. Add the key to `en-US` and `zh-CN` catalogues in the same change (📅).
2. Follow `area.component.element` naming ([I18N_L10N](../specification/ui/I18N_L10N.md) §7).
3. Use ICU for plurals and numbers.
4. If the term is new to the product, add it to [GLOSSARY](../reference/GLOSSARY.md) first.

## 8. Add a Test

| Layer | Where | Minimum |
|---|---|---|
| Unit | `web/src/__tests__/` | Pure logic, state machines, formatting |
| Component | co-located (📅) | Rendering and interaction of a component family |
| E2E | `web/tests/e2e/` (📅) | One user-visible path |
| Visual | `web/tests/visual/` (📅) | Any layout-affecting change |
| A11y | `web/tests/a11y/` (📅) | Changed surface, axe rules |
| Performance | `web/tests/perf/` (📅) | Changed hot path |

**Rule:** a test that cannot fail for the stated reason is not evidence.

## 9. Debugging

| Symptom | First check |
|---|---|
| Scene empty | `reloadScene()` token, `/api/mesh` response, WebGL context |
| Selection not updating | `appStore.selectedIds`, tree visibility |
| Creation stuck | `creationPhase`, `creationPoints`, form validation errors |
| Panel blank | error boundary log, `/api/*` response, provider order |
| Theme not applied | `data-theme` on `<html>`, token file import order |
| String shows a key | missing key in the active locale, fallback chain |

## 10. Non-Goals

1. A tutorial for React or three.js.
2. Backend development recipes (see [ARCHITECTURE_MAP](ARCHITECTURE_MAP.md)).
3. API reference duplication (see [API](../reference/API.md)).

## 11. Open Questions

| # | Question | Owner |
|---|---|---|
| Q1 | Should `ThreeViewer` be split into camera, scene, interaction, and overlays before the shell rewrite? | Frontend |
| Q2 | Do we keep Zustand, or move to a command-store model? | Frontend |
| Q3 | Should component tests use Testing Library or Playwright component mode? | Frontend |
