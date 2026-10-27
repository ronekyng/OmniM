# OmniM 领域覆盖范围

## 1. 概述

OmniM 是一个统一的工程信息平台，支持四大工程领域，且不偏重任何一个领域：

1. 建筑工程
2. 土木工程
3. 3D 工厂设计
4. 机械设计

所有领域共享相同的核心数据模型、几何模型、属性系统、关系系统、AMIR 指令层和工程项目生命周期模型。OmniM 不仅是模型容器，还为每个领域管理需求、设计、采购、施工、调试验收、资产、运营、维护、生产以及退役数据。

---

## 2. 建筑工程

建筑工程涵盖建筑、结构、MEP 以及设施相关信息。

### 核心对象

| 类别 | 对象 |
|------|------|
| 空间 | Site, Building, BuildingStorey, Space, Zone |
| 建筑 | Wall, Door, Window, Roof, Ceiling, CurtainWall, Stair, Ramp |
| 结构 | Beam, Column, Slab, Footing, Pile, Brace, Truss, Rebar, Connection |
| MEP | Duct, Pipe, CableTray (直段→LayerExtrusion, 弯头→LinearComponent), Valve, Damper, Pump, Fan, AirTerminal |
| 设备 | Chiller, Boiler, AHU, Transformer, Switchboard, Fixture |

### 关键工作流
- BIM 创建与协调
- 工程量统计
- 碰撞检测
- 规范合规性检查
- 设施移交与资产管理

### 标准对齐
- IFC 2x3 / IFC 4 / IFC 4x3
- ISO 19650 信息管理
- COBie 资产移交

---

## 3. 土木工程

土木工程涵盖基础设施、交通、土地开发、公用设施以及大型场地工程。

### 核心对象

| 类别 | 对象 |
|------|------|
| 线形 | HorizontalAlignment, VerticalAlignment, Profile, Station |
| 道路 | Road, Lane, Shoulder, PavementLayer, Median, Curb |
| 桥梁 | Bridge, Pier, Abutment, Girder, Deck, Bearing, ExpansionJoint |
| 隧道 | Tunnel, Lining, Portal, Shaft, Segment, Ring, CrossPassage |
| 水利 | Dam, Culvert, Channel, Drainage, RetentionBasin |
| 公用设施 | WaterPipe, SewerPipe, StormPipe, Manhole, UtilityPole, DuctBank |
| 土石方 | TerrainModel, CutVolume, FillVolume, GradingSurface, Slope, FeatureLine, ContourLine, SiteComponent, EarthworkBalance, RetainingWall |

> **Bentley 桥梁/隧道构件细节**（`docs/research/bentley_openroads_components.md`）：
> - **支座类型**（§2.3.1）：板式橡胶支座（Elastomeric Pad）、盆式支座（Pot）、圆盘支座（Disc）、机械式支座（Mechanical）
> - **伸缩缝类型**（§2.4）：浅埋伸缩缝（Strip Seal）、模数式（Modular）、钢梳齿板式（Finger Plate）、压缩密封（Compression Seal）
> - **桥面伸缩缝断缝（Deck Breakbacks）**（§2.2.2）：桥面可在跨内通过断缝非连续，以适应伸缩缝或斜向分段。断缝建模为桥面 LinearComponent 轮廓序列中的有意图隙位置。
> - **桥墩配置**（§3.1.1）：桥墩类型（独柱式 / 多柱式 / 墙式 / 锤头式）；盖梁形状（锥形 / 矩形 / 倒 T 形 / 可变）；柱形（圆形 / 矩形 / 自定义）
> - **TBM 隧道管节环（UTR）**（§4.4.2）：锥度、销钉数量、首环位置、排除环位置、环名前缀
> - **TBM UTR 曲线拟合机制**（§4.4）：拼装式衬砌通过**管环锥度**拟合平曲线——每个通用管环有最大/最小宽度差（即*锥度*）。连续管环通过**销钉间距角**（由销钉数量导出，如 360°/N）相对旋转，使衬砌轴线沿设计曲线转向。管环排序遵循**继承**（跟随模板/首环）和**排除环位置**（不得与首环组合的位置）规则。这是实现曲线 TBM 推进的机械原理——在 OmniM 中建模为管环序列 `Assembly`，其每环锥度方向为序列 `VariationRule`。
> - **隧道模板目录**（§4）：常规/ TBM 隧道、洞口、竖井、洞室、开挖、衬砌、辅助设施、材料
>
> **Civil3D 管网规则**（`docs/research/civil3d_components.md §6.1`）：管网（重力流和压力流）受工程规则约束：最小覆土深度、最大/最小坡度、最小坡度、最大埋深、最小管长。这些由系列闸门（Series gate）作为管道 ComponentDef 参数上的 SpecComponent 约束强制执行。
>
> **管网零件列表（Pipe Network Parts List）**（源自 Civil3D §6.1）：一个管网使用网络范围的**零件列表**——该网络专用的管道+结构精选集合（如给定雨/污水系统可用的管径、管件和结构类型）。这在概念上区别于项目级系列决策树：零件列表绑定到单个网络，控制可在其中放置/扩展的零件，而系列是适用于所有构件的项目级规范。在 OmniM 中，网络范围的零件列表映射为在系列闸门之上叠加的网络本地目录/约束集。
>
> **Bentley 土木模型数据容器**（`docs/research/bentley_openroads_components.md §1.3`）：聚合线状构件、点状构件、3D 线性元素、地形模型、廊道、超高、土木对象、土木单元和参考模型。OmniM 的对应物是项目对象存储（Project Object Store），通过 ObjectType 分类和关系管理器（Relation Manager）实现跨对象关联。

### 关键工作流
- 廊道与线形建模
- 地形与整平
- 土石方计算
- 公用设施协调
- 基础设施资产管理

### 标准对齐
- IFC 4x3 基础设施扩展
- LandXML
- GeoJSON / Shapefile / GML
- 国家道路与桥梁标准

---

## 4. 3D 工厂设计

3D 工厂设计涵盖工艺工厂、工业设施、设备布置、管道、仪表以及结构支承。

### 核心对象

| 类别 | 对象 |
|------|------|
| 设备 | Tank, Vessel, Reactor, HeatExchanger, Pump, Compressor, Filter, Support |
| 管道 | Pipe, PipeSegment, Elbow, Tee, Reducer, Flange, Nozzle, Valve |
| 仪表 | Instrument, Sensor, Transmitter, ControlValve, JunctionBox, Cable, Conduit |
| 结构 | StructuralSteel, Platform, Stair, Ladder, Handrail, Support |
| 工艺 | ProcessLine, System, Unit, Area, PipingSpec, EquipmentSpec |
| 电气 | CableTray, Cable, Conduit, Panel, MotorControlCenter |

> **AVEVA 仪表/电气模块结构**（`docs/research/e3d_instrumentation_components.md §1.4-§1.5`）：AVEVA 仪表系统包含 4 个模块——Instrument Engineer（索引/规范）、Wiring Manager（端子/回路）、Designer（安装图）、Browser/Calculation 工具。AVEVA 电气系统包含 3 个模块——Engineer、Wiring Manager、Designer。OmniM 将这些统一为单一的逻辑对象 + 装配体模型：Instrument Engineer ↔ LogicalObject Manager、Wiring Manager ↔ Relation Manager、Designer ↔ Assembly GenRule。
>
> **MCC 层级装配体**（`docs/research/e3d_instrumentation_components.md §3.4.2`）：MotorControlCenter（MCC）是**3 级嵌套装配体**——`MCC → 柜体（Cubicle）→ 隔室（Compartment）→ 单元（Unit）`（断路器 + 接触器 + 过载继电器 + 控制变压器 + 端子排）——外加共享的**母线系统（Busbar System）**和**电缆槽（Cable Alley）**。它携带 3 类接线（动力/控制/辅助，按 IEC 61439）和启动方式（DOL / 星三角 / VFD）。
>
> **配电盘参数集**（`docs/research/e3d_instrumentation_components.md §3.3`）：`Panel` 携带配电盘参数——总断路器额定值/类型；各支路断路器额定值（MCB / MCCB / ELCB / RCCB）；回路数；备用回路数；每相负载 + 总和 + 需用系数 + 三相平衡；进线方式；以及 IP / NEMA / Ex 防爆等级。
>
> **支吊架作为装配体**（`docs/research/e3d_instrumentation_components.md §5.2.3`）：`Support`（支吊架）应为目录标准件的**装配体（Assembly）**——吊杆、弹簧盒、U 形螺栓/U 形夹、滑动支座、导向支座、锚固件、底板——而非单个图元。

### 关键工作流
- 设备布置
- 管道敷设
- 管道规范与轴测图
- 碰撞检测
- P&ID 关联
- 施工工作包

### 标准对齐
- ISO 15926
- P&ID 标准
- ASME 管道规范
- 在适用场合使用 IFC 设备与流段

---

## 5. 机械设计

机械设计涵盖零件、装配体、机械系统、可用于制造的几何以及产品结构。

### 核心对象

| 类别 | 对象 |
|------|------|
| 产品结构 | Assembly, SubAssembly, Part, Configuration, Variant |
| 机械元件 | Gear, Shaft, Bearing, Coupling, Spring, Seal, Bushing |
| 紧固件 | Bolt, Nut, Washer, Screw, Rivet, Pin, ThreadedInsert |
| 运动 | Joint, Constraint, Axis, KinematicPair, Actuator |
| 制造 | Hole, Pocket, Slot, Chamfer, Fillet, Thread, PMI |
| 机电 | Motor, Encoder, Sensor, Housing, Connector |

### 关键工作流
- 装配建模
- 精确 BRep 几何编辑
- 产品结构管理
- GD&T / PMI 处理
- STEP 导入/导出
- 物料清单生成

### 标准对齐
- STEP AP203 / AP214 / AP242
- ISO 10303
- ISO GPS / GD&T 标准
- 制造商目录标准

---

## 6. 跨领域对象关系

OmniM 支持跨领域边界的关系。

### 示例

| 关系 | 示例 |
|------|------|
| 建筑 ↔ 土木 | 建筑连接至场地公用设施与道路 |
| 建筑 ↔ 工厂 | 工业建筑包含工艺设备 |
| 工厂 ↔ 机械 | 泵装配体以机械产品结构表示 |
| 土木 ↔ 机械 | 桥梁支座作为机械组件建模 |
| 工厂 ↔ 建筑 MEP | HVAC 设备在建筑系统与工艺系统间共享 |

---

## 7. 跨领域共享概念

所有领域共享以下核心概念和统一的 GenRule 架构：

| 概念 | 描述 |
|------|------|
| 统一几何内核 | Box/Cylinder/Cone/Sphere/Extrusion/Sweep/Revolve/Loft + 布尔运算（所有领域共用） |
| 默认模板注册中心 (Default Template Registry) | 将每个 ObjectType 映射到目录构件，实现跨领域的统一 `create <type> <name>` 命令 |
| 对象 (Object) | 任何物理、空间、系统或逻辑工程实体 |
| 属性 (Property) | 类型化元数据、参数、计算与规格 |
| 关系 (Relation) | 包含、连接、空间、语义或系统关系 |
| 材料 (Material) | 物理与视觉材质定义 |
| 版本 (Version) | 变更历史与生命周期状态 |
| 生命周期记录 (Lifecycle Record) | 全生命周期数据 |
| AMIR | AI 中间指令表示 |

### 7.1 领域 GenRule 分类（历史遗留）

> **注**：自第六阶段起，主要几何生成改用**目录默认模板**（参见 [DATA_MODEL.zh-CN.md §6](DATA_MODEL.zh-CN.md#6-对象生成规则系统-genrule)）。下面的 GenRule 类型仅作为**概念分类**——描述各领域几何如何*结构化*，但实际生成通过 `default_catalog_component()` 映射到 ComponentDef 特征树。
>
> 四类构件分类（面/线/点/体构件，见 [数据模型 §1](DATA_MODEL.zh-CN.md#1-概述)）提供了更高层的概念分类：墙和板是**面构件**（LayerExtrusion），梁和管道是**线构件**（LinearComponent），储罐和泵是**体构件**（Primitive/Assembly），点云/地形是**点构件**（非 GenRule）。

每个工程领域的对象通过不同的 GenRule 分类描述其几何结构：

```
领域       主要 GenRule             典型对象          次要 GenRule
──────     ────────────────────    ──────────────    ───────────────────
建筑       ② LayerExtrusion        Wall, Slab        ⑤ BooleanSubtract
           (墙体)                                            (门窗开洞)
           ③ LinearComponent       Beam, Column
           (结构构件)                (固定截面)
           ③ LinearComponent       Rebar
           (钢筋)                    (圆截面 + 路径 + 阵列)
           ④ Assembly              Connection
           (钢结构节点)              (Tekla 式过程化构件)
工厂       ① Primitive             Tank, Pump        ③ LinearComponent
           (PDMS 基本体)           Valve, Pipe           (管廊/管道)
                                   Instrument, Sensor   (线缆、线槽)
Filter     ⑤ BooleanSubtract
            ④ Assembly              Transmitter,                 (SNO 剪切)
            (设备装配体)            ControlValve, JB,            (面板开孔)
                                    Panel, MCC, Support
           (CableTray 细节: 直段=LayerExtrusion, 弯头/三通=LinearComponent/Assembly)
市政       ③ LinearComponent       Road, Bridge      ① Primitive
           (数维线构件/走廊)        Tunnel, Culvert, Dam  (独立构件)
                                   Pier: 盖梁=LinearComponent, 柱=LinearComponent/Primitive,
                                   承台=Primitive, 桩=Primitive
                                   Abutment: 台身=LinearComponent, 翼墙=LinearComponent
                                   (斜向路径), 搭板=LayerExtrusion
                                   Deck: 多层 LinearComponent (曲线路径)
                                   Tunnel Lining: 多线 LinearComponent
机械       ④ Assembly              设备装配体         ① Primitive
           (零件组合)              Gear, Shaft           (基础零件)
```

### 7.2 线构件：跨领域统一的路径式建模

线构件（LinearComponent）是 OmniM 的核心创新之一。它将 Civil3D 的道路（线位+横断面+变宽规则）、PDMS 的管道（管径+路径）和建筑结构构件（梁/柱的固定截面+路径）统一为同一抽象：**路径 + 变化横断面 + 外部约束**。

建筑结构梁/柱 = 不变截面的线构件（路径 + 固定截面轮廓）。
道路 = 变截面的线构件（平面线形 + 竖曲线 + 分段横断面 + 地形目标约束）。

> **墙不是线构件**：墙是**面构件**（见 [数据模型 §1](DATA_MODEL.zh-CN.md#1-概述)），其几何由面边界（矩形或草图轮廓）定义，层结构作为厚度沿法向拉伸生成。墙使用 LayerExtrusion GenRule，而非 LinearComponent。

---

### 7.3 构件创建流程

本节描述每种构件类型的**正确用户侧创建流程**，源自跨平台调研（Revit、Civil3D、Bentley、E3D、Tekla、SolidWorks）。四类分类（面/线/点/体构件，见 §1）决定了点选模式和参数需求。

#### 面构件

| 类型 | 点选方式 | 点击次数 | 创建流程 | GenRule |
|------|---------|---------|---------|---------|
| **Wall（墙）** | 折线路径 | 2+ | 在平面图中绘制墙中心线（2+点）→ 指定 WallType（层叠结构）→ 设置高度 → 完成 | LayerExtrusion |
| **Slab（楼板）** | 闭合边界多边形 | N | 在平面图中绘制闭合边界（N点，自动闭合末点→首点）→ 指定 FloorType（层叠结构）→ 完成 | LayerExtrusion |
| **Roof（屋顶）** | 拉伸轮廓+范围（或足迹） | 2（拉伸）或 N（足迹） | **拉伸模式**：绘制轮廓草图 + 拾取起点/终点范围。**足迹模式**：绘制闭合边界多边形 → 指定 RoofType → 完成 | LayerExtrusion |
| **Floor（地板）** | 闭合边界多边形 | N | 同楼板：绘制闭合边界 → 指定 FloorType → 完成 | LayerExtrusion |

> **代码偏差**：所有面构件当前使用 2 点点选（同墙路径），但 slab/roof/floor 需要 N 点闭合多边形。参见 [CODE_DEVIATIONS.md](../status/CODE_DEVIATIONS.md) §3.1。

#### 线构件

| 类型 | 点选方式 | 点击次数 | 创建流程 | GenRule |
|------|---------|---------|---------|---------|
| **Beam（梁）** | 起点/终点 | 2 | 拾取起点 → 拾取终点 → 指定截面轮廓 → 完成 | LinearComponent |
| **Column（柱）** | 定位点（或底/顶） | 1–2 | **Revit 式**：拾取定位点 → 指定截面 → 设置高度。**Tekla 式**：拾取底点 → 拾取顶点 → 完成 | LinearComponent |
| **Pipe（管道）** | 起点/终点（连接器到连接器） | 2 | 拾取起点连接器或点 → 拾取终点连接器或点 → 指定管道规格（管径、壁厚等级）→ 完成 | LinearComponent |
| **Duct（风管）** | 起点/终点（连接器到连接器） | 2 | 同管道：起点 → 终点 → 指定风管规格 → 完成 | LinearComponent |
| **Road（道路）** | 线位（多点走廊） | N（复杂） | **完整流程**：绘制平面线形 → 指定竖曲线 → 指定装配（横断面）→ 设置目标（地形、宽度）→ 完成。**简化版**：2点路径 + 默认横断面 | LinearComponent |
| **Bridge（桥梁）** | 线位+下部结构 | N（复杂） | 绘制线位 → 指定桥墩/桥台位置 → 指定桥面横断面 → 指定支座类型 → 完成 | LinearComponent + Assembly |
| **Tunnel（隧道）** | 线位+模板 | N（复杂） | 绘制线位 → 指定隧道模板（衬砌、管片配置）→ 完成 | LinearComponent |

> **代码偏差**：Road/bridge/tunnel 使用简化 2 点创建，完整走廊工作流未实现。参见 [CODE_DEVIATIONS.md](../status/CODE_DEVIATIONS.md) §3.2。

#### 点构件

| 类型 | 点选方式 | 点击次数 | 创建流程 | GenRule |
|------|---------|---------|---------|---------|
| **PointCloud（点云）** | 放置点 | 1 | 拾取放置点 → 引用点云文件 → 完成 | PointCloudGeometry |
| **TerrainModel（地形）** | 放置点 | 1 | 拾取放置点 → 提供源数据（点文件、等高线、TIN）→ 完成 | TerrainModel |
| **FeatureLine（特征线）** | 折线 | N | 绘制 3D 折线（N点，定义断裂线）→ 指定高程/样式 → 完成 | CurveGeometry |
| **ContourLine（等高线）** | 折线 | N | 绘制 2D 折线（N点，等高）→ 指定高程 → 完成 | CurveGeometry |

> **代码偏差**：FeatureLine/ContourLine 使用 1 点放置，但需要 N 点折线点选。参见 [CODE_DEVIATIONS.md](../status/CODE_DEVIATIONS.md) §3.3。

#### 体构件

| 类型 | 点选方式 | 点击次数 | 创建流程 | GenRule |
|------|---------|---------|---------|---------|
| **Tank（储罐）** | 放置点 | 1 | 拾取放置点 → 选择目录构件（储罐类型、尺寸）→ 完成 | Primitive / Assembly |
| **Pump（泵）** | 放置点 | 1 | 拾取放置点 → 选择目录构件 → 完成 | Primitive / Assembly |
| **Valve（阀门）** | 放置点（在线） | 1 | 在管道上拾取放置点 → 选择阀门类型 → 完成 | Primitive / Assembly |
| **Gear（齿轮）** | 放置点 | 1 | 拾取放置点 → 选择目录构件 → 完成 | Primitive / Assembly |
| **Equipment（设备）** | 放置点 | 1 | 拾取放置点 → 选择目录构件 → 完成 | Assembly |
| **Part（零件）** | 放置点 | 1 | 拾取放置点 → 选择目录构件 → 完成 | Primitive |
| **Assembly（装配体）** | 放置点 | 1 | 拾取放置点 → 选择目录构件 → 完成 | Assembly |

> 所有体构件已正确实现 1 点点选。参见 [CODE_DEVIATIONS.md](../status/CODE_DEVIATIONS.md) §3.4。

---

## 8. 领域扩展规则

新增的领域特定对象类型必须定义：

1. 对象类名与命名空间
2. 在 OmniM 层级中的父类
3. 在适用时定义 IFC / STEP / GIS 或其他标准映射
4. 必需的属性集
5. 几何约束
6. 关系规则
7. 生命周期行为
8. 导入/导出映射规则

---

## 9. 待解决问题

1. 领域特定的校验规则应放在插件中还是核心中？
2. 对于工厂设计，应包含多少 ISO 15926 语义映射？
3. 机械约束是否应表示为一等关系？
4. 如何协调 BIM LOD 与机械 CAD 精度要求？
5. 领域分类代码是否应按国家或行业实现可插拔？
