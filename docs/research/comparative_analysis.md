# 跨平台内置构件逻辑对比分析与 OmniM 映射

> 初版调研日期：2026-07-09（Civil 3D / Revit / E3D 三平台）
> 扩展调研日期：2026-08-07（补充 SolidWorks / Bentley / Tekla / E3D Instrumentation 四平台，共七平台）
> 目标：对比七款主流工程软件的内置构件底层逻辑，映射到 OmniM 的 GenRule 体系，验证统一几何内核的可行性

---

## 1. 整体架构对比

| 维度 | Revit (建筑) | Civil 3D (土木) | AVEVA E3D (工厂) | Bentley (基建) | Tekla (钢结构) | SolidWorks (机械) | OmniM |
|------|-------------|----------------|-----------------|---------------|---------------|------------------|-------|
| 核心哲学 | 族参数化 | 走廊横断面 | 基本体拼装 | 走廊模板+Civil Cell | 零件+组件节点 | 特征树+配置 | 统一几何+分层语义 |
| 基本体素 | Extrusion/Sweep/Blend | Point/Link/Shape | BOX/CYLI/CONE/NOZZ | Corridor/Linear/Surface Template | Beam/ContourPlate/Brep | Extrude/Revolve/Sweep/Loft | Box/Cylinder/Cone/Extrusion/Sweep/Revolve/Loft |
| 构件封装 | Family/Symbol | Subassembly | Primitive + Template | Template Drop + Civil Cell | Part + Component | Part + Assembly | ComponentDef |
| 参数化 | 族参数 | 子装配参数 | 基本体参数 | 模板变量+变量表 | 截面+位置+组件输入 | 草图尺寸+设计表 | Feature 参数 |
| 连接机制 | 无显式连接系统 | 点编码 | P-Point 系统 | Point Code+Target | Component Input(Primary/Secondaries) | Mate 配合 | Connector 系统 |
| 布尔运算 | 空心/实心族 | 无 | 负实体 | Cut/Fill+Trimming | BooleanPart/Fitting/CutPlane | Extrude Cut | BooleanSubtract |
| 变截面 | 族类型切换 | VariationRules | 无原生支持 | 6 种 Variation+Beam Rows | profile 切换 | 配置切换 | VariationRule |
| 层次结构 | Element/Type/Instance | Subassembly/Assembly/Corridor | EQUI/SUBE/Primitive | Template/Corridor/Cell | Part/Assembly/Component | Part/Assembly/SubAssembly | Object/Component/Feature/Sketch |

---

## 2. 面构件对比 (Layer/Surface-based)

### 2.1 Revit Wall

```
Revit Wall
├── 定位线 (Curve)
├── 层结构 (WallType.Structure)
│   ├── Layer 1: 砖饰面 (10mm)
│   ├── Layer 2: 保温层 (50mm)
│   └── Layer 3: 混凝土 (200mm)
├── 高度约束 (Top/Bottom)
└── 开洞 (Door/Window)
```

### 2.2 Civil 3D Pavement

```
Civil 3D Pavement
├── 横断面 (Subassembly)
│   ├── 层 1: 面层 (厚度 50mm)
│   ├── 层 2: 基层 (厚度 200mm)
│   └── 层 3: 底基层 (厚度 300mm)
├── Alignment (平面路径)
├── Profile (纵断面高程)
└── Targets (曲面目标 → 边坡)
```

### 2.3 E3D Panel (PANE)

```
E3D Panel
├── 轮廓 (Profile)
├── 厚度 (Thickness)
├── 定位 (Position)
└── 方向 (Orientation)
```

### 2.4 OmniM LayerExtrusionGen 统一映射

```
OmniM LayerExtrusionGen
├── MainPath (路径 — 对应 Wall 定位线/Corridor Alignment)
├── Layers (层结构 — 统一 Revit 层结构 + Civil 3D 路面结构)
│   ├── Layer 1: {name, thickness, material, function}
│   ├── Layer 2: {name, thickness, material, function}
│   └── Layer 3: {name, thickness, material, function}
├── HeightConstraint (高度约束 — Revit Top/Bottom)
├── TerrainTarget (地形目标 — Civil 3D Target)
├── Openings (开洞 — Revit Door/Window)
└── VariationRules (变化规则 — 变截面板)
```

**验证案例**：

| 对象 | Revit 等价 | Civil 3D 等价 | E3D 等价 |
|------|-----------|--------------|---------|
| 建筑墙 (Wall) | Wall | RetainingWall | PANE |
| 楼板 (Slab) | Floor | Pavement | PANE |
| 道路路面 (Road) | — | PavementStructure | — |
| 挡土墙 | Wall | RetainingWall | PANE + GENSEC |

---

## 3. 线构件对比 (Linear/Curve-based)

### 3.1 Revit Beam/Column

```
Revit Beam
├── 定位线 (Curve) — 两点定义
├── 族类型 (FamilySymbol) — 截面尺寸
│   ├── 宽度 (Width)
│   ├── 高度 (Height)
│   └── 材质 (Material)
├── 结构类型 (StructuralType.Beam)
└── 标高 (Level)
```

### 3.2 Civil 3D Corridor

```
Civil 3D Corridor
├── Alignment (平面线形)
├── Profile (纵断面)
├── Assembly (横断面模板)
│   ├── Subassembly 1: Lane (宽度 3.5m)
│   ├── Subassembly 2: Curb (高度 150mm)
│   └── Subassembly 3: Daylight (坡度 1:1.5)
├── Targets (外部目标)
│   ├── Surface → 边坡到达地面
│   └── Feature Line → 偏移控制
└── VariationRules (变化规则)
    ├── Start Station: 0+000, Width: 3.5m
    └── End Station: 1+000, Width: 3.0m
```

### 3.3 E3D PIPE/BRANCH

```
E3D BRANCH
├── Head (起点) → 起始管嘴
├── Tail (终点) → 终止管嘴
├── Components (元件序列)
│   ├── FLAN (法兰)
│   ├── VALV (阀门)
│   ├── PIPE (直管段)
│   ├── ELBO (弯头)
│   └── TEE (三通)
├── Routing (布管路径)
└── Spec (等级规范)
```

### 3.4 OmniM LinearComponentGen 统一映射

```
OmniM LinearComponentGen
├── MainPath (主路径 — 对应 Beam 定位线/Corridor Alignment/BRANCH 路径)
├── CrossSectionLayers (横断面层 — 对应 Civil 3D Assembly Layers)
│   ├── Layer 1: {name, width, thickness, material}
│   └── Layer 2: {name, width, thickness, material}
├── VariationRules (变化规则 — 对应 Civil 3D VariationRules)
│   ├── StartStation, EndStation, Parameter, Value
│   └── 支持变截面: 梁的变截面、道路宽度变化
├── Components (元件序列 — 对应 E3D BRANCH Components)
│   └── 支撑管道系统元件链
├── EdgePaths (边路径 — 约束对齐)
├── TerrainTarget (地形目标 — 对应 Civil 3D Surface Target)
└── ExternalConstraints (外部约束)
```

**验证案例**：

| 对象 | Revit 等价 | Civil 3D 等价 | E3D 等价 |
|------|-----------|--------------|---------|
| 梁 (Beam) | Beam | — | GENSEC |
| 柱 (Column) | Column | — | CYLI/GENSEC |
| 管道 (Pipe) | Pipe | PipeNetwork | BRANCH |
| 道路 (Road) | — | Corridor | — |
| 桥梁 (Bridge) | — | Corridor+Sub | — |

---

## 4. 体构件对比 (Primitive/Solid-based)

### 4.1 Revit GenericForm

```
Revit GenericForm
├── Extrusion (拉伸) — 轮廓 + 方向 + 深度
├── Sweep (放样) — 路径 + 轮廓
├── Revolve (旋转) — 轮廓 + 轴 + 角度
├── Blend (融合) — 顶轮廓 + 底轮廓
├── SweptBlend (放样融合) — 路径 + 顶/底轮廓
└── Loft (放样) — 多个轮廓
```

### 4.2 E3D Primitives

```
E3D Primitives
├── CYLI (圆柱) — 直径 + 高度
├── BOX (立方体) — X/Y/Z 尺寸
├── CONE (圆台) — 顶径 + 底径 + 高度
├── SNOUT (偏心圆台) — 顶径 + 底径 + 高度 + 偏心距
├── PYRA (棱台) — 顶部/底部 X/Y + 高度
├── DISH (碟形封头) — 直径 + 深度
├── SPHR (球体) — 直径
├── TORUS (圆环) — 主/次半径
├── EXTR (拉伸体) — 轮廓 + 拉伸
└── REVO (旋转体) — 轮廓 + 轴
```

### 4.3 Civil 3D Parts

```
Civil 3D Parts
├── Pipes (管道)
│   ├── Circular (圆形)
│   ├── Elliptical (椭圆形)
│   └── Box (矩形)
├── Structures (结构)
│   ├── Manhole (检查井)
│   ├── CatchBasin (雨水口)
│   └── JunctionBox (接线盒)
└── Pressure Fittings (压力管件)
    ├── Bend (弯头)
    ├── Tee (三通)
    └── Reducer (变径)
```

### 4.4 OmniM PrimitiveGen/AssemblyGen 统一映射

```protobuf
// 体构件 — 直接图元
message PrimitiveGen {
  // 参数直接映射到基本体
  // 适用于: 泵、阀、齿轮、管件
  GeometryPrimitiveType type;  // BOX, CYLI, CONE, SPHR, etc.
  map<string, double> params;  // 参数映射
}

// 组合装配 — 多个图元组合
message AssemblyGen {
  // 适用于: 设备、装配体、复杂管件
  repeated PrimitiveAssembly members;  // 子图元列表
  repeated BooleanOperation bools;     // 布尔操作
  repeated Connector connectors;       // 连接点
}
```

**验证案例**：

| 对象 | Revit 等价 | Civil 3D 等价 | E3D 等价 |
|------|-----------|--------------|---------|
| 泵 (Pump) | FamilyInstance | — | EQUI + Primitives |
| 阀门 (Valve) | FamilyInstance | PipePart | VALV |
| 储罐 (Tank) | FamilyInstance | — | EQUI + CYLI + DISH |
| 检查井 (Manhole) | FamilyInstance | Manhole | — |

---

## 5. 五种 GenRule 的跨平台验证

### 5.1 PrimitiveGen — 直接图元

| 来源 | 示例 | 参数 |
|------|------|------|
| Revit | 族中的 Extrusion | 轮廓+深度 |
| Civil 3D | 管件 Parts | 直径+壁厚 |
| E3D | CYLI/BOX/CONE | 直径+高度/XYZ |

### 5.2 LayerExtrusionGen — 层拉伸

| 来源 | 示例 | 参数 |
|------|------|------|
| Revit | Wall/Floor/Roof | 定位线+层结构+高度 |
| Civil 3D | PavementStructure | 横断面层+厚度 |
| E3D | PANE | 轮廓+厚度 |

### 5.3 LinearComponentGen — 线构件

| 来源 | 示例 | 参数 |
|------|------|------|
| Revit | Beam/Column | 定位线+截面 |
| Civil 3D | Corridor | 平面线形+纵断面+横断面模板 |
| E3D | BRANCH | 路径+元件序列 |

### 5.4 AssemblyGen — 组合装配

| 来源 | 示例 | 参数 |
|------|------|------|
| Revit | 嵌套族 | 子族+约束 |
| Civil 3D | Assembly | 多个 Subassembly |
| E3D | EQUI | 多个 Primitive |

### 5.5 BooleanSubtractGen — 布尔剪切

| 来源 | 示例 | 参数 |
|------|------|------|
| Revit | 空心族 (Void) | 布尔差集 |
| E3D | Negative Primitive | 负实体 |
| Civil 3D | — | 不直接支持 |

---

## 6. 连接器系统对比

| 系统 | Civil 3D | Revit | E3D | OmniM |
|------|---------|-------|-----|-------|
| 连接点 | Point Code | Connector Element | P-Point | Connector |
| 方向性 | 从点编码推断 | 通过 Connector 方向 | 固定 P0-P6 | Direction + Axis |
| 类型编码 | ETW/BOC/Daylight | 系统分类 | 管嘴/法兰 | ConnectorType Enum |
| 连接逻辑 | Target Mapping | 物理连接 | Head/Tail | ConnectorPair |
| 层级 | Subassembly 内部 | 族内部 | 基本体之间 | Component/Feature 之间 |

---

## 7. 参数化/属性系统对比

| 特性 | Revit | Civil 3D | E3D | OmniM |
|------|-------|---------|-----|-------|
| 类型参数 | FamilySymbol | Subassembly Param | Spec Parameter | Series |
| 实例参数 | Instance Param | Corridor Param | Position/Orientation | Component Property |
| 属性集 | PropertySet | Extended Data | Attribute | PropertySet |
| 材质 | Material | Material | MatSpec | MaterialId |
| 约束 | Dimension Constraint | Design Rule | Spec Rule | ConstraintSet |

---

## 8. 核心结论与设计建议

### 8.1 统一几何内核的可行性

**已验证**：Civil 3D、Revit、E3D 三者的几何图元可以统一到 OmniM 的 7 种基本几何体（Box/Cylinder/Cone/Sphere/Extrusion/Sweep/Revolve） + 布尔运算。这是 OmniM 统一几何内核的理论基础。

### 8.2 五种 GenRule 的覆盖性

**已验证**：5 种 GenRule 可以覆盖三个平台的所有内置构件类型：

| GenRule | 覆盖数 | 未覆盖 |
|---------|--------|--------|
| PrimitiveGen | 中等 | 需扩展至 15+ 基本体类型 |
| LayerExtrusionGen | 高 | 几乎全覆盖 |
| LinearComponentGen | 高 | 超过 Civil 3D Corridor 的表达力 |
| AssemblyGen | 高 | 支持嵌套组合 |
| BooleanSubtractGen | 中 | Civil 3D 不直接支持 |

### 8.3 关键设计建议

1. **Connection 系统应优先参考 E3D P-Point**：P-Point 系统成熟、稳定，且与 Connector 概念高度一致。

2. **LinearComponent 应参考 Civil 3D Corridor**：Corridor 的"路径+横断面+变化规则"模型是线构件的最完整实现。

3. **LayerExtrusion 应参考 Revit Wall**：Wall 的"定位线+层结构+开洞"模式简洁完整。

4. **Catalog 系统应参考 E3D Spec**：E3D 的元件目录+等级库体系是工业级最成熟的实现。

5. **参数化变截面应参考 SAC VariationRules**：Subassembly Composer 的流程图逻辑+条件分支是变截面设计的最佳实践。

### 8.4 三种设计范式的统一

```
┌─────────────────────────────────────────────────────────────┐
│                    OmniM 统一几何内核                        │
│  Box  Cylinder  Cone  Sphere  Extrusion  Sweep  Revolve    │
│                    Union  Subtract  Intersect                │
├─────────────────────────────────────────────────────────────┤
│                    GenRule 分类层                            │
│  PrimitiveGen  LayerExtrusionGen  LinearComponentGen        │
│  AssemblyGen  BooleanSubtractGen                            │
├─────────────────────────────────────────────────────────────┤
│                    三种设计范式                              │
├─────────────────┬─────────────────┬─────────────────────────┤
│  Revit 式        │  Civil 3D 式    │  E3D 式                 │
│  (语义驱动)      │  (线构件驱动)    │  (基本体+目录驱动)       │
│                  │                 │                         │
│  Wall            │  Corridor       │  EQUI                   │
│  Floor           │  PipeNetwork    │    + CYLI               │
│  Beam            │  Assembly       │    + BOX                │
│  族参数化         │  横断面+路径     │    + NOZZ               │
│  层结构           │  目标映射        │    + 目录规范            │
│  开洞             │  变化规则        │    + P-Point            │
└─────────────────┴─────────────────┴─────────────────────────┘
```

**核心结论**：OmniM 的"统一几何内核 + 5 种 GenRule 分类 + 三种设计范式"架构设计合理，Civil 3D、Revit、E3D 的调研结果都验证了这一架构的可行性。三种设计范式可以统一在同一个几何底层之上，差异仅在于 GenRule 的复杂度和面向用户的交互方式。

---

## 9. 扩展平台调研补充（2026-08-07）

> 在初版三平台（Revit/Civil3D/E3D）基础上，补充 SolidWorks / Bentley / Tekla / E3D Instrumentation 四平台调研，将 GenRule 验证从 3 平台扩展到 7 平台。

### 9.1 SolidWorks（机械域）— 特征树范式

> 本节基于 SOLIDWORKS 2026 官方文档（help.solidworks.com/2026/）重新调研，修正了初版基于通用知识的多处臆造内容。

| 关键发现 | 对 OmniM 的启示 |
|---------|---------------|
| Part/Assembly/SubAssembly 三文档体系 | 验证 Primitive（Part）+ Assembly（Assembly/SubAssembly）二分 |
| FeatureManager 特征树（Extrude/Revolve/Sweep/Loft/Boundary 五类草图特征 + Hole Wizard 7 类孔 + Fillet 8 子类型 + Chamfer + Pattern） | 验证 Feature 作为 Primitive 之下的几何构造单元；**Loft 与 Boundary 是独立特征**（Boundary 质量通常更高），OmniM Primitive::Loft 应支持两种算法变体 |
| **Toolbox 官方仅含 15 类硬件**（Bolts/Nuts/Washers/Screws/Pins/Bearings/Gears/Cams/Sprockets/Timing belt pulleys/Retaining rings/Jig bushings/PEM inserts/Structural shapes/Unistrut），支持 11 标准（ANSI/AS/GB/BSI/CISC/DIN/ISO/IS/JIS/KS） | 验证 Catalog ComponentDef 库；**重要更正**：Toolbox 不含铆钉/密封件/弹簧/键/联轴器/衬套/管件（初版臆造）；齿轮非真渐开线（仅设计示意），紧固件无精确螺纹（不适合应力分析） |
| 配置（Configuration）+ 设计表（Design Table，Excel 驱动）| 验证 SeriesStore 作为独立模块的必要性；设计表是 Series Store 的工业原型（列=选择器，行=变体） |
| **21 类 Mates**（标准 8 + 高级 6 + 机械 7，含 Profile Center/Cam/Slot/Path Mate/Linear-Linear Coupler） | 验证 Constraint/Connector 概念；机械域 Mate=OmniM Connector，应完整支持 21 类 |
| MBD/DimXpert（按 ASME Y14.5.1M-1994 自动生成尺寸与几何公差）+ STEP AP242 PMI 导入 | OmniM 应支持 PMI/GD&T 标注作为 Feature 附加 |
| Weldments（结构构件型材沿路径扫掠 + 焊缝 + 加强板） | **LinearComponent 在机械域的体现**（窄应用）：型材截面沿路径扫掠，与 Tekla Beam 几何等价但应用场景为机械框架 |
| Sheet Metal（Base/Edge/Miter/Swept Flange + 折弯系数） | LayerExtrusion 在机械域的体现（钣金单层折弯）；需独立 SheetMetal Feature 支持展开/折叠 |
| 多实体零件（Part 内多实体 + Combine/Move/Copy + Feature Scope） | Part 与 Assembly 的中间形态，OmniM ObjectType::Part 应支持多实体 |

### 9.2 Bentley OpenBridge/OpenRoads/OpenTunnel（基建域）— 走廊模板范式

| 关键发现 | 对 OmniM 的启示 |
|---------|---------------|
| 走廊模板（Corridor Template）= 2D 横断面沿 3D 路径扫掠 | **LinearComponent 是基建域核心 GenRule**，覆盖 80%+ 构件（Girder/Deck/Lining/Channel/Dam/Slope） |
| 6 种 Variation 插值（None/Linear/Parabolic±/Circular±） | OmniM VariationRule 需扩展至 6 种插值方法（当前可能仅支持 Linear） |
| Beam Rows + Different End Template 分段变截面 | 验证 `ComponentSequenceItem` 段内插值+段间跳跃设计；OTD 明确不支持全局 loft |
| ExternalTarget 外部约束（End Condition Daylight 到地形） | LinearComponent 需支持外部约束目标（曲面/特征线） |
| Civil Cell 范式（预封装几何+模板+地形+约束） | 验证 Assembly 应支持表达式驱动的参照点（子件位置引用外部对象） |
| Terrain Model（TIN 曲面）作为独立类型 | **建议新增 TerrainModel 作为独立 ObjectType**，与 5 类 GenRule 并列（非 GenRule） |
| CutVolume/FillVolume = 两 TIN 曲面布尔差 | BooleanSubtract 需支持曲面间布尔，不仅实体间 |

### 9.3 Tekla Structures（钢结构域）— 零件+组件节点范式

| 关键发现 | 对 OmniM 的启示 |
|---------|---------------|
| Beam 类统一所有线构件（梁/柱/支撑/檩条/墙梁/桩），仅 BeamTypeEnum + 截面 + 位置区分 | **建议合并 Column 到 Beam 作为"用途标签"**（与 Tekla 一致），减少 ObjectType 冗余 |
| ContourPlate = 2D 多边形 + 板厚拉伸 | Primitive::Extrusion 覆盖节点板/底板/加劲肋/独立基础 |
| LoftedPlate = 双 Polycurve 导线放样 | **建议新增 Primitive::Loft**（双导线放样） |
| Connection/Component/Detail/Seam 过程化节点 | **Tekla 组件系统是 OmniM Assembly 的工业原型**：Primary + Secondaries + Positions + Run() 过程化生成子零件，支持嵌套。建议 OmniM Assembly 实现"组件脚本"机制（参考 ConnectionBase.Run()） |
| BooleanPart（CUT/ADD）+ Fitting（平面拟合）+ CutPlane（平面切割） | BooleanSubtract 需支持两种模式：实体切割 + 半空间平面切割（后者更高效） |
| 7 类钢筋（SingleRebar/RebarGroup/CircleRebarGroup/CurvedRebarGroup/RebarMesh/RebarStrand/RebarSet） | Rebar=LinearComponent（圆截面+任意折线路径+阵列）；RebarMesh=LayerExtrusion 或 Assembly；建议建立 Rebar 专用 ComponentDef（封装弯钩/弯曲半径/保护层/modifier） |
| BoltGroup（Array/Circle/XYList）+ Weld | 建议新增 Feature::BoltArray/BoltCircle/BoltXYList 阵列 Pattern + Weld Feature |
| Brep（Part 子类）API 命名误导 | Tekla Brep 实际是截面扫掠体，非自由 BREP；真 BREP 走 FacetedBrep。OmniM Primitive::Brep 命名需澄清 |

### 9.4 AVEVA Instrumentation / E3D Electrical（仪表电气域）— 逻辑-物理双视图范式

| 关键发现 | 对 OmniM 的启示 |
|---------|---------------|
| 逻辑视图（SCHEMATIC/ENGINEERING）+ 物理视图（MODEL）双架构 | **OmniM 当前缺失逻辑对象层**——Loop/Tag 无几何关联。建议新增 LogicalObject 层承载回路/位号/连通性 |
| Instrument/Transmitter/ControlValve/JunctionBox/Panel/MCC/Support = 多子件 Assembly | Assembly 在仪表电气域是主体 GenRule（壳体+内部件+接口） |
| Cable/Conduit/ProcessLine = 截面沿中心线扫掠 | LinearComponent 覆盖电缆/导管/工艺管线；多芯电缆用多层截面 |
| CableTray 直段=LayerExtrusion（U/C 多层）/ 弯头=LinearComponent / 三通十字=Assembly | 同一构件在不同拓扑下映射不同 GenRule，验证 GenRule 按几何构造而非构件类型分类的设计 |
| Panel/MCC 面板开孔 = BooleanSubtract | BooleanSubtract 场景从 Door/Window 墙开洞扩展到盘柜面板开孔 |
| ISA-5.1 Tag 体系（FT-101A）+ Loop 回路 | 逻辑属性需绑定到物理对象（Tag/Loop/P&ID Ref 作为 PropertySet） |
| ProcessLine 元件序列模式（直管段+弯头+阀门） | 与 E3D BRANCH 一致，验证 LinearComponent 的"元件序列"子模式 |

### 9.5 七平台 GenRule 收敛验证总表

| GenRule | Revit | Civil3D | E3D | Bentley | Tekla | SolidWorks | E3D Instr | 收敛度 |
|---------|-------|---------|-----|---------|-------|------------|-----------|--------|
| **Primitive** | ✅ 族 Extrusion | ✅ 管件 Parts | ✅ CYLI/BOX/CONE | ✅ Bearing/Functional Cell | ✅ ContourPlate/LoftedPlate | ✅ Part/Toolbox | ✅ Sensor(简单) | 7/7 全覆盖 |
| **LayerExtrusion** | ✅ Wall/Floor/Roof | ✅ Pavement | ✅ PANE | ✅ Surface Template/Deck(直线) | ✅ RebarMesh/SurfaceTreatment | ✅ Sheet Metal 钣金 | ✅ CableTray 直段 | 7/7 |
| **LinearComponent** | ✅ Beam/Column | ✅ Corridor | ✅ BRANCH | ✅ Girder/Lining/Dam/Slope | ✅ Beam/Rebar | ✅ Weldment 结构构件(窄) | ✅ Cable/Conduit | 7/7 |
| **Assembly** | ✅ 嵌套族 | ✅ Assembly | ✅ EQUI | ✅ Pier/Abutment/Civil Cell | ✅ Connection/Component | ✅ Assembly/SubAssembly | ✅ Instrument/Panel/MCC | 7/7 全覆盖 |
| **BooleanSubtract** | ✅ 空心族 Void | — | ✅ 负实体 | ✅ Cut/Fill/Trimming | ✅ BooleanPart/Fitting/CutPlane | ✅ Extrude Cut | ✅ Panel 开孔 | 6/7（Civil3D 不直接支持） |

**收敛结论**：
- **Primitive、Assembly、LayerExtrusion、LinearComponent 在 7 平台全覆盖**（SolidWorks 重新调研后确认 Weldment=LinearComponent、Sheet Metal=LayerExtrusion），是最稳健的四类 GenRule
- **LinearComponent 在机械域应用较窄**（仅 Weldment 结构构件，因机械零件不走"截面沿路径"范式），但几何同构性成立
- **LayerExtrusion 在机械域为松散映射**（钣金是单层折弯而非多层），需独立 SheetMetal Feature 支持展开/折叠
- **BooleanSubtract 在 6/7 平台收敛**（Civil3D 走 Target 映射而非布尔）

### 9.6 扩展调研后的设计建议更新

在初版 §8.3 五条建议基础上，补充：

6. **VariationRule 需支持 6 种插值方法**（None/Linear/Parabolic±/Circular±）—— Bentley OBM Beam Definition 的工业实践

7. **Assembly 应实现"组件脚本"机制**（输入契约 + 过程化子对象 + 嵌套）—— 参考 Tekla ConnectionBase.Run()，这是 Assembly 在连接/节点场景的工业原型

8. **新增 TerrainModel 作为独立 ObjectType**（TIN 曲面，非 GenRule）—— Bentley Terrain Model 的定位，与 5 类 GenRule 并列，作为 LinearComponent 的外部约束目标

9. **BooleanSubtract 需支持半空间平面切割**（Fitting/CutPlane 模式）与曲面间布尔（Cut/Fill Volumes）—— Tekla + Bentley 的工业实践

10. **新增 LogicalObject 层承载逻辑关联**（Loop/Tag/连通性）—— E3D Instrumentation 逻辑-物理双视图的启示，当前 OmniM 仅物理层

11. **合并 Column 到 Beam 作为"用途标签"**—— Tekla 用 Beam 统一所有线构件的实践，减少 ObjectType 冗余

12. **Primitive 需扩展 Loft（双导线放样）**—— Tekla LoftedPlate / SolidWorks Loft 的共同需求

### 9.7 三种设计范式 → 七平台映射

```
┌──────────────────────────────────────────────────────────────────┐
│                      OmniM 统一几何内核                            │
│  Box  Cylinder  Cone  Sphere  Extrusion  Sweep  Revolve  Loft    │
│              Union  Subtract  Intersect  + 半空间平面切割          │
├──────────────────────────────────────────────────────────────────┤
│                      GenRule 分类层                                │
│  Primitive  LayerExtrusion  LinearComponent                       │
│  Assembly  BooleanSubtract  + TerrainModel(独立)                  │
│              + LogicalObject(逻辑层)                               │
├──────────────────────────────────────────────────────────────────┤
│                      三种设计范式 → 七平台                          │
├──────────────┬──────────────┬──────────────┬─────────────────────┤
│  Revit 式     │  Civil3D 式   │  E3D 式       │  机械/钢结构式       │
│  (语义驱动)   │  (线构件驱动) │  (基本体+目录) │  (特征树/零件+节点) │
│              │              │              │                     │
│  Revit Wall  │  Civil3D     │  E3D EQUI    │  SolidWorks Part    │
│  Tekla Beam  │  Corridor    │    + CYLI    │  Tekla Beam+        │
│              │  Bentley     │    + NOZZ    │    Connection       │
│              │  Corridor    │  E3D Instr   │  SolidWorks         │
│              │  Template    │    Panel/MCC │    Assembly         │
└──────────────┴──────────────┴──────────────┴─────────────────────┘
```

**扩展结论**：七平台调研进一步验证了 OmniM "统一几何内核 + GenRule 分类 + 设计范式"架构的可行性与完备性。新增的 TerrainModel（独立类型）与 LogicalObject（逻辑层）是初版未覆盖的两个维度，建议纳入架构补充。LinearComponent 在基建（Bentley）与结构（Tekla）域的覆盖广度，确认其作为跨域核心 GenRule 的定位。

---

## 10. 场地与地形跨平台对比（2026-08-07 新增）

> 来源：`site_terrain_components.md` 专项调研。本节将场地（Site）与地形（Terrain）作为独立维度进行六平台（Revit/Civil 3D/Bentley/E3D/Tekla/SolidWorks）对比分析。

### 10.1 场地组织与空间层级对比

| 维度 | Revit | Civil 3D | Bentley | E3D | Tekla | SolidWorks |
|------|-------|---------|---------|-----|-------|------------|
| **Site 模型** | IfcSite 空间容器 | 项目空间容器 | Civil Model 容器 | SITE 组织容器 | 无 | 无 |
| **层级结构** | Site→Building→Storey→Space | 项目→地块→设计 | Civil Model→Alignment→Corridor | Project→Site→Area→Unit | 无对应 | Part→Assembly→Drawing |
| **地理坐标** | ✅ Lat/Long + Project Base | ✅ 测量坐标系统 | ✅ 地理坐标系 | ⚠️ 有限支持 | ❌ 无 | ❌ 无 |
| **边界定义** | ✅ Site Boundary | ✅ 地块边界 | ✅ Civil Model 边界 | ⚠️ 区域边界 | ❌ 无 | ❌ 无 |
| **构件归属** | Building 归属 Site | 构件归属地块 | 构件归属 Civil Model | 设备归属 Site/Area | 构件归属 Assembly | Part 归属 Assembly |

**关键洞察**：
- Revit 的 Site 定位最清晰（IfcSite → IfcBuilding → IfcBuildingStorey → IfcSpace），完全遵循 IFC 空间层级
- Civil 3D 和 Bentley 不强调 IFC 空间层级，而是以工程对象（Alignment/Corridor）为组织核心
- E3D 的 Site 是工厂布局的组织/权限容器，无地理坐标要求
- Tekla 和 SolidWorks 无场地概念，项目组织基于装配树

### 10.2 地形建模能力对比（TIN 曲面）

| 维度 | Revit | Civil 3D | Bentley | E3D | Tekla | SolidWorks |
|------|-------|---------|---------|-----|-------|------------|
| **TIN 支持** | ✅ 基础 TIN | ✅ 完整 TIN | ✅ 完整 TIN | ❌ 无 | ❌ 无 | ❌ 无 |
| **数据源** | 点/等高线 | 点/等高线/断裂线/曲面 | 点/等高线/断裂线/边界 | 外部引用 | ❌ 无 | ❌ 无 |
| **断裂线（Breakline）** | ❌ 无 | ✅ 完整支持 | ✅ 完整支持 | ❌ 无 | ❌ 无 | ❌ 无 |
| **边界约束** | 基础支持 | ✅ Outer/Benchmark | ✅ 外边界/空洞 | ❌ 无 | ❌ 无 | ❌ 无 |
| **地形编辑** | 点级编辑 | 顶点+断裂线+边界 | 顶点+断裂线+边界 | ❌ 无 | ❌ 无 | ❌ 无 |
| **地形合成** | 基础 | 走廊→曲面 | 走廊→曲面 | ❌ 无 | ❌ 无 | ❌ 无 |
| **地形显示** | 三角网/等高线/坡度 | 三角网/等高线/坡度/汇水 | 三角网/等高线/坡度 | ❌ 无 | ❌ 无 | ❌ 无 |

**关键洞察**：
- Civil 3D 和 Bentley 拥有业界最完整的 TIN 实现，支持断裂线（三角形边不可跨越）和边界约束
- Revit 仅支持基础 TIN（点级编辑），缺少断裂线和高级边界约束
- TIN 不是通用构件——仅 BIM/基建平台实现，机械 CAD 无此需求

### 10.3 土方与平整能力对比

| 维度 | Revit | Civil 3D | Bentley | E3D | Tekla | SolidWorks |
|------|-------|---------|---------|-----|-------|------------|
| **土方计算** | ❌ 无 | ✅ 三种算法 | ✅ 三种算法 | ❌ 无 | ❌ 无 | ❌ 无 |
| **平整系统** | ❌ 无 | ✅ Grading Group | ⚠️ 通过模板 | ❌ 无 | ❌ 无 | ❌ 无 |
| **边坡设计** | ⚠️ 手动/Dynamo | ✅ 完整支持 | ✅ 完整支持 | ❌ 无 | ❌ 无 | ❌ 无 |
| **Daylight 求交** | ❌ 无 | ✅ 自动求解 | ✅ 自动求解 | ❌ 无 | ❌ 无 | ❌ 无 |
| **土方平衡** | ❌ 无 | ✅ 自动计算 | ✅ 自动计算 | ❌ 无 | ❌ 无 | ❌ 无 |
| **工程量报表** | ⚠️ 基础 | ✅ 详细 | ✅ 详细 | ❌ 无 | ❌ 无 | ❌ 无 |

**关键洞察**：
- Civil 3D 的 Grading Group 是业界最成熟的平整系统，支持平整面/平整规则/目标曲面三级结构
- Bentley 通过走廊模板的 End Condition 实现边坡和 Daylight，但无独立的平整系统
- Revit 完全缺失土方和平整功能，需通过 Dynamo 脚本或 Civil 3D 集成实现

### 10.4 地形-结构/设施交互对比

| 交互方式 | Revit | Civil 3D | Bentley | E3D | Tekla | SolidWorks |
|---------|-------|---------|---------|-----|-------|------------|
| 建筑→地形 | 基础（自动剪切） | 完整（Target Mapping） | 完整（End Condition） | 无 | 无 | 无 |
| 基础→地形 | 自动剪切 | 精确土方 | 精确土方 | 无 | 简化（Z 坐标） | 无 |
| 管线→地形 | 标高控制 | 埋深/架空 | 埋深/架空 | 地坪标高 | 无 | 无 |
| 道路→地形 | 无原生 | 完整（Corridor+Target） | 完整（Corridor Template） | 无 | 无 | 无 |
| 桥/隧→地形 | 无原生 | 部分 | 完整（End Condition） | 无 | 无 | 无 |
| 边坡→地形 | 无原生 | 完整（Daylight） | 完整（Daylight） | 无 | 无 | 无 |

**关键洞察**：
- Civil 3D 和 Bentley 实现了"构件 Target 到地形"的完整交互机制——构件通过引用目标曲面自动调整高程
- Revit 的建筑-地形交互仅限基础/地下室的自动剪切，道路/桥梁与地形的联动需通过 Civil 3D
- 地形交互是基建域的核心需求，建筑域需求较弱

### 10.5 场地/地形能力综合评分

| 平台 | 场地组织 | 地形建模 | 土方平整 | 地形交互 | **综合** | 主要定位 |
|------|---------|---------|---------|---------|---------|---------|
| **Civil 3D** | 4/5 | 5/5 | 5/5 | 5/5 | **19/20** | 场地/地形标杆 |
| **Bentley** | 4/5 | 5/5 | 5/5 | 5/5 | **19/20** | 场地/地形标杆 |
| **Revit** | 5/5 | 3/5 | 1/5 | 3/5 | **12/20** | 空间容器标杆 |
| **E3D** | 3/5 | 1/5 | 0/5 | 1/5 | **5/20** | 逻辑容器 |
| **Tekla** | 1/5 | 0/5 | 0/5 | 1/5 | **2/20** | 无场地 |
| **SolidWorks** | 0/5 | 0/5 | 0/5 | 0/5 | **0/20** | 无场地 |

### 10.6 OmniM 场地/地形设计建议

基于六平台对比，提出以下设计建议：

#### 10.6.1 新增 ObjectType 清单

| ObjectType | GenRule 分类 | 说明 | 优先级 | 主要参考 |
|-----------|-------------|------|--------|---------|
| **Site** | 无（空间容器） | 地理/空间容器 | P0 | Revit IfcSite |
| **Building** | 无（空间容器） | 建筑容器 | P0 | Revit IfcBuilding |
| **BuildingStorey** | 无（空间容器） | 楼层容器 | P0 | Revit IfcBuildingStorey |
| **Space** | 无（空间容器） | 空间单元 | P0 | Revit IfcSpace |
| **TerrainModel** | **独立类型（非 GenRule）** | TIN 曲面地形 | P0 | Civil 3D/Bentley |
| **CutVolume** | BooleanSubtract | 挖方量 | P1 | Civil 3D/Bentley |
| **FillVolume** | BooleanSubtract | 填方量 | P1 | Civil 3D/Bentley |
| **Slope** | LinearComponent | 边坡 | P1 | Civil 3D/Bentley |
| **GradingSurface** | TerrainModel（role=design） | 平整设计面 | P1 | Civil 3D |
| **FeatureLine** | 线性元素（无厚度） | 特征线/控制线 | P1 | Civil 3D/Bentley |
| **SiteComponent** | ComponentDef on Terrain | 地形表面附属物 | P2 | Revit |
| **ContourLine** | 曲线元素 | 等高线显示 | P2 | Civil 3D/Bentley |

#### 10.6.2 TerrainModel 数据结构建议

```typescript
// TerrainModel — 独立 ObjectType，非 GenRule
interface TerrainModelData {
  id: string
  vertices: TerrainVertex[]       // 顶点：x, y, z, id
  triangles: TerrainTriangle[]     // 三角形：vertex[3] 索引
  breaklines: TerrainBreakline[]   // 断裂线：polyline（三角形边不可跨越）
  boundaries: TerrainBoundary[]    // 外边界 + 内边界（洞）
  surfaceType: 'existing' | 'design' | 'subgrade' | 'datum'
  coordinateSystem: CoordinateSystem // 地理坐标系统
}

interface TerrainVertex {
  id: number
  x: number
  y: number
  z: number
}

interface TerrainBreakline {
  vertices: number[]               // 顶点索引序列
  enforce: boolean                 // 是否强制（三角形边不可跨越）
}

interface TerrainBoundary {
  vertices: number[]               // 顶点索引序列（闭合）
  isOuter: boolean                 // true=外边界，false=内边界（洞）
}
```

#### 10.6.3 TerrainModel 核心 API 建议

```typescript
class TerrainModel {
  // 数据创建
  static fromPoints(points: Point3d[]): TerrainModel
  static fromContours(contours: Polyline[]): TerrainModel
  static fromCorridor(corridor: LinearComponent): TerrainModel

  // 编辑操作
  addVertex(point: Point3d): void
  removeVertex(id: number): void
  editVertexElevation(id: number, z: number): void
  addBreakline(vertices: number[], enforce?: boolean): void
  addBoundary(vertices: number[], isOuter: boolean): void
  merge(other: TerrainModel): TerrainModel
  split(breakline: number[]): TerrainModel[]

  // 查询
  getElevation(x: number, y: number): number  // 双线性插值
  getContours(interval: number): Polyline[]
  getSlopeAnalysis(): ColorMap

  // 土方计算
  computeCutFill(target: TerrainModel): { cut: number; fill: number }
  computeEarthworkBalance(target: TerrainModel): EarthworkReport
}
```

#### 10.6.4 跨平台统一架构补充

```
┌──────────────────────────────────────────────────────────────────┐
│                  OmniM 空间层级结构                                │
│  Site (IfcSite)                                                  │
│  ├── Building (IfcBuilding)                                      │
│  │   ├── BuildingStorey (IfcBuildingStorey)                      │
│  │   │   └── Space (IfcSpace)                                    │
│  │   └── ...                                                     │
│  └── TerrainModel (地形)                                         │
│      ├── CutVolume / FillVolume (土方)                           │
│      ├── Slope (边坡)                                            │
│      ├── GradingSurface (平整面)                                 │
│      └── SiteComponent (场地附属物)                              │
├──────────────────────────────────────────────────────────────────┤
│                  GenRule + 独立类型                                │
│  Primitive | LayerExtrusion | LinearComponent                    │
│  Assembly | BooleanSubtract                                      │
│  + TerrainModel (独立类型)                                       │
│  + LogicalObject (逻辑层：Loop/Tag/连通性)                        │
├──────────────────────────────────────────────────────────────────┤
│                  外部互操作                                        │
│  IFC 4x3: IfcSite / IfcGeographicElement                         │
│  LandXML: Surfaces / Breaklines / Boundaries                     │
│  Civil 3D DWG/DXF 地形数据                                       │
└──────────────────────────────────────────────────────────────────┘
```

#### 10.6.5 优先级路线图

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

### 10.7 IFC 标准互操作

| OmniM ObjectType | IFC 4x3 映射 | LandXML 映射 |
|-----------------|-------------|-------------|
| Site | IfcSite | `Project` |
| TerrainModel | IfcGeographicElement | `Surfaces/Surface` |
| CutVolume / FillVolume | IfcGeographicElement（土方） | `Grade` |
| Slope | IfcGeographicElement（边坡） | — |
| GradingSurface | IfcGeographicElement（设计面） | `Grade` |
| SiteComponent | IfcElement（场地附属物） | — |

**互操作结论**：
- IFC 4x3 提供了 IfcSite 和 IfcGeographicElement 两个核心类型，但地形支持仍有限
- LandXML 是地形数据交换的事实标准，Civil 3D 和 Bentley 都原生支持
- 建议 OmniM 同时支持 IFC（空间层级）和 LandXML（地形数据）双格式导入导出的定位。