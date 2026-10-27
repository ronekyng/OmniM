# Civil 3D 内置构件底层逻辑调研

> 调研日期：2026-07-09
> 来源：Autodesk Civil 3D 2026 Help、Subassembly Composer 文档、SDC Publications

---

## 1. 概述

Civil 3D 是面向土木工程（道路、场地、管线）的 BIM 平台。其内置构件的核心逻辑围绕 **点/线/面** 几何图元展开，通过 **装配 (Assembly) → 走廊 (Corridor) → 横断面 (Section)** 三层架构驱动三维模型生成。

---

## 2. 核心概念：Subassembly（子装配）

Subassembly 是 Civil 3D 最基本的构件单元，代表道路横断面的一个组成部分（如车道、路缘石、人行道、边坡）。

### 2.1 几何构成

每个 Subassembly 由三种基本几何元素构成：

```
Subassembly
├── Point（点）—— 用于标注、定位、连接
│   ├── P1: 行车道边缘
│   ├── P2: 路缘石后沿
│   └── P3: 人行道外缘
├── Link（链接）—— 代表潜在表面，用于坡度/高程标注
│   ├── L1: 路面层
│   ├── L2: 路缘石
│   └── L3: 人行道
└── Shape（形状）—— 整个子装配的闭合区域
    ├── S1: 路面结构
    └── S2: 路基
```

### 2.2 点/链接/形状编码系统

Civil 3D 使用编码系统标识每个几何元素的用途：

| 编码类型 | 前缀 | 用途 | 示例 |
|---------|------|------|------|
| 点编码 | `ETW` | 行车道边缘 | Edge of Travel Way |
| 点编码 | `BOC` | 路缘石后沿 | Back of Curb |
| 点编码 | `Daylight` | 边坡坡脚 | Daylight point |
| 链接编码 | `Top` | 顶面 | Pavement Top |
| 链接编码 | `Datum` | 基准面 | Subgrade Datum |
| 链接编码 | `Pave` | 路面结构 | Pavement structure |
| 形状编码 | `Pavement` | 路面 | Pavement structure |
| 形状编码 | `Sidewalk` | 人行道 | Sidewalk area |

### 2.3 参数化定义

Subassembly 通过参数控制几何形状：

```csharp
// 示例：BasicLane 子装配参数
public class BasicLaneParameters {
    double Width;         // 车道宽度
    double Slope;         // 横坡 (%)
    double Thickness;     // 路面厚度
    SideType Side;        // 左侧/右侧
    bool UseSuperelevation; // 使用超高
}
```

---

## 3. Assembly（装配）

Assembly 是横断面的完整模板，由多个 Subassembly 沿中心线（基准线）对称或非对称排列。

```
          Assembly
┌─────────────────────────────────────┐
│  ┌──┐  ┌──────────┐  ┌──────────┐  │
│  │CW│  │   Lane   │  │   Lane   │  │  ← 子装配
│  └──┘  └──────────┘  └──────────┘  │
│  ┌────┐          ┌────┐            │
│  │SW  │          │  SW│            │  ← 子装配
│  └────┘          └────┘            │
│     ▲                              │
│     └── 基准线 (Baseline)           │
└─────────────────────────────────────┘
```

### 3.1 装配构建流程

1. 创建 Assembly 对象（指定基准线位置）
2. 附着 Subassembly 到基准线或已有 Subassembly 的连接点
3. 设置 Subassembly 参数（宽度、坡度、厚度等）
4. 定义目标映射（Target Mapping）——关联到曲面、对齐、纵断面等

---

## 4. Corridor（走廊模型）

走廊是 Civil 3D 最核心的 3D 建模机制，将 Assembly 沿 Alignment（平面线形）+ Profile（纵断面）扫掠生成三维实体。

### 4.1 数据流

```
Alignment (平面线形) ──┐
                       ├──→ Corridor ──→ 3D Solid
Profile (纵断面)     ──┘       │
                               ├──→ Corridor Surface
Assembly (横断面装配) ────      ├──→ Section Views
                               └──→ Quantity Takeoff
```

### 4.2 参数化控制

```
Corridor
├── Baselines（基准线，可多条，如主线、匝道）
│   ├── Alignment（平面线形）
│   ├── Profile（纵断面）
│   └── Baseline Regions（区域划分）
│       ├── Start Station (起始桩号)
│       ├── End Station (终止桩号)
│       └── Assembly（横断面模板）
└── Targets（目标映射）
    ├── Surface Target（曲面目标——边坡到达地面）
    ├── Offset Target（偏移目标——对齐到特征线）
    ├── Elevation Target（高程目标——对齐到纵断面）
    └── Width Target（宽度目标——根据条件变化）
```

### 4.3 频率控制

```python
# 采样频率设置
frequency_settings = {
    "Along_Curves": 1.0,    # 曲线上 1m 采样
    "Along_Tangents": 5.0,  # 直线上 5m 采样
    "At_Profile_High_Low": True,  # 纵断面极值点
    "At_Superelevation_Critical": True,  # 超高关键点
}
```

---

## 5. Subassembly Composer（子装配编辑器）

SAC 是 Civil 3D 的可视化子装配编程工具，使用流程图方式定义子装配的逻辑和几何。

### 5.1 流程图节点类型

| 节点类型 | 功能 | 说明 |
|---------|------|------|
| **Geometry** | 几何定义 | 创建点、链接、形状 |
| **Decision** | 条件判断 | 基于参数值选择不同路径 |
| **Switch** | 多路分支 | 枚举类型切换 |
| **Sequence** | 顺序执行 | 组织几何节点顺序 |
| **Input/Output** | 参数定义 | 暴露可调参数 |
| **Target** | 目标映射 | 关联外部对象 |

### 5.2 几何节点 API

```python
# 示例：创建车道子装配的几何节点
P1 = Point.Create(0, 0)          # 基准点
P2 = Point.Create(Width, 0)      # 车道边缘
P3 = Point.Create(Width, Slope)  # 车道边缘高程
P4 = Point.Create(0, Slope)      # 基准线高程

L1 = Link.Create(P1, P2)         # 底面
L2 = Link.Create(P2, P3)         # 边线
L3 = Link.Create(P3, P4)         # 顶面
L4 = Link.Create(P4, P1)         # 边线

Shape.Create([L1, L2, L3, L4])   # 闭合形状
```

### 5.3 条件逻辑示例

```python
# 条件逻辑：根据环境类型选择不同构造
if CurbType == "Urban":
    # 城市道路：包含人行道
    Create_Sidewalk()
    Create_Curb_and_Gutter()
else:
    # 公路：直接边坡
    Create_Daylight()
```

---

## 6. Pipe Network（管网系统）

### 6.1 重力管网 (Gravity Pipe Network)

```
Pipe Network
├── Parts List（部件列表）
│   ├── Pipes（管道）
│   │   ├── Circular Concrete Pipe
│   │   ├── Elliptical Pipe
│   │   └── Box Culvert
│   └── Structures（结构）
│       ├── Manhole
│       ├── Junction Box
│       └── Catch Basin
├── Pipe Runs（管线）
│   ├── Parts（管道段）
│   │   ├── Geometry（几何）
│   │   │   ├── Start/End Invert Elevation
│   │   │   ├── Diameter/Shape
│   │   │   ├── Length/Slope
│   │   │   └── Horizontal/Vertical Curvature
│   │   └── Connectors（连接点）
│   └── Structures（检查井等）
└── Rules（布管规则）
    ├── Minimum Cover
    ├── Minimum Slope
    └── Maximum Depth
```

### 6.2 压力管网 (Pressure Pipe Network)

Civil 3D 2021+ 中压力管网重构为**路径驱动**模式：

```csharp
// 压力管道核心属性
public class PressurePipe {
    double StartElevation;        // 起点中心高程
    double EndElevation;          // 终点中心高程
    double NominalDiameter;       // 公称直径
    double OuterDiameter;         // 外径
    double WallThickness;         // 壁厚
    string Material;              // 材料
    double PressureClass;         // 压力等级
    Alignment ReferenceAlignment; // 参考线形
    Profile ReferenceProfile;    // 参考纵断面
}
```

### 6.3 Pipe Runs 管线编辑

```csharp
// 管线编辑操作
PipeRunCommands
├── ExtendPipeRun()       // 延伸管线
├── AddToPipeRun()        // 添加到管线
├── SplitPipeRun()        // 拆分管线
├── MergePipeRuns()       // 合并管线
├── ReversePipeRun()      // 反转方向
└── StructureSwap()       // 替换结构
```

---

## 7. 构件分类体系

### 7.1 Subassembly 分类

| 类别 | 说明 | 典型子装配 |
|------|------|-----------|
| Basic | 基础构件 | BasicLane, BasicCurb, BasicSidewalk |
| Roadway | 道路构件 | LaneSuperelevation, ShoulderExtend |
| Structures | 结构构件 | RetainingWall, JerseyBarrier |
| Daylight | 边坡构件 | DaylightBasin, DaylightStandard |
| Generic | 通用构件 | LinkOffsetOnSurface, PointOnSurface |

### 7.2 管件分类

```xml
<PartsList>
  <GravityPipes>
    <Circular>Concrete, PVC, HDPE</Circular>
    <Elliptical>Concrete, Steel</Elliptical>
    <Box>Concrete</Box>
  </GravityPipes>
  <PressurePipes>
    <DuctileIron>CIP, CML</DuctileIron>
    <Steel>AWWA, API</Steel>
    <PVC>Schedule 40, 80</PVC>
  </PressurePipes>
  <Structures>
    <Manhole>Precast, Cast-in-place</Manhole>
    <CatchBasin>Standard, Type C</CatchBasin>
  </Structures>
</PartsList>
```

---

## 8. 对 OmniM 的启示

| Civil 3D 概念 | OmniM 等价概念 | 借鉴点 |
|---------------|---------------|--------|
| Subassembly (点/链接/形状) | ComponentDef + Feature | 横断面层结构映射到 LayerExtrusion |
| Assembly (横断面模板) | LinearComponentGen | 线构件横断面定义 |
| Corridor (Alignment + Profile + Assembly) | Alignment3D + CrossSectionLayer | 路径+变化横断面完整模型 |
| Target Mapping (曲面/特征线) | TerrainTarget + External Constraints | 外部约束系统 |
| Subassembly Composer (流程图) | GenRule + VariationRule | 条件逻辑驱动的几何生成 |
| Pipe Runs | Series + Connector | 管线/系列库概念 |
| Point/Link/Shape Codes | ConnectorType | 连接点类型编码 |

**核心结论**：Civil 3D 的"线构件"（LinearComponent）思想是 OmniM 的 LinearComponentGen 的直接参考。其 Subassembly 的"点/链接/形状"编码系统与 OmniM 的 Connector 系统高度对应，Target Mapping 概念对应 OmniM 的约束求解器。