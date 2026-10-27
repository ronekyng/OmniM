# OmniM Glossary

This document defines all terminology used in the OmniM system with Chinese translations, descriptions, and code references.

---

## 1. Core Hierarchy

OmniM uses a four-layer object hierarchy:

| Layer | Chinese | English | Description | Code Reference |
|-------|---------|---------|-------------|----------------|
| Object | 对象 | OmniObject | Base class for all entities including buildings, equipment, parts, etc. | `web/src/shared/types.ts:OmniObject` |
| Component | 构件 | Component | A 3D geometric body composed of one or more features; the geometric representation of an Object | `crates/model/src/lib.rs:ComponentDef` |
| Feature | 特征 | Feature | An operation that generates 3D geometry from a sketch (extrude, revolve, loft, etc.) | `crates/model/src/feature.rs:Feature` |
| Sketch | 草图 | Sketch | A 2D geometric profile containing entities and constraints; input for features | `crates/sketch/src/lib.rs:Sketch` |
| Entity | 图元 | SketchEntity | A single geometric element in a sketch (line, circle, rectangle, etc.) | `crates/sketch/src/lib.rs:SketchEntity` |
| Constraint | 约束 | SketchConstraint | A rule that constrains geometric relationships between entities (horizontal, vertical, tangent, etc.) | `crates/sketch/src/lib.rs:SketchConstraint` |

**Parent-child relationship:**
```
OmniObject
  └── ComponentDef
        ├── Sketch (profile)
        ├── Feature (extrude/revolve/loft/fusion, etc.)
        │     └── Sketch (input)
        └── Feature ...
```

**Series (系列)**: A collection of same-class objects, filtered by a selector decision tree.

---

## 2. Object Types

OmniM supports 50+ object types, grouped by engineering domain.

> **E3D element type codes cross-reference**: AVEVA E3D uses short element type codes (`docs/research/aveva_e3d_primitives.md §2.2`). These map onto OmniM ObjectTypes as reference for import/export:
>
> | E3D Code | Meaning | OmniM ObjectType |
> |----------|---------|------------------|
> | EQUI | Equipment | OBJECT_TYPE_TANK / PUMP / etc. |
> | SUBE | Sub-equipment | OBJECT_TYPE_* (sub-component) |
> | STRU | Structure | OBJECT_TYPE_STRUCTURAL_STEEL |
> | FRMW | Framework | OBJECT_TYPE_ASSEMBLY |
> | SBFR | Sub-framework | OBJECT_TYPE_ASSEMBLY |
> | GENSEC | Structural section (IPE300) | OBJECT_TYPE_STRUCTURAL_STEEL |
> | PANE | Panel | OBJECT_TYPE_* (Panel) |
> | PIPE | Pipe run | OBJECT_TYPE_PIPE_PLANT |
> | BRAN | Branch | OBJECT_TYPE_PIPE_PLANT (segment) |
> | NOZZ | Nozzle | OBJECT_TYPE_* (Nozzle) |

### 2.1 Spatial Elements

| Enum Value | Chinese | English | Description |
|------------|---------|---------|-------------|
| `OBJECT_TYPE_SITE` (1) | 场地 | Site | Project site |
| `OBJECT_TYPE_BUILDING` (2) | 建筑 | Building | Building |
| `OBJECT_TYPE_BUILDING_STOREY` (3) | 楼层 | Building Storey | Building floor |
| `OBJECT_TYPE_SPACE` (4) | 空间 | Space | Room/area |

### 2.2 Building Elements

| Enum Value | Chinese | English | Description |
|------------|---------|---------|-------------|
| `OBJECT_TYPE_WALL` (100) | 墙 | Wall | Building wall |
| `OBJECT_TYPE_BEAM` (101) | 梁 | Beam | Structural beam |
| `OBJECT_TYPE_COLUMN` (102) | 柱 | Column | Structural column |
| `OBJECT_TYPE_SLAB` (103) | 板 | Slab | Floor/slab |
| `OBJECT_TYPE_DOOR` (104) | 门 | Door | Building door |
| `OBJECT_TYPE_WINDOW` (105) | 窗 | Window | Building window |
| `OBJECT_TYPE_STAIR` (106) | 楼梯 | Stair | Building stair |
| `OBJECT_TYPE_ROOF` (107) | 屋顶 | Roof | Building roof |
| `OBJECT_TYPE_FLOOR` (108) | 地板 | Floor | Floor level |
| `OBJECT_TYPE_DUCT` (109) | 风管 | Duct | HVAC duct |
| `OBJECT_TYPE_CABLE_TRAY` (110) | 电缆桥架 | Cable Tray | Electrical cable tray |
| `OBJECT_TYPE_AIR_TERMINAL` (111) | 风口 | Air Terminal | HVAC diffuser/grille |
| `OBJECT_TYPE_AHU` (112) | 空调机组 | Air Handler (AHU) | Air handling unit |
| `OBJECT_TYPE_CHILLER` (113) | 冷水机 | Chiller | Chiller unit |
| `OBJECT_TYPE_TRANSFORMER` (114) | 变压器 | Transformer | Power transformer |
| `OBJECT_TYPE_LUMINAIRE` (115) | 灯具 | Luminaire | Lighting fixture |

### 2.3 Civil Elements

| Enum Value | Chinese | English | Description |
|------------|---------|---------|-------------|
| `OBJECT_TYPE_ROAD` (200) | 道路 | Road | Road/highway |
| `OBJECT_TYPE_BRIDGE` (201) | 桥梁 | Bridge | Bridge structure |
| `OBJECT_TYPE_TUNNEL` (202) | 隧道 | Tunnel | Tunnel structure |
| `OBJECT_TYPE_DAM` (203) | 大坝 | Dam | Dam structure |
| `OBJECT_TYPE_CULVERT` (204) | 涵洞 | Culvert | Drainage culvert |
| `OBJECT_TYPE_RETAINING_WALL` (205) | 挡土墙 | Retaining Wall | Retaining structure |
| `OBJECT_TYPE_PIPE_CIVIL` (206) | 管道(土木) | Pipe (Civil) | Municipal pipe |
| `OBJECT_TYPE_PIER` (207) | 桥墩 | Pier | Bridge pier |
| `OBJECT_TYPE_MANHOLE` (208) | 检查井 | Manhole | Manhole |
| `OBJECT_TYPE_ABUTMENT` (209) | 桥台 | Abutment | Bridge abutment |
| `OBJECT_TYPE_CUT_VOLUME` (220) | 挖方量 | Cut Volume | Excavation volume between two surfaces |
| `OBJECT_TYPE_FILL_VOLUME` (221) | 填方量 | Fill Volume | Fill volume between two surfaces |
| `OBJECT_TYPE_GRADING_SURFACE` (222) | 平整曲面 | Grading Surface | Designed graded surface |
| `OBJECT_TYPE_SLOPE` (223) | 边坡 | Slope | Embankment or cut slope |
| `OBJECT_TYPE_FEATURE_LINE` (224) | 特征线 | Feature Line | 3D breakline for grading control |
| `OBJECT_TYPE_CONTOUR_LINE` (225) | 等高线 | Contour Line | Elevation contour line |
| `OBJECT_TYPE_SITE_COMPONENT` (226) | 场地构件 | Site Component | Reusable site layout element |
| `OBJECT_TYPE_EARTHWORK_BALANCE` (227) | 土方平衡 | Earthwork Balance | Cut/fill balance analysis result |

### 2.4 Plant Elements

| Enum Value | Chinese | English | Description |
|------------|---------|---------|-------------|
| `OBJECT_TYPE_TANK` (300) | 储罐 | Tank | Process tank |
| `OBJECT_TYPE_PUMP` (301) | 泵 | Pump | Process pump |
| `OBJECT_TYPE_VALVE` (302) | 阀门 | Valve | Process valve |
| `OBJECT_TYPE_HEAT_EXCHANGER` (303) | 换热器 | Heat Exchanger | Heat exchange equipment |
| `OBJECT_TYPE_REACTOR` (304) | 反应器 | Reactor | Chemical reactor |
| `OBJECT_TYPE_PIPE_PLANT` (305) | 管道(工厂) | Pipe (Plant) | Process pipe |
| `OBJECT_TYPE_STRUCTURAL_STEEL` (306) | 钢结构 | Structural Steel | Plant structural steel |
| `OBJECT_TYPE_FITTING` (307) | 管件 | Pipe Fitting | Elbow/tee/reducer, etc. |
| `OBJECT_TYPE_COMPRESSOR` (308) | 压缩机 | Compressor | Gas compressor |
| `OBJECT_TYPE_INSTRUMENT` (309) | 仪表 | Instrument | Process instrument/sensor |
| `OBJECT_TYPE_CONTROL_VALVE` (310) | 调节阀 | Control Valve | Valve with actuator + positioner |
| `OBJECT_TYPE_CABLE` (311) | 电缆 | Cable | Power/signal/control cable |
| `OBJECT_TYPE_CONDUIT` (312) | 导管 | Conduit | Cable protection conduit |

> **Pipe fitting types** (E3D catalog, `docs/research/aveva_e3d_primitives.md §4.2`): ELBO (elbow), BEND (bend), TEE (tee), VALV (valve), FLAN (flange), GASK (gasket), REDUCER (reducer), VENT (vent), DRAIN (drain), CAP (cap), OLET (olet). These are parameterized catalog entries defined under `OBJECT_TYPE_FITTING`.

> **Control Valve details** (`docs/research/e3d_instrumentation_components.md §2.4.1`): 
> - **Actuator types**: Pneumatic Diaphragm, Pneumatic Piston, Electric, Solenoid, Manual Handwheel, Spring Return
> - **Failure modes**: FO (Fail Open), FC (Fail Close), FL (Fail Last Position)
> - **GenRule**: Assembly (valve body + actuator + positioner + accessories)

### 2.5 Mechanical Elements

| Enum Value | Chinese | English | Description |
|------------|---------|---------|-------------|
| `OBJECT_TYPE_PART` (400) | 零件 | Part | Mechanical part |
| `OBJECT_TYPE_ASSEMBLY` (401) | 组件 | Assembly | Mechanical assembly |
| `OBJECT_TYPE_FASTENER` (402) | 紧固件 | Fastener | Bolt/nut/washer |
| `OBJECT_TYPE_GEAR` (403) | 齿轮 | Gear | Transmission gear |
| `OBJECT_TYPE_SHAFT` (404) | 轴 | Shaft | Drive shaft |
| `OBJECT_TYPE_BEARING` (405) | 轴承 | Bearing | Support bearing |
| `OBJECT_TYPE_MOTOR` (406) | 电机 | Motor | Drive motor |
| `OBJECT_TYPE_SPRING` (407) | 弹簧 | Spring | Elastic element |
| `OBJECT_TYPE_SEAL` (408) | 密封件 | Seal | Sealing device |

### 2.6 Other

| Enum Value | Chinese | English | Description |
|------------|---------|---------|-------------|
| `OBJECT_TYPE_GENERIC` (999) | 通用 | Generic | Generic/uncategorized object |
| `OBJECT_TYPE_COMPONENT` (500) | 构件 | Component | Component definition (catalog) |

---

## 3. Feature Types

Features are operations that generate 3D geometry from sketches.

| Enum Value | Chinese | English | Description | Input Sketches |
|------------|---------|---------|-------------|----------------|
| `Extrude` (1) | 拉伸 | Extrude | Extrude a profile along a direction | 1 |
| `Revolve` (2) | 旋转 | Revolve | Revolve a profile around an axis | 1 (profile + axis) |
| `Sweep` (3) | 扫掠 | Sweep | Move a profile along a path | 1 (profile + path) |
| `Loft` (4) | 放样 | Loft | Connect multiple cross-sections | 2+ (sections) |
| `Boolean` (5) | 布尔 | Boolean | Union/subtract/intersect of two solids | — |
| `Chamfer` (6) | 倒角 | Chamfer | Edge chamfering | — |
| `Fillet` (7) | 圆角 | Fillet | Edge rounding | — |
| `Pattern` (8) | 阵列 | Pattern | Linear or circular replication | — |
| `Shell` (9) | 抽壳 | Shell | Convert solid to thin-wall | — |

> **Fillet sub-types** (from SOLIDWORKS, `docs/research/solidworks_components.md §3.3`): 8 fillet sub-types are supported:
> 1. Constant Size — uniform radius along edge
> 2. Multiple Radius — different radii per edge in one operation
> 3. Round Corner — smooth corner transition
> 4. Setback — variable radius at vertex (3 edges meeting)
> 5. Variable Size — radius varies along edge
> 6. Face — fillet an entire face
> 7. Full Round — three-face tangent fillet
> 8. Asymmetric — two different radii on adjacent faces

### Feature Parameters

| Feature | Parameter | Type | Default | Description |
|---------|-----------|------|---------|-------------|
| Extrude | `depth` | number | 1.0 | Extrusion depth |
| Extrude | `direction_x/y/z` | number | 0/0/1 | Extrusion direction vector |
| Revolve | `angle` | number | 0.0 | Rotation angle (radians, 0=360°) |
| Boolean | `operation` | select | subtract | union/subtract/intersect |
| Chamfer | `distance` | number | 0.1 | Chamfer distance |
| Fillet | `radius` | number | 0.1 | Fillet radius |
| Pattern | `pattern_type` | select | linear | linear/rectangular/circular |
| Pattern | `count` | number | 2 | Replication count |
| Pattern | `spacing` | number | 1.0 | Replication spacing |
| Shell | `wall_thickness` | number | 0.1 | Wall thickness |

### Cuttable Types

The following features support the `is_cut` toggle for creating holes/pockets:
`extrude`, `revolve`, `sweep`, `loft`

---

## 4. Sketch Entity Types

| Enum Value | Chinese | English | Description | Required Points |
|------------|---------|---------|-------------|-----------------|
| `SketchLine` | 线段 | Line | Straight line segment between two points | 2 |
| `SketchArc` | 圆弧 | Arc | Three-point arc | 3 |
| `SketchCircle` | 圆 | Circle | Center + radius | 1 (center) |
| `SketchPoint` | 点 | Point | Zero-dimensional geometric point | 1 |
| `SketchSpline` | 样条曲线 | Spline | Smooth curve through multiple control points | 2+ |
| `SketchSlot` | 槽口 | Slot | Slot with 3 variants: Straight/Arc/Conical | 2 |
| `SketchConic` | 圆锥曲线 | Conic | Endpoints + Rho value (elliptical/parabolic/hyperbolic) | 2 |
| `SketchParabola` | 抛物线 | Parabola | Vertex + focus offset + start/end | 4 |
| `SketchText` | 草图文字 | Text | 2D text on a curve/edge, extrudable/cuttable into 3D | 1 |
| `BeltChain` | 皮带/链 | Belt Chain | Layout sketch for pulleys or sprocket mechanisms | 2+ |
| `EquationCurve` | 方程曲线 | Equation Curve | Parametric / equation-driven curve | 2 |

### Entity Properties

| Property | Applies To | Description |
|----------|------------|-------------|
| `points` | All | Array of control point coordinates `{x, y, z}` |
| `radius` | Circle | Circle radius |

---

## 5. Sketch Constraint Types

| Enum Value | Chinese | English | Symbol | Description |
|------------|---------|---------|--------|-------------|
| `HORIZONTAL` | 水平 | Horizontal | H | Line or two-point line stays horizontal |
| `VERTICAL` | 垂直 | Vertical | V | Line or two-point line stays vertical |
| `COINCIDENT` | 重合 | Coincident | ⦿ | Two points coincide at same position |
| `PARALLEL` | 平行 | Parallel | ∥ | Two lines stay parallel |
| `PERPENDICULAR` | 正交 | Perpendicular | ⊥ | Two lines stay perpendicular |
| `LENGTH` | 长度 | Length | L= | Line length equals specified value |
| `DISTANCE` | 距离 | Distance | D= | Distance between two points equals specified value |
| `ANGLE` | 角度 | Angle | ∠ | Angle between two lines equals specified value |
| `RADIUS` | 半径 | Radius | R= | Circle/arc radius equals specified value |
| `DIAMETER` | 直径 | Diameter | Φ | Circle/arc diameter equals specified value |
| `TANGENT` | 相切 | Tangent | ⌒ | Two entities are tangent |
| `SYMMETRIC` | 对称 | Symmetric | \| | Two entities are symmetric about an axis |
| `FIXED` | 固定 | Fixed | ⚓ | Entity position/size is immutable |
| `MIDPOINT` | 中点 | Midpoint | ◉ | Point is at line midpoint |
| `OFFSET` | 偏移 | Offset | ↕ | Offset distance between two entities |
| `EQUAL_LENGTH` | 等长 | Equal Length | = | Two lines have equal length |
| `EQUAL_RADIUS` | 等半径 | Equal Radius | R= | Two circles/arcs have equal radius |
| `PT_ON_LINE` | 点在线上 | Point on Line | — | Point lies on a line |
| `PT_ON_CIRCLE` | 点在圆上 | Point on Circle | ○ | Point lies on a circle/arc |
| `SAME_ORIENTATION` | 同向 | Same Orientation | → | Two entities have same direction |
| `COLLINEAR` | 共线 | Collinear | — | Two lines share the same infinite line |
| `CORADIAL` | 同心等径 | Coradial | ◎ | Two arcs/circles share the same radius and center |
| `EQUAL_CURVATURE` | 等曲率 | Equal Curvature | G2 | Curvature continuity (G2) between splines |
| `EQUAL_CURVE_LENGTH` | 等曲线长度 | Equal Curve Length | L= | Two curves have equal length |
| `MERGE_POINTS` | 合并点 | Merge Points | ⊕ | Two endpoints merge into one |
| `PIERCE` | 穿透 | Pierce | ⊗ | A point lies on a curve (sketch entity passes through another) |
| `FIX_SLOT` | 固定槽口 | Fix Slot | ⊟ | Fix a slot entity's position and orientation |
| `ON_EDGE` | 在边上 | On Edge | — | Sketch entity lies on a model edge |
| `ON_PLANE` | 在平面上 | On Plane | — | Sketch entity lies on a reference plane |
| `ON_SURFACE` | 在曲面上 | On Surface | — | Sketch entity lies on a model surface |
| `TANGENT_TO_FACE` | 与面相切 | Tangent to Face | ⌒ | Sketch entity is tangent to a model face |
| `TRACTION` | 牵引 | Traction | → | Traction constraint for sweep guide curves |
| `TORSION_CONTINUITY` | 扭转连续性 | Torsion Continuity | G3 | G3 torsion continuity for sweep guide curves |

---

## 6. Sketch Terms

| Chinese | English | Description |
|---------|---------|-------------|
| 轮廓草图 | Profile Sketch | Closed sketch defining a solid cross-section |
| 轴线草图 | Axis Sketch | Rotation axis for revolve features |
| 截面草图 | Section Sketch | Cross-section for loft/fusion features |
| 路径草图 | Path Sketch | Guide path for sweep/lofted-fusion features |
| Z 高度 | Z Height / Origin Z | Z-axis offset of the sketch plane |
| 法线 | Normal | Sketch plane normal vector; (0,0,1) = XY plane |
| 原点 | Origin | Sketch coordinate system origin (x,y,z) |
| 闭合 | Closed | Profile endpoints connect forming a closed region |
| 开放 | Open | Profile endpoints do not connect |
| 链式绘制 | Chain Drawing | Continuous drawing mode; previous endpoint auto-becomes next startpoint |

### Multi-Sketch Labels

| Feature Type | Required Sketch Labels | Description |
|--------------|----------------------|-------------|
| revolve | 轮廓, 轴线 | Profile = cross-section, Axis = rotation axis |
| sweep | 轮廓, 路径 | Profile = cross-section, Path = guide line |
| loft | 截面1, 截面2 | Multiple sections to connect |
| fusion | 截面1, 截面2 | Two sections to transition |
| lofted_fusion | 截面1, 截面2, 路径 | Two sections transitioning along path |

---

## 7. 3D Geometry Types

**Reference Geometry** (from SOLIDWORKS, `docs/research/solidworks_components.md §9`): 7 types of reference geometry support assembly and feature construction:
1. **Plane** — construction plane for sketches and features
2. **Axis** — construction axis for revolve/pattern/assembly mates
3. **Coordinate System** — local coordinate system reference
4. **Reference Point** — point in 3D space for positioning
5. **Center of Mass Point** — dynamic COM point on the object
6. **Bounding Box** — oriented bounding box for fit/spatial analysis
7. **Grid System** — 2D grid layout for pattern/assembly placement

These are stored as `ReferenceGeometry` sub-objects within the Object's feature tree, alongside sketches and features.

| Enum Value | Chinese | English | Description |
|------------|---------|---------|-------------|
| `GEOMETRY_TYPE_PARAMETRIC` (1) | 参数化 | Parametric | Geometry defined by parameters and formulas |
| `GEOMETRY_TYPE_BREP` (2) | 边界表示 | B-Rep | Boundary representation solid |
| `GEOMETRY_TYPE_MESH` (3) | 网格 | Mesh | Triangle mesh representation |
| `GEOMETRY_TYPE_POINT_CLOUD` (4) | 点云 | Point Cloud | 3D point collection |
| `GEOMETRY_TYPE_CURVE` (5) | 曲线 | Curve | 3D curve |
| `GEOMETRY_TYPE_EXTERNAL` (6) | 外部引用 | External | External file reference |

### Unified Geometry Primitives

| Primitive | Chinese | Description |
|-----------|---------|-------------|
| Box | 长方体 | Six-faced solid primitive |
| Cylinder | 圆柱体 | Circular cross-section column |
| Cone | 圆锥体 | Circular cross-section cone |
| Sphere | 球体 | Spherical primitive |
| Extrusion | 拉伸体 | Solid formed by extruding a sketch |
| Sweep | 扫掠体 | Solid formed by sweeping a sketch along a path |
| Revolve | 旋转体 | Solid formed by revolving a sketch around an axis |
| Loft | 放样体 | Solid formed by connecting multiple cross-sections |

### Boolean Operations

| Enum Value | Chinese | English | Description |
|------------|---------|---------|-------------|
| `BOOLEAN_UNSPECIFIED` (0) | 未指定 | Unspecified | Default value (not a valid operation) |
| `Union` (1) | 并集 | Union | Merge two solids |
| `Subtract` (2) | 差集 | Subtract | Subtract one solid from another |
| `Intersect` (3) | 交集 | Intersect | Keep only the common volume |

---

## 8. Action Types

All actions are recorded in the OMCOM change log.

### Object Operations

| Action | Chinese | Description |
|--------|---------|-------------|
| `CREATE_OBJECT` | 创建对象 | Create a new OmniObject |
| `MODIFY_OBJECT` | 修改对象 | Modify object properties |
| `DELETE_OBJECT` | 删除对象 | Delete object |
| `COPY_OBJECT` | 复制对象 | Copy object |
| `MOVE_OBJECT` | 移动对象 | Move object position |

### Geometry Operations

| Action | Chinese | Description |
|--------|---------|-------------|
| `CREATE_GEOMETRY` | 创建几何 | Create new geometry |
| `MODIFY_GEOMETRY` | 修改几何 | Modify geometry parameters |
| `DELETE_GEOMETRY` | 删除几何 | Delete geometry |
| `TRANSFORM_GEOMETRY` | 变换几何 | Translate/rotate/scale |
| `BOOLEAN_OPERATION` | 布尔运算 | Union/subtract/intersect |

### Property Operations

| Action | Chinese | Description |
|--------|---------|-------------|
| `SET_PROPERTY` | 设置属性 | Set object property value |
| `DELETE_PROPERTY` | 删除属性 | Delete object property |
| `ADD_PROPERTY_SET` | 添加属性集 | Add property set |
| `REMOVE_PROPERTY_SET` | 删除属性集 | Delete property set |

### Relation Operations

| Action | Chinese | Description |
|--------|---------|-------------|
| `ADD_CONTAINMENT` | 添加包含关系 | Establish parent-child containment |
| `REMOVE_CONTAINMENT` | 删除包含关系 | Remove containment |
| `ADD_CONNECTION` | 添加连接关系 | Establish connection |
| `REMOVE_CONNECTION` | 删除连接关系 | Remove connection |

### Batch Operations

| Action | Chinese | Description |
|--------|---------|-------------|
| `ARRAY` | 阵列 | Linear/circular array replication |
| `BATCH_DELETE` | 批量删除 | Batch delete multiple objects |
| `BATCH_TRANSFORM` | 批量变换 | Batch transform multiple objects |

### Material Operations

| Action | Chinese | Description |
|--------|---------|-------------|
| `SET_MATERIAL` | 设置材料 | Assign material to object |

### Lifecycle Operations

| Action | Chinese | Description |
|--------|---------|-------------|
| `CREATE_LIFECYCLE_RECORD` | 创建生命周期记录 | Create new record |
| `MODIFY_LIFECYCLE_RECORD` | 修改生命周期记录 | Update record content |
| `DELETE_LIFECYCLE_RECORD` | 删除生命周期记录 | Delete record |
| `LINK_RECORD_TO_OBJECT` | 关联记录与对象 | Bind record to object |
| `UPDATE_SCHEDULE` | 更新进度计划 | Update schedule data |
| `UPDATE_COST` | 更新成本 | Update cost data |
| `UPDATE_PROGRESS` | 更新进度 | Update actual progress |
| `CREATE_WORK_ORDER` | 创建工单 | Create work order |
| `UPDATE_WORK_ORDER` | 更新工单 | Update work order status |
| `INGEST_SENSOR_DATA` | 采集传感器数据 | Import sensor data |

### Import/Export

| Action | Chinese | Description |
|--------|---------|-------------|
| `IMPORT_FILE` | 导入文件 | Import IFC/STEP files |
| `EXPORT_FILE` | 导出文件 | Export model file |
| `PARSE_TO_OMCOM` | 解析为 OMCOM | Parse file to OMCOM format |

### Sketch Operations

| Action | Chinese | Description |
|--------|---------|-------------|
| `CREATE_SKETCH` | 创建草图 | Create new sketch |
| `ADD_SKETCH_ENTITY` | 添加图元 | Add entity to sketch |
| `ADD_SKETCH_CONSTRAINT` | 添加约束 | Add constraint to sketch |
| `SOLVE_SKETCH` | 求解草图 | Run constraint solver |
| `DELETE_SKETCH_ENTITY` | 删除图元 | Delete entity from sketch |
| `REMOVE_SKETCH_CONSTRAINT` | 删除约束 | Delete constraint from sketch |
| `DELETE_SKETCH` | 删除草图 | Delete entire sketch |

### Feature Operations

| Action | Chinese | Description |
|--------|---------|-------------|
| `CREATE_COMPONENT` | 创建构件 | Create new component definition |
| `ADD_FEATURE` | 添加特征 | Add feature to component (extrude/revolve, etc.) |
| `REMOVE_FEATURE` | 删除特征 | Remove feature from component |
| `REBUILD_FEATURES` | 重建特征 | Recompute geometry for all features |

### Pipe Run Operations (Civil 3D)

> **Research reference**: Civil 3D pipe run editing actions (`docs/research/civil3d_components.md §6.3`). These edit individual pipe runs within a pipe network.

| Action | Chinese | Description |
|--------|---------|-------------|
| `EXTEND_PIPE_RUN` | 延长管道 | Extend a pipe run and its connected structures |
| `SPLIT_PIPE_RUN` | 拆分管道 | Split a pipe run into two at a station |
| `MERGE_PIPE_RUNS` | 合并管道 | Merge two adjacent pipe runs into one |
| `REVERSE_PIPE_RUN` | 反转管道 | Reverse the direction of a pipe run flow |
| `STRUCTURE_SWAP` | 替换结构 | Swap a structure for a different structure type in the network |
| `ADD_TO_PIPE_RUN` | 添加至管道 | Add a pipe/segment to an existing pipe run |

### Validation

| Action | Chinese | Description |
|--------|---------|-------------|
| `VALIDATE` | 验证 | Validate model data integrity |

---

## 9. Material Categories

| Enum Value | Chinese | English | Description |
|------------|---------|---------|-------------|
| `MATERIAL_CATEGORY_METAL` (1) | 金属 | Metal | Steel, aluminum, copper, etc. |
| `MATERIAL_CATEGORY_CONCRETE` (2) | 混凝土 | Concrete | Normal/prestressed concrete |
| `MATERIAL_CATEGORY_WOOD` (3) | 木材 | Wood | Timber/glulam |
| `MATERIAL_CATEGORY_MASONRY` (4) | 砌体 | Masonry | Brick/stone masonry |
| `MATERIAL_CATEGORY_PLASTIC` (5) | 塑料 | Plastic | Polymer materials |
| `MATERIAL_CATEGORY_COMPOSITE` (6) | 复合材料 | Composite | Carbon fiber/glass fiber, etc. |
| `MATERIAL_CATEGORY_GLASS` (7) | 玻璃 | Glass | Architectural glass |
| `MATERIAL_CATEGORY_INSULATION` (8) | 保温材料 | Insulation | Thermal insulation |
| `MATERIAL_CATEGORY_SOIL` (9) | 土壤 | Soil | Earthwork material |
| `MATERIAL_CATEGORY_WATER` (10) | 水 | Water | Water body |
| `MATERIAL_CATEGORY_OTHER` (99) | 其他 | Other | Unclassified material |

---

## 10. Lifecycle Phases

| Enum Value | Chinese | English | Description |
|------------|---------|---------|-------------|
| `LIFECYCLE_PHASE_PLANNING` (1) | 规划 | Planning | Project initiation, feasibility study |
| `LIFECYCLE_PHASE_DESIGN` (2) | 设计 | Design | Architecture, structure, MEP, civil, plant, mechanical design |
| `LIFECYCLE_PHASE_PROCUREMENT` (3) | 采购 | Procurement | Tendering, contracts, material & equipment procurement |
| `LIFECYCLE_PHASE_CONSTRUCTION` (4) | 施工 | Construction | Construction, schedule management |
| `LIFECYCLE_PHASE_COMMISSIONING_ACCEPTANCE` (5) | 调试验收 | Commissioning & Acceptance | Testing, inspection, trial operation |
| `LIFECYCLE_PHASE_ASSET` (6) | 资产 | Asset | Asset registration, warranty setup |
| `LIFECYCLE_PHASE_OPERATION_MAINTENANCE` (7) | 运维 | Operation & Maintenance | Operations, maintenance, facility management |
| `LIFECYCLE_PHASE_PRODUCTION` (8) | 生产 | Production | Industrial production operations |
| `LIFECYCLE_PHASE_DECOMMISSIONING` (9) | 退役拆除 | Decommissioning | Scrap, demolition, site restoration |

---

## 11. Record Status

| Enum Value | Chinese | English | Description |
|------------|---------|---------|-------------|
| `RECORD_STATUS_DRAFT` (1) | 草稿 | Draft | Initial draft state |
| `RECORD_STATUS_OPEN` (2) | 待处理 | Open | Awaiting processing |
| `RECORD_STATUS_IN_PROGRESS` (3) | 进行中 | In Progress | Currently being processed |
| `RECORD_STATUS_BLOCKED` (4) | 阻塞 | Blocked | Blocked, cannot proceed |
| `RECORD_STATUS_COMPLETED` (5) | 已完成 | Completed | Processing completed |
| `RECORD_STATUS_APPROVED` (6) | 已批准 | Approved | Passed review |
| `RECORD_STATUS_REJECTED` (7) | 已拒绝 | Rejected | Failed review |
| `RECORD_STATUS_CLOSED` (8) | 已关闭 | Closed | Closed/archived |

---

## 12. Object Status

| Enum Value | Chinese | English | Description |
|------------|---------|---------|-------------|
| `OBJECT_STATUS_DRAFT` (1) | 草稿 | Draft | Under design |
| `OBJECT_STATUS_PROPOSED` (2) | 提议 | Proposed | Proposed, pending review |
| `OBJECT_STATUS_APPROVED` (3) | 已批准 | Approved | Approved |
| `OBJECT_STATUS_CONSTRUCTION` (4) | 施工中 | Construction | Under construction |
| `OBJECT_STATUS_AS_BUILT` (5) | 竣工 | As-Built | Construction completed |
| `OBJECT_STATUS_OPERATIONAL` (6) | 运营中 | Operational | In operation |
| `OBJECT_STATUS_DECOMMISSIONED` (7) | 已退役 | Decommissioned | Decommissioned |

---

## 13. Relationship Types

### Containment Types

| Enum Value | Chinese | English | Description |
|------------|---------|---------|-------------|
| `CONTAINMENT_TYPE_SPATIAL` (1) | 空间包含 | Spatial | Building → Storey → Room |
| `CONTAINMENT_TYPE_PHYSICAL` (2) | 物理包含 | Physical | Component → Feature → Entity |
| `CONTAINMENT_TYPE_SYSTEM` (3) | 系统包含 | System | System → Subsystem |
| `CONTAINMENT_TYPE_GROUP` (4) | 分组包含 | Group | Logical grouping |

### Connection Types

| Enum Value | Chinese | English | Description |
|------------|---------|---------|-------------|
| `CONNECTION_TYPE_STRUCTURAL` (1) | 结构连接 | Structural | Structural load-bearing connection |
| `CONNECTION_TYPE_MECHANICAL` (2) | 机械连接 | Mechanical | Mechanical connection/drive |
| `CONNECTION_TYPE_ELECTRICAL` (3) | 电气连接 | Electrical | Electrical connection |
| `CONNECTION_TYPE_HYDRAULIC` (4) | 水力连接 | Hydraulic | Hydraulic connection |
| `CONNECTION_TYPE_SPATIAL_ADJACENT` (5) | 空间相邻 | Spatial Adjacent | Spatial adjacency |

---

## 14. Spatial Relations

| Chinese | English | Description |
|---------|---------|-------------|
| 位于...内部 | Inside | Object A is inside Object B |
| 位于...外部 | Outside | Object A is outside Object B |
| 穿过 | Through | Object A passes through Object B |
| 相邻 | Adjacent | Object A is adjacent to Object B |
| 包围 | Envelops | Object A envelops Object B |
| 分隔 | Separates | Object A separates Object B and C |
| 连接 | Connects | Object A connects Object B and C |
| 悬挂于 | SuspendedFrom | Object A is suspended from Object B |
| 支撑 | Supports | Object A supports Object B |
| 嵌入 | EmbedsIn | Object A is embedded in Object B |
| 附着于 | AttachedTo | Object A is attached to Object B |
| 贯穿 | Penetrates | Object A penetrates Object B |
| 邻近 | Near | Object A is near Object B |
| 对齐 | AlignsWith | Object A aligns with Object B |
| 在...上方 | Above | Object A is above Object B |
| 在...下方 | Below | Object A is below Object B |

---

## 15. Breakdown Levels

| Enum Value | Chinese | English | Description |
|------------|---------|---------|-------------|
| `BREAKDOWN_LEVEL_PROGRAM` (1) | 计划 | Program | Program/portfolio |
| `BREAKDOWN_LEVEL_PROJECT` (2) | 项目 | Project | Single project |
| `BREAKDOWN_LEVEL_SUB_PROJECT` (3) | 子项目 | Sub-Project | Project sub-item |
| `BREAKDOWN_LEVEL_UNIT` (4) | 单元 | Unit | Process unit/building unit |
| `BREAKDOWN_LEVEL_BUILDING` (5) | 建筑 | Building | Building |
| `BREAKDOWN_LEVEL_STOREY` (6) | 楼层 | Storey | Building floor |
| `BREAKDOWN_LEVEL_ZONE` (7) | 区域 | Zone | Functional zone |
| `BREAKDOWN_LEVEL_SYSTEM` (8) | 系统 | System | MEP/structural system |
| `BREAKDOWN_LEVEL_ELEMENT` (9) | 构件 | Element | Individual component |
| `BREAKDOWN_LEVEL_PART` (10) | 零件 | Part | Mechanical part |

### Breakdown Conventions

| Enum Value | Chinese | English | Domain |
|------------|---------|---------|--------|
| `BREAKDOWN_CONVENTION_BUILDING` (1) | 建筑 | Building | Building engineering |
| `BREAKDOWN_CONVENTION_PLANT` (2) | 工厂 | Plant | Plant design |
| `BREAKDOWN_CONVENTION_CIVIL` (3) | 土木 | Civil | Civil engineering |
| `BREAKDOWN_CONVENTION_MECHANICAL` (4) | 机械 | Mechanical | Mechanical design |

---

## 16. Administrative Levels

| Enum Value | Chinese | English | Description |
|------------|---------|---------|-------------|
| `ADMINISTRATIVE_LEVEL_WORLD` (1) | 世界 | World | Global level |
| `ADMINISTRATIVE_LEVEL_COUNTRY` (2) | 国家 | Country | Country |
| `ADMINISTRATIVE_LEVEL_PROVINCE` (3) | 省份 | Province | Province/state |
| `ADMINISTRATIVE_LEVEL_PREFECTURE` (4) | 地区 | Prefecture | Prefecture |
| `ADMINISTRATIVE_LEVEL_COUNTY` (5) | 县 | County | County/district |
| `ADMINISTRATIVE_LEVEL_TOWNSHIP` (6) | 乡镇 | Township | Township |
| `ADMINISTRATIVE_LEVEL_COMMUNITY` (7) | 社区 | Community | Community/village |
| `ADMINISTRATIVE_LEVEL_BLOCK` (8) | 地块 | Block | Block |
| `ADMINISTRATIVE_LEVEL_PARCEL` (9) | 宗地 | Parcel | Parcel |

---

## 17. GenRule Types

> **⚠️ Phase 6 — Two layers, different deprecation status:**
>
> | Layer | What it is | Status |
> |-------|-----------|--------|
> | **GenRule as Classification** | `GenRuleHint` enum, `primary_gen_rule()` (Wall→LayerExtrusion, Beam→LinearComponent, etc.) | ✅ **Active** — conceptual taxonomy for geometry structure, used by `default_catalog_component()` |
> | **GenRule as Generation** | `generate_geometry()` calling `build_wall()`/`build_pipe()` etc. | ❌ **Deprecated** — superseded by catalog `FeatureTreeTemplate` → Feature Engine |
>
> The 5 GenRule types below describe *what kind* of geometry each object has, but the actual geometry generation is now via the **Default Template Registry** and **Component Catalog**.

GenRules map domain semantics to geometry primitives.

> **Conceptual layer**: The GenRule types implement the **4-class component taxonomy** (Surface/Linear/Point/Body, see [Data Model §1](../specification/DATA_MODEL.md#1-overview)): LayerExtrusion → Surface, LinearComponent → Linear, Primitive/Assembly → Body, and PointCloud/TerrainModel (non-GenRule) → Point.

| Type | Chinese | Description | Typical Objects | Component Class |
|------|---------|-------------|-----------------|-----------------|
| (1) Primitive | 基本体 | Direct use of geometric primitives | Tanks, pumps, valves | Body |
| (2) LayerExtrusion | 分层拉伸 | Expand layer structure into multiple extrusions; a Surface boundary + material stack | Walls, slabs, roofs | Surface |
| (3) LinearComponent | 线性构件 | Components arranged along a path with variable cross-section + external constraints | Beam, Column, Road, Bridge, Pipe, Dam, CableTray (bend) | Linear |
| (4) Assembly | 装配体 | Combination of primitives, with conditional params + ConnectionScript support | Equipment assemblies, gears, steel connections | Body |
| (5) BooleanSubtract | 布尔减法 | Subtract from a solid (solid-solid, plane cut, or edge chamfer) | Door/window openings, pipe cuts, Tekla Fitting | — |
| (—) PointCloud / TerrainModel | 点/曲面 | Point-defined or TIN surface geometry; not one of the 5 GenRule types | Point clouds, terrain surfaces | Point |

---

## 17a. Default Template Registry (Phase 6+)

> **Session-level global**: The Default Template Registry is provided as session-level global functions (in the gen_rule module), accessed alongside the Library, not held by the Project or the Library struct.

Maps each `ObjectType` to a default catalog component, so that `create <type> <name>` automatically instantiates a parametric component from the catalog.

| ObjectType | Default Catalog Component | Domain |
|-----------|--------------------------|--------|
| Wall | Concrete-Wall | Building |
| Slab / Floor / Roof | Concrete-Slab | Building |
| Beam | Concrete-Beam-Rect | Building |
| Column | Concrete-Column-Rect | Building |
| Door | Door-Single | Building |
| Window | Window-Fixed | Building |
| PipePlant / PipeCivil | Pipe-CS | Plant |
| Road | Road-Asphalt | Civil |
| Tank | VerticalTank | Plant |
| Pump | CentrifugalPump | Plant |
| StructuralSteel | Steel-Beam-IPE | Structural |
| Valve | GateValve-CS | Plant |
| Fitting | Elbow-LR-CS | Plant |

See: `default_catalog_component()` in `crates/model/src/gen_rule/mod.rs`,
`create_object_from_default_template()` in `crates/model/src/project.rs`

## 17b. Series Library (等级库 / 系列库)

> **Independent module**: SeriesStore is an **independent module**, not a part of Library or Project. It defines spec decision trees (Series → Selector → SpecComponent) that reference Library content by ComponentDef ID. Series can reside in three places: (1) project-private SeriesStore (embedded in `.omnim` file), (2) external `.omnimseries` files referenced by path, (3) built-in seeded series. Only components reachable through loaded series can be used (series gate).

Spec decision tree for project-approved component selection. A Series contains a tree of Selector nodes (questions → answers) that lead to SpecComponent leaf nodes, each referencing a ComponentDef by ID with parameter constraints.

| Series Name | Domain | Standard | Components |
|------------|--------|----------|------------|
| CS150 | Plant/Piping | ASME B31.3 | Pipe-CS, Elbow-LR-CS, EqualTee-CS, Reducer-Conc-CS, Flange-WN-CS, GateValve-CS |
| SS300 | Plant/Piping | ASME B31.3 | Pipe-CS (SS304), Elbow-LR-CS (SS304), Flange-WN-CS (SS304) |
| Steel-Q355 | Structural | EN 1993-1-1 | Steel-Beam-IPE, Steel-Column-HEA |
| Concrete-C30 | Building | EN 1992-1-1 | Concrete-Beam-Rect, Concrete-Column-Rect |
| Highway-GB | Civil | GB JTG B01 | Road-Asphalt, Road-Concrete |

Note: `SeriesStore` is designed as an independent module. Historically it was co-located with `CatalogStore` in `crates/model/src/catalog.rs`; planned to be split into its own module (`series_store.rs`).

**Profile type library**: Based on Tekla Structures profile catalog (`docs/research/tekla_components.md §2.1`), the catalog defines 17 profile types. These are the section families from which structural `ComponentDef` cross-sections are selected:

| # | Profile | English | Notes |
|---|---------|---------|-------|
| 0 | 自由 | Free | Custom/user profile |
| 1 | I | I-beam | Standard I section |
| 2 | 焊接空心 | Welded hollow | Box/rectangular hollow section |
| 3 | U | U-channel | Channel section |
| 4 | L | L-angle | Angle section |
| 5 | 圆棒 | Round bar | Solid circle |
| 6 | 圆管 | Round tube | Hollow circle |
| 7 | RHS | RHS | Rectangular hollow section |
| 8 | T | T-section | Structural tee |
| 9 | 矩形棒 | Rectangular bar | Solid rectangle |
| 10 | Z | Z-section | Z purlin |
| 11 | C | C-section | C purlin |
| 12 | Ω | Omega | Omega purlin |
| 13 | Σ | Sigma | Sigma purlin |
| 14 | 轨道 | Rail | Crane rail |
| 16 | 钢筋 | Rebar | Steel reinforcement bar |

These map to OmniM `ComponentDef` profiles (e.g., IPE, HEA, W, angle, channel, RHS, rebar) referenced by the catalog and validated by the Series gate.

## 17c. Catalog Instantiation Pipeline

The end-to-end flow from user command to generated geometry. Note: Project does **not** hold a Library reference; the Executor accesses Library via the global static `LIBRARY`.

```
create wall W1 --param=thickness=200
  ↓
Executor resolves:
  default_catalog_component(Wall) → "Concrete-Wall"    ← Library (global static)
  ↓
LIBRARY.catalog.find_by_name("Concrete-Wall") → ComponentDef   ← Library
  ↓
┌─ Series Gate ─────────────────────────────────┐
│  Is "Concrete-Wall" reachable from any        │
│  loaded series in the Project?                │
│  - Yes: proceed                               │
│  - No series loaded: allow (open project)     │
│  - No (series loaded but not approved): reject│
└───────────────────────────────────────────────┘
  ↓
ComponentDef.feature_tree_id → FeatureTreeTemplate   ← Library
  ↓
instantiate_template() → creates sketch + features   ← Project
  ↓
Project.rebuild_component() → Feature Engine → BRep → Mesh
  ↓
Object in Project with cached geometry, feature history,
series reference, and catalog reference
```

See: `instantiate_catalog_component()` in `crates/model/src/project.rs`,
`instantiate_template()` in `crates/model/src/feature_template.rs`

## 17d. Library vs Project Architecture

Three independent frontend modules correspond to the three-layer backend architecture:

```
┌─────────────────────────────────────────────────────┐
│  Library (session-level global static)               │
│  • CatalogStore            ← ComponentDef entries   │
│  • ComponentStore          ← Component templates    │
│  • FeatureTreeRegistry     ← feature tree templates │
│  • Default Template Registry ← ObjectType→Def map   │
│                                                      │
│  Loaded once per session, shared across all projects │
│  Accessed via static LIBRARY (not held by Project)  │
│  Frontend: Library Manager (独立管理界面)             │
└──────────────────────┬──────────────────────────────┘
                       │  referenced by ComponentDef ID
              ┌────────▼────────────────────────┐
              │  SeriesStore (independent module) │
              │  • Series: spec decision trees    │
              │    - Built-in seed (read-only)    │
              │    - External .omnimseries files  │
              │  • SpecComponent → ComponentDef ID│
              │  • SelectionEngine                │
              │  Frontend: SeriesStore Manager     │
              └────────┬────────────────────────┘
                       │  referenced by Project
              ┌────────┼──────────────┐
              │        │              │
     ┌────────▼───┐  ┌─▼────────┐  ┌─▼────────┐
     │ Project A  │  │Project B │  │ Project C │
     │ ┌────────┐ │  │┌────────┐│  │┌────────┐ │
     │ │ Object │ │  ││ Object ││  ││ Object │ │
     │ │ Store  │ │  ││ Store  ││  ││ Store  │ │
     │ ├────────┤ │  │├────────┤│  │├────────┤ │
     │ │ Series │ │  ││ Series ││  ││ Series │ │
     │ │ (私有)  │ │  ││ (私有)  ││  ││ (私有)  │ │
     │ ├────────┤ │  │├────────┤│  │├────────┤ │
     │ │ext refs│ │  ││ext refs││  ││ext refs│ │
     │ └────────┘ │  │└────────┘│  │└────────┘ │
     │ Frontend:  │  │          │  │          │
     │ Project    │  │          │  │          │
     │ Workspace  │  │          │  │          │
     └────────────┘  └──────────┘  └──────────┘

  Rule: Only ComponentDefs reachable through the
  project's loaded Series may be instantiated.
  If no series is loaded, all catalog components
  are available (open mode).
```

---

## 17e. TerrainModel (TIN Surface)

> **Research ref**: Bentley OpenRoads Terrain Model (`docs/research/bentley_openroads_components.md §6.1`).

A `TerrainModel` is a **6th geometry type** alongside the 5 GenRule types. It represents an irregular triangulated network (TIN) surface and is **not** a sweep/loft/boolean — it does not fit any of the 5 GenRule categories. See [Data Model §6.8](../specification/DATA_MODEL.md#68-terrain-model-terrainmodel-tin-surface).

| Concept | Chinese | Description |
|---------|---------|-------------|
| TIN surface | 三角网曲面 | Surface defined by points + breaklines + triangles |
| BreakLine | 断裂线 | Edge that triangles cannot cross |
| VolumeOption | 计方选项 | Existing / Design / Subgrade / None |

**TerrainRole system**: Every `TerrainModel` carries a `TerrainRole` classifying its purpose: `Existing` (现状), `Design` (设计), `Subgrade` (路基/地基), or `Reference` (参考). The role determines how the terrain participates in cut/fill computation and whether it is a target for daylighting.

**Derived objects**: Cut/Fill volume computation between two TIN surfaces (a BooleanSubtract between surfaces) produces derived site objects: `CutVolume`, `FillVolume`, `GradingSurface`, `Slope`, `FeatureLine`, `ContourLine`, `SiteComponent`. These are exposed as OmniM ObjectTypes (see §2.3) and tracked by `EarthworkBalance` for net cut/fill analysis.

Used for: LinearComponent End Condition daylight solving, Cut/Fill volume computation (BooleanSubtract between two TINs), and terrain/earthwork visualization.

## 17f. LogicalObject

> **Research ref**: AVEVA Instrumentation Loop/Tag (`docs/research/e3d_instrumentation_components.md §4`).

A `LogicalObject` is a geometry-less engineering entity (Tag/Loop/LineNumber) that references physical objects via `member_object_ids`. It is a new base type parallel to `Object`. See [Data Model §9](../specification/DATA_MODEL.md#69-logical-object-logicalobject-tag--loop--linenumber).

| Concept | Chinese | Description |
|---------|---------|-------------|
| Tag | 位号 | ISA-5.1 instrument tag (e.g. "FT-101A") |
| Loop | 回路 | Control loop grouping multiple tags |
| LineNumber | 管线号 | Process line number (e.g. 12"-3A1-CS-SCH40) |

Key principles: no geometry, no GenRule, no feature tree; holds references to physical objects; cross-view traceability (P&ID ↔ 3D).

**ISA-5.1 symbol conventions** (`docs/research/e3d_instrumentation_components.md §2.1.1`): The ISA-5.1 standard defines instrument symbol shapes for P&ID readability:
- Circle = field-mounted instrument
- Circle + single horizontal line = control room/panel-mounted
- Circle + double horizontal line = field panel-mounted
- Square inscribed in circle = DCS (Distributed Control System) function
- Signal line types: dashed = electrical, double-dashed = pneumatic, solid with double slash = capillary

## 17g. Constraint (Mates)

> **Research ref**: SOLIDWORKS 21 mate types (`docs/research/solidworks_components.md §4`).

A `Constraint` governs degrees of freedom between Assembly child components. Stored in `RelationStore` alongside connections. See [Data Model §6.12](../specification/DATA_MODEL.md#612-constraint-system-constraint-mates).

| Class | Chinese | Types |
|-------|---------|-------|
| Standard | 标准配合 | Coincident, Concentric, Parallel, Perpendicular, Tangent, Distance, Angle, Lock |
| Advanced | 高级配合 | Profile Center, Symmetric, Width, Path Mate, Linear/Linear Coupler, Limit |
| Mechanical | 机械配合 | Cam, Slot, Hinge, Gear, Rack Pinion, Screw, Universal Joint |

## 17h. ConnectionScript

> **Research ref**: Tekla ConnectionBase (`docs/research/tekla_components.md §3`).

A `ConnectionScript` is a processual script component reference used by Assembly GenRule to generate steel connections (with optional stiffeners, end-plates, bolts driven by condition parameters). See [Data Model §6.11](../specification/DATA_MODEL.md#611-assembly-enhancement).

**Tekla ConnectionBase plugin architecture** (`docs/research/tekla_components.md §3.3`): ConnectionScript follows the Tekla ConnectionBase contract:
1. **Input contract**: The script receives main part(s), secondary part(s), positions, and a polygon (via `ComponentInput` pattern — `AddInputObject`, `AddInputPositions`, `AddInputPolygon`; order-sensitive)
2. **Plugin attributes**: `[Plugin("name")]`, `[PluginUserInterface]`, `[SecondaryType]`, `[AutoDirectionType]`, `[PositionType]` — these annotate the script with UI hints and part selection behavior
3. **Run contract**: The `Run()` method generates child components (plates, bolts, welds) based on input parts and condition parameters
4. **Example**: EndPlate connection (144) params — plate position, thickness, width, bolt layout, weld size, stiffener options, haunch options

## 17i. InterpolationMethod

> **Research ref**: Bentley OBM beam variation (`docs/research/bentley_openroads_components.md §2.1.3`).

The cross-section interpolation method for `CrossSectionSegment`, extended from a single linear mode to 6 methods: Constant, Linear, Parabolic Up/Down, Circular Up/Down. See [Data Model §6.4](../specification/DATA_MODEL.md#64-linearcomponent-linearcomponent).

---

## 18. Editor Terms

### Tools

| Tool | Shortcut | Chinese | Description |
|------|----------|---------|-------------|
| Select | V | 选择 | Select entity |
| Line | L | 线段 | Draw line segment |
| Circle | C | 圆 | Draw circle |
| Arc | A | 圆弧 | Draw arc |
| Rect | R | 矩形 | Draw rectangle |
| Polygon | P | 正多边形 | Draw regular polygon |
| Ellipse | E | 椭圆 | Draw ellipse |
| Spline | S | 样条曲线 | Draw spline curve |
| Polyline | N | 折线 | Draw connected polyline |

### UI Operations

| Operation | Chinese | Description |
|-----------|---------|-------------|
| 吸附 | Snap | Auto-snap to endpoint/midpoint/center |
| 撤销 | Undo | Undo previous operation |
| 重做 | Redo | Redo previous operation |
| 删除 | Delete / Backspace | Delete selected entity |
| 完成 | Enter / Right-click | Complete current drawing |
| 取消 | Escape | Cancel current operation |
| 属性面板 | Properties Panel | Edit selected entity properties |
| 实体创建模式 | Feature Creation | Enter sketch drawing mode after selecting feature type |

### Snap Types

| Snap | Chinese | English | Description |
|------|---------|---------|-------------|
| Endpoint | 端点 | Endpoint | Line/arc endpoint |
| Midpoint | 中点 | Midpoint | Line midpoint |
| Center | 圆心 | Center | Circle/arc center |
| Threshold | 捕捉阈值 | Snap Threshold | 8 pixels |

### Feature Creation Mode Flow

1. Click feature button (e.g., "拉伸") → Enter creation mode
2. Switch to sketch labeled by required tag (e.g., "轮廓")
3. Draw profile in sketch
4. Press Enter/right-click to complete current sketch → Auto-switch to next sketch
5. All sketches completed → Feature parameter dialog appears
6. Confirm parameters → Backend API creates feature

---

## 19. Domain-Specific Terms

### Piping

| Chinese | English | Description |
|---------|---------|-------------|
| 管道等级 | Piping Spec | Piping material specification |
| 管件 | Pipe Fitting | Elbow, tee, reducer, etc. |
| 大小头 | Reducer | Variable-diameter fitting (round→round or square→square) |
| 天圆地方 | Round-to-Square | Transition piece from circular to square |
| 管架 | Pipe Rack | Pipe support rack |
| 管道支架 | Pipe Support | Pipe support fixture |
| 法兰 | Flange | Pipe connection flange |
| 喷嘴 | Nozzle | Equipment nozzle |

### Structural

| Chinese | English | Description |
|---------|---------|-------------|
| 配筋 | Rebar | Rebar configuration |
| 荷载 | Load | Structural load |
| 应力 | Stress | Material stress |
| 变形 | Deformation | Structural deformation |
| 支座 | Support | Structural support |

### HVAC

| Chinese | English | Description |
|---------|---------|-------------|
| 风管 | Duct | Air duct |
| 散流器 | Diffuser | Air diffuser |
| 回风口 | Return Air Grille | Return air grille |
| 新风 | Fresh Air | Outdoor fresh air |
| 冷冻水 | Chilled Water | Chilled water |
| 冷却水 | Cooling Water | Cooling tower water |

### Electrical

| Chinese | English | Description |
|---------|---------|-------------|
| 配电柜 | Switchboard | Power distribution panel |
| 桥架 | Cable Tray | Cable tray |
| 母线 | Busbar | Power busbar |
| 回路 | Circuit | Electrical circuit |
| 接地 | Grounding | Grounding system |

### Municipal

| Chinese | English | Description |
|---------|---------|-------------|
| 路面 | Pavement | Road surface |
| 路肩 | Shoulder | Road shoulder |
| 路缘石 | Curb | Road curb |
| 排水沟 | Drainage Channel | Drainage facility |
| 检查井 | Manhole | Underground inspection chamber |

---

## 20. API Endpoints

| Endpoint | Method | Chinese | Description |
|----------|--------|---------|-------------|
| `/api/objects` | GET | 获取对象列表 | Query all objects |
| `/api/objects/{id}` | GET | 获取对象详情 | Query single object |
| `/api/objects` | POST | 创建对象 | Create new object |
| `/api/objects/{id}` | PUT | 更新对象 | Modify object properties |
| `/api/objects/{id}` | DELETE | 删除对象 | Delete object |
| `/api/objects/{id}/children` | GET | 获取子对象 | Query child nodes |
| `/api/objects/{id}/tree` | GET | 获取对象树 | Query full hierarchy tree |
| `/api/objects/{id}/mesh` | GET | 获取网格 | Get 3D mesh data |
| `/api/stats` | GET | 获取统计 | Project statistics |
| `/api/import/ifc` | POST | 导入 IFC | Import IFC file |
| `/api/sketches` | POST | 创建草图 | Create new sketch |
| `/api/sketches/{id}` | GET | 获取草图 | Query sketch details |
| `/api/sketches/{id}` | PUT | 更新草图 | Modify sketch properties (incl. Z height) |
| `/api/sketches/{id}/entities` | POST | 添加图元 | Add entity to sketch |
| `/api/sketches/{id}/entities/{eid}` | PUT | 更新图元 | Modify entity properties |
| `/api/sketches/{id}/entities/{eid}` | DELETE | 删除图元 | Delete entity |
| `/api/sketches/{id}/constraints` | POST | 添加约束 | Add constraint to sketch |
| `/api/sketches/{id}/solve` | POST | 求解草图 | Run constraint solver |
| `/api/components` | POST | 创建构件 | Create new component |
| `/api/components/{id}` | GET | 获取构件 | Query component details |
| `/api/components/{id}/features` | POST | 添加特征 | Add feature to component |
| `/api/components/{id}/features/{fid}` | GET | 获取特征 | Query feature details |
| `/api/components/{id}/features/{fid}` | DELETE | 删除特征 | Delete feature |
| `/api/components/{id}/rebuild` | POST | 重建构件 | Recompute all feature geometries |
| `/api/amir/translate` | POST | AI 翻译 | Natural language to commands |
| `/api/amir/chat` | POST | AI 对话 | Conversational modeling |
| `/api/amir/describe` | POST | AI 描述 | Generate object description |
| `/api/amir/explain` | POST | AI 解释 | Explain error messages |

---

## 21. Actor Types

| Enum Value | Chinese | English | Description |
|------------|---------|---------|-------------|
| `USER` (1) | 用户 | User | Human user |
| `AI_AGENT` (2) | AI 代理 | AI Agent | AI-assisted modeling |
| `SYSTEM` (3) | 系统 | System | System automatic operation |
| `IMPORT` (4) | 导入 | Import | File import operation |

---

## 22. Parameter Types

| Type | Description |
|------|-------------|
| `string_value` | String parameter |
| `int_value` | Integer parameter |
| `double_value` | Floating-point parameter |
| `bool_value` | Boolean parameter |

---

## 23. Coordinate Systems

| Chinese | English | Description |
|---------|---------|-------------|
| 世界坐标系 | World Coordinate System (WCS) | Global fixed coordinate system |
| 物体坐标系 | Object Coordinate System (OCS) | Individual object local coordinate system |
| 草图坐标系 | Sketch Coordinate System | Sketch plane 2D coordinate system |
| 局部坐标系 | Local Coordinate System (LCS) | Component/feature local coordinate system |
| 投影坐标系 | Projected CRS (CRS) | Geographic projected coordinate system |
| 地理坐标系 | Geographic CRS | Latitude/longitude coordinate system |

---

## 24. Mesh Data

| Field | Chinese | Description |
|-------|---------|-------------|
| `vertices` | 顶点坐标 | 3D vertex coordinate array (repeated Point3D) |
| `normals` | 法线 | Vertex normal array (repeated Vector3D) |
| `uvs` | 纹理坐标 | Texture coordinate array (repeated Point2D) |
| `faces` | 面索引 | Triangle face index array (repeated FaceIndices) |
| `draco_compressed` | Draco 压缩 | Optional Draco compressed geometry blob |

---

## 25. Storage Model

| Chinese | English | Description |
|---------|---------|-------------|
| 对象存储 | Object Store | Stores all object data in JSON format |
| 元数据存储 | Metadata Store | Object index and relationship data |
| 原始文件存储 | Raw File Store | Original IFC/STEP files |
| 向量索引 | Vector Index | Semantic search vector index |
| 图关系索引 | Graph Index | Object relationship graph index |

---

## 26. Protocols & Formats

| Abbreviation | Full Name | Chinese | Description |
|--------------|-----------|---------|-------------|
| OMNIM | OmniM Internal Model | OmniM 内部模型 | Internal unified data format |
| OMCOM | OmniM Command | OmniM 命令 | Unified typed command system (OmniCommand schema via protobuf-as-JSON types; Executor+Validator in omnim-omcom). The Omcom text surface is a Draft RFC (RFC-0001), not implemented. |
| AMIR | AI Intermediate Representation | AI 中间表示 | Natural language modeling interface |
| IFC | Industry Foundation Classes | 工业基础类 | Building industry data exchange standard |
| STEP | Standard for the Exchange of Product Data | 产品数据交换标准 | ISO 10303 |
| B-Rep | Boundary Representation | 边界表示法 | 3D solid representation method |
| IGES | Initial Graphics Exchange Specification | 初始图形交换规范 | CAD data exchange format |
| glTF | GL Transmission Format | GL 传输格式 | 3D model transmission format |
| PMI | Product Manufacturing Information | 产品制造信息 | Manufacturing annotation information |
| LOD | Level of Detail | 细节层次 | Model detail level |
| COBie | Construction Operations Building Information Exchange | 施工运维信息交换 | Operations data exchange format |
