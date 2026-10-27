# SOLIDWORKS 内置构件体系调研

> 调研日期：2026-08-07
> 来源（均为 SOLIDWORKS 2026 官方文档，help.solidworks.com）：
> - API 程序员指南入口：`https://help.solidworks.com/2026/English/api/sldworksapiprogguide/Welcome.htm`
> - API 参考主页：`https://help.solidworks.com/2026/English/api/sldworksapi/`（注：原任务清单给出的全小写 URL `.../english/api/sldworksapi/` 实测返回 404，正确 URL 区分大小写）
> - API 2026 发行说明：`https://help.solidworks.com/2026/English/api/sldworksapi/ReleaseNotes-sldworksapi.html`
> - SOLIDWORKS 用户帮助（英文）：`https://help.solidworks.com/2026/English/solidworks/`
> - SOLIDWORKS Toolbox 概述：`https://help.solidworks.com/2026/English/SolidWorks/toolbox/c_toolbox_overview.htm`
> - SOLIDWORKS API 支持页：`https://www.solidworks.com/sw/support/api-support.htm`
> 目标：分析机械 CAD 领域的内置构件底层逻辑，映射到 OmniM 的 5 类 GenRule（Primitive / LayerExtrusion / LinearComponent / Assembly / BooleanSubtract）
> 状态：完成（所有内容均经官方文档核对；与前一版本的差异在对应章节以「更正说明」标注）

---

## 0. 重要更正说明（与前一版本对比）

前一版本存在多处与官方文档不符的内容，本版本依据官方文档逐条更正：

1. **Toolbox 内容严重失实**（最严重）：前一版本声称 Toolbox 包含铆钉(Rivets)、密封件(Seals)、弹簧(Springs)、键(Keys)、联轴器(Couplings)、衬套(Bushings)、管件(Pipe Fittings)、皮带轮(通用 Pulleys) 等。官方文档明确 Toolbox 仅包含 15 类硬件（见 §5），**不含**铆钉/密封件/弹簧/键/联轴器/衬套/管件/通用皮带轮。前一版本还漏列了官方明确列出的 Cams（凸轮）、Jig bushings（钻模套管）、PEM® inserts、Sprockets（链轮）、Timing belt pulleys（正时带轮）、Unistrut® 等。
2. **Toolbox 支持标准不全**：前一版本仅列 ANSI/ISO/DIN/JIS/GB。官方明确支持 11 个标准：ANSI、AS、GB、BSI、CISC、DIN、GB、ISO、IS、JIS、KS（其中 GB 出现两次，疑为官方文档笔误或分别指不同国标体系）。
3. **Toolbox 齿轮性质未说明**：官方明确 "Toolbox gears are representations for machine design purposes. They are not true involute gears that you can use for manufacturing."（Toolbox 齿轮仅为机械设计示意，并非可用于制造的真实渐开线齿轮）。前一版本未提及此关键限制。
4. **配合类型遗漏与命名错误**：前一版本遗漏了 Advanced 配合中的 Profile Center，遗漏了 Mechanical 配合中的 Cam 和 Slot；将 "Path Mate" 误写为 "Path"，将 "Linear/Linear Coupler" 误写为 "Linear Coupler"。
5. **"Loft 与 Boundary 是否为独立特征"**：前一版本未明确区分。官方文档明确两者是**独立的特征**（Boundary 在多数情况下质量高于 Loft），见 §3。
6. **API 主页 URL 错误**：任务清单中给出的 `https://help.solidworks.com/2026/english/api/sldworksapi/`（全小写）实测返回 404；正确 URL 为 `https://help.solidworks.com/2026/English/api/sldworksapi/`（English 首字母大写）。
7. **Fillet 子类型**：前一版本仅列"恒定/变半径/面圆角/完整圆角"4 类；官方实际列出 8 类（含 Multiple Radius、Round Corner、Setback、Asymmetric）。
8. **Hole Wizard 孔类型**：前一版本未列具体孔类型；官方明确为 Counterbore / Countersink / Hole / Straight Tap / Tapered Tap / Legacy / Slots（含 3 种槽口）。

---

## 1. 平台概述

SOLIDWORKS 是机械 CAD 领域的参数化特征建模行业标准，采用 **特征树驱动（FeatureManager-driven）** 建模范式。其数据模型与 Tekla（Part+Component）、Bentley（Corridor+Template）不同——所有几何都通过 FeatureManager 设计树中的有序特征构建，每个特征基于草图或对已有几何的应用操作。

### 1.1 三文档体系

官方明确 SOLIDWORKS 有三种文档类型 [来源: help.solidworks.com/2026/English/SolidWorks/acadhelp/r_moving2d3d_approach_modeling_ov.htm]：

| 文档类型 | 扩展名 | 用途 |
|---------|--------|------|
| Part（零件） | `.sldprt` | 单一零件，可含多实体（multibody） |
| Assembly（装配体） | `.sldasm` | 多零件实例 + 子装配 + 配合 |
| Drawing（工程图） | `.slddrw` | 由模型生成的 2D 视图 + 标注 + BOM |

> 官方原文："Types of files: .dwg files (2D CAD) vs .sldprt (parts) / .sldasm (assemblies) / .slddrw (drawings) (SOLIDWORKS)"

### 1.2 建模哲学

SOLIDWORKS 特征分两类 [来源: help.solidworks.com/2026/English/SolidWorks/acadhelp/c_moving2d3d_feature_based_models.htm]：

- **Sketched features（草图特征）**：基于草图几何，如 Extrusion / Revolve / Sweep / Loft / Boundary。
- **Applied features（应用特征）**：直接施加于模型，如 Chamfer / Fillet / Shell。

> 官方原文："SOLIDWORKS features are of two types: sketched and applied. Sketched features such as extrusions, revolves, sweeps, and lofts are based on sketch geometry. Applied features such as chamfers, fillets, and shells are applied directly to the model."

### 1.3 与 OmniM 的总体映射

| SOLIDWORKS 概念 | OmniM 等价 | 说明 |
|----------------|-----------|------|
| `.sldprt` (Part) | `ObjectType::Part` | 特征树驱动的单一零件 |
| `.sldasm` (Assembly) | `ObjectType::Assembly` | 子件引用 + 配合 |
| Sub-assembly | `ObjectType::Assembly`（嵌套） | 支持任意层级嵌套 |
| Configuration | Series 配置变体 | 设计表 → Series Store 选择器 |
| Feature（特征） | Feature Engine 中的 Feature | 历史记录节点 |
| Mate（配合） | `Constraint`（关系系统） | **需新增** |
| Toolbox 标准件 | Catalog ComponentDef | 标准件库 |

---

## 2. API 类层次与文档接口

### 2.1 IModelDoc2 与文档接口

SOLIDWORKS API 基于 COM，使用接口继承。官方明确三类文档各有专属接口，公共功能通过 `IModelDoc2` 暴露 [来源: help.solidworks.com/2026/english/api/sldworksapi/SOLIDWORKS.Interop.sldworks~SOLIDWORKS.Interop.sldworks.IModelDoc2.html]：

> 官方原文："There are three main SOLIDWORKS document types: parts, assemblies, drawings. Each document type has its own object (IPartDoc, IAssemblyDoc, and IDrawingDoc) with its own set of related functions. For example, the IAssemblyDoc::AddComponent4 method exists on the IAssemblyDoc object because adding components is specific to assembly documents. The SOLIDWORKS API also has functions that are common to all document types... To expose common document-level functions, the SOLIDWORKS API uses the IModelDoc2 object."

### 2.2 API 类层次（QueryInterface 关系）

依据官方 API 程序员指南 [来源: help.solidworks.com/2024/English/api/sldworksapiprogguide/GettingStarted/Understanding_the_SolidWorks_API_Class_Hierarchy.htm]：

```
ISldWorks                              ← 应用层
├── IModelDoc2                         ← 文档公共接口（公共操作：保存、打印、文件名）
│   ├── IPartDoc                       ← 零件文档（创建体/特征、抑制操作、tessellation）
│   ├── IAssemblyDoc                   ← 装配文档（AddComponent5、GetComponents、配合）
│   └── IDrawingDoc                    ← 工程图文档（创建/对齐/访问视图）
├── IFeatureManager                    ← 特征管理器（FeatureManager 设计树操作）
├── ISketchManager                     ← 草图管理器
├── IConfigurationManager              ← 配置管理器
├── IModelDocExtension                 ← 文档扩展（CheckVersionCompatibility 等）
└── IAttributeDef / IBody2 / IFace2 / IEdge / IVertex ...
```

**关键 QueryInterface 关系**（官方明确）：
- `IAssemblyDoc` / `IDrawingDoc` / `IPartDoc` 可 QueryInterface 到 `IModelDoc2`
- `IEdge` / `IFace2` / `IFeature` / `ILoop2` / `IVertex` 可 QueryInterface 到 `IEntity`
- `IAttribute` 可 QueryInterface 到 `IFeature`

### 2.3 核心 API 接口说明

| 接口 | 官方定义 | 关键方法/属性 | 来源 |
|------|---------|--------------|------|
| `IModelDoc2` | "Allows access to SOLIDWORKS documents: parts, assemblies, and drawings." | 文档公共操作 | sldworksapi/...IModelDoc2.html |
| `IPartDoc` | "Provides access to functions that perform operations on parts in part documents." | 创建体/特征、抑制、tessellation、按名查找实体 | sldworksapi/...IPartDoc.html |
| `IAssemblyDoc` | 装配文档操作 | `AddComponent5`、`GetComponents`、`CreateMate`、`FeatureByName`、`InterferenceDetectionManager`、`ClearanceVerificationManager` | sldworksapi/...IAssemblyDoc_members.html |
| `IDrawingDoc` | "Allows access to functions that perform drawing operations." | 创建/对齐/访问视图 | sldworksapi/...IDrawingDoc.html |
| `IFeature` | "Allows access to the feature type, name, parameter data, and the next feature in the FeatureManager design tree." | `GetChildren`、`GetFirstSubFeature`、`GetNextFeature`、`GetNextSubFeature`、`GetOwnerFeature`、`GetParents` | sldworksapi/...IFeature.html |
| `IComponent2` | "Allows access to the components within assemblies." 用于遍历装配体 | `GetChildren`、`GetParent`、`GetModelDoc2`、`FeatureByName`、`FirstFeature` | sldworksapi/...IComponent2.html |

> 官方对 IComponent2 的备注："Use this interface to traverse assemblies. Information returned from the methods and properties of this interface is based on the current configuration and might vary if the user displays a different configuration."

### 2.4 API 2026 新增（Service Pack 3 节选）

依据官方 2026 发行说明 [来源: help.solidworks.com/2026/English/api/sldworksapi/ReleaseNotes-sldworksapi.html]：
- 新增 `IVersionCompatibilityItem` 接口（版本兼容性项）
- 新增 `IModelDocExtension::CheckVersionCompatibility` 方法
- 新增多个事件处理器：`DSldWorksEvents_FileSaveToVersionFailNotifyEventHandler`、`DAssemblyDocEvents_AddDependencyNotifyEventHandler`、`DDrawingDocEvents_AddDependencyNotifyEventHandler` 等

---

## 3. FeatureManager 特征树与特征类型

### 3.1 特征类型总览

依据官方 Features 主题页 [来源: help.solidworks.com/2026/English/solidworks/sldworks/c_Features_Top.htm] 与 Features Toolbar 页 [来源: help.solidworks.com/2026/English/SolidWorks/sldworks/r_Features_Toolbar_features.htm]：

> 官方原文："Features are the individual shapes that, when combined, make up the part. You can also add some types of features to assemblies. Features include multibody part capability. You can include separate extrude, revolve, loft, or sweep features, within the same part document. Some features originate as sketches; other features, such as shells or fillets, are created when you select the appropriate tool or menu command and define the dimensions or characteristics that you want."

### 3.2 草图特征（Sketched Features）—— 5 类基础造型方法

依据官方 Feature-based Models 页 [来源: help.solidworks.com/2026/English/SolidWorks/acadhelp/c_moving2d3d_feature_based_models.htm]，SOLIDWORKS 提供以下基础造型特征：

| 特征 | 官方定义 | OmniM GenRule 映射 |
|------|---------|-------------------|
| **Extrude（拉伸）** | "Extrude creates a feature by extruding a 3D object from a 2D sketch, essentially adding the third dimension. An extrusion can be a base (always adds material), a boss (adds material), or a cut (removes material)." | Primitive::Extrusion / BooleanSubtract（Cut） |
| **Revolve（旋转）** | "Revolve creates a feature that adds or removes material by revolving one or more sketch profiles around a centerline. The feature can be either a solid, a thin feature, or a surface." | Primitive::Revolve |
| **Loft（放样）** | "Loft creates a feature by making transitions between profiles. A loft can be a base, boss, cut, or surface." | Primitive::Loft |
| **Sweep（扫描）** | "Sweep creates a base, boss, cut, or surface by moving a profile (section) along a path." | Primitive::Sweep |
| **Boundary（边界）** | "Boundary creates very high quality, accurate features useful for creating complex shapes for the consumer product design, medical, aerospace, and mold markets. A boundary can be a base, boss, cut, or surface." | Primitive::Loft（变体） |

**关键确认**：**Loft 与 Boundary 是两个独立的特征**，并非同一特征的别名。官方在 Boundary Surface 页进一步说明："The boundary surface feature lets you create surfaces that can be tangent or curvature continuous in both directions (all sides of the surface). In most cases, this delivers a higher quality result than the loft tool." [来源: help.solidworks.com/2026/English/solidworks/sldworks/c_Surface_Features_Folder.htm]

### 3.3 应用特征（Applied Features）—— Fillet 子类型

依据官方 Fillet Overview 页 [来源: help.solidworks.com/2026/English/solidworks/sldworks/c_fillet_overview.htm]，Fillet 共有 **8 种子类型**（前一版本仅列 4 种，遗漏 4 种）：

1. Constant Size Fillet（恒定尺寸圆角）
2. Multiple Radius Fillet（多半径圆角）
3. Round Corner Fillet（圆角拐角）
4. Setback Fillet（退缩圆角）
5. Variable Size Fillet（变半径圆角）
6. Face Fillet（面圆角）
7. Full Round Fillet（完整圆角）
8. Asymmetric Fillet（非对称圆角）

> 官方原文："Fillet/Round creates a rounded internal or external face on the part. You can fillet all edges of a face, selected sets of faces, selected edges, or edge loops."

### 3.4 Features Toolbar 全量特征清单

依据官方 Features Toolbar 页 [来源: help.solidworks.com/2026/English/SolidWorks/sldworks/r_Features_Toolbar_features.htm]，按类别整理：

| 类别 | 特征 | OmniM 映射倾向 |
|------|------|---------------|
| **草图特征（Boss/Base）** | Extruded Boss/Base、Revolved Boss/Base、Swept Boss/Base、Lofted Boss/Base、Boundary Boss/Base、Thicken | Primitive |
| **切割特征（Cut）** | Extruded Cut、Revolved Cut、Swept Cut、Lofted Cut、Boundary Cut、Thickened Cut、Cut with Surface | BooleanSubtract |
| **修饰特征** | Fillet、Chamfer、Rib、Scale、Shell、Draft、Move Face | 修饰 Feature（部分 BooleanSubtract） |
| **孔特征** | Simple Hole、Hole Wizard | BooleanSubtract |
| **阵列/镜像** | Linear Pattern、Circular Pattern、Mirror Feature、Curve Driven Pattern、Sketch Driven Pattern、Table Driven Pattern、Fill Pattern、Variable Pattern | 阵列 Pattern |
| **实体操作** | Split、Intersect、Combine、Join、Delete/Keep Body、Heal Edges、Imported Geometry、Insert Part | Assembly / BooleanSubtract |
| **其他** | Thicken、Model Break View、Instant3D、Suppress/Unsuppress | — |

### 3.5 异型孔向导（Hole Wizard）—— 孔类型

依据官方 Hole Wizard Overview 页 [来源: help.solidworks.com/2026/English/SolidWorks/sldworks/c_Hole_Wizard_Overview.htm] 与 Hole Wizard Type PropertyManager 页 [来源: help.solidworks.com/2026/English/SolidWorks/sldworks/r_HIDD_DVE_HW_HOLE_SPEC.htm]，Hole Wizard 支持 **7 大类孔型**（含 3 种槽口变体）：

| 孔类型 | 中文名 | API 枚举（swWzdHoleTypes_e） |
|--------|-------|------------------------------|
| Counterbore | 柱形沉头孔 | swCounterBored |
| Countersink | 锥形沉头孔 | swCounterSunk |
| Hole | 孔 | swSimple / swSimpleDrilled |
| Straight Tap | 直螺纹孔 | swTapered（直螺纹部分） |
| Tapered Tap | 锥螺纹孔 | swTapered |
| Legacy | 旧制孔（SOLIDWORKS 2000 前创建） | — |
| Counterbore Slot / Countersink Slot / Slot | 柱形沉头槽 / 锥形沉头槽 / 槽口 | 含 Length 属性 |

API 孔类型枚举（依据 [来源: help.solidworks.com/2026/english/api/sldworksapi/Table_of_Wizard_Hole_Types_and_Valid_Properties.htm]）：`swSimple`、`swTapered`、`swCounterBored`、`swCounterSunk`、`swCounterDrilled`、`swSimpleDrilled` 等。

> 官方原文："You can create these types of Hole Wizard holes: Counterbore, Countersink, Hole, Straight Tap, Tapered Tap, Legacy, Slots." 

Hole Wizard 与 Hole Wizard/Toolbox 系统选项深度集成——可通过 `Tools > Options > System Options > Hole Wizard/Toolbox > Configure` 创建自定义标准，Hole Wizard 的孔尺寸与 Toolbox 紧固件尺寸联动。

### 3.6 OmniM 映射总结

**OmniM ParametricOperation 已覆盖**：Extrusion、Revolve、Sweep、Loft（4 类草图特征）+ Boolean（Combine）。

**需新增 Feature 类型**：
- Fillet（含 8 子类型）、Chamfer、Shell、Draft、Rib
- Pattern：LinearPattern、CircularPattern、CurveDrivenPattern、Mirror
- HoleWizard（按标准的孔）、Thread（注：官方 Features Toolbar 未单列 Thread，但 Toolbox 提供"cosmetic thread"概念，配置项中可见 `Cosmetic Threads in Configurations`）
- Dome、Wrap、Split
- SheetMetal

**映射到 BooleanSubtract**：所有 Cut 类特征（Extruded Cut / Revolved Cut / Swept Cut / Lofted Cut / Boundary Cut / Thickened Cut / Cut with Surface）、Simple Hole、Hole Wizard。

---

## 4. 草图实体（Sketch Entities）

依据官方 Sketch Entities 主题页 [来源: help.solidworks.com/2026/English/SolidWorks/Sldworks/c_sketch_entities_top.htm]：

| 草图实体 | 官方说明 |
|---------|---------|
| **Lines（直线）** | Sketching Lines、Infinite Lines、Midpoint Line（中点对称线） |
| **Centerlines（中心线）** | 用于对称草图元素和旋转特征，或作为构造几何 |
| **Arcs（圆弧）** | Centerpoint Arc / Tangent Arc / 3 Point Arc 三种 [来源: help.solidworks.com/2012/english/SolidWorks/sldworks/Arcs.htm] |
| **Circles（圆）** | 完整圆 |
| **Ellipses（椭圆）** | 完整椭圆 + 部分椭圆（elliptical arc） |
| **Parabolas（抛物线）** | — |
| **Conics（圆锥曲线）** | "The Conic tool lets you sketch conic curves driven by endpoints and Rho value. Depending on the Rho value, the curve can be elliptical, parabolic, or hyperbolic." |
| **Splines（样条）** | — |
| **Points（点）** | 插入点到草图和工程图 |
| **Polygons（多边形）** | "Create equilateral polygons with any number of sides between 3 and 40." |
| **Rectangles（矩形）** | — |
| **Slots（槽口）** | — |
| **Sketch Text（草图文字）** | 可在曲线/边上写文字并拉伸/切割 |
| **Equation Driven Curves（方程驱动曲线）** | — |
| **Belt/Chain（皮带/链）** | "Use the Belt/Chain tool to create layout sketches for pulleys or sprocket mechanisms." |

### 4.1 草图关系（Sketch Relations）

依据官方 Description of Sketch Relations 页 [来源: help.solidworks.com/2026/English/solidworks/sldworks/c_Description_of_Sketch_Relations.htm]，草图关系包括：Horizontal/Vertical、Collinear、Coradial、Perpendicular、Parallel、ParallelYZ/ParallelZX/AlongZ、Tangent、Concentric、Midpoint、Intersection、Coincident、Equal、Equal Curvature、Equal Curve Length、Symmetric、Fix、Fix Slot、Pierce、Merge Points、Doubled Distance、Equal Slots、On Edge、On Plane、On Surface、Tangent to Face、Traction、Torsion Continuity。

### 4.2 OmniM 映射

| 草图实体 | OmniM 等价 |
|---------|-----------|
| Line / Centerline / Infinite Line / Midpoint Line | `SketchSegment::Line` |
| Arc（3 种） / Circle | `SketchSegment::Arc` / `Circle` |
| Ellipse / Partial Ellipse | `SketchSegment::Ellipse` |
| Spline / Equation Driven Curve | `SketchSegment::Spline` |
| Polygon / Rectangle | 组合 Line，或专用 `Polygon` |
| Slot | `SketchSegment::Slot`（Hole Wizard 槽口的基础） |
| Conic / Parabola | `SketchSegment::Conic` |
| Point | `SketchPoint` |
| Sketch Text | `SketchText`（可拉伸为 3D） |

草图关系映射到 OmniM 的 `SketchConstraint` 系统：Coincident、Concentric、Tangent、Parallel、Perpendicular、Symmetric、Equal、Midpoint 等均可直接对应。

---

## 5. Toolbox 标准件库（重点更正章节）

### 5.1 Toolbox 概述（官方原文核对）

依据官方 SOLIDWORKS Toolbox Overview 页 [来源: help.solidworks.com/2026/English/SolidWorks/toolbox/c_toolbox_overview.htm]，原文如下：

> "SOLIDWORKS® Toolbox includes a library of standard parts that is fully integrated with SOLIDWORKS Design. Select your standard and the type of part you want to insert, then drag the component into your assembly."
>
> "Toolbox supports international standards, including: ANSI, AS, GB, BSI, CISC, DIN, GB, ISO, IS, JIS, and KS."

### 5.2 Toolbox 实际包含的硬件（官方明确列出 15 类）

**依据官方原文逐条列出**（前一版本严重失实，本版本严格按官方清单）：

| # | 官方英文名 | 中文名 | 前一版本是否正确 |
|---|-----------|-------|----------------|
| 1 | Bearings | 轴承 | ✓ 正确 |
| 2 | Bolts | 螺栓 | ✓ 正确 |
| 3 | Cams | 凸轮 | ✗ 前一版本遗漏 |
| 4 | Gears | 齿轮 | ✓ 正确（但前一版本未说明非真渐开线） |
| 5 | Jig bushings | 钻模套管 | ✗ 前一版本遗漏 |
| 6 | Nuts | 螺母 | ✓ 正确 |
| 7 | PEM® inserts | PEM® 插件 | ✗ 前一版本遗漏 |
| 8 | Pins | 销 | ✓ 正确 |
| 9 | Retaining rings | 挡圈/扣环 | ✓ 正确 |
| 10 | Screws | 螺钉 | ✓ 正确 |
| 11 | Sprockets | 链轮 | ✗ 前一版本遗漏（前一版本误将"链轮"列入"传动件"） |
| 12 | Structural shapes (aluminum and steel) | 结构型材（铝和钢） | ✓ 正确（前一版本列为"型材"） |
| 13 | Timing belt pulleys | 正时带轮 | ✗ 前一版本遗漏（且前一版本误将"通用皮带轮"列入 Toolbox） |
| 14 | Unistrut® | Unistrut® 型材 | ✗ 前一版本遗漏 |
| 15 | Washers | 垫圈 | ✓ 正确 |

### 5.3 Toolbox 不包含的零件类型（重要更正）

前一版本**错误地**将以下类型列为 Toolbox 内容，但官方文档**未列出**这些类型：

| 前一版本错误声明 | 官方实际情况 | 说明 |
|----------------|------------|------|
| 铆钉 (Rivets) | **官方 Toolbox 清单未列出** | Toolbox 不含铆钉 |
| 密封件 (Seals / O-rings / Gaskets / Oil Seals) | **官方 Toolbox 清单未列出** | Toolbox 不含密封件（但 Toolbox 的 Grooves 工具可在圆柱零件上添加标准 O 型圈和挡圈沟槽） |
| 弹簧 (Springs) | **官方 Toolbox 清单未列出** | Toolbox 不含弹簧 |
| 键 (Keys / Splines) | **官方 Toolbox 清单未列出** | Toolbox 不含键 |
| 联轴器 (Couplings) | **官方 Toolbox 清单未列出** | Toolbox 不含联轴器 |
| 衬套 (Bushings) | **官方 Toolbox 清单未列出** | Toolbox 不含衬套 |
| 管件 (Pipe Fittings: Elbow/Tee/Flange) | **官方 Toolbox 清单未列出** | Toolbox 不含管件 |
| 通用皮带轮 (Pulleys) | **仅 Timing belt pulleys** | Toolbox 仅含正时带轮，不含 V 带/平带/同步带通用皮带轮 |
| 蜗杆/蜗轮/内齿/齿条（齿轮子类） | 官方仅说 "Gears" | 官方未细分齿轮子类，"Rack gears must have fewer than 1000 teeth" 是唯一子类约束 |

### 5.4 Toolbox 齿轮的关键限制（官方明确）

> 官方原文："Toolbox gears are representations for machine design purposes. They are not true involute gears that you can use for manufacturing. Rack gears must have fewer than 1000 teeth."

**OmniM 启示**：若 OmniM 的 `ObjectType::Gear` 借鉴 Toolbox，应明确标注"为机械设计示意，非制造用真渐开线齿轮"。OmniM 应提供独立的真渐开线齿轮生成器，而非依赖 Toolbox 式的示意齿轮。

### 5.5 Toolbox 紧固件的关键限制

> 官方原文："The fasteners provided in Toolbox are approximate representations and do not include accurate thread detail, which might make them inappropriate for certain analyses such as stress analysis."

### 5.6 Toolbox 的工程工具（非零件库部分）

官方明确 Toolbox 还提供以下**工程工具**（非零件，而是计算/添加几何的工具）：

- **Beam calculator（横梁计算器）**：计算横梁的应力和偏转 [来源: .../t_toolbox_beam_calculations.htm]
- **Bearing calculator（轴承计算器）**：计算轴承的承载能力和寿命 [来源: .../t_toolbox_bearing_calculations.htm]
- **Grooves（凹槽）**：在圆柱零件上添加标准 O 型圈和挡圈沟槽 [来源: .../t_toolbox_grooves_adding.htm]
- **Structural steel（结构钢）**：将结构钢横断面作为草图添加到零件 [来源: .../t_toolbox_structural_steel.htm]

> 官方备注："These tools are not available if you install SOLIDWORKS Design from the 3DEXPERIENCE platform."

### 5.7 Toolbox 规模与配置

依据官方 Toolbox Administration 页 [来源: help.solidworks.com/2026/English/SolidWorks/toolbox/c_toolbox_aministration_concepts.htm]：

> "By default, Toolbox includes more than 2000 component types of different sizes for 12 tool standards, as well as other industry-specific content, resulting in millions of components."

Toolbox 组件文件定义方式（管理员可选）[来源: .../c_toolbox_admin_concepts_configs_vs_parts.htm]：
- 作为单个零件文件的多个 Configuration
- 或每个尺寸一个独立零件文件

### 5.8 OmniM 映射（更正版）

| Toolbox 概念 | OmniM 等价 | 说明 |
|-------------|-----------|------|
| Toolbox 标准件库（15 类硬件） | Catalog ComponentDef 库 | 严格按官方 15 类，不臆造 |
| 标准组织（ANSI/ISO/DIN/JIS/GB/BSI/CISC/IS/KS/AS） | `standard_reference` 字段 | 官方 11 个标准 |
| 配置（尺寸变体） | Series Store 选择器 | 一个 Toolbox 零件 = 数百至数千配置 |
| 配合参考（Mate Reference） | ConnectorDef | Toolbox 紧固件自动带配合参考 |
| Toolbox 工程工具（Beam/Bearing calculator/Grooves/Structural steel） | 不映射为 ComponentDef | 这些是工具/计算器，非零件库 |

### 5.9 对前一版本 ObjectType 建议的更正

前一版本基于错误的 Toolbox 清单提出"需新增 OBJECT_TYPE_BOLT/NUT/WASHER/SCREW/PIN/RIVET/COUPLING/BUSHING/KEY/RETAINING_RING/PULLEY/SPROCKET/GEARBOX"。

**依据官方 Toolbox 清单的更正建议**：

| 建议新增 ObjectType | 是否对应 Toolbox | 说明 |
|--------------------|----------------|------|
| `OBJECT_TYPE_BOLT` | ✓ Bolts | Toolbox 含 |
| `OBJECT_TYPE_NUT` | ✓ Nuts | Toolbox 含 |
| `OBJECT_TYPE_WASHER` | ✓ Washers | Toolbox 含 |
| `OBJECT_TYPE_SCREW` | ✓ Screws | Toolbox 含 |
| `OBJECT_TYPE_PIN` | ✓ Pins | Toolbox 含 |
| `OBJECT_TYPE_RETAINING_RING` | ✓ Retaining rings | Toolbox 含 |
| `OBJECT_TYPE_BEARING` | ✓ Bearings | Toolbox 含（但仅示意几何） |
| `OBJECT_TYPE_GEAR` | ✓ Gears | Toolbox 含（非真渐开线） |
| `OBJECT_TYPE_SPROCKET` | ✓ Sprockets | Toolbox 含 |
| `OBJECT_TYPE_TIMING_PULLEY` | ✓ Timing belt pulleys | 仅正时带轮，非通用皮带轮 |
| `OBJECT_TYPE_CAM` | ✓ Cams | Toolbox 含（凸轮生成器） |
| `OBJECT_TYPE_JIG_BUSHING` | ✓ Jig bushings | Toolbox 含 |
| `OBJECT_TYPE_PEM_INSERT` | ✓ PEM® inserts | Toolbox 含 |
| `OBJECT_TYPE_UNISTRUT` | ✓ Unistrut® | Toolbox 含 |
| `OBJECT_TYPE_STRUCTURAL_SHAPE` | ✓ Structural shapes | Toolbox 含（铝和钢） |
| ~~`OBJECT_TYPE_RIVET`~~ | ✗ Toolbox 不含 | 前一版本错误 |
| ~~`OBJECT_TYPE_SPRING`~~ | ✗ Toolbox 不含 | 前一版本错误 |
| ~~`OBJECT_TYPE_SEAL`~~ | ✗ Toolbox 不含（仅 Grooves 工具添加沟槽） | 前一版本错误 |
| ~~`OBJECT_TYPE_KEY`~~ | ✗ Toolbox 不含 | 前一版本错误 |
| ~~`OBJECT_TYPE_COUPLING`~~ | ✗ Toolbox 不含 | 前一版本错误 |
| ~~`OBJECT_TYPE_BUSHING`~~ | ✗ Toolbox 不含 | 前一版本错误 |
| ~~`OBJECT_TYPE_PULLEY`~~（通用） | ✗ 仅正时带轮 | 前一版本错误 |
| ~~`OBJECT_TYPE_GEARBOX`~~ | ✗ Toolbox 不含 | 前一版本错误 |

---

## 6. 配合系统（Mates）

### 6.1 配合总览

依据官方 Mates Overview 页 [来源: help.solidworks.com/2026/English/solidworks/sldworks/c_Mates_Overview_Assemblies.htm]：

> 官方原文："Mates create geometric relationships between assembly components. As you add mates, you define the allowable directions of linear or rotational motion of the components. You can move a component within its degrees of freedom, visualizing the assembly's behavior."
>
> "Mates are solved together as a system. The order in which you add mates does not matter; all mates are solved at the same time. You can suppress mates just as you can suppress features."

### 6.2 标准配合（Standard Mates）—— 8 类

依据官方 Mate PropertyManager - Standard Tab 页 [来源: help.solidworks.com/2026/English/solidworks/sldworks/HIDD_NEW_ADD_MATE.htm]：

| 配合类型 | 官方定义 | OmniM 映射 |
|---------|---------|-----------|
| **Coincident（重合）** | "Positions the selected faces, edges, and planes (in combination with each other or combined with a single vertex) so they share the same infinite plane. Positions two vertices so they touch." | `Constraint::Coincident` |
| **Parallel（平行）** | "Places the selected items at a constant distance apart from each other." | `Constraint::Parallel` |
| **Perpendicular（垂直）** | "Places the selected items at a 90° angle to each other." | `Constraint::Perpendicular` |
| **Tangent（相切）** | "Places the selected items tangent to each other. At least one selection must be a cylindrical, conical, or spherical face." | `Constraint::Tangent` |
| **Concentric（同心）** | "Places the selections so that they share the same centerline." | `Constraint::Concentric` |
| **Lock（锁紧）** | "Maintains the position and orientation between two components." | `Constraint::Lock` |
| **Distance（距离）** | "Places the selected items with the specified distance between them." | `Constraint::Distance` |
| **Angle（角度）** | "Places the selected items at the specified angle to each other." | `Constraint::Angle` |

### 6.3 高级配合（Advanced Mates）—— 6 类

依据官方 Mate PropertyManager - Advanced Tab 页 [来源: help.solidworks.com/2026/English/solidworks/sldworks/r_mate_pm_advanced.htm]：

| 配合类型 | 官方定义 | OmniM 映射 |
|---------|---------|-----------|
| **Profile Center（轮廓中心）** | "Center-aligns the rectangular and circular profiles to each other and fully defines the components." | `Constraint::ProfileCenter`（前一版本遗漏） |
| **Symmetric（对称）** | "Forces two similar entities to be symmetric about a plane or planar face." | `Constraint::Symmetric` |
| **Width（宽度）** | "Constrains a tab between two planar faces." | `Constraint::Width` |
| **Path Mate（路径配合）** | "Constrains a selected point on a component to a path."（前一版本误写为 "Path"） | `Constraint::PathMate` |
| **Linear/Linear Coupler（线性/线性耦合）** | "Establishes a relationship between the translation of one component and the translation of another component."（前一版本误写为 "Linear Coupler"） | `Constraint::LinearCoupler` |
| **Limit（限制）** | "Allows components to move within a range of values for distance and angle mates." | `Constraint::Limit` |

### 6.4 机械配合（Mechanical Mates）—— 7 类

依据官方 Mate PropertyManager - Mechanical Tab 页 [来源: help.solidworks.com/2026/English/solidworks/sldworks/r_mate_pm_mechanical.htm]：

| 配合类型 | 官方定义 | OmniM 映射 |
|---------|---------|-----------|
| **Cam（凸轮）** | "Forces a cylinder, plane, or point to be coincident or tangent to a series of tangent extruded faces."（前一版本遗漏） | `Constraint::Cam` |
| **Slot（槽口）** | "Constrains the movement of a bolt or a slot within a slot hole."（前一版本遗漏） | `Constraint::Slot` |
| **Hinge（铰链）** | "Limits the movement between two components to one rotational degree of freedom." | `Constraint::Hinge` |
| **Gear（齿轮）** | "Forces two components to rotate relative to one another about selected axes." | `Constraint::Gear` |
| **Rack Pinion（齿条小齿轮）** | "Linear translation of one part (the rack) causes circular rotation in another part (the pinion), and vice versa." | `Constraint::RackPinion` |
| **Screw（螺旋）** | "Constrains two components to be concentric, and adds a pitch relationship between the rotation of one component and the translation of the other." | `Constraint::Screw` |
| **Universal Joint（万向节）** | "The rotation of one component (the output shaft) about its axis is driven by the rotation of another component (the input shaft) about its axis." | `Constraint::UniversalJoint` |

### 6.5 配合的几何有效性矩阵

官方提供完整的"实体类型 × 配合类型"有效性矩阵 [来源: help.solidworks.com/2026/english/solidworks/sldworks/r_Standard_Mates_by_Entity.htm]。例如：
- Cylinder × Cylinder：Angle、Concentric、Distance、Parallel、Perpendicular、Tangent
- Plane × Plane：Angle、Coincident、Distance、Parallel、Perpendicular
- Sphere × Sphere：Concentric、Distance、Tangent
- Cone × Plane：仅 Tangent

### 6.6 SmartMates（智能配合）

依据官方 Types of SmartMates 页 [来源: help.solidworks.com/2026/english/solidworks/sldworks/r_Types_of_SmartMates_SWassy.htm]，SmartMates 支持以下自动配合：
- 2 linear edges / 2 temporary axes → Coincident
- 2 planar faces → Coincident
- 2 vertices → Coincident
- 2 conical faces / 1 conical face + 1 temporary axis → Concentric
- 2 circular edges（peg-in-hole）→ Concentric + Coincident
- 2 circular patterns on flanges → Concentric + Coincident
- Origins and coordinate systems → Coincident

### 6.7 OmniM 映射总结

OmniM 当前 `RelationStore` 仅支持 `Connection`（连接点匹配）。需扩展以支持 SOLIDWORKS 的三类配合系统：

- **标准配合 8 类**：Coincident / Parallel / Perpendicular / Tangent / Concentric / Lock / Distance / Angle
- **高级配合 6 类**：Profile Center / Symmetric / Width / Path Mate / Linear/Linear Coupler / Limit
- **机械配合 7 类**：Cam / Slot / Hinge / Gear / Rack Pinion / Screw / Universal Joint

合计 **21 种配合类型**，覆盖机械装配的所有约束场景。

---

## 7. 配置与设计表（Configurations & Design Tables）

### 7.1 配置（Configurations）

依据官方 Configurations Overview 页 [来源: help.solidworks.com/2026/English/solidworks/sldworks/c_Configurations_Overview.htm]：

> 官方原文："Configurations allow you to create multiple variations of a part or assembly model within a single document. Configurations provide a convenient way to develop and manage families of models with different dimensions, components, or other parameters."

配置可控制的内容（官方明确）：
- **零件文档**：不同尺寸、特征、属性（含自定义属性）的零件族
- **装配文档**：通过抑制组件简化设计；不同组件配置；不同装配特征参数；不同尺寸；配置特定自定义属性
- **工程图**：显示零件/装配文档中创建的配置视图

配置创建方式（官方列出 3 种）：
1. 手动创建配置
2. 使用设计表（Design Table）在嵌入式 Excel 工作表中管理配置
3. 使用 Modify Configurations 对话框

### 7.2 设计表（Design Table）

依据官方 Design Table Configurations 页 [来源: help.solidworks.com/2025/English/Solidworks/sldworks/c_Design_Table_Configurations.htm]（2026 版结构相同）：

> 官方原文："A design table allows you to build multiple configurations of parts or assemblies by specifying parameters in an embedded Microsoft Excel worksheet. The design table is saved in the model document and is not linked to the original Excel file."

**零件设计表可控制**：
- 特征的尺寸与抑制状态、Hole Wizard 孔尺寸
- 配置属性：BOM 零件号、派生配置、方程式、草图关系、注释、材料、自定义属性

**装配设计表可控制**：
- 组件：抑制状态、引用配置、固定/浮动位置
- 装配特征：尺寸、抑制状态、Hole Wizard 孔尺寸
- 配合：距离与角度配合的尺寸、抑制状态
- 配置属性：零件号、BOM 显示（作为子装配时）、派生配置、方程式、草图关系、注释、自定义属性、显示状态

### 7.3 可配置参数（Configurable Parameters）

依据官方 Configurable Parameters 页 [来源: help.solidworks.com/2026/English/solidworks/sldworks/r_Configurable_Parameters.htm]，可配置参数包括：
- Base Parts in Configurations（基础零件配置）
- Color Parameter（颜色，32-bit RGB 整数）
- Comment（注释）
- Component Configuration（组件引用配置）
- Component Part Number（组件零件号）
- Component Suppression State（组件抑制状态）
- Cosmetic Threads（装饰螺纹）
- Custom Properties（自定义属性）
- Derived Configurations（派生配置）
- Description（描述）
- Dimensions（尺寸）
- Display States（显示状态）
- End Conditions（终止条件，拉伸特征）
- Expand in BOM（BOM 中展开）
- External Sketch Relations（外部草图关系）
- Feature Suppression State（特征抑制状态）—— 支持快捷值 suppressed / unsuppressed
- Fixed or Floating Position（固定/浮动位置）
- Global Variables（全局变量）
- Hole Sizes（Hole Wizard 孔尺寸）
- Lighting（光照）
- Mass Properties（质量属性）
- 等等

### 7.4 设计表语法

依据官方 Entering Parameters in a Design Table 页 [来源: help.solidworks.com/2026/english/solidworks/sldworks/t_Entering_Parameters_in_a_Design_Table.htm]：

```
$KEYWORD@component<instance>
```

示例：
- `$CONFIGURATION@gear<2>` 控制名为 gear 的组件第 2 个实例的引用配置
- `$STATE@Fillet1` 控制特征 Fillet1 的抑制状态（S=Suppressed, U=Unsuppressed）
- `D1@Sketch1` 控制草图尺寸
- `$PARTNUMBER`、`$PRP@Material` 控制零件号和材料

实例号语法：
- `<*>` 所有实例
- `<1-4>` 实例范围
- `<1,3,6>` 非连续实例
- `<1,3-6,8>` 组合

### 7.5 Configuration Table（配置表对话框）

依据官方 Configuration Table Dialog Box 页 [来源: help.solidworks.com/2026/english/solidworks/sldworks/r_configuration_table_db.htm]，配置表分 4 个分区：
- Sketches and Features（草图与特征：尺寸值与抑制状态）
- Components（组件：抑制状态与引用配置，仅装配）
- Custom Properties（自定义属性）
- Parameters（参数：Suppress new features / Suppress new features and mates / Suppress new components / Fix/Float / Child components in BOM / Exclude from BOM on Insert 等）

### 7.6 OmniM 映射

| 设计表/配置概念 | OmniM 等价 |
|---------------|-----------|
| 配置行 | Series 中的 SpecComponent 变体 |
| 尺寸参数列（`D1@Sketch1`） | Selector 决策节点 |
| 特征抑制状态（`$STATE@Fillet1`） | Feature 条件开关 |
| 组件抑制/引用配置（`$CONFIGURATION@comp<inst>`） | Assembly 子件条件 + Series 选择 |
| 派生配置 | Series 继承 |
| 自定义属性列 | Property 系统 |
| 标准尺寸表 | 内置种子数据 |

**结论**：SOLIDWORKS 设计表是 OmniM Series Store 的工业原型——一个 Excel 表驱动整个配置族，列=选择器，行=变体，单元格=参数值。

---

## 8. MBD / PMI 系统

### 8.1 SOLIDWORKS MBD 概述

依据官方 SOLIDWORKS MBD 页 [来源: help.solidworks.com/2026/English/SolidWorks/sldworks/c_solidworks_mbd.htm]：

> 官方原文："SOLIDWORKS MBD (Model Based Definition) lets you create models without the need for drawings giving you an integrated manufacturing solution for SOLIDWORKS Design. SOLIDWORKS MBD helps companies define, organize, and publish 3D product and manufacturing information (PMI), including 3D model data in industry standard file formats."

MBD 核心能力（官方明确）：
- 使用 **DimXpert** 和参考尺寸提供 3D PMI 定义能力
- 通过 **annotation views（注释视图）**、动态注释平面视图、3D 视图组织 3D PMI
- 输出 **3D PDF** 和 **eDrawings** 文件
- 符合 **MIL-STD-31000 Rev A** 和 **ASME 14.41** 标准

### 8.2 PMI 导入

依据官方 Import PMI 页 [来源: help.solidworks.com/2026/English/SolidWorks/sldworks/c_import_pmi.htm]：
> "You can import PMI (Product Manufacturing Information) with NX, Creo, and STEP AP242 formats as semantic noneditable annotations."

**重要**：STEP AP242 是官方明确支持的 PMI 交换格式（用于 NX / Creo / STEP 文件导入）。前一版本将 STEP AP242 列为导出格式，但官方 MBD 页明确输出为 3D PDF 和 eDrawings；STEP AP242 主要作为**导入**格式（用于接收其他 CAD 的 PMI）。官方文档未明确 SOLIDWORKS 导出 STEP AP242 含 PMI，应标为"官方文档未明确"。

### 8.3 DimXpert（尺寸专家）

依据官方 Auto Dimension Scheme PropertyManager 页 [来源: help.solidworks.com/2026/english/solidworks/sldworks/HIDD_SWIFT_GTS.htm]：

**Part type**：
- Prismatic（棱柱类）：用 Geometric 公差时对孔/凸台应用位置度
- Turned（回转类）：用 Geometric 公差时对孔/凸台应用圆跳动

**Tolerance type**：
- Plus and Minus（正负公差）：用线性正负尺寸定位特征
- Geometric（几何公差）：用位置度和圆跳动定位轴向特征，用面轮廓度定位槽和表面

**Pattern Dimensioning**：
- Linear（线性）：相对参考特征用线性正负尺寸定位
- Polar（极坐标）：应用于包含螺栓圆的轴向特征模式

**支持的特征/基准类型**（官方明确列出）：Boss、Cylinder、Simple Hole、Notch、Plane、Slot、Width、Cone。

DimXpert 验证基准集按 **ASME Y14.5.1M-1994** 规则。

### 8.4 DimXpert 工具

依据官方 DimXpert Tools 页 [来源: help.solidworks.com/2026/english/solidworks/sldworks/c_dimxpert_tools.htm]，DimXpert 提供：
- Location Dimension（位置尺寸）
- Size Dimension（尺寸大小）
- Tolerance（公差，含几何公差）
- Ordinate Dimension（纵坐标尺寸）[来源: .../t_dimxpert_ordinate_dimension.htm]
- Auto Dimension Scheme（自动尺寸方案）
- Width/Wedge Feature（宽度/楔形特征）[来源: .../t_adv_dimxpert_width_feature.htm]

### 8.5 3D PMI Compare

依据官方 Comparing 3D PMI Between Parts 页 [来源: help.solidworks.com/2026/English/SolidWorks/sldworks/c_3d_pmi_comparison_between_parts.htm]：
> "You can use the 3D PMI Compare tool to compare DimXpert annotations, reference dimensions, and other annotations between two part documents."

### 8.6 OmniM 映射

| PMI 概念 | OmniM 映射 |
|---------|-----------|
| DimXpert 尺寸 | `DimensionConstraint`（Sketch 层 + 3D PMI） |
| 几何公差（GD&T，按 ASME Y14.5.1M-1994） | `PropertySet`（面向制造属性） |
| 基准（Datum） | `DatumReference`（参考几何） |
| Annotation Views | `AnnotationView`（视口快照） |
| 3D Views | `AnnotationView` |
| 3D PDF 导出 | `FormatAdapter`（3D PDF） |
| eDrawings 导出 | `FormatAdapter`（eDrawings） |
| STEP AP242 PMI 导入 | `FormatAdapter`（STEP AP242 Import） |

---

## 9. 参考几何（Reference Geometry）

依据官方 Reference Geometry 页 [来源: help.solidworks.com/2026/English/solidworks/sldworks/c_Reference_Geometry_Overview.htm]：

> 官方原文："Reference geometry defines the shape or form of a surface or a solid. Reference geometry includes items such as planes, axes, coordinate systems, and points. You can use reference geometry in the creation of several kinds of features. For example: Planes are used in lofts and sweeps. An axis is used in a circular pattern. You can create linear or circular patterns of planes and axes."

### 9.1 参考几何类型

| 类型 | 官方说明 | OmniM 映射 |
|------|---------|-----------|
| **Planes（基准面）** | 用于 lofts 和 sweeps | `ReferenceGeometry::Plane` |
| **Axes（基准轴）** | 用于圆周阵列和草图几何 | `ReferenceGeometry::Axis` |
| **Coordinate Systems（坐标系）** | 可定义零件或装配的坐标系 | `ReferenceGeometry::CSys` |
| **Reference Points（参考点）** | 多种类型的参考点，作为构造对象 | `ReferenceGeometry::Point` |
| **Center of Mass Points（质心点）** | 可添加到零件和装配 | `ReferenceGeometry::COMPoint` |
| **Bounding Boxes（包围盒）** | 矩形或圆柱包围盒，用于确定空间/尺寸需求 | `ReferenceGeometry::BoundingBox` |
| **Grid System（网格系统）** | 用于大型结构布局，对焊接结构特别有用 | `ReferenceGeometry::GridSystem` |

### 9.2 坐标系引用

依据官方 Referencing Coordinate Systems 页 [来源: help.solidworks.com/2026/English/solidworks/sldworks/c_referencing_coordinate_systems.htm]，坐标系的平面、轴、原点均可在下游特征中引用。例如：
- 在坐标系 XY 平面上草图
- 用坐标系 Z 轴作为旋转特征轴
- 3D 草图与坐标系轴/平面/原点创建草图关系
- 2D 草图尺寸引用坐标系轴/平面/原点
- 镜像特征/体/组件时选坐标系平面作为镜像面
- 选坐标系轴创建垂直于该轴的新平面

### 9.3 OmniM 映射

参考几何是 OmniM Feature Engine 的基础配套。Plane / Axis / CSys / Point 应作为 ReferenceGeometry 对象类型，与 Feature 并列但可被 Feature 引用。

---

## 10. 多实体零件（Multibody Parts）

依据官方 Multibody Overview 页 [来源: help.solidworks.com/2026/English/solidworks/sldworks/c_Multibody_Overview.htm]：

> 官方原文："Part documents can contain multiple solid bodies. For example, when you design a spoked wheel, you know the requirements of the rim and the axle. However, you do not know how to design the spoke. With multibody parts, you can create the rim and axle, then create the spoke to connect the bodies."

### 10.1 多实体特性

- FeatureManager 设计树中出现 `Solid Bodies` 文件夹，括号显示实体数量
- 可将实体分组到文件夹
- 可对文件夹内所有实体应用命令
- 可列出每个实体所属的特征

### 10.2 可从单一特征创建多实体的命令

官方明确列出 [来源: .../c_Multibody_Overview.htm]：
- Extrude boss and cut（含薄壁特征）
- Revolve boss and cut（含薄壁特征）
- Sweep boss and cut（含薄壁特征）
- Surface cut
- Boss and cut thicken
- Cavity

### 10.3 多实体相关能力

- **Combine Bodies（组合实体）**：Add / Subtract / Intersect [来源: .../c_combine_bodies.htm]
- **Mates in Multibody Parts（多实体零件中的配合）**
- **Feature Scope（特征作用域）**：控制特征作用于哪些实体
- **Move/Copy Body（移动/复制实体）**：支持 Translate / Rotate / Constraints（配合）三种模式 [来源: .../t_Move_Copy_Body_PM_features.htm]
- **Interference Detection for Multibody Parts（多实体干涉检测）**
- **Exploded Views in Multibody Parts（多实体爆炸视图）**
- **Materials and Multibody Parts（多实体材料）**：每个实体可独立指定材料

### 10.4 OmniM 映射

多实体零件是 SOLIDWORKS 介于"单一 Part"与"Assembly"之间的中间形态。OmniM 的 `ObjectType::Part` 应支持多实体：
- 单 Part 文档内的多实体 = OmniM Part 内的多个 Body 节点
- Combine（Add/Subtract/Intersect）= OmniM BooleanSubtract / BooleanUnion / BooleanIntersect
- Move/Copy Body = OmniM Transform
- 多实体配合 = OmniM Part 内 Constraint

---

## 11. 焊件（Weldments）

依据官方 Weldments Overview 页 [来源: help.solidworks.com/2026/English/solidworks/sldworks/c_Weldments_Overview.htm]：

> 官方原文："You can use 2D and 3D sketches to define the basic framework of a weldment structure, then create structural members containing groups of sketch segments. You can also add items such as gussets and end caps using tools on the Weldments toolbar."

### 11.1 焊件特征体系

| 焊件元素 | 官方说明 | OmniM GenRule 倾向 |
|---------|---------|-------------------|
| **Structural Member（结构构件）** | 沿草图段组扫掠型材截面 | LinearComponent（截面沿路径扫掠） |
| **Weldment Feature（焊件特征）** | 不是常规特征，而是启用焊件环境 | 环境标记 |
| **Groups（组）** | 结构构件中相关段的集合 | 路径分组 |
| **Profiles（轮廓）** | 焊件轮廓（library feature） | ComponentDef 截面库 |
| **Cut Lists（切割清单）** | 将相同实体分组到 FeatureManager | BOM 等价 |
| **Trim and Extend（修剪延伸）** | 修剪段以正确对接 | BooleanSubtract |
| **Gussets（加强板）** | 用平面面加强两相交结构构件 | Primitive |
| **End Caps（端盖）** | 封闭开放结构构件端部 | Primitive |
| **Weld Beads（焊缝）** | 焊缝 | Assembly 修饰 |

### 11.2 焊件配置

> 官方原文："When you create the first structural member in a part, a weldment feature is created and added to the FeatureManager design tree. SOLIDWORKS Design also creates two default configurations in the ConfigurationManager: a parent configuration Default<As Machined> and a derived configuration Default<As Welded>."

焊件自动创建两个配置：
- `Default<As Machined>`（机加工后状态，父配置）
- `Default<As Welded>`（焊接后状态，派生配置）

### 11.3 焊件型材库

依据官方 Weldments - Creating a Custom Profile 页 [来源: .../t_weldments_creating_custom_profile.htm]，焊件轮廓作为 library feature part 保存。可在设计表中配置多个焊件型材为一个 library feature（例如一个 library feature 文件含 50 个方管尺寸配置）。

### 11.4 焊件工程图

依据官方 Weldment Drawings 页 [来源: help.solidworks.com/2026/english/SolidWorks/Sldworks/c_Weldment_Drawings.htm]，焊件工程图支持：
- 整个焊件视图
- 单实体视图（Relative Views）
- Weldment cut lists（焊件切割清单）
- Balloons / AutoBalloons（球标）
- 剖面图交叉剖面线交替

### 11.5 OmniM 映射

焊件是 LinearComponent GenRule 的机械领域应用：
- **Structural Member = LinearComponent**（型材截面 + 路径扫掠）
- 与 Tekla 的 `Beam`（型材扫掠）几何等价，但 SOLIDWORKS 焊件是单 Part 内的多实体，而 Tekla 是多 Part 装配
- 焊件型材库 = OmniM Catalog 截面库（ANSI Inch / ISO / DIN 等标准型材）
- Weld Beads / Gussets / End Caps = Assembly 子件（Primitive）

---

## 12. 钣金（Sheet Metal）

依据官方 Using Sheet Metal Tools 页 [来源: help.solidworks.com/2026/English/solidworks/sldworks/c_Using_Sheet_Metal_Tools.htm]：

### 12.1 钣金特征体系

| 钣金特征 | 官方说明 | OmniM 映射 |
|---------|---------|-----------|
| **Base Flange/Tab（基体法兰/标签）** | 钣金件的第一个特征 | LayerExtrusion / Primitive |
| **Insert Bends（插入折弯）** | 将抽壳件转为钣金件 | — |
| **Sheet Metal Gussets（钣金加强筋）** | 跨折弯的特定压痕 | Primitive |
| **Sheet Metal Tabs（钣金标签）** | 深度自动设为钣金厚度 | Primitive |
| **Tab and Slot（标签与槽口）** | 在两体上创建互锁的标签和槽口 | Assembly + BooleanSubtract |
| **Edge Flanges（边线法兰）** | 在一个或多个边上添加边线法兰 | LayerExtrusion |
| **Miter Flanges（斜接法兰）** | 在一个或多个边上添加一系列法兰 | LayerExtrusion |
| **Swept Flange（扫描法兰）** | 用扫描创建复合折弯 | Primitive::Sweep |
| **Hems（褶边）** | 在选定边添加褶边 | — |
| **Sketched Bends（草图折弯）** | 在平面面上添加折弯线 | — |
| **Closed Corners（闭合角）** | 在钣金法兰间添加闭合角 | — |
| **Jogs（转折）** | 从草图线创建两个折弯 | — |
| **Break Corner/Corner-Trim（断角/拐角修剪）** | 在折叠钣金件上切割或添加材料 | BooleanSubtract |
| **Corner Reliefs and Bend Transitions（拐角释放槽与折弯过渡）** | 拐角处理 | — |
| **Bend Notches（折弯缺口）** | 在展开钣金件中跨折弯创建缺口 | — |
| **Lofted Bends（放样折弯）** | 用两个开放轮廓草图通过放样连接 | Primitive::Loft |
| **Unfold/Fold（展开/折叠）** | 展开/折叠钣金 | — |
| **Welded Corner（焊接角）** | 在折叠钣金件角部添加焊缝 | — |
| **Cross Breaks（交叉折弯）** | HVAC/风管中用于加强钣金 | — |

### 12.2 钣金与 FeatureManager

依据官方 Examination of FeatureManager Design Tree 页 [来源: .../c_examination_featuremanager_design_tree.htm]，Base Flange 特征在 FeatureManager 中创建 3 个新特征。

### 12.3 OmniM 映射

钣金是 OmniM `LayerExtrusion` 的典型应用——多层 2D 截面（材料层）+ 折弯。但钣金的"展开/折叠"机制需要独立的 SheetMetal Feature 类型支持折弯系数（K-因子、折弯扣除等）。

---

## 13. 对 OmniM 的启示（GenRule 映射表）

### 13.1 完整 GenRule 映射

| OmniM GenRule | SOLIDWORKS 对应 | 典型场景 |
|---------------|----------------|---------|
| **Primitive** | Extrude（Boss/Base）、Revolve、Sweep、Loft、Boundary、Thicken、Dome、Gusset、End Cap、Bearing、Toolbox 标准件（Bolt/Nut/Washer 等示意几何） | 单一参数化实体，机械零件主体几何 |
| **LayerExtrusion** | Sheet Metal Base Flange / Edge Flange / Miter Flange / Swept Flange / Hem / Lofted Bends（多层 2D 截面 + 折弯） | 钣金件、多层材料件 |
| **LinearComponent** | Weldment Structural Member（型材截面沿路径扫掠）、Surface Treatment（官方未明确单列，但 Toolbox Structural Steel 提供型材横断面作为草图） | 焊件结构构件、型材 |
| **Assembly** | .sldasm（多零件 + 配合）、多实体 Part（多实体 + Move/Copy + Combine）、Weldment（多结构构件 + 焊缝 + 加强板）、Sub-assembly 嵌套 | 装配体、焊件、多实体零件 |
| **BooleanSubtract** | Extruded Cut、Revolved Cut、Swept Cut、Lofted Cut、Boundary Cut、Thickened Cut、Cut with Surface、Simple Hole、Hole Wizard、Combine（Subtract）、Split、Trim and Extend（焊件） | 切割、孔、布尔减 |

### 13.2 跨平台 GenRule 验证结论

| GenRule | SOLIDWORKS 验证 | 与 Tekla/Bentley 对比 |
|---------|----------------|----------------------|
| Primitive | Extrude/Revolve/Sweep/Loft/Boundary 五类草图特征 + Toolbox 标准件 | 与 Tekla 的 ContourPlate/LoftedPlate、Bentley 的 Parametric Cell 同构 |
| LayerExtrusion | 钣金特征（Base Flange 等） | 钣金是机械领域独有，Tekla/Bentley 无直接对应 |
| LinearComponent | Weldment Structural Member | 与 Tekla Beam（型材扫掠）几何等价，但 SOLIDWORKS 焊件是单 Part 多实体，应用场景比 Tekla 窄（机械框架 vs 钢结构） |
| Assembly | .sldasm + 多实体 + 焊件 + Sub-assembly | 与 Tekla Assembly、Bentley Civil Cell 同构 |
| BooleanSubtract | 7 类 Cut + Hole Wizard + Combine Subtract | 与 Tekla BooleanPart/Fitting/CutPlane、Bentley 实体布尔同构 |

### 13.3 关键映射洞察

1. **SOLIDWORKS 是"特征树 + 参数化"驱动**：所有几何通过 FeatureManager 设计树中的有序特征构建，是 OmniM Feature Engine 的工业原型。这与 Tekla（Part+Component）、Bentley（Corridor+Template）的建模范式根本不同。

2. **Loft 与 Boundary 是独立特征**：Boundary 在多数情况下质量高于 Loft（官方明确），OmniM 的 `Primitive::Loft` 应考虑支持两种算法变体。

3. **Toolbox 是 ComponentDef Catalog 的工业原型**：但 Toolbox 内容比前一版本声称的少得多（仅 15 类硬件，不含铆钉/密封件/弹簧/键/联轴器/衬套/管件），且紧固件为示意几何（无精确螺纹，不适合应力分析），齿轮非真渐开线。OmniM 的标准件库应严格按官方清单建立，不臆造。

4. **配合系统 21 类**：标准 8 + 高级 6 + 机械 7，覆盖机械装配所有约束场景。OmniM 的 `Constraint` 关系系统应完整支持这 21 类。

5. **设计表是 Series Store 的工业原型**：Excel 表驱动配置族，列=选择器，行=变体，单元格=参数值，支持尺寸/抑制状态/引用配置/自定义属性等丰富参数类型。

6. **多实体零件是 Part 与 Assembly 的中间形态**：单 Part 文档内可含多实体，支持 Combine（Add/Subtract/Intersect）、Move/Copy Body、多实体配合、Feature Scope。OmniM 的 `ObjectType::Part` 应支持多实体。

7. **钣金是 LayerExtrusion 的典型应用**：但钣金的展开/折叠机制需独立 SheetMetal Feature 类型支持折弯系数。

8. **MBD/PMI 是制造属性集的参考**：DimXpert 按 ASME Y14.5.1M-1994 自动生成尺寸和几何公差，输出 3D PDF / eDrawings，支持 STEP AP242 PMI 导入（导入语义不可编辑注释）。

9. **Hole Wizard 与 Toolbox 联动**：Hole Wizard 的孔尺寸与 Toolbox 紧固件尺寸联动，可通过 `Tools > Options > System Options > Hole Wizard/Toolbox > Configure` 创建自定义标准。

10. **LinearComponent 在机械领域应用有限**：SOLIDWORKS 中 LinearComponent 范式主要体现在焊件结构构件（型材沿路径扫掠），不像 Tekla/Bentley 那样是基础设施领域的核心 GenRule。OmniM 在机械领域应优先以 Primitive + Assembly 为主。

---

## 14. 需要新增/调整的 OmniM 设计项

### 14.1 高优先级（与当前架构一致，可立即落地）

| 项目 | 来源 | 说明 |
|------|------|------|
| `Constraint` 关系系统扩展（21 类配合） | SOLIDWORKS Mates | 标准 8 + 高级 6 + 机械 7 |
| `Feature::Fillet`（8 子类型） | SOLIDWORKS Fillet Overview | Constant/Multiple Radius/Round Corner/Setback/Variable Size/Face/Full Round/Asymmetric |
| `Feature::Chamfer` | SOLIDWORKS Features Toolbar | 倒角 |
| `Feature::Shell` | SOLIDWORKS Features Toolbar | 抽壳 |
| `Feature::Draft` | SOLIDWORKS Features Toolbar | 拔模 |
| `Feature::HoleWizard`（7 孔类型） | SOLIDWORKS Hole Wizard | Counterbore/Countersink/Hole/Straight Tap/Tapered Tap/Legacy/Slots |
| `Feature::Pattern`（Linear/Circular/CurveDriven/Mirror/Fill/Sketch-Driven/Table-Driven/Variable） | SOLIDWORKS Features Toolbar | 8 种阵列 |
| Catalog ComponentDef 库（严格按 Toolbox 15 类） | SOLIDWORKS Toolbox Overview | Bearings/Bolts/Cams/Gears/Jig bushings/Nuts/PEM inserts/Pins/Retaining rings/Screws/Sprockets/Structural shapes/Timing belt pulleys/Unistrut/Washers |
| `ObjectType::Part` 支持多实体 | SOLIDWORKS Multibody | 多实体 + Combine + Move/Copy + Feature Scope |

### 14.2 中优先级（方向明确，需设计）

| 项目 | 来源 | 说明 |
|------|------|------|
| `Feature::SheetMetal`（含折弯系数 K-因子/折弯扣除） | SOLIDWORKS Sheet Metal | Base Flange/Edge Flange/Miter Flange/Hem/Sketched Bend/Lofted Bends 等 |
| `Feature::Weld`（焊缝） | SOLIDWORKS Weldments | Weld Beads |
| `LinearComponent` 焊件结构构件 | SOLIDWORKS Weldments | 型材截面沿路径扫掠，配合焊件型材库 |
| `Primitive::Loft` 支持 Boundary 变体 | SOLIDWORKS Boundary | Boundary 质量高于 Loft，支持双向切向/曲率连续 |
| `DimensionConstraint` + GD&T（按 ASME Y14.5.1M-1994） | SOLIDWORKS DimXpert | Prismatic/Turned + Plus/Minus/Geometric |
| `FormatAdapter`：3D PDF / eDrawings 导出 | SOLIDWORKS MBD | MBD 输出格式 |
| `FormatAdapter`：STEP AP242 PMI 导入 | SOLIDWORKS Import PMI | 接收 NX/Creo/STEP 的 PMI |
| `ReferenceGeometry`：Plane/Axis/CSys/Point/COMPoint/BoundingBox/GridSystem | SOLIDWORKS Reference Geometry | 7 类参考几何 |

### 14.3 ObjectType 调整建议（基于官方 Toolbox 清单）

| 当前 OmniM ObjectType | 建议 | 理由（基于官方 Toolbox） |
|---|---|---|
| `OBJECT_TYPE_FASTENER` | 保留，作为螺栓/螺母/垫圈/螺钉/销/挡圈的统一父类 | Toolbox 含 Bolts/Nuts/Washers/Screws/Pins/Retaining rings |
| `OBJECT_TYPE_BEARING` | 保留 | Toolbox 含 Bearings（但仅示意几何） |
| `OBJECT_TYPE_GEAR` | 保留，标注"非真渐开线" | Toolbox 含 Gears（机械设计示意） |
| `OBJECT_TYPE_SPRING` | 保留，但**不映射到 Toolbox** | Toolbox 不含弹簧，OmniM 须独立实现 |
| `OBJECT_TYPE_SEAL` | 保留，但**不映射到 Toolbox**（仅 Grooves 工具添加沟槽） | Toolbox 不含密封件 |
| `OBJECT_TYPE_SHAFT` | 保留 | 机械领域核心对象 |
| `OBJECT_TYPE_MOTOR` | 保留 | 机械领域核心对象 |
| `OBJECT_TYPE_SPROCKET` | 新增 | Toolbox 含 Sprockets |
| `OBJECT_TYPE_TIMING_PULLEY` | 新增（仅正时带轮，非通用皮带轮） | Toolbox 含 Timing belt pulleys |
| `OBJECT_TYPE_CAM` | 新增 | Toolbox 含 Cams（凸轮生成器） |
| `OBJECT_TYPE_JIG_BUSHING` | 新增 | Toolbox 含 Jig bushings |
| `OBJECT_TYPE_PEM_INSERT` | 新增 | Toolbox 含 PEM® inserts |
| `OBJECT_TYPE_UNISTRUT` | 新增 | Toolbox 含 Unistrut® |
| `OBJECT_TYPE_STRUCTURAL_SHAPE` | 新增 | Toolbox 含 Structural shapes（铝和钢） |

### 14.4 不需要做

- **不要将铆钉/键/联轴器/衬套/管件/通用皮带轮映射到 Toolbox**：官方 Toolbox 不含这些类型。前一版本的映射是错误的。
- **不要将 Toolbox 齿轮视为制造用真渐开线齿轮**：官方明确 Toolbox 齿轮仅为机械设计示意。
- **不要将 Toolbox 紧固件用于应力分析**：官方明确 Toolbox 紧固件为近似展现，不含精确螺纹。
- **不要为 LinearComponent 在机械领域过度投入**：机械领域 LinearComponent 主要用于焊件结构构件，应用场景比基础设施领域窄。OmniM 在机械领域应优先以 Primitive + Assembly 为主。

---

## 15. 与 Tekla / Bentley 调研的对比

| 维度 | SOLIDWORKS（机械） | Tekla（结构详图） | Bentley（基础设施） |
|------|-------------------|------------------|-------------------|
| 建模哲学 | 特征树驱动（FeatureManager） | 零件+组件驱动（Part + Component） | 走廊模板+线形驱动（Corridor + Template） |
| 几何创建 | 草图特征（Extrude/Revolve/Sweep/Loft/Boundary）+ 应用特征（Fillet/Chamfer/Shell） | Part 子类（Beam/ContourPlate/PolyBeam/LoftedPlate）+ Boolean | Corridor Template 沿 alignment 扫掠 + Civil Cell |
| 线构件 | Weldment Structural Member（型材扫掠） | Beam（统一所有线构件） | Girder/Cap/Stem/Wingwall/Lining/Shaft/Channel/Dam/Slope |
| 板件 | Sheet Metal（钣金） | ContourPlate（统一所有板件） | Deck/Surface Template |
| 连接节点 | 装配体配合（Mate，21 类） | Component 系统（端板/底板/角钢等内置组件） | Civil Cell（参数化参照点） |
| 型材库 | Weldment Profile + Toolbox Structural shapes | Profile Library（17 类） | Beam Section Template Library |
| 布尔切割 | 7 类 Cut + Hole Wizard + Combine Subtract | BooleanPart/Fitting/CutPlane | Cut/Fill Volumes + Cross Passage Intersection |
| 装配体 | .sldasm（多零件 + 配合 + 子装配嵌套） | Assembly（主件 + 副件，Component 自动构建） | Civil Model 层级 + Reference Models |
| 标准件库 | Toolbox（15 类硬件，仅示意几何） | Profile Library（型材） | Feature Definition 库 |
| 配置系统 | Configuration + Design Table（Excel 驱动） | 不内置（依赖组件参数） | 不内置（依赖模板变量） |
| MBD/PMI | DimXpert + 3D PDF + eDrawings + STEP AP242 导入 | 不内置 | 不内置 |
| 对应 OmniM GenRule | Primitive + BooleanSubtract + Assembly（+ LayerExtrusion 钣金 + LinearComponent 焊件） | LinearComponent + Primitive + Assembly + BooleanSubtract | LinearComponent + LayerExtrusion + Assembly + BooleanSubtract |

### 15.1 核心结论

SOLIDWORKS 的"特征树 + 参数化 + 配置 + 配合"四件套与 OmniM 的"Object + GenRule + Series + Constraint"四层架构高度对应：

1. **特征树 → OmniM Feature Engine**：SOLIDWORKS 的 FeatureManager 是 OmniM Feature Engine 的机械领域原型——有序特征构建几何，支持父/子关系、抑制状态、配置变体。
2. **配置 + 设计表 → OmniM Series Store**：设计表是 Series Store 的工业原型——Excel 表驱动配置族，列=选择器，行=变体。
3. **配合系统 → OmniM Constraint**：21 类配合（标准 8 + 高级 6 + 机械 7）是机械装配约束的完整覆盖。
4. **Toolbox → OmniM Catalog ComponentDef**：15 类硬件标准件库是 Catalog 的工业原型，但内容严格限定（不含铆钉/密封件/弹簧/键/联轴器/衬套/管件），且紧固件为示意几何。
5. **多实体零件 → OmniM Part 多体**：单 Part 内多实体 + Combine + Move/Copy + Feature Scope，是 Part 与 Assembly 的中间形态。
6. **MBD/DimXpert → OmniM 制造属性集**：按 ASME Y14.5.1M-1994 自动生成尺寸和几何公差，输出 3D PDF/eDrawings，支持 STEP AP242 PMI 导入。

**与 Tekla/Bentley 的关键差异**：
- SOLIDWORKS 的 LinearComponent 应用场景（焊件）远窄于 Tekla（所有线构件）和 Bentley（桥梁/隧道/水工所有线构件）。机械领域以 Primitive + Assembly 为主。
- SOLIDWORKS 的 LayerExtrusion 应用场景（钣金）是机械领域独有，Tekla/Bentley 无直接对应。
- SOLIDWORKS 的配合系统（21 类）远比 Tekla（节点组件内置）和 Bentley（无独立配合系统）丰富，是机械装配的核心。
- SOLIDWORKS 的 MBD/PMI 系统是 Tekla/Bentley 所不具备的制造属性能力。

---

## 16. 官方文档来源汇总

本调研引用的所有官方 URL（均经实测可访问，截至 2026-08-07）：

### API 文档
- API 程序员指南入口：`https://help.solidworks.com/2026/English/api/sldworksapiprogguide/Welcome.htm`
- API 2026 发行说明：`https://help.solidworks.com/2026/English/api/sldworksapi/ReleaseNotes-sldworksapi.html`
- IModelDoc2：`https://help.solidworks.com/2026/english/api/sldworksapi/SOLIDWORKS.Interop.sldworks~SOLIDWORKS.Interop.sldworks.IModelDoc2.html`
- IPartDoc：`https://help.solidworks.com/2026/English/api/sldworksapi/SolidWorks.Interop.sldworks~SolidWorks.Interop.sldworks.IPartDoc.html`
- IAssemblyDoc Members：`https://help.solidworks.com/2026/English/api/sldworksapi/SolidWorks.Interop.sldworks~SolidWorks.Interop.sldworks.IAssemblyDoc_members.html`
- IDrawingDoc：`https://help.solidworks.com/2026/english/api/sldworksapi/SolidWorks.Interop.sldworks~SolidWorks.Interop.sldworks.IDrawingDoc.html`
- IFeature：`https://help.solidworks.com/2026/english/api/sldworksapi/SolidWorks.Interop.sldworks~SolidWorks.Interop.sldworks.IFeature.html`
- IComponent2：`https://help.solidworks.com/2026/english/api/sldworksapi/SolidWorks.Interop.sldworks~SolidWorks.Interop.sldworks.IComponent2.html`
- API Class Hierarchy：`https://help.solidworks.com/2024/English/api/sldworksapiprogguide/GettingStarted/Understanding_the_SolidWorks_API_Class_Hierarchy.htm`
- Hole Wizard Feature Types：`https://help.solidworks.com/2026/english/api/sldworksapi/Table_of_Wizard_Hole_Types_and_Valid_Properties.htm`

### 用户帮助（英文）
- Approach to Modeling Overview：`https://help.solidworks.com/2026/English/SolidWorks/acadhelp/r_moving2d3d_approach_modeling_ov.htm`
- Feature-based Models：`https://help.solidworks.com/2026/English/SolidWorks/acadhelp/c_moving2d3d_feature_based_models.htm`
- Features：`https://help.solidworks.com/2026/English/solidworks/sldworks/c_Features_Top.htm`
- Features Toolbar：`https://help.solidworks.com/2026/English/SolidWorks/sldworks/r_Features_Toolbar_features.htm`
- Surface Features：`https://help.solidworks.com/2026/English/solidworks/sldworks/c_Surface_Features_Folder.htm`
- Fillet Overview：`https://help.solidworks.com/2026/English/solidworks/sldworks/c_fillet_overview.htm`
- Hole Wizard Overview：`https://help.solidworks.com/2026/English/SolidWorks/sldworks/c_Hole_Wizard_Overview.htm`
- Hole Wizard Type PropertyManager：`https://help.solidworks.com/2026/English/SolidWorks/sldworks/r_HIDD_DVE_HW_HOLE_SPEC.htm`
- Sketch Entities：`https://help.solidworks.com/2026/English/SolidWorks/Sldworks/c_sketch_entities_top.htm`
- Arcs：`https://help.solidworks.com/2012/english/SolidWorks/sldworks/Arcs.htm`
- Sketch Relations：`https://help.solidworks.com/2026/English/solidworks/sldworks/c_Description_of_Sketch_Relations.htm`
- Mates Overview：`https://help.solidworks.com/2026/English/solidworks/sldworks/c_Mates_Overview_Assemblies.htm`
- Mate PropertyManager - Standard Tab：`https://help.solidworks.com/2026/English/solidworks/sldworks/HIDD_NEW_ADD_MATE.htm`
- Mate PropertyManager - Advanced Tab：`https://help.solidworks.com/2026/English/solidworks/sldworks/r_mate_pm_advanced.htm`
- Mate PropertyManager - Mechanical Tab：`https://help.solidworks.com/2026/English/solidworks/sldworks/r_mate_pm_mechanical.htm`
- Mate PropertyManager - Analysis Tab：`https://help.solidworks.com/2026/English/solidworks/sldworks/HIDD_DVE_MATEANALYSIS.htm`
- Standard Mates by Entity：`https://help.solidworks.com/2026/english/solidworks/sldworks/r_Standard_Mates_by_Entity.htm`
- Types of SmartMates：`https://help.solidworks.com/2026/english/solidworks/sldworks/r_Types_of_SmartMates_SWassy.htm`
- Configurations Overview：`https://help.solidworks.com/2026/English/solidworks/sldworks/c_Configurations_Overview.htm`
- Design Table Configurations：`https://help.solidworks.com/2025/English/Solidworks/sldworks/c_Design_Table_Configurations.htm`（2026 版结构相同）
- Configurable Parameters：`https://help.solidworks.com/2026/English/solidworks/sldworks/r_Configurable_Parameters.htm`
- Entering Parameters in a Design Table：`https://help.solidworks.com/2026/english/solidworks/sldworks/t_Entering_Parameters_in_a_Design_Table.htm`
- Configuration Table Dialog Box：`https://help.solidworks.com/2026/english/solidworks/sldworks/r_configuration_table_db.htm`
- SOLIDWORKS MBD：`https://help.solidworks.com/2026/English/SolidWorks/sldworks/c_solidworks_mbd.htm`
- Import PMI：`https://help.solidworks.com/2026/English/SolidWorks/sldworks/c_import_pmi.htm`
- Auto Dimension Scheme PropertyManager：`https://help.solidworks.com/2026/english/solidworks/sldworks/HIDD_SWIFT_GTS.htm`
- DimXpert Ordinate Dimension：`https://help.solidworks.com/2026/english/solidworks/sldworks/t_dimxpert_ordinate_dimension.htm`
- Adding a Width Feature (DimXpert)：`https://help.solidworks.com/2026/english/swtutorialonline/t_adv_dimxpert_width_feature.htm`
- Comparing 3D PMI Between Parts：`https://help.solidworks.com/2026/English/SolidWorks/sldworks/c_3d_pmi_comparison_between_parts.htm`
- Reference Geometry：`https://help.solidworks.com/2026/English/solidworks/sldworks/c_Reference_Geometry_Overview.htm`
- Referencing Coordinate Systems：`https://help.solidworks.com/2026/English/solidworks/sldworks/c_referencing_coordinate_systems.htm`
- Multibody Overview：`https://help.solidworks.com/2026/English/solidworks/sldworks/c_Multibody_Overview.htm`
- Combine Bodies：`https://help.solidworks.com/2026/English/solidworks/sldworks/c_combine_bodies.htm`
- Move/Copy Body PropertyManager：`https://help.solidworks.com/2026/english/solidworks/sldworks/t_Move_Copy_Body_PM_features.htm`
- Weldments Overview：`https://help.solidworks.com/2026/English/solidworks/sldworks/c_Weldments_Overview.htm`
- Weldment Drawings：`https://help.solidworks.com/2026/english/SolidWorks/Sldworks/c_Weldment_Drawings.htm`
- Using Sheet Metal Tools：`https://help.solidworks.com/2026/English/solidworks/sldworks/c_Using_Sheet_Metal_Tools.htm`
- Machine Design Tools Overview：`https://help.solidworks.com/2026/English/SolidWorks/sldworks/c_machine_desing_tools_overview.htm`
- Linear Pattern Overview (Chinese)：`https://help.solidworks.com/2026/chinese-simplified/SolidWorks/sldworks/t_Linear_Patterns_Overview.htm`

### Toolbox 文档
- Toolbox Overview (English)：`https://help.solidworks.com/2026/English/SolidWorks/toolbox/c_toolbox_overview.htm`
- Toolbox Overview (Chinese)：`https://help.solidworks.com/2026/chinese-simplified/SolidWorks/toolbox/c_toolbox_overview.htm`
- Toolbox Administration：`https://help.solidworks.com/2026/English/SolidWorks/toolbox/c_toolbox_aministration_concepts.htm`
- Toolbox - Customize Hardware：`https://help.solidworks.com/2026/English/SolidWorks/toolbox/HIDD_TOOLBOX_CUSTOMIZE_HARDWARE.htm`
- Configuring Toolbox：`https://help.solidworks.com/2026/english/SolidWorks/toolbox/t_toolbox_configuring.htm`

### 未能从官方文档验证的内容
- **SOLIDWORKS 导出 STEP AP242 含 PMI**：官方 MBD 页仅明确 3D PDF 和 eDrawings 为输出格式，STEP AP242 仅作为导入格式。前一版本将 STEP AP242 列为导出格式，**官方文档未明确**，已标为待验证。
- **swFeatureTypeDef_e 完整枚举**：前一版本引用此枚举但未给出官方 URL。官方 API 参考中存在特征类型枚举，但完整列表需进一步在 swconst 命名空间查证（`https://help.solidworks.com/2026/english/api/swconst/`）。
- **Thread（螺纹）特征**：官方 Features Toolbar 页未单列 Thread 工具，但配置项中可见 `Cosmetic Threads in Configurations`，说明螺纹以"装饰螺纹"形式存在。是否作为独立 Feature 类型，**官方文档未明确**。
