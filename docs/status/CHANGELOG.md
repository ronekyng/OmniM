---
title: "Changelog"
status: history
last_updated: 2026-11-10
applies_to: "repo"
evidence: "audited against code 2026-10-27"
---
# Changelog

All notable changes to the OmniM project are documented in this file. Substantive batch summaries only — this is not a commit log.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project does not currently use semantic versioning; entries are organized by roadmap batch.

**Last updated: 2026-11-10**

## Keep a Changelog legend

- **Added** — new capabilities
- **Changed** — changes to existing behavior
- **Fixed** — bug fixes / deviations resolved
- **Not implemented (known gaps)** — planned but absent capabilities, stated explicitly for honesty

---

## [2026-11-10] — I18N-01 full externalisation close-out (UI-I18N-01 ✅ · CJK 988→51)

### Changed
- **Two real perf fixes (UI-PRF-01..04)**: the ~3.5 s re-mount stall was attributed to SwiftShader first-frame rendering (PCFSoft shadow maps + PMREM environment rendered by an always-on rAF loop); `ThreeViewer` now detects software rasterisers (disables shadows/env) and renders on demand (camera delta / scene mutation / interactive tool, mutators call `invalidate()`); the fixture gained a re-open measurement, a workspace_mount time-origin fix and replaced the broken idle probe (wall-clock, always 1) with Long Tasks — measured open/re-open 0.5-0.64 s (budget 3 s ✓), command palette 0.63-0.84 s (budget 0.12 s ✗, p95 re-run pending), idle main thread 0; PERFORMANCE contract notes and register updated.
- **UI-I18N-01 ✅**: every UI string now goes through `t()`/dictionary. This close-out batch clears the last high-frequency surfaces: `LibraryManagerPage` (full), `SeriesManagerPage` (full), `CreatePanel` remainder, `AddFeatureDialog` (14 parameter fields), `ComponentEditorPage` (role keys/failure toasts/constraint names), `EditorToolbar`, `AssemblyPanel` remainder, `AvailableComponentsPanel`, `FeaturePanel`, `GridPanel` (full), `MaterialBrowser` (full), `DomainPanels`/`CatalogParams`/`MainToolbar`/`ThreeViewer`, the creation pick-hint set (`pick.*`), `validateForm` strings (`create.err.*`), `catalog.ts` keyification (`DOMAIN_GROUPS` labelKey/labelKeys → `nav.group.*`/`nav.type.*`; `objectTypeLabelKey` + kernel `hasMessage`), `MATERIAL_CATEGORY_KEYS` (`mat.cat.*`), and panel-registry `titleKey`. Residual machine-scanned CJK: **51 runs** = layer templates / default material names 50 runs (model data per I18N §1) + language self-name 1 run (§3 "中文").
- Dictionaries gained ~260 new keys (`create.*`, `layer.*`, `mat.*`, `library.*`, `series.*`, `assy.*`, `toolbar.*`, `catp.*`, `viewer.*`, `avail.*`, `feature.*`, `grid.*`, `pick.*`, `create.err.*`, …); zh/en share keys by unit test; multiple specs force zh-CN.
- Gate board updated: **G2 32/37** (I18N-01 🔀→✅); the debt-ledger entry "UI strings not externalized (31 files)" is replaced by "hard-coded English JSX not yet scanned" (📅 follow-up).

## [2026-11-04] — G2 closing batch II (SHL-09/11 · FBK-03 · DS-01/06/12 · INT-05/10/11 · CMP-05)

### Added
- **Single shortcut registry (UI-INT-03 ✅)**: `app/commands.ts` `buildWorkspaceCommands` is the single producer of the command array; the new `global_keydown` gate collapses window keyboard listeners to an allow-list; the component editor attaches its page keys to the shell container.
- **Configurable viewport navigation (UI-INT-12 ✅)**: registry command `view.nav` toggles shipped/contract schemes (contract: left inert = selection-only, middle orbit, right pan), `omnim.nav` persistence + `applyNavScheme`; the SHIFT+middle mapping and contract-default stay in CODE_DEVIATIONS §7.
- **Project metadata & auto-save semantics (UI-DAT-04/05/07 ✅)**: `GET/PUT /api/project/meta` (file rename transaction); title-bar Project settings; every mutation persists, disk failure logged non-blocking, zero-loss reopen (`project_meta.rs` / `e2e/meta.spec.ts`).
- **i18n kernel & language switcher (I18N-02/03/04 ✅, I18N-01 🔀)**: `shared/i18n.ts` + dictionaries, title-bar switcher with instant switching/`<html lang>` sync; status bar & save-state copy externalised; `e2e/language.spec.ts`.
- **Unit suffixes everywhere (UI-INT-10 ✅)**: all numeric CreatePanel inputs carry a `.field-unit` (m/mm layers: elevations/offsets/sections/primitives/linear coordinates) and catalog params echo `{value} {unit}`; `e2e/units.spec.ts`.
- **1280×720 walkthrough (UI-CMP-05 ✅)**: the CMP-05 block in `e2e/narrow.spec` (full layout usable, no folding/blocking notice).
- **Context-menu keyboard invocation confirmed (UI-INT-05 ✅)**: Shift+F10 menu with menu/menuitem roles, arrows and Esc (`e2e/a11y.spec`), contract §6 annotation added.
- **Viewport token pair (UI-DS-12 ✅)**: `--viewport-label-*` fully defined in both themes; theme.spec covers live switching/AA contrast; contract §10 annotation.
- **Token gate confirmation (UI-DS-01 ✅)**: `--tokens` runs colors/colors_func = 0 (the existing gate is the evidence).
- **Single type scale & tabular numerals (UI-DS-06 ✅)**: every `font-size` in styles.css is on `--fs-*` (new subtitle/title/display steps); the new `font_size_raw` gate = 0; body-wide `tabular-nums`.
- **Double-click semantics closed (UI-INT-11 ✅)**: divider double-click resets the width; INTERACTION §7.4's context table matches the code (tree row = in-place rename).
- **Narrow-band folding (UI-SHL-11 ✅)**: 1024–1279 px folds both docks to 44 px icon strips (left icons re-open a drawer, right dock via per-panel strip icons, ≥1280 restores, <1024 blocking notice wins); `e2e/narrow.spec.ts`.
- **Floating panel drag / resize / position persistence (UI-SHL-09 ✅)**: drag strip + corner resize; `clampFloating` (32 px margins / 260×180 min); `omnim.layout.v1.floating[]` per panel id, restored on reload; `e2e/floating.spec.ts`.
- **Import progress past 1 s and cancellation (UI-FBK-03 ✅)**: indeterminate progress (role=progressbar) + elapsed time; cancellation via AbortController; a server-side cancel endpoint is backlog; `e2e/importProgress.spec.ts`.

## [2026-11-03] — Phase 8-1/2/3/4 and the first 8-5 slice: domain delivery batch

### Added
- **i18n kernel & language switcher (G2 · I18N-02/03/04 ✅)**: `shared/i18n.ts` (`t()`/`tPlural()`/`useI18n()`; detection order stored choice → navigator intersection → zh-CN; missing keys fall back to en-US; `Intl.PluralRules` plurals; coordinate/grouped-count/short-date/clock/angle formatters), `i18n/{zh-CN,en-US}.ts` dictionaries, a title-bar switcher that applies instantly without reload, persists to `omnim.locale` and syncs `<html lang>`; 19 unit tests + `e2e/language.spec.ts`. Title bar / leave-confirm / save-as strings externalised (I18N-01 🔀).
- **Domain panel content (first 8-5 slice)**: the ribbon domain switches also open the matching right-dock panel (mechanical = motion-joint list queried by selection via `?object_id`; building = grouped door/window schedule with refresh). Explicit empty states, retry on load failure; panels ride PanelRegistration/token styles with no new literals.
- **Opening panel variants**: door/window requests accept `leaf_count` (doors 1–2 / windows 1–4 with mullions), `with_sill` (outward sill slab) and `with_lintel` (spanning beam); objects record the frame-variant properties.
- **Door/window schedule**: `GET /api/schedules/openings?root_id=` groups Door/Window objects over the root's subtree (or the whole project) by (type, opening size [mm], frame variants).
- **Terrain TIN fitting**: `genrule: terrain` fits a Bowyer–Watson triangulation from ≥3 survey points (`ObjectType::TerrainModel` surface mesh + survey/stats properties); duplicate XY keeps the first elevation, collinear surveys are refused.
- **Elevation query & earthwork**: `GET /api/terrain/{id}/elevation?x=&y=` barycentric lookup (404 outside the hull); `POST /api/earthwork` (inline points or a terrain object + design_z/cell) integrates grid cut/fill/net; outside-hull cells are counted, not extrapolated.
- **Bridges/tunnels follow the full alignment**: new `genrule: bridge` (layered deck, bridge × linear) and `genrule: tunnel` (circular lining tube swept along the full polyline via the pipe kernel) — replacing the first→last-point-only construction.
- **Branched piping**: pipe requests support `branch_of`/`branch_point`/`target` — the branch point snaps onto the host route (tolerance 0.1 m), creating a tee fitting + a routed branch pipe (Pipe-Path instantiation) in one undo step.
- **Motion joints**: `/api/joints` (revolute/prismatic/fixed/spherical) — axis normalization, paired-limit validation (radians/metres), unordered-pair same-kind dedup; deleting an endpoint object cascades the touching joints and one undo step restores the object and the joint.
- **BOM/quantity endpoint**: `GET /api/objects/{id}/bom` returns **all descendants** of a container (kind=component/part/proxy, catalog source, bbox volume in m³) with `totals`; a leaf returns itself; unknown id 404. The properties panel renders the table for selected Assembly containers (with a "bbox estimate" total).
- **Tree row grid layout fix**: nested subtrees used to be flex siblings that squeezed the row label to zero width (clicking an assembly row selected a child); rows are now grid.

### Added
- **Terrain TIN fitting**: `genrule: terrain` fits a Bowyer–Watson triangulation from ≥3 survey points (`ObjectType::TerrainModel` surface mesh + survey/stats properties); duplicate XY keeps the first elevation, collinear surveys are refused.
- **Elevation query & earthwork**: `GET /api/terrain/{id}/elevation?x=&y=` barycentric lookup (404 outside the hull); `POST /api/earthwork` (inline points or a terrain object + design_z/cell) integrates grid cut/fill/net; outside-hull cells are counted, not extrapolated.
- **Bridges/tunnels follow the full alignment**: new `genrule: bridge` (layered deck, bridge × linear) and `genrule: tunnel` (circular lining tube swept along the full polyline via the pipe kernel) — replacing the first→last-point-only construction.
- **Branched piping**: pipe requests support `branch_of`/`branch_point`/`target` — the branch point snaps onto the host route (tolerance 0.1 m), creating a tee fitting + a routed branch pipe (Pipe-Path instantiation) in one undo step.
- **Motion joints**: `/api/joints` (revolute/prismatic/fixed/spherical) — axis normalization, paired-limit validation (radians/metres), unordered-pair same-kind dedup; deleting an endpoint object cascades the touching joints and one undo step restores the object and the joint.
- **BOM/quantity endpoint**: `GET /api/objects/{id}/bom` returns **all descendants** of a container (kind=component/part/proxy, catalog source, bbox volume in m³) with `totals`; a leaf returns itself; unknown id 404. The properties panel renders the table for selected Assembly containers (with a "bbox estimate" total).
- **Tree row grid layout fix**: nested subtrees used to be flex siblings that squeezed the row label to zero width (clicking an assembly row selected a child); rows are now grid.

## [2026-11-02] — G2 design-system / accessibility slice

### Added
- **Semantic tokens and dual themes**: `globals.css` is the single token source with surface/border/text/accent/status/button/syntax/viewport layers; the light theme overrides every colour token and `theme.spec.ts` covers reload-free switching, persistence and AA contrast in both themes.
- **Z-index scale**: §7 `--z-*` tokens replace every magic number; a `zindex` contract check prevents regressions.
- **Single icon set**: title bar, tree, tabs, library/series managers, sketch features and domain quick actions now use 16×16 monochrome SVGs; emoji/pictographic icons are at zero.
- **Dialog and focus contract**: `useDialogFocus` centralizes the focus trap, inert background, Esc and focus restore; destructive confirms default to Cancel.
- **Destructive impact**: deletes state container descendants by type, promoted children for non-containers, and undo; the tree context-menu path now uses the same confirmation.
- **ARIA and keyboard**: object tree `tree/treeitem` with roving Tab/arrows/Enter/Home/End; `tablist/tab/tabpanel`; toolbar, menu, property table, toast and viewport semantics; `F6` region cycling plus route-heading and error-fallback focus.
- **axe audit**: launch, open, create, select, failed save and error fallback are covered by `axe.spec.ts` with 0 violations; run it with `npm run test:a11y`.
- **Undo/redo regression**: 60-step depth, 200-command cap, redo replay and grid-update endpoint now have explicit assertions.
- **Command surface**: `Ctrl/Cmd+K` palette (filter, arrows, Enter), `F1` shortcut help and global undo/redo shortcuts; a keyboard-only J1 (create storey + save) walkthrough is now covered.
- **Inline rename**: tree rows enter an input on F2/double-click, commit on Enter/blur, cancel on Esc; property rows navigate with ↑↓/Home/End.

### Fixed
- **Duplicate tokens and build failure**: removed the duplicated `:root` block in `styles.css`; repaired the dangling toast `max-width` that broke `vite build`.
- **Sample-project 409**: the project home could race an empty list or an orphaned slug; it now waits for the list, dedupes by display name and file slug, and retries a 409 with a suffix.
- **Tree keyboard double-move**: nested `treeitem` keydown bubbling moved focus twice; only the event-source row acts now.

## [2026-10-30] — Creation display & orientation fixes

### Fixed
- **New walls/slabs/roofs/floors were invisible**: PathSweep/genrule domain objects carry metre-scale geometry, but `/api/mesh` scaled every object with a `catalog_ref` by 0.001 as if it were millimetres (a 5 m wall became 5 mm). Only genuine catalog `Component` instances (template geometry authored in mm) are scaled now; domain objects render in metres.
- **Beams/pipes/ducts were created vertical**: catalog templates extrude along local +Z, and the old placement only sent a Z yaw, which cannot lay the length axis onto the picked segment. `InstantiateCatalogOperation` now accepts an optional `rotation` quaternion (applied before the position translation); the UI derives it from local +Z → picked direction and anchors the part at the segment start. The duplicate midpoint translation after catalog linear creation was also removed.

## [2026-10-30] — Assembly container + parts

### Added
- **`POST /api/assemblies`**: creates an `Assembly` **container** whose direct children are ordinary objects — catalog `Component` parts (`catalog_ref`, feature-tree geometry, params in mm, pose in m) or primitive `Part` bodies (box/cylinder/sphere/cone, baked pose). Creation is one `Command::CreateBatch` (including instantiation sketches), so it is a single undo step.
- **Subtree delete**: `DELETE /api/objects/{id}?cascade_children=true` (the default for containers) removes every descendant and snapshots the subtree in `DeleteCascade.deleted_children`; one undo restores the whole subtree.
- **Cascading transform**: `PUT /api/objects/{container}/transform` applies the matrix to every direct child and records one grouped undo entry (`Command::Batch`).
- New OMCOM surface: `ASSEMBLY_CREATE` (130) with `CreateAssemblyOperation`, `DELETE_OBJECT_SUBTREE` (131); the AssemblyPanel now supports catalog parts with parameter editing plus primitive parts and pose inputs.

### Changed
- An `Assembly` object is now a structural container (no geometry of its own) instead of an opaque merged mesh. The legacy `POST /api/genrule` `type: "assembly"` path remains for scripts and the built-in equipment types (`Stair`, `AHU`, `Chiller`, `Transformer`, `Reactor`) that reuse `AssemblyGen`.
- Route table: 87 unique paths / 88 registrations.

### Not implemented (known gaps)
- Per-part live placement (poses are baked), BOM/quantity endpoint, kinematic joints and nested assemblies — tracked in [ROADMAP](ROADMAP.md) and the [ASSEMBLY](../specification/ASSEMBLY.md) contract.

## [2026-10-30] — Audit remediation cycle (code / docs / UI-UX)

### Fixed
- **Model P0/P1**: `remove_containment` deleted from the vector and rebuilt both indexes; stirrup `z` now spreads along the rebar cage; `resolve_param` honours operator precedence (`1+2*3 = 7`); `set_parent` is atomic with a cycle guard; `CatalogStore.by_category` is rebuilt on deserialize; object deletion cascades (children promoted, relations/lifecycle/breakdown cleaned) with one-step undo (`DeleteCascade`); create-time `ifc_class`/`tags` writes are recorded as `ModifyObject`.
- **Fake success (omcom/amir)**: unknown sketch-constraint codes and out-of-range entity/constraint indexes now fail with rollback; array fan-out capped at 10k; AMIR `amir-undo`/`amir-redo` sentinels routed to the real `Executor::undo/redo`; failed sketch solves no longer write back coordinates; approximate boolean/chamfer/fillet now surface `warnings` (new `ExecutionResult.warnings` field, consumed by the console and AI panel).
- **IFC**: statement splitting is a string-literal-aware state machine (names with `;` survive), `''` escaping fixed, recursion depth caps on `parse_value`/`convert_ifc_item`/placement chains.
- **Server**: unauthenticated API now binds loopback by default (`OMNIM_HOST` to publish); import endpoints confined to allowed roots (traversal + symlink escapes rejected, including `/api/omcom` `LoadSeries`); export artifacts moved to `~/.omnim/exports` with nanosecond suffixes; 100 MB body limit scoped to IFC import; failed auto-save surfaces `persist_failed` in the response.
- **Web**: all 31 `api.ts` methods funnel through `request`/`requestVoid` (non-ok throws) with contract tests; 17 swallowed `.catch(() => {})` sites now report; Assembly/LayerEditor/catalog panels carry unit labels; the "Unsaved changes" false alarm now reads "Auto-saved server-side · Ctrl+S to snapshot"; grid spans use a semantic toggle; default object names dropped timestamps for `<prefix>-<nnn>`; constraint toolbar has titles/aria labels; sketch canvas adopted the shell graphite palette.

### Changed
- CI: `clippy --all-targets` (failures now visible) and `web` typecheck step; `check_write_paths.py` refuses to pass when it scans fewer than 20 handlers.
- Measured: 467 Rust tests (was 454), 100 web tests (was 94), 0 clippy errors under `--all-targets` (was a compile failure), `check_docs.py --strict-parity` green with no permanent exemptions.

### Not implemented (known gaps)
- G2/G3 frontend items: full token migration (712 hard-coded colours), i18n, A11Y, editor visual unification, icon-system rework.
- Structural debt tracked in the audit progress notes: server auth middleware/`TimeoutLayer`, persist off the write lock, omcom file split + `Result` executor, real CSG kernel, `App.tsx` selector granularity, react-query hooks enable-or-remove, clippy warning backlog ratchet.

## [2026-10-30] — Workspace shell re-layout (P9-S3)

### Changed
- **Toolbar is a real ribbon row**: the floating centered pill is gone; the command bar spans the workspace, groups Create/Modify/History/Exchange/Editors/Disciplines with separators, never shrinks labels into vertical text, and scrolls horizontally only when needed.
- **Left dock tabs**: Objects and Grids are separate tabs (object count badge) instead of stacked panels; the redundant "OmniM Editor" heading is removed and the tree gets the full dock height.
- **Grid editor**: the cramped inline form is replaced by labelled Label/Direction/Coordinate/From/To fields with full-width actions and a scrollable line list.
- **Viewport chrome**: the view tool strip is a 42 px edge rail; the storey selector moved to a top-left viewport control; the shortcut hint is a compact bottom-right chip.
- **Title bar**: duplicate project name in the breadcrumb removed, clean-state save text hidden, Save/Save-as/Reset grouped; the default tool is now "none" instead of an always-active Transform.
- **Bottom dock**: collapsed Console/AI tabs sit on the viewport edge without the previous empty strip.
- **Polish**: the grid form follows its list instead of being pinned to the bottom; the shortcut hint is a compact chip with the full list in its tooltip; the Component action has a stable width and group separators have more air.

### Not implemented (known gaps)
- No visual/e2e regression test for this re-layout; requirement statuses stay partial until the screenshot/e2e evidence exists.

## [2026-10-29] — G1 project home (P9-S2)

### Added
- **`/projects` home**: new project name, sample project, current session card, recent/available table with size, modified time and missing-file badge. Recent entries live in `localStorage` (`omnim.recent.projects.v1`, max 10) and never in the project file.
- **Project session API**: `GET /api/projects` (existing `.omnim` files with name/size/modified), `GET /api/projects/current` (`open` flag survives only while the server session does), `POST /api/projects/new` (unique slug, optional sample seed, writes the file, clears undo), `POST /api/projects/open` (loads, retargets saves, clears undo).
- **Sample project**: two storeys (0 / 3.9 m) and a four-line grid (A/B, 1/2), geometry-free so it opens instantly.
- **Workspace gating**: after a server restart the workspace redirects to `/projects` until the user opens/creates a project; the title-bar project name also returns to the home with an unsaved-changes prompt.
- **Allowed-root enforcement**: relative paths resolve under `OMNIM_PROJECTS_DIR` (default `~/.omnim/projects`, or the current project's parent); `..`, outside absolute paths, non-`.omnim` names and symlink escapes are rejected with `400` for open and save-as (`UI-SEC-03`).

### Changed
- Routes: 86 unique paths / 87 registrations (+4 project-session endpoints).
- Frontend tests: 9 files / 94 tests; Rust 454.
- Default project file moves to the project root instead of the server working directory.

### Not implemented (known gaps)
- Project metadata editing (`UI-DAT-07`), dependency report (`UI-DAT-09`), autosave/recovery (`UI-DAT-04/05`), missing-file "locate…", in-app navigation guard and theme/loading-state work.

## [2026-10-29] — G1 alpha shell and reliability (P9-S1)

### Added
- **Docked workspace shell**: properties dock on the right by default (260–600 px), width and dock mode persist in `localStorage` under `omnim.layout.v1`, and **Reset layout** restores the documented defaults. A blocking notice appears below 1024 px.
- **Title bar and status bar**: project name with dirty mark, breadcrumb (`Project / Workspace / tool`), explicit **Save** / **Save as…**, and a read-only status bar with cursor coordinates, selection, units and save state (FPS only in developer mode).
- **Error isolation**: application-, workspace- and panel-scoped `ErrorBoundary`; a crashing panel keeps the shell usable and offers Retry.
- **Shared dialog + notification layer**: `ConfirmDialog` (Escape cancels, Enter confirms) replaces every native dialog; `notify` lets non-hook modules raise typed toasts. Error toasts now persist until dismissed.
- **Explicit save state machine**: `saveState.ts` (clean/dirty/saving/failed, last-saved clock), `Ctrl/Cmd+S`, `beforeunload` interception while dirty, and `POST /api/project/save-as` (validates `.omnim`, retargets the session).
- **Auto-naming**: `naming.ts` implements `Type-nnn` with lowest-free reuse and duplicate suffixes, wired to creation, quick-create, child objects, duplication and drag-instantiation.

### Changed
- Routes: 82 unique paths / 83 registrations (added `/api/project/save-as`).
- Frontend tests: 8 files / 91 tests; Rust 450. UI contract scan: native dialogs 0.

### Fixed
- Removed the parallel-test race behind the intermittent `test_find_and_rebuild_catalog_instances` failure: tests that may re-seed the global `LIBRARY` now serialize through `LIBRARY_TEST_LOCK`.

### Not implemented (known gaps)
- Project home (`/projects`) and open/new endpoints (`UI-DAT-01`); in-app navigation guard for unsaved changes (`UI-DAT-06` is `beforeunload` only); dark/light theme (`UI-DS-02`); loading/empty states and editor breadcrumbs.

## [2026-10-29] — Frontend documentation baseline (G0)

### Added
- **Product layer** (`docs/product/`): vision and scope, personas and five journeys, a 96-requirement PRD, and a requirements traceability register (RTM).
- **UI contracts** (`docs/specification/ui/`): shell/IA, design system, interaction, feedback and errors, data integrity, accessibility, i18n, performance, compatibility, security, telemetry — 11 bilingual pairs.
- **Truth layer**: `WEB_ARCHITECTURE` (current state with measured debt), `FRONTEND_MATURITY` (ISO/IEC 25010 scorecard, 1.7/5), `FRONTEND_BASELINE`, `RELEASE_READINESS` (G0–G3).
- **Process layer**: definition of done, design operations, release and support checklist, and the G0 independent review record.
- **Reference**: keyboard map and a stable `OMNI-*` error-code catalogue.
- **Tooling**: `scripts/check_ui_contracts.py` (requirement IDs, keymap conflicts, error codes, debt metrics with `--strict`), `scripts/gen_keymap.py --check`; both plus `npm test` are now in CI. `check_docs.py` also validates front matter.

### Changed
- **Gate rebalance (O1):** six completeness requirements moved from G2 to G3 (`UI-DS-05/07/10`, `UI-A11Y-06/08`, `UI-I18N-05`); `UI-DS-08` retained at G2. Gate totals are now G1 25 · G2 37 · G3 34. Status totals remain ✅1 · 🔀17 · 📅78.
- **Corrected claims**: 81 unique REST paths (was 79/83/"~120"), 731 hex + 61 rgb/hsl colour literals, 858 CJK runs across 31 files, 63 emoji occurrences, minimum supported width 1280×720 with 1024–1279 degraded, `FCP ≤1.2 s` separated from `FMP ≤2.5 s`.
- **Fixed stale references**: `OmcomConsole.tsx` → `OmniPanel.tsx`, `CatalogBrowser.tsx` → `AvailableComponentsPanel.tsx`; `test-all.sh` no longer runs web tests twice.

### Not implemented (known gaps)
- Visual baseline screenshots, the performance reference machine, e2e/visual/a11y/perf suites, and English-literal detection in the string scan. Tracked as O3–O6 in the G0 review record.

## [2026-10-29] — Storey view + section link (P8-S4c)

### Added
- **Storey view selector** in the viewer toolbar: "全部楼层" plus one entry per `BuildingStorey` (labelled with its elevation). Selecting a storey filters the viewport to objects associated with it (`level_id`, falling back to `bottom_level_id`; unassigned geometry stays visible) and enables a plan section cut 1.2 m above the storey datum. Clearing the selection restores everything and disables the storey-driven section.
- **`/api/mesh` carries storey association** (`level_id`, `bottom_level_id`, `elevation`) so the filter is client-side; the mesh endpoint shape is otherwise unchanged.

## [2026-10-29] — Pipe instance connectors + elbow fittings (P8-S4b)

### Added
- **`OmniObject.instance_connectors`** (serde default): physical attach points with position, outward unit direction, bore (m) and an optional connection type code. Routed pipes create `start`/`end` connectors (bore = 2·radius).
- **Connector binding in `connect()`**: the nearest connector pair is selected, `from_point`/`to_point` are recorded (previously always empty) and `connection_detail` names the pair (`from:end;to:start`). Bore mismatch (> 1.5 mm) and type-code mismatch → `400`. Objects without instance connectors behave as before.
- **Elbow fittings**: each internal bend of a routed pipe creates a `Fitting` object (object type 307, parented to the pipe, `fitting_type: "elbow"`, bore) — placeholder sphere at 1.15× radius until real mitered/arc geometry lands.
- `Command::CreateBatch`: a routed pipe and its fittings are created as **one undo step** (`record_create_batch`), instead of N separate entries.

## [2026-10-29] — Pipe routing on the feature-tree path (P8-S4a)

### Added
- **`PathSweepSection`** (`Layered` | `Circular { radius }`) on `PathSweepParams` (serde default keeps old projects loading) and a `Circular` template step kind.
- **`Pipe-Path`** seeded ComponentDef + feature-tree template; routed `genrule: "pipe"` now instantiates it (`PathSweep` circular section) instead of building a loose mesh — the route remains recorded (`route_points`/`pipe_radius`/`route_length`), and the mesh is byte-identical to the `gen_rule::pipe_route` kernel (parity test).
- Straight pipes (no path) keep the legacy mesh builder; level-driven extent height adjustments only apply to layered sections.

## [2026-10-29] — Pipe routing (P8-S3c)

### Added
- **Routed pipes** (`POST /api/genrule`, `genrule: "pipe"` + `path`): a circular section is swept along the picked N-point polyline through the LinearComponent kernel — bends are continuous/mitered rather than disjoint cylinders. The route is recorded on the object (`route_points` JSON, `pipe_radius`, `route_length`), the object type is `PipePlant`.
- Validation: `< 2` points, a closed path, zero-length segments and `radius ≤ 0` all return `400` **before** the object is created (no orphans), and a series-selected routed pipe no longer falls into the straight catalog path.
- Legacy `pipe` (no path, `length` + `radius`) is unchanged.

### Still open
- Branches, fitting/elbow objects and a feature-tree pipe template remain out of scope; the routed pipe is a mesh-based Linear-family path (recorded transitional in CODE_DEVIATIONS §6.3).

## [2026-10-29] — Standalone openings (P8-S3b)

### Added
- **`ObjectType::Opening` (116)** mapped to `IfcOpeningElement` (import + export): a host-relative **void volume** created through `POST /api/genrule` (`genrule: "opening"`) with `wall_id`, `opening_width`, `opening_height`, `position_x/z`. The host must be an existing `Wall` (404/400 otherwise); the opening is parented to it and records `host_wall_id`. Created/deleted through the single write path (undo/redo covered).
- Web: `opening` appears in the building type list and reuses the door/window host form (target wall + opening size + offsets).

### Still open
- The host wall mesh is **not** re-cut (feature-tree geometry stays authoritative; same transitional behaviour as doors/windows). Recorded in the capability matrix.

## [2026-10-29] — Grid datum UI + viewport (P8-S3a)

### Added
- **轴网 panel** (left dock): list/add/edit/delete grid lines (`label`, axis family, coordinate, extents), wired to `/api/grids`; mutations trigger a scene reload.
- **Viewport rendering**: datum grid lines are drawn as coloured axis lines (numbered vs lettered families) with tag sprites at the extents; 0/0 extents render as a bounded ±20 m span. Purely a datum overlay — never part of picking or the mesh sync.

## [2026-10-29] — Level UI + IFC elevation export (P8-S3)

### Added
- **Level selectors in the creation panel**: walls/slabs/floors/roofs offer bottom/top storey selects (options from `GET /api/objects`, sorted by elevation, labelled with metres). Values flow into the `layered` genrule and the generic `POST /api/objects` body; the pair is client-validated (both-or-neither, distinct) and server-validated (ordering).
- **IFC elevation export**: `IFCBUILDINGSTOREY` is written schema-shaped (10 attributes) so the native `Elevation` attribute carries `OmniObject.elevation` (`$` when unset); export stays importable.

### Still open
- Catalog `Component` (Extrude-feature) placements are not re-evaluated from level constraints; plain box geometry still gets the datum only.

## [2026-10-29] — Level-driven vertical extent (P8-S2)

### Added
- **`bottom_level_id` / `top_level_id`** on `OmniObject` (both-or-neither): the base resolves to the bottom storey's elevation and, for path-driven (`PathSweep`) geometry, the path base shifts onto that level and the sweep height becomes `top − bottom`. Top must be above bottom; a storey without an elevation counts as 0 m.
- **Storey-elevation cascade**: changing a storey's `elevation` re-resolves every constrained object inside **one undo step** (`Command::ModifyObject.cascade` carries before/after images; redo is symmetric).
- API: `POST`/`PUT /api/objects` accept the extent refs; the `layered` genrule variant validates them **before** instantiating (no orphan objects on rejection).
- Deleting a storey still referenced by `level_id`/`bottom_level_id`/`top_level_id` is rejected with **400** ("detach them first").

### Fixed
- **Layered genrule + selected series** no longer 400s with "series not supported for this type": the layered family always takes the path-driven template route and carries the series id as metadata.

### Still open
- Plain-geometry objects receive the `elevation` datum only (mesh not rewritten); catalog `Component` (Extrude-feature) placements are not re-evaluated from level constraints yet. IFC export of the elevation value remains planned.

## [2026-10-29] — Building datum: storey elevation & grid (P8-S1)

### Added
- **Storey datum elevation**: `OmniObject.elevation` (optional, metres, IfcBuildingStorey.Elevation) on create/update; the web CreatePanel takes an elevation input for `storey` (negatives allowed for basements).
- **Level attachment**: `OmniObject.level_id` — objects can attach to an existing `BuildingStorey`; the omcom validator rejects unknown ids and non-storey targets (400 with the reason). `top_level_id`/`bottom_level_id` auto-adjust remains planned (DATA_MODEL §5.8 note).
- **Grid datum** (`Project.grids`, new `crates/model/src/grid.rs`): grid lines are datum annotations, not objects — `POST/GET /api/grids`, `PUT/DELETE /api/grids/{id}`, labels unique per axis family (`along_x` numbered / `along_y` lettered), sorted listing, undo/redo via three new ops (omcom oneof 74–76, actions 231–233) and persistence with the project (serde default keeps old `.omnim` files loading).
- **Web**: storey elevation field + validation; `createInitialForm('storey')` now defaults `IfcBuildingStorey` (was `IfcWall`) and elevation `0`.

### Fixed
- **`ModifyObject` undo now covers every modifiable field** — a Wave 1 gap left `ifc_class`/`tags` edits unrestorable, and redo was a no-op; the command now carries an after-image and undo/redo are symmetric (P8 fields ride along).
- **`POST /api/objects` returns 400 with the validator's reason** on validation failure (previously a bare 500).
- Docs: capability matrix "78 routes" line was stale pre-Wave-1 — route counts now consistent at **83** (API.md, matrix, both languages).

## [2026-10-28] — Series & connection enforcement (Wave 4)

### Added
- **Series gate enforces specs** (audit P0-5): when a series is loaded and the component is reachable, the located `SpecComponent` enforces `ParamConstraint` (`fixed` equality, `min`/`max` bounds → 400 with the reason) and applies `SpecComponent.material` to the instance (id → case-insensitive name; unresolvable → 400 listing available materials).
- **`connect()` hardening**: rejects self-connections and duplicate pairs, accepts `?connection_type=` (structural default), and checks connector `connection_type_code` compatibility between catalog instances.
- **`ConnectorDef.up_x/y/z_expr`** (proto + model + `/api/catalog/{id}/connectors`): flange orientation fields, expression-resolved like position/direction.

### Fixed
- Seeded materials now include the stainless grades the seeded piping specs reference (`A312-TP304/TP316`) — the enforcement turned a latent seed gap into a hard failure, so the gap is fixed at the source.

### Still open
- Resolved-value orientation checks at connect time, `step` enforcement, `from_point`/`to_point` population (CODE_DEVIATIONS §6.4).

## [2026-10-28] — Geometry-path convergence (Wave 3, layered family)

### Added
- **`FeatureType::PathSweep`** feature + `gen_rule::path_sweep` façade over the LayerExtrusion kernel (the class-agnostic constant-section primitive, DATA_MODEL §1 governance 3). Exact-mesh parity tests against the legacy builders (straight, bent, closed).
- **Path-driven templates** (`Wall-Path` / `Slab-Path` / `Floor-Path` / `Roof-Path`): one `layered_path_sweep` step; the picked path and the LayerEditor layer stack arrive via `TemplateContext` (per-instance inputs a static profile template cannot express).
- **`Project::instantiate_domain_object`**: feature-tree instantiation that produces the **domain object type** (`Wall`/`Slab`/`Floor`/`Roof`) with `catalog_ref` + `param_values` — previously only `ObjectType::Component` was reachable from the tree.
- The genrule **layered** family now instantiates through those templates: same kernel, same type, same traceability from every entry point. Path validation errors (zero-length, degenerate area, self-intersection) fail the instantiation with the kernel reason.

### Still transitional (documented)
- GenRule rect quick-forms (`wall`/`slab` variants without a path), roof 2-point profiles, and the catalog's extrude-based pipe/road templates (LinearComponent convergence is a later slice).

## [2026-10-28] — Path contract & genesis-form authorization (Wave 2)

### Added
- **`path { points, closed }` contract** on `/api/genrule` (`layered`, `road`, `line`); legacy `path[]`/`boundary[]`/`alignment[]` aliases normalize with their historical closure; ambiguous double-spelling returns 400.
- **`(objectType, form)` authorization matrix** (`gen_rule::matrix`, DATA_MODEL §1 governance 4): authorized rows wall×panel/region, slab/floor/roof×region, roof×panel (2-point profile), road×linear, line×point; everything else returns 400 with the contract reason. Created objects record `genesis_class`/`genesis_form` properties.
- **Shared pure-geometry layer** (`gen_rule::path_geom`): `iter_segments` / `merge_collinear` / `validate_path` dispatch by open/closed — below classification, no class semantics (§6.4 boundary note is now a module-header contract on both engines).
- **Frontend**: `CreationForm.form` field (wall panel/region dropdown, roof sub-modes feed it), single `form → pickMode` lookup, `withGenesisForm` helper; unified `addPathPreview` (region → outline+fill, panel → layered ghost with miter, linear/point → guide outline).

### Tests
- `genrule_contract.rs` (9 seed cases: contract/alias/region/profile/matrix-400/closed-rejections/ambiguity/bowtie), `path_geom` + `matrix` unit tests, web `genesisForm.test.ts` (10 cases). Full suite: Rust 385, web 69.

## [2026-10-28] — Single write path & persistence (Wave 1)

### Changed
- **Single write path restored** (audit P0-2): every mutating REST handler now builds an `OmniCommand` and runs it through `Validator` + `Executor`, or delegates to `Executor` while holding the lock. Direct `Project` writes went **42 → 0**; new ops cover materials, sketch/component/catalog detail, series, lifecycle, measurements, administrative/breakdown, catalog instantiation and `load_series`.
- **Persistence defaults on** (audit P0-3): mutations save to `$OMNIM_PROJECT` (default `project.omnim`); `OMNIM_EPHEMERAL=1` disables writes. `Project::save` is atomic (`{path}.tmp` → rename, parent dir auto-created). New route `POST /api/project/save` (79 deduplicated routes).
- **Undo/redo coverage is complete**: endpoint-level tests in `crates/server/tests/write_paths.rs` and `crates/server/tests/instantiate_import.rs` cover objects/properties/materials, sketches, components, catalog definitions, series, lifecycle, measurements, administrative/breakdown, instantiation and IFC import. Fixed pre-existing broken undo/redo for connection delete, lifecycle records and record links.
- **IFC import is one transaction**: `Command::ReplaceProject { before, after }` — undo restores the pre-import project, redo re-applies. **Catalog preview is read-only** (built on a throwaway project), and catalog instantiation records the object plus generated sketches as one command.
- **CI guards**: `scripts/check_write_paths.py` (0 direct writes required) and `scripts/check_docs.py` run in CI; `scripts/test-all.sh` now runs both plus `cargo test` and web unit tests.

### Fixed
- Documented exceptions: `section_handler` (cache-only write lock), `import_ifc` (single whole-project command), `POST /api/catalog/import` (session-global `LIBRARY`, not project state). See [WRITE_PATHS](../architecture/WRITE_PATHS.md).

## [2026-10-28] — Git branching & workflow policy

### Added
- Branching policy: `main` (release baseline) + `develop` (staging, pushed to `origin/develop`); short-lived `<type>/<phase>-<slug>` branches; docs-first split (`docs/*` before `feat/*` for new specifications). See [BRANCHING](../guides/BRANCHING.md).
- Commit convention (Conventional Commits, scopes from crate/area names) and merge policy (squash into `develop`, merge-commit `release/*` → `main`, immediate `main` → `develop` back-merge after every release/hotfix).
- Phase-style annotated tags (`phase-7` baseline, `phase-8`, `phase-8.1`, `phase-8-rc.1`); the project still does not use SemVer.
- Tooling: `.github/workflows/ci.yml` (fmt/clippy/test/web build/docs check), PR base/head guard (`scripts/ci/branch_guard.py`), PR template, `.gitattributes`, local hooks (`.githooks/commit-msg`, `.githooks/pre-push`), commit template (`.gitmessage`), one-shot setup (`scripts/setup-git.sh`).

## [2026-10-27] — Documentation reorganization & dead-code cleanup

### Added
- Reorganized `docs/` into `architecture/`, `guides/`, `specification/`, `reference/`, `status/`, `RFC/` (bilingual en/zh per file), with a docs portal (`docs/README.md`).
- New reference: full REST API reference (`reference/API.md`, **78** deduplicated routes audited against `crates/server/src/lib.rs`).
- New specifications: Component Catalog & Series Library (`specification/CATALOG_SERIES.md`), Data Flow, Architecture Decision Records (6 seed ADRs).
- New guides: Getting Started, Building, Architecture Map, Testing.
- RFC-0001: Omcom Text DSL (Draft) — captures the former "text DSL" vision, now correctly framed as unimplemented.
- End-to-end CHANGELOG (this file).
- N-point picking geometry: preserved picked elevation in layered wall/slab generation (`base_z`), roof extrusion-profile builder, road polyline alignment (`Road.alignment`), bend-corner mitering, boundary validation (area/self-intersection/collinearity) with 400 errors, collinear-point merging; +9 kernel tests.
- Web interaction polish: Backspace removes the last picked point during picking, count-aware four-mode picking hints, polygon loop-close highlight segment; road creation submits picked alignment; +6 state-machine tests.
- Picking preview overhaul: whole-path layered wall ghost preview (per-segment boxes + bend miter matching the server kernel, correct mm→m scale — fixes a 20× thickness error), hover endpoint resolved through the same snap pipeline as clicks, lit translucent materials with no depth-write, invalid polygon footprints (self-intersecting/degenerate) preview in red; preview lifecycle split into confirmed markers/chain vs per-frame hover groups; +10 preview-geometry tests.

### Changed
- `PLATFORM_ARCH.md` rewritten from a cloud-first aspirational vision to an honest description of the current implementation (REST + JSON-file persistence, no WebSocket/DB/WASM/mobile).
- `FORMAT_COMPAT.md` corrected: IFC import/export is partial (hand-written parser); DXF/DWG/STEP-CAD/GIS/glTF are not implemented.
- `ROADMAP.md` statuses corrected (Phase 1 text-DSL claim restated as RFC-0001; evidence lines added per phase).
- `DATA_MODEL.md` audited against code: ~25 stale/false claims fixed (e.g. implemented Torus/Dish primitives, real FeatureType enum, prototype-vs-proto divergences marked 🗓 Planned).
- `OMCOM_REFERENCE.md` rewritten: was a reference for a fictional text DSL; now documents the real typed OmniCommand system (proto/omcom.proto + crates/omcom Validator/Executor).

### Removed
- `crates/renderer` — orphan wgpu/winit demo crate (no dependents); removed from the workspace. Rendering is three.js in the browser; a native viewer would be re-proposed via RFC.
- Unused dependencies: `prost-types` (core), `tower` (server), `omnim-omcom` (amir).
- Unimplemented Action enum values (~62/80/81/82) annotated as reserved in `proto/omcom.proto` (values kept, ids not reused).
- Dead frontend components: `CatalogBrowser.tsx`, `SeriesBrowser.tsx`, `OmcomConsole.tsx`, `ViewCube.tsx` (superseded by OmniPanel / ViewCube3D).
- Repo junk: stray web helper scripts (`fix_delim*.sh`, `test.sh`, committed `web/dist/`), stale `scripts/update_roadmap.py`, debug `eprintln!`/`console.log`.
- Repo-root test fixtures relocated to `fixtures/ifc/` (docs updated); `logo.png` moved to `docs/img/` and embedded in both READMEs.

### Fixed
- Compiler warnings reduced to zero across the workspace (unused vars/imports, needless `mut`).
- All intradocumentation links verified (0 broken).
- Audit remediation (docs/status/DOC_AUDIT_2026-10-27): quickstart commands corrected (real console syntax, `/api/undo`, raw-text IFC import) plus an explicit persistence warning; README vision/current-state separation; IFC_EXTENSION rewritten to match the partial-IFC reality; ADR-0006 marked "Accepted (target) — Partially implemented" with a deviation note; new `architecture/WRITE_PATHS.md`; route count corrected to 78; CODE_DEVIATIONS §6 records the audit P0s; Chinese DATA_MODEL aligned (+371 lines: §5.6, §5.7.1, §16.4, §16a, §17, renumbering, fence fixes); DOMAINS/GLOSSARY carry proto-status marks; new `CAPABILITY_MATRIX` and `MODELER_HANDBOOK`; front matter on all 71 docs; `scripts/check_docs.py` (links/fences/numbering/bilingual parity) added. Code-level P0s (single write path, save policy, geometry convergence, series enforcement) are recorded in the ROADMAP technical-debt backlog.

### Not implemented (known gaps)
- Unchanged: REST-only (no WebSocket/real-time collaboration), JSON-file persistence (no DB/version history), no WASM shared core, no desktop/mobile packaging (see `PLATFORM_ARCH` future-directions section).

---

> **About batch labels:** the historical entries below keep the commit labels used at the time. `P8-Sx` is a slice inside batch 8; `P9-Sx` is a slice inside the frontend-productization batch 7b (**not** Phase 9 / AMIR). Calendar targets are no longer maintained; see the [roadmap](ROADMAP.md) for current status, gates and dependencies.

## [Phase 8 — In Progress] (batch label)

### Added
- Production-grade catalog data: steel sections (IPE/HEA/HEB/CHS/SHS per EN 10365, GB/T 706), pipe schedules (CS150, SS300 / ASME B36.10), concrete wall/beam/column/slab templates, connector definitions with flange ratings.
- Material assignments mapped to real standards (EN 10025, GB/T 1591, ASTM A36); BRep correctness validation of component templates.

### Not implemented (known gaps)
- Domain operations (MEP, earthwork/alignment, pipe routing, joints), quantity take-off, domain-specific UI panels.

### Fixed
- See [CODE_DEVIATIONS.md §1/§2](CODE_DEVIATIONS.md): 4-class component taxonomy and mm-vs-metre catalog scaling — **both RESOLVED 2026-08-11**.

---

## [Phase 7 — User Interface] (batch label) — Completed

### Added
- Project Workspace: three.js viewer (orbit/pan/zoom, selection, section cuts, measurement, wireframe/shaded/x-ray/edge view modes), feature tree, Available Components Panel (series-filtered, sliders, drag-to-place), property panel, Omcom Console (syntax highlighting, history, tab completion), AI chat panel.
- Library Manager (`/library-manager`): ComponentDef tree, parameter editor, feature-template assignment, 3D preview, `.omnimlib` import/export.
- SeriesStore Manager (`/series-manager`): decision-tree editor, seed viewer, `.omnimseries` load/save, drag-to-map, constraint editor.
- UI architecture: Zustand, React Query, PanelRegistry, resizable AppLayout, CSS Modules.

### Not implemented (known gaps)
- N-point polygon/polyline in-scene picking (see CODE_DEVIATIONS §3/§5); AI command preview/accept workflow (deferred to Phase 9, partial).

---

## [Phase 6 / 6b — GenRule Refactor & Library Infrastructure] (batch label) — Completed

### Added
- Library ↔ SeriesStore ↔ Project three-way split; Library as session-level global static; project-private SeriesStore + `external_series_refs`.
- Series gate (`check_series_gate`), GenRule → catalog/series instantiation, `CatalogRef` on instances, default-template registry, parameter validation against catalog domains.
- `.omnimlib` / `.omnimseries` external file formats, series resolution priority, session auto-load.

### Changed
- `generate_geometry()` deprecated in favor of the feature engine; `POST /api/genrule` now routes through the catalog/series system.

---

## [Phase 5 — Catalog Component Modeling] (batch label) — Completed

### Added
- 20+ parameterized catalog components across structural (IPE/HEA/HEB/CHS/SHS/Angle), piping (pipe, elbow, tee, reducers, flange, valves), and building (concrete/door/window) families; series definitions CS150, SS300, Concrete-C30, Steel-Q355, Highway-GB.
- Profile section library, ASME B36.10M wall-thickness table, FeatureTemplate + FeatureTreeRegistry, instantiation pipeline, REST `/api/catalog/instantiate` and `/api/catalog/preview`.

---

## [Phase 4 — Component Catalog & Series Library] (batch label) — Completed

### Added
- PARAGON-style component catalog (ComponentDef/ParamDef/ConnectorDef, Category hierarchy) and SPECON-style series library (selector decision trees, SpecComponent constraints, numeric-range matching), selection engine, connector type-code compatibility, external file storage.

---

## [Phase 3 — Feature Engine & BRep Kernel] (batch label) — Completed

### Added
- Feature as first-class entity (cached MeshGeometry, dirty tracking, rebuild pipeline, rollback/suppression); 9 feature types (Extrude, Revolve, Sweep, Loft, Boolean, Chamfer, Fillet, Pattern, Shell).
- truck-BRep integration (solid extrude/revolve, sweep along path, loft, booleans on BRep solids), tessellation to meshes, parameter system with meval expressions.
- Revit-style Component Editor UI (feature panel/ribbon, auto-sketch workflow, drag-reorder, Rebuild All).

---

## [Phase 2 — Constraint Solver & Sketch] (batch label) — Completed

### Added
- Sketch data model (SketchPlane, entities, 11 constraint types, SketchStore); slvs-based constraint solver with DOF analysis and over/under-constraint reporting; sketch/feature/component commands with undo/redo; standalone Sketch Editor web page; `EXTRUDE_SKETCH` / `REVOLVE_SKETCH` to 3D meshes.

---

## [Phase 1 — Core Data Model & Command Layer — Partial] (batch label)

### Added
- Unified data model (OmniObject hierarchy, properties/property sets, materials, ObjectType across 4 domains), project lifecycle & breakdown structures, administrative-division hierarchy, relationship system.
- `OmniCommand` typed schema used purely as serde-JSON types (no protobuf wire runtime) and a complete typed command executor with full undo/redo.

### Not implemented (known gaps)
- **Omcom text DSL / nom parser** — the Web UI console executes JSON commands plus a few ad-hoc shortcuts. Text DSL is a draft RFC: [RFC-0001](../RFC/README.md).

---

## Baseline (current runtime)

- Axum REST server (**78** deduplicated routes) with JSON-file (`.omnim`) persistence; **no WebSocket/real-time collaboration, no database/version history/audit trail**.
- AMIR AI engine (OpenAI/Anthropic/Mock providers): NL→OmniCommand translation, chat, reverse model description.
- Hand-written IFC (2x3/4) import/export with **partial entity coverage**; no STEP/DXF/GIS/glTF support.
- Web UI: three.js-based workspace, sketch editor, component editor, series/library managers, AI chat, Omcom console.
