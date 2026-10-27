# 跨平台调研整合分析 — 选择与实施策略

> 基于 8 份平台调研文档的综合分析（含场地/地形专项调研）
> 2026-08-07

---

## 1. 综述

### 1.1 已调研平台与专项

| 平台/专项 | 领域 | 文档 | 核心范式 |
|-----------|------|------|---------|
| Revit 2027 API | 建筑 | `revit_components.md` | 族参数化 + GenericForm 几何 |
| Civil 3D 2026 | 市政 | `civil3d_components.md` | 走廊模板 + 横断面 + 点编码 |
| AVEVA E3D 2.1 | 工厂 | `aveva_e3d_primitives.md` | 基本体拼装 + 目录规范 + P-Point |
| SOLIDWORKS 2026 | 机械 | `solidworks_components.md` | 特征树 + 设计表 + 配合 |
| Bentley ORD/OBD/OTD | 市政(桥梁/隧道) | `bentley_openroads_components.md` | 走廊模板 + 变截面 + 地形目标 |
| Tekla Structures 2025/2026 | 结构 | `tekla_components.md` | Part + Component 双层建模 |
| AVEVA Instrumentation/E3D Electrical | 工厂(仪表/电气) | `e3d_instrumentation_components.md` | 逻辑-物理双视图 + 仪表装配 |
| **场地/地形专项** | 跨六平台 | `site_terrain_components.md` | 场地空间层级 + TIN 地形 + 土方平整 |

### 1.2 核心发现

8 份调研文档虽覆盖建筑、市政、工厂、机械四大领域，但底层几何范式高度收敛。OmniM 的 5 类 GenRule 已验证可覆盖全部平台的所有内置构件类型。关键差异在于**各平台的建模哲学与组织方式**，而非底层几何。

**场地/地形专项补充发现**（`site_terrain_components.md`）：
- Civil 3D 和 Bentley 是场地/地形建模的标杆（评分 19/20），实现了完整的 TIN 地形、土方计算、平整系统和地形-结构交互
- Revit 的场地空间层级定位最清晰（IfcSite → IfcBuilding → IfcBuildingStorey → IfcSpace），但地形建模能力有限
- E3D 验证了 Site 作为逻辑/组织容器的另一种用法（工厂布局）
- Tekla 和 SolidWorks 无场地概念，验证地形建模的非普遍性
- TerrainModel 应作为独立 ObjectType（非 GenRule），与 5 类 GenRule 并列

---

## 2. 关键选择点

### 2.1 选择一：GenRule 分类的粒度

**问题**：当前 5 类 GenRule 是否足够？是否需要细分？

**各平台启示**：

| 平台 | 隐含分类粒度 | 对 OmniM 的启示 |
|------|-------------|----------------|
| Revit | 3 类（面/线/体）+ 布尔 | 5 类足够 |
| Civil 3D | 3 类（曲面/走廊/部件）+ 规则 | 5 类足够 |
| E3D | 2 类（基本体/装配）+ 布尔 + 目录 | 5 类足够 |
| SOLIDWORKS | 1 类（特征树）+ 配合 | Primitive 需扩展 |
| Bentley | 6 类（走廊/线性/曲面/Cell/参数化/布尔） | 5 类 + 地形 |
| Tekla | 5 类（扫掠/板/布尔/组件/装配） | 5 类 + 组件脚本 |
| Instrumentation | 4 类（装配/线/层/布尔）+ 逻辑对象 | 5 类 + 逻辑层 |

**结论**：**保留 5 类 GenRule 不变**，但：
- 新增 `TerrainModel` 作为独立几何类型（非 GenRule），用于 TIN 地形
- 新增 `LogicalObject` 作为独立对象类型（无几何），用于 Loop/Tag

### 2.2 选择二：Assembly 的建模方式

**问题**：Assembly 应该是一个"列出子对象"的容器，还是"过程化脚本"？

| 方案 | 采用者 | 优势 | 劣势 |
|------|--------|------|------|
| **声明式**（列出子件 + Transform） | E3D EQUI + SUBE | 简单、序列化友好 | 无法表达条件逻辑 |
| **过程化**（输入 + Run() 脚本） | Tekla ConnectionBase | 灵活、可表达复杂连接规则 | 调试困难、序列化复杂 |
| **混合**（模板 + 条件参数） | Bentley Civil Cell | 兼顾灵活性与可复用性 | 设计复杂度高 |

**建议**：**采用混合方案**。Assembly 的 `FeatureTreeTemplate` 应支持：
1. 子对象列表（声明式，支持嵌套）
2. 条件参数（如"是否生成加劲肋"=true/false → 控制子对象是否生成）
3. 表达式驱动的子对象位置（引用外部对象，如桥轴线、支承线）

### 2.3 选择三：LinearComponent 的变截面策略

**问题**：变截面应该用"段内连续插值"还是"段间跳跃"？

| 方案 | 采用者 | 优势 | 劣势 |
|------|--------|------|------|
| **段内连续插值**（loft） | 当前 OmniM 设计 | 光滑过渡 | 复杂、可能不稳定 |
| **段间跳跃**（多等截面段） | OTD Tunnel Unit | 稳健、工程常用 | 不光滑 |
| **混合**（段内插值 + 段间跳跃） | OBM Beam Rows | 灵活 | 两种模式都需要支持 |

**建议**：**保留当前段内插值设计**，但增加：
1. 插值方法扩展：从 current(Linear) → 6 种（Linear/Parabolic±/Circular±，参考 Bentley）
2. 多段支持：`LinearComponentInput` 增加 `segments: Vec<CrossSectionSegment>`，每段可独立定义起止截面 + 插值方法
3. 段间跳跃：当某段起止截面相同时，不插值直接扫掠

### 2.4 选择四：型材/截面库的管理方式

**问题**：标准截面（IPE/HEA/角钢/槽钢等）应该硬编码还是目录驱动？

| 方案 | 采用者 | 优势 | 劣势 |
|------|--------|------|------|
| **硬编码**（代码中定义） | 简单 | 灵活 | 扩展困难 |
| **目录驱动**（Catalog ComponentDef） | E3D/Tekla | 可扩展、按标准组织 | 需要初始数据 |

**建议**：**目录驱动**。在 Catalog 中建立 `Profiles/` 分类，按标准组织（AISC/ISO/DIN/GB/EN）。每个 Profile 是一个 ComponentDef，其 FeatureTreeTemplate 生成 2D 截面多边形，由 LinearComponent 沿路径扫掠。

### 2.5 选择五：BooleanSubtract 的输入模式

**问题**：是否支持半空间平面切割？

| 方案 | 采用者 | 优势 | 劣势 |
|------|--------|------|------|
| **仅实体切割** | 当前 OmniM | 简单 | 端部拟合效率低 |
| **实体 + 平面切割** | Tekla (Fitting/CutPlane) | 高效、灵活 | 需新增 Plane 输入类型 |

**建议**：**增加平面切割模式**。`BooleanSubtract` 新增 `PlaneCut` 变体，输入 Plane + 方向，效率高于实体布尔。参考 Tekla `Fitting`/`CutPlane` 与 Revit `CutWithHalfSpace`。

### 2.6 选择六：零件/组件/装配体命名

**问题**：Beam/Column/Brace 是否应合并为一个 ObjectType？

| 方案 | 采用者 | 优势 | 劣势 |
|------|--------|------|------|
| **分开**（BEAM/COLUMN/BRACE） | 当前 OmniM | 语义明确 | 重复代码 |
| **合并**（统一 Beam，用途标签区分） | Tekla | 简洁、API 统一 | 查询需过滤标签 |

**建议**：**保留分开但统一 GenRule**。`ObjectType::Beam`/`Column`/`Brace` 均映射到 `LinearComponent`，差异仅通过 Catalog 分类路径和用途标签区分。语义上保留区分便于用户查询，几何上统一处理。

### 2.7 选择七：逻辑对象层

**问题**：是否需要独立的逻辑对象（无几何）？

**各平台启示**：
- AVEVA Instrumentation：Loop/Tag 是核心逻辑实体，无几何
- Revit：Phases、Design Options 是逻辑概念
- Civil 3D：Pipe Network Rules 是逻辑规则
- SOLIDWORKS：Design Table 参数是逻辑属性

**建议**：**新增 LogicalObject 层**。与物理 Object 并列，无几何属性，但持有：
- Tag/Loop/Line Number 等标识
- 成员物理对象引用列表
- 关联规则（如 Pipe Network 的最小覆盖规则）

### 2.8 选择八：连接/约束系统

**问题**：SOLIDWORKS 的 Mates（21 类）和 Tekla 的 Connections（过程化组件）如何整合？

**分析**：
- SOLIDWORKS Mates = 运动学约束（Coincident/Concentric/Parallel/Tangent...）
- Tekla Connections = 钢结构节点生成规则（EndPlate/BasePlate/ClipAngle...）
- E3D P-Point = 连接点匹配（Head/Tail 配对）
- 当前 OmniM Connector = 连接点类型码 + 角色

**建议**：**分层设计**：
1. **Connector**（当前已有）：连接点位置/方向/口径/类型码，用于 Catalog 构件间的配对
2. **Constraint**（新增）：运动学约束（Coincident/Concentric/Distance/Angle...），用于 Assembly 子件间的定位
3. **ConnectionScript**（新增）：过程化节点生成规则（参考 Tekla ConnectionBase），在 Assembly 中按输入参数生成子零件

---

## 3. 整合优先级

### P0 — 立即实施（当前架构已支持）

| 项目 | 来源 | 工作量 | 说明 |
|------|------|--------|------|
| VariationRule 扩展 6 种插值 | Bentley §8.1, LinearComponent | 小 | 枚举扩展 + 插值函数 |
| BooleanSubtract 平面切割模式 | Tekla §8.3, BooleanSubtract | 小 | 新增 PlaneCut 输入 |
| CableTray GenRule 细化 | E3D Instr. §3.1, GenRule | 小 | 直段=LayerExtrusion，弯头=LinearComponent |

### P1 — 高优先级（需设计，方向明确）

| 项目 | 来源 | 工作量 | 说明 |
|------|------|--------|------|
| TerrainModel 独立类型 | Bentley §6.1, ObjectType | 中 | 新增 ObjectType + 几何存储 |
| LogicalObject 层 | E3D Instr. §4, 架构 | 中 | 新增 Tag/Loop 逻辑对象 |
| 截面库（Profiles/） | Tekla §2.4, Catalog | 中 | 20+ 标准截面 ComponentDef |
| ComponentSequenceItem | Bentley §8.1, LinearComponent | 中 | 元件序列模式 |
| ExternalTarget 约束 | Bentley §8.1, LinearComponent | 中 | 截面端点求交到外部曲面 |

### P2 — 中优先级（需原型设计）

| 项目 | 来源 | 工作量 | 说明 |
|------|------|--------|------|
| ConnectionScript 机制 | Tekla §3, Assembly | 大 | 过程化组件脚本 |
| 多段变截面 | Bentley §8.1, LinearComponent | 中 | segments: Vec<CrossSectionSegment> |
| Assembly 表达式驱动位置 | Bentley §8.3, Assembly | 中 | 子件位置引用外部对象 |
| Rebar 专用模板 | Tekla §6, Catalog | 中 | Rebar/ 分类 + 弯钩/半径规则 |

### P3 — 低优先级（需架构讨论）

| 项目 | 来源 | 工作量 | 说明 |
|------|------|--------|------|
| Constraint 系统 | SOLIDWORKS §6, 关系 | 大 | 21 类运动学约束 |
| Feature::Chamfer/Fillet | SOLIDWORKS §3, Feature Engine | 中 | 修饰特征 |
| Feature::Pattern | SOLIDWORKS §3, Feature Engine | 中 | 阵列特征 |
| MBD/PMI 属性集 | SOLIDWORKS §8, 属性 | 中 | 面向制造属性 |
| 双向同步机制 | E3D Instr. §7, 架构 | 大 | 逻辑-物理双视图同步 |
| 施工阶段时序属性 | Bentley §8.1, 生命周期 | 低 | 4D 施工模拟 |

---

## 4. 架构调整建议

### 4.1 当前架构评估

```
当前架构：
Object → Component → Feature → ParametricOperation
  │          │           │            │
  │   GenRuleHint    FeatureType   Box/Cylinder...
  │   (分类层)        (Extrude/
  │                   Revolve/...)
  │
  └── GenRule (已弃用)
```

### 4.2 建议调整后的架构

```
新增逻辑层：
LogicalObject (Tag/Loop/LineNumber)
  └── 持有物理对象引用列表

物理层扩充：
Object → Component → Feature → ParametricOperation
  │          │           │            │
  │   GenRuleHint    FeatureType   Box/Cylinder/Cone/
  │   (5类不变)       (Extrude/     Sphere/Extrusion/
  │                    Revolve/     Revolve/Loft/Sweep/
  │                    Sweep/       Boolean/
  │                    Loft/        Transform/
  │                    Fillet/      (新增: Torus/Dish)
  │                    Chamfer/
  │                    Pattern/)
  │
  └── TerrainModel (独立几何类型，TIN)

Constraint 系统 (新增，与 Connector 并列)：
RelationStore
  ├── Connection (连接点匹配，已有)
  ├── Constraint (运动学约束，新增)
  │   ├── Coincident / Concentric / Distance / Angle / Parallel / Perpendicular / Tangent
  │   └── Gear / RackPinion / Screw / Hinge / UniversalJoint
  └── ConnectionScript (过程化节点，新增)
      ├── Input: Primary + Secondaries + Positions
      └── Output: [子零件列表]

Assembly 增强：
AssemblyGen
  ├── children: [SubComponent]  (声明式子件列表，已有)
  ├── conditions: {name: bool}  (条件参数，新增)
  └── script: ConnectionScript  (过程化脚本，新增)

LinearComponent 增强：
LinearComponentInput
  ├── alignment: Vec<Point3D>  (路径，已有)
  ├── cross_sections: Vec<CrossSectionAtStation>  (截面，已有)
  ├── segments: Vec<CrossSectionSegment>  (多段变截面，新增)
  │   ├── start_station, end_station
  │   ├── start_section, end_section
  │   └── interpolation: Linear/ParabolicUp/ParabolicDown/CircularUp/CircularDown
  ├── component_sequence: Vec<ComponentSequenceItem>  (元件序列，新增)
  │   ├── component_def_id
  │   ├── position_along_path
  │   └── params: {name: value}
  └── external_targets: Vec<ExternalTarget>  (外部约束，新增)
      ├── target_type: Terrain/FeatureLine/Plane
      └── constraint: Daylight/Offset/Width
```

### 4.3 不需要做的

基于调研分析，以下项目明确**不需要**实施：

| 项目 | 理由 |
|------|------|
| 创建独立 PrimitiveGen 模块 | 当前 Catalog + Feature Engine 路径已足够（所有平台验证） |
| 为 Girt/Purlin/WindBrace 定义独立 ObjectType | Tekla 用 Beam 统一处理，OmniM 应保持一致 |
| BooleanSubtract 降级为属性 | 独立模块更清晰（Tekla 验证） |
| 重构 GenRule 分类层 | 5 类 GenRule 已验证跨平台完备（所有平台验证） |
| 刚接/铰接作为 GenRule 层概念 | 应作为 Connection 组件参数，非独立 GenRule（Tekla 验证） |
| 创建独立钢筋 GenRule | 钢筋本质是 LinearComponent（圆截面+路径+阵列），Tekla 验证 |

---

## 5. 实施路线图

### 阶段 1: 快速胜利（2 周）

```
第 1 周:
  VariationRule 扩展 6 种插值方法
  BooleanSubtract 平面切割模式
  CableTray GenRule 细化

第 2 周:
  TerrainModel 独立 ObjectType
  LogicalObject 层 (Tag/Loop)
  截面库 Profiles/ 初始数据 (IPE/HEA/角钢/槽钢)
```

### 阶段 2: 核心架构（4 周）

```
第 3-4 周:
  ComponentSequenceItem (元件序列模式)
  ExternalTarget 约束
  LinearComponent 多段变截面

第 5-6 周:
  ConnectionScript 机制 (过程化组件)
  Assembly 条件参数
  Assembly 表达式驱动位置
```

### 阶段 3: 扩展功能（6 周）

```
第 7-8 周:
  Rebar 专用模板 (弯钩/半径/阵列)
  Feature::Chamfer / Feature::Fillet
  Feature::Pattern (Linear/Circular)

第 9-12 周:
  Constraint 系统 (21 类运动学约束)
  MBD/PMI 属性集
  BoltArray/BoltCircle/BoltXYList 阵列 Pattern
  Weld Feature
```

---

## 6. 关键决策摘要

| # | 决策 | 选择 | 理由 |
|---|------|------|------|
| 1 | GenRule 分类粒度 | 5 类不变 + TerrainModel + LogicalObject | 跨平台验证完备 |
| 2 | Assembly 建模方式 | 混合（声明式 + 条件参数 + 过程化脚本） | 兼顾灵活性与可复用性 |
| 3 | 变截面策略 | 保留段内插值，增加 6 种方法 + 多段支持 | 工程稳健性 |
| 4 | 截面库管理 | 目录驱动（Catalog ComponentDef） | 可扩展性 |
| 5 | BooleanSubtract 输入 | 实体 + 平面切割 | 效率 |
| 6 | Beam/Column 命名 | 保留分开，统一 GenRule | 语义明确 + 几何统一 |
| 7 | 逻辑对象层 | 新增 LogicalObject | 覆盖 AVEVA 双视图架构 |
| 8 | 连接/约束系统 | 3 层：Connector + Constraint + ConnectionScript | 覆盖全部平台 |

---

## 7. 与现有代码的对应关系

### 7.1 已有代码可直接使用的

| 功能 | 代码位置 | 说明 |
|------|---------|------|
| ParametricOperation (Box/Cylinder/Cone/Sphere/Extrusion/Revolve/Loft/Sweep/Boolean) | `proto/omnim.proto` | 已有，可覆盖 SOLIDWORKS 特征树 |
| LinearComponent (固定截面 + 变截面插值) | `crates/model/src/gen_rule/linear_component.rs` | 已有，核心引擎 |
| Series Store (设计表/等级库) | `crates/model/src/series_store.rs` | 已有，对应 Design Table / Spec |
| ConnectorDef (P-Point 连接点) | `crates/model/src/catalog.rs` | 已有，对应 E3D P-Point / Tekla Mate Ref |
| BooleanSubtract | `crates/model/src/gen_rule/boolean_subtract.rs` | 已有，需扩展平面切割 |
| Assembly | `crates/model/src/gen_rule/assembly.rs` | 已有，需增强条件参数 |

### 7.2 需新增的代码

| 功能 | 建议文件 | 参考 |
|------|---------|------|
| VariationRule 6 种插值 | `linear_component.rs` | Bentley 6 种 Variation |
| BooleanSubtract PlaneCut | `boolean_subtract.rs` | Tekla Fitting/CutPlane |
| ComponentSequenceItem | `linear_component_input` 结构体 | E3D BRANCH |
| ExternalTarget | `linear_component.rs` | Bentley End Condition |
| ConnectionScript | `crates/model/src/gen_rule/connection_script.rs` 新文件 | Tekla ConnectionBase |
| Constraint types | `proto/omnim.proto` + `crates/model/src/relation.rs` | SOLIDWORKS Mates |
| TerrainModel | `proto/omnim.proto` + `crates/model/src/terrain.rs` 新文件 | Bentley TIN |
| LogicalObject | `proto/omnim.proto` + `crates/model/src/logical.rs` 新文件 | AVEVA Loop/Tag |
| Profile 截面库 | `crates/model/src/seed_catalog.rs` 扩展 | Tekla Profile Library |
| Feature::Chamfer/Fillet/Pattern | `crates/model/src/feature.rs` | SOLIDWORKS Feature |