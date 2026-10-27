# OmniM 术语表 (Glossary)

本文档定义 OmniM 系统中所有术语的中英文对照、含义说明及代码引用。

---

## 1. 核心层级 (Core Hierarchy)

OmniM 采用四层对象层级模型：

| 层级 | 中文 | 英文 | 说明 | 代码位置 |
|------|------|------|------|----------|
| Object | 对象 | OmniObject | 系统中所有实体的基类，包括建筑、设备、零件等 | `web/src/shared/types.ts:OmniObject` |
| Component | 构件 | Component | 由一个或多个特征组成的三维几何体，是 Object 的几何表示 | `crates/model/src/lib.rs:ComponentDef` |
| Feature | 特征 | Feature | 从草图生成三维实体的操作（拉伸、旋转、放样等） | `crates/model/src/feature.rs:Feature` |
| Sketch | 草图 | Sketch | 二维几何轮廓，包含图元和约束，是特征的输入 | `crates/sketch/src/lib.rs:Sketch` |
| Entity | 图元 | SketchEntity | 草图中的单个几何元素（线、圆、矩形等） | `crates/sketch/src/lib.rs:SketchEntity` |
| Constraint | 约束 | SketchConstraint | 限制图元之间几何关系的规则（水平、垂直、相切等） | `crates/sketch/src/lib.rs:SketchConstraint` |

**父子关系**：
```
OmniObject
  └── ComponentDef
        ├── Sketch (轮廓草图)
        ├── Feature (拉伸/旋转/放样/融合等)
        │     └── Sketch (输入草图)
        └── Feature ...
```

**系列 (Series)**：同类对象的集合，通过选择器决策树筛选可用构件。

---

## 2. 对象类型 (Object Types)

OmniM 支持 50+ 种对象类型，按工程领域分组。

> **E3D 元素类型代码交叉引用**：AVEVA E3D 使用简短的元素类型代码（`docs/research/aveva_e3d_primitives.md §2.2`）。这些映射到 OmniM ObjectType，作为导入/导出的参考：
>
> | E3D 代码 | 含义 | OmniM ObjectType |
> |----------|------|------------------|
> | EQUI | 设备 | OBJECT_TYPE_TANK / PUMP 等 |
> | SUBE | 子设备 | OBJECT_TYPE_*（子构件） |
> | STRU | 结构 | OBJECT_TYPE_STRUCTURAL_STEEL |
> | FRMW | 框架 | OBJECT_TYPE_ASSEMBLY |
> | SBFR | 子框架 | OBJECT_TYPE_ASSEMBLY |
> | GENSEC | 结构型材（IPE300） | OBJECT_TYPE_STRUCTURAL_STEEL |
> | PANE | 面板 | OBJECT_TYPE_*（Panel） |
> | PIPE | 管道段 | OBJECT_TYPE_PIPE_PLANT |
> | BRAN | 支管 | OBJECT_TYPE_PIPE_PLANT（管段） |
> | NOZZ | 管口 | OBJECT_TYPE_*（Nozzle） |

### 2.1 空间元素 (Spatial Elements)

| 枚举值 | 中文 | 英文 | 说明 |
|--------|------|------|------|
| `OBJECT_TYPE_SITE` (1) | 场地 | Site | 项目场地 |
| `OBJECT_TYPE_BUILDING` (2) | 建筑 | Building | 建筑物 |
| `OBJECT_TYPE_BUILDING_STOREY` (3) | 楼层 | Building Storey | 建筑楼层 |
| `OBJECT_TYPE_SPACE` (4) | 空间 | Space | 房间/区域 |

### 2.2 建筑工程 (Building Elements)

| 枚举值 | 中文 | 英文 | 说明 |
|--------|------|------|------|
| `OBJECT_TYPE_WALL` (100) | 墙 | Wall | 建筑墙体 |
| `OBJECT_TYPE_BEAM` (101) | 梁 | Beam | 结构梁 |
| `OBJECT_TYPE_COLUMN` (102) | 柱 | Column | 结构柱 |
| `OBJECT_TYPE_SLAB` (103) | 板 | Slab | 楼板/底板 |
| `OBJECT_TYPE_DOOR` (104) | 门 | Door | 建筑门 |
| `OBJECT_TYPE_WINDOW` (105) | 窗 | Window | 建筑窗 |
| `OBJECT_TYPE_STAIR` (106) | 楼梯 | Stair | 建筑楼梯 |
| `OBJECT_TYPE_ROOF` (107) | 屋顶 | Roof | 建筑屋顶 |
| `OBJECT_TYPE_FLOOR` (108) | 地板 | Floor | 地面层 |
| `OBJECT_TYPE_DUCT` (109) | 风管 | Duct | 暖通风管 |
| `OBJECT_TYPE_CABLE_TRAY` (110) | 电缆桥架 | Cable Tray | 电气电缆桥架 |
| `OBJECT_TYPE_AIR_TERMINAL` (111) | 风口 | Air Terminal | 暖通风口/散流器 |
| `OBJECT_TYPE_AHU` (112) | 空调机组 | Air Handler (AHU) | 空气处理机组 |
| `OBJECT_TYPE_CHILLER` (113) | 冷水机 | Chiller | 冷冻机组 |
| `OBJECT_TYPE_TRANSFORMER` (114) | 变压器 | Transformer | 电力变压器 |
| `OBJECT_TYPE_LUMINAIRE` (115) | 灯具 | Luminaire | 照明灯具 |

### 2.3 土木工程 (Civil Elements)

| 枚举值 | 中文 | 英文 | 说明 |
|--------|------|------|------|
| `OBJECT_TYPE_ROAD` (200) | 道路 | Road | 公路/城市道路 |
| `OBJECT_TYPE_BRIDGE` (201) | 桥梁 | Bridge | 桥梁结构 |
| `OBJECT_TYPE_TUNNEL` (202) | 隧道 | Tunnel | 隧道结构 |
| `OBJECT_TYPE_DAM` (203) | 大坝 | Dam | 水利大坝 |
| `OBJECT_TYPE_CULVERT` (204) | 涵洞 | Culvert | 排水涵洞 |
| `OBJECT_TYPE_RETAINING_WALL` (205) | 挡土墙 | Retaining Wall | 支挡结构 |
| `OBJECT_TYPE_PIPE_CIVIL` (206) | 管道(土木) | Pipe (Civil) | 市政管道 |
| `OBJECT_TYPE_PIER` (207) | 桥墩 | Pier | 桥梁墩台 |
| `OBJECT_TYPE_MANHOLE` (208) | 检查井 | Manhole | 市政检查井 |
| `OBJECT_TYPE_ABUTMENT` (209) | 桥台 | Abutment | 桥梁桥台 |
| `OBJECT_TYPE_CUT_VOLUME` (220) | 挖方量 | Cut Volume | 两曲面之间的挖方量 |
| `OBJECT_TYPE_FILL_VOLUME` (221) | 填方量 | Fill Volume | 两曲面之间的填方量 |
| `OBJECT_TYPE_GRADING_SURFACE` (222) | 平整曲面 | Grading Surface | 设计整平面 |
| `OBJECT_TYPE_SLOPE` (223) | 边坡 | Slope | 填方或挖方边坡 |
| `OBJECT_TYPE_FEATURE_LINE` (224) | 特征线 | Feature Line | 用于整平控制的 3D 特征线 |
| `OBJECT_TYPE_CONTOUR_LINE` (225) | 等高线 | Contour Line | 高程等高线 |
| `OBJECT_TYPE_SITE_COMPONENT` (226) | 场地构件 | Site Component | 可复用的场地布局构件 |
| `OBJECT_TYPE_EARTHWORK_BALANCE` (227) | 土方平衡 | Earthwork Balance | 挖填平衡分析结果 |

### 2.4 工厂设计 (Plant Elements)

| 枚举值 | 中文 | 英文 | 说明 |
|--------|------|------|------|
| `OBJECT_TYPE_TANK` (300) | 储罐 | Tank | 工艺储罐 |
| `OBJECT_TYPE_PUMP` (301) | 泵 | Pump | 工艺泵 |
| `OBJECT_TYPE_VALVE` (302) | 阀门 | Valve | 工艺阀门 |
| `OBJECT_TYPE_HEAT_EXCHANGER` (303) | 换热器 | Heat Exchanger | 热交换设备 |
| `OBJECT_TYPE_REACTOR` (304) | 反应器 | Reactor | 化学反应器 |
| `OBJECT_TYPE_PIPE_PLANT` (305) | 管道(工厂) | Pipe (Plant) | 工艺管道 |
| `OBJECT_TYPE_STRUCTURAL_STEEL` (306) | 钢结构 | Structural Steel | 工厂钢结构 |
| `OBJECT_TYPE_FITTING` (307) | 管件 | Pipe Fitting | 弯头/三通/变径等 |
| `OBJECT_TYPE_COMPRESSOR` (308) | 压缩机 | Compressor | 气体压缩机 |
| `OBJECT_TYPE_INSTRUMENT` (309) | 仪表 | Instrument | 过程仪表/传感器 |
| `OBJECT_TYPE_CONTROL_VALVE` (310) | 调节阀 | Control Valve | 带执行器+定位器的阀门 |
| `OBJECT_TYPE_CABLE` (311) | 电缆 | Cable | 电力/信号/控制电缆 |
| `OBJECT_TYPE_CONDUIT` (312) | 导管 | Conduit | 电缆保护导管 |

> **管件类型**（E3D 目录，`docs/research/aveva_e3d_primitives.md §4.2`）：ELBO（弯头）、BEND（弯管）、TEE（三通）、VALV（阀门）、FLAN（法兰）、GASK（垫片）、REDUCER（大小头）、VENT（排气口）、DRAIN（排液口）、CAP（管帽）、OLET（管座）。这些是定义在 `OBJECT_TYPE_FITTING` 下的参数化目录条目。
>
> **调节阀细节**（`docs/research/e3d_instrumentation_components.md §2.4.1`）：
> - **执行器类型**：气动薄膜式、气动活塞式、电动式、电磁式、手动手轮式、弹簧复位式
> - **故障模式**：FO（故障开）、FC（故障关）、FL（故障保位）
> - **GenRule**：Assembly（阀体 + 执行器 + 定位器 + 附件）

### 2.5 机械设计 (Mechanical Elements)

| 枚举值 | 中文 | 英文 | 说明 |
|--------|------|------|------|
| `OBJECT_TYPE_PART` (400) | 零件 | Part | 机械零件 |
| `OBJECT_TYPE_ASSEMBLY` (401) | 组件 | Assembly | 机械组件 |
| `OBJECT_TYPE_FASTENER` (402) | 紧固件 | Fastener | 螺栓/螺母/垫片 |
| `OBJECT_TYPE_GEAR` (403) | 齿轮 | Gear | 传动齿轮 |
| `OBJECT_TYPE_SHAFT` (404) | 轴 | Shaft | 传动轴 |
| `OBJECT_TYPE_BEARING` (405) | 轴承 | Bearing | 支撑轴承 |
| `OBJECT_TYPE_MOTOR` (406) | 电机 | Motor | 驱动电机 |
| `OBJECT_TYPE_SPRING` (407) | 弹簧 | Spring | 弹性元件 |
| `OBJECT_TYPE_SEAL` (408) | 密封件 | Seal | 密封装置 |

### 2.6 其他

| 枚举值 | 中文 | 英文 | 说明 |
|--------|------|------|------|
| `OBJECT_TYPE_GENERIC` (999) | 通用 | Generic | 通用/未分类对象 |
| `OBJECT_TYPE_COMPONENT` (500) | 构件 | Component | 构件定义（目录） |

---

## 3. 特征类型 (Feature Types)

特征是从草图生成三维实体的操作。

| 枚举值 | 中文 | 英文 | 说明 | 输入草图数 |
|--------|------|------|------|-----------|
| `Extrude` (1) | 拉伸 | Extrude | 将轮廓沿方向拉伸成实体 | 1 |
| `Revolve` (2) | 旋转 | Revolve | 将轮廓绕轴旋转成实体 | 1 (轮廓+轴线) |
| `Sweep` (3) | 扫掠 | Sweep | 将轮廓沿路径移动成实体 | 1 (轮廓+路径) |
| `Loft` (4) | 放样 | Loft | 连接多个截面成实体 | 2+ (截面) |

| `Boolean` (5) | 布尔 | Boolean | 两个实体的并集/差集/交集运算 | — |
| `Chamfer` (6) | 倒角 | Chamfer | 对边缘进行倒角处理 | — |
| `Fillet` (7) | 圆角 | Fillet | 对边缘进行圆角处理 | — |
| `Pattern` (8) | 阵列 | Pattern | 线性或环形复制排列 | — |
| `Shell` (9) | 抽壳 | Shell | 将实体变为薄壁 | — |

> **圆角子类型**（源自 SOLIDWORKS，`docs/research/solidworks_components.md §3.3`）：支持 8 种圆角子类型：
> 1. Constant Size（等径圆角）——沿边的均匀半径
> 2. Multiple Radius（多半径圆角）——一次操作中每条边不同半径
> 3. Round Corner（圆角过渡）——平滑转角过渡
> 4. Setback（顶点回退圆角）——顶点处可变半径（三边交汇）
> 5. Variable Size（变径圆角）——半径沿边变化
> 6. Face（面圆角）——对整个面做圆角
> 7. Full Round（全圆角）——三面相切圆角
> 8. Asymmetric（非对称圆角）——相邻两面使用两种不同半径

### 特征参数

| 特征 | 参数 | 类型 | 默认值 | 说明 |
|------|------|------|--------|------|
| Extrude | `depth` | number | 1.0 | 拉伸深度 |
| Extrude | `direction_x/y/z` | number | 0/0/1 | 拉伸方向向量 |
| Revolve | `angle` | number | 0.0 | 旋转角度（弧度，0=360°） |
| Boolean | `operation` | select | subtract | union/subtract/intersect |
| Chamfer | `distance` | number | 0.1 | 倒角距离 |
| Fillet | `radius` | number | 0.1 | 圆角半径 |
| Pattern | `pattern_type` | select | linear | linear/rectangular/circular |
| Pattern | `count` | number | 2 | 复制数量 |
| Pattern | `spacing` | number | 1.0 | 复制间距 |
| Shell | `wall_thickness` | number | 0.1 | 壁厚 |

### 可切割特征 (Cuttable Types)

以下特征支持 `is_cut` 开关，用于挖孔/开槽：
`extrude`, `revolve`, `sweep`, `loft`

---

## 4. 草图图元类型 (Sketch Entity Types)

| 枚举值 | 中文 | 英文 | 说明 | 必需点数 |
|--------|------|------|------|---------|
| `SketchLine` | 线段 | Line | 两点之间的直线段 | 2 |
| `SketchArc` | 圆弧 | Arc | 三点定义的圆弧 | 3 |
| `SketchCircle` | 圆 | Circle | 圆心+半径 | 1 (圆心) |
| `SketchPoint` | 点 | Point | 零维几何点 | 1 |
| `SketchSpline` | 样条曲线 | Spline | 通过多个控制点的平滑曲线 | 2+ |
| `SketchSlot` | 槽口 | Slot | 3 种变体的槽：Straight/Arc/Conical | 2 |
| `SketchConic` | 圆锥曲线 | Conic | 端点 + Rho 值（椭圆/抛物线/双曲线） | 2 |
| `SketchParabola` | 抛物线 | Parabola | 顶点+焦点偏移+起点/终点 | 4 |
| `SketchText` | 草图文字 | Text | 曲线/边上的 2D 文本，可拉伸/切除成 3D | 1 |
| `BeltChain` | 皮带/链 | Belt Chain | 带轮/链轮机构布局草图 | 2+ |
| `EquationCurve` | 方程曲线 | Equation Curve | 参数方程驱动曲线 | 2 |

### 图元属性

| 属性 | 适用类型 | 说明 |
|------|---------|------|
| `points` | 所有 | 控制点坐标数组 `{x, y, z}` |
| `radius` | Circle | 圆的半径 |

---

## 5. 草图约束类型 (Sketch Constraint Types)

| 枚举值 | 中文 | 英文 | 符号 | 说明 |
|--------|------|------|------|------|
| `HORIZONTAL` | 水平 | Horizontal | H | 线段或两点连线保持水平 |
| `VERTICAL` | 垂直 | Vertical | V | 线段或两点连线保持垂直 |
| `COINCIDENT` | 重合 | Coincident | ⦿ | 两个点重合于同一位置 |
| `PARALLEL` | 平行 | Parallel | ∥ | 两条线段保持平行 |
| `PERPENDICULAR` | 正交 | Perpendicular | ⊥ | 两条线段保持垂直 |
| `LENGTH` | 长度 | Length | L= | 线段长度等于指定值 |
| `DISTANCE` | 距离 | Distance | D= | 两点之间距离等于指定值 |
| `ANGLE` | 角度 | Angle | ∠ | 两条线段夹角等于指定值 |
| `RADIUS` | 半径 | Radius | R= | 圆/弧半径等于指定值 |
| `DIAMETER` | 直径 | Diameter | Φ | 圆/弧直径等于指定值 |
| `TANGENT` | 相切 | Tangent | ⌒ | 两个图元相切 |
| `SYMMETRIC` | 对称 | Symmetric | \| | 两个图元关于轴对称 |
| `FIXED` | 固定 | Fixed | ⚓ | 图元位置/大小不可变 |
| `MIDPOINT` | 中点 | Midpoint | ◉ | 点位于线段中点 |
| `OFFSET` | 偏移 | Offset | ↕ | 两个图元之间的偏移距离 |
| `EQUAL_LENGTH` | 等长 | Equal Length | = | 两条线段长度相等 |
| `EQUAL_RADIUS` | 等半径 | Equal Radius | R= | 两个圆/弧半径相等 |
| `PT_ON_LINE` | 点在线上 | Point on Line | — | 点位于线段上 |
| `PT_ON_CIRCLE` | 点在圆上 | Point on Circle | ○ | 点位于圆/弧上 |
| `SAME_ORIENTATION` | 同向 | Same Orientation | → | 两个图元方向一致 |
| `COLLINEAR` | 共线 | Collinear | — | 两条线段位于同一直线上 |
| `CORADIAL` | 同心等径 | Coradial | ◎ | 两个弧/圆共享相同半径和圆心 |
| `EQUAL_CURVATURE` | 等曲率 | Equal Curvature | G2 | 样条间的曲率连续性 (G2) |
| `EQUAL_CURVE_LENGTH` | 等曲线长度 | Equal Curve Length | L= | 两条曲线长度相等 |
| `MERGE_POINTS` | 合并点 | Merge Points | ⊕ | 两个端点合并为一个 |
| `PIERCE` | 穿透 | Pierce | ⊗ | 点位于曲线上（草图图元穿过另一图元） |
| `FIX_SLOT` | 固定槽口 | Fix Slot | ⊟ | 固定槽图元的位置和方向 |
| `ON_EDGE` | 在边上 | On Edge | — | 草图图元位于模型边线上 |
| `ON_PLANE` | 在平面上 | On Plane | — | 草图图元位于参考平面上 |
| `ON_SURFACE` | 在曲面上 | On Surface | — | 草图图元位于模型曲面上 |
| `TANGENT_TO_FACE` | 与面相切 | Tangent to Face | ⌒ | 草图图元与模型面相切 |
| `TRACTION` | 牵引 | Traction | → | 扫掠引导曲线的牵引约束 |
| `TORSION_CONTINUITY` | 扭转连续性 | Torsion Continuity | G3 | 扫掠引导曲线的 G3 扭转连续性 |
### 约束操作 (EditorToolbar)

UI 上显示的约束按钮：

| 约束 | 按钮标签 | 说明 |
|------|---------|------|
| Horizontal | H | 水平约束 |
| Vertical | V | 垂直约束 |
| Coincident | ⦿ | 重合约束 |
| Parallel | ∥ | 平行约束 |
| Perpendicular | ⊥ | 正交约束 |
| Length | L= | 长度约束 |
| Distance | D= | 距离约束 |
| Angle | ∠ | 角度约束 |
| Fixed | ⚓ | 固定约束 |
| Tangent | ⌒ | 相切约束 |
| AtMidpoint | ◉ | 中点约束 |

---

## 6. 草图术语 (Sketch Terms)

| 中文 | 英文 | 说明 |
|------|------|------|
| 轮廓草图 | Profile Sketch | 定义实体截面的闭合草图 |
| 轴线草图 | Axis Sketch | 旋转特征的旋转轴线 |
| 截面草图 | Section Sketch | 放样/融合特征的截面轮廓 |
| 路径草图 | Path Sketch | 扫掠/放样融合的引导路径 |
| Z 高度 | Z Height / Origin Z | 草图平面在 Z 轴上的偏移量 |
| 法线 | Normal | 草图平面的法线方向 (0,0,1) 表示 XY 平面 |
| 原点 | Origin | 草图坐标系原点 (x,y,z) |
| 闭合 | Closed | 草图轮廓首尾相连形成封闭区域 |
| 开放 | Open | 草图轮廓未闭合 |
| 链式绘制 | Chain Drawing | 连续绘制模式，前一条线段的终点自动成为下一条的起点 |

### 多草图标签 (Multi-Sketch Labels)

| 特征类型 | 所需草图标签 | 说明 |
|---------|------------|------|
| revolve | 轮廓, 轴线 | 轮廓=截面，轴线=旋转轴 |
| sweep | 轮廓, 路径 | 轮廓=截面，路径=引导线 |
| loft | 截面1, 截面2 | 多个截面连接 |
| fusion | 截面1, 截面2 | 两个截面过渡 |
| lofted_fusion | 截面1, 截面2, 路径 | 两个截面沿路径过渡 |

---

## 7. 三维几何类型 (3D Geometry Types)

**参考几何**（源自 SOLIDWORKS，`docs/research/solidworks_components.md §9`）：7 种参考几何类型支持装配体和特征构建：
1. **Plane（基准面）**——用于草图和特征的构造平面
2. **Axis（基准轴）**——用于旋转/阵列/装配配合的构造轴
3. **Coordinate System（坐标系）**——局部坐标系参考
4. **Reference Point（参考点）**——三维空间中的定位点
5. **Center of Mass Point（质心点）**——对象上的动态质心点
6. **Bounding Box（包围盒）**——用于拟合/空间分析的有向包围盒
7. **Grid System（网格系统）**——用于阵列/装配布置的二维网格布局

这些作为 `ReferenceGeometry` 子对象存储在对象的特征树中，与草图和特征并列。

| 枚举值 | 中文 | 英文 | 说明 |
|--------|------|------|------|
| `GEOMETRY_TYPE_PARAMETRIC` (1) | 参数化 | Parametric | 由参数和公式定义的几何 |
| `GEOMETRY_TYPE_BREP` (2) | 边界表示 | B-Rep | 边界表示法实体 |
| `GEOMETRY_TYPE_MESH` (3) | 网格 | Mesh | 三角网格表示 |
| `GEOMETRY_TYPE_POINT_CLOUD` (4) | 点云 | Point Cloud | 三维点集合 |
| `GEOMETRY_TYPE_CURVE` (5) | 曲线 | Curve | 三维曲线 |
| `GEOMETRY_TYPE_EXTERNAL` (6) | 外部引用 | External | 引用外部文件 |

### 统一几何内核图元

| 图元 | 中文 | 说明 |
|------|------|------|
| Box | 长方体 | 六面体基本体 |
| Cylinder | 圆柱体 | 圆形截面柱体 |
| Cone | 圆锥体 | 圆形截面锥体 |
| Sphere | 球体 | 球形基本体 |
| Extrusion | 拉伸体 | 草图拉伸形成的实体 |
| Sweep | 扫掠体 | 草图沿路径扫掠形成的实体 |
| Revolve | 旋转体 | 草图绕轴旋转形成的实体 |
| Loft | 放样体 | 多个截面连接形成的实体 |

### 布尔运算

| 枚举值 | 中文 | 英文 | 说明 |
|--------|------|------|------|
| `BOOLEAN_UNSPECIFIED` (0) | 未指定 | Unspecified | 默认值（非有效操作） |
| `Union` (1) | 并集 | Union | 合并两个实体 |
| `Subtract` (2) | 差集 | Subtract | 从一个实体中减去另一个 |
| `Intersect` (3) | 交集 | Intersect | 取两个实体的公共部分 |

---

## 8. 操作类型 (Action Types)

所有操作记录在 OMCOM 变更日志中。

### 对象操作

| 操作 | 中文 | 说明 |
|------|------|------|
| `CREATE_OBJECT` | 创建对象 | 创建新的 OmniObject |
| `MODIFY_OBJECT` | 修改对象 | 修改对象属性 |
| `DELETE_OBJECT` | 删除对象 | 删除对象 |
| `COPY_OBJECT` | 复制对象 | 复制对象 |
| `MOVE_OBJECT` | 移动对象 | 移动对象位置 |

### 几何操作

| 操作 | 中文 | 说明 |
|------|------|------|
| `CREATE_GEOMETRY` | 创建几何 | 创建新的几何体 |
| `MODIFY_GEOMETRY` | 修改几何 | 修改几何参数 |
| `DELETE_GEOMETRY` | 删除几何 | 删除几何体 |
| `TRANSFORM_GEOMETRY` | 变换几何 | 平移/旋转/缩放 |
| `BOOLEAN_OPERATION` | 布尔运算 | 并集/差集/交集 |

### 属性操作

| 操作 | 中文 | 说明 |
|------|------|------|
| `SET_PROPERTY` | 设置属性 | 设置对象属性值 |
| `DELETE_PROPERTY` | 删除属性 | 删除对象属性 |
| `ADD_PROPERTY_SET` | 添加属性集 | 添加属性集 |
| `REMOVE_PROPERTY_SET` | 删除属性集 | 删除属性集 |

### 关系操作

| 操作 | 中文 | 说明 |
|------|------|------|
| `ADD_CONTAINMENT` | 添加包含关系 | 建立父子包含关系 |
| `REMOVE_CONTAINMENT` | 删除包含关系 | 移除包含关系 |
| `ADD_CONNECTION` | 添加连接关系 | 建立连接关系 |
| `REMOVE_CONNECTION` | 删除连接关系 | 移除连接关系 |

### 批量操作

| 操作 | 中文 | 说明 |
|------|------|------|
| `ARRAY` | 阵列 | 线性/环形阵列复制 |
| `BATCH_DELETE` | 批量删除 | 批量删除多个对象 |
| `BATCH_TRANSFORM` | 批量变换 | 批量变换多个对象 |

### 材料操作

| 操作 | 中文 | 说明 |
|------|------|------|
| `SET_MATERIAL` | 设置材料 | 为对象分配材料 |

### 生命周期操作

| 操作 | 中文 | 说明 |
|------|------|------|
| `CREATE_LIFECYCLE_RECORD` | 创建生命周期记录 | 创建新记录 |
| `MODIFY_LIFECYCLE_RECORD` | 修改生命周期记录 | 更新记录内容 |
| `DELETE_LIFECYCLE_RECORD` | 删除生命周期记录 | 删除记录 |
| `LINK_RECORD_TO_OBJECT` | 关联记录与对象 | 将记录绑定到对象 |
| `UPDATE_SCHEDULE` | 更新进度计划 | 更新进度数据 |
| `UPDATE_COST` | 更新成本 | 更新成本数据 |
| `UPDATE_PROGRESS` | 更新进度 | 更新实际进度 |
| `CREATE_WORK_ORDER` | 创建工单 | 创建运维工单 |
| `UPDATE_WORK_ORDER` | 更新工单 | 更新工单状态 |
| `INGEST_SENSOR_DATA` | 采集传感器数据 | 导入传感器数据 |

### 导入/导出

| 操作 | 中文 | 说明 |
|------|------|------|
| `IMPORT_FILE` | 导入文件 | 导入 IFC/STEP 等文件 |
| `EXPORT_FILE` | 导出文件 | 导出模型文件 |
| `PARSE_TO_OMCOM` | 解析为 OMCOM | 将文件解析为 OMCOM 格式 |

### 草图操作

| 操作 | 中文 | 说明 |
|------|------|------|
| `CREATE_SKETCH` | 创建草图 | 创建新的草图 |
| `ADD_SKETCH_ENTITY` | 添加图元 | 向草图添加几何图元 |
| `ADD_SKETCH_CONSTRAINT` | 添加约束 | 向草图添加约束 |
| `SOLVE_SKETCH` | 求解草图 | 运行约束求解器 |
| `DELETE_SKETCH_ENTITY` | 删除图元 | 从草图删除图元 |
| `REMOVE_SKETCH_CONSTRAINT` | 删除约束 | 从草图删除约束 |
| `DELETE_SKETCH` | 删除草图 | 删除整个草图 |

### 特征操作

| 操作 | 中文 | 说明 |
|------|------|------|
| `CREATE_COMPONENT` | 创建构件 | 创建新的构件定义 |
| `ADD_FEATURE` | 添加特征 | 向构件添加特征（拉伸/旋转等） |
| `REMOVE_FEATURE` | 删除特征 | 从构件删除特征 |
| `REBUILD_FEATURES` | 重建特征 | 重新计算所有特征的几何 |

### 管道编辑操作（Pipe Run Operations，Civil 3D）

> **调研参考**：Civil 3D 管道编辑操作（`docs/research/civil3d_components.md §6.3`）。这些操作编辑管网中的单条管道。

| 操作 | 中文 | 说明 |
|------|------|------|
| `EXTEND_PIPE_RUN` | 延长管道 | 延长一条管道及其连接的构筑物 |
| `SPLIT_PIPE_RUN` | 拆分管道 | 在某桩号处将一条管道拆分为两条 |
| `MERGE_PIPE_RUNS` | 合并管道 | 将两条相邻管道合并为一条 |
| `REVERSE_PIPE_RUN` | 反转管道 | 反转一条管道的流向 |
| `STRUCTURE_SWAP` | 替换结构 | 将管网中的构筑物替换为不同类型 |
| `ADD_TO_PIPE_RUN` | 添加至管道 | 向现有管道添加管段/构件 |

### 验证

| 操作 | 中文 | 说明 |
|------|------|------|
| `VALIDATE` | 验证 | 验证模型数据完整性 |

---

## 9. 材料类别 (Material Categories)

| 枚举值 | 中文 | 英文 | 说明 |
|--------|------|------|------|
| `MATERIAL_CATEGORY_METAL` (1) | 金属 | Metal | 钢、铝、铜等 |
| `MATERIAL_CATEGORY_CONCRETE` (2) | 混凝土 | Concrete | 普通/预应力混凝土 |
| `MATERIAL_CATEGORY_WOOD` (3) | 木材 | Wood | 木材/胶合木 |
| `MATERIAL_CATEGORY_MASONRY` (4) | 砌体 | Masonry | 砖/石砌体 |
| `MATERIAL_CATEGORY_PLASTIC` (5) | 塑料 | Plastic | 高分子材料 |
| `MATERIAL_CATEGORY_COMPOSITE` (6) | 复合材料 | Composite | 碳纤维/玻璃纤维等 |
| `MATERIAL_CATEGORY_GLASS` (7) | 玻璃 | Glass | 建筑玻璃 |
| `MATERIAL_CATEGORY_INSULATION` (8) | 保温材料 | Insulation | 保温/隔热材料 |
| `MATERIAL_CATEGORY_SOIL` (9) | 土壤 | Soil | 土方材料 |
| `MATERIAL_CATEGORY_WATER` (10) | 水 | Water | 水体 |
| `MATERIAL_CATEGORY_OTHER` (99) | 其他 | Other | 未分类材料 |

---

## 10. 生命周期阶段 (Lifecycle Phases)

| 枚举值 | 中文 | 英文 | 说明 |
|--------|------|------|------|
| `LIFECYCLE_PHASE_PLANNING` (1) | 规划 | Planning | 项目立项、可行性研究 |
| `LIFECYCLE_PHASE_DESIGN` (2) | 设计 | Design | 建筑/结构/MEP/土木/工厂/机械设计 |
| `LIFECYCLE_PHASE_PROCUREMENT` (3) | 采购 | Procurement | 招标、合同、材料设备采购 |
| `LIFECYCLE_PHASE_CONSTRUCTION` (4) | 施工 | Construction | 施工建造、进度管理 |
| `LIFECYCLE_PHASE_COMMISSIONING_ACCEPTANCE` (5) | 调试验收 | Commissioning & Acceptance | 调试、检验、试运行 |
| `LIFECYCLE_PHASE_ASSET` (6) | 资产 | Asset | 资产登记、保修设置 |
| `LIFECYCLE_PHASE_OPERATION_MAINTENANCE` (7) | 运维 | Operation & Maintenance | 运营维护、设施管理 |
| `LIFECYCLE_PHASE_PRODUCTION` (8) | 生产 | Production | 工业生产运行 |
| `LIFECYCLE_PHASE_DECOMMISSIONING` (9) | 退役拆除 | Decommissioning | 报废、拆除、场地恢复 |

---

## 11. 生命周期记录状态 (Record Status)

| 枚举值 | 中文 | 英文 | 说明 |
|--------|------|------|------|
| `RECORD_STATUS_DRAFT` (1) | 草稿 | Draft | 初始草稿状态 |
| `RECORD_STATUS_OPEN` (2) | 待处理 | Open | 等待处理 |
| `RECORD_STATUS_IN_PROGRESS` (3) | 进行中 | In Progress | 正在处理 |
| `RECORD_STATUS_BLOCKED` (4) | 阻塞 | Blocked | 被阻塞无法继续 |
| `RECORD_STATUS_COMPLETED` (5) | 已完成 | Completed | 处理完成 |
| `RECORD_STATUS_APPROVED` (6) | 已批准 | Approved | 已通过审批 |
| `RECORD_STATUS_REJECTED` (7) | 已拒绝 | Rejected | 未通过审批 |
| `RECORD_STATUS_CLOSED` (8) | 已关闭 | Closed | 已关闭归档 |

---

## 12. 对象状态 (Object Status)

| 枚举值 | 中文 | 英文 | 说明 |
|--------|------|------|------|
| `OBJECT_STATUS_DRAFT` (1) | 草稿 | Draft | 设计中 |
| `OBJECT_STATUS_PROPOSED` (2) | 提议 | Proposed | 已提出待审核 |
| `OBJECT_STATUS_APPROVED` (3) | 已批准 | Approved | 已批准 |
| `OBJECT_STATUS_CONSTRUCTION` (4) | 施工中 | Construction | 正在施工 |
| `OBJECT_STATUS_AS_BUILT` (5) | 竣工 | As-Built | 已完工 |
| `OBJECT_STATUS_OPERATIONAL` (6) | 运营中 | Operational | 正常运营 |
| `OBJECT_STATUS_DECOMMISSIONED` (7) | 已退役 | Decommissioned | 已退役 |

---

## 13. 关系类型 (Relationship Types)

### 包含关系 (Containment Types)

| 枚举值 | 中文 | 英文 | 说明 |
|--------|------|------|------|
| `CONTAINMENT_TYPE_SPATIAL` (1) | 空间包含 | Spatial | 建筑→楼层→房间 |
| `CONTAINMENT_TYPE_PHYSICAL` (2) | 物理包含 | Physical | 组件→特征→图元 |
| `CONTAINMENT_TYPE_SYSTEM` (3) | 系统包含 | System | 系统→子系统 |
| `CONTAINMENT_TYPE_GROUP` (4) | 分组包含 | Group | 逻辑分组 |

### 连接关系 (Connection Types)

| 枚举值 | 中文 | 英文 | 说明 |
|--------|------|------|------|
| `CONNECTION_TYPE_STRUCTURAL` (1) | 结构连接 | Structural | 结构受力连接 |
| `CONNECTION_TYPE_MECHANICAL` (2) | 机械连接 | Mechanical | 机械连接/传动 |
| `CONNECTION_TYPE_ELECTRICAL` (3) | 电气连接 | Electrical | 电气连接 |
| `CONNECTION_TYPE_HYDRAULIC` (4) | 水力连接 | Hydraulic | 管道/水力连接 |
| `CONNECTION_TYPE_SPATIAL_ADJACENT` (5) | 空间相邻 | Spatial Adjacent | 空间相邻关系 |

---

## 14. 空间关系 (Spatial Relations)

| 中文 | 英文 | 说明 |
|------|------|------|
| 位于...内部 | Inside | 对象 A 在对象 B 内部 |
| 位于...外部 | Outside | 对象 A 在对象 B 外部 |
| 穿过 | Through | 对象 A 穿过对象 B |
| 相邻 | Adjacent | 对象 A 与 B 相邻 |
| 包围 | Envelops | 对象 A 包围对象 B |
| 分隔 | Separates | 对象 A 分隔对象 B 与 C |
| 连接 | Connects | 对象 A 连接对象 B 与 C |
| 悬挂于 | SuspendedFrom | 对象 A 悬挂于对象 B |
| 支撑 | Supports | 对象 A 支撑对象 B |
| 嵌入 | EmbedsIn | 对象 A 嵌入对象 B |
| 附着于 | AttachedTo | 对象 A 附着于对象 B |
| 贯穿 | Penetrates | 对象 A 贯穿对象 B |
| 邻近 | Near | 对象 A 邻近对象 B |
| 对齐 | AlignsWith | 对象 A 与 B 对齐 |
| 在...上方 | Above | 对象 A 在 B 上方 |
| 在...下方 | Below | 对象 A 在 B 下方 |

---

## 15. 分解层级 (Breakdown Levels)

| 枚举值 | 中文 | 英文 | 说明 |
|--------|------|------|------|
| `BREAKDOWN_LEVEL_PROGRAM` (1) | 计划 | Program | 项目群/计划 |
| `BREAKDOWN_LEVEL_PROJECT` (2) | 项目 | Project | 单个项目 |
| `BREAKDOWN_LEVEL_SUB_PROJECT` (3) | 子项目 | Sub-Project | 项目子项 |
| `BREAKDOWN_LEVEL_UNIT` (4) | 单元 | Unit | 工艺单元/建筑单元 |
| `BREAKDOWN_LEVEL_BUILDING` (5) | 建筑 | Building | 建筑物 |
| `BREAKDOWN_LEVEL_STOREY` (6) | 楼层 | Storey | 建筑楼层 |
| `BREAKDOWN_LEVEL_ZONE` (7) | 区域 | Zone | 功能区域 |
| `BREAKDOWN_LEVEL_SYSTEM` (8) | 系统 | System | 机电系统/结构系统 |
| `BREAKDOWN_LEVEL_ELEMENT` (9) | 构件 | Element | 单个构件 |
| `BREAKDOWN_LEVEL_PART` (10) | 零件 | Part | 机械零件 |

### 分解惯例 (Breakdown Conventions)

| 枚举值 | 中文 | 英文 | 适用领域 |
|--------|------|------|---------|
| `BREAKDOWN_CONVENTION_BUILDING` (1) | 建筑 | Building | 建筑工程 |
| `BREAKDOWN_CONVENTION_PLANT` (2) | 工厂 | Plant | 工厂设计 |
| `BREAKDOWN_CONVENTION_CIVIL` (3) | 土木 | Civil | 土木工程 |
| `BREAKDOWN_CONVENTION_MECHANICAL` (4) | 机械 | Mechanical | 机械设计 |

---

## 16. 行政区划级别 (Administrative Levels)

| 枚举值 | 中文 | 英文 | 说明 |
|--------|------|------|------|
| `ADMINISTRATIVE_LEVEL_WORLD` (1) | 世界 | World | 全球级别 |
| `ADMINISTRATIVE_LEVEL_COUNTRY` (2) | 国家 | Country | 国家 |
| `ADMINISTRATIVE_LEVEL_PROVINCE` (3) | 省份 | Province | 省/州 |
| `ADMINISTRATIVE_LEVEL_PREFECTURE` (4) | 地区 | Prefecture | 地级市/地区 |
| `ADMINISTRATIVE_LEVEL_COUNTY` (5) | 县 | County | 县/区 |
| `ADMINISTRATIVE_LEVEL_TOWNSHIP` (6) | 乡镇 | Township | 乡镇/街道 |
| `ADMINISTRATIVE_LEVEL_COMMUNITY` (7) | 社区 | Community | 社区/村 |
| `ADMINISTRATIVE_LEVEL_BLOCK` (8) | 地块 | Block | 地块 |
| `ADMINISTRATIVE_LEVEL_PARCEL` (9) | 宗地 | Parcel | 宗地 |

---

## 17. 生成规则 (GenRule Types)

> **⚠️ 第六阶段 — 两个层次，弃用状态不同：**
>
> | 层次 | 内容 | 状态 |
> |------|------|------|
> | **GenRule 作为分类** | `GenRuleHint` 枚举，`primary_gen_rule()` 映射（Wall→LayerExtrusion、Beam→LinearComponent 等） | ✅ **活跃**——作为领域几何结构的分类法，用于 `default_catalog_component()`（模板选择）和 `default_series_for_type()`（系列选择） |
> | **GenRule 作为生成** | `generate_geometry()` 调用 `build_wall()`、`build_pipe()` 等直接生成原始网格 | ❌ **已弃用**——被目录特征树模板（`FeatureTreeTemplate` → Feature Engine → BRep）取代 |
>
> 下面的 5 种 GenRule 类型描述每个对象*有什么类型*的几何，但实际的*几何生成*现在通过**默认模板注册中心**和**构件目录**完成。

GenRule 是将领域语义映射到几何图元的规则系统。

> **概念层**：GenRule 实现了**四类构件分类**（面/线/点/体构件，见 [数据模型 §1](../specification/DATA_MODEL.zh-CN.md#1-概述)）：LayerExtrusion → 面构件，LinearComponent → 线构件，Primitive/Assembly → 体构件，PointCloud/TerrainModel（非 GenRule）→ 点构件。

| 类型 | 中文 | 说明 | 典型对象 | 构件类别 |
|------|------|------|---------|---------|
| (1) Primitive | 基本体 | 直接使用基本几何体 | 罐、泵、阀门 | 体构件 |
| (2) LayerExtrusion | 分层拉伸 | 将面边界 + 层结构展开为多条拉伸；**面构件**的主要 GenRule | 墙、楼板、屋顶 | 面构件 |
| (3) LinearComponent | 线性构件 | 沿路径排列的构件，变截面 + 外部约束；**线构件**的 GenRule | 梁、柱、道路、桥梁、管道、大坝、电缆桥架（弯头） | 线构件 |
| (4) Assembly | 装配体 | 基本体组合，支持条件参数 + ConnectionScript | 设备组件、齿轮、钢结构节点 | 体构件 |
| (5) BooleanSubtract | 布尔减法 | 从实体中挖去部分（实体-实体、平面切割、边倒角） | 门窗洞口、管道开孔、Tekla Fitting | — |
| (—) PointCloud / TerrainModel | 点/曲面 | 点定义或 TIN 曲面几何；不属于 5 种 GenRule | 点云、地形曲面 | 点构件 |

---

---

## 17a. 默认模板注册中心 (Default Template Registry)

> **Session 级全局**：默认模板注册中心是 session 级全局函数（位于 gen_rule 模块），与 Library 一起在 session 级提供，不属于 Project 或 Library 结构体。

将每个 `ObjectType` 映射到一个默认的目录构件，使得 `create <type> <name>` 自动从目录实例化一个参数化构件。

| ObjectType | 默认目录构件 | 领域 |
|-----------|-------------|------|
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

## 17b. 系列库 (Series Library / 等级库)

> **独立模块**：SeriesStore 是独立模块，不属于 Library 或 Project。它定义选择器决策树（Series → Selector → SpecComponent），通过 ComponentDef ID 引用 Library 内容。系列可以存在于：(1) 项目私有 SeriesStore（嵌入 `.omnim` 文件），(2) 外部 `.omnimseries` 文件按路径引用，(3) 系统内置种子系列。只有通过项目系列可达的构件才能使用（series gate）。

| 系列名称 | 领域 | 标准 | 构件 |
|---------|------|------|------|
| CS150 | Plant/Piping | ASME B31.3 | Pipe-CS, Elbow-LR-CS, EqualTee-CS, Reducer-Conc-CS, Flange-WN-CS, GateValve-CS |
| SS300 | Plant/Piping | ASME B31.3 | Pipe-CS (SS304), Elbow-LR-CS (SS304), Flange-WN-CS (SS304) |
| Steel-Q355 | Structural | EN 1993-1-1 | Steel-Beam-IPE, Steel-Column-HEA |
| Concrete-C30 | Building | EN 1992-1-1 | Concrete-Beam-Rect, Concrete-Column-Rect |
| Highway-GB | Civil | GB JTG B01 | Road-Asphalt, Road-Concrete |

注：`SeriesStore` 设计为独立模块。历史上与 `CatalogStore` 同位于 `crates/model/src/catalog.rs`，计划拆分为独立模块（`series_store.rs`）。

**截面类型库**：基于 Tekla Structures 型材目录（`docs/research/tekla_components.md §2.1`），目录定义了 17 种截面类型。这些是结构 `ComponentDef` 横截面选取的型材族：

| # | 截面 | 英文 | 说明 |
|---|------|------|------|
| 0 | 自由 | Free | 自定义/用户截面 |
| 1 | I | I-beam | 标准工字钢 |
| 2 | 焊接空心 | Welded hollow | 箱形/矩形空心截面 |
| 3 | U | U-channel | 槽钢 |
| 4 | L | L-angle | 角钢 |
| 5 | 圆棒 | Round bar | 实心圆钢 |
| 6 | 圆管 | Round tube | 空心圆管 |
| 7 | RHS | RHS | 矩形空心截面 |
| 8 | T | T-section | 结构 T 型钢 |
| 9 | 矩形棒 | Rectangular bar | 实心矩形 |
| 10 | Z | Z-section | Z 型檩条 |
| 11 | C | C-section | C 型檩条 |
| 12 | Ω | Omega | 欧米伽型檩条 |
| 13 | Σ | Sigma | 西格玛型檩条 |
| 14 | 轨道 | Rail | 起重机轨道 |
| 16 | 钢筋 | Rebar | 钢筋 |

这些映射到 OmniM `ComponentDef` 截面（如 IPE、HEA、W、角钢、槽钢、RHS、钢筋），由目录引用并通过 Series gate 验证。

## 17c. 目录实例化管线 (Catalog Instantiation Pipeline)

从用户命令到生成几何的完整流程。注意：Project **不持有** Library 引用，Executor 通过全局静态 `LIBRARY` 访问。

```
create wall W1 --param=thickness=200
  ↓
Executor 解析:
  default_catalog_component(Wall) → "Concrete-Wall"  ← Library (全局静态)
  ↓
LIBRARY.catalog.find_by_name("Concrete-Wall") → ComponentDef  ← Library
  ↓
┌─ Series Gate ─────────────────────────────────┐
│  该构件是否在项目已加载的系列的可达范围内？      │
│  - 是：继续                                    │
│  - 未加载系列：开放模式，允许                   │
│  - 系列已加载但构件不在范围内：拒绝              │
└───────────────────────────────────────────────┘
  ↓
ComponentDef.feature_tree_id → FeatureTreeTemplate  ← Library
  ↓
instantiate_template() → 创建草图+特征              ← Project
  ↓
Project.rebuild_component() → 特征引擎 → BRep → Mesh
  ↓
Project 中的对象，带缓存几何、特征历史、
系列引用和目录引用 (CatalogRef)
```

## 17d. Library vs Project 架构 (Library vs Project Architecture)

三个独立前端模块对应后端三层架构：

```
┌─────────────────────────────────────────────────────┐
│  Library (session 级全局静态)                        │
│  • CatalogStore            ← ComponentDef 条目      │
│  • ComponentStore          ← Component 模板         │
│  • FeatureTreeRegistry     ← 特征树模板             │
│  • 默认模板注册中心          ← ObjectType→Def 映射  │
│                                                      │
│  一次加载，跨项目共享                                │
│  通过全局静态 LIBRARY 访问（Project 不持有）         │
│  前端：Library Manager（独立管理界面）                │
└──────────────────────┬──────────────────────────────┘
                       │ 通过 ComponentDef ID 引用
              ┌────────▼────────────────────────┐
              │  SeriesStore (独立模块)           │
              │  • Series: 决策树                │
              │    - 内置种子（只读）             │
              │    - 外部 .omnimseries 文件       │
              │  • SpecComponent → ComponentDef  │
              │  • SelectionEngine               │
              │  前端：SeriesStore Manager         │
              └────────┬────────────────────────┘
                       │ 被 Project 引用
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
     │ 前端：      │  │          │  │          │
     │ Project    │  │          │  │          │
     │ Workspace  │  │          │  │          │
     └────────────┘  └──────────┘  └──────────┘

  规则：只有通过项目已加载系列可达的 ComponentDef 才能被实例化。
  如果未加载任何系列，则为开放模式，所有目录构件可用。
```

---

## 17e. 地形模型 (TerrainModel TIN 曲面)

> **调研参考**：Bentley OpenRoads Terrain Model（`docs/research/bentley_openroads_components.md §6.1`）。

`TerrainModel` 是 5 类 GenRule 之外的第 6 种几何类型。它表示不规则三角网（TIN）曲面，**不属于**扫掠/放样/布尔中的任一类。参见 [数据模型 §6.8](../specification/DATA_MODEL.zh-CN.md#68-地形模型-terrainmodel-tin-曲面)。

| 概念 | 中文 | 说明 |
|------|------|------|
| TIN surface | 三角网曲面 | 由点 + 断裂线 + 三角形定义的曲面 |
| BreakLine | 断裂线 | 三角形不可跨越的边 |
| TerrainRole | 地形角色 | Existing / Design / Subgrade / Reference |
| VolumeOption | 计方选项 | Existing / Design / Subgrade / None |

**角色系统（TerrainRole）**：`TerrainRole` 枚举控制曲面如何参与体积计算：`TERRAIN_EXISTING`（"之前"的曲面，被减）、`TERRAIN_DESIGN`（"之后"的设计面）、`TERRAIN_SUBGRADE`（独立计方的路基面，如道路基层）、`TERRAIN_REFERENCE`（参考面，不参与体积计算）。

**派生对象**（源自 `docs/research/site_terrain_components.md §7.3`）：TerrainModel 是下列对象的基础：**CutVolume**（挖方量，Existing − Design）、**FillVolume**（填方量，Design − Existing）、**GradingSurface**（角色为 `TERRAIN_DESIGN` 的整平面，由 Grading Operation 创建）、**Grading Operation**（一组整平规则，坡度比 `x:1`、目标曲面、投影）、**EarthworkBalance**（比较有界区域挖填总量的逻辑对象）、**Slope**（横断面为坡面、沿边界/边线扫掠的 LinearComponent，带 ExternalTarget）、**SiteComponent**（附着于地形表面的 ComponentDef，如树木、电线杆、消防栓）、**FeatureLine**（零厚度控制/特征线，用作整平参考）、**ContourLine**（固定高程等高线，用于显示/分析）。

用途：LinearComponent End Condition 的 daylight 求解、挖/填方体积计算（两 TIN 曲面间 BooleanSubtract）、地形/土方可视化。

## 17f. 逻辑对象 (LogicalObject)

> **调研参考**：AVEVA Instrumentation Loop/Tag（`docs/research/e3d_instrumentation_components.md §4`）。

`LogicalObject` 是无几何的工程实体（Tag/Loop/LineNumber），通过 `member_object_ids` 引用物理对象。是与 `Object` 并列的新基类型。参见 [数据模型 §6.9](../specification/DATA_MODEL.zh-CN.md#69-逻辑对象logicalobject-tag--loop--linenumber)。

| 概念 | 中文 | 说明 |
|------|------|------|
| Tag | 位号 | ISA-5.1 仪表位号（如 "FT-101A"） |
| Loop | 回路 | 控制回路，分组多个位号 |
| LineNumber | 管线号 | 工艺管线号（如 12"-3A1-CS-SCH40） |

关键原则：无几何、无 GenRule、无特征树；持有物理对象引用；支持跨视图追溯（P&ID ↔ 3D）。

**ISA-5.1 符号约定**（`docs/research/e3d_instrumentation_components.md §2.1.1`）：ISA-5.1 标准定义了仪表符号形状，用于提高 P&ID 可读性：
- 圆圈 = 现场安装仪表
- 圆圈 + 单条水平线 = 控制室/盘装仪表
- 圆圈 + 双条水平线 = 现场盘装仪表
- 圆圈内接正方形 = DCS（集散控制系统）功能
- 信号线类型：虚线 = 电气，双虚线 = 气动，带双斜线的实线 = 毛细管

## 17g. 约束 (Constraint Mates)

> **调研参考**：SOLIDWORKS 21 类配合（`docs/research/solidworks_components.md §4`）。

`Constraint` 约束 Assembly 子对象之间的自由度关系。存储在 `RelationStore` 中，与连接并列。参见 [数据模型 §6.12](../specification/DATA_MODEL.zh-CN.md#612-约束系统constraint-mates)。

| 类别 | 中文 | 类型 |
|------|------|------|
| Standard | 标准配合 | Coincident, Concentric, Parallel, Perpendicular, Tangent, Distance, Angle, Lock |
| Advanced | 高级配合 | Profile Center, Symmetric, Width, Path Mate, Linear/Linear Coupler, Limit |
| Mechanical | 机械配合 | Cam, Slot, Hinge, Gear, Rack Pinion, Screw, Universal Joint |

## 17h. ConnectionScript

> **调研参考**：Tekla ConnectionBase（`docs/research/tekla_components.md §3`）。

`ConnectionScript` 是 Assembly GenRule 使用的过程化脚本构件引用，用于生成钢结构节点（由条件参数驱动可选的加劲肋、端板、螺栓等）。参见 [数据模型 §6.11](../specification/DATA_MODEL.zh-CN.md#611-assembly-增强)。

**Tekla ConnectionBase 插件架构**（`docs/research/tekla_components.md §3.3`）：ConnectionScript 遵循 Tekla ConnectionBase 契约：
1. **输入契约**：脚本接收主部件、次部件、位置和多边形（通过 `ComponentInput` 模式——`AddInputObject`、`AddInputPositions`、`AddInputPolygon`；对顺序敏感）
2. **插件属性**：`[Plugin("name")]`、`[PluginUserInterface]`、`[SecondaryType]`、`[AutoDirectionType]`、`[PositionType]`——这些为脚本标注 UI 提示和部件选择行为
3. **运行契约**：`Run()` 方法基于输入部件和条件参数生成子构件（板、螺栓、焊缝）
4. **示例**：端板节点（144）参数——板位置、厚度、宽度、螺栓布置、焊缝尺寸、加劲肋选项、加腋选项

## 17i. 插值方法 (InterpolationMethod)

> **调研参考**：Bentley OBM 拼装梁变截面（`docs/research/bentley_openroads_components.md §2.1.3`）。

`CrossSectionSegment` 的横断面插值方法，从单一线性扩展为 6 种：Constant、Linear、Parabolic Up/Down、Circular Up/Down。参见 [数据模型 §6.4](../specification/DATA_MODEL.zh-CN.md#64-线构件-lineancomponent)。

---

## 18. 编辑器术语 (Editor Terms)

### 工具 (Tools)

| 工具 | 快捷键 | 中文 | 说明 |
|------|--------|------|------|
| Select | V | 选择 | 选择图元 |
| Line | L | 线段 | 绘制直线段 |
| Circle | C | 圆 | 绘制圆 |
| Arc | A | 圆弧 | 绘制圆弧 |
| Rect | R | 矩形 | 绘制矩形 |
| Polygon | P | 正多边形 | 绘制正多边形 |
| Ellipse | E | 椭圆 | 绘制椭圆 |
| Spline | S | 样条曲线 | 绘制样条曲线 |
| Polyline | N | 折线 | 绘制连续折线 |

### UI 操作

| 操作 | 中文 | 说明 |
|------|------|------|
| 吸附 | Snap | 自动捕捉到端点/中点/圆心 |
| 撤销 | Undo | 撤销上一步操作 |
| 重做 | Redo | 重做上一步操作 |
| 删除 | Delete / Backspace | 删除选中的图元 |
| 完成 | Enter / 右键 | 完成当前绘制 |
| 取消 | Escape | 取消当前操作 |
| 属性面板 | Properties Panel | 编辑选中图元属性 |
| 实体创建模式 | Feature Creation | 选择特征类型后进入草图绘制模式 |

### Snap 捕捉类型

| 捕捉 | 中文 | 英文 | 说明 |
|------|------|------|------|
| Endpoint | 端点 | Endpoint | 线段/弧的端点 |
| Midpoint | 中点 | Midpoint | 线段的中点 |
| Center | 圆心 | Center | 圆/弧的圆心 |
| Threshold | 捕捉阈值 | Snap Threshold | 8 像素 |

### 实体创建模式流程

1. 点击特征按钮（如"拉伸"）→ 进入创建模式
2. 根据所需草图标签（如"轮廓"）切换到对应草图
3. 在草图中绘制轮廓
4. 按 Enter/右键完成当前草图 → 自动切换到下一个草图
5. 所有草图完成后 → 弹出特征参数对话框
6. 确认参数 → 调用后端 API 创建特征

---

## 19. 领域术语 (Domain-Specific Terms)

### 管道工程 (Piping)

| 中文 | 英文 | 说明 |
|------|------|------|
| 管道等级 | Piping Spec | 管道材料等级规范 |
| 管件 | Pipe Fitting | 弯头、三通、变径等 |
| 大小头 | Reducer | 变径管件（圆→圆或方→方） |
| 天圆地方 | Round-to-Square | 圆形到方形的过渡件 |
| 管架 | Pipe Rack | 管道支撑架 |
| 管道支架 | Pipe Support | 管道支撑件 |
| 法兰 | Flange | 管道连接法兰 |
| 喷嘴 | Nozzle | 设备管口 |

### 结构工程 (Structural)

| 中文 | 英文 | 说明 |
|------|------|------|
| 配筋 | Rebar | 钢筋配置 |
| 荷载 | Load | 结构荷载 |
| 应力 | Stress | 材料应力 |
| 变形 | Deformation | 结构变形 |
| 支座 | Support | 结构支座 |

### 暖通空调 (HVAC)

| 中文 | 英文 | 说明 |
|------|------|------|
| 风管 | Duct | 空气输送管道 |
| 散流器 | Diffuser | 空气出风口 |
| 回风口 | Return Air Grille | 空气回入口 |
| 新风 | Fresh Air | 室外新鲜空气 |
| 冷冻水 | Chilled Water | 空调冷水 |
| 冷却水 | Cooling Water | 冷却塔循环水 |

### 电气工程 (Electrical)

| 中文 | 英文 | 说明 |
|------|------|------|
| 配电柜 | Switchboard | 电力分配柜 |
| 桥架 | Cable Tray | 电缆敷设桥架 |
| 母线 | Busbar | 电力母线 |
| 回路 | Circuit | 电气回路 |
| 接地 | Grounding | 接地系统 |

### 市政工程 (Municipal)

| 中文 | 英文 | 说明 |
|------|------|------|
| 路面 | Pavement | 道路面层 |
| 路肩 | Shoulder | 道路肩部 |
| 路缘石 | Curb | 道路边缘石 |
| 排水沟 | Drainage Channel | 排水设施 |
| 检查井 | Manhole | 地下管线检查井 |

---

## 20. API 端点 (API Endpoints)

| 端点 | 方法 | 中文 | 说明 |
|------|------|------|------|
| `/api/objects` | GET | 获取对象列表 | 查询所有对象 |
| `/api/objects/{id}` | GET | 获取对象详情 | 查询单个对象 |
| `/api/objects` | POST | 创建对象 | 创建新对象 |
| `/api/objects/{id}` | PUT | 更新对象 | 修改对象属性 |
| `/api/objects/{id}` | DELETE | 删除对象 | 删除对象 |
| `/api/objects/{id}/children` | GET | 获取子对象 | 查询子节点 |
| `/api/objects/{id}/tree` | GET | 获取对象树 | 查询完整层级树 |
| `/api/objects/{id}/mesh` | GET | 获取网格 | 获取三维网格数据 |
| `/api/stats` | GET | 获取统计 | 项目统计数据 |
| `/api/import/ifc` | POST | 导入 IFC | 导入 IFC 文件 |
| `/api/sketches` | POST | 创建草图 | 创建新草图 |
| `/api/sketches/{id}` | GET | 获取草图 | 查询草图详情 |
| `/api/sketches/{id}` | PUT | 更新草图 | 修改草图属性(含 Z 高度) |
| `/api/sketches/{id}/entities` | POST | 添加图元 | 向草图添加图元 |
| `/api/sketches/{id}/entities/{eid}` | PUT | 更新图元 | 修改图元属性 |
| `/api/sketches/{id}/entities/{eid}` | DELETE | 删除图元 | 删除图元 |
| `/api/sketches/{id}/constraints` | POST | 添加约束 | 向草图添加约束 |
| `/api/sketches/{id}/solve` | POST | 求解草图 | 运行约束求解 |
| `/api/components` | POST | 创建构件 | 创建新构件 |
| `/api/components/{id}` | GET | 获取构件 | 查询构件详情 |
| `/api/components/{id}/features` | POST | 添加特征 | 向构件添加特征 |
| `/api/components/{id}/features/{fid}` | GET | 获取特征 | 查询特征详情 |
| `/api/components/{id}/features/{fid}` | DELETE | 删除特征 | 删除特征 |
| `/api/components/{id}/rebuild` | POST | 重建构件 | 重新计算所有特征几何 |
| `/api/amir/translate` | POST | AI 翻译 | 自然语言转命令 |
| `/api/amir/chat` | POST | AI 对话 | 对话式建模 |
| `/api/amir/describe` | POST | AI 描述 | 生成对象描述 |
| `/api/amir/explain` | POST | AI 解释 | 解释错误信息 |

---

## 21. 角色类型 (Actor Types)

| 枚举值 | 中文 | 英文 | 说明 |
|--------|------|------|------|
| `USER` (1) | 用户 | User | 人类用户 |
| `AI_AGENT` (2) | AI 代理 | AI Agent | AI 辅助建模 |
| `SYSTEM` (3) | 系统 | System | 系统自动操作 |
| `IMPORT` (4) | 导入 | Import | 文件导入操作 |

---

## 22. 参数类型 (Parameter Types)

| 类型 | 说明 |
|------|------|
| `string_value` | 字符串参数 |
| `int_value` | 整数参数 |
| `double_value` | 浮点数参数 |
| `bool_value` | 布尔参数 |

---

## 23. 坐标系 (Coordinate Systems)

| 中文 | 英文 | 说明 |
|------|------|------|
| 世界坐标系 | World Coordinate System (WCS) | 全局固定坐标系 |
| 物体坐标系 | Object Coordinate System (OCS) | 单个对象的局部坐标系 |
| 草图坐标系 | Sketch Coordinate System | 草图平面的二维坐标系 |
| 局部坐标系 | Local Coordinate System (LCS) | 构件/特征的局部坐标系 |
| 投影坐标系 | Projected CRS (CRS) | 地理投影坐标系 |
| 地理坐标系 | Geographic CRS | 经纬度坐标系 |

---

## 24. 网格数据 (Mesh Data)

| 字段 | 中文 | 说明 |
|------|------|------|
| `vertices` | 顶点坐标 | 三维顶点坐标数组 [x,y,z,...] |
| `normals` | 法线 | 顶点法线数组 [nx,ny,nz,...] |
| `uvs` | UV 坐标 | 纹理 UV 坐标数组 [u,v,...] |
| `faces` | 三角面 | 三角面索引数组 [i0,i1,i2,...] |
| `draco_compressed` | Draco 压缩数据 | Draco 压缩的网格二进制数据 |

---

## 25. 存储模型 (Storage Model)

| 中文 | 英文 | 说明 |
|------|------|------|
| 对象存储 | Object Store | 以 JSON 格式存储所有对象数据 |
| 元数据存储 | Metadata Store | 对象索引和关系数据 |
| 原始文件存储 | Raw File Store | 原始 IFC/STEP 文件 |
| 向量索引 | Vector Index | 语义搜索向量索引 |
| 图关系索引 | Graph Index | 对象关系图索引 |

---

## 26. 协议/格式 (Protocols & Formats)

| 缩写 | 全称 | 中文 | 说明 |
|------|------|------|------|
| OMNIM | OmniM Internal Model | OmniM 内部模型 | 系统内部统一数据格式 |
| OMCOM | OmniM Command | OmniM 命令 | 统一类型化命令系统（OmniCommand 经 protobuf-as-JSON 类型序列化；Validator/Executor 位于 omnim-omcom）。Omcom 文本表面为 RFC-0001 草案，尚未实现。 |
| AMIR | AI Intermediate Representation | AI 中间表示 | 自然语言建模接口 |
| IFC | Industry Foundation Classes | 工业基础类 | 建筑行业数据交换标准 |
| STEP | Standard for the Exchange of Product Data | 产品数据交换标准 | ISO 10303 |
| B-Rep | Boundary Representation | 边界表示法 | 三维实体表示方法 |
| IGES | Initial Graphics Exchange Specification | 初始图形交换规范 | CAD 数据交换格式 |
| glTF | GL Transmission Format | GL 传输格式 | 三维模型传输格式 |
| PMI | Product Manufacturing Information | 产品制造信息 | 制造标注信息 |
| LOD | Level of Detail | 细节层次 | 模型精细度级别 |
| COBie | Construction Operations Building Information Exchange | 施工运维信息交换 | 运维数据交换格式 |
