# OmniM 统一数据模型

> **最后更新：2026-10-27，已对照代码审计**（`proto/omnim.proto`、`proto/omcom.proto`、`crates/model/**`、`crates/sketch/**`、`crates/ifc/**`）。标注 *设计规范* 的内容为目标模型，尚未落入 proto/代码；标注 🗓 计划中 的能力是尚未实现的设计目标。

## 1. 概述

> **关联文档**
> - 元件库与系列库：[CATALOG_SERIES.zh-CN.md](CATALOG_SERIES.zh-CN.md)
> - 工程生命周期与变更流程：[LIFECYCLE.zh-CN.md](LIFECYCLE.zh-CN.md)

OmniM 的数据模型被设计为 **IFC 超集**，并对土木工程、工厂设计和机械设计领域进行了原生扩展。它为工程对象、项目生命周期记录、文档、进度计划、成本、资产、工单、运营数据以及数字孪生数据流提供了单一、统一的表示，贯穿整个工程项目生命周期。

### 设计原则

- **IFC 兼容**：核心类在适用的地方与 IFC 4x3 对齐
- **领域可扩展**：为非建筑领域提供清晰的扩展机制
- **几何优先**：将丰富的几何表示作为一等公民
- **关系丰富**：原生支持包含、连接、空间、生命周期、合同和语义关系
- **生命周期原生**：规划、设计、采购、施工、调试验收、资产、运维、生产和退役数据均是一等模型概念
- **版本化**：每个对象、生命周期记录和变更均进行版本化以供审计
- **统一几何，分层语义**：建筑、工厂、市政共用同一套参数化几何图元，领域语义通过 GenRule（生成规则）映射到几何，而非写死在不同代码路径中
- **高性能**：针对存储和查询性能进行优化

### 三层对象模型

OmniM 采用 **统一几何内核 + 分层语义** 的架构，在同一个几何底层之上支撑 Revit 式（语义驱动）、PDMS 式（基本体拼装 + 目录规范）、Civil3D 式（线构件）和 AVEVA Instrumentation 式（逻辑对象）四种设计范式：

> **E3D 数据库类型**（`docs/research/aveva_e3d_primitives.md §1.3`）：AVEVA E3D 使用 8 种数据库类型，映射到 OmniM 的存储层：
> | E3D 数据库 | OmniM 对应 | 用途 |
> |-----------|-----------|------|
> | MODEL | Project Object Store | 3D 模型几何 |
> | PADD/DRAW | — | 2D 图纸生成 |
> | ISOD | — | 轴测图生成 |
> | SCHEMATIC | LogicalObject Store | P&ID / 原理图数据 |
> | ENGINEERING | Property System | 工程属性 |
> | CATALOGUE | Library CatalogStore | 构件目录 |
> | LEXICON | Property System | UDA/UDET（用户自定义属性/类型） |
> | PROPERTIES | Property System | 元素属性 |
>
> OmniM 将独立的 MODEL、SCHEMATIC 和 ENGINEERING 数据库统一为单一对象图，通过 LogicalObject ↔ Object 交叉视图实现；将 CATALOGUE/LEXICON/PROPERTIES 统一为 Library + Property System。

```
┌──────────────────────────────────────────────────────┐
│          第四层：领域语义层 (Domain Semantics)         │
│  WallType / PipeSpec / Alignment / AssetRecord       │
│  - 类型定义、层结构、规格参数、约束条件                 │
│  - 数字孪生数据绑定（IoT 流、工单、告警）              │
└──────────────────────────┬───────────────────────────┘
                           │ 通过系列库实例化
┌──────────────────────────▼───────────────────────────┐
│   ╔══════════════════════════════════════════════╗    │
│   ║    Library (共享库 — session 级全局静态)      ║    │
│   ║  • CatalogStore: ComponentDef 定义           ║    │
│   ║  • ComponentStore: Component 模板            ║    │
│   ║  • FeatureTreeRegistry: 特征树模板           ║    │
│   ║  • 默认模板注册中心                           ║    │
│   ║  一次加载，跨项目共享                          ║    │
│   ╚══════════════════════════════════════════════╝    │
│                           │ 被 Series 引用           │
│          第三层：构件目录 & 系列库                      │
│  ComponentDef: 参数化三维模型定义                      │
│  SeriesStore (独立模块)  ◄── Project 引用              │
│  Connectors: 带类型码的连接点                          │
│  - PARAGON 式元件库                                   │
│  - SPECON 式系列库/等级库                              │
└──────────────────────────┬───────────────────────────┘
                           │ 关联 GenRule
┌──────────────────────────▼───────────────────────────┐
│          第二层：对象生成层 (Object Generation)        │
│  ObjectTypeDef → GenRule → [图元列表 + 布尔操作]      │
│  - 5 种 GenRule 覆盖全部领域                          │
│  - 变化规则 (VariationRule) 驱动变截面线性构件         │
└──────────────────────────┬───────────────────────────┘
                           │ 输出
┌──────────────────────────▼───────────────────────────┐
│          第一层：统一几何内核 (Unified Geometry)       │
│  Box / Cylinder / Cone / Sphere / Extrusion          │
│  Sweep / Revolve / Loft                              │
│  + Union / Subtract / Intersect (布尔运算)            │
│  + Transform (Origin + Orientation)                   │
│       ↓                                              │
│  BRep 实体 → 三角网格 → 渲染输出                      │
└──────────────────────────────────────────────────────┘
```

**核心思想**：建筑墙和工厂管道在几何层都是 Extrusion/Cylinder，差异仅在 GenRule 的复杂度——建筑墙的 GenRule 把"层结构"展开为多条 Extrusion，工厂管道的 GenRule 几乎直接透传参数。

### 构件分类：面 / 线 / 点 / 体构件

在全部四个工程领域中，构件几何按维度概念上分为四类。这是**主概念分类**——描述每个构件*是什么类型*的几何，而 GenRule 层（见 §6）描述*如何*生成该几何：

| 类别 | 英文 | 典型对象 | GenRule 映射 | 说明 |
|------|------|---------|--------------|------|
| **面构件** | Surface | Wall, Slab, Roof, Floor, GradingSurface | LayerExtrusion（或 TIN） | 2D 面域元素，含厚度/材质层；由边界（矩形或草图）定义，层材质叠层沿法向作为厚度。主几何是**面**本身，而非中心线路径。 |
| **线构件** | Linear | Beam, Column, Pipe, Road, Duct, CableTray | LinearComponent | 1D 路径元素，带截面轮廓；由路线（路径）+ 沿路径变化的截面定义。主几何是**路径**，而非面。 |
| **点构件** | Point | PointCloud, TerrainModel（顶点/特征线）, FeatureLine, ContourLine | PointCloudGeometry / TerrainModel（非 GenRule） | 0D 点定义的构件；几何即点集本身——无扫掠、无放样、无布尔。地形 TIN 曲面由点（顶点+特征线）构造。 |
| **体构件** | Body | Tank, Pump, Valve, Gear, Equipment, StructuralSteel | Primitive / Assembly | 3D 实体积体；由直接基本体操作（box、cylinder、cone、sphere）或多个基本体的装配定义。 |

**墙是面构件**：与梁（线构件）或储罐（体构件）不同，墙定义一个 2D 面域——其长和高构成矩形（或用草图线绘制的任意形状），层材质叠层作为厚度。这通过 LayerExtrusion 生成（§6.2）：面边界提供拉伸轮廓，层叠层提供厚度。墙**不是** LinearComponent 路径扫掠——其主几何是**面**，而非中心线路径。参见 §6.4 的区别。

> **跨平台参考**：四类构件分类源自跨平台分析（`docs/research/comparative_analysis.md §2-4`）：Revit Wall/Floor/Roof（面构件）、Revit Beam/Column + Civil 3D Corridor（线构件）、点云 + TIN 曲面（点构件）、E3D 基本体 + SolidWorks 零件（体构件）。

**Library vs Project**：**Library** 是 **session 级全局单例**，持有主元件目录（`CatalogStore`）、构件模板库（`ComponentStore`）和特征树模板（`FeatureTreeRegistry`）。一次加载，跨项目共享。**SeriesStore** 是 **独立模块**——它定义选择器决策树，通过 ComponentDef ID 引用 Library 中的内容。**Project** 持有私有的 `SeriesStore`（项目内建的系列）和 `external_series_refs`（外部系列引用）。Project **不持有** Library 引用。参见 [§13](#13-构件目录元件库--paragon) 和 [§14](#14-系列库等级库--specon)。

---

## 2. 核心对象层次结构

```
OmniObject (base for all objects)
├── SpatialElement
│   ├── Site
│   ├── Building
│   ├── BuildingStorey
│   └── Space
├── Element (IFC-aligned base for all physical elements)
│   ├── BuildingElement (IFC compatible building domain)
│   │   ├── Wall, Beam, Column, Slab
│   │   ├── Door, Window, Stair, Roof
│   │   ├── MEP elements (Duct, Pipe, etc.)
│   │   └── Furniture, Fixtures, Equipment
│   ├── CivilElement (Civil Engineering extension)
│   │   ├── Road, Bridge, Tunnel, Dam
│   │   ├── Culvert, RetainingWall
│   │   ├── Alignment, Profile, CrossSection
│   │   ├── TerrainModel (TIN 曲面，独立类型)
│   │   ├── CutVolume, FillVolume (BooleanSubtract 曲面-曲面)
│   │   ├── GradingSurface (设计地形)
│   │   ├── Slope (LinearComponent + ExternalTarget)
│   │   ├── FeatureLine, ContourLine
│   │   ├── SiteComponent (附着于地形的固定设施)
│   │   └── EarthworkBalance (分析结果)
│   ├── PlantElement (3D Plant Design extension)
│   │   ├── Tank, Pump, Valve
│   │   ├── HeatExchanger, Reactor, Vessel
│   │   ├── Pipe, Nozzle, Flange
│   │   └── StructuralSteel, Platform, Ladder
│   └── MechanicalElement (Mechanical Design extension)
│       ├── Part, Assembly, Fastener
│       ├── Gear, Shaft, Bearing
│       ├── Motor, PumpBody, Housing
│       └── Spring, Seal, Bushing
├── System
│   ├── BuildingSystem
│   ├── CivilSystem
│   ├── PlantSystem
│   └── MechanicalSystem
└── ProjectLifecycleRecord
    （通过自由形式的 `record_type` 和 `phase` 字段区分类型，
     无固定子类——详见第 4 节）
```
> **场地地理属性**（源自 Revit IfcSite + Civil 3D 测量，`docs/research/site_terrain_components.md §1.1`）：Site 是空间容器，本身没有几何。其物理范围由以下属性定义：
> - `name` — 场地名称
> - `address` — 地理地址
> - `latitude / longitude` — 地理坐标（WGS84）
> - `site_rotation` — 相对于真北的朝向
> - `project_base_point` — 测量坐标原点（Easting、Northing、Elevation）
> - `survey_point` — 真实世界坐标参考
> - `boundary` — 定义场地范围的可选边界多边形
> - `geo_ref` — `GeographicReference`（坐标系 / 投影 / EPSG 代码）
>
> 空间层级遵循 IFC 4x3：`Site → Building → BuildingStorey → Space`。土木和工厂元素同样归属于 Site 之下，按系统类型而非空间楼层组织。
>
> **多实体零件的支持**（源自 SOLIDWORKS，`docs/research/solidworks_components.md §10`）：一个 `Part` 可以包含多个不相连的实体（Solid Body）。操作包括：实体间的合并（Combine：Add/Subtract/Intersect）、特征范围（Feature Scope，选择特征影响哪些实体）、移动/复制实体（Move/Copy Body）、逐实体材质以及爆炸视图（Exploded Views）。OmniM 按如下设计**支持该能力（🗓 计划中）**：一个 Part 的特征树将生成多个独立实体，由 `Boolean` 特征操作控制实体间的组合。（当前特征引擎对每个 Component 只产出单一合并网格。）
>
> **本层次的实现状态**：`proto/omnim.proto` 中的 `ObjectType` 枚举目前实现——空间：Site、Building、BuildingStorey、Space；建筑：Wall、Beam、Column、Slab、Door、Window、Stair、Roof、Floor，外加 MEP（Duct、CableTray、AirTerminal、Ahu、Chiller、Transformer、Luminaire）；市政：Road、Bridge、Tunnel、Dam、Culvert、RetainingWall、PipeCivil、Pier、Manhole、Abutment；工厂：Tank、Pump、Valve、HeatExchanger、Reactor、PipePlant、StructuralSteel、Fitting、Compressor、Instrument；机械：Part、Assembly、Fastener、Gear、Shaft、Bearing、Motor、Spring、Seal；另有 Component、FeatureLine、ContourLine、Generic。上文其余分支（Nozzle、Flange、Platform、Ladder、Vessel、PumpBody、Housing、Bushing；Alignment、Profile、CrossSection、TerrainModel、CutVolume、FillVolume、GradingSurface、Slope、SiteComponent、EarthworkBalance；各 System 类）为概念设计目标——枚举扩展 🗓 计划中。

---

## 3. 基类：OmniObject

OmniM 中的所有对象都构建于 `OmniObject` 基类之上，它提供通用的标识和生命周期字段。以下消息为**设计规范**——proto 中尚无 `OmniObject` 消息：已实现类型是 `crates/model/src/object.rs` 中的 serde JSON `OmniObject` 结构体。

```protobuf
message OmniObject {
  // Identity
  string id = 1;                    // UUID v4
  string name = 2;                  // Human readable name
  string description = 3;           // Optional description
  ObjectType type = 4;              // Object classification
  
  // Geometry
  Geometry geometry = 5;            // Primary geometry representation
  Transform transform = 6;          // World space transform
  
  // Properties
  repeated Property properties = 7; // Dynamic property bag
  repeated PropertySet property_sets = 8; // Grouped IFC-style property sets
  
  // Classification
  string ifc_class = 10;            // Corresponding IFC class if applicable
  repeated string tags = 11;        // Flexible tagging
  string material = 12;             // Material reference
  
  // Lifecycle
  ObjectStatus status = 13;
  google.protobuf.Timestamp created_at = 14;
  string created_by = 15;
  google.protobuf.Timestamp modified_at = 16;
  string modified_by = 17;
  
  // Versioning
  string version_id = 18;
  string previous_version_id = 19;
  
  // Relationships are stored separately in the relation index

  // 目录引用
  CatalogRef catalog_ref = 20;  // 关联到目录构件定义

  // 行政分区 & 项目分解
  optional string ad_id = 21;            // 所属行政分区（地块或更高级别）
  repeated string pbs_ids = 22;          // 所属 PBS 节点（通常为 1 个）

  // 坐标系
  string crs_id = 23;                    // 该对象 Transform 所用的坐标系（通常为 project_crs）
}

enum ObjectStatus {
  OBJECT_STATUS_UNSPECIFIED = 0;
  DRAFT = 1;
  PROPOSED = 2;
  APPROVED = 3;
  CONSTRUCTION = 4;
  AS_BUILT = 5;
  OPERATIONAL = 6;
  DECOMMISSIONED = 7;
}
```

> **当前已实现**（Rust 结构体，`crates/model/src/object.rs`）：`id`、`name`、`description`、`object_type`、`status`、`properties`、`property_sets`、`geometry`、`material_id`（可选；上文以 `material` 表示）、`parent_id`（结构树父节点）、`tags`、`ifc_class`，以及建模字段 `features`、`sketch_ids`、`dirty`、`catalog_ref`。上方 `ObjectStatus` 取值与 `proto/omnim.proto` 完全一致。
>
> **🗓 计划中的字段**（结构体与 proto 均未有）：`transform`（每对象世界变换；当前通过 API `PUT /api/objects/{id}/transform` 在接口层应用）、`created_at`/`created_by`、`modified_at`/`modified_by`、`version_id`、`previous_version_id`（版本/审计记录见 §10，本身也是设计规范）、`ad_id`（§16）、`pbs_ids`（§17）、`crs_id`（§19）。

---

## 4. 项目生命周期模型

OmniM 将项目生命周期数据视为一等模型实体，而非外部的项目管理元数据。工程项目生命周期划分为九个阶段，生命周期记录按阶段组织。每条记录都可以引用工程对象、几何、文档、用户、组织、进度计划、成本和外部系统。

```
Planning → Design → Procurement → Construction → Commissioning & Acceptance → Asset → Operation & Maintenance → Production → Decommissioning
```

```protobuf
message ProjectLifecycleRecord {
  string id = 1;
  LifecyclePhase phase = 2;          // Which lifecycle phase this record belongs to
  string record_type = 3;            // Free-form type tag (e.g. "schedule", "cost", "delivery", "inspection", "work_order")
  string name = 4;
  string description = 5;
  repeated string related_object_ids = 6;
  repeated string related_document_ids = 7;
  repeated Property properties = 8;
  RecordStatus status = 9;
  google.protobuf.Timestamp planned_start = 10;
  google.protobuf.Timestamp planned_finish = 11;
  google.protobuf.Timestamp actual_start = 12;
  google.protobuf.Timestamp actual_finish = 13;
  double planned_cost = 14;
  double actual_cost = 15;
  string responsible_party = 16;
  string version_id = 17;
}

// 工程项目生命周期阶段 (9 phases, lifecycle closed loop)
// 规划 → 设计 → 采购 → 施工 → 调试验收 → 资产 → 运维 → 生产 → 退役拆除
enum LifecyclePhase {
  LIFECYCLE_PHASE_UNSPECIFIED = 0;
  PLANNING = 1;                  // 规划: site selection, feasibility, zoning, project requirements
  DESIGN = 2;                    // 设计: BIM/CAD models, drawings, specifications, calculations
  PROCUREMENT = 3;               // 采购: bill of materials, suppliers, contracts, delivery tracking
  CONSTRUCTION = 4;              // 施工: schedule, progress, quality, safety, as-built
  COMMISSIONING_ACCEPTANCE = 5;  // 调试验收: testing, inspection, acceptance, handover
  ASSET = 6;                     // 资产: asset registry, warranty, handover to owner
  OPERATION_MAINTENANCE = 7;     // 运维: operation, maintenance, work orders, energy management
  PRODUCTION = 8;                // 生产: actual production operation (plant/manufacturing)
  DECOMMISSIONING = 9;           // 退役拆除: decommissioning, demolition, disposal, end of life
}

enum RecordStatus {
  RECORD_STATUS_UNSPECIFIED = 0;
  DRAFT = 1;
  OPEN = 2;
  IN_PROGRESS = 3;
  BLOCKED = 4;
  COMPLETED = 5;
  APPROVED = 6;
  REJECTED = 7;
  CLOSED = 8;
}
```

### 生命周期关系

生命周期记录通过阶段作用域的关系与工程对象连接。一条记录携带一个 `phase` 和一个自由格式的 `record_type`，并可以通过 `link_role` 与任何工程对象关联，`link_role` 描述对象与记录的关系方式（例如 "scope"、"target"、"reference"）。

| 阶段 | 记录示例 | 关联对象示例 |
|-------|-----------------|------------------------|
| Planning | 可行性研究、场地约束、分区要求 | Site、地形、公用设施线形 |
| Design | 设计包、图纸、计算书、设计变更 | 墙、梁、柱、管道、设备 |
| Procurement | 物料清单、合同、交付、供应商数据 | 阀门、泵、钢构件、材料 |
| Construction | 施工活动、进度任务、检查、问题 | 安装中的物理构件 |
| Commissioning & Acceptance | 测试记录、验收报告、移交包 | 设备、系统、竣工构件 |
| Asset | 资产登记条目、质保记录 | 移交资产、设备、组件 |
| Operation & Maintenance | 工单、维护日志、能耗记录 | 运营设备、系统、空间 |
| Production | 生产批次、产量记录、停机日志 | 生产线、反应器、容器 |
| Decommissioning | 退役计划、拆除记录、处置日志 | 退役资产、待拆除结构 |

---

## 5. 几何表示

OmniM 针对不同的使用场景支持多种几何表示。几何管线如下：

```
ComponentDef(目录) → 实例化 → 草图(2D约束) → 特征(拉伸/旋转) → BRep(精确) → Mesh(可视化)
```

### 5.1 几何类型

```protobuf
message Geometry {
  string id = 1;
  GeometryType type = 2;
  
  oneof representation {
    ParametricGeometry parametric = 3;
    BRepGeometry brep = 4;
    MeshGeometry mesh = 5;
    PointCloudGeometry point_cloud = 6;
    CurveGeometry curve = 7;
    ExternalGeometryReference external = 8;
  }
  
  BoundingBox bbox = 9;
  uint32 level_of_detail = 10;  // LOD 100-500 scale
}

enum GeometryType {
  GEOMETRY_TYPE_UNSPECIFIED = 0;
  PARAMETRIC = 1;    // Parametric primitives and operations
  BREP = 2;          // Boundary representation (precise)
  MESH = 3;          // Polygon mesh (visualization)
  POINT_CLOUD = 4;   // Scan data
  CURVE = 5;         // 2D/3D curves and lines
  EXTERNAL = 6;      // Reference to external file
}
```

> **实现状态**：上方 `Geometry` 消息与 6 值 `GeometryType` 枚举在 `proto/omnim.proto` 中，但表示 `oneof` 目前仅接通 `parametric`、`mesh`、`external`。`brep`、`point_cloud`、`curve` 表示臂为设计规范——🗓 计划中（`crates/model/src/brep.rs` 已有基于 truck-modeling 的 BRep 引擎，但 BRep 结果被细分为 mesh，而非以 `brep` 表示存储）。`PointCloudGeometry` 与 `CurveGeometry` 消息尚未定义。

### 5.2 参数化几何

```protobuf
message ParametricGeometry {
  repeated ParametricOperation operations = 1;
}

message ParametricOperation {
  oneof operation {
    Box box = 1;
    Cylinder cylinder = 2;
    Sphere sphere = 3;
    Cone cone = 4;
    Extrusion extrusion = 5;
    Revolve revolve = 6;
    Sweep sweep = 7;
    Loft loft = 8;
    Torus torus = 9;      // 弯头段（罐体 / 管道）
    Dish dish = 10;       // 容器 / 储罐封头
  }
}
```

> **基本体已实现**（`proto/omnim.proto`）：上述 10 种操作均已存在，包括 `Torus`（弯头段）与 `Dish`（容器/储罐封头）——映射详见 `docs/research/aveva_e3d_primitives.md`。`BooleanOperation` 与 `Transform` 特意**不**作为 `ParametricOperation` 成员：布尔组合位于特征层（`BooleanFeature` / `BooleanOp`），将其加入 `ParametricOperation` 仍是 §6.7 的未来增强。

### 5.3 边界表示（BRep）

> **设计规范（🗓 计划中）**：以下消息是 BRep 序列化的目标 schema，尚未加入 `proto/omnim.proto`。已交付的 BRep 引擎（`crates/model/src/brep.rs`，基于 truck-modeling）在内存中构建 `Solid`/`Shell`/`Wire` 并细分为 `MeshGeometry`，但尚未存储或序列化这种消息形式。

```protobuf
message BRepGeometry {
  repeated Vertex vertices = 1;
  repeated Edge edges = 2;
  repeated Face faces = 3;
  repeated Shell shells = 4;
  repeated Solid solids = 5;
}

message Vertex {
  Point3D point = 1;
}

message Edge {
  int32 start_vertex = 1;
  int32 end_vertex = 2;
  Curve curve = 3;  // Optional curve geometry for edge
}

message Face {
  repeated int32 edge_indices = 1;
  Surface surface = 2;  // Optional surface geometry
}
```

### 5.5 草图几何（2D 约束草图）

草图是参数化建模的基础。草图位于一个平面上，包含由几何约束约束的 2D 图元。

> **实现状态**：已交付的草图系统为 `omnim_sketch`（`crates/sketch/`，持久化于 `Project.sketches`，经 `proto/omcom.proto` 的 `SketchData` 对外暴露）。已实现图元类型：`Point, Line, Circle, Arc, Rectangle, Polygon, Ellipse, Spline, Polyline`（每个图元存控制点，无类型化子消息）。已实现约束类型（19 种）：`Horizontal, Vertical, Coincident, Parallel, Perpendicular, Length, Distance, Angle, Radius, Diameter, Tangent, Symmetric, Fixed, AtMidpoint, EqualLength, EqualRadius, PtOnLine, PtOnCircle, SameOrientation`。工作平面由 `origin` + `normal` 字段承载，而非独立 `SketchPlane` 消息。下文超出该集合的部分（类型化子消息、Slot/Conic/Parabola/SketchText/BeltChain/EquationCurve、3D 草图约束）是源自 SOLIDWORKS 的设计规范——🗓 计划中。

```protobuf
message Sketch {
  string id = 1;
  string name = 2;
  SketchPlane plane = 3;
  repeated SketchEntity entities = 4;
  repeated Constraint constraints = 5;
  bool solved = 6;
}

message SketchPlane {
  Point3D origin = 1;
  Vector3D normal = 2;
  Vector3D ref_direction = 3;  // X 轴参考方向
}

message SketchEntity {
  string id = 1;
  oneof entity {
    SketchLine line = 2;
    SketchArc arc = 3;
    SketchCircle circle = 4;
    SketchPoint point = 5;
    SketchSpline spline = 6;
    SketchSlot slot = 7;
    SketchConic conic = 8;
    SketchParabola parabola = 9;
    SketchText text = 10;
    BeltChain belt_chain = 11;
    EquationCurve equation_curve = 12;
  }
}

// Slot 3 种变体（SOLIDWORKS §4.0）：Straight / Arc / Conical
message SketchSlot {
  SlotType type = 1;          // STRAIGHT / ARC / CONICAL
  double center_x = 2; double center_y = 3;
  double width = 4;           // 槽宽度
  double length = 5;          // 槽长度（直槽）
  optional double radius = 6; // 弧/锥槽半径
  optional double start_angle = 7;
  optional double end_angle = 8;
  bool is_ccw = 9;
}

enum SlotType {
  SLOT_STRAIGHT = 0;
  SLOT_ARC = 1;
  SLOT_CONICAL = 2;
}

// Conic：端点 + Rho 值驱动（SOLIDWORKS §4.0）
message SketchConic {
  double start_x = 1; double start_y = 2;
  double end_x = 3;   double end_y = 4;
  double rho = 5;          // Rho 值：<0.5 椭圆，=0.5 抛物线，>0.5 双曲线
}

message SketchParabola {
  double vertex_x = 1; double vertex_y = 2;
  double focus_offset_x = 3; double focus_offset_y = 4;
  double start_x = 5; double start_y = 6;
  double end_x = 7;   double end_y = 8;
}

// SketchText：沿曲线/边的 2D 文本，可拉伸或切除为 3D（SOLIDWORKS §4.0）
message SketchText {
  string text = 1;
  double pos_x = 2; double pos_y = 3;
  double height = 4;
  optional string font = 5;
  optional string curve_entity_id = 6;  // 跟随的曲线/边
  bool reverse_direction = 7;
  bool extrude = 8;   // true = 可拉伸/切除的 3D 文本
}

// BeltChain：带轮/链轮机构布局草图（SOLIDWORKS §4.0）
message BeltChain {
  repeated string pulley_entity_ids = 1;  // 圆/弧（带轮或链轮）
  double belt_offset = 2;                 // 皮带内侧/外侧偏移
  bool tangent_lines = 3;
}

// EquationCurve：参数方程驱动的曲线（SOLIDWORKS §4.0）
message EquationCurve {
  string x_formula = 1;   // 参数式 x(t)
  string y_formula = 2;   // 参数式 y(t)
  double t_start = 3; double t_end = 4;
  int32 samples = 5;
}

message SketchLine {
  double start_x = 1; double start_y = 2;
  double end_x = 3;   double end_y = 4;
}

message SketchArc {
  double center_x = 1; double center_y = 2;
  double radius = 3;
  double start_angle = 4; double end_angle = 5;
  bool is_ccw = 6;
}

message SketchCircle {
  double center_x = 1; double center_y = 2;
  double radius = 3;
}

> **计划中的草图图元**（源自 SOLIDWORKS，`docs/research/solidworks_components.md §4.0`）：`Slot`（3 种变体：Straight/Arc/Conical）、`Conic`（端点 + Rho 值——基于 Rho 值的椭圆/抛物线/双曲线）、`Parabola`、`SketchText`（可拉伸/切除的 3D 文本）、`BeltChain`（带轮/链轮布局草图）、`EquationCurve`（参数方程驱动曲线）。设计为 `SketchEntity` oneof 的扩展——🗓 计划中，尚未进入 proto/omcom.proto 或 `omnim_sketch`。

message Constraint {
  string id = 1;
  ConstraintType type = 2;
  repeated string entity_ids = 3;    // 被约束的图元
  optional double value = 4;         // 参数值（距离、角度、半径）
  optional string reference_entity_id = 5;  // 参考图元
  optional int32 point_index_a = 6;  // 图元 a 上的点索引
  optional int32 point_index_b = 7;  // 图元 b 上的点索引
}

enum ConstraintType {
  HORIZONTAL = 0;        // 水平
  VERTICAL = 1;          // 垂直
  FIXED = 2;             // 固定
  COINCIDENT = 3;        // 重合
  PARALLEL = 4;          // 平行
  PERPENDICULAR = 5;     // 垂直
  TANGENT = 6;           // 相切
  EQUAL_LENGTH = 7;      // 等长
  EQUAL_RADIUS = 8;      // 等半径
  COLLINEAR = 9;         // 共线
  MIDPOINT = 10;         // 中点
  SYMMETRIC = 11;        // 对称
  DISTANCE = 12;         // 距离
  ANGLE = 13;            // 角度
  RADIUS = 14;           // 半径
  LENGTH = 15;           // 长度
  DIAMETER = 16;         // 直径
  OFFSET = 17;           // 偏移

  // 3D 草图约束（SOLIDWORKS §4.1）
  CORADIAL = 18;           // 弧/圆共享相同圆心和半径
  EQUAL_CURVATURE = 19;    // 曲线连接处曲率连续
  EQUAL_CURVE_LENGTH = 20; // 曲线长度相等
  MERGE_POINTS = 21;       // 合并两个端点
  PIERCE = 22;             // 曲线穿过平面/曲面
  FIX_SLOT = 23;           // 固定槽口几何
  ON_EDGE = 24;            // 位于模型边上
  ON_PLANE = 25;           // 位于参考平面上
  ON_SURFACE = 26;         // 位于模型曲面上
  TANGENT_TO_FACE = 27;    // 与面相切
  TRACTION = 28;           // 牵引约束（接触）
  TORSION_CONTINUITY = 29; // G3 扭转连续性
}

> **计划中的 3D 草图约束**（源自 SOLIDWORKS，`docs/research/solidworks_components.md §4.1`）：`Collinear`（已在 =9 处）、`Coradial`、`EqualCurvature`、`EqualCurveLength`、`MergePoints`、`Pierce`、`FixSlot`、`OnEdge`、`OnPlane`、`OnSurface`、`TangentToFace`、`Traction`、`TorsionContinuity`——对 3D 草图机械设计至关重要。设计为 `ConstraintType` 枚举的扩展——🗓 计划中，尚未进入 proto/omcom.proto 或 `omnim_sketch`。
```

### 5.6 特征建模（基于草图的 3D 操作）

特征是**一等 3D 实体**，位于 Component 内部。每个特征：
1. 引用一个或多个**草图**作为其 2D 轮廓
2. 拥有由重建管线计算得到的自身**缓存 Geometry**（MeshGeometry）
3. 输入变化时被标记为 **dirty**，触发自动重建
4. 与同一 Component 内的其他特征构成依赖链

**草图是 Component 内的基础实体。**草图是 2D 约束轮廓（线、圆、弧），作为特征的输入。草图由 Component 持有，可被多个特征引用。

> **设计规范**：实现的 `Feature`（`crates/model/src/feature.rs`，经 `proto/omcom.proto` 的 `FeatureData` 对外暴露）是**扁平类型化结构体**，而非操作 `oneof`：`id, name, feature_type (FeatureType 枚举), dependency_ids, sketch_id, input_sketch_ids, depth, direction, angle, axis_sketch_id, is_cut, order, suppressed, cached_geometry, dirty, lod, boolean_op (BooleanOp {Union, Subtract, Intersect}), pattern_count_*`。

```protobuf
message Feature {
  string id = 1;
  string name = 2;
  FeatureType type = 3;
  repeated string dependency_ids = 4;   // 父特征
  string sketch_id = 5;                  // 源草图
  repeated FeatureParameter parameters = 6;
  bool suppressed = 7;                   // 临时禁用

  oneof operation {
    ExtrudeFeature extrude = 10;
    RevolveFeature revolve = 11;
    SweepFeature sweep = 12;
    LoftFeature loft = 13;
    ChamferFeature chamfer = 14;
    FilletFeature fillet = 15;
    BooleanFeature boolean = 16;
    PatternFeature pattern = 17;
    ShellFeature shell = 18;
  }
}

enum FeatureType {
  FEATURE_TYPE_UNSPECIFIED = 0;
  EXTRUDE = 1;          // 沿方向拉伸轮廓
  REVOLVE = 2;          // 绕轴旋转轮廓
  SWEEP = 3;            // 沿路径扫掠轮廓
  LOFT = 4;             // 多截面放样
  BOOLEAN = 5;          // 布尔运算（切/并/交）
  CHAMFER = 6;          // 边倒角
  FILLET = 7;           // 边圆角
  PATTERN = 8;          // 线性/圆形阵列
  SHELL = 9;            // 抽壳
  HOLE = 10;            // 简单孔/异型孔向导（7 种孔型）
  WELD = 11;            // 焊缝特征（Tekla）
  SHEET_METAL = 12;     // 钣金折弯/展平（SOLIDWORKS）
  BOLT_GROUP = 13;      // 螺栓阵列/圆周/XY 列表（Tekla）
  REFERENCE_GEOMETRY = 14; // 参照几何（基准面/轴/坐标系/点/边界框/网格系统）
}

> **实现状态**：已交付的 `FeatureType`（Rust，`crates/model/src/feature.rs`；`proto/omcom.proto` 中有对应 `FeatureType`）实现如下：Extrude、Revolve、Sweep、Loft、Fusion、LoftedFusion、Boolean、Chamfer、Fillet、Pattern、Shell（`Fusion`/`LoftedFusion` 为上表未列出的已实现附加类型）。`HOLE`、`WELD`、`SHEET_METAL`、`BOLT_GROUP`、`REFERENCE_GEOMETRY` 为 🗓 计划中——下方调研标注描述其目标行为。实现的布尔特征在扁平结构体上使用 `BooleanOp { Union, Subtract, Intersect }`，而非上文的 `BooleanFeature` 消息。

> **异型孔向导子类型**（源自 SOLIDWORKS，`docs/research/solidworks_components.md §3.5`）：7 种孔型：Counterbore、Countersink、Hole、Straight Tap、Tapered Tap、Legacy、Slots（3 种槽口变体：Straight/Arc/Conical）。

> **螺栓组子类型**（源自 Tekla，`docs/research/tekla_components.md §7`）：BoltArray（矩形网格）、BoltCircle（圆周）、BoltXYList（任意位置）。

> **螺栓组 HoleType**（源自 Tekla，`docs/research/tekla_components.md §8.5`）：`HoleType` 变体——`STANDARD`、`OVERSIZED`、`SLOTTED`（带 XY 槽口尺寸）、`BLIND`（带深度）。`BoltHolesAttributes` 对象控制孔公差和螺纹偏好。

> **焊缝特征参数**（源自 Tekla，`docs/research/tekla_components.md §8.6`）：`SizeAbove`/`SizeBelow`（焊脚尺寸）、`AngleAbove`/`AngleBelow`（坡口角度）、`AroundWeld`（全周焊 vs 侧焊）、`ContourAbove`/`ContourBelow`（平/凸/凹）、`Shape`（连续/间断/交错）、`Workshop`/`Site`（制作地点）。焊缝几何是沿焊缝路径的扫掠；`PolygonWeld` 变体沿拾取的多边形路径。

> **钣金特征**（源自 SOLIDWORKS，`docs/research/solidworks_components.md §12`）：Base Flange、Miter Flange、Edge Flange、Swept Flange、Hem、Jog、Louver、Forming Tool、Break Corner。特征引擎支持展平/折叠以生成平板展开图。

> **圆角子类型**（源自 SOLIDWORKS，`docs/research/solidworks_components.md §3.3`）：8 种子类型——`Constant Size`、`Multiple Radius`、`Round Corner`、`Setback`、`Variable Size`、`Face`、`Full Round`、`Asymmetric`。每种需要不同的参数模式（如 Setback 在顶点处使用回退距离；Variable Size 沿边使用控制点）。当前的 `FilletFeature`（edge_ref + radius）仅满足 Constant Size。

> **阵列子类型**（源自 SOLIDWORKS，`docs/research/solidworks_components.md §3.4`）：缺失的子类型——`Curve-Driven`、`Fill`、`Sketch-Driven`、`Table-Driven`、`Variable`。当前的 `PatternFeature`（PatternType + count + spacing）仅支持 Linear/ Rectangular/Circular。`Curve-Driven` 需要参考曲线；`Fill` 填充边界区域；`Sketch-Driven` 从草图分布阵列点；`Table-Driven` 使用 (x,y,z) 坐标表；`Variable` 对每个实例应用缩放/旋转。

message ExtrudeFeature {
  double depth = 1;
  bool is_symmetric = 2;       // 两侧拉伸
  double draft_angle = 3;      // 拔模角度
  bool is_cut = 4;             // 切除（布尔减）
  ExtrudeEndCondition end_condition = 5;
}

enum ExtrudeEndCondition {
  BLIND = 0;          // 指定深度
  THROUGH_ALL = 1;    // 贯通
  TO_NEXT = 2;        // 到下一个面
  TO_SURFACE = 3;     // 到指定面
}

message RevolveFeature {
  double angle = 1;
  Point2D axis_start = 2;
  Point2D axis_end = 3;
  bool is_cut = 4;
}

message SweepFeature {
  string path_sketch_id = 1;
  bool is_solid = 2;
  bool is_cut = 3;
}

message LoftFeature {
  repeated string sketch_ids = 1;
  bool is_solid = 2;
  bool is_cut = 3;
}

message ChamferFeature {
  string edge_ref = 1;
  double distance = 2;
}

message FilletFeature {
  string edge_ref = 1;
  double radius = 2;
}

message PatternFeature {
  PatternType pattern_type = 1;
  int32 count_x = 2; int32 count_y = 3; int32 count_z = 4;
  double spacing_x = 5; double spacing_y = 6; double spacing_z = 7;
}

enum PatternType {
  PATTERN_LINEAR = 0;       // 线性阵列
  PATTERN_RECTANGULAR = 1;  // 矩形阵列
  PATTERN_CIRCULAR = 2;     // 环形阵列
}

message BooleanFeature {
  BooleanOperationType type = 1;
  repeated string target_feature_ids = 2;
}

enum BooleanOperationType {
  BOOLEAN_UNSPECIFIED = 0;
  UNION = 1;        // 并集
  SUBTRACTION = 2;  // 差集
  INTERSECTION = 3; // 交集
}
```

构建历史（特征树）：

```
特征树:
  BaseExtrude (wall_s1, depth=0.2)
    └─ CutExtrude (door_cut, depth=0.15)     ← 依赖于 BaseExtrude
         └─ Fillet (door_cut.edges, r=0.05)   ← 依赖于 CutExtrude

操作:
  - add_feature → 追加到树，重算下游
  - edit_feature(id, params) → 设置回滚游标，重算所有下游
  - suppress_feature(id) → 临时禁用
  - rollback_to(id) → 设置游标，隐藏后续特征
```

### 5.7 参数系统

命名参数及其表达式驱动特征尺寸。参数之间可以通过表达式相互引用。

```protobuf
message Parameter {
  string name = 1;
  string expression = 2;       // "5000" 或 "width * 2" 或 "sin(angle) * length"
  double value = 3;            // 计算后的值
  string unit = 4;             // mm / deg / ...
  string description = 5;
}

message FeatureParameter {
  string feature_id = 1;
  string param_name = 2;       // 映射到特征字段（如 "depth"、"radius"）
  string expression = 3;       // 可引用其他参数
}
```

表达式引擎支持：

> **实现状态**：当前模板参数由 `crates/model/src/feature_template.rs` 中的 `resolve_param()` 解析，支持直接查找（`"length"`）、简单算术（`"1.5 * DN"`）、字面量（`"360"`）。完整的函数集（`sin()`、`if()` 等）、单位与跨参数引用 🗓 计划中。

- 算术运算: `+`, `-`, `*`, `/`, `^`
- 函数: `sin()`, `cos()`, `tan()`, `sqrt()`, `abs()`, `min()`, `max()`
- 引用: `length`, `width`, `wall_1.height`
- 条件: `if(condition, val_true, val_false)`

## 6. 对象生成规则系统 (GenRule)

> **⚠️ 第六阶段更新**：自 GenRule 重构（第六阶段）后，主要几何生成路径已从 GenRule 迁移到**默认模板注册中心 + 构件目录**。
>
> 当通过命令系统创建墙时，当前流程为（命令为类型化的 `OmniCommand` 消息——`proto/omcom.proto`，以 protobuf/serde JSON 序列化，由 `crates/omcom` 中的 Executor 执行；`create wall W1` *文本*形式是 [RFC-0001](../RFC/RFC-0001-omcom-text-dsl.zh-CN.md) 定义的未来交互面，目前为 Draft）：
> 1. `default_catalog_component(Wall)` 解析为 `"Concrete-Wall"`（来自 **Library**）
> 2. 从 Library 的 `CatalogStore` 中查找 `Concrete-Wall` 的 `ComponentDef`
> 3. 实例化其关联的 `FeatureTreeTemplate`（来自 Library 的 `FeatureTreeRegistry`）
> 4. **系列检查**：如果项目已加载系列，验证 `Concrete-Wall` 是否被至少一个系列允许
> 5. 特征引擎通过 BRep 操作重建构件
>
> **重要：GenRule 有两个层次，弃用状态不同：**
>
> | 层次 | 内容 | 状态 |
> |------|------|------|
> | **GenRule 作为分类** | `GenRuleHint` 枚举，`primary_gen_rule()` 映射（Wall → LayerExtrusion，Beam → LinearComponent 等） | ✅ **活跃**——作为领域几何结构的分类法，用于 `default_catalog_component()`（模板选择）和 `default_series_for_type()`（系列选择） |
> | **GenRule 作为生成** | `generate_geometry()` 调用 `build_wall()`、`build_pipe()` 等直接生成原始网格 | ❌ **已弃用**——被目录特征树模板（`FeatureTreeTemplate` → Feature Engine → BRep）取代 |
>
> 简言之：**5 种 GenRule 类型仍然描述每个对象*有什么类型*的几何**（墙的 LayerExtrusion、梁的 Primitive），但实际的*生成*现在由目录数据驱动，而非硬编码函数。

`generate_geometry()` 函数（旧 GenRule 路径）**已弃用**。
下面的 GenRule 类型保留为**概念分类层**（由 `primary_gen_rule()` 用于类型提示），但不再驱动几何生成。

GenRule 是连接领域语义层与统一几何内核的分类层。每个 ObjectType 有一个概念性的 GenRule 类型，描述其几何结构。

> **设计规范**：本节的 `ObjectTypeDef`、`GenRule`、`LinearComponentGen`、`AssemblyGen`、`BooleanSubtractGen` protobuf 消息为目标 schema——在代码中，GenRule 以 `GenRuleHint` Rust 枚举（`crates/model/src/gen_rule/mod.rs`）及输入结构体 `LayerExtrusionInput`、`LinearComponentInput`、`AssemblyInput`、`BooleanSubtractInput` 存在；这些消息均尚未进入 proto。

**架构说明（历史）**：原有的 GenRule 架构基于特征引擎和约束求解器实现。当用户键入 `create wall std_wall 0,0 10,0 3.6`（现对应类型化 `OmniCommand`）时，GenRule 内部会：
1. 创建包含墙体轮廓的草图
2. 应用默认约束（底部水平、高度竖直）
3. 以指定厚度拉伸草图
4. 返回得到的特征树

自第六阶段起，该展开过程改由**目录特征树模板**完成。下列 5 种 GenRule 类型仍作为几何分类的**概念分类法**。

### 6.1 ObjectTypeDef 定义

```protobuf
message ObjectTypeDef {
  ObjectType type = 1;
  GenRule generation = 2;       // 几何生成规则
  BehaviorSet behaviors = 3;    // 交互行为
  ConstraintSet constraints = 4; // 约束规则
}
```

### 6.2 五种 GenRule

所有领域（建筑、工厂、市政、机械）的几何生成统一归类为 5 种 GenRule：

```protobuf
message GenRule {
  oneof rule {
    PrimitiveGen primitive = 1;   // 直接图元
    LayerExtrusionGen layered = 2; // 层拉伸
    LinearComponentGen linear = 3; // 线构件
    AssemblyGen assembly = 4;     // 组合装配
    BooleanSubtractGen subtract = 5; // 布尔剪切
  }
}
```

| GenRule | 适用领域 | 典型对象 | 说明 |
|---------|---------|---------|------|
| 1. Primitive | 工厂、机械 | 罐、泵、阀、齿轮 | 直接将类型参数映射为单个体元（**体构件**） |
| 2. LayerExtrusion | 建筑 | Wall、Slab、Floor、Roof | 将面边界 + 层结构展开为多条 Extrusion；**面构件**的主要 GenRule |
| 3. LinearComponent | 建筑、市政、工厂 | Beam、Column、Road、Bridge、Pipe、CableTray | 路径 + 变截面横断面 + 外部约束；**线构件**的 GenRule |
| 4. Assembly | 机械、工厂 | 设备、装配体 | 组合多个图元为一个对象（**体构件**） |
| 5. BooleanSubtract | 全部 | 开孔、SNO | 在已有几何上做布尔差集 |

> **注意**：表中移除了 Wall 作为 LinearComponent 的归属。墙是**面构件**（见 [§1](#1-概述)），通过 LayerExtrusion 生成，而非 LinearComponent。

### 6.3 领域 GenRule 映射

```
领域           主要 GenRule        次要 GenRule
----------    ----------------    -------------
建筑           2. LayerExtrusion    5. BooleanSubtract
工厂           1. Primitive         3. LinearComponent + 5
市政           3. LinearComponent   1. Primitive
机械           4. Assembly          1. Primitive
数字孪生       (复用上述规则)       + IoT 数据绑定
```

### 6.4 线构件 (LinearComponent)

线构件是 OmniM 对 Civil3D 走廊模型和数维道路线构件的抽象，将 Extrusion 和 Sweep 统一为单一概念：路径 + 变化横断面 + 外部约束。建筑结构构件（Beam、Column）是不变截面的线构件特例——路径 + 固定截面轮廓。道路 = 变截面的线构件（平面线形 + 竖曲线 + 分段横断面 + 地形目标约束）。

> **墙不是线构件**：与梁/柱/管道（路径驱动）不同，墙是**面构件**（见 [§1](#1-概述)），其几何由面边界（矩形或草图轮廓）定义，层材质叠层沿法向拉伸生成厚度。墙使用 **LayerExtrusion**（§6.2），而非 LinearComponent。面边界提供拉伸轮廓，层叠层提供厚度。这一区分至关重要：墙的几何由*面*属性（层、边界草图）定义，而线构件的几何由*路径*+截面属性定义。

```protobuf
message LinearComponentGen {
  Alignment3D main_path = 1;
  repeated Alignment3D edge_paths = 2;
  repeated CrossSectionLayer layers = 3;
  repeated VariationRule variations = 4;
  optional TerrainTarget terrain = 5;
  // 未来增强（参见 RESEARCH_INTEGRATION.md §2.3）：
  repeated CrossSectionSegment segments = 6;          // 多段变截面
  repeated ComponentSequenceItem component_sequence = 7; // 元件序列模式
  repeated ExternalTarget external_targets = 8;       // 外部约束求解
}

message CrossSectionLayer {
  string name = 1;
  double default_width = 2;
  double default_thickness = 3;
  string material_id = 4;
  string function = 5;
}

enum InterpolationMethod {
  INTERPOLATION_CONSTANT = 0;
  INTERPOLATION_LINEAR = 1;
  INTERPOLATION_PARABOLIC_UP = 2;   // OBM 拼装梁变截面
  INTERPOLATION_PARABOLIC_DOWN = 3;
  INTERPOLATION_CIRCULAR_UP = 4;
  INTERPOLATION_CIRCULAR_DOWN = 5;
}

message VariationRule {
  double start_station = 1;
  double end_station = 2;
  string target = 3;
  InterpolationMethod interpolation = 4;   // 扩展为 6 种方法（参考 Bentley）
  double start_value = 5;
  double end_value = 6;
}

// 多段变截面（OBM Beam Rows / OTD Tunnel Unit）
message CrossSectionSegment {
  double start_station = 1;
  double end_station = 2;
  CrossSectionAtStation start_section = 3;
  CrossSectionAtStation end_section = 4;   // == start_section => 简单扫掠，无放样
  InterpolationMethod interpolation = 5;
}

// 元件序列项（E3D BRANCH 构件）
message ComponentSequenceItem {
  string component_def_id = 1;   // 目录 ComponentDef 引用
  double position_along_path = 2;
  map<string, string> params = 3;
}

// 外部约束（ORD End Condition / Daylight）
message ExternalTarget {
  TargetType target_type = 1;   // Surface / Offset / Elevation / Width
  ConstraintType constraint = 2; // Daylight / Offset / Elevation / Width
  string target_id = 3;
  int32 priority = 4;           // ORD End Condition Exception：多个目标按优先级顺序尝试
  optional double max_distance = 5; // 最大 daylight 搜索距离

  // 标高目标语义（Civil 3D）：横断面在每个桩号处所符合的纵断面或曲面高程
  optional string elevation_profile_id = 10;
  optional string elevation_surface_id = 11;

  // 宽度目标语义（Civil 3D）：条件驱动的宽度变化
  optional string width_condition_id = 12;  // 如 "daylight 命中现有地面则 width = 3.6m，否则 3.0m"
  optional double default_width = 13;
}
```

> **已实现的输入 schema**（`crates/model/src/gen_rule/linear_component.rs`）：`LinearComponentInput { alignment: Vec<Point3D>, cross_sections: Vec<CrossSectionAtStation>, segments, ring_segments }`；`CrossSectionLayer` 承载 `name`、`shape`（`CrossSectionShape`）、`default_width`、`default_thickness`、`material_id: Option<String>`、`function`。截面沿采样路径放样（变截面，`segments` 为静态计数——见 §6.6）。上方更丰富的 `LinearComponentGen` 消息（边缘路径、变化规则、地形/外部目标）为设计规范——🗓 计划中。

**四类目标族**（源自 Civil 3D Target Mapping，`docs/research/site_terrain_components.md §2.5`）：Civil 3D 建模了一组四类目标族，`ExternalTarget`/`TargetType` 应予以反映：

| 目标类型 | 语义 | OmniM 映射 |
|---------|------|-----------|
| **Surface** | Daylight/附着到 TIN 曲面（向地形找平、桥面到设计面、路基基层） | `target_id` 中的 `TerrainModel` id |
| **Offset** | 特征线或曲面的偏移作为侧向目标 | `FeatureLine`/曲面偏移 id |
| **Elevation** | 横断面在每个桩号处所符合的纵断面或曲面高程 | `elevation_profile_id` / `elevation_surface_id` |
| **Width** | 条件驱动的宽度（宽度根据地貌/设计条件变化） | `width_condition_id` + `default_width` |

现有的 `TargetType`（Surface/FeatureLine/Plane）是 **Surface** 和 **Offset** 类型的子集。独立的 **Elevation** 和 **Width** 目标新增：(1) 驱动竖向线形的*纵断面/曲面高程*约束，以及 (2) 由条件而非固定值选择有效横断面宽度的*按条件变宽*语义。这匹配 Civil 3D 中廊道宽度可依据目标 daylight 成败或设计条件自动切换的方式。

**外部目标异常处理**：基于 Bentley ORD End Condition Exception 分析（`docs/research/bentley_openroads_components.md §6.3.1`），可以指定多个 `ExternalTarget` 条目，各带不同的 `priority` 值。求解器按优先级顺序依次尝试每个目标；第一个成功 daylight（即在 `max_distance` 内与有效曲面相交）的目标被采用。这实现了回退行为：例如"先尝试向现有地面（Existing Ground）找平；若 50m 内无交点，则尝试向设计标高平面（Design Grade）找平。"

门、窗可以表示为横断面的变化规则（在某桩号位置上某层厚度变为 0），而非独立对象做布尔差集——这是线构件思想对建筑领域的重新诠释。

**显式墙体开洞集合**（源自 Revit，`docs/research/revit_components.md §5.1`）：Revit 将开洞作为命名的、可统计的、可独立编辑的构件（门/窗开洞）来跟踪。作为上述横断面变化方法之外的替代方案，开洞可以建模为墙体上的显式 `Openings` 集合——每个开洞都是一个带自身标识、尺寸和统计数据的子对象，引用宿主墙及沿墙路径的位置。

**变截面插值方法**：基于 Bentley OBM 拼装钢梁分析（`docs/research/bentley_openroads_components.md §2.1.3`），插值方法从单一线性扩展为 6 种：Constant、Linear、Parabolic Up/Down、Circular Up/Down。覆盖所有已调研平台的结构变截面场景。

**多段变截面**：基于 Bentley OBM Beam Rows + Different End Template 和 OTD 多 Tunnel Unit 分析，`CrossSectionSegment` 列表支持每段独立定义截面。当段内 `start_section == end_section` 时，该段为简单扫掠（无放样）——匹配 OTD §4.1 观察到的"段内插值 + 段间跳跃"工程现实。

**多层同心截面（电缆/导管/管道）**：除上述结构和廊道截面外，LinearComponent 还可用于电缆和导管的**同心多层截面**建模。基于 AVEVA Instrumentation 电缆分析（`docs/research/e3d_instrumentation_components.md §3.5.2`），电缆为 5 层同心截面：导体（Conductor）→ 绝缘层（Insulation）→ 屏蔽层（Screen/Shield）→ 铠装层（Armor）→ 外护套（Outer Sheath）。这些映射为 `CrossSectionLayer` 条目，其中 `default_width` 为外径、`default_thickness` 为径向层厚（同心排序）。未来应增强 `CrossSectionLayer`，增加 `center_radius` 和 `radial_thickness` 以显式定义同心截面。

> **电缆桥架填充率与敷设**（`docs/research/e3d_instrumentation_components.md §3.1.4, §3.5.2`）：电缆桥架敷设由**填充率**规则驱动（电缆截面积之和 / 桥架截面积），外加信号/电源分离和自动路径选择。每条电缆携带 `cross_section` 属性（mm²）用于计算桥架填充率。填充率同时约束敷设可行性和每桥架的最大电缆数量。

**钢筋作为 LinearComponent**（源自 Tekla Structures，`docs/research/tekla_components.md §6.1`）：配筋（Rebar）是带圆形截面沿路径（直线或弯折）扫掠的 LinearComponent。除基础 LinearComponent 参数外，关键参数包括：Size（直径）、Grade（等级，如 B500B）、RadiusValues（弯曲半径）、OnPlaneOffsets / FromPlaneOffset（定位）、StartHook / EndHook（弯钩角度）、SpliceType / SpliceOffsets（搭接）、StirrupType（箍筋 vs 非箍筋）、ExcludeType（分布模式）。钢筋修改器（RebarPropertyModifier、RebarSplitter、RebarEndDetailModifier）作为特征引擎中的后生成特征操作来建模。

**标高约束的竖向范围**（源自 Revit，`docs/research/revit_components.md §5.1`）：建筑竖向构件（墙/板/屋顶）约束到标高（顶部/底部）而非绝对高度。LinearComponentGen 支持 `top_level_id` / `bottom_level_id` 约束概念，以实现建筑领域对等：竖向范围相对于命名的标高高度解析，使得标高变化时高度可自动调整。

**连接点到连接点的创建模式**（源自 Revit，`docs/research/revit_components.md §7`）：Revit 中的 MEP 构件（管道/风管/电缆桥架）通过选择起点和终点连接器创建，而非仅指定路径。这是线构件的一种额外创建模式：用户不是在现有构件上拾取两个连接器，而是指定几何路径，系统在两者之间自动敷设路径（尊重坡度、净距和连接类型规则）。`main_path` 随后自动生成，而非用户指定。

### 6.5 线构件元件序列模式（E3D BRANCH）

> **调研参考**：E3D BRANCH 将管道描述为沿路径的离散构件序列（管段、弯头、三通、大小头）。详见 `docs/research/aveva_e3d_primitives.md`、`docs/research/comparative_analysis.md`、`docs/research/genrule_design_analysis.md`。

线构件的第三种模式——**元件序列**——尚未实现。不同于固定截面模式（Beam/Column）或变横断面模式（Road），该模式沿路径依次放置离散构件：

```
ComponentSequence
├── ComponentItem: PipeSegment (length=6000, DN=100)
├── ComponentItem: Elbow (angle=90°, DN=100)
├── ComponentItem: PipeSegment (length=3000, DN=100)
├── ComponentItem: Reducer (DN=100→80)
└── ComponentItem: Valve (DN=80)
```

每个 `ComponentItem` 引用目录中的 `ComponentDef` 并指定其在路径上的位置。这是 E3D BRANCH 模式——管道不是连续拉伸，而是目录构件的序列。

**实现计划**：在未来阶段向 `LinearComponentInput` 增加 `ComponentSequenceItem`。详见 `docs/research/genrule_design_analysis.md §5.2`。

### 6.6 自适应采样与 LOD 策略

> **调研参考**：`docs/research/genrule_design_analysis.md §5.2`。

**自适应采样**：当前 `LinearComponentInput` 的 `segments` 参数是固定值。未来实现应根据路径曲率动态调整分段密度——曲率大处加密，直线处稀疏——以平衡几何精度与性能。

**Civil 3D 采样频率节点**（源自 Civil3D §4.3）：廊道采样密度应由具体触发器驱动，而非仅靠曲率。以下采样频率节点类型源自 Civil 3D 的廊道频率设置：
- `AlongCurves` — 沿平曲线/竖曲线加密采样（如基于半径每 1-2m）
- `AlongTangents` — 沿直线段稀疏采样（如每 10-20m）
- `AtProfileHighLowPoints` — 在竖曲线的每个凸点/凹点插入采样
- `AtSuperelevationCriticalPoints` — 在超高过渡桩号处插入采样（加宽起点/终点、全超高）
- `AtAlignmentGeometryPoints` — 在每个线形交点、直圆点、圆直点及竖曲线变坡点处插入采样
- `AtStructureCrossings` — 在每个相交管道、涵洞或结构物位置插入采样

这些采样节点类型应作为 `repeated SamplingNode sampling_nodes` 字段添加到 `LinearComponentGen` 输入中，使廊道引擎能够在正确桩号处生成几何，兼顾精度和性能。

**LOD（细节层次）策略**：当前 `lod`/`level_of_detail` 字段是单一的曲面细分容差。未来应支持多级 LOD 系统（粗糙 / 中等 / 精细），根据视角距离切换渐进网格，实现大型模型的高效渲染。

### 6.7 其他调研发现

> **调研参考**：`docs/research/genrule_design_analysis.md §5.3`、`docs/research/comparative_analysis.md §8.3`。

**ParametricOperation 中的布尔运算**：现有的 `BooleanSubtract` GenRule 模块保留为独立模块。未来应在 `ParametricOperation` 中增加 `BooleanOperation` 类型（如 `BooleanUnion`、`BooleanIntersect`、`BooleanSubtract`），使特征引擎能直接在特征树内执行布尔运算，而不仅限于 GenRule 层面。

**流程图式 VariationRule（SAC 模式）**：当前 `VariationRule` 仅支持简单插值（constant/linear/smooth）。未来应支持 Civil3D Subassembly Composer 风格的流程图逻辑——`Sequence`、`Decision`、`Switch` 节点——实现基于参数、目标和外部条件的条件几何生成。

> **设计规范**：TerrainModel、LogicalObject、BooleanSubtract 增强、Assembly 增强、约束系统（§6.8–§6.12）均为设计规范。其 protobuf 消息仅定义于本文档，尚未实现于代码库/proto 文件。

### 6.8 地形模型 (TerrainModel TIN 曲面)

> **调研参考**：多平台场地/地形分析（`docs/research/site_terrain_components.md`），Bentley OpenRoads Terrain Model（`docs/research/bentley_openroads_components.md §6.1`）。地形是 TIN 曲面，不属于扫掠/放样/布尔中的任一类——不适用 5 类 GenRule。

`TerrainModel` 是与 5 类 GenRule 并列的独立几何类型，表示不规则三角网（TIN）曲面。

```protobuf
message TerrainModel {
  repeated TerrainVertex vertices = 1;
  repeated TerrainTriangle triangles = 2;    // 由顶点+断裂线自动生成
  repeated TerrainBreakline break_lines = 3; // 三角形不可跨越的边
  repeated TerrainBoundary boundaries = 4;   // 外边界 / 基准边界 / 空洞边界
  repeated Void voids = 5;                   // 地形空洞
  TerrainRole role = 6;                      // Existing / Design / Subgrade / Reference
  optional GeographicReference geo_ref = 7;  // 坐标系（EPSG，投影）
  map<string, string> properties = 8;        // 来源、精度、元数据
}

message TerrainVertex {
  string id = 1;
  double x = 2; double y = 3; double z = 4;
  map<string, string> properties = 5;       // 例如 "source" = "survey", "accuracy" = "0.01m"
}

message TerrainTriangle {
  string id = 1;
  uint32 v0 = 2; uint32 v1 = 3; uint32 v2 = 4;  // 顶点索引
}

message TerrainBreakline {
  string id = 1;
  repeated uint32 vertex_indices = 2;        // 有序点链
  bool enforced = 3;                         // 三角形不可跨越
}

message TerrainBoundary {
  string id = 1;
  repeated uint32 vertex_indices = 2;        // 闭合多边形
  BoundaryType type = 3;                     // OUTER / BENCHMARK / VOID
}

enum BoundaryType {
  BOUNDARY_OUTER = 0;    // 地形外边边界
  BOUNDARY_BENCHMARK = 1; // 基准边界（限制体积计算区域）
  BOUNDARY_VOID = 2;     // 地形空洞
}

enum TerrainRole {
  TERRAIN_EXISTING = 0;   // 被减面（现有地面）
  TERRAIN_DESIGN = 1;     // 加面（设计标高）
  TERRAIN_SUBGRADE = 2;   // 独立计方面（路基面）
  TERRAIN_REFERENCE = 3;  // 参考面，不参与体积计算
}
```

**角色系统**：源自 Bentley Terrain Model 的 Volume Option（`docs/research/site_terrain_components.md §8.3`）。`TerrainRole` 枚举控制曲面如何参与体积计算：
- `TERRAIN_EXISTING`："之前"的曲面——在挖/填方分析中被减
- `TERRAIN_DESIGN`："之后"的曲面——设计意图标高
- `TERRAIN_SUBGRADE`：用于路基方量计算的独立曲面（如道路基层）
- `TERRAIN_REFERENCE`：参考曲面，不参与体积计算

**创建来源**：TerrainModel 可通过以下方式创建：
1. 测量点（从 `.txt` / `.csv` / 全站仪数据导入）
2. 等高线（从 DWG/DXF/LandXML 导入）
3. 廊道输出（Civil 3D / Bentley 廊道 → TIN 曲面）
4. 平面地形（给定高程的水平面）
5. 外部引用（LandXML、IFC 或点云）
6. 手动输入点（交互式添加点）

**编辑 API**：TerrainModel 支持以下操作：
- `add_vertex(x, y, z)` / `remove_vertex(id)` / `modify_vertex(id, z)`
- `add_breakline(indices)` / `remove_breakline(id)`
- `add_boundary(indices, type)` / `remove_boundary(id)`
- `merge(other)` — 合并两个 TIN 曲面
- `split(plane)` — 沿平面分割地形
- `smooth(iterations)` — 平滑 TIN 曲面
- `auto_triangulate()` — 从顶点+断裂线重新生成三角形

**分析 API**：TerrainModel 支持空间查询：
- `get_elevation(x, y)` → 某点的插值高程
- `get_slope(x, y)` → 某点的坡度
- `get_contour_lines(elevation_interval)` → 等高线曲线
- `volume_analysis(other, method)` → CutVolume / FillVolume / EarthworkBalance
- `surface_area()` → 总表面积
- `daylight_analysis(linear_component, slope_ratio)` → 与线构件端部条件的相交曲线

**用途**：TerrainModel 作为：
1. LinearComponent End Condition 的外部约束目标（daylight 求解）
2. 挖/填方体积计算的源（两 TIN 曲面间 BooleanSubtract）
3. 地形、场地平整、走廊输出的可视化曲面
4. 派生对象的基础：CutVolume、FillVolume、GradingSurface、EarthworkBalance

**体积计算方法**：基于 Bentley ORD 分析（`docs/research/bentley_openroads_components.md §6.2.1`）和 Civil 3D 整平（`docs/research/site_terrain_components.md §7.3`，三种算法），通过 `volume_analysis()` 支持：
1. **Element Component Quantities（元素构件工程量）**——LinearComponent 各层的逐元素体积
2. **Corridor Component Quantities（廊道构件工程量）**——按层的廊道整体体积
3. **End Area Volumes（平均端面积法）**——基于断面的桩号间体积（Average End Area）
4. **Terrain-to-Terrain（曲面到曲面）**——两 TIN 曲面间 BooleanSubtract（现有 − 设计）
5. **Terrain-to-Plane（曲面到平面）**——TIN 曲面与水平面之间的 BooleanSubtract
6. **Quantities Report by Named Boundary（按命名边界工程量报告）**——地形某个围合区域内的体积
7. **Composite / Conic / Prismoidal**——Civil 3D 的三种体积算法（Conic 对弯曲廊道最精确）

**派生对象**（源自 `docs/research/site_terrain_components.md §7.3`）：
- **CutVolume** — 挖方体积结果（BooleanSubtract：Existing − Design）
- **FillVolume** — 填方体积结果（BooleanSubtract：Design − Existing）
- **GradingSurface** — 角色为 `TERRAIN_DESIGN` 的 TerrainModel，由 Grading Group 生成（见 §6.8.1 下方）
- **Grading Group** — 一个 `GradingGroup` 绑定一条特征线、一个整平面、一个目标曲面和整平规则（见 §6.8.1 下方）
- **EarthworkBalance** — 比较有界区域内挖方与填方总量的逻辑对象
- **Slope** — 横断面为坡面、沿边界/边线扫掠的 LinearComponent，带 `ExternalTarget` 用于 daylight 相交
- **SiteComponent** — 附着于地形表面的 `ComponentDef`（树木、电线杆、消防栓）
- **FeatureLine** — 零厚度的控制/特征线（LinearElement），用作整平参考
- **ContourLine** — 表示固定高程等高线的曲线元素，用于显示/分析

#### 6.8.1 整平组（Civil 3D Grading Group）

> **调研参考**：`docs/research/site_terrain_components.md §2.3`。一个 Grading Group 将设计整平面、现有地形目标以及从特征线驱动的坡度规则绑定在一起，生成一个带可选台阶/平台坡度的设计整平曲面。

```protobuf
message GradingGroup {
  string id = 1;
  string name = 2;

  // 驱动整平的特征线（场地、边界、平台边缘）
  string feature_line_id = 3;

  // 正在创建设计整平曲面（role=DESIGN）
  string grading_surface_id = 4;

  // 目标/现有地形曲面（role=EXISTING）
  string target_surface_id = 5;

  // 应用于特征线的整平规则
  repeated GradingRule rules = 6;

  // 可选：多平台台阶式整平
  repeated BenchLevel bench_levels = 7;
}

message GradingRule {
  string name = 1;                  // 如 "挖方 1:1.5", "填方 1:2"
  GradingRuleType type = 2;         // CUT / FILL / DAYLIGHT
  double cut_slope = 3;             // 如 1.5 (1V:1.5H)
  double fill_slope = 4;            // 如 2.0 (1V:2H)
  optional double max_cut_depth = 5; // 最大挖深，超过则设台阶
  optional double max_fill_height = 6; // 最大填高，超过则设台阶
  double daylight_max_distance = 7; // daylight 相交搜索范围
  double min_slope_length = 8;      // 最小坡面长度（设计标准）
  double max_slope_ratio = 9;       // 最大坡度比（设计标准）
}

enum GradingRuleType {
  GRADING_RULE_DAYLIGHT = 0;   // 坡度到地形相交
  GRADING_RULE_TO_SURFACE = 1; // 坡度到指定高程曲面
  GRADING_RULE_TO_ELEVATION = 2; // 坡度到固定高程
  GRADING_RULE_TO_DISTANCE = 3; // 坡度投影固定距离
}

// 台阶式整平：多平台阶梯式整平
message BenchLevel {
  string name = 1;               // 如 "平台 1", "截水沟 1"
  double elevation = 2;          // 平台标高
  double bench_width = 3;        // 平台水平宽度
  double bench_slope = 4;        // 平台横坡（排水用）
  optional double ditch_width = 5;  // 平台内侧可选排水沟宽度
  optional double ditch_depth = 6;
}
```

**整平工作流**（源自 `site_terrain_components.md §2.3`）：
1. 在设计曲面上定义一条**特征线**（场地边缘、边界或平台边缘）
2. 创建一个**Grading Group**，引用该特征线、一个目标曲面（现有地形）以及一个或多个**Grading Rules**
3. 整平规则携带设计标准：`CutSlope`（如 1V:1.5H）、`FillSlope`（如 1V:2H）、`MaxCutDepth`、`MaxFillHeight`、`min_slope_length`、`max_slope_ratio`、`bench_width`
4. 求解器通过根据规则向外投影特征线，计算与目标曲面的 daylight 相交，生成整平曲面
5. **台阶式整平**（多平台/阶梯式）：当坡度超过最大挖深时，在 `bench_levels[n].elevation` 处以指定 `bench_width` 插入一个平台，形成阶梯式坡面

### 6.9 逻辑对象 (LogicalObject Tag / Loop / LineNumber)

> **调研参考**：AVEVA Instrumentation 的 Loop/Tag 系统（`docs/research/e3d_instrumentation_components.md §4`）。Loop 和 Tag 是无几何的逻辑实体，但它们是核心工程对象。

`LogicalObject` 是新增的基类型，用于无几何但携带工程数据并引用物理对象的实体。与 `Object`（物理）并列。

```protobuf
message LogicalObject {
  string id = 1;
  string name = 2;
  LogicalObjectType type = 3;
  repeated string member_object_ids = 4;     // 引用物理对象
  map<string, string> properties = 5;        // tag, loop_number, line_number 等
}

enum LogicalObjectType {
  LOGICAL_TAG = 0;          // ISA-5.1 位号（如 "FT-101A"）
  LOGICAL_LOOP = 1;         // 控制回路（如 Loop 101）
  LOGICAL_LINE_NUMBER = 2;  // 工艺管线号（如 "12\"-3A1-CS-SCH40"）
  LOGICAL_SYSTEM = 3;       // 工程系统
  LOGICAL_UNIT = 4;         // 工厂单元/区域
  LOGICAL_CABLE = 5;        // 电缆清册条目
}
```

**关键设计原则**：
1. LogicalObject 无几何、无 GenRule、无特征树
2. 通过 `member_object_ids` 持有物理对象引用
3. 携带工程属性（Tag 字符串、Loop 编号等）
4. 支持跨视图追溯：逻辑侧（P&ID）↔ 物理侧（3D 模型）

> **管线号结构化编码**（`docs/research/e3d_instrumentation_components.md §4.2.1`）：LineNumber LogicalObject 携带结构化编码 `DN"-"Spec"-"Material"-"Schedule"-"HeatTracing"-"Insulation"`（如 `12"-3A1-CS-SCH40-HT1-INS`）。这支持 P&ID 逻辑管线与 3D 物理管线运行之间的解析、校验和跨视图匹配。

### 6.10 BooleanSubtract 增强

> **调研参考**：Tekla 的 BooleanPart/Fitting/CutPlane（`docs/research/tekla_components.md §8`）。Tekla 支持三种布尔模式：实体-实体切割、平面切割、边倒角。

当前 `BooleanSubtract` GenRule 仅支持**实体-实体**差集。基于 Tekla 分析，需要增加三种模式：

```protobuf
message BooleanSubtractGen {
  // 现有：实体-实体差集
  optional string base_id = 1;       // 被切割对象
  optional string cutter_id = 2;     // 切割对象（实体）

  // 新增：平面切割（Tekla Fitting / CutPlane）
  // 用半空间平面切割基础对象，比实体布尔更高效
  optional PlaneCut plane_cut = 10;

  // 新增：边倒角/圆角（Tekla EdgeChamfer / SOLIDWORKS Fillet）
  optional EdgeChamfer edge_chamfer = 11;

  // 新增：曲面-曲面差集（挖/填方体积）
  // Civil 3D / Bentley 土方工程：两个 TIN 曲面间的布尔运算
  optional SurfaceSubtract surface_subtract = 12;
}

// 曲面-曲面布尔运算（TIN 土方工程）
message SurfaceSubtract {
  string base_terrain_id = 1;    // 例如现有地面
  string cutter_terrain_id = 2;  // 例如设计面
  VolumeMethod method = 3;       // COMPOSITE / CONIC / PRISMOIDAL / END_AREA
  VolumeResult result = 4;       // CUT / FILL / BALANCE
}

enum VolumeMethod {
  VOLUME_COMPOSITE = 0;   // Civil 3D 默认，最精确
  VOLUME_CONIC = 1;       // 弯曲廊道
  VOLUME_PRISMOIDAL = 2;  // 棱柱体修正
  VOLUME_END_AREA = 3;    // 桩号间平均端面积
}

enum VolumeResult {
  VOLUME_RESULT_CUT = 0;    // 挖方（Existing − Design > 0）
  VOLUME_RESULT_FILL = 1;   // 填方（Design − Existing > 0）
  VOLUME_RESULT_BALANCE = 2; // 净平衡比较
}

message PlaneCut {
  Plane cut_plane = 1;
  bool keep_positive_side = 2;
}

message EdgeChamfer {
  repeated uint32 edge_indices = 1;
  double distance = 2;
  optional double radius = 3;
  ChamferType type = 4;              // CHAMFER / FILLET
}
```

> **布尔并集（Tekla BooleanPart）**（源自 Tekla，`docs/research/tekla_components.md §8.1`）：Tekla `BooleanPart` 通过 `BooleanTypeEnum` 同时支持 `CUT` 和 `ADD` 模式。`BooleanSubtractGen` GenRule 应可泛化为 `BooleanOperationType` 枚举（CUT/ADD），以支持并集运算。ADD 模式对应将 `OperativePart` 几何合并到 `Father` 中——相当于 OmniM 的 `BooleanOperationType::UNION`。

### 6.11 Assembly 增强

> **调研参考**：Tekla ConnectionBase（`docs/research/tekla_components.md §3`）、Bentley Civil Cell（`docs/research/bentley_openroads_components.md §8.3`）。

Assembly GenRule 增加条件参数和过程化脚本能力：

```protobuf
message AssemblyGen {
  // 现有：声明式自对象列表
  repeated SubComponent children = 1;

  // 新增：条件参数（Tekla Connection 参数开关）
  // 如 "has_stiffeners" = true → 生成加劲肋子对象
  map<string, bool> conditions = 10;

  // 新增：过程化脚本引用（ConnectionScript）
  // 引用实现生成逻辑的目录 ComponentDef
  optional string script_component_id = 11;

  // 新增：表达式驱动子对象位置（Bentley Civil Cell）
  // 如 "cap.position_x = bridge_alignment.station(0.5).x"
  map<string, string> position_expressions = 12;
}

message SubComponent {
  string component_def_id = 1;
  Transform transform = 2;
  GenRuleHint gen_rule = 3;
  optional string condition_ref = 4;
}
```

> **ComponentInput 顺序敏感性**（源自 Tekla，`docs/research/tekla_components.md §3.2`）：Tekla 的 `ComponentInput` 对顺序敏感——`AddInputObject` + `AddInputPosition` 的顺序颠倒会产生不同结果。`AssemblyGen` 中的 `script_component_id` 引用应定义输入槽（主部件、次部件、位置）并明确顺序，因为下游 ConnectionScript 的 `Run()` 方法按顺序消费它们。

> **参数化板生成模式**（源自 Tekla，`docs/research/tekla_components.md §4.1`）：节点板、端板和底板是 ConnectionScript 的常见模式。脚本接收主部件（梁/柱）+ 次部件（支撑）+ 参数（边距、角度、螺栓位置），生成一个带参数化多边形轮廓的 `ContourPlate`。这是一个典型的 `AssemblyGen` 用例：一个 `SubComponent` 使用 `Primitive::Extrusion`，其轮廓顶点由表达式驱动。

### 6.12 约束系统 (Constraint Mates)

> **调研参考**：SOLIDWORKS 21 类 Mates（`docs/research/solidworks_components.md §4`）。SOLIDWORKS 支持 3 类配合：标准（8 种）、高级（6 种）、机械（7 种）。

在 `RelationStore` 中新增 `Constraint` 系统，与现有 `Connection` 系统并列。约束控制 Assembly 子对象之间的自由度关系。

```protobuf
message Constraint {
  string id = 1;
  string name = 2;
  ConstraintType type = 3;
  string entity_a_id = 4;
  string entity_b_id = 5;
  optional double distance = 6;
  optional double angle = 7;
  optional double ratio = 8;
  optional double min_value = 9;
  optional double max_value = 10;
}

enum ConstraintType {
  // 标准配合（SOLIDWORKS）
  CONSTRAINT_COINCIDENT = 0;
  CONSTRAINT_CONCENTRIC = 1;
  CONSTRAINT_PARALLEL = 2;
  CONSTRAINT_PERPENDICULAR = 3;
  CONSTRAINT_TANGENT = 4;
  CONSTRAINT_DISTANCE = 5;
  CONSTRAINT_ANGLE = 6;
  CONSTRAINT_LOCK = 7;

  // 高级配合
  CONSTRAINT_SYMMETRIC = 10;
  CONSTRAINT_WIDTH = 11;
  CONSTRAINT_PATH = 12;
  CONSTRAINT_LIMIT = 13;
  CONSTRAINT_SLOT = 14;

  // 机械配合
  CONSTRAINT_GEAR = 20;
  CONSTRAINT_RACK_PINION = 21;
  CONSTRAINT_SCREW = 22;
  CONSTRAINT_HINGE = 23;
  CONSTRAINT_UNIVERSAL_JOINT = 24;
}
```

---

## 7. 属性系统

> **UDA/UDET（用户自定义属性/类型）**：E3D 的 LEXICON 数据库（`docs/research/e3d_instrumentation_components.md §1.3`）提供 UDA（User Defined Attributes，用户自定义属性）和 UDET（User Defined Element Types，用户自定义元素类型）。OmniM 的属性系统通过动态属性集提供等效支持：任何对象可通过 `PropertySet` 携带任意键值对（已实现——与 `proto/omnim.proto` 的 `Property`/`PropertySet` 一致），`crates/model/src/property_def.rs` 提供按 ObjectType 的属性定义。注意：`ObjectType` 枚举中并没有专门的"用户自定义 900+"范围——唯一的通用值为 `OBJECT_TYPE_GENERIC = 999`；正式的用户自定义类型机制 🗓 计划中。

OmniM 同时支持扁平属性和 IFC 风格的属性集。

```protobuf
message Property {
  string name = 1;
  PropertyValue value = 2;
  string unit = 3;
  string description = 4;
}

message PropertyValue {
  oneof value_type {
    string string_value = 1;
    int64 int_value = 2;
    double double_value = 3;
    bool bool_value = 4;
    Point3D point_value = 5;
    Vector3D vector_value = 6;
    bytes binary_value = 7;
    google.protobuf.Timestamp timestamp_value = 8;
  }
}

message PropertySet {
  string name = 1;
  string description = 2;
  repeated Property properties = 3;
}
```

### 标准属性集（IFC 对齐）
- `Pset_WallCommon`
- `Pset_BeamCommon`
- `Pset_ColumnCommon`
- `Pset_SlabCommon`
- `Pset_DoorCommon`
- `Pset_WindowCommon`
- 以及领域专用的属性集

---

## 8. 关系系统

OmniM 拥有原生关系系统，支持：

### 8.1 包含关系（层次结构）

```protobuf
message Containment {
  string parent_id = 1;
  string child_id = 2;
  ContainmentType type = 3;
}

enum ContainmentType {
  CONTAINMENT_TYPE_UNSPECIFIED = 0;
  SPATIAL = 1;      // Spatial hierarchy (Site → Building → Storey)
  PHYSICAL = 2;     // Physical assembly (Assembly → Part)
  SYSTEM = 3;       // System membership (PipeSystem → Pipe segments)
  GROUP = 4;        // Logical grouping
}
```

### 8.2 连接关系

```protobuf
message Connection {
  string from_object_id = 1;
  string to_object_id = 2;
  ConnectionType type = 3;
  Point3D from_point = 4;
  Point3D to_point = 5;
  string connection_detail = 6;  // e.g., "Welded", "Bolted", "Flanged"
}

enum ConnectionType {
  CONNECTION_TYPE_UNSPECIFIED = 0;
  STRUCTURAL = 1;    // Structural connection (beam to column)
  MECHANICAL = 2;    // Mechanical (pipe flange)
  ELECTRICAL = 3;    // Electrical (wire to terminal)
  HYDRAULIC = 4;     // Hydraulic connection
  SPATIAL_ADJACENT = 5;  // Objects are adjacent
}
```

### 8.3 空间关系

```protobuf
message SpatialRelation {
  string object_a_id = 1;
  string object_b_id = 2;
  SpatialRelationType type = 3;
  double distance = 4;
}

enum SpatialRelationType {
  SPATIAL_RELATION_TYPE_UNSPECIFIED = 0;
  INTERSECTS = 1;
  CONTAINS = 2;
  WITHIN = 3;
  OVERLAPS = 4;
  TOUCHES = 5;
  DISJOINT = 6;
}
```

> **设计规范（🗓 计划中）**：`SpatialRelation` 尚未进入 `proto/omnim.proto`——关系系统当前实现 Containment 与 Connection（`RelationStore`，`crates/model/src/relation.rs`）。空间谓词查询（相交/包含/…）需依赖 §11 的空间索引，而索引本身也尚未实现。

---

## 9. 材料系统

```protobuf
message Material {
  string id = 1;
  string name = 2;
  MaterialCategory category = 3;
  
  // Physical properties
  double density = 4;        // kg/m³
  double youngs_modulus = 5; // Pa
  double poissons_ratio = 6;
  double yield_strength = 7; // Pa
  double ultimate_strength = 8; // Pa
  double thermal_conductivity = 9; // W/(m·K)
  double specific_heat = 10; // J/(kg·K)
  
  // Visual properties
  Color color = 11;
  double roughness = 12;
  double metallic = 13;
  double transparency = 14;
  
  // Classification
  string ifc_material = 15;
  repeated string standards = 16; // e.g., "ASTM A36", "EN 10025"
}

enum MaterialCategory {
  MATERIAL_CATEGORY_UNSPECIFIED = 0;
  METAL = 1;
  CONCRETE = 2;
  WOOD = 3;
  MASONRY = 4;
  PLASTIC = 5;
  COMPOSITE = 6;
  GLASS = 7;
  INSULATION = 8;
  SOIL = 9;
  WATER = 10;
  OTHER = 99;
}

message Color {
  float r = 1;
  float g = 2;
  float b = 3;
  float a = 4;
}
```

---

## 10. 版本与审计系统

```protobuf
message ObjectVersion {
  string object_id = 1;
  string version_id = 2;
  string previous_version_id = 3;
  google.protobuf.Timestamp timestamp = 4;
  string author_id = 5;
  string change_description = 6;
  repeated string changed_fields = 7;
  ChangeType change_type = 8;
}

enum ChangeType {
  CHANGE_TYPE_UNSPECIFIED = 0;
  CREATED = 1;
  MODIFIED = 2;
  GEOMETRY_CHANGED = 3;
  PROPERTIES_CHANGED = 4;
  RELATIONSHIP_CHANGED = 5;
  DELETED = 6;
}
```

---

## 11. 分类映射

### 10.1 IFC 映射

核心建筑构件直接映射到 IFC 类。已对照 `crates/ifc/src/lib.rs`（`export_ifc_class()`）验证：

| OmniM 类 | IFC 类 | 导出状态 |
|-------------|-----------|---------|
| Site | IfcSite | ✅ 已导出/导入（`IFCSITE`） |
| Building | IfcBuilding | ✅ |
| BuildingStorey | IfcBuildingStorey | ✅ |
| Space | IfcSpace | ✅ |
| Wall | IfcWall | ✅（导入的 `IfcWallStandardCase` 再导出时归一化为 `IfcWall`） |
| Beam | IfcBeam | ✅ |
| Column | IfcColumn | ✅ |
| Slab | IfcSlab | ✅ |
| Door | IfcDoor | ✅ |
| Window | IfcWindow | ✅ |
| Stair | IfcStair | 🗓 计划中（当前导出为 `IfcBuildingElementProxy`） |
| Roof | IfcRoof | 🗓 计划中（当前为 `IfcBuildingElementProxy`） |

其余所有对象类型导出时回退为 `IfcBuildingElementProxy`；几何以 `IfcFacetedBrep` 导出。IFC 导入（`import_ifc()`）与导出（`export_ifc()`）均已在 `crates/ifc` 交付。

### 11.2 领域扩展

非建筑领域使用扩展分类：

- **Civil**：在适用处遵循 ISO 12006，并针对道路/桥梁进行扩展
- **Plant**：遵循 ISO 15926 和 P&ID 标准
- **Mechanical**：遵循 STEP AP242 和 ISO 10303

---

## 11. 索引与查询优化

查询性能的设计目标；当前 `ObjectStore`（`crates/model/src/store.rs`）为 ID→对象映射并支持父节点查找——下列物理索引 🗓 计划中：

为支持高性能查询：

### 主索引
- 对象 ID（主键）
- 对象类型
- 空间包围盒（R-tree）
- 包含层次结构

### 辅助索引
- 属性值索引
- 标签索引
- 材料索引
- IFC 类映射
- 关系索引

---

## 12. 待解决问题

1. 如何处理同一对象的不同细节层次（LOD）？
2. 是否应为导入的对象保留参数化历史？
3. 土木工程中线形和纵断面的最优表示是什么？
4. 如何处理复合材料和分层材料？
5. 约束关系是否应作为一等公民？

---

## 13. 构件目录（元件库 / PARAGON）

元件库是所有参数化三维构件定义的母库，属于 **Library**（session 级全局单例），不属于 Project。Project 通过 Series 间接访问 Library，从不直接持有 Library 引用。每个构件使用与常规参数化模型相同的草图→约束→特征管线建模，然后发布到目录中供复用。

### 13.1 核心概念

| 概念 | PDMS 对应 | 说明 |
|------|-----------|------|
| **ComponentDef** | SCOM（目录构件） | 带类型参数、特征树和连接点的参数化三维模型定义 |
| **Category** | CATE（类型） | 具有相似几何的构件逻辑分组（如"长半径弯头"、"IPE 梁"） |
| **Section** | SECT（分段） | 领域分组（如"管道"、"结构"、"设备"） |
| **Connector** | P-point（点集） | 带位置、方向、口径和类型码的连接点，用于兼容性匹配 |

**内建构件（In-Place Component）**（源自 Revit，`docs/research/revit_components.md §3.1`）：一个 Component 可以标记为 `in_place`（项目本地、不可重用、不可提升至目录）。与位于 Library 的 CatalogStore 中且可跨项目重用的标准 `ComponentDef` 不同，内建构件直接在项目的 Object Store 中定义，不能被系列引用，也不可通过 `catalog list` 获取。这一区别对应于 Revit 的"内建体量"与"可载入族"范式。`in_place` 标记位于项目中的 `Component` 消息上——它没有 `ComponentDef` 和 `CatalogRef`。

### 13.2 数据模型

```protobuf
message ComponentDef {
  string id = 1;                        // e.g., "IPE240", "Elbow-LR-ASME-B16.9"
  string name = 2;                      // 可读名称
  string category = 3;                  // 分类路径（如 "Structural/Steel/Beam"）
  string description = 4;

  // 参数
  repeated ParamDef params = 10;

  // 几何：参数化模型定义
  // 使用与项目对象相同的草图+约束+特征管线
  // 但尺寸是参数化的
  string feature_tree_id = 11;          // 引用特征树

  // 连接点
  repeated ConnectorDef connectors = 12;

  // 材质
  repeated string allowed_materials = 13;
  string default_material = 14;

  // 元数据
  string ifc_class = 20;
  string standard_reference = 21;       // e.g., "ASME B16.9", "EN 10025", "GB/T 11263"
  repeated string tags = 22;
  string domain = 23;                   // building / civil / plant / mechanical
  map<string, string> properties = 24;
  bool published = 25;                  // 发布标记（proto/omnim.proto，PUBLISH_COMPONENT 使用）
}

message ParamDef {
  string name = 1;
  ParamType type = 2;
  double default_value = 3;
  double min_value = 4;
  double max_value = 5;
  double step = 6;
  string unit = 7;
  string formula = 8;
  repeated string choices = 9;
  string description = 10;
}

enum ParamType {
  PARAM_TYPE_UNSPECIFIED = 0;
  LENGTH = 1;
  ANGLE = 2;
  INTEGER = 3;
  CHOICE = 4;
  BOOLEAN = 5;
  RATIO = 6;
}

message ConnectorDef {
  string id = 1;
  string name = 2;
  // 位置和方向用参数表达式表示
  string position_x_expr = 10;
  string position_y_expr = 11;
  string position_z_expr = 12;
  string direction_x_expr = 13;
  string direction_y_expr = 14;
  string direction_z_expr = 15;
  string bore_expr = 16;
  string connection_type_code = 17;     // e.g., "FAA", "BW"
  ConnectorRole role = 18;

  // 未来：跨平台编码映射
  // map<string, string> cross_reference = 19;  // { "civil3d_point_code": "ETW", "e3d_ppoint": "P1" }
}

> **跨平台编码映射**：未来将在 `ConnectorDef` 中增加 `cross_reference` 字段，并建立 `ConnectorMapping` 注册表，在 Civil3D Point Code、OmniM ConnectorType 和 E3D P-Point 标识符之间进行映射。详见 `docs/research/genrule_design_analysis.md §2.5`。

> **ComponentDef 参数交叉引用**（源自 SOLIDWORKS，`docs/research/solidworks_components.md §3.5`）：异型孔向导 ↔ Toolbox 表明一个 `ComponentDef` 参数可以被另一个 `ComponentDef` 的尺寸约束。启用 `ParamDef.formula` 引用同级别构件的参数——例如，螺栓孔直径 `ComponentDef` 的 `diameter` 参数由 `${bolt_def.size}` 引用驱动（螺栓尺寸 → 孔径）。这对应于异型孔向导孔尺寸与 Toolbox 紧固件尺寸联动的方式。

enum ConnectorRole {
  CONNECTOR_ROLE_UNSPECIFIED = 0;
  INLET = 1;
  OUTLET = 2;
  BIDIRECTIONAL = 3;
  MOUNT = 4;
  BRANCH = 5;
}
```

### 13.3 目录组织

```
ComponentLib
├── Piping (管件)
│   ├── Pipe-CS (碳钢管)
│   ├── Elbow-LR (长半径弯头)
│   ├── Tee-Equal (等径三通)
│   ├── Reducer-Conc (同心大小头)
│   ├── Flange-WN-CL150 (带颈对焊法兰)
│   ├── Valve-Gate (闸阀)
│   ├── Valve-Ball (球阀)
│   └── ...
├── Structural (结构)
│   ├── Steel-Beam-IPE (IPE 钢梁)
│   ├── Steel-Column-HEA (HEA 钢柱)
│   ├── Concrete-Beam-Rect (混凝土梁)
│   ├── Concrete-Column-Circ (混凝土圆柱)
│   └── ...
├── Building (建筑)
│   ├── Door-Single (单开门)
│   ├── Door-Sliding (推拉门)
│   ├── Window-Casement (平开窗)
│   └── ...
├── Equipment (设备)
│   ├── Pump-Centrifugal (离心泵)
│   ├── Tank-Vertical (立式储罐)
│   └── ...
└── Civil (市政)
    ├── Road-Asphalt (沥青路面)
    └── ...
```

---

## 14. 系列库（等级库 / SPECON）

系列库是**独立模块**——它定义选择器决策树，通过 ComponentDef ID 引用 Library 中的内容。一个系列（等级库）是一个**决策树**，它选择哪些构件可用，并对参数范围做出约束。系列可以存在于三个位置：(1) 项目私有的 SeriesStore（嵌入在 `.omnim` 文件中），(2) 外部 `.omnimseries` 文件（按路径引用），(3) 系统内置种子系列。

### 14.1 核心概念

| 概念 | PDMS 对应 | 说明 |
|------|-----------|------|
| **Series** | SPEC | 定义某项目或系统中可用构件的命名决策树 |
| **Selector** | SELE | 决策节点：问题→答案→子选择器或构件 |
| **SpecComponent** | SPCO | 叶子节点，引用 ComponentDef 并带参数约束 |
| **Question** | QUES | 决策标准（如"generic_type?"、"material?"） |
| **Answer** | ANSW | 匹配值（如"ELBO"、"FLAN"）或数值范围 |

### 14.2 数据模型

```protobuf
message Series {
  string id = 1;
  string name = 2;
  string description = 3;
  string domain = 4;
  string standard_reference = 5;
  Selector root_selector = 10;
  repeated string applicable_phases = 11;
  map<string, string> metadata = 12;
}

message Selector {
  string id = 1;
  string question = 2;
  repeated Answer answers = 3;
}

message Answer {
  string id = 1;
  string value = 2;
  bool is_numeric_range = 3;
  double range_min = 4;
  double range_max = 5;
  oneof outcome {
    Selector next_selector = 10;    // 继续细化
    SpecComponent component = 11;   // 叶子：选中此构件
  }
}

message SpecComponent {
  string id = 1;
  string component_def_id = 2;
  repeated ParamConstraint param_constraints = 10;
  string material = 11;
  string connection_type_code = 12;
  map<string, string> design_params = 13;
}

message ParamConstraint {
  string param_name = 1;
  double default_value = 2;
  double min_value = 3;
  double max_value = 4;
  double step = 5;
  string fixed_value = 6;
}

message CatalogRef {
  string component_def_id = 1;
  string series_id = 2;
  string selector_path = 3;
  map<string, double> param_values = 4;
}
```

### 14.3 选择算法

当设计师通过系列创建对象时：

1. 从 `root_selector` 开始
2. 根据设计上下文评估 `question`
3. 找到第一个匹配 `value` 的 `answer`
4. 如果匹配到 `Selector`，转到步骤 2（继续细化）
5. 如果匹配到 `SpecComponent`，实例化该构件
6. 应用 `param_constraints` 限制构件的参数
7. **系列门 (Series Gate)**：确认目标 `ComponentDef` 可以从项目已加载的至少一个系列中到达；否则拒绝
8. 从 Library 的 `CatalogStore` 查找 `ComponentDef`
9. 以约束后的参数实例化 ComponentDef

选择引擎是上下文感知的：第一个问题通常是 "generic_type?"（什么类型的构件？），系统根据设计者要放置的内容（如弯头、法兰、阀门）提供答案。

**系列门行为**：
- 已加载系列 + 构件已批准 → 允许
- 已加载系列 + 构件不在任何系列中 → 拒绝
- 未加载系列 → 开放模式（所有目录构件可用）

---

## 15. 各领域系列树设计

### 15.1 建筑：现浇混凝土结构 (C30)

```
Selector: "element_type?"
├── wall → Selector: "function?"
│   ├── shear → t=200..400
│   ├── retaining → t=300..800
│   ├── basement → t=250..600
│   └── infill → t=150..250
├── column → Selector: "shape?"
│   ├── rectangular → w,d=200..1000
│   └── circular → D=300..1200
├── beam → Selector: "function?"
│   ├── main → w=250..800, d=400..1200
│   └── secondary → w=200..600, d=300..800
├── slab → Selector: "system?"
│   ├── one_way → t=100..250
│   ├── two_way → t=120..300
│   ├── flat → t=150..350
│   └── waffle → t=300..600
└── foundation → Selector: "type?"
    ├── strip → w=500..2000
    ├── pad → size=800..3000
    └── raft → t=300..1000
```

### 15.2 建筑：钢结构 (Q355)

```
Selector: "element_type?"
├── column → Selector: "section?"
│   ├── HEA → HEA100..HEA900
│   ├── HEB → HEB100..HEB900
│   ├── CHS → D=100..500
│   └── SHS → size=100..400
├── beam → Selector: "section?"
│   ├── IPE → IPE80..IPE600
│   ├── HEA → HEA100..HEA900
│   └── UB → UB127..UB914
└── bracing → Selector: "section?"
    ├── angle → L50..L200
    └── CHS → D=50..200
```

### 15.3 工厂：管道等级 (碳钢150磅级)

以 PDMS SPECON 风格定义的管道等级系列：

```
Selector: "generic_type?"
├── PIPE → SpecComponent: Pipe-CS
│   constraints: DN=15..600, schedule=40,80,STD,XS, material=A106-B
├── ELBO → Selector: "radius?"
│   ├── LR → Elbow-LR-CS, R=1.5D, DN=15..600
│   ├── SR → Elbow-SR-CS, R=1.0D, DN=15..600
│   └── 3D → Elbow-3D-CS, R=3.0D, DN=50..400
├── TEE → Selector: "type?"
│   ├── equal → DN=15..600
│   └── reducing → run=15..600, branch=15..run
├── FLAN → Selector: "face?"
│   ├── RF → Flange-WN-CL150-RF, DN=15..600
│   ├── FF → Flange-WN-CL150-FF, DN=15..600
│   └── RTJ → Flange-WN-CL150-RTJ, DN=100..600
├── GASK → Selector: "type?"
│   ├── spiral → Gasket-SpiralWound
│   ├── ring → Gasket-Ring
│   └── sheet → Gasket-Sheet
├── BOLT → BoltSet, material=A193-B7/A194-2H
├── VALV → Selector: "valve_type?"
│   ├── gate → Valve-Gate, DN=15..600
│   ├── globe → Valve-Globe, DN=15..300
│   ├── ball → Valve-Ball, DN=15..400
│   ├── check → Valve-Check, DN=15..400
│   └── butterfly → Valve-Butterfly, DN=50..600
├── REDU → Selector: "type?"
│   ├── concentric → DN=15..600
│   └── eccentric → DN=15..600
└── CAP → Cap-CS, DN=15..600
```

### 15.4 道路：公路 (GB JTG B01)

```
Selector: "road_class?"
├── expressway → Selector: "design_speed?"
│   ├── 120 → lanes=2..4, lane_width=3.75
│   ├── 100 → lanes=2..4, lane_width=3.75
│   └── 80 → lanes=2..4, lane_width=3.75
├── arterial → Selector: "design_speed?"
│   ├── 60 → lanes=2..6, lane_width=3.5
│   └── 40 → lanes=2..4, lane_width=3.5
├── collector → lanes=2, lane_width=3.25..3.5
└── local → lanes=2, lane_width=3.0..3.25
```

---

## 16. 行政分区（Administrative Division）

行政分区建模项目所在的地理/行政层级。这与空间坐标系不同——它提供的是**管辖背景**，用于审批、合规、土地用途和报告。

### 16.1 层级结构

| 层级 | 标准 | 示例 |
|------|------|------|
| **世界** | — | 地球 |
| **国家/地区** | ISO 3166-1 | CN、US、GB、DE、JP |
| **省/州/邦** | ISO 3166-2、GB/T 2260（省级） | 广东省、California、Bayern |
| **地级市/地区** | GB/T 2260（地级）、各地等效体系 | 深圳市、大阪府、Los Angeles County |
| **县/区/县级市** | GB/T 2260（县级） | 南山区、曼哈顿区、新宿区 |
| **乡/镇/街道** | GB/T 2260（乡级） | 粤海街道、银座 |
| **社区/村** | 属地管理 | 华侨城社区、Soho |
| **街区** | 城市规划 | C12 地块、Lot 47 |
| **地块** | 不动产登记 | G2024-0015 地块 |

### 16.2 数据模型

```protobuf
message AdministrativeDivision {
  string id = 1;                         // 如 "CN/44/03/0305"
  string name = 2;                       // 如 "Nanshan District"
  string local_name = 3;                 // 如 "南山区"（当地文字）
  DivisionLevel level = 4;               // WORLD, COUNTRY, PROVINCE, PREFECTURE, COUNTY, TOWNSHIP, COMMUNITY, BLOCK, PARCEL
  string parent_id = 5;                  // 上级行政区划 ID
  optional string code_iso_3166_1 = 6;   // "CN"
  optional string code_iso_3166_2 = 7;   // "CN-GD"
  optional string code_gb_t2260 = 8;     // "440305"（GB/T 2260 县级代码）
  optional GeoBoundary boundary = 9;     // 地理边界（Polygon/MultiPolygon）
  map<string, string> attributes = 10;   // 人口、面积、时区等
}

enum DivisionLevel {
  WORLD = 0;
  COUNTRY = 1;
  PROVINCE = 2;
  PREFECTURE = 3;
  COUNTY = 4;
  TOWNSHIP = 5;
  COMMUNITY = 6;
  BLOCK = 7;
  PARCEL = 8;
}

> **设计规范与 proto 的差异**：上方消息 schema 为目标模型。`proto/omnim.proto` 当前定义 `AdministrativeDivision { id, level, name, native_name, code, parent_id, boundary, properties }`，枚举为 `AdministrativeLevel`（UNSPECIFIED = 0，WORLD = 1 … PARCEL = 9，即 1 起始的层级编号），并以 `GeoBoundary { boundary_type, coordinates: repeated Point3D, crs }` 承载边界而非 WKT 字节形式。AD 存储（`crates/model/src/administrative.rs`，`/api/administrative*`）已实现。将文档 schema 与 proto 对齐 🗓 计划中。

message GeoBoundary {
  bytes wkt_polygon = 1;                // Well-Known Text 多边形表示
  string srid = 2;                      // "EPSG:4326"
}
```

### 16.3 用途

- **项目**关联到一个或多个 AdministrativeDivision（如场地跨越南山区+福田区）
- 行政分区提供**适用标准**（建筑规范、环保法规、消防规范）
- **报表**按行政边界汇总对象
- **地块**是最细粒度，映射到不动产登记

### 16.4 范围：仅参考数据

行政分区数据**不在 AMIR 和 Omcom 的范围内**。这是来自政府公报的不可变参考数据：
- **不存在**用于创建、编辑或删除 AD 记录的 Omcom 动词
- **不存在**针对行政分区的 AMIR 意图
- AD 数据由管理员通过批量导入（来自国家统计局或民政部的 CSV/GeoJSON）加载到系统中
- 项目对象通过 `ad_id` 引用 AD，作为合规和报告的只读上下文

---

## 17. 项目分解结构（Project Breakdown Structure）

项目分解结构（PBS）定义项目工作范围的层次化分解。与行政分区（地理/政治）或空间层次（物理放置）不同，PBS 是一种**管理和工程分解**，遵循特定领域的惯例。

### 范围：仅项目设置

PBS 通过**项目初始化和配置**管理，而非通过建模命令：
- **不存在**针对 PBS 节点的 `create` / `delete` / `modify` Omcom 动词
- **不存在**生成 PBS 结构的 AMIR 意图
- PBS 由项目团队在项目设置期间定义（通过配置文件或项目向导）
- 对象通过声明式规则**自动或手动分配**到 PBS 节点（例如"L2 楼层中的所有梁 → PBS: Tower/L2/Structural-Zone"）
- PBS 树在建模期间只读，不被 Omcom 命令改变

### 17.1 支持的分解惯例

| 惯例 | 领域 | 标准 |
|------|------|------|
| **项目群→项目→子项目→装置→系统→构件** | 通用工程 | ISO 19650、PMI PMBOK |
| **项目→建筑→楼层→分区→构件** | 建筑 | GB/T 51301-2018、UniClass |
| **项目→区域→工段→系统→组件** | 工厂 | ISO 81346、PDMS |
| **项目→标段→区段→构件** | 土木（线性） | GB/T 51253-2017 |
| **项目→系统→子系统→装配体→零件** | 机械 | STEP AP242 |

### 17.2 数据模型

```protobuf
message ProjectBreakdownNode {
  string id = 1;                         // 如 "P1/BLDG-A/L2/Z3/BEAM-045"
  string name = 2;                       // "主楼—2层—3区"
  string code = 3;                       // "A2.3.045"（领域简写）
  BreakdownLevel level = 4;              // PROGRAM, PROJECT, SUBPROJECT, UNIT, BUILDING, STOREY, ZONE, SECTION, SEGMENT, SYSTEM, ELEMENT, PART
  string parent_id = 5;                  // 上级 PBS 节点
  string project_id = 6;                 // 根项目引用
  BreakdownConvention convention = 7;    // GENERAL, BUILDING, PLANT, CIVIL, MECHANICAL
  repeated string tags = 8;              // 功能标签（如 "structural"、"fire-zone-A"）
  repeated string object_ids = 9;        // 属于本节点的 OmniObject ID
  map<string, string> attributes = 10;   // 阶段、承包商、预算代码等
}

> **设计规范与 proto 的差异**：`proto/omnim.proto` 当前定义 `BreakdownNode { id, level, name, code, parent_id, convention, related_object_ids, administrative_division_ids, properties }`，枚举为 `BreakdownLevel`（UNSPECIFIED = 0，PROGRAM = 1 … PART = 10）——没有 `SECTION`/`SEGMENT` 层级，也没有 `tags`/`attributes`/`project_id` 字段。分解存储（`crates/model/src/breakdown.rs`，`/api/breakdown*`）已实现；补齐缺失层级/字段 🗓 计划中。

enum BreakdownLevel {
  PROGRAM = 0;
  PROJECT = 1;
  SUBPROJECT = 2;
  UNIT = 3;
  BUILDING = 4;
  STOREY = 5;
  ZONE = 6;
  SECTION = 7;
  SEGMENT = 8;
  SYSTEM = 9;
  ELEMENT = 10;
  PART = 11;
}

enum BreakdownConvention {
  GENERAL = 0;
  BUILDING = 1;
  PLANT = 2;
  CIVIL = 3;
  MECHANICAL = 4;
}
```

### 17.3 领域示例

**建筑：**
```
上海中心大厦（PROJECT）
├── 塔楼（BUILDING）
│   ├── L1-L10（STOREY 范围）
│   │   ├── 核心筒区（ZONE）
│   │   │   ├── 核心筒墙-北（ELEMENT）
│   │   │   └── 核心筒墙-南（ELEMENT）
│   │   └── 外围区（ZONE）
│   │       ├── 梁-G1-01（ELEMENT）
│   │       └── 柱-C1-01（ELEMENT）
│   └── L11-L20（STOREY 范围）
```

**工厂：**
```
炼油厂扩建（PROJECT）
├── 常减压装置（UNIT）
│   ├── 常压塔系统（SYSTEM）
│   │   ├── 塔-C103（ELEMENT）
│   │   └── 管道-P-101..150（ELEMENT 组）
│   └── 预热系统（SYSTEM）
│       ├── 换热器-E201（ELEMENT）
│       └── 管道-P-201..230（ELEMENT 组）
└── 减压装置（UNIT）
```

**土木（线性）：**
```
高铁三号线（PROJECT）
├── A标段（SECTION: km 0–45）
│   ├── A1区段（SEGMENT: km 0–12）
│   │   ├── 桥-B1（ELEMENT）
│   │   └── 桥-B2（ELEMENT）
│   └── A2区段（SEGMENT: km 12–28）
│       ├── 隧道-T1（ELEMENT）
│       └── 路堤-E1（ELEMENT）
└── B标段（SECTION: km 45–98）
```

---

## 18. 行政分区、项目分解结构与空间层次的关系

这三种层次服务于不同目的，在数据模型中并存：

| 层次 | 划分依据 | 示例 | 粒度 |
|------|----------|------|------|
| **行政分区（AD）** | 管辖权/政治 | 南山区、深圳市 | 地块→世界 |
| **项目分解（PBS）** | 管理/工程 | 塔楼→楼层→分区→梁 | 零件→项目群 |
| **空间（包含）** | 物理放置 | 建筑→墙→开洞 | 子对象→场景 |

### 18.1 交叉引用

```protobuf
message OmniObject {
  // ... 已有字段 ...
  optional string ad_id = 21;            // 所属行政分区（地块或更高级别）
  repeated string pbs_ids = 22;          // 所属 PBS 节点（通常为 1 个）
}

message Project {
  string id = 1;
  string name = 2;
  repeated string ad_ids = 3;            // 项目跨越的行政分区
  string root_pbs_node_id = 4;          // 项目分解树根节点
  ProjectCoordinateSystem coord_system = 5;  // 坐标系配置
  // ... 其他字段 ...
}
```

> **设计规范与已实现 `Project`（`crates/model/src/project.rs`）的差异**：已实现结构体持有 `name`、`objects`（ObjectStore）、`materials`、`lifecycle`、`relations`（Containment/Connection）、`administrative`、`breakdown`、`sketches`、`series`（SeriesStore）、`external_series_refs`、`measurements`——AD 关联与 PBS 树保存在专用存储中，而非 `ad_ids`/`root_pbs_node_id` 字段，且尚无 `coord_system` 字段（🗓 计划中，§19）。proto 中无 `Project` 消息；项目为 serde JSON。

### 18.2 关键规则

1. **一个项目属于 1+ 个行政分区**（其场地范围）
2. **每个 OmniObject 属于恰好 1 个 PBS 叶子节点**（覆盖它的最细 PBS 层级）
3. **对象的空间父级可能与 PBS 父级不同**（如梁空间上在楼层内，但 PBS 父级可能是结构分区）
4. **AD 是不可变参考数据**（来自政府公报；无 Omcom/AMIR 操作）
5. **PBS 是项目特定设置**（在项目初始化时定义；无建模命令改变它）
6. **空间包含是物理的**（由几何放置决定，可自动检测）
7. **AMIR 和 Omcom 仅操作工程模型内容**（对象、草图、特征、参数、目录构件）——从不操作 AD 或 PBS 结构

### 18.3 数字化交付合规

这一三层次模型满足以下标准的结构要求：

- **ISO 19650**：信息容器分解（PBS 映射到"容器"层次）
- **GB/T 51301-2018**：模型单元分解层级（项目级→功能级→构件级→零件级）
- **GB/T 51253-2017**：建筑全生命期 BIM 应用
- **中国数字化交付标准**：项目元数据中的行政分区代码 + 用于模型组织的层次化 PBS

---

## 19. 坐标系模型（Coordinate System Model）

> **设计规范（🗓 计划中）**：本节的 CRS 消息为目标模型——`CoordinateReferenceSystem`、`ProjectCoordinateSystem`、`LatLonHeight`、`HeightSystem` 均尚未存在于 `proto/omnim.proto` 或 `crates/model`，`OmniObject.crs_id` 也未实现（§3）。以下规则描述预期行为；CRS 配置仍是项目设置层面的设计工作。

OmniM 支持**双层坐标系**——每个项目有且仅有一个坐标系：

| 层级 | 用途 | 类型 |
|------|------|------|
| **Global CRS（地理坐标系）** | 场地的真实地理位置 | `GEOGRAPHIC` 或 `PROJECTED`（EPSG） |
| **Project CRS（施工坐标系）** | 所有建模操作使用的局部工程坐标 | `ENGINEERING`（自定义原点+旋转） |

所有建模操作在 **Project CRS** 中完成。跨项目查看或导入/导出时，坐标值实时换算到 Global CRS。

一个项目有**且仅有一个坐标系**——子项目或标段共享同一个原点和 CRS。

### 19.1 数据模型

```protobuf
// 坐标参考系统
message CoordinateReferenceSystem {
  string id = 1;                         // "EPSG:4490" 或自定义 UUID
  string name = 2;                       // "CGCS2000" / "WGS84" / "Site-Origin"
  CRSType type = 3;                      // GEOGRAPHIC, PROJECTED, ENGINEERING, VERTICAL
  optional int32 epsg_code = 4;          // 4490 (CGCS2000), 4326 (WGS84) 等
  optional string definition = 5;        // 自定义 CRS 的 PROJ 字符串或 WKT
}

enum CRSType {
  GEOGRAPHIC = 0;                        // 经纬度（如 WGS84, CGCS2000）
  PROJECTED = 1;                         // 投影坐标系（如 UTM, 高斯-克吕格）
  ENGINEERING = 2;                       // 局部工程/施工坐标
  VERTICAL = 3;                          // 高程系统/垂直基准
}

// 项目坐标系
message ProjectCoordinateSystem {
  // 地理（全局）参考
  CoordinateReferenceSystem global_crs = 1;     // 如 CGCS2000 (EPSG:4490)

  // 工程（局部）CRS
  CoordinateReferenceSystem project_crs = 2;    // ENGINEERING 类型，自定义

  // 变换：Project CRS → Global CRS
  // 项目原点 (0,0,0) 的地理位置
  LatLonHeight origin_geo = 3;
  double rotation = 4;                          // 顺时针偏离真北的角度（度）
  double scale_factor = 5;                      // 通常为 1.0

  // 高程系统
  HeightSystem height_system = 6;
  optional CoordinateReferenceSystem vertical_crs = 7;  // 如 "EPSG:5730"（1985 国高程基准）
}

// 地理位置
message LatLonHeight {
  double latitude = 1;                   // 纬度（度，WGS84）
  double longitude = 2;                  // 经度（度，WGS84）
  double height = 3;                     // 高度（米，项目高程系统）
}

// 高程系统
enum HeightSystem {
  HEIGHT_UNSPECIFIED = 0;
  NATIONAL_1985 = 1;                     // 1985 国家高程基准（中国）
  ELLIPSOIDAL = 2;                       // 椭球高 (h)
  MEAN_SEA_LEVEL = 3;                    // 平均海平面
  LOCAL_DATUM = 4;                       // 地方/自定义高程基准
}
```

### 19.2 Project CRS 定义步骤

1. **选择 Global CRS**——如 CGCS2000 (EPSG:4490) 或 WGS84 (EPSG:4326)
2. **选择高程系统**——如 1985 国家高程基准
3. **设置项目原点**——现场 (0,0,0) 点的测绘 GPS 坐标
4. **设置旋转角**——项目 X 轴偏离真北的方向角（用户输入或测绘文件导入）
5. **比例因子**——通常为 1.0；高精度测量时可调整

原点和旋转角可以通过以下方式设置：
- **用户手动输入**
- **从测绘文件导入**（如 .csv、.txt 测站坐标）
- **从 GIS 地图拾取**（未来功能）

### 19.3 坐标变换

```
┌─────────────────────────────────────────────────────────┐
│                    实时显示换算                           │
│                                                          │
│  用户输入/查看的坐标在 Project CRS 中                     │
│                                                          │
│  ┌──────────┐    ┌───────────┐    ┌──────────────────┐ │
│  │ Project  │───→│ Coordinate │───→│ Global CRS       │ │
│  │ CRS      │    │ Service    │    │ (用于展示/导出/  │ │
│  │ (局部)   │←───│ (PROJ)     │←───│  报告)           │ │
│  └──────────┘    └───────────┘    └──────────────────┘ │
│                                                          │
│  显示切换（如 CGCS2000 ↔ WGS84）：                       │
│  Project CRS → 源 Global CRS → 目标 Global CRS         │
└─────────────────────────────────────────────────────────┘
```

**关键规则：**
- 所有对象坐标（`Transform`）以 **Project CRS** 存储
- Coordinate Service 仅换算**原点坐标和方向**用于显示
- 不做逐顶点的几何精度换算——只转换显示的原点/方向，使用户能关联到地理坐标
- 在 Global CRS 变体间切换（如 CGCS2000 ↔ WGS84）时，通过 PROJ 的七参数变换重新计算显示的原点坐标
- 导出到 GIS 格式时使用完整的 PROJ 管线

### 19.4 内置 CRS 注册表

| 名称 | EPSG | 类型 | 说明 |
|------|------|------|------|
| WGS 84 | 4326 | 地理坐标 | 全球标准（GPS） |
| CGCS2000 | 4490 | 地理坐标 | 2000国家大地坐标系（2018年起中国强制） |
| 北京1954 | 4214 | 地理坐标 | 旧版（已弃用） |
| 西安1980 | 4610 | 地理坐标 | 旧版（已弃用） |
| Web Mercator | 3857 | 投影坐标 | 网络地图 |
| UTM N区 | 326xx | 投影坐标 | xx = 区号 |
| 1985国家高程基准 | 5730 | 高程 | 中国高程基准 |

用户也可以通过 PROJ 字符串或 WKT 定义**自定义 CRS**。

### 19.5 Omcom / AMIR 范围

坐标系配置属于**项目设置**，非建模操作：
- **不存在**用于 CRS 创建或换算的 Omcom 动词
- **不存在**针对坐标系的 AMIR 意图
- CRS 在项目初始化时设置（配置文件或项目向导）
- 可提供一个 `project set-coord` Omcom 命令用于**初始设置**，但非建模工作流的一部分
