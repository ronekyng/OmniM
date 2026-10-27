# CAD/BIM 平台场地与地形构件逻辑研究

> 调研日期：2026-08-07
> 目标：研究 Revit、Civil 3D、Bentley OpenRoads/OpenBridge、AVEVA E3D、Tekla Structures、SolidWorks 六平台的场地（Site）与地形（Terrain）实现逻辑，进行横纵对比分析并映射到 OmniM 架构
> 状态：完成

---

## 0. 研究范围与定义

### 0.1 "场地"与"地形"的范畴界定

本文研究的 "Site/Terrain" 涵盖以下工程概念：

| 概念 | 定义 | 典型操作 |
|------|------|---------|
| **Site（场地）** | 工程项目的地理/空间容器，包含边界、位置、朝向等 | 场地边界定义、建筑放置、场地规划 |
| **Terrain（地形）** | 地面的三维几何表示（TIN/网格/等高线） | 地形建模、地形编辑、地形分析 |
| **Earthwork（土方）** | 基于地形的挖填计算与施工 | 挖方/填方量计算、土方平衡 |
| **Grading（平整）** | 场地设计面的创建与地形修改 | 设计面创建、平整规则、边坡生成 |
| **Slope（边坡）** | 场地或道路边缘的斜面过渡 | 边坡设计、坡比控制、daylight 求交 |
| **Topography（地形学）** | 等高线、地貌特征的表示 | 等高线显示、坡度分析、汇水分析 |

### 0.2 平台覆盖

| 平台 | 厂商 | 主要领域 | 场地/地形支持程度 |
|------|------|---------|----------------|
| Revit | Autodesk | 建筑 BIM | 中等（Site 容器 + 基础地形） |
| Civil 3D | Autodesk | 土木工程 | **强**（完整地形+土方+平整体系） |
| Bentley | Bentley Systems | 基础设施 | **强**（TIN 地形+走廊+土方+边坡） |
| AVEVA E3D | AVEVA | 工厂设计 | 弱（Site 空间引用，无原生地形） |
| Tekla Structures | Trimble | 钢结构/混凝土详图 | 无（仅车间/现场标签，无地形） |
| SolidWorks | Dassault Systèmes | 机械 CAD | 无（无场地/地形概念） |

---

## 1. Revit：Site 作为空间容器 + Toposurface 基础地形

### 1.1 Site 的定位

在 Revit 中，**Site 是 Spatial 层级结构的根节点**，对应 IFC 的 `IfcSite`：

```
Spatial Hierarchy
├── Site (IfcSite)        ← 地理/空间容器
│   ├── Building (IfcBuilding)
│   │   ├── BuildingStorey (IfcBuildingStorey)
│   │   │   └── Space (IfcSpace)
│   │   └── ...
│   └── 其他：道路、地形、绿化等
```

Revit Site 的核心属性：
- `Site Name`：场地名称
- `Site Address`：地理地址
- `Latitude / Longitude`：地理坐标
- `Site Rotation`：场地朝向
- `Building Location`：建筑在场地中的位置
- `Project Base Point`：项目基点（测量坐标）
- `Survey Point`：测量点（真实世界坐标）

### 1.2 Toposurface（地形面）

Revit 提供 **Toposurface**（地形面）作为地形的几何表达：

```
Toposurface
├── 数据来源
│   ├── 导入点文件（.txt / .csv）
│   ├── 导入等高线（DWG/DXF）
│   ├── 手动创建点
│   └── 从导入的 CAD 线生成
├── 几何表示
│   ├── TIN（不规则三角网）
│   └── 三角面片 + 高程值
├── 编辑操作
│   ├── 添加/删除点
│   ├── 调整点高程
│   ├── 分割地形面
│   └── 合并地形面
└── 显示形式
    ├── 三角网
    ├── 等高线
    ├── 坡度分析（Color Scheme）
    └── 点云显示
```

### 1.3 场地构件与大地形交互

| 交互方式 | Revit 实现 | 说明 |
|---------|-----------|------|
| 建筑放置 | Building 放置到 Site 内 | 建筑底标高可自动关联地形 |
| 地形切割 | 基础/地下室在地形中自动"剪切" | 类似 Void 布尔差集 |
| 楼板匹配地形 | Floor/Slab 使用 "Shape Editing" 修改顶点 | 板角点高程可贴合地形 |
| 地形子区域 | Toposurface 支持子区域（Sub-region） | 可指定不同区域的显示/材质 |
| 场地构件 | Site 构件（树木、电线杆、消防栓等） | 基于 Toposurface 表面的 "Site Component" |

### 1.4 Revit Site 的局限

- **无 TIN 编辑工具**：仅支持点级别编辑，无断裂线（breakline）、边界约束
- **无精确土方计算**：Revit 不提供工程级的挖填方量计算
- **无平整模板**：缺少 Civil 3D/Bentley 那样的设计面模板系统
- **无边坡自动生成**：边坡需手动创建或通过 Dynamo 脚本实现
- **无地形分析**：缺少汇水线、坡度分析、等高线插值等分析功能
- **地形与结构交互弱**：建筑基础与地形的联动修改不够流畅

### 1.5 GenRule 映射

| Revit 概念 | OmniM 映射 | 说明 |
|-----------|-----------|------|
| Site (IfcSite) | `ObjectType::Site` | 空间容器，无几何 |
| Toposurface | `ObjectType::TerrainModel` | TIN 曲面，独立 ObjectType（非 GenRule） |
| Toposurface 点编辑 | `TerrainModel.edit_vertices()` | 顶点级高程编辑 |
| Building Location on Site | `Relation::SpatialContainment` | 空间包含关系 |
| Site Component | `ComponentDef` on Terrain | 地形表面附属构件 |

---

## 2. Civil 3D：最完整的场地-地形-土方体系

### 2.1 核心架构

Civil 3D 拥有业界最完整的场地与地形建模体系，围绕 **TIN 曲面 + 走廊 (Corridor) + 装配 (Assembly)** 三大支柱构建：

```
Civil 3D 场地/地形架构
├── TIN Surface（地形面）
│   ├── 数据来源
│   │   ├── 测量点（Survey Points）
│   │   ├── 导入点文件
│   │   ├── 等高线（Contours）
│   │   ├── 断裂线（Breaklines）
│   │   ├── 边界（Boundaries）
│   │   └── 从走廊生成（Corridor → Surface）
│   ├── 曲面类型
│   │   ├── Existing Surface（现有地面）
│   │   ├── Design Surface（设计面）
│   │   ├── Subgrade Surface（路基面）
│   │   └── Datum Surface（基准面）
│   └── 编辑操作
│       ├── 添加/删除点
│       ├── 断裂线约束
│       ├── 边界约束（Outer/Benchmark）
│       ├── 曲面合并
│       ├── 曲面分割
│       └── 高程调整
├── Grading（平整）
│   ├── Grading Group（平整组）
│   │   ├── 平整面（Grading Surface）
│   │   ├── 平整规则（Grading Rule）
│   │   └── 目标曲面（Target Surface）
│   ├── Feature Line（特征线）
│   │   ├── 地坪线
│   │   ├── 场地边界线
│   │   └── 排水路径
│   └── 设计标准（Design Criteria）
│       ├── 最小坡长
│       ├── 最大坡比
│       └── 平台宽度
├── Earthwork（土方）
│   ├── Cut/Fill 计算
│   │   ├── 平均端面积法（Average End Area）
│   │   ├── 棱柱体积法（Prismoidal）
│   │   └── 三角网法（Triangulation）
│   ├── 土方平衡
│   │   ├── 挖填平衡计算
│   │   ├── 余方/缺方
│   │   └── 运距优化
│   └── 工程量报表
│       ├── 断面工程量
│       ├── 总体工程量
│       └── 分段工程量
└── Corridor（走廊）
    ├── Alignment（平面线形）
    ├── Profile（纵断面）
    ├── Assembly（横断面模板）
    ├── Targets（目标曲面）
    └── 土方集成（Corridor → Surface → Earthwork）
```

### 2.2 TIN Surface 数据结构

Civil 3D TIN Surface 是一个**功能完善的不规则三角网**：

```csharp
// Civil 3D TIN Surface API（简化）
public class TinSurface : Surface {
    public Point3dCollection Points { get; }          // 顶点集合
    public BreaklineCollection Breaklines { get; }     // 断裂线（三角形边不可跨越）
    public BoundaryCollection Boundaries { get; }      // 外边界 + 内边界（洞）
    public int TriangleCount { get; }                  // 三角形数量
    
    // 核心操作
    public void AddPoint(Point3d point);
    public void AddBreakline(Polyline breakline);
    public void AddBoundary(Polyline boundary, bool isOuter);
    public void RemovePoint(int index);
    public double GetElevation(double x, double y);    // 双线性插值
    public TinSurface CreateFromCorridor(Corridor corridor);
}
```

### 2.3 Grading（平整）系统

Civil 3D 的 Grading 是基于 **Feature Line + Grading Surface + Target Surface** 的设计方法：

```
Grading Workflow
1. 绘制 Feature Line（特征线）
   ├── 地坪线（按指定坡度/高程）
   ├── 场地边界线
   └── 台阶/平台边缘线
   
2. 创建 Grading Group
   ├── 指定 Grading Surface（平整面）
   ├── 指定 Target Surface（现有地形）
   └── 指定 Grading Rule：
       ├── CutSlope（挖方坡比，如 1:1.5）
       ├── FillSlope（填方坡比，如 1:2）
       ├── MaxCutDepth（最大挖深）
       └── Daylight（边坡与地面相交）

3. 生成设计面
   ├── Feature Line + Grading Rule → 3D TIN Surface
   ├── 自动计算边坡与地形的 daylight 交点
   └── 支持多台阶平台（Bench Grading）
   
4. 土方计算
   ├── Design Surface vs Existing Surface
   ├── 自动生成分段工程量
   └── 支持土方平衡优化
```

### 2.4 Earthwork（土方计算）

Civil 3D 提供**三种土方计算方法**：

| 方法 | 算法 | 精度 | 适用场景 |
|------|------|------|---------|
| **Average End Area（平均端面积法）** | 两相邻横断面面积平均值 × 桩号间距 | 中等（工程常用） | 线路工程（道路/渠道） |
| **Prismoidal（棱柱体积法）** | 三断面棱柱体积公式（辛普生法） | 较高 | 精确土方计算 |
| **Triangulation（三角网法）** | 两 TIN 对应三角形形成棱柱 | 高（最精确） | 大面积场地平整 |

Civil 3D 土方计算的核心数据：

```csharp
// 土方计算输出
public class EarthworkResult {
    public double CutVolume { get; set; }      // 挖方量（m³）
    public double FillVolume { get; set; }     // 填方量（m³）
    public double NetVolume { get; set; }      // 净方量（挖-填）
    public double SurfaceArea { get; set; }    // 表面积（m²）
    public StationRange Range { get; set; }    // 桩号范围
    public bool IsBalanced { get; }            // 是否挖填平衡
}
```

### 2.5 与地形交互的 Target Mapping

Civil 3D 的 Target Mapping 是实现走廊/平整与地形联动的核心机制：

```
Target Mapping 类型
├── Surface Target（曲面目标）
│   ├── 边坡到达地面（Daylight to Terrain）
│   ├── 路面贴附设计面
│   └── 路基底面贴附 Subgrade
├── Offset Target（偏移目标）
│   ├── 特征线偏移
│   └── 曲面偏移
├── Elevation Target（高程目标）
│   ├── 纵断面高程
│   └── 曲面高程
└── Width Target（宽度目标）
    ├── 路面宽度随条件变化
    └── 基于地形条件的宽度调整
```

### 2.6 GenRule 映射

| Civil 3D 概念 | OmniM 映射 | 说明 |
|--------------|-----------|------|
| TinSurface | `ObjectType::TerrainModel` | TIN 曲面（独立 ObjectType，非 GenRule） |
| Surface 编辑 | `TerrainModel` 顶点/边操作 | 顶点添加/删除、断裂线、边界 |
| Grading Surface | `TerrainModel`（设计面类型） | 设计 TIN 曲面 |
| Grading Rule | `GenRule::LinearComponent` 的 End Condition | 边坡断面 + Daylight 目标 |
| CutVolume / FillVolume | `GenRule::BooleanSubtract` | 两 TIN 曲面布尔差 |
| Corridor + Target | `GenRule::LinearComponent` + `ExternalTarget` | 路径 + 横断面 + 地形约束 |
| Feature Line | `LinearElement`（无厚度曲线） | 特征线/控制线 |
| Earthwork 计算 | `TerrainModel.volume_analysis()` | 两 TIN 体积差分析 |

---

## 3. Bentley OpenRoads/OpenBridge/OpenTunnel：走廊模板+TIN 地形

### 3.1 地形模型（Terrain Model）

Bentley 的地形模型是与 Civil 3D TIN 类似的 **不规则三角网**：

```
Terrain Model
├── Points（点数据：x, y, z）
├── Break Lines（断裂线：三角形边不可跨越）
├── Boundary（外边界：约束三角网范围）
├── Void / Break Void（空洞/带高程空洞）
└── Triangles（三角形：由点与断裂线自动三角化）
```

Bentley 地形的**六种 Volume Option** 决定其在土方计算中的角色：

| Volume Option | 含义 | 对应 OmniM |
|--------------|------|-----------|
| Existing | 现有地面（被减面） | `TerrainModel(role="existing")` |
| Design | 设计面（被加面） | `TerrainModel(role="design")` |
| Subgrade | 路基面（独立计方） | `TerrainModel(role="subgrade")` |
| None | 不参与体积计算 | `TerrainModel(role="reference")` |

### 3.2 挖填方计算（Cut Fill Volumes）

Bentley 的 `Create Cut Fill Volumes` 工具：

```
输入：Existing Surface + Design Surface
输出：3D mesh solid + 体积属性
算法：从 Design Surface 向 Existing Surface 垂直拉伸边界 → 闭合实体 → 计算体积
```

三种体积计算方法：

| 方法 | 工具 | 特点 |
|------|------|------|
| End Area Volumes | Corridor Modeling > End Area Volume | **推荐**，按横断面平均端面积法 |
| Analyze Volumes | Civil Analysis > Analyze Volume | 三角棱柱体积法 |
| Terrain to Plane | Civil Analysis > Analyze Volume | 洪水位/堆料体积 |

### 3.3 边坡（Slope）

Bentley 边坡有两种实现方式：

```
方式 1：Corridor End Condition（走廊端部条件）
├── 从路面边缘向外延伸边坡（如 1V:6H 填方、1V:1H 挖方）
├── Target：Active Terrain Model
├── Daylight Point：边坡与地形相交点（求解器自动求交）
├── End Condition Exception：特定桩号区间覆盖默认边坡
└── Priority：多组端部条件按序尝试

方式 2：Independent Linear Template（独立线性模板）
├── 沿场地边界线扫掠边坡断面
├── 独立于走廊的边坡设计
└── 支持护坡层（Surface Template）
```

### 3.4 地形与走廊的协同

Bentley 三产品（ORD/OBD/OTD）共用统一的数据模型：

```
Alignment → Profile → Corridor → Terrain Interaction
                                          │
                                          ├── End Condition Daylight → Terrain
                                          ├── Create Terrain Model from Corridor
                                          ├── Cut Fill Volumes (Existing vs Design)
                                          └── Surface Template (路面铺装到地形)
```

### 3.5 GenRule 映射

| Bentley 概念 | OmniM 映射 | 说明 |
|-------------|-----------|------|
| Terrain Model (TIN) | `ObjectType::TerrainModel` | TIN 曲面（独立 ObjectType，非 GenRule） |
| Volume Option | `TerrainModel.role` | existing/design/subgrade/reference |
| Cut Fill Volumes | `GenRule::BooleanSubtract` | 两 TIN 曲面间布尔差 |
| Slope (End Condition) | `GenRule::LinearComponent` + `ExternalTarget` | 边坡断面沿边缘线扫掠 + Daylight |
| Surface Template | `GenRule::LayerExtrusion` | 多层截面在地形上铺装 |
| Terrain from Corridor | `TerrainModel` 合成 | 走廊输出 TIN 曲面 |
| Pond Control Line → Pond Surface | `GenRule::LinearComponent` + BooleanSubtract | 池断面扫掠 + 与地形求挖填 |

---

## 4. AVEVA E3D：Site 作为空间引用容器

### 4.1 Site 的定位

在 AVEVA E3D 工厂设计中，**Site 是项目层级中的地理/组织容器**：

```
E3D 项目层级
├── PROJECT（项目）
│   ├── SITE（场地）← 地理/组织容器
│   │   ├── AREA（区域）
│   │   │   ├── UNIT（单元）
│   │   │   │   └── 设备/管线/仪表...
│   │   │   └── ...
│   │   └── ...
│   └── ...
```

### 4.2 Site 的功能

E3D Site 的核心功能是**逻辑/组织容器**，而非几何建模：

| 功能 | 说明 |
|------|------|
| 地理定位 | 存储场地坐标、朝向 |
| 区域划分 | 组织 Area/Unit 层级 |
| 设备归属 | 管理设备到场地的归属关系 |
| 参照数据 | 参考外部地形/场地数据 |
| 权限控制 | 基于 Site 的用户权限管理 |

### 4.3 与地形的交互

E3D **无原生地形建模能力**，但支持通过**引用外部数据**实现地形交互：

```
E3D 地形交互方式
├── 外部参照（Reference Model）
│   ├── 引用 Civil 3D / MicroStation 地形文件
│   └── 地形作为背景参照
├── 场地平面图（Site Plan）
│   ├── 2D 场地布置图
│   └── 设备位置在场地平面上的投影
├── 地坪标高（Floor Elevation）
│   ├── 设备地坪标高指定
│   └── 管线埋深/架空高度
└── 无 TIN、无土方、无平整
```

### 4.4 GenRule 映射

| E3D 概念 | OmniM 映射 | 说明 |
|---------|-----------|------|
| SITE | `ObjectType::Site` | 空间/组织容器，无几何 |
| AREA | `ObjectType::Zone` | 子区域（逻辑容器） |
| Site Reference | `ExternalReference` | 外部地形数据引用 |
| Floor Elevation | `PropertySet` | 地坪标高属性 |

---

## 5. Tekla Structures：无地形概念，车间/现场标签

### 5.1 场地/地形的缺失

Tekla Structures **完全没有地形/场地建模能力**——它是纯粹的钢结构/混凝土结构详图工具，关注的是从基础到屋顶的结构构件详图设计。

Tekla 中与"场地"最接近的概念是：

| Tekla 概念 | 说明 | 与场地/地形的关系 |
|-----------|------|----------------|
| **Workshop/Site** | 焊接/制造位置标识 | "车间焊" vs "现场焊"——制造位置标签，非地理空间 |
| **Cast Unit** | 预制/现浇标识 | `CAST_IN_PLACE` / `PRECAST`——施工方法标签 |
| **Position** | 构件在模型中的位置 | 相对坐标定位，无地理坐标概念 |
| **Assembly** | 构件装配层级 | 逻辑分组，无空间/地理含义 |

### 5.2 结构与地形的交互模式

Tekla 中结构与"地面"的交互通过以下简化方式实现：

```
Tekla 结构-地面交互
├── 基础（Footing）
│   ├── ContourPlate（底板）+ Beam（桩）
│   └── 基础底面作为"地面"参考
├── 高程（Elevation）
│   ├── 构件有 Z 坐标（高程）
│   └── 相对高程，无绝对地理坐标
├── 切割（Cut）
│   ├── BooleanPart / Fitting / CutPlane
│   └── 构件间的几何切割，非地形切割
└── 无 TIN、无平整、无土方、无地形分析
```

### 5.3 对 OmniM 的启示

Tekla 的缺失反而是一个重要验证：**不是所有工程 CAD 都需要地形建模**。OmniM 应将 TerrainModel 作为**可选扩展**，而非强制核心功能。

---

## 6. SolidWorks：无场地/地形概念

### 6.1 机械 CAD 的范式差异

SolidWorks 是纯粹的**机械零件/装配体设计工具**，其建模范式与建筑/土木完全不同：

```
SolidWorks 范式
├── Part（零件）→ .sldprt
│   └── FeatureManager 特征树
├── Assembly（装配体）→ .sldasm
│   └── Mate 配合约束
└── Drawing（工程图）→ .slddrw
    └── 2D 视图 + 标注 + BOM
```

**无以下概念**：
- ❌ 场地（Site）
- ❌ 地形（Terrain）
- ❌ TIN / 等高线
- ❌ 土方计算
- ❌ 平整/边坡
- ❌ 地理坐标/项目基点

### 6.2 机械域中的"类似概念"

SolidWorks 中与场地/地形**功能上类似但本质不同**的概念：

| SolidWorks 概念 | 类似功能 | 本质差异 |
|----------------|---------|---------|
| **Sketch Plane** | 设计的"基面" | 无限平面，非地形曲面 |
| **Top Plane / Front Plane** | 参考坐标面 | 有限平面，无地理含义 |
| **Coordinate System** | 坐标基准 | 笛卡尔坐标系，无地理投影 |
| **Ground Plane (Simulation)** | 仿真中的"地面" | 虚拟平面，仅用于仿真 |
| **Surface Model** | 曲面建模 | 参数化曲面（非 TIN），用于产品外形 |
| **Sheet Metal Base** | 钣金基体 | 平面板材，非地形 |

### 6.3 对 OmniM 的启示

SolidWorks 的缺失进一步验证：**地形建模是土木/建筑/基础设施领域的专属需求**。OmniM 的跨域架构中，TerrainModel 应作为**Civil Domain 的可选扩展模块**，不强制 Mechanical Domain 支持。

---

## 7. 跨平台对比分析

### 7.1 场地组织对比

| 维度 | Revit | Civil 3D | Bentley | E3D | Tekla | SolidWorks |
|------|-------|---------|---------|-----|-------|------------|
| **Site 模型** | IfcSite 空间容器 | 项目空间容器 | Civil Model 容器 | SITE 组织容器 | 无 | 无 |
| **层级结构** | Site→Building→Storey | 项目→地块→设计 | Civil Model→Alignment→Corridor | Project→Site→Area→Unit | 无对应 | Part→Assembly→Drawing |
| **地理坐标** | ✅ Lat/Long + Project Base | ✅ 测量坐标系统 | ✅ 地理坐标系 | ⚠️ 有限支持 | ❌ 无 | ❌ 无 |
| **边界定义** | ✅ Site Boundary | ✅ 地块边界 | ✅ Civil Model 边界 | ⚠️ 区域边界 | ❌ 无 | ❌ 无 |
| **构件归属** | Building 归属 Site | 构件归属地块 | 构件归属 Civil Model | 设备归属 Site/Area | 构件归属 Assembly | Part 归属 Assembly |

### 7.2 地形建模能力对比

| 维度 | Revit | Civil 3D | Bentley | E3D | Tekla | SolidWorks |
|------|-------|---------|---------|-----|-------|------------|
| **TIN 支持** | ✅ 基础 TIN | ✅ 完整 TIN | ✅ 完整 TIN | ❌ 无 | ❌ 无 | ❌ 无 |
| **数据源** | 点/等高线 | 点/等高线/断裂线/曲面 | 点/等高线/断裂线/边界 | 外部引用 | ❌ 无 | ❌ 无 |
| **断裂线** | ❌ 无 | ✅ 完整支持 | ✅ 完整支持 | ❌ 无 | ❌ 无 | ❌ 无 |
| **边界约束** | 基础支持 | ✅ Outer/Benchmark | ✅ 外边界/空洞 | ❌ 无 | ❌ 无 | ❌ 无 |
| **地形编辑** | 点级编辑 | 顶点+断裂线+边界 | 顶点+断裂线+边界 | ❌ 无 | ❌ 无 | ❌ 无 |
| **地形合成** | 基础 | 走廊→曲面 | 走廊→曲面 | ❌ 无 | ❌ 无 | ❌ 无 |

### 7.3 土方与平整能力对比

| 维度 | Revit | Civil 3D | Bentley | E3D | Tekla | SolidWorks |
|------|-------|---------|---------|-----|-------|------------|
| **土方计算** | ❌ 无 | ✅ 三种算法 | ✅ 三种算法 | ❌ 无 | ❌ 无 | ❌ 无 |
| **平整系统** | ❌ 无 | ✅ Grading Group | ⚠️ 通过模板 | ❌ 无 | ❌ 无 | ❌ 无 |
| **边坡设计** | ⚠️ 手动/Dynamo | ✅ 完整支持 | ✅ 完整支持 | ❌ 无 | ❌ 无 | ❌ 无 |
| **Daylight 求交** | ❌ 无 | ✅ 自动求解 | ✅ 自动求解 | ❌ 无 | ❌ 无 | ❌ 无 |
| **土方平衡** | ❌ 无 | ✅ 自动计算 | ✅ 自动计算 | ❌ 无 | ❌ 无 | ❌ 无 |
| **工程量报表** | ⚠️ 基础 | ✅ 详细 | ✅ 详细 | ❌ 无 | ❌ 无 | ❌ 无 |

### 7.4 地形-结构/设施交互对比

| 交互方式 | Revit | Civil 3D | Bentley | E3D | Tekla | SolidWorks |
|---------|-------|---------|---------|-----|-------|------------|
| 建筑→地形 | 基础（自动剪切） | 完整（Target Mapping） | 完整（End Condition） | 无 | 无 | 无 |
| 基础→地形 | 自动剪切 | 精确土方 | 精确土方 | 无 | 简化（Z 坐标） | 无 |
| 管线→地形 | 标高控制 | 埋深/架空 | 埋深/架空 | 地坪标高 | 无 | 无 |
| 道路→地形 | 无原生 | 完整（Corridor+Target） | 完整（Corridor Template） | 无 | 无 | 无 |
| 桥/隧→地形 | 无原生 | 部分 | 完整（End Condition） | 无 | 无 | 无 |
| 边坡→地形 | 无原生 | 完整（Daylight） | 完整（Daylight） | 无 | 无 | 无 |

### 7.5 六平台综合评分

| 平台 | 场地组织 | 地形建模 | 土方平整 | 地形交互 | **综合** |
|------|---------|---------|---------|---------|---------|
| **Civil 3D** | 4/5 | 5/5 | 5/5 | 5/5 | **19/20** |
| **Bentley** | 4/5 | 5/5 | 5/5 | 5/5 | **19/20** |
| **Revit** | 5/5 | 3/5 | 1/5 | 3/5 | **12/20** |
| **E3D** | 3/5 | 1/5 | 0/5 | 1/5 | **5/20** |
| **Tekla** | 1/5 | 0/5 | 0/5 | 1/5 | **2/20** |
| **SolidWorks** | 0/5 | 0/5 | 0/5 | 0/5 | **0/20** |

---

## 8. OmniM 映射与设计建议

### 8.1 OmniM Site/Terrain 构件清单

基于六平台对比，建议 OmniM 在 Site/Terrain 领域支持以下 ObjectType：

| ObjectType | GenRule | 说明 | 优先级 | 主要参考平台 |
|-----------|---------|------|--------|------------|
| **Site** | 无（空间容器） | 地理/空间容器 | P0 | Revit (IfcSite), Civil 3D |
| **TerrainModel** | 独立类型（非 GenRule） | TIN 曲面地形 | P0 | Civil 3D, Bentley |
| **CutVolume** | BooleanSubtract | 挖方量（Existing − Design） | P1 | Civil 3D, Bentley |
| **FillVolume** | BooleanSubtract | 填方量（Design − Existing） | P1 | Civil 3D, Bentley |
| **Slope** | LinearComponent | 边坡（断面沿边缘线扫掠） | P1 | Civil 3D, Bentley |
| **GradingSurface** | TerrainModel（role=design） | 平整设计面 | P1 | Civil 3D |
| **FeatureLine** | 线性元素（无厚度） | 特征线/控制线 | P1 | Civil 3D, Bentley |
| **SiteComponent** | ComponentDef on Terrain | 地形表面附属物 | P2 | Revit |
| **EarthworkBalance** | 逻辑对象 | 土方平衡计算结果 | P2 | Civil 3D, Bentley |
| **ContourLine** | 曲线元素 | 等高线显示 | P2 | Civil 3D, Bentley |

### 8.2 TerrainModel 数据结构建议

```protobuf
message TerrainModel {
  string id = 1;
  string name = 2;
  
  enum Role {
    EXISTING = 0;     // 现有地面
    DESIGN = 1;       // 设计面
    SUBGRADE = 2;     // 路基面
    REFERENCE = 3;     // 参考面（不参与土方）
  }
  Role role = 3;
  
  repeated TerrainVertex vertices = 4;
  repeated TerrainTriangle triangles = 5;
  repeated TerrainBreakline breaklines = 6;
  repeated TerrainBoundary boundaries = 7;
  repeated TerrainHole holes = 8;
  
  // 地理参照
  GeographicReference geo_ref = 9;
  
  // 属性
  map<string, Value> properties = 10;
}

message TerrainVertex {
  string id = 1;
  double x = 2;
  double y = 3;
  double z = 4;
  // 可选：顶点属性（如地表类型、材料）
  map<string, Value> properties = 5;
}

message TerrainTriangle {
  string id = 1;
  uint32 v0_index = 2;
  uint32 v1_index = 3;
  uint32 v2_index = 4;
}

message TerrainBreakline {
  string id = 1;
  repeated uint32 vertex_indices = 2;
  bool enforced = 3;  // 是否强制（三角形边不可跨越）
}

message TerrainBoundary {
  string id = 1;
  repeated uint32 vertex_indices = 2;
  bool is_outer = 3;  // 外边界 vs 内边界
}
```

### 8.3 TerrainModel 核心 API 建议

```
TerrainModel API
├── 创建
│   ├── from_points(vertices[]) → TerrainModel
│   ├── from_contours(contours[], interval) → TerrainModel
│   ├── from_corridor(corridor) → TerrainModel
│   ├── from_plane(elevation, bounds) → TerrainModel
│   └── from_external_reference(file) → TerrainModel
├── 编辑
│   ├── add_vertex(x, y, z) → vertex_id
│   ├── remove_vertex(vertex_id)
│   ├── modify_vertex(vertex_id, x?, y?, z?)
│   ├── add_breakline(vertex_ids[], enforced) → breakline_id
│   ├── add_boundary(vertex_ids[], is_outer) → boundary_id
│   ├── merge(other_terrain)
│   ├── split(boundary) → [TerrainModel]
│   └── smooth(iterations, factor)
├── 查询
│   ├── get_elevation(x, y) → z
│   ├── get_slope(x, y) → slope
│   ├── get_contours(interval) → [ContourLine]
│   ├── get_triangle_count() → int
│   └── get_bounding_box() → Box
├── 分析
│   ├── volume_analysis(other, mode) → {cut, fill, net}
│   ├── surface_area() → double
│   ├── earthwork_balance(target) → {balance, cut, fill, movement}
│   └── daylight_analysis(feature_line, slope_ratio) → [DaylightPoint]
└── 导出
    ├── to_mesh() → Mesh
    ├── to_point_cloud() → PointCloud
    ├── to_contours(interval) → [ContourLine]
    └── to_ifc() → IfcTerrain/IfcSite
```

### 8.4 GenRule 映射补充

| OmniM GenRule | 适用场景 | 地形相关应用 |
|--------------|---------|------------|
| **TerrainModel（独立类型）** | TIN 曲面地形 | 核心地形表达，所有平台共有概念 |
| **BooleanSubtract（曲面-曲面变体）** | 两 TIN 曲面布尔差 | CutVolume / FillVolume 计算 |
| **LinearComponent + ExternalTarget** | 线构件 + 地形约束 | 道路/桥梁/隧道/边坡与地形的 Daylight 求交 |
| **LinearComponent（边坡专用）** | 边坡断面沿边缘线扫掠 | Slope 构件，支持变坡比、马道 |
| **LayerExtrusion（地形铺装）** | 多层截面在地形上铺装 | 路面结构层在地形上的铺设 |
| **Assembly（地形附属）** | 地形表面附属构件 | Site Component（树木、消防栓等） |

### 8.5 跨平台统一架构建议

```
┌──────────────────────────────────────────────────────────────────┐
│                    OmniM 统一场地-地形架构                         │
├──────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  Site（空间容器，无几何）                                 │    │
│  │  ├── 地理坐标系统（GeographicReference）                  │    │
│  │  ├── 边界（Boundary Polygon）                             │    │
│  │  └── 层级：Site → Building / Civil / Plant               │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│                              ▼                                   │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  TerrainModel（独立 ObjectType，非 GenRule）              │    │
│  │  ├── 数据：Vertices + Triangles + Breaklines + Boundaries │    │
│  │  ├── 角色：Existing / Design / Subgrade / Reference      │    │
│  │  ├── 编辑：点/断裂线/边界 CRUD + 合并/分割/平滑          │    │
│  │  └── 分析：体积/土方/边坡/等高线                         │    │
│  └─────────────────────────────────────────────────────────┘    │
│                              │                                   │
│               ┌─────────────┼─────────────┐                     │
│               ▼             ▼             ▼                     │
│  ┌─────────────────┐ ┌─────────────┐ ┌─────────────────────┐   │
│  │  LinearComponent │ │ BooleanSub. │ │  TerrainModel 派生   │   │
│  │  + ExternalTarget│ │  (曲面-曲面) │ │  ├── CutVolume      │   │
│  │  ├── 道路/桥梁   │ │ ├── 挖方量   │ │  ├── FillVolume     │   │
│  │  ├── 隧道/渠道   │ │ └── 填方量   │ │  ├── GradingSurface │   │
│  │  └── 边坡(Slope) │ │             │ │  └── EarthworkBalance│   │
│  └─────────────────┘ └─────────────┘ └─────────────────────┘   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  地形附属：ComponentDef on TerrainSurface                │    │
│  │  ├── SiteComponent（树木、电线杆、消防栓等）              │    │
│  │  ├── FeatureLine（特征线/控制线）                         │    │
│  │  └── ContourLine（等高线显示）                            │    │
│  └─────────────────────────────────────────────────────────┘    │
│                                                                  │
└──────────────────────────────────────────────────────────────────┘
```

### 8.6 优先级路线图

| 阶段 | 内容 | 参考平台 | 理由 |
|------|------|---------|------|
| **Phase 1** | Site ObjectType + TerrainModel（TIN 基础） | Civil 3D, Bentley | 核心抽象，所有土木领域的基础 |
| **Phase 2** | TerrainModel 编辑 API（点/断裂线/边界） | Civil 3D, Bentley | 地形建模的必要操作 |
| **Phase 3** | BooleanSubtract 曲面-曲面变体（Cut/Fill） | Civil 3D, Bentley | 土方计算核心能力 |
| **Phase 4** | LinearComponent + ExternalTarget（Daylight） | Civil 3D, Bentley | 线构件与地形的关键交互 |
| **Phase 5** | GradingSurface + Slope + EarthworkBalance | Civil 3D | 场地平整完整工作流 |
| **Phase 6** | SiteComponent + ContourLine + FeatureLine | Revit, Civil 3D | 场地附属物与显示 |
| **Phase 7** | IFC/LandXML 地形互操作 | IFC 4x3, LandXML | 数据交换标准支持 |

### 8.7 关键设计决策

1. **TerrainModel 作为独立 ObjectType（非 GenRule）**：TIN 曲面不属于 Primitive/LayerExtrusion/LinearComponent/Assembly/BooleanSubtract 五类 GenRule 的任何一类，应作为与 GenRule 并列的独立类型。这一决策已被 Bentley 调研验证（§6.1.4）。

2. **BooleanSubtract 需扩展曲面-曲面变体**：当前 BooleanSubtract 仅支持实体间布尔。Civil 3D 和 Bentley 的土方计算都是基于 TIN 曲面间的布尔差，需扩展支持 "曲面-曲面" 模式。

3. **TerrainModel 的角色系统（Role）**：Existing/Design/Subgrade/Reference 四种角色借鉴自 Bentley Terrain Model 的 Volume Option（§6.1.3），是土方计算和设计工作流的基础。

4. **Daylight 求解器**：边坡与地形的自动求交（Daylight）是 Civil 3D 和 Bentley 的核心能力，OmniM 应实现该求解器作为 `TerrainModel.daylight_analysis()` 方法。

5. **可选扩展，不强制所有域**：Tekla 和 SolidWorks 不支持地形建模，OmniM 应将 TerrainModel 作为 Civil Domain 的可选扩展，不强制 Mechanical Domain 实现。

6. **地理坐标系统**：Site 应支持地理坐标（Lat/Long + 投影），参考 Revit 的 Project Base Point 和 Civil 3D 的测量坐标系统。

### 8.8 与现有原型的整合

| 现有 OmniM 概念 | 整合方式 | 变更说明 |
|----------------|---------|---------|
| `ObjectType::Site` | 保留，补充 TerrainModel 关联 | 增加地理坐标、边界属性 |
| `GenRule::BooleanSubtract` | 扩展曲面-曲面模式 | 新增 `surface_surface` 布尔变体 |
| `GenRule::LinearComponent` | 增加 `ExternalTarget` 约束 | 支持 Daylight 目标 |
| `create site` 命令 | 扩展 `--bounds` + 地理坐标 | 支持场地边界和坐标指定 |
| Component Inventory | 新增 Terrain/CutVolume/FillVolume/Slope | 补充 Civil Domain 构件 |

---

## 9. IFC 标准映射

### 9.1 IFC 4x3 中的 Site/Terrain

| OmniM ObjectType | IFC 4x3 映射 | 说明 |
|-----------------|-------------|------|
| Site | IfcSite | 直接映射 |
| TerrainModel | IfcGeographicElement（提议） | IFC 4x3 新增的地理元素类型 |
| CutVolume / FillVolume | IfcGeographicElement（土方） | 自定义类型或 Quantity |
| Slope | IfcGeographicElement（边坡） | 自定义类型 |
| GradingSurface | IfcGeographicElement（设计面） | 自定义类型 |
| SiteComponent | IfcElement（场地附属物） | 如 IfcLightFixture、IfcCarpet（景观） |

### 9.2 LandXML 互操作

Civil 3D 和 Bentley 都支持 LandXML 格式的地形数据交换：

| LandXML 元素 | OmniM 映射 |
|-------------|-----------|
| `Surfaces` | `TerrainModel` |
| `Surface` | `TerrainModel`（单曲面） |
| `Definition` | `TerrainVertex` + `TerrainTriangle` |
| `Breaklines` | `TerrainBreakline` |
| `Boundary` | `TerrainBoundary` |
| `Grade` | `GradingSurface` |

---

## 10. 总结

### 10.1 核心发现

1. **Civil 3D 和 Bentley 是场地/地形建模的标杆**——两者都实现了完整的 TIN 地形、土方计算、平整系统和地形-结构交互。OmniM 的设计应主要参考这两个平台。

2. **Revit 的场地定位最清晰**——Site 作为 IfcSite 空间容器的定位准确，但地形建模能力有限。OmniM 应继承 Revit 的 Site 定位，同时补充 Civil 3D/Bentley 的地形能力。

3. **E3D 验证了 Site 作为逻辑容器的另一种用法**——在工厂设计中，Site 主要是组织/权限容器，无地形要求。OmniM 的 Site 应同时支持几何容器和逻辑容器两种模式。

4. **Tekla 和 SolidWorks 验证了地形建模的非普遍性**——不是所有工程 CAD 都需要地形支持。OmniM 应将 TerrainModel 作为可选扩展。

5. **TerrainModel 应作为独立 ObjectType（非 GenRule）**——TIN 曲面不属于任何现有 GenRule 类别，应独立建模。

### 10.2 六平台场地/地形能力矩阵

| 能力维度 | Revit | Civil 3D | Bentley | E3D | Tekla | SolidWorks | OmniM 目标 |
|---------|-------|---------|---------|-----|-------|------------|-----------|
| Site 空间容器 | ✅ | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ P0 |
| TIN 地形 | ⚠️ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ P0 |
| TIN 编辑 | ⚠️ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ P1 |
| 土方计算 | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ P1 |
| 场地平整 | ❌ | ✅ | ⚠️ | ❌ | ❌ | ❌ | ✅ P2 |
| 边坡 Daylight | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ P2 |
| 地形-结构交互 | ⚠️ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ P1 |
| 地理坐标 | ✅ | ✅ | ✅ | ⚠️ | ❌ | ❌ | ✅ P1 |
| IFC 互操作 | ✅ | ✅ | ⚠️ | ❌ | ❌ | ❌ | ✅ P2 |
| LandXML 互操作 | ❌ | ✅ | ✅ | ❌ | ❌ | ❌ | ✅ P3 |

### 10.3 设计建议优先级

| 优先级 | 功能 | 核心参考 | 依赖 |
|--------|------|---------|------|
| **P0** | Site ObjectType（空间容器） | Revit IfcSite | 无 |
| **P0** | TerrainModel 基础（TIN 数据结构） | Civil 3D/Bentley TIN | 地理坐标系统 |
| **P1** | TerrainModel 编辑 API | Civil 3D/Bentley | TerrainModel P0 |
| **P1** | BooleanSubtract 曲面-曲面变体 | Civil 3D/Bentley CutFill | TerrainModel + BooleanSubtract |
| **P1** | Terrain-结构交互（Building/Terrain 剪切） | Revit 基础 | TerrainModel |
| **P2** | Daylight 求解器 | Civil 3D/Bentley | TerrainModel + LinearComponent |
| **P2** | GradingSurface + EarthworkBalance | Civil 3D | TerrainModel + BooleanSubtract |
| **P2** | Slope（边坡） | Civil 3D/Bentley | LinearComponent + Daylight |
| **P3** | SiteComponent（地形附属物） | Revit | TerrainModel + ComponentDef |
| **P3** | ContourLine / FeatureLine | Civil 3D/Bentley | TerrainModel |
| **P3** | IFC/LandXML 互操作 | Civil 3D/Bentley | TerrainModel |