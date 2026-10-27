---
title: "OmniM IFC 兼容性与扩展策略"
status: design
last_updated: 2026-10-27
applies_to: "proto/, crates/model"
evidence: "n/a (design contract)"
---
# OmniM IFC 兼容性与扩展策略

> **最后更新：2026-10-27。** 状态标记与 [FORMAT_COMPAT.zh-CN.md](../status/FORMAT_COMPAT.zh-CN.md) 一致：✅ 当前 / 🔀 部分 / 🗓 计划中。代码是唯一事实来源；下文描述 `crates/ifc/src/lib.rs` 中手写解析器/写出器的实际能力。

**与 FORMAT_COMPAT 的关系**：本文是 [FORMAT_COMPAT.zh-CN.md](../status/FORMAT_COMPAT.zh-CN.md) 的*策略与类映射*配套文档，FORMAT_COMPAT 是权威实现矩阵。两者不一致时以 FORMAT_COMPAT 为准。特别地：当前仅实现 IFC 2x3/IFC 4 的一个子集，不支持 IFC 4x3 特有实体，也不承诺无损往返或 LandXML 能力。

## 1. 概述

OmniM 的数据模型被设计为 **IFC 超集**——兼容 IFC 的一个子集，同时为标准未覆盖的领域添加概念。本文说明扩展命名空间策略与逐类映射状态。

**当前现实（2026-10-27）**：IFC 导入/导出是部分实现的手写 STEP Physical File 解析器/写出器，覆盖 IFC 2x3/IFC 4 的一部分实体与简单几何。它**不**实现 IFC 4x3、材质/工程量/分类映射、高级 BRep/CSG/映射项几何，也不实现无损往返。下文的扩展命名空间与属性集策略仍属设计目标（🗓）。

---

## 2. 核心兼容性原则

### 2.1 尽可能对齐 IFC —— 🔀 部分

对于建筑领域对象，OmniM 对齐 **IFC 2x3/IFC 4 的一个子集**：

- 映射部分对象类型与层级（见 §3）。
- ✅ 属性集：导入时重建单值 `IfcPropertySet`。
- 🔀 关系语义：重建包含关系；不重建连接语义。
- 🗓 IFC 4x3 分类/结构：不作为目标，也不支持。

### 2.2 清晰的扩展机制 —— 🗓 计划中

对于超出 IFC 范围的领域（土木、工厂、机械），设计采用：

- 标准化扩展前缀（`OMNI_CIVIL_`、`OMNI_PLANT_`、`OMNI_MECH_`）。
- 隔离的领域命名空间。
- 不修改核心 IFC 语义。

该命名空间方案仅为设计约定；当前没有任何扩展实体被写成独立的 IFC 实体。扩展对象导出时回退为 `IfcBuildingElementProxy`（🔀），也不会作为类型化领域对象往返（🗓）。

### 2.3 无损往返 —— 🗓 计划中（不作承诺）

IFC 导入 → OmniM 操作 → IFC 导出目前**不是**无损的。旧目标（“所有实体、所有属性集与数量、所有关系、0.1 mm 以内几何精度”）仅作为未来目标保留：

- 🗓 所有 IFC 实体类型——不支持；仅支持 §3 的匹配表。
- 🗓 所有属性集与数量——只读取单值属性集；数量未转换。
- 🗓 所有关系与连接——只重建聚合/包含关系。
- 🗓 0.1 mm 以内几何精度——无此保证；几何被细分并简化。

---

## 3. IFC 类映射（建筑领域）

状态图例：✅ 当前（类型化导入和/或导出）· 🔀 部分（仅导入、导出回退 Proxy，或反之）· 🗓 计划中（代码中无类型化映射）。

### 3.1 空间结构

| OmniM 类 | IFC 类 | 状态 | 备注 |
|-------------|-----------|--------|-------|
| Site | IfcSite | ✅ | 作为空间容器导入/导出 |
| Building | IfcBuilding | ✅ | |
| BuildingStorey | IfcBuildingStorey | ✅ | |
| Space | IfcSpace | ✅ | |

### 3.2 建筑构件

| OmniM 类 | IFC 类 | 状态 | 备注 |
|-------------|-----------|--------|-------|
| Wall | IfcWall / IfcWallStandardCase | ✅ | |
| Beam | IfcBeam / IfcBeamStandardCase | ✅ | |
| Column | IfcColumn / IfcColumnStandardCase | ✅ | |
| Slab | IfcSlab / IfcPlate | ✅ | 楼板、屋顶、基础也导入为 Slab |
| Door | IfcDoor | ✅ | |
| Window | IfcWindow | ✅ | |
| Stair | IfcStair / IfcStairFlight | 🔀 | 导入为 Stair；**导出为 `IfcBuildingElementProxy`**（DATA_MODEL §10.1） |
| Roof | IfcRoof | 🔀 | 导入为 Roof；**导出为 `IfcBuildingElementProxy`**（DATA_MODEL §10.1） |
| Railing | IfcRailing | 🗓 | 不在导入匹配表中；导出回退 Proxy |
| Ramp | IfcRamp | 🔀 | 导入为 Generic；无类型化 Ramp 往返 |
| Plate | IfcPlate | ✅ | 导入为 Slab |
| Member | IfcMember / IfcMemberStandardCase | 🔀 | 导入为 Generic |

### 3.3 MEP 构件

| OmniM 类 | IFC 类 | 状态 | 备注 |
|-------------|-----------|--------|-------|
| Duct | IfcDuctSegment | 🗓 | 无类型化导入映射；导出回退 `IfcBuildingElementProxy` |
| Pipe | IfcPipeSegment | 🗓 | 同上 |
| DuctFitting | IfcDuctFitting | 🗓 | 同上 |
| PipeFitting | IfcPipeFitting | 🗓 | 同上 |
| Valve | IfcValve | 🗓 | 同上 |
| FlowTerminal | IfcFlowTerminal | 🗓 | 风口、送风口 |
| FlowController | IfcFlowController | 🗓 | 风阀等 |
| EnergyConversion | IfcEnergyConversionDevice | 🗓 | 冷水机组、锅炉 |

---

## 4. 领域扩展

OmniM 的设计在 IFC 之外扩展了三个领域专用对象层级。所有类型化扩展实体均为 🗓 计划中；当前只存在下文注明的导入名称匹配与 Proxy 回退。

### 4.1 土木工程扩展

**命名空间前缀**：`OMNI_CIVIL_`（🗓 设计约定）

| 对象类 | 描述 | IFC 等价物 | 状态 |
|--------------|-------------|----------------|--------|
| Road | 道路平纵线形与路面 | IfcCivilElement（提案） | 🔀 出现 `IfcRoad` 时导入；导出为 Proxy；无 4x3 一致性 |
| Pavement | 道路路面结构（多层） | IfcPavement（IFC 4x3） | 🗓 |
| Bridge | 桥梁上部结构与下部结构 | IfcCivilElement（提案） | 🔀 出现 `IfcBridge` 时导入；导出为 Proxy |
| Tunnel | 隧道结构与衬砌 | IfcCivilElement（提案） | 🗓 |
| Dam | 大坝结构与附属设施 | 无 | 🗓 |
| Culvert | 涵洞结构 | 无 | 🗓 |
| RetainingWall | 挡土墙结构 | 无（使用 IfcWall） | 🗓（仅当写成 IfcWall 时作为 Wall 导入） |
| Alignment | 平纵线形 | 无 | 🗓 |
| Profile | 横截面线形 | 无 | 🗓 |
| Site | 项目场地空间容器 | IfcSite | ✅（与 §3.1 同一空间容器） |
| TerrainModel | TIN 曲面地形 | IfcGeographicElement（提案，IFC 4x3） | 🗓 |
| CutVolume / FillVolume | 土方挖填方量 | IfcGeographicElement（数量） | 🗓 |
| GradingSurface | 设计整平面 | IfcGeographicElement（设计面） | 🗓 |
| Slope | 沿边界线边坡面 | IfcGeographicElement（边坡） | 🗓 |
| FeatureLine | 控制/特征线 | 无（IfcAnnotation） | 🗓 |
| ContourLine | 固定高程等高线 | 无（IfcAnnotation） | 🗓 |
| SiteComponent | 地形附着设施 | IfcElement（如 IfcLightFixture） | 🗓 |
| EarthworkBalance | 土方平衡结果 | IfcGeographicElement（数量） | 🗓 |

### 4.2 工厂设计扩展

**命名空间前缀**：`OMNI_PLANT_`（🗓 设计约定）

| 对象类 | 描述 | IFC 等价物 | 状态 |
|--------------|-------------|----------------|--------|
| Tank | 储罐 | IfcTank | 🗓（导出回退 Proxy） |
| Pump | 离心泵、容积泵 | IfcPump | 🗓 |
| Valve | 闸阀、球阀、蝶阀、控制阀 | IfcValve | 🗓 |
| HeatExchanger | 管壳式、板式 | 无 | 🗓 |
| Reactor | 化工反应器 | 无 | 🗓 |
| Vessel | 压力容器 | IfcVessel | 🗓 |
| Piping | 工厂管道网络 | IfcPipeSegment | 🗓 |
| Nozzle | 设备连接口 | 无 | 🗓 |
| Flange | 管道连接法兰 | 无 | 🗓 |
| StructuralSteel | 钢构件与连接 | IfcBeam, IfcColumn | 🗓 |
| Platform | 检修平台 | 无 | 🗓 |
| Ladder | 检修爬梯 | 无 | 🗓 |

### 4.3 机械设计扩展

**命名空间前缀**：`OMNI_MECH_`（🗓 设计约定）

| 对象类 | 描述 | IFC 等价物 | 状态 |
|--------------|-------------|----------------|--------|
| Part | 单个机械零件 | IfcElementAssembly | 🗓 |
| Assembly | 零件装配体 | IfcElementAssembly | 🗓 |
| Fastener | 螺栓、螺母、螺钉、垫圈 | 无 | 🗓 |
| Gear | 直齿、锥齿、斜齿、蜗轮 | 无 | 🗓 |
| Shaft | 旋转轴 | 无 | 🗓 |
| Bearing | 球轴承、滚子轴承、滑动轴承 | 无 | 🗓 |
| Motor | 电动机 | IfcMotor | 🗓 |
| Spring | 压缩、拉伸、扭转弹簧 | 无 | 🗓 |
| Seal | 垫片、O 型圈 | 无 | 🗓 |
| Bushing | 轴套 | 无 | 🗓 |
| Housing | 外壳或箱体 | 无 | 🗓 |

---

## 5. 属性集策略

### 5.1 IFC 属性集（Pset）—— 🔀 部分

- ✅ 导入时读取 `IfcRelDefinesByProperties` + `IfcPropertySet` + `IfcPropertySingleValue`，并重建为原生属性集。
- ✅ 导出时为支持的实体写出单值属性（IfcGlobalId/name/description 及简单属性值）。
- 🗓 不保证完整覆盖标准属性集（`Pset_WallCommon`、`Pset_BeamCommon`、`Pset_ColumnCommon` 等）。
- 🗓 `Qto_*` 数量不转换。
- 🗓 材质关联（`IfcMaterial`、`IfcRelAssociatesMaterial`）、材料层与分类引用不转换。

### 5.2 OmniM 扩展属性集 —— 🗓 计划中

领域扩展设计使用基于前缀的属性集：

| 领域 | 前缀 | 示例 |
|--------|--------|---------|
| 土木 | `Pset_Civil_` | `Pset_Civil_Road` |
| 工厂 | `Pset_Plant_` | `Pset_Plant_Pump` |
| 机械 | `Pset_Mech_` | `Pset_Mech_Fastener` |

### 5.3 自定义属性集 —— 🗓 计划中

用户自定义属性集设计使用：

- 无保留前缀。
- 应用专用前缀：`Pset_<AppName>_`。
- 公司专用前缀：`Pset_<Company>_`。

未知属性集的往返保留尚未实现。

---

## 6. IFC 导出策略

### 6.1 标准 IFC 对象 —— ✅ 当前（子集）/ 🔀 部分

具有已实现 IFC 等价物的对象以原生 IFC 实体导出（Wall、Beam、Column、Slab、Door、Window、Site、Building、BuildingStorey、Space）。几何在可用时写成 `IFCEXTRUDEDAREASOLID` / `IFCREVOLVEDAREASOLID` 或分面 BRep。

- 🔀 Stair 与 Roof 导出为 `IfcBuildingElementProxy`（见 DATA_MODEL §10.1），而非 `IfcStair` / `IfcRoof`。
- 🔀 其他所有对象类型回退为 `IfcBuildingElementProxy`；不输出类型化领域实体。
- 🗓 全保真导出所有属性与关系尚未实现。

### 6.2 扩展领域对象 —— 🔀 Proxy 回退 / 🗓 设计

对于没有直接 IFC 等价物的对象，目标策略为：

1. **映射到最接近的 IFC 超类型**：例如 `IfcCivilElement`、`IfcDiscreteAccessory`。
2. **将 OmniM 类型添加到属性集**：`Pset_OmniM_ObjectType = "HeatExchanger"`。
3. **在各自的属性集中添加领域专用属性**。
4. **通过自定义属性标记为扩展实体**，便于重新识别。

当前代码只执行 Proxy 回退（🔀）。第 2–4 步为 🗓 计划中；`Pset_OmniM_ObjectType` 目前不会写出。

### 6.3 导出配置选项 —— 🗓 计划中

```
Export Profile:
  ├── Full (all objects, all properties)
  ├── Geometry Only (no properties)
  ├── Spatial Only (site/building/storey hierarchy)
  └── Coordination View (for clash detection)

LOD Filter:
  ├── LOD 100 (Concept)
  ├── LOD 200 (Design)
  ├── LOD 300 (Detailed)
  ├── LOD 400 (Construction)
  └── LOD 500 (As-built)
```

当前服务器未暴露导出配置或 LOD 过滤选项。

---

## 7. IFC 导入策略

### 7.1 标准 IFC 实体 —— ✅ 当前（子集）

- ✅ 固定匹配表将实体类型映射到 OmniM 对象类型（见 FORMAT_COMPAT §2 与上文 §3）。
- ✅ 单值属性集原生导入。
- 🔀 仅将 `IfcRelAggregates` 与 `IfcRelContainedInSpatialStructure` 重建为包含关系；其他关系不重建。

### 7.2 扩展 IFC 实体 —— 🗓 计划中

- 🗓 带 `Pset_OmniM_ObjectType` 的对象 → 映射到扩展领域类型。
- 🔀 未知属性集不会作为自定义属性往返。
- 🔀 未知实体类型会被解析进原始实体表，但**不会**创建对象；当前不存在为它们生成 Generic 回退对象的机制。

### 7.3 几何导入流水线 —— 🔀 部分

```
IFC Geometry Definition
    ↓
Parse (extruded-area solids, revolved solids, faceted BRep)
    ↓
Convert to meters, standard orientation
    ↓
Tessellate to mesh (for rendering)
    ↓
Store as native OmniM geometry
```

- ✅ 当前流水线解析拉伸/旋转轮廓与分面 BRep，归一化放置/原点，并细分为 `MeshGeometry`。
- 🗓 几何修复（修复无效拓扑）未实现。
- 🗓 LOD 变体生成未实现。
- 🗓 高级 BRep、CSG、扫掠实体、组合实体，以及带变换的映射项均不支持。

---

## 8. 版本兼容性

### 8.1 支持的 IFC 版本

| 版本 | 导入 | 导出 | 状态 |
|---------|--------|--------|--------|
| IFC 2x3 | ✅ 子集 | ✅ 子集 | 🔀 部分——与 IFC 4 共用同一代码路径 |
| IFC 4 | ✅ 子集 | ✅ 子集 | 🔀 部分 |
| IFC 4x3 | ❌ | ❌ | 🗓 不支持——无 IFC 4x3 特有实体 |

**不作 IFC 4x3 支持承诺。** 手写解析器可能机会性地识别 `IfcRoad` / `IfcBridge` 名称，但这不属于模式级 4x3 一致性。

### 8.2 版本转换 —— 🗓 计划中

导出时的自动版本转换未实现。旧设计（IFC 4 → 2x3 降级并警告；2x3 → 4 升级并做属性集映射）仍为目标。

---

## 9. LandXML 地形互操作 —— 仅 🗓 计划中

> **调研参考**：`docs/research/site_terrain_components.md §9.2`。Civil 3D 和 Bentley 原生支持 LandXML 格式的地形数据交换。

LandXML **未实现**。仓库中没有 LandXML 读取器/写出器，也不存在双格式交换。下表仅为设计目标（🗓）。

| LandXML 元素 | OmniM 映射（计划） |
|-------------|-------------------|
| `Surfaces` | TerrainModel 集合 |
| `Surface` | TerrainModel（单曲面） |
| `Definition` | TerrainVertex + TerrainTriangle |
| `Breaklines` | TerrainBreakline |
| `Boundary` | TerrainBoundary |
| `Grade` | GradingSurface |

旧文档所称“IFC 4x3 提供空间容器层级与 TerrainModel 的地理元素映射”同样为 🗓 计划中，因为 IFC 4x3 不受支持。

---

## 10. 测试与验证

### 10.1 参考模型 —— 🗓 计划中

维护一套 BuildingSMART 参考模型、真实项目示例与边界情况模型是设计目标；当前测试覆盖中不存在该套件。

### 10.2 验证检查 —— 🔀 部分

每次导入/导出，当前代码执行其中一部分检查：

1. 🔀 几何有效性仅限细分/解析器可接受的范围；不运行全局非流形/闭合实体校验。
2. ✅ 报告导入对象数量（`ImportIfcResponse.objects`）。
3. 🗓 属性往返检查未自动化。
4. 🔀 关系完整性仅限包含关系重建。
5. 🗓 文件大小与性能基准未维护。

---

## 11. 待解决问题

1. 土木/工厂/机械扩展是否应遵循任何现有标准（ISO 15926 等）？
2. 表示线形与横截面以进行 IFC 导出的最佳方式是什么？
3. 参数化历史应在 IFC 中保留，还是仅保留最终几何？
4. 如何跨领域处理紧固件标准与目录数据？
5. 应将何种程度的 STEP AP242 PMI 映射到 OmniM 属性？
