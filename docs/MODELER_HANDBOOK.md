---
title: "OmniM Modeler Handbook"
status: current
last_updated: 2026-10-30
applies_to: "web/, crates/server, crates/model, crates/ifc"
evidence: "task flows checked against web/src/app/components/{MainToolbar,create,viewer}, web/src/app/{App,ProjectsPage,TitleBar}.tsx, docs/reference/API.md and docs/guides/GETTING_STARTED.md 2026-10-30"
---
# OmniM Modeler Handbook
A task-oriented guide for the current web app and REST API. Every workflow below is executable **today**; the gaps are stated where they occur rather than hidden. For a row-by-row capability view, use [CAPABILITY_MATRIX](CAPABILITY_MATRIX.md).

## 1. Before You Start
### 1.1 Units, and why two of them
- **Viewport, picks, GenRule and transforms are in metres.** A wall picked from (0,0,0) to (5,0,0) is 5 m long. `POST /api/genrule` fields such as `length`, `width`, `height`, `radius`, `alignment` and `boundary` are metres.
- **Catalog `ComponentDef` parameters are in millimetres.** A wall definition's `length` is `5000`, not `5`. Catalog meshes are scaled by `1/1000` for display (`crates/server/src/lib.rs` `get_mesh`, `preview_catalog_component`); the stored model stays in mm.
- **The layer editor is in millimetres.** `LayerEditor` values such as `200` mean 200 mm; `MainToolbar.tsx` divides by 1000 before sending the `layered` GenRule, whose layer thicknesses are metres.
- **Catalog placement `position` is metres**, even though its `params` are mm (the server converts to mm internally); **Assembly GenRule dimensions are metres** (`/api/genrule` `type:"assembly"`), unlike catalog parameters.

Rule of thumb: if you type a number into a catalog parameter panel, think **mm**; if you pick in the viewport or call `/api/genrule`, think **m**.

### 1.2 Precision and geometry caveats
- `PUT /api/objects/{id}/transform` converts any geometry to a **Mesh** and rewrites it. It works on everything with geometry, but it flattens feature trees: after a move, the parametric features and catalog parameters no longer drive the shape. It **is undoable** (restores the pre-transform geometry).
- A wall `path` may carry different z values, but planar extrusion uses the **first point's z** as the base elevation; there is no per-point grading. Slab/roof/floor boundaries use the first boundary point's z.
- Bend corners are mitered in the kernel and mirrored in preview; this is a planar extrusion, not a full 3D sweep.
- Slab/floor/roof boundaries are validated server-side: fewer than 3 points, collinear or zero-length edges, degenerate area and bowtie self-intersection are rejected. Structural validity, overlaps and openings are not checked.
- Bridging and tunnel creation still use the **first and last picked points only**; `road` is the only civil type that consumes the full alignment.
- `POST /api/objects` accepts the string `"floor"` but maps it to `BuildingStorey` (`parse_type` in `crates/server/src/lib.rs`). To create a real Floor, use `/api/genrule` with `mode:"floor"`.

### 1.3 Persistence: read this before modeling
> ✅ **Persistence defaults on** (2026-10-28). The server writes `$OMNIM_PROJECT` (default `project.omnim`) after every executor-path mutation; `POST /api/project/save` forces a save at any time; `OMNIM_EPHEMERAL=1` disables persistence.

- **Saved**: create/delete object, `/api/genrule`, `/api/omcom`, undo/redo, sketch mutations, object rename, feature add/update/remove/rebuild.
- **Persisted**: everything else — transform/parent/connect, materials, measurements, administrative/breakdown, series, catalog definitions, catalog placement and IFC import.
- Safer workaround for a session: `GET /api/export/ifc` (or the toolbar **Export**) keeps a geometry snapshot, despite the IFC limits in §6.2.
- Undo covers every mutation endpoint (objects, properties, transform, parent/connection, materials, sketch, components, catalog, series, measurements, administrative/breakdown, instantiation, IFC import). The two documented exceptions live in [WRITE_PATHS](architecture/WRITE_PATHS.md): `.omnimlib` import into the global library (no project-level undo entry, library file is not the project) and the `section_handler` cache refresh (not a user mutation).

### 1.4 Snapping
- The viewport toolbar has a **Toggle snap** button (`IcoSnap`, title "Toggle snap"). Snapping is applied by `snapPoint()` on every pick and on the hover endpoint.
- Defaults: enabled, grid + vertex + endpoint on, edge and midpoint off, threshold **0.5 m**, grid **1.0 m**. Change points in `web/src/app/components/viewer/three/Snapping.ts` if you need a different threshold.
- Snapping is geometric only: it does not infer walls, levels or parametric relationships. Grid visibility is toggled with **G**.

### 1.5 Interface shell and shortcuts
- The current build opens the workspace by default and redirects to the `/projects` home (new/open/recent/samples live, 2026-10-29) when no session is open — see [DATA_INTEGRITY](specification/ui/DATA_INTEGRITY.md) §2 and [CODE_DEVIATIONS](status/CODE_DEVIATIONS.md) P9.
- The properties panel currently opens as a floating overlay over the viewport; the contract docks it on the right. See [CODE_DEVIATIONS §7](status/CODE_DEVIATIONS.md).
- Verified shortcuts today: `W`/`E` move/rotate, `F` fit, `H` home, `G` grid, `O` ortho, `X` space, `Del` delete, `Enter` finish picking, `Backspace` remove the last point, `Esc` cancel. Full reference: [KEYBOARD_MAP](reference/KEYBOARD_MAP.md).

## 2. Building
### 2.1 Create a wall (panel by default; region optional)
1. Start the server and web app ([GETTING_STARTED](guides/GETTING_STARTED.md)); open <http://localhost:5173>.
2. Click **Create** (defaults to wall) or **建筑** → **墙体**.
3. Pick the genesis form in the Create panel: **中心线（面板）** (default, polyline picks) or **封闭轮廓（块）** (closed polygon picks). The pick mode follows the form.
4. Click at least two points in the viewport (panel) or three boundary points (region). **Backspace** removes the last point, **Esc** cancels, **Enter** finishes.
5. In the Create panel, edit the layer stack in **LayerEditor** (thicknesses are mm; templates include exterior/interior/partition walls). Optionally pick a **系列** (series).
6. Click **创建**. The result is an object whose type is `Wall`, with a layered extrusion, plus `genesis_class`/`genesis_form` properties.
7. API equivalent: `POST /api/genrule` with `{"type":"layered","mode":"wall","height":3,"layers":[...],"path":{"points":[[x,y,z],…],"closed":false}}` (legacy bare `path[]` still accepted). The created object records `genesis_class`/`genesis_form`. See [API §GenRule](reference/API.md#genrule).

What you do **not** get: a WallType/compound-type object, wall joins, host propagation or openings as objects. The layer stack is also stored as a `layers` property for round-tripping, but it is not a reusable type.

### 2.2 Slab, floor and roof (Path closed, region form)
1. **Slab**: **建筑** → **楼板**. Click 3+ boundary points; close by returning to the first point or double-click/Enter.
2. **Roof**: **建筑** → **屋顶**, then choose **轮廓（多边形）** (footprint, ≥3 points) or **坡屋面（段）** (two points, profile extrusion) in 屋顶模式.
3. **Floor**: there is **no menu entry**. Use `POST /api/genrule` with `mode:"floor"` and a `boundary` array, or add a UI entry. Do not use `POST /api/objects` with `"floor"` (§1.2).
4. Set the layer stack in **LayerEditor** (mm), then **创建**.
5. API equivalent: `{"type":"layered","mode":"slab|floor|roof","boundary":[[x,y,z],...],"layers":[...]}`. A roof profile instead uses a 2-point `path` and `width`.

All three remain **region-form** surface objects: a closed boundary plus layers, not a swept solid, and without structural analysis, openings or level association.

### 2.3 Door and window (host wall required)
1. Create a **Wall** first (§2.1). Without one, the door/window panel shows “请先创建一面墙体”.
2. Click **Create** (or the object-type dropdown) and choose **门** or **窗**.
3. Pick the target wall from the dropdown (it lists objects whose tree type is `Wall`), set **洞口宽/洞口高**, **水平位** and **离地高/窗台高**.
4. Click **创建**. The door/window is parented to the wall, and the opening is boolean-subtracted from the wall's bounding box.
5. API: `POST /api/genrule` `{"type":"door","wall_id":"...","opening_width":1.0,"opening_height":2.1,"position_x":0,"position_z":0}`; an unknown `wall_id` returns 404.

Limits: the opening is cut from the wall **bbox**, not from the layered solid, and there is no opening object, frame type, sill/head parameter set, fire rating or schedule. The quick-action **门/窗** buttons in the 建筑 panel instantiate catalog `Door-Single`/`Window-Fixed` as **Component** objects — those are not hosted doors/windows.

### 2.4 Sketch → feature → BRep (the official precise path)
1. Click **+ Component** in the main toolbar. This creates a component and navigates to `/component-editor?component=<id>`.
2. Draw the profile with Line/Rectangle/Circle/Arc/Polyline tools in **2D-Top**.
3. Add constraints from the constraint toolbar; click **Solve** and check the Solved/Unsolved status.
4. Click a feature button in the ribbon (拉伸 Extrude, 旋转 Revolve, 扫掠 Sweep, 放样 Loft, 布尔 Boolean, 倒角 Chamfer, 圆角 Fillet, 阵列 Pattern, 抽壳 Shell). If the feature needs a profile, draw it and click **✓ Done**.
5. Set depth/angle in the dialog, then inspect the result in the **3D** tab. Feature add/update/remove and **Rebuild** use the truck BRep kernel.
6. API equivalent: `POST /api/sketches`, `POST /api/sketches/{id}/entities`, `/constraints`, `/solve`, then `POST /api/components/{id}/features` or `/api/sketches/{id}/extrude`.

This path creates real `Component` objects with a feature tree; it is not the same as the GenRule toolbar path. Its geometry is the most trustworthy, but the object type remains `Component`, not `Wall`/`Beam`.

### 2.5 Catalog placement (creates `Component` objects)
1. Click **Catalog** to open **Available Components** (series-filtered, parameter sliders).
2. Drag a definition into the viewport, or click **添加到模型**. A `Component` object is created with a `catalog_ref`.
3. Parameters in the catalog panel are **mm**; position is **m**. The **系列** selector narrows the list by domain.
4. `skip_series_gate` is true when no series is selected; with a series, the gate checks only reachability.

Domain panels (**建筑/土木/工厂/机械**) also offer one-click quick actions that bypass the series gate and instantiate catalog components. They create `Component` objects, so do not expect a semantic `Tank`/`Beam`/`Door` object from them.

### 2.6 What is not available in Building
- **Levels/elevation on storeys**: a base elevation exists — settable when creating a storey (`elevation` in metres, mapped to `IfcBuildingStorey.Elevation`, P8-S1); vertical extents cascade one step via `bottom_level_id`/`top_level_id` on path-driven objects (P8-S2); the floor view selector (P8-S4) filters meshes by level and places the section plane. Still missing: `level_id` attachment for ordinary geometry is API-only, and there is no grid/axis object.
- **Grid objects**: no grid/axis object type or UI; only the viewport grid.
- **Wall types**: no `WallType`/`FloorType`/`RoofType` library object; layers are edited per instance.
- **Opening objects**: doors/windows are objects, but the cut is a bbox boolean; there is no `Opening` element, sill/head constraints or opening schedule.
- **Stairs**: no stair generator; the button produces a primitive placeholder. **Ceiling/CurtainWall/Ramp** are planned-only. **Clash, quantity take-off, drawings/schedules**: no implementation at any layer.

Roadmap and gap ledger: [ROADMAP Phase 8](status/ROADMAP.md), [CODE_DEVIATIONS §3](status/CODE_DEVIATIONS.md).

## 3. Civil
### 3.1 Road with alignment (implemented)
1. Click **土木** → **道路**.
2. Pick the alignment points in plan; **Enter** completes. The server lays a road segment along each polyline segment, honoring **断面宽** and **断面高** (metres).
3. Optionally select a series; for roads the series is metadata/attribution, not a corridor template.
4. Click **创建**, or call `POST /api/genrule` with `{"type":"road","alignment":[[x,y,z],...],"width":7,"thickness":0.3}`.

Roads are per-segment extrusions. There is **no vertical profile, superelevation, corridor assembly, terrain target, lane/shoulder/pavement-layer model or stationing**. Those remain Phase 8 work ([ROADMAP](status/ROADMAP.md)).

### 3.2 Feature lines and contour lines
1. **土木** → **特征线** or **等高线**.
2. Pick N points (Path open); **Enter** completes.
3. The server generates a thin guide tube (fixed Ø0.05 m), not a TIN breakline or constant-elevation contour.

These types are transitional: proto IDs 501/502 exist and the toolbar works, but native Point-class/TIN geometry is still design intent ([DATA_MODEL §1](specification/DATA_MODEL.md#1-overview)).

### 3.3 Terrain, earthwork and the rest (planned)
- **TerrainModel/PointCloud/GradingSurface/Slope** have no ObjectType, UI or API creation path; **cut/fill/earthwork balance** have no implementation.
- **Alignment/Profile/Station/Lane/Shoulder/PavementLayer/Median/Curb** are planned-only.
- **Bridge/Tunnel** currently create a two-point primitive placeholder; **RetainingWall/Dam/Culvert/Pier/Abutment** are API-only or planned.

See [ROADMAP Phase 8](status/ROADMAP.md) and [CODE_DEVIATIONS §3.2](status/CODE_DEVIATIONS.md).

## 4. Plant
### 4.1 Straight pipe segments and series selection
1. Click **工厂** → **管道**, or **Create** and choose **pipe**.
2. Pick start and end (two picks). The catalog path uses the selected `ComponentDef` and its **mm** parameters; the GenRule path uses `radius`/`length` in **m**.
3. Optionally choose a **系列**. The selector filters by domain; the server gate only checks that the component is reachable from a loaded series.
4. Click **创建**. The catalog path creates a `Component`; the GenRule path creates a `Pipe` with a straight cylinder mesh.

Limits: straight segments only — **no elbows, tees, reducers, flanges, branches, supports or pipe-run topology**. `POST /api/objects/{from}/connect/{to}` creates a structural connection but only checks that both objects exist; it does not validate connectors, bore, type codes or pressure rating, and writes no connection points. Series materials are **not applied** (`SpecComponent.material` is ignored). See [CODE_DEVIATIONS §6.4](status/CODE_DEVIATIONS.md).

### 4.2 Equipment placement
- **工厂** quick actions provide vertical tank, centrifugal pump, gate valve, heat exchanger and compressor; each instantiates a catalog definition as a **Component** object.
- Reactor, instrument, structural steel and fitting are in proto but have **no UI**; create them through `/api/objects` or `/api/catalog/instantiate` if you need a placeholder.

### 4.3 Routing and connections (partial/planned)
- Piping **routing, branches, elbows, pipe specs and isometrics** are Phase 8 items ([ROADMAP](status/ROADMAP.md)).
- Connections are API-only and unvalidated (§4.1). There is no P&ID or system topology.
- `ConnectorDef` stores only a direction; there is no up-axis, so flange bolt-hole or valve-stem orientation cannot be expressed ([CODE_DEVIATIONS §6.4](status/CODE_DEVIATIONS.md)).

## 5. Mechanical
### 5.1 Parts
1. **Create** → object type **part**, choose Box/Cylinder/Sphere/Cone and dimensions (metres), then **创建**. This is a primitive placeholder.
2. For a real part, use the catalog path (§2.5) or the component editor (§2.4): sketch → constrain → solve → feature → BRep.
3. `POST /api/objects` with `"part"` creates a `Part`; catalog instantiation creates a `Component`. Pick the path that matches what you need downstream.

### 5.2 Assemblies (container + parts, 2026-10-30)
1. Click **Assembly** in the main toolbar to open **AssemblyPanel**.
2. Name the assembly, optionally pick a parent, then add parts one of two ways:
   - **Catalog part**: pick a `ComponentDef`, edit its parameters (mm), set position (m) and Z rotation (deg);
   - **Primitive part**: box/cylinder/sphere/cone with dimensions and position (m).
3. Click **创建 Assembly**. The result is an `Assembly` **container object** with its parts as ordinary child objects: the tree lets you select, delete and export them individually; part poses are baked into geometry at creation.
4. API: `POST /api/assemblies` (e.g. `{"name":"...","parts":[{"name":"base","shape":"box","p1":2,"p2":1,"p3":0.2,"position":{"x":0,"y":0,"z":0}},{"name":"pump","component_def_id":"<ComponentDef id>","params":{...},"position":{"x":0,"y":0,"z":0.3},"rotation_z":0}]}`); `skip_series_gate` optional. Creation is one undo step.
5. Moving the container cascades to all parts (one undo); deleting it removes the whole subtree by default (`DELETE /api/objects/{id}` cascades for containers; other objects still promote children).
6. The legacy `POST /api/genrule` `{"type":"assembly",...}` path still works but returns a **merged mesh** (no structure), for scripts/backward compatibility and built-in equipment types (Stair/AHU/Chiller/Transformer/Reactor).

Limits: parts have no live placement (poses are baked), no BOM/quantity endpoint, no kinematic joints or constrained assemblies; nested assemblies are not exposed. Contract: [ASSEMBLY](specification/ASSEMBLY.md).

### 5.3 Kinematics and manufacturing (planned)
- **Joints, constraints, axes, kinematic pairs and actuators** are planned-only; no mate solver or motion study. **Fasteners** (bolt/nut/washer/screw/rivet/pin) and the manufacturing object types (hole/pocket/slot/thread/PMI) are planned-only.
- Generic **chamfer/fillet features** do exist in the component feature tree; that is not the same as the planned manufacturing object model.

See [ROADMAP Phase 8](status/ROADMAP.md) and [CODE_DEVIATIONS](status/CODE_DEVIATIONS.md).

## 6. Data and Exchange
### 6.1 Save and load `.omnim`
- The project is one JSON document loaded at startup from `OMNIM_PROJECT` (default `project.omnim`). Every mutation endpoint auto-saves (`OMNIM_EPHEMERAL=1` disables); force a save any time with `POST /api/project/save`, or use the title-bar **Save / Save as** buttons (shortcut see [KEYBOARD_MAP](reference/KEYBOARD_MAP.md)).
- Session habit: run with `OMNIM_PROJECT=./dev-project.omnim`; everything persists automatically — `POST /api/project/save` is equally available for an explicit flush. Verify the file timestamp before closing.
- Undo does not protect you across a restart: the undo stack is in-memory only.

### 6.2 IFC import and export
- Toolbar **Import** / **Export** map to `POST /api/import/ifc` and `GET /api/export/ifc`.
- Import takes **raw IFC text** (not multipart). Query options: `normalize` (default true) and `mode` (`merge` default | `replace`). Parse failures return HTTP 200 with `success:false`.
- Supported input is an **IFC 2x3 / IFC 4 subset** with a hand-written parser: Wall/Beam/Column/Slab/Door/Window/Stair/Roof/Road/Bridge/Building/Storey/Site/Space plus Generic mappings. Materials, quantities, classifications and advanced geometry are not mapped.
- Export writes mapping-based entities; **Stair and Roof export as `IfcBuildingElementProxy`**, as does every non-building type. Round-trip fidelity is not guaranteed.
- No STEP, DXF/DWG, GIS, glTF, OBJ/FBX/STL. See [FORMAT_COMPAT](status/FORMAT_COMPAT.md).

### 6.3 Library and series files
- **Library Manager** at `/library-manager` edits `ComponentDef`s; the global library is `~/.omnim/library.omnimlib`. API: `POST /api/catalog/export` writes a timestamped path under `/tmp`; `POST /api/catalog/import` reads a path you supply.
- **Series Manager** at `/series-manager` edits decision trees; API: `POST /api/series/save` / `POST /api/series/load` for `.omnimseries` files. These endpoints return/accept a filesystem path; they do not choose a project-relative location.
- Library and series **definitions** are undoable and persist with the project; repository-level `.omnimlib` / `.omnimseries` export still needs an explicit file export.

### 6.4 Measurements and sections
- Distance/angle/area measurement buttons live in the viewport toolbar; area finishes with right-click. Measurements are stored in `Project.measurements` via `POST /api/measurements`; they are undoable and persisted with the project.
- **Section cut** is a view-only clip plane (X/Y/Z slider). `GET /api/components/{id}/section?plane_z=` returns section lines for a `Component`; there is no drawing/sheet output.

For the complete status picture, read [ROADMAP](status/ROADMAP.md), [FORMAT_COMPAT](status/FORMAT_COMPAT.md) and [CODE_DEVIATIONS](status/CODE_DEVIATIONS.md) before promising a deliverable.
