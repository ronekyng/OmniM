---
title: "OmniM Web Architecture — Current State"
status: current
last_updated: 2026-10-29
applies_to: "web/, crates/server"
evidence: "audited against web/src on 2026-10-29 (21,800 lines across ts/tsx/css; 94 unit tests passing)"
---
# OmniM Web Architecture — Current State

> **Document role.** This is the *architecture* layer: what the frontend actually is today, verified against the code. Design intent lives in [UI_ARCHITECTURE family](../specification/ui/IA_SHELL.md); maturity scores live in [FRONTEND_MATURITY](../status/FRONTEND_MATURITY.md).
>
> **Dual-track maintenance obligation**: `WEB_ARCHITECTURE.zh-CN.md` is a maintained translation. Keep section numbering and evidence rows aligned in the same change.
>
> **Honesty rule.** Every claim here cites a file or a measured count. Where the target differs, the difference is stated, not smoothed over.

## 1. Purpose and Scope

The web client is a React 19 single-page application served by Vite, talking to the Axum server over REST, rendering with three.js. This document maps its layers, data flow, rendering pipeline, state ownership, build, and known debt.

## 2. System Context

| Component | Technology | Evidence |
|---|---|---|
| Client | React 19, TypeScript 6, Vite 8 | `web/package.json` |
| Routing | react-router-dom 7 | `web/src/main.tsx`, `web/src/app/App.tsx` |
| State | Zustand 5 + TanStack Query 5 | `web/src/app/store/appStore.ts`, `web/src/shared/queries.ts` |
| Rendering | three.js 0.185, `@mlightcad/three-viewcube` | `web/src/app/components/ThreeViewer.tsx`, `ViewCube3D.tsx` |
| Transport | `fetch` wrapper | `web/src/shared/api.ts` (371 lines) |
| Server | Axum REST, JSON file persistence | `crates/server/src/lib.rs` |
| Build output | Static bundle in `web/dist` | `web/dist/` |

Routes today: `/projects` (project home), `/` (workspace), `/component-editor`, `/series-manager`, `/library-manager`.

## 3. Frontend Layering

```text
web/src/
├── main.tsx                 # providers: QueryClient, Router, Toast
├── app/
│   ├── App.tsx              # routes + global shortcuts + toolbar wiring (236 lines)
│   ├── layout/AppLayout.tsx # shell: sidebar + viewport + right panel
│   ├── panels/              # PanelRegistry, defaultPanels, domain/property/tree panels
│   ├── components/
│   │   ├── ThreeViewer.tsx  # 3D scene, picking, preview, section, measure (1794 lines)
│   │   ├── MainToolbar.tsx  # creation flow, import/export, discipline actions (460 lines)
│   │   ├── OmniPanel.tsx    # Omcom console: highlighting, completion, history (937 lines)
│   │   ├── create/          # creation forms, layer editor, catalog params
│   │   └── viewer/three/    # snapping, picking, markers, datum grids
│   ├── store/appStore.ts    # UI + session state (189 lines)
│   └── context/             # ThreeViewerContext
├── shared/                  # api, types, catalog, toast, geometry, creation state machine
├── sketch/                  # component editor + 2D/3D sketch canvases (1359 + 1038 lines)
├── library/                 # LibraryManagerPage (597 lines)
├── series/                  # SeriesManagerPage (648 lines)
└── __tests__/               # 5 Vitest files, 79 tests
```

Sizes are from `wc -l` on 2026-10-29. The concentration in `ThreeViewer.tsx`, `styles.css`, and `ComponentEditorPage.tsx` is the main maintainability risk.

## 4. Data Flow

A creation click currently follows this path:

```text
Ribbon / toolbar
  → appStore creationDispatch (state machine)
  → pick points in ThreeViewer (raycast + snapping)
  → form validation (shared/types/creation)
  → shared/api.ts POST /api/genrule | /api/objects | /api/catalog/instantiate
  → server Validator + Executor + persist
  → response
  → refresh() + reloadScene()  (full tree + full mesh reload)
```

Observations:

1. **Full reload after every mutation.** `refresh()` and `reloadScene()` refetch the whole tree and mesh; there is no delta path.
2. **Two write surfaces.** Creation uses `genrule`/`instantiate`; the console uses `POST /api/omcom`. Both converge server-side but diverge client-side.
3. **No optimistic state.** The UI waits for the server round trip before showing the object.
4. **Errors are strings.** `api.ts` throws `Error(message)`; there is no error code, so the UI cannot map failures to recovery actions.

## 5. Rendering Pipeline

| Stage | Implementation | Evidence |
|---|---|---|
| Renderer | `WebGLRenderer` with antialias | `ThreeViewer.tsx` |
| Tone mapping | ACES Filmic, exposure 1.2 | same |
| Shadows | PCF soft, 2048², single directional light | same |
| Environment | `RoomEnvironment` texture | same |
| Scene | Fog, hemisphere + ambient + two fill lights | same |
| Edges | `EdgesGeometry` overlay per mesh | same |
| Loop | Continuous `requestAnimationFrame` | same |
| Theme | Hardcoded scene colors (background, grid, edges) | same |

**Debt:** the loop runs continuously even when idle, and scene colors are literals rather than the palette contract in [DESIGN_SYSTEM §10](../specification/ui/DESIGN_SYSTEM.md).

## 6. State Ownership

| State | Owner | Persistence | Notes |
|---|---|---|---|
| Server data (tree, objects, meshes) | TanStack Query / store | Server | Refetched wholesale |
| Selection | `appStore` | None | String array |
| Creation session | `appStore` (`creationPhase`, points, form) | None | Mirrors `stateMachine.ts` |
| View settings (grid, ortho, space, visual mode) | `appStore` | None | Lost on reload |
| Layout (dock widths, panel mode) | `AppLayout` local state | None | Lost on reload |
| Theme | — | — | Not implemented |
| Locale | — | — | Not implemented |
| Project path / dirty | — | — | Not implemented |

**Gap:** there is no `layoutStore`, no theme provider, no i18n provider, and no project session state. These are prerequisites for `UI-SHL-06`, `UI-DS-02`, and `UI-I18N-02`.

## 7. Build and Tooling

| Item | State |
|---|---|
| Dev server | `vite --host 0.0.0.0 --port 5173` |
| Build | `vite build` → `web/dist` |
| Type check | `tsc --noEmit` passes |
| Unit tests | Vitest, 5 files, 79 tests |
| Lint / format | **None configured for `web/`** |
| E2E | **None** |
| Visual regression | **None** |
| Bundle analysis | **None** |

## 8. Known Debt

Counts measured on 2026-10-29; they are the baseline for [FRONTEND_MATURITY](../status/FRONTEND_MATURITY.md). **P9 update:** D1 (docked properties), D2 (project home/save), D6 (auto-naming), D7 (native dialogs) and D8 (error boundaries) are resolved; the rows remain as the measurement snapshot.


| # | Debt | Evidence | Requirement |
|---|---|---|---|
| D1 | Properties panel defaults to a centered floating overlay | `appStore.ts:89` `panelMode: 'floating'`; `AppLayout` `.floating-panel-overlay` uses `inset:0` + center | UI-SHL-03 |
| D2 | No project home, new/open/save-as; client never calls `POST /api/project/save` | route table; zero `saveProject` references | UI-DAT-01, UI-DAT-02 |
| D3 | 712 hardcoded colors in the 4,110-line global stylesheet (731 across all CSS) | `web/src/styles.css` | UI-DS-04 |
| D4 | Tokens defined twice, then bypassed | `globals.css` and `styles.css` both define the same `:root` block | UI-DS-01 |
| D5 | 63 emoji occurrences; 25 used as panel icons | `DomainPanels.tsx` `icon: '▦'` etc. | UI-DS-09 |
| D6 | 52 `Date.now()` call sites, most building object names | `wall-${Date.now()}` pattern | UI-INT-09 |
| D7 | 5 native dialogs | `window.confirm/alert/prompt` | UI-FBK-05 |
| D8 | No error boundary | zero `ErrorBoundary`/`componentDidCatch` | UI-FBK-06 |
| D9 | No i18n; 858 CJK runs inline across 31 files | source scan | UI-I18N-01 |
| D10 | Accessibility near zero | 1 `aria-*`, 0 `role` | UI-A11Y-04 |
| D11 | No instancing, LOD, or list virtualization | zero `InstancedMesh`/virtualizer references | UI-PRF-03, UI-PRF-07 |
| D12 | Continuous render loop | unconditional `requestAnimationFrame` | UI-PRF-04 |
| D13 | Stale docs reference removed files | `ARCHITECTURE_MAP` cites `OmcomConsole.tsx`, `CatalogBrowser.tsx`, `SeriesBrowser.tsx` | — |

## 9. Extension Points

| Extension | Mechanism | Contract |
|---|---|---|
| New panel | `PanelRegistry.register` + `defaultPanels` | [IA_SHELL §4](../specification/ui/IA_SHELL.md) |
| New command | Command registry (target) | [INTERACTION §2](../specification/ui/INTERACTION.md) |
| New dialog | `DialogProvider` (target) | [INTERACTION §7](../specification/ui/INTERACTION.md) |
| New token | `tokens.css` semantic layer | [DESIGN_SYSTEM §2](../specification/ui/DESIGN_SYSTEM.md) |
| New locale | `shared/i18n/<locale>.ts` | [I18N_L10N §3](../specification/ui/I18N_L10N.md) |
| New test | `src/__tests__` (unit), `tests/e2e` (target) | [TESTING](../guides/TESTING.md) |

## 10. Non-Goals

1. Server-side rendering.
2. Micro-frontend decomposition.
3. A public plugin API for panels.
4. Replacing three.js in this product cycle.

## 11. Open Questions

| # | Question | Owner |
|---|---|---|
| Q1 | Should `ThreeViewer.tsx` be split before or after the shell rewrite? | Frontend |
| Q2 | Does the app need a global store split (layout / project / session)? | Frontend |
| Q3 | Should the mesh endpoint gain a delta mode before instancing work starts? | Frontend / Backend |
