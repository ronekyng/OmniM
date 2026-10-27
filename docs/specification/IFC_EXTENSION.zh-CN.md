# OmniM IFC 兼容性与扩展策略

## 1. 概述

OmniM 的数据模型被设计为 **IFC 超集** - 完全兼容 IFC，同时为标准未覆盖的领域添加支持。本文档概述了兼容性策略与扩展机制。

---

## 2. 核心兼容性原则

### 2.1 尽可能对齐 IFC
对于建筑领域对象，OmniM 遵循 IFC 4x3 的分类与结构：
- 相同的对象类型与层级
- 兼容的属性集定义
- 相似的关系语义

### 2.2 清晰的扩展机制
对于超出 IFC 范围的领域（土木、工厂、机械），OmniM 采用：
- 标准化的扩展前缀
- 隔离的领域命名空间
- 不修改核心 IFC 语义

### 2.3 无损往返
IFC 导入 → OmniM 操作 → IFC 导出 应保留：
- 所有 IFC 实体类型
- 所有属性集与数量
- 所有关系与连接
- 0.1mm 以内的几何精度

---

## 3. IFC 类映射（建筑领域）

### 3.1 空间结构

| OmniM 类 | IFC 类 | 备注 |
|-------------|-----------|-------|
| Site | IfcSite | |
| Building | IfcBuilding | |
| BuildingStorey | IfcBuildingStorey | |
| Space | IfcSpace | |

### 3.2 建筑构件

| OmniM 类 | IFC 类 | 备注 |
|-------------|-----------|-------|
| Wall | IfcWall / IfcWallStandardCase | |
| Beam | IfcBeam | |
| Column | IfcColumn | |
| Slab | IfcSlab | 包括楼板、屋顶、基础 |
| Door | IfcDoor | |
| Window | IfcWindow | |
| Stair | IfcStair / IfcStairFlight | |
| Roof | IfcRoof | |
| Railing | IfcRailing | |
| Ramp | IfcRamp | |
| Plate | IfcPlate | |
| Member | IfcMember | |

### 3.3 MEP 构件

| OmniM 类 | IFC 类 | 备注 |
|-------------|-----------|-------|
| Duct | IfcDuctSegment | |
| Pipe | IfcPipeSegment | |
| DuctFitting | IfcDuctFitting | |
| PipeFitting | IfcPipeFitting | |
| Valve | IfcValve | |
| FlowTerminal | IfcFlowTerminal | 风口、送风口 |
| FlowController | IfcFlowController | 风阀等 |
| EnergyConversion | IfcEnergyConversionDevice | 冷水机组、锅炉 |

---

## 4. 领域扩展

OmniM 在 IFC 之外扩展了三个额外的领域专用对象层级。

### 4.1 土木工程扩展

**命名空间前缀**：`OMNI_CIVIL_`

| 对象类 | 描述 | IFC 等价物 |
|--------------|-------------|----------------|
| Road | 道路平纵线形与路面 | IfcCivilElement（提案） |
| Pavement | 道路路面结构（多层） | IfcPavement（IFC 4x3） |
| Bridge | 桥梁上部结构与下部结构 | IfcCivilElement（提案） |
| Tunnel | 隧道结构与衬砌 | IfcCivilElement（提案） |
| Dam | 大坝结构与附属设施 | 无 |
| Culvert | 涵洞结构 | 无 |
| RetainingWall | 挡土墙结构 | 无（使用 IfcWall） |
| Alignment | 平纵线形 | 无 |
| Profile | 横截面线形 | 无 |
| Site | 项目场地空间容器 | IfcSite |
| TerrainModel | TIN 曲面地形 | IfcGeographicElement（提案，IFC 4x3） |
| CutVolume / FillVolume | 土方挖填方量 | IfcGeographicElement（数量） |
| GradingSurface | 设计整平面 | IfcGeographicElement（设计面） |
| Slope | 沿边界线边坡面 | IfcGeographicElement（边坡） |
| FeatureLine | 控制/特征线 | 无（IfcAnnotation） |
| ContourLine | 固定高程等高线 | 无（IfcAnnotation） |
| SiteComponent | 地形附着设施 | IfcElement（如 IfcLightFixture） |
| EarthworkBalance | 土方平衡结果 | IfcGeographicElement（数量） |

### 4.2 工厂设计扩展

**命名空间前缀**：`OMNI_PLANT_`

| 对象类 | 描述 | IFC 等价物 |
|--------------|-------------|----------------|
| Tank | 储罐 | IfcTank |
| Pump | 离心泵、容积泵 | IfcPump |
| Valve | 闸阀、球阀、蝶阀、控制阀 | IfcValve |
| HeatExchanger | 管壳式、板式 | 无 |
| Reactor | 化工反应器 | 无 |
| Vessel | 压力容器 | IfcVessel |
| Piping | 工厂管道网络 | IfcPipeSegment |
| Nozzle | 设备连接口 | 无 |
| Flange | 管道连接法兰 | 无 |
| StructuralSteel | 钢构件与连接 | IfcBeam, IfcColumn |
| Platform | 检修平台 | 无 |
| Ladder | 检修爬梯 | 无 |

### 4.3 机械设计扩展

**命名空间前缀**：`OMNI_MECH_`

| 对象类 | 描述 | IFC 等价物 |
|--------------|-------------|----------------|
| Part | 单个机械零件 | IfcElementAssembly |
| Assembly | 零件装配体 | IfcElementAssembly |
| Fastener | 螺栓、螺母、螺钉、垫圈 | 无 |
| Gear | 直齿、锥齿、斜齿、蜗轮 | 无 |
| Shaft | 旋转轴 | 无 |
| Bearing | 球轴承、滚子轴承、滑动轴承 | 无 |
| Motor | 电动机 | IfcMotor |
| Spring | 压缩、拉伸、扭转弹簧 | 无 |
| Seal | 垫片、O 型圈 | 无 |
| Bushing | 轴套 | 无 |
| Housing | 外壳或箱体 | 无 |

---

## 5. 属性集策略

### 5.1 IFC 属性集（Pset）

OmniM 原生支持所有标准 IFC 属性集：
- `Pset_WallCommon`、`Pset_BeamCommon`、`Pset_ColumnCommon` 等
- `Qto_WallBaseQuantities` 等用于数量

这些属性集严格按照 IFC 规范进行存储与导出。

### 5.2 OmniM 扩展属性集

领域扩展使用基于前缀的属性集：

| 领域 | 前缀 | 示例 |
|--------|--------|---------|
| 土木 | `Pset_Civil_` | `Pset_Civil_Road` |
| 工厂 | `Pset_Plant_` | `Pset_Plant_Pump` |
| 机械 | `Pset_Mech_` | `Pset_Mech_Fastener` |

### 5.3 自定义属性集

用户自定义属性集使用：
- 无保留前缀
- 应用专用前缀：`Pset_<AppName>_`
- 公司专用前缀：`Pset_<Company>_`

---

## 6. IFC 导出策略

### 6.1 标准 IFC 对象
具有 IFC 等价物的对象以原生 IFC 实体导出，保持完全保真度。

### 6.2 扩展领域对象
对于没有直接 IFC 等价物的对象：
1. **映射到最接近的 IFC 超类型**：例如 `IfcCivilElement`、`IfcDiscreteAccessory`
2. **将 OmniM 类型添加到属性集**：`Pset_OmniM_ObjectType = "HeatExchanger"`
3. **在各自的属性集中添加领域专用属性**
4. **通过自定义属性标记为扩展实体**，便于重新识别

### 6.3 导出配置选项
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

---

## 7. IFC 导入策略

### 7.1 标准 IFC 实体
- 直接映射到 OmniM 对象类型
- 属性集原生导入
- 关系保留为 OmniM 关系

### 7.2 扩展 IFC 实体
- 带有 `Pset_OmniM_ObjectType` 的对象 → 映射到扩展领域类型
- 未知属性集 → 作为自定义属性保留
- 未知实体类型 → 映射到 Generic 对象，并存储原始 IFC 类

### 7.3 几何导入流水线
```
IFC Geometry Definition
    ↓
Parse (Extrusion, BRep, CSG, etc.)
    ↓
Convert to meters, standard orientation
    ↓
Geometry Healing (fix invalid topology)
    ↓
Tessellate to mesh (for rendering)
    ↓
Generate LOD variants
    ↓
Store as native OmniM geometry
```

---

## 8. 版本兼容性

### 8.1 支持的 IFC 版本

| 版本 | 导入 | 导出 |
|---------|--------|--------|
| IFC 2x3 | ✅ 是 | ✅ 是 |
| IFC 4 | ✅ 是 | ✅ 是 |
| IFC 4x3 | ✅ 是 | ✅ 是 |

### 8.2 版本转换
OmniM 支持在导出时进行自动版本转换：
- IFC 4 → IFC 2x3：降级并对不支持的功能给出警告
- IFC 2x3 → IFC 4：升级并进行属性集映射

---

## 9. LandXML 地形互操作

> **调研参考**：`docs/research/site_terrain_components.md §9.2`。Civil 3D 和 Bentley 原生支持 LandXML 格式的地形数据交换。OmniM 支持双格式交换：**IFC 4x3** 用于空间层级（IfcSite）+ **LandXML** 用于地形/TIN 数据。

| LandXML 元素 | OmniM 映射 |
|-------------|-----------|
| `Surfaces` | TerrainModel 集合 |
| `Surface` | TerrainModel（单曲面） |
| `Definition` | TerrainVertex + TerrainTriangle |
| `Breaklines` | TerrainBreakline |
| `Boundary` | TerrainBoundary |
| `Grade` | GradingSurface |

LandXML 是地形数据交换的事实标准；IFC 4x3 提供空间容器层级（IfcSite → IfcBuilding → IfcBuildingStorey → IfcSpace）和地理元素映射（IfcGeographicElement 对应 TerrainModel）。

---

## 10. 测试与验证

### 10.1 参考模型
OmniM 维护一套参考模型测试套件：
- Building Smart 官方测试用例
- 真实项目示例
- 边界情况模型（复杂几何、大文件）

### 10.2 验证检查
对每次导入/导出：
1. 几何有效性（无非流形边、闭合实体）
2. 对象数量核对
3. 属性往返检查
4. 关系完整性
5. 文件大小与性能基准

---

## 11. 待解决问题

1. 土木/工厂/机械扩展是否应遵循任何现有标准（ISO 15926 等）？
2. 表示线形与横截面以进行 IFC 导出的最佳方式是什么？
3. 参数化历史应在 IFC 中保留，还是仅保留最终几何？
4. 如何跨领域处理紧固件标准与目录数据？
5. 应将何种程度的 STEP AP242 PMI 映射到 OmniM 属性？
