# Tekla Structures 内置构件逻辑研究

> 调研日期：2026-08-07
> 来源：Tekla Open API 2025/2026 Reference (developer.tekla.com)、Tekla Structures User's Guide (support.tekla.com)、Tekla OpenAPI 二次开发权威参考、TSOpenAPIExamples (GitHub)
> 目标：分析钢结构/混凝土结构详图 CAD 领域的内置构件底层逻辑，映射到 OmniM 的 GenRule 体系
> 状态：完成

---

## 1. 平台概述

Tekla Structures（Trimble）是钢结构与混凝土结构详图设计（detailing）领域的行业标准 BIM 软件。其建模哲学与 Revit（族驱动）和 SOLIDWORKS（特征树驱动）均不同，采用 **Part-based + Component-based** 双层建模体系：

- **Part-based（零件驱动）**：所有物理实体均为 `Part` 的子类（`Beam` / `ContourPlate` / `PolyBeam` / `Brep` / `LoftedPlate`）。零件由"型材截面（Profile）+ 路径或轮廓 + 材质 + 位置"四要素定义，几何由内置几何引擎根据型材库展开并扫掠/拉伸生成。
- **Component-based（节点驱动）**：钢连接、混凝土节点、基础、楼梯等通过"组件（Component / Connection / Detail / Seam）"过程化生成多个子零件（板、螺栓、焊缝、加劲肋、切割）。组件以主件（Primary）+ 副件（Secondaries）+ 输入点（Positions）为输入，运行时调用 `Run()` 方法生成一组 `Part` 与 `BoltGroup` / `Weld` / `BooleanPart`，是 Tekla 最具特色的"过程化节点"机制。
- **Profile Library（型材库）**：截面通过 `Profile.ProfileString`（如 `"IPE300"`、`"HEA200"`、`"PL10"`、`"D20"`）引用，型材库按标准（AISC/ISO/DIN/GB/JIS）组织。截面类型分为 0–16 类（自由截面/I 型/U 型/L 型/圆钢/圆管/矩形管/T 型/矩形条/Z 型/C 型/Omega/Sigma/钢轨/钢筋），见 `import.asc` 文件描述。

### 1.1 Open API 命名空间架构

```
Tekla.Structures.Model           ← 模型层（Beam/ContourPlate/Connection/BoltGroup/...）
├── Part (抽象基类)
│   ├── Beam           ← 直线起止点 + 型材截面扫掠
│   ├── PolyBeam       ← 多段折线轮廓 + 型材截面扫掠
│   ├── ContourPlate   ← 平面多边形轮廓 + 板厚拉伸
│   ├── Brep           ← 边界表示零件（StartPoint/EndPoint + Profile）
│   └── LoftedPlate    ← 双导线（Polycurve）放样板
├── BaseComponent (抽象)
│   ├── Connection     ← 连接（主件 + N 个副件）
│   ├── Component      ← 通用组件
│   ├── Detail         ← 细节（单主件 + 输入点）
│   └── Seam           ← 缝（两件之间的连续节点）
├── BoltGroup (抽象)
│   ├── BoltArray      ← 矩形阵列螺栓
│   ├── BoltCircle     ← 圆周阵列螺栓
│   └── BoltXYList     ← 任意坐标列表螺栓
├── Boolean (抽象)
│   ├── BooleanPart    ← 零件布尔（切割/添加）
│   ├── Fitting        ← 平面端部拟合（裁切）
│   └── CutPlane       ← 平面切割
├── Reinforcement (抽象)
│   ├── SingleRebar        ← 单根钢筋
│   ├── RebarGroup         ← 直线分布钢筋组
│   ├── CircleRebarGroup   ← 圆周分布钢筋组
│   ├── CurvedRebarGroup   ← 沿曲线分布钢筋组
│   ├── RebarMesh          ← 钢筋网
│   ├── RebarStrand        ← 预应力钢绞线
│   └── RebarSet           ← 现代钢筋集（带 modifier/splitter）
├── BaseWeld
│   ├── Weld           ← 两件间焊缝
│   └── PolygonWeld    ← 多边形焊缝（沿拾取点序列）
└── Assembly / EdgeChamfer / SurfaceObject / LoftedPlate / BentPlate ...

Tekla.Structures.Plugins        ← 插件层
├── PluginBase          ← 通用插件基类
└── ConnectionBase      ← 连接/细节/缝插件基类（Primary/Secondaries/Positions + Run）
```

### 1.2 ModelObjectEnum 全量分类（节选）

`ModelObject.ModelObjectEnum` 枚举列出所有具体模型对象类型，是查询模型的过滤器。关键值（2026 版）：

| 值 | 成员 | 中文 | GenRule 倾向 |
|----|------|------|--------------|
| 1  | BEAM | 直线梁/柱 | LinearComponent |
| 2  | POLYBEAM | 折线/曲线梁 | LinearComponent |
| 3  | CONTOURPLATE | 轮廓板 | Primitive（拉伸体） |
| 4  | BOOLEANPART | 布尔切割/添加 | BooleanSubtract |
| 5  | FITTING | 端部拟合 | BooleanSubtract |
| 6  | CUTPLANE | 平面切割 | BooleanSubtract |
| 7  | SURFACE_TREATMENT | 表面处理 | LayerExtrusion（薄层） |
| 8  | WELD / 35 POLYGON_WELD / 36 LOGICAL_WELD | 焊缝 | Assembly（修饰） |
| 9  | ASSEMBLY | 装配 | Assembly |
| 10 | SINGLEREBAR | 单根钢筋 | LinearComponent |
| 11 | REBARGROUP | 钢筋组 | LinearComponent（阵列） |
| 12 | REBARMESH | 钢筋网 | LayerExtrusion（双向阵列） |
| 13 | REBARSTRAND | 预应力钢绞线 | LinearComponent |
| 15-17 | BOLT_ARRAY / BOLT_CIRCLE / BOLT_XYLIST | 螺栓组 | Assembly |
| 24 | CONNECTION | 连接 | Assembly |
| 25 | COMPONENT | 通用组件 | Assembly |
| 26 | SEAM | 缝节点 | Assembly |
| 27 | DETAIL | 细节节点 | Assembly |
| 37 | CIRCLEREBAR / 44 CIRCLE_REBARGROUP | 圆钢筋/圆钢筋组 | LinearComponent |
| 45 | CURVED_REBARGROUP | 曲线钢筋组 | LinearComponent |
| 46 | EDGE_CHAMFER | 边倒角 | BooleanSubtract（修饰） |
| 47 | POUR_OBJECT | 浇筑对象 | Assembly（合并体） |
| 52 | BREP | Brep 零件 | Primitive |
| 55 | REBAR_SET | 钢筋集（现代） | LinearComponent（带 modifier） |
| 69 | LOFTED_PLATE | 放样板 | Primitive（Loft） |
| 70 | LEG_SURFACE_OBJECT | 腿面（放样融合） | Primitive（Loft） |

### 1.3 与 OmniM 的总体映射

| Tekla 概念 | OmniM 等价 | 说明 |
|------------|-----------|------|
| Part（零件） | `ObjectType::Part` | 物理实体的统一抽象 |
| Beam / Column（梁/柱） | `ObjectType::Beam` / `Column` | Tekla 中柱就是垂直方向的 Beam |
| ContourPlate（轮廓板） | `ObjectType::Plate` | 任意多边形板 |
| Profile + StartPoint/EndPoint | `LinearComponentGen` 输入 | 截面 + 路径 |
| ContourPlate 的 Contour + Profile | `Primitive::Extrusion` | 轮廓 + 厚度方向拉伸 |
| LoftedPlate 的 BaseCurves | `Primitive::Loft` | 双导线放样 |
| Connection / Component | `ObjectType::Connection` + `GenRule::Assembly` | 过程化节点 |
| BoltGroup | `ObjectType::Fastener`（螺栓阵列） | 螺栓组 |
| BooleanPart / Fitting / CutPlane | `GenRule::BooleanSubtract` | 布尔切割 |
| RebarGroup / RebarSet | `ObjectType::Rebar` + `GenRule::LinearComponent` | 沿路径扫掠的圆截面 |
| Profile Library | Catalog ComponentDef 库 | 型材库 |
| Component Input（Primary/Secondaries/Positions） | ConnectorDef + 输入参数 | 节点输入契约 |

---

## 2. 基础构件 (Base Parts)

所有物理实体均继承自 `Part`。`Part` 提供公共属性：`Name` / `Class` / `Profile` / `Material` / `Finish` / `Position` / `PartNumber` / `AssemblyNumber` / `CastUnitType` / `PourPhase` / `DeformingData`。

### 2.1 梁 Beam（Beam 类，型材扫掠）

`Beam` 是 `Part` 的密封子类，代表一根有起点和终点的线性构件——包括钢梁、混凝土梁、支撑、柱（柱是垂直方向的 Beam）。

**几何模型**：起点 `StartPoint` + 终点 `EndPoint` + 型材截面 `Profile` → 截面沿起止点连线方向扫掠生成实体。这与 OmniM 的 `LinearComponentGen`（固定截面模式）完全等价。

```csharp
public sealed class Beam : Part
{
    public Point StartPoint { get; set; }
    public Point EndPoint { get; set; }
    public Offset StartPointOffset { get; set; }  // 沿局部 X 轴偏移
    public Offset EndPointOffset { get; set; }
    public BeamTypeEnum Type { get; }              // BEAM / COLUMN
}

public enum BeamTypeEnum {
    BEAM,    // 梁（默认）
    COLUMN   // 柱（垂直方向）
}
```

**构造函数**：
- `Beam()` — 起止点均位于原点
- `Beam(BeamTypeEnum)` — 指定梁/柱类型
- `Beam(Point, Point)` — 直接指定起止点

**关键属性**（继承自 `Part`）：
- `Profile.ProfileString`：型材截面名（如 `"IPE300"`、`"HEA200"`、`"D20"`、`"PL10"`）
- `Material.MaterialString`：材质（如 `"S235JR"`、`"K40-1"`、`"Steel_Undefined"`）
- `Position.Depth` / `Position.Plane` / `Position.Rotation`：定位（中/前/后、左/中/右、旋转角）
- `StartPointOffset` / `EndPointOffset`：起止点偏移（用于缩放实际长度）
- `CastUnitType`：`CAST_IN_PLACE`（现浇）/ `PRECAST`（预制）

**代码示例**：
```csharp
// 创建一根钢梁
Beam beam = new Beam();
beam.StartPoint = new Point(0, 0, 0);
beam.EndPoint = new Point(6000, 0, 0);
beam.Profile.ProfileString = "IPE300";
beam.Material.MaterialString = "S235JR";
beam.Position.Depth = Position.DepthEnum.MIDDLE;
beam.Insert();

// 创建一根柱（垂直方向）
Beam column = new Beam(Beam.BeamTypeEnum.COLUMN);
column.StartPoint = new Point(0, 0, 0);
column.EndPoint = new Point(0, 0, 4000);
column.Profile.ProfileString = "HEA200";
column.Insert();
```

**截面类型**（来自 `import.asc` 文件描述，profile type 0–16）：
- `0` 自由截面（多边形板用）
- `1` I 型截面（IPE/HEA/HEB/W 系列）
- `2` 焊接空心截面（HK/HQ）
- `3` U 型截面（槽钢）
- `4` L 型截面（角钢）
- `5` 圆钢
- `6` 圆管
- `7` 矩形空心截面（RHS/Box）
- `8` T 型截面
- `9` 矩形条（FL/PL 板）
- `10` Z 型截面
- `11` C 型截面（冷弯）
- `12` Omega / `13` Sigma（冷弯薄壁）
- `14` 钢轨
- `16` 钢筋（DH）

### 2.2 柱 Column（垂直 Beam）

Tekla **没有独立的 Column 类**。柱在 API 层就是 `Beam.BeamTypeEnum.COLUMN`，几何上就是一根起点在下方、终点在上方（或反之）的 Beam。这与 Revit 用 `FamilyInstance + StructuralType.Column` 区分柱/梁的逻辑一致，但更简洁——所有线构件都是 Beam，仅以 `BeamTypeEnum` 和方向区分。

**OmniM 启示**：`ObjectType::Beam` 和 `ObjectType::Column` 在 Tekla 中是同一 API 类。OmniM 的 `LinearComponentGen` 可以统一处理两者，仅以方向参数（垂直/水平/斜向）和语义标签区分，无需独立 GenRule。

### 2.3 板 ContourPlate（任意轮廓板）

`ContourPlate` 是 `Part` 的密封子类，代表"由平面多边形轮廓 + 板厚"定义的板件。这是 Tekla 中**节点板、底板、加劲肋、楼板、基础**等所有板类构件的几何基础。

**几何模型**：`Contour`（含 N 个 `ContourPoint`，最多 99 个）+ `Profile.ProfileString`（如 `"PL10"` 定义 10mm 厚）→ 沿轮廓法线方向拉伸板厚生成实体。等价于 OmniM 的 `Primitive::Extrusion`。

```csharp
public sealed class ContourPlate : Part
{
    public Contour Contour { get; set; }
    public ContourPlateTypeEnum Type { get; }    // UNKNOWN / PLATE / SLAB
}

// 轮廓点支持倒角
ContourPlate plate = new ContourPlate();
plate.AddContourPoint(new ContourPoint(new Point(0,    0,    0), new Chamfer()));
plate.AddContourPoint(new ContourPoint(new Point(1000, 0,    0), new Chamfer()));
plate.AddContourPoint(new ContourPoint(new Point(1000, 1000, 0), new Chamfer()));
plate.AddContourPoint(new ContourPoint(new Point(0,    1000, 0), new Chamfer()));
plate.Profile.ProfileString  = "PL10";   // 10mm 厚
plate.Material.MaterialString = "S235JR";
plate.Position.Depth = Position.DepthEnum.MIDDLE;
plate.Insert();
```

**关键特性**：
- 角点支持 `Chamfer`（倒角/圆角/弧线倒角），通过 `Chamfer.ChamferTypeEnum` 控制（直角/圆角/倒角/弧线）
- 圆形板可通过"4 个角点 + 圆角倒角（半径=边长/2）"实现
- `Position.Depth` 控制板厚方向的位置（MIDDLE/FRONT/BEHIND）
- `GetContourPolycurve()` 返回 `Polycurve`，便于几何查询
- 构造时默认 `Class="99"`、`Name="PLATE"`、`Position.Depth=MIDDLE`

**OmniM 启示**：Tekla 的 ContourPlate = OmniM 的 `Primitive::Extrusion`（2D 闭合轮廓 + 拉伸深度）。倒角通过轮廓点的 `Chamfer` 属性附加，是 OmniM Sketch 层应支持的"角点修饰"特性。

### 2.4 型材库 Profile Library

Tekla 的型材库（`profdb.ini` / `Profit.db`）按 **Profile type 0–16** 分类，每个 `ProfileString` 对应一个截面定义。截面可以是：
- **参数化截面**：如 `"IPE300"` → 高度 300、翼缘 150、腹板 7、翼厚 10.7
- **标准截面表**：AISC / DIN / ISO / GB / JIS 系列
- **自由截面**：通过多边形顶点定义（profile type 0）

`Profile` 类的 API：
```csharp
public class Profile {
    public string ProfileString { get; set; }   // "IPE300" / "PL10" / "D20"
    // 可通过 GetProfileItem() 查询具体尺寸（高度/宽度/厚度）
}
```

**OmniM 启示**：Tekla 的型材库 = OmniM 的 Component Catalog 中的截面库。`ProfileString` 即 Catalog Key。OmniM 应建立 IPE/HEA/W/角钢/槽钢/圆管/RHS 的内置截面库，每个截面生成一个标准 2D 多边形或参数化 2D 草图，由 `LinearComponentGen` 沿路径扫掠。

### 2.5 折线梁 PolyBeam（路径扫掠）

`PolyBeam` 是 `Part` 的密封子类，代表"沿多段折线（含弧线段）路径扫掠型材截面"的构件——如弧形梁、螺旋楼梯梁、曲线檩条。

**几何模型**：`Contour`（含 N 个 `ContourPoint`，每点带 `Chamfer` 定义转角类型：直角/圆弧/倒角）+ `Profile` → 截面沿折线扫掠，转角处按倒角类型平滑过渡。等价于 OmniM 的 `LinearComponentGen`（沿任意路径扫掠）。

```csharp
public sealed class PolyBeam : Part {
    public Contour Contour { get; set; }   // 路径（含倒角定义）
}
```

**特殊用途**：通过"构造圆 + 3 点 + 全弧倒角"可创建 90°/180°/360° 弧形梁，复制旋转后组成完整环形构件（如混凝土圆筒仓）。

**OmniM 启示**：PolyBeam 验证了 `LinearComponentGen` 必须支持任意路径（含弧线段、含转角倒角）。OmniM 的 `path: PolyCurve` + `cross_section: Polygon` 设计完全匹配。

### 2.6 Brep 零件与 FacetedBrep 几何

Tekla API 中有两个"Brep"概念，需区分：

- **`Tekla.Structures.Model.Brep`（Part 子类）**：实际上是一个"起点+终点+Profile"的零件类，与 `Beam` 几何生成方式相同（截面扫掠），仅 `Name` 默认为 `"BREP"`、`Class="1"`。**它不是真正的自由边界表示**，而是用 BREP 数据结构存储的扫掠体。从 `Brep(Point, Point)` 构造函数可见，本质仍是线构件。
- **`Tekla.Structures.Geometry3d.FacetedBrep`（几何类）**：这才是真正的多面体边界表示，用于 `DirectShape` 类的第三方几何导入：
  ```csharp
  public FacetedBrep(
      Vector[] vertices,                     // 顶点数组
      int[][] outerWires,                    // 每个面的外环（顶点索引序列）
      IDictionary<int, int[][]> innerWires   // 每个面的内环（孔洞）
  )
  ```

**OmniM 启示**：Tekla 的 `Brep`（Part 子类）实际是 LinearComponent 的特例，**不应映射到 OmniM 的 Primitive::Brep**。真正的自由 BREP 几何通过 `FacetedBrep` 表达，对应 OmniM 的 `MeshGeometry` / `BRep` 导入路径（类似 Revit 的 `DirectShape`）。

### 2.7 放样板 LoftedPlate（双导线放样）

`LoftedPlate` 是 `Part` 的密封子类，代表"由两条导线（Polycurve）放样生成的板"——如变截面钢板、双曲板。

**几何模型**：`BaseCurves`（至少两条 `Polycurve`）+ `Profile`（板厚）+ `FaceType` → 两条导线的起点相连、终点相连，沿导线放样出曲面，再按板厚偏移生成实体。等价于 OmniM 的 `Primitive::Loft`。

支持的导线组合：构造线 ↔ 构造线、构造线 ↔ 构造弧、构造弧 ↔ 构造弧、构造圆 ↔ 构造圆、构造多曲线 ↔ 构造多曲线。混凝土版同机制称为 `Lofted Slab`。

**OmniM 启示**：LoftedPlate 是 `Primitive::Loft` 的工业验证。OmniM 的 Loft 应支持"两条 Polycurve 导线"作为输入。

---

## 3. 钢结构连接 (Steel Connections)

Tekla 的"组件（Component）"系统是其最具特色的能力：用户选择主件 + 副件 + 输入点，组件过程化地生成一组板件、螺栓、焊缝、切割。这是 OmniM `Assembly` GenRule 的工业原型。

### 3.1 连接类继承体系

```
ModelObject
└── BaseComponent (抽象)
    ├── Connection    ← 连接（主件 + N 副件，如端板/底板/角钢连接）
    ├── Component     ← 通用组件（任意输入，如桁架/楼梯/基础）
    ├── Detail        ← 细节（单主件 + 输入点，如加劲肋/切割）
    └── Seam          ← 缝（两件之间的连续节点，如焊缝组）
```

`Connection` 类的关键属性：

```csharp
public sealed class Connection : BaseComponent {
    public string Name { get; set; }              // 组件名（标识自定义组件/插件）
    public int Number { get; }                    // >0 系统组件；CUSTOM_OBJECT_NUMBER 自定义；PLUGIN_OBJECT_NUMBER 插件
    public string Code { get; set; }              // 连接代码（≤20 字符，用于分类/报告）
    public AutoDirectionTypeEnum AutoDirectionType { get; set; }
    public Vector UpVector { get; set; }          // 上方向
    public PositionTypeEnum PositionType { get; set; }
    public ConnectionStatusEnum Status { get; }   // 状态（颜色指示）
    // 从 BaseComponent 继承：LoadAttributesFromFile / SetAttribute / SetComponentInput / Insert
}
```

### 3.2 组件输入契约（ComponentInput）

每个组件接受一个 `ComponentInput`，最多 10 个输入项，按顺序区分：

```csharp
public sealed class ComponentInput {
    void AddInputObject(ModelObject obj);          // 输入零件
    void AddInputObjects(ArrayList objects);       // 输入零件列表
    void AddOneInputPosition(Point p);             // 单输入点
    void AddTwoInputPositions(Point p1, Point p2); // 双输入点
    void AddInputPolygon(Polygon polygon);         // 输入多边形
}
```

**顺序敏感**：点 + 零件 ≠ 零件 + 点。Tekla 通过 `joints.def` 文件定义组件的默认输入顺序。

### 3.3 自定义连接插件（ConnectionBase）

`Tekla.Structures.Plugins.ConnectionBase` 是自定义连接插件的基类，通过特性（Attribute）声明元数据，在 `Run()` 方法中编写生成逻辑：

```csharp
[Plugin("BeamConnection")]
[PluginUserInterface(UserInterfaceDefinitions.Plugin3)]
[SecondaryType(SecondaryType.SECONDARYTYPE_ONE)]
[AutoDirectionType(AutoDirectionTypeEnum.AUTODIR_BASIC)]
[PositionType(PositionTypeEnum.COLLISION_PLANE)]
public class BeamConnection : ConnectionBase {
    public BeamConnection(StructuresData3 data) { ... }

    public override void Run() {
        // Primary、Secondaries、Positions 已自动注入
        // 在此创建 Beam / ContourPlate / BoltArray / Fitting / Weld 并 Insert()
    }
}
```

`ConnectionBase` 提供的输入：
- `Primary`：主件 Identifier
- `Secondaries`：副件 Identifier 列表
- `Positions`：位置点列表（细节 1 个，缝 N 个）
- `Code`：连接代码

**OmniM 启示**：这就是 OmniM `Assembly` GenRule 的工业原型——一个"输入契约（Primary/Secondaries/Positions）+ 过程化生成逻辑（Run）+ 子零件列表"的封装。OmniM 的 `AssemblyGen` 应支持类似的"组件脚本"机制：定义输入参数、声明生成的子对象列表、每个子对象有自己的 GenRule。

### 3.4 端板连接 EndPlate Connection（系统组件 144）

`End plate (144)` 是 Tekla 内置系统组件，连接梁到柱或梁到梁，使用螺栓端板。

**输入**：主件（柱或梁）+ 副件（梁）。选择副件后自动创建。

**过程化生成的对象**：
- End plate（端板，ContourPlate）
- Bent plate（弯板，可选）
- Shim plates（垫板，可选）
- Stiffeners（加劲肋，可选）
- Haunch plates（腋板，可选）
- Holes（孔，由 BoltGroup 创建）
- Bolts（螺栓，由 BoltGroup 创建）
- Seat angles（座角钢，可选）
- Welds（焊缝，由 Weld 创建）
- Cuts（切割，由 BooleanPart/Fitting 创建）

**典型场景**：
- 梁对梁端板连接（带缺角选项）
- 梁对柱翼缘/腹板端板连接
- 全深端板连接
- 端板带柱加劲肋
- 柱端板连接（副件旋转）

**参数**：板位置（上下边缘距离）、板厚、板宽、螺栓布置（数量/间距/边距）、孔径、焊缝尺寸等。所有参数通过 `Connection.SetAttribute("name", value)` 设置，或通过 `LoadAttributesFromFile("standard")` 加载属性文件。

**OmniM 映射**：端板连接 = `Assembly`（包含 1 个端板 Primitive + N 个螺栓 Primitive + N 个焊缝 + 可选加劲肋 Primitive + 可选切割 BooleanSubtract）。OmniM 应提供"端板连接"作为内置 Catalog ComponentDef，其 FeatureTreeTemplate 描述子零件的相对位置与参数化规则。

### 3.5 底板连接 BasePlate Connection（系统组件 1004/1069 等）

底板连接将柱底通过底板与基础（混凝土或钢）相连。

**`Cast-in plate (1069)`** 在梁端创建预埋板，可选附加：
- Cast-in plate（预埋板，ContourPlate）
- Shear tab（抗剪板）或 Clip angle（角钢）
- Bolts / Studs（螺栓/焊钉）
- Anchors（地脚螺栓， bent anchor bolts 弯折锚栓）
- Washer plates（垫圈板）
- Welds

**`Bearing plate (7)`** 将梁连接到柱，包含：
- Console seat plate（牛腿承托板，焊接于柱）
- End plate（端板）
- Shim plate（垫板）
- Clip angle（L 型角钢）
- Bolts / Welds

**OmniM 映射**：底板连接 = `Assembly`（底板 Primitive + 锚栓/螺栓 Primitive + 焊缝 + 可选抗剪板/角钢）。锚栓组的几何生成与螺栓组类似，但通常带弯钩（hook）嵌入混凝土。

### 3.6 角钢连接 ClipAngle Connection（系统组件 44）

`Clip angles (44)` 在板与柱/梁翼缘或腹板之间创建角钢连接，可单侧或双侧。

**输入**：3 个位置点（定义 XY 平面）+ 板 + 柱/梁。

**生成对象**：
- Clip angles（近端 + 远端，均为 Beam with L profile）
- Bolts
- Welds

**参数**：角钢截面（Near side L / Far side L，从型材库选择，如 `L100-6`）、边距、单/双侧、安全连接、方向、旋转、偏移、焊接位置。

**OmniM 映射**：角钢连接 = `Assembly`（2 个 LinearComponent 角钢 + 螺栓组 + 焊缝）。角钢本身是 `Beam` with L 型截面，沿短段路径扫掠。

### 3.7 刚接/铰接 Moment/Pinned Connection

Tekla 不区分"刚接/铰接"作为独立组件类型，而是通过组件的**生成内容**体现：
- **刚接（Moment）**：端板上下延伸、加劲肋、双腹板角钢、焊缝——传递弯矩
- **铰接（Pinned）**：仅座角钢/承托板/单排螺栓——仅传剪力

例如 `End plate (144)` 通过"Full depth end plate"（全深端板）选项实现刚接，通过"短端板 + 单排螺栓"实现铰接。`Bearing plate (7)` 默认是铰接（仅承托 + 上翼缘角钢）。

**OmniM 启示**：OmniM 不应在 GenRule 层区分刚接/铰接，而应在 Connection ComponentDef 的参数中暴露"连接刚度"开关，由 FeatureTreeTemplate 条件性地生成不同的子零件组合。这与 SOLIDWORKS 的"配置抑制"机制类似。

---

## 4. 支撑与桁架 (Braces & Trusses)

### 4.1 支撑 Brace

支撑在 Tekla API 层就是一根 `Beam`——通常为圆管（`Dxxx`）/角钢（`Lxxx`）/槽钢截面，起止点为两端节点。支撑的"连接"由独立的 `Connection` 组件（如 `Tube gusset (20)`）生成。

**`Tube gusset (20)`** 连接 1–10 根空心截面支撑到梁或柱，使用节点板：

**生成对象**：
- Gusset plate（节点板，ContourPlate）
- Connection plates（连接板）
- Clip angles（角钢）
- End plates / seal plates（端板/密封板）
- Tongue plates（舌板）
- Cover plates（盖板）
- Stiffeners（加劲肋）
- Bolts / Welds

**输入**：主件（柱/梁）+ 1–10 个副件（支撑，RHS 或圆管截面）。

**节点板形状参数**：上边缘距离、角部角度、垂直边缘长度、支撑长度等——通过这些参数过程化生成节点板的 `Contour` 多边形。

**OmniM 映射**：
- 支撑本身 = `LinearComponent`（圆管/角钢截面 + 直线路径）
- 支撑连接 = `Assembly`（节点板 Primitive + 连接板 + 角钢 LinearComponent + 螺栓组 + 焊缝 + 切割）

### 4.2 桁架 Truss

Tekla 没有独立的 `Truss` 类。桁架由以下方式构建：
- **逐杆件建模**：每根弦杆/腹杆是一根 `Beam`，节点处用 `Connection` 组件连接
- **桁架组件**：Tekla Warehouse 提供参数化桁架组件（如 `Truss (90)`），作为 `Component` 输入两点 + 参数，过程化生成所有弦杆/腹杆/节点板
- **`Beam system`（梁系统）**：按间距自动布置一组平行梁

**OmniM 映射**：桁架 = `Assembly`（顶层），其子对象为：
- 弦杆：N 根 `LinearComponent`（弦杆截面 + 直线路径）
- 腹杆：M 根 `LinearComponent`（腹杆截面 + 直线路径）
- 节点：K 个 `Assembly`（每个节点是支撑连接的 Assembly）

这验证了 OmniM `Assembly` 的嵌套能力：桁架 Assembly 包含子 Assembly（节点）。

---

## 5. 基础 (Foundations)

### 5.1 独立基础 Footing（pad footing / strip footing）

Tekla Structures（非 Structural Designer）中，基础通过 **Concrete 选项卡 > Foundation** 创建：

- **Pad footing（独立基础）**：本质上是一个 `ContourPlate`（矩形或八边形轮廓 + 厚度），或参数化 `Component`（如 India 环境的 `Footing_iso` 组件，包含 PCC 垫层 + 基础 + 短柱 + 灌浆层 + 抗剪键 + 施工缝选项）
- **Strip footing（条形基础）**：本质上是一根 `Beam`（沿路径的截面，类似条形梁），属性文件扩展名 `*.csf`

**条形基础属性**：
- 名称、型材/截面、材料、抛光、等级
- 位置（在平面上/旋转/在深度）
- 末端偏移（Dx/Dy/Dz，沿参考线/垂直/竖向）
- 浇筑体（前缀/起始编号/预制 vs 现浇/浇筑状态）
- 弯曲（平面/半径，沿路径弯曲）

**几何模型**：
- 独立基础 = `ContourPlate`（轮廓 + 厚度，等价 `Primitive::Extrusion`）
- 条形基础 = `Beam`（截面 + 路径，等价 `LinearComponent`）
- 复合基础（垫层+基础+短柱+灌浆）= `Assembly`（多层 ContourPlate/Beam 堆叠）

**OmniM 映射**：
- 单体独立基础 → `Primitive::Extrusion`（矩形/八边形 + 厚度）
- 单体条形基础 → `LinearComponent`（矩形截面 + 路径）
- 含垫层/短柱/抗剪键的复合基础 → `Assembly`（多个 Primitive/LinearComponent 堆叠）

### 5.2 桩 Pile

Tekla Structures 中桩通常建模为 `Beam`（圆管/方管截面，垂直或斜向）。预应力混凝土桩可用 `Brep` 或 `Beam` + `RebarStrand`（预应力钢绞线）。

**Tekla Warehouse 提供的桩组件**（India 环境）：
- `CIP_Pile_2` / `CIP_Pile_3` / `CIP_Pile_5` / `CIP_Pile_6` / `CIP_Pile_9`：分别对应 2/3/5/6/9 桩承台，可调桩偏移与插入承台的深度，桩可为斜桩（skewed rectangular 或 square）

**几何模型**：桩 = `Beam`（圆管/方管截面 + 直线路径，等价 `LinearComponent`）。

**OmniM 映射**：桩 = `LinearComponent`（固定截面 + 直线路径）。预应力桩 = `Assembly`（外层 LinearComponent 混凝土 + 内层 RebarStrand LinearComponent）。

### 5.3 桩承台 PileCap

桩承台在 Tekla 中通过两种方式创建：
- **`ContourPlate` + 桩组件**：承台本身是一个 `ContourPlate`（矩形/多边形 + 厚度），下方连接多个 `Beam`（桩）
- **参数化桩承台组件**：如 `CIP_Pile_N` 系列，输入桩数 + 桩位偏移 + 承台尺寸，过程化生成承台板 + 桩 + 钢筋

**Tekla Structural Designer（设计软件，非 Structures）** 提供独立的 `Pile Cap Column` 命令，自动布置承台并支持：
- Auto-design depth / size / reinforcement
- Pile arrangement（标准布置或用户自定义布置，含偏心）
- Pile Catalogue（桩类型库，含抗压/抗拉承载力）

**OmniM 映射**：桩承台 = `Assembly`（顶层），其子对象为：
- 承台板：1 个 `Primitive::Extrusion`（矩形/多边形 + 厚度）
- 桩：N 个 `LinearComponent`（圆管/方管 + 直线路径）
- 钢筋：可选 N 组 `LinearComponent`（钢筋 + 路径）

---

## 6. 钢筋 (Reinforcement)

Tekla 拥有业内最成熟的钢筋详图能力。钢筋类继承体系：

```
ModelObject
└── Reinforcement (抽象)
    ├── SingleRebar          ← 单根钢筋
    ├── BaseRebarGroup (抽象)
    │   ├── RebarGroup        ← 直线分布钢筋组（主筋）
    │   ├── CircleRebarGroup  ← 圆周分布钢筋组
    │   └── CurvedRebarGroup  ← 沿曲线分布钢筋组
    ├── RebarMesh             ← 钢筋网
    ├── RebarStrand           ← 预应力钢绞线
    └── RebarSet              ← 现代钢筋集（带 modifier/splitter/end-detail modifier）
```

### 6.1 主筋 Rebar（RebarGroup / SingleRebar）

`SingleRebar` 代表单根钢筋。`RebarGroup` 代表一组沿方向分布的钢筋（主筋）。两者均继承自 `Reinforcement`。

**几何模型**：钢筋 = `Polygon`（形状多边形，定义钢筋的折线形状，含弯钩）+ `Size`（直径）+ `Grade`（钢种）+ `RadiusValues`（弯折半径）+ 分布方向/间距 → 沿形状多边形扫掠圆截面生成实体。等价于 OmniM 的 `LinearComponent`（圆截面 + 任意折线路径）。

**SingleRebar 关键属性**：
```csharp
public sealed class SingleRebar : Reinforcement {
    public Polygon Polygon { get; set; }              // 钢筋形状多边形（折线）
    public string Size { get; set; }                  // 直径，如 "20"
    public string Grade { get; set; }                 // 钢种，如 "A500HW"
    public RebarHook StartHook { get; set; }          // 起点弯钩
    public RebarHook EndHook { get; set; }            // 终点弯钩
    public ArrayList RadiusValues { get; set; }       // 弯折半径
    public ArrayList OnPlaneOffsets { get; set; }     // 平面内偏移（每肢）
    public double FromPlaneOffset { get; set; }       // 离平面偏移（保护层）
    public Reinforcement.RebarOffsetTypeEnum StartPointOffsetType { get; set; }
    public double StartPointOffsetValue { get; set; } // 起点保护层/肢长
    // ... EndPoint 同上
}
```

**RebarGroup 关键属性**（继承自 `BaseRebarGroup`）：
```csharp
public sealed class RebarGroup : BaseRebarGroup {
    // 从 BaseRebarGroup 继承：
    public Point StartPoint { get; set; }       // 分布方向起点
    public Point EndPoint { get; set; }         // 分布方向终点
    public RebarHook StartHook { get; set; }
    public RebarHook EndHook { get; set; }
    public double FromPlaneOffset { get; set; }
    public double StartFromPlaneOffset { get; set; }
    public double EndFromPlaneOffset { get; set; }
    public ExcludeTypeEnum ExcludeType { get; set; }   // 省略首/末/两端
    public RebarGroupStirrupTypeEnum StirrupType { get; set; }  // 是否作为箍筋
    // 自身：
    public ArrayList Polygons { get; set; }     // 形状多边形列表（每根钢筋一个，或一个模板）
    public RebarGroupSpliceTypeEnum SpliceType { get; set; }
    public ArrayList SpliceOffsets { get; set; }
    public ArrayList SpliceGapValues { get; set; }
}
```

**RebarGroup.StirrupTypeEnum** 冘认值 `STIRRUP_NOT_STIRRUP`，设为其他值时该组作为箍筋处理（影响弯曲半径与详图规则）。

**Rebar Set（现代钢筋集，2016+ 引入）**：取代旧版 RebarGroup，支持 `RebarPropertyModifier`（属性修改器）、`RebarSplitter`（拆分器）、`RebarEndDetailModifier`（端部细节修改器），可基于宿主混凝土面自动适配。

**代码示例**（创建箍筋）：
```csharp
SingleRebar stirrup = new SingleRebar();
stirrup.Father = myPart;
stirrup.Size = "8";
stirrup.Grade = "A500HW";
stirrup.RadiusValues.Add(16.0);
stirrup.OnPlaneOffsets.Add(20.0);
stirrup.FromPlaneOffset = 50;
stirrup.Name = "Stirrup";
stirrup.Class = 4;
stirrup.Polygon.Points.Add(...);   // 箍筋形状（矩形/多边形）
stirrup.Insert();
```

**OmniM 映射**：
- 单根钢筋 = `LinearComponent`（圆形截面 + 折线路径 + 弯钩修饰）
- 钢筋组（直线分布）= `LinearComponent`（截面 + 路径）+ 阵列（沿分布方向）
- 钢筋组（圆周分布）= `LinearComponent` + 圆周阵列
- 钢筋组（曲线分布）= `LinearComponent` + 沿曲线阵列
- Rebar Set = `LinearComponent` + modifier（属性修改器）+ splitter（拆分器）

Tekla 钢筋验证了 OmniM `LinearComponentGen` 必须支持：
1. **圆形截面**作为内置截面（钢筋直径即截面直径）
2. **任意折线路径**（含弯钩，含弯折半径）
3. **沿分布方向阵列**（按数量或按间距）
4. **沿曲线/圆周阵列**（CurvedRebarGroup / CircleRebarGroup）
5. **modifier 机制**（Rebar Set 的属性修改器，影响局部钢筋的尺寸/间距/弯钩）

### 6.2 箍筋 Stirrup（StirrupGroup）

箍筋在 Tekla API 中**没有独立类**。箍筋是 `SingleRebar` 或 `RebarGroup` 的特例：
- 形状多边形是闭合的（矩形/U 形/多边形）
- `RebarGroup.StirrupType` 设为非 `STIRRUP_NOT_STIRRUP`
- 默认 Class = 4（区分主筋 Class = 7）
- 弯曲半径按"箍筋/拉筋"规则（通常比主筋小）

系统组件 `Rebar in beam (90)` 自动创建梁/柱的箍筋与纵筋：
- Stirrups（箍筋）
- Perpendicular rebars（垂直附加筋）

**OmniM 映射**：箍筋 = `LinearComponent`（圆形截面 + 闭合多边形路径）。OmniM 不应单独定义 Stirrup ObjectType，而应作为 Rebar 的"形状子类型"（闭合 vs 开放）+ Class 标签。

### 6.3 钢筋网 Mesh（RebarMesh）

`RebarMesh` 代表双向钢筋网，由纵向（longitudinal）和横向（transverse）两组钢筋组成。

**几何模型**：网格边界（多边形）+ 纵向间距/直径 + 横向间距/直径 → 在边界内按间距阵列生成双向钢筋。每根钢筋仍是一根 `LinearComponent`（圆截面 + 直线路径）。等价于 OmniM 的 `LayerExtrusion`（双向阵列的薄层）或 `Assembly`（双向阵列的 LinearComponent 集合）。

**属性**：`Name` / `Class` / `Grade` / `Size`（纵/横直径）/ `Spacing`（纵/横间距）/ `CoverThickness`（保护层）/ `StartHook` / `EndHook`。

**OmniM 映射**：钢筋网 = `LayerExtrusion`（两层圆截面钢筋沿正交方向阵列）或 `Assembly`（N+M 根 LinearComponent）。考虑到钢筋网本质是离散钢筋而非连续层，`Assembly` + 双向阵列 Pattern 更准确。

---

## 7. 次级钢构件 (Secondary Steel)

次级钢构件（墙梁 Girt、檩条 Purlin、抗风支撑 WindBrace）在 Tekla API 层都是 `Beam`，区别仅在于：
- 截面类型（C/Z 型冷弯薄壁用于 Girt/Purlin；角钢/圆管用于 WindBrace）
- 位置（墙梁在墙面、檩条在屋面、抗风支撑在墙/屋面斜向）
- 连接方式（Girt/Purlin 通常用 Clip angle 或螺栓连接到主框架；WindBrace 用 gusset plate 连接）

### 7.1 墙梁 Girt

Girt 是墙面横向次要构件，承受墙面荷载并传递到主柱。Tekla 中：
- 类：`Beam`（C/Z 型冷弯截面，如 `C200-20`、`Z200-20`）
- 位置：墙面，垂直于墙柱方向
- 连接：通常 Clip angle（角钢）或螺栓连到主柱

### 7.2 檩条 Purlin

Purlin 是屋面纵向次要构件，承受屋面荷载并传递到主梁/桁架。Tekla 中：
- 类：`Beam`（C/Z 型冷弯截面）
- 位置：屋面，沿屋面坡度
- 连接：螺栓/角钢连到主梁/桁架上弦

### 7.3 抗风支撑 WindBrace / KickerBrace

WindBrace 是墙面或屋面的斜向支撑，承受风荷载产生的水平力。Tekla 中：
- 类：`Beam`（角钢/圆管/槽钢截面）
- 位置：墙/屋面斜向
- 连接：gusset plate（节点板）+ 螺栓，类似普通支撑（见 4.1）

**OmniM 映射**：所有次级钢构件 = `LinearComponent`（C/Z/角钢/圆管截面 + 直线路径）。连接 = `Assembly`（角钢/螺栓/焊缝）。OmniM 不应为 Girt/Purlin/WindBrace 定义独立 ObjectType，而应作为 `ObjectType::Beam` 的"用途标签"（usage tag），通过 Catalog ComponentDef 分类路径区分（如 `Steel/Girt/C-Section`）。

---

## 8. 切割与特征 (Cuts & Features)

Tekla 的切割/特征系统通过 `Boolean` 抽象基类统一：

```
ModelObject
└── Boolean (抽象)
    ├── BooleanPart    ← 零件布尔（用另一个 Part 切割或添加）
    ├── Fitting        ← 端部拟合（用平面裁切零件端部）
    └── CutPlane       ← 平面切割（用平面切开零件）
```

### 8.1 BooleanPart（零件布尔切割/添加）

`BooleanPart` 用一个"操作件（OperativePart）"对"父件（Father）"执行布尔运算：

```csharp
public sealed class BooleanPart : Boolean {
    public ModelObject Father { get; set; }              // 被切割的零件
    public ModelObject OperativePart { get; set; }       // 切割工具（必须 Class = BooleanOperativeClass）
    public BooleanTypeEnum Type { get; set; }            // CUT（切割）/ ADD（添加）
}

public const string BooleanOperativeClassName = "..." ;  // 操作件必须设置的 Class
```

**用法**：
```csharp
// 1. 创建被切割件
Beam beam1 = new Beam(new Point(0,0,0), new Point(6000,0,0));
beam1.Profile.ProfileString = "IPE300";
beam1.Insert();

// 2. 创建切割工具（一个 Beam，Class 设为 BooleanOperativeClass）
Beam cutter = new Beam(new Point(3000,-500,0), new Point(3000,500,0));
cutter.Profile.ProfileString = "IPE300";
cutter.Class = BooleanPart.BooleanOperativeClassName;
cutter.Insert();

// 3. 创建布尔切割
BooleanPart cut = new BooleanPart();
cut.Father = beam1;
cut.SetOperativePart(cutter);   // 自动用 cutter 的副本替换 OperativePart
cut.Type = BooleanPart.BooleanTypeEnum.CUT;
cut.Insert();

// 4. 删除原始切割工具（已复制为 OperativePart）
cutter.Delete();
```

**OmniM 映射**：BooleanPart 完全等价于 OmniM 的 `BooleanSubtract` GenRule——base 几何（Father）+ cutting 几何（OperativePart）+ 类型（CUT/ADD）。Tekla 的 `ADD` 类型对应 OmniM 的 `BooleanUnion`（GenRule 当前仅支持 Subtract，需扩展为 Union）。

### 8.2 Fitting（端部拟合）

`Fitting` 用一个平面裁切零件的端部——常用于将梁端切平以贴合柱翼缘：

```csharp
Fitting fitting = new Fitting();
fitting.Father = beam;
fitting.Plane = new Plane {
    Origin = new Point(3000, 0, 0),
    AxisX  = new Vector(0, 1000, 0),
    AxisY  = new Vector(0, 0, -1000)
};
fitting.Insert();
```

**几何模型**：平面 + 父件 → 用平面切除平面一侧的几何。等价于 OmniM 的 `BooleanSubtract`（半空间切割），与 Revit 的 `CutWithHalfSpace` 完全对应。

### 8.3 CutPlane（平面切割）

`CutPlane` 与 `Fitting` 类似，但用于在零件中部切开（而非端部）。同样用一个 `Plane` 定义切割面。

### 8.4 EdgeChamfer（边倒角）

`EdgeChamfer` 在零件边缘创建倒角/圆角：

```csharp
EdgeChamfer chamfer = new EdgeChamfer(new Point(0,-100,0), new Point(0,200,0));
chamfer.FirstChamferEndType = EdgeChamfer.ChamferEndTypeEnum.BEVELLED;
chamfer.FirstBevelDimension = 50;
chamfer.Chamfer.X = 20;
chamfer.Chamfer.Y = 20;
chamfer.Father = beam;
chamfer.Insert();
```

**OmniM 映射**：EdgeChamfer 对应 SOLIDWORKS 的 `Feature::Chamfer` / `Feature::Fillet`。OmniM 的 Feature Engine 应新增 `Chamfer` / `Fillet` 修饰特征类型，作用于零件边。

### 8.5 BoltGroup（螺栓组与孔）

`BoltGroup` 是抽象基类，子类按布置形状区分：
- `BoltArray`：矩形阵列（行数 × 列数 × 间距）
- `BoltCircle`：圆周阵列（数量 × 节圆直径）
- `BoltXYList`：任意坐标列表

**关键属性**：
```csharp
public abstract class BoltGroup : ModelObject {
    public double BoltSize { get; set; }            // 直径，如 20
    public string BoltStandard { get; set; }        // 标准，如 "8.8"
    public BoltTypeEnum BoltType { get; set; }
    public bool Bolt { get; set; }                  // true=螺栓，false=仅孔
    public BoltHolesAttributes BoltHolesAttributes { get; set; }
    public HoleType HoleType { get; set; }          // 标准/加大/长圆/盲孔
    public double BlindHoleDepth { get; set; }
    public Point FirstPosition { get; set; }        // 第一定位点
    public Point SecondPosition { get; set; }       // 第二定位点
    public ArrayList BoltPositions { get; }         // 所有螺栓位置（XY 平面）
    // ... CutLength / ExtraLength / Tolerance / SlottedHoleX / SlottedHoleY 等
}
```

**几何模型**：螺栓组在每个零件上钻孔（BooleanSubtract 圆柱）+ 添加螺栓实体（圆柱 + 六角头，Primitive）。Tekla 用一个对象同时表达"孔 + 螺栓"，通过 `Bolt` 属性切换。

**OmniM 映射**：
- 螺栓孔 = `BooleanSubtract`（圆柱切割零件）
- 螺栓实体 = `Primitive`（圆柱 + 六角头组合）
- 螺栓组 = `Assembly`（N 个螺栓 Primitive + N 个孔 BooleanSubtract）

OmniM 应提供 `BoltArray` / `BoltCircle` / `BoltXYList` 三种布置 Pattern 作为 Assembly 的内置阵列生成器。

### 8.6 Weld / PolygonWeld（焊缝）

`Weld` 表示两件之间的焊缝，`PolygonWeld` 表示沿用户拾取的多边形路径的焊缝。两者均继承自 `BaseWeld`。

**关键属性**（`BaseWeld`）：
- `SizeAbove` / `SizeBelow`（上下焊脚尺寸）
- `AngleAbove` / `AngleBelow`（坡口角度）
- `AroundWeld`（周围焊 vs 边焊）
- `ContourAbove` / `ContourBelow`（轮廓：平/凸/凹）
- `Workshop/Site`（车间/现场焊）
- `Shape`（连续/间断/交错）
- `ElectrodeClassification` / `ElectrodeStrength` / `ElectrodeCoefficient`
- `Polygon`（PolygonWeld 专属，多边形路径）

**`WeldGeometry` 类**（`Tekla.Structures.Model.Welding`）表示焊缝几何：`Polygons`（焊缝截面多边形列表）+ `Position`（上/下）。通过 `weld.GetWeldGeometries()` 获取。

**OmniM 映射**：焊缝在 OmniM 中作为 `Assembly` 的修饰对象（非独立 GenRule），几何上是一个 `Primitive`（沿焊缝路径扫掠焊缝截面多边形）。OmniM 应在 Feature Engine 中支持 `Weld` 特征类型，参数化生成焊缝实体。

---

## 9. 对 OmniM 的启示 (Mapping to GenRule)

### 9.1 完整映射表

| OmniM GenRule | 职责 | 对应 Tekla API 类 / ModelObjectEnum | 典型场景 |
|---|---|---|---|
| **Primitive** | 单一基础几何原语（Box/Cylinder/Cone/Sphere/Extrusion/Sweep/Revolve/Loft）+ 布尔 | `ContourPlate` (3) — 2D 轮廓拉伸<br>`LoftedPlate` (69) — 双导线放样<br>`FacetedBrep` — 多面体 BREP 导入<br>`Beam`/`Brep` — 截面扫掠（特例）<br>`PolygonWeld` — 焊缝扫掠体 | 节点板、加劲肋、底板、垫板、焊缝、预埋件、第三方导入几何 |
| **LayerExtrusion** | 多层 2D 截面沿路径拉伸 | `ContourPlate` 多层堆叠（垫层+基础+灌浆）<br>`SurfaceTreatment` (7) — 表面处理薄层<br>`RebarMesh` (12) — 双向钢筋网（圆截面阵列） | 多层基础、表面处理、钢筋网 |
| **LinearComponent** | 单截面沿中心线扫掠，可选变截面 | `Beam` (1) — 直线梁/柱/支撑<br>`PolyBeam` (2) — 折线/曲线梁<br>`RebarGroup` (11) / `SingleRebar` (10) — 钢筋沿形状多边形扫掠<br>`CircleRebarGroup` (44) / `CurvedRebarGroup` (45) — 钢筋沿圆/曲线分布<br>`RebarStrand` (13) — 预应力钢绞线<br>`RebarSet` (55) — 现代钢筋集<br>条形基础（`*.csf`） | 梁、柱、支撑、檩条、墙梁、抗风支撑、桩、钢筋、钢绞线、条形基础 |
| **Assembly** | 多个子对象组合成父对象，每个子对象有自己的 GenRule | `Connection` (24) — 端板/底板/角钢/抗剪连接<br>`Component` (25) — 桁架/楼梯/基础组件<br>`Detail` (27) — 加劲肋/细节<br>`Seam` (26) — 缝节点<br>`Assembly` (9) — 装配体<br>`PourObject` (47) — 浇筑体（合并多个混凝土件）<br>`BoltArray`/`BoltCircle`/`BoltXYList` (15-17) — 螺栓组<br>`Reinforced concrete stair (95)` — 含楼梯+平台+钢筋<br>`CIP_Pile_N` — 桩承台组件 | 钢连接节点、混凝土节点、桁架、楼梯、桩承台、复合基础、螺栓组、装配体 |
| **BooleanSubtract** | 基础几何减去切割体 | `BooleanPart` (4) — CUT 类型<br>`Fitting` (5) — 端部平面拟合<br>`CutPlane` (6) — 平面切割<br>`EdgeChamfer` (46) — 边倒角/圆角<br>`BoltGroup` 的孔生成部分 | 零件端部切割、节点板切角、螺栓孔、加劲肋切角、梁端缺口 |

### 9.2 关键映射洞察

1. **Tekla 的 Connection/Component/Detail/Seam 四类组件 = OmniM 的 `Assembly` GenRule**：这是 Tekla 最具特色的能力——过程化生成多个子零件。OmniM 的 `AssemblyGen` 应支持"组件脚本"：定义 `Primary` + `Secondaries` + `Positions` 输入契约，过程化生成一组子对象（每个子对象有自己的 GenRule），并支持嵌套（如桁架 Assembly 包含节点 Assembly）。

2. **Tekla 的 Profile + StartPoint/EndPoint = OmniM 的 `LinearComponent`（固定截面模式）**：所有线构件（梁/柱/支撑/檩条/墙梁/桩）在 Tekla 中都是 `Beam`，本质是"截面 + 直线路径"。这验证了 OmniM `LinearComponentGen` 的"固定截面"模式应作为最常用 GenRule，覆盖结构工程 80% 的线构件。

3. **Tekla 的 ContourPlate = OmniM 的 `Primitive::Extrusion`**：所有板件（节点板/底板/加劲肋/独立基础）都是"2D 多边形轮廓 + 厚度方向拉伸"。Tekla 不使用"扫掠板"——板永远是平面拉伸。这提示 OmniM 的 `Primitive::Extrusion` 应优先支持"2D 闭合轮廓 + 法线方向拉伸"，而非通用 3D 扫掠。

4. **Tekla 的 PolyBeam = OmniM 的 `LinearComponent`（任意路径模式）**：弧形梁、螺旋梁通过"多段路径 + 倒角"实现，验证了 `LinearComponent` 必须支持任意 `PolyCurve` 路径（含弧线段、含转角倒角）。

5. **Tekla 的 LoftedPlate = OmniM 的 `Primitive::Loft`**：双导线放样是工业验证的 Loft 输入形式。OmniM 的 Loft 应支持"两条 Polycurve 导线"作为标准输入。

6. **Tekla 的 BooleanPart/Fitting/CutPlane = OmniM 的 `BooleanSubtract`**：Tekla 区分"零件布尔"（用另一个 Part 切割）和"平面切割"（用 Plane 切割）。OmniM 的 `BooleanSubtract` 应同时支持"实体切割体"和"半空间平面切割"两种模式（后者更高效，等价于 Revit 的 `CutWithHalfSpace`）。

7. **Tekla 的 RebarGroup/RebarSet = OmniM 的 `LinearComponent`（圆截面 + 阵列）**：钢筋本质是"圆形截面 + 任意折线路径"，再加上"沿方向/圆周/曲线分布的阵列"。这验证了 `LinearComponent` 必须支持圆形截面 + 任意折线路径 + 阵列 modifier。

8. **Tekla 的 Brep（Part 子类）≠ 自由 BREP**：Tekla 的 `Brep` 类实际是"截面扫掠体"的 BREP 存储形式，不是自由边界表示。真正的自由 BREP 通过 `FacetedBrep`（几何类）表达，对应 OmniM 的 `MeshGeometry` 导入路径。**OmniM 不应将 Tekla Brep 映射为 Primitive::Brep**，而应映射为 LinearComponent。

9. **Tekla 的"刚接/铰接"不是 GenRule 层概念**：连接刚度通过组件的子零件组合体现（端板延伸/加劲肋/双腹板角钢等），而非独立 GenRule。OmniM 应在 Connection ComponentDef 的参数中暴露"连接刚度"开关，由 FeatureTreeTemplate 条件性生成子零件。

10. **Tekla 的 Profile Library = OmniM 的 Catalog 截面库**：`ProfileString` 即 Catalog Key。OmniM 应建立 IPE/HEA/HEB/W/角钢/槽钢/圆管/RHS/C/Z 等内置截面库，每个截面生成标准 2D 多边形，由 `LinearComponentGen` 沿路径扫掠。

---

## 10. 关键发现与建议

### 10.1 核心发现

1. **Part-based + Component-based 双层建模是 Tekla 的核心范式**：所有物理实体都是 `Part` 子类（几何层），所有节点都是 `Component` 子类（过程化层）。这与 OmniM 的"Object（对象）+ GenRule（生成规则）"双层架构高度一致。

2. **Component 系统是 OmniM `Assembly` GenRule 的工业原型**：Tekla 的 `Connection` / `Component` / `Detail` / `Seam` 通过 `Primary` + `Secondaries` + `Positions` 输入 + `Run()` 过程化逻辑生成多个子零件。OmniM 的 `AssemblyGen` 应实现类似的"组件脚本"机制，支持输入契约 + 过程化子对象生成 + 嵌套。

3. **Profile 库是 LinearComponent 的核心配套**：Tekla 的型材库（17 类截面）覆盖了结构工程所有常用截面。OmniM 必须建立等效的截面库，作为 `LinearComponentGen` 的标准配套。

4. **柱/支撑/檩条/墙梁在 API 层都是 Beam**：Tekla 不为这些"语义角色"定义独立类，而是用 `BeamTypeEnum` + 截面类型 + 位置区分。OmniM 应采用相同策略——`ObjectType::Beam` 统一处理，通过 Catalog ComponentDef 分类路径与"用途标签"区分。

5. **钢筋是 LinearComponent 的重要应用场景**：Tekla 的 7 类钢筋对象（SingleRebar/RebarGroup/CircleRebarGroup/CurvedRebarGroup/RebarMesh/RebarStrand/RebarSet）全部本质是"圆截面 + 任意路径 + 阵列"。这扩展了 `LinearComponent` 的应用域——不仅结构钢梁，也包括混凝土钢筋。

6. **BooleanSubtract 需支持两种模式**：Tekla 同时支持"零件布尔"（用 Part 切割）和"平面切割"（用 Plane 切割，对应 Fitting/CutPlane）。OmniM 的 `BooleanSubtract` 应支持"实体切割体"和"半空间平面"两种输入，后者更高效。

7. **Tekla 的 Brep 类是误导性的**：API 中的 `Brep`（Part 子类）实际是截面扫掠体，不是自由 BREP。真正的自由 BREP 通过 `FacetedBrep` 表达。OmniM 在映射时应避免被命名误导。

### 10.2 对 OmniM 的建议

#### 10.2.1 高优先级（与当前架构一致，可立即落地）

1. **完善 LinearComponent 的截面库**：内置 IPE/HEA/HEB/W/UC/UB/角钢/槽钢/圆管/RHS/C/Z/Sigma/Omega 等标准截面（参考 Tekla profile type 0–16），每个截面生成 2D 多边形 + 参数化尺寸表。

2. **实现 Assembly 的"组件脚本"机制**：参考 Tekla `ConnectionBase.Run()`，OmniM 的 `AssemblyGen` 应支持：
   - 输入契约（Primary + Secondaries + Positions）
   - 过程化子对象生成（FeatureTreeTemplate 中嵌入脚本/规则）
   - 嵌套 Assembly（如桁架 Assembly 包含节点 Assembly）

3. **扩展 BooleanSubtract 支持半空间平面切割**：参考 Tekla `Fitting`/`CutPlane`，新增 `PlaneCut` 输入模式（Plane + 方向），效率高于实体布尔。

4. **新增 Primitive::Loft 操作**：参考 Tekla `LoftedPlate`，支持"两条 Polycurve 导线"放样。

#### 10.2.2 中优先级（方向明确，需设计）

5. **建立 Rebar 专用 ComponentDef 分类**：虽然钢筋本质是 LinearComponent，但其 modifier/splitter/end-detail 机制复杂。应在 Catalog 中建立 `Rebar/` 分类，提供 `SingleRebar` / `RebarGroup` / `RebarSet` 三种模板，封装钢筋特有的弯钩/弯曲半径/保护层/阵列规则。

6. **新增 Feature::Chamfer / Feature::Fillet**：参考 Tekla `EdgeChamfer` 与 SOLIDWORKS `Fillet`/`Chamfer`，在 Feature Engine 中支持边倒角/圆角修饰。

7. **新增 BoltArray / BoltCircle / BoltXYList 阵列 Pattern**：参考 Tekla 三种螺栓布置，在 Assembly 中提供内置阵列生成器，自动生成孔 BooleanSubtract + 螺栓 Primitive。

8. **新增 Weld Feature**：参考 Tekla `Weld`/`PolygonWeld`，参数化生成焊缝实体（沿路径扫掠焊缝截面）。

#### 10.2.3 ObjectType 调整建议

| 当前 OmniM ObjectType | 建议 | 理由 |
|---|---|---|
| `OBJECT_TYPE_BEAM` | 保留，统一覆盖 Beam/Column/Brace/Girt/Purlin/WindBrace/Pile | Tekla 用 Beam 统一所有线构件，仅以 BeamTypeEnum + 截面 + 位置区分 |
| `OBJECT_TYPE_COLUMN` | 合并到 BEAM，作为"用途标签" | Tekla Column = Beam.BeamTypeEnum.COLUMN |
| `OBJECT_TYPE_PLATE` | 保留 | 对应 ContourPlate，是节点板/底板/加劲肋/独立基础的统一抽象 |
| `OBJECT_TYPE_CONNECTION` | 新增 | 对应 Tekla Connection/Component/Detail/Seam，作为 Assembly 的语义化别名 |
| `OBJECT_TYPE_REBAR` | 新增 | 对应 Reinforcement 体系，虽然几何是 LinearComponent，但需独立分类便于钢筋详图 |
| `OBJECT_TYPE_FASTENER` | 已有，扩展为 BoltArray/BoltCircle/BoltXYList | 对应 Tekla 三种螺栓组布置 |
| `OBJECT_TYPE_WELD` | 新增 | 对应 Tekla Weld/PolygonWeld，作为 Assembly 的修饰对象 |

#### 10.2.4 不需要做

- **创建独立 PrimitiveGen 模块**：Tekla 的 Primitive 类（ContourPlate/LoftedPlate）通过 FeatureTreeTemplate + ParametricOperation 处理，与 OmniM 当前 Catalog + Feature Engine 路径一致。
- **为 Girt/Purlin/WindBrace 定义独立 ObjectType**：Tekla 用 Beam 统一处理，OmniM 应保持相同策略。
- **将 Tekla Brep 类映射为 Primitive::Brep**：Tekla Brep 实际是截面扫掠体，应映射为 LinearComponent。

### 10.3 与 Revit / SOLIDWORKS 调研的对比

| 维度 | Revit（建筑/MEP） | SOLIDWORKS（机械） | Tekla（结构详图） |
|------|------------------|-------------------|------------------|
| 建模哲学 | 族驱动（Family/Symbol/Instance） | 特征树驱动（FeatureManager） | 零件+组件驱动（Part + Component） |
| 几何创建 | GenericForm（Extrusion/Sweep/Revolve/Loft/SweptBlend） | 特征（Extrusion/Revolve/Sweep/Loft/Fillet/Chamfer/...） | Part 子类（Beam/ContourPlate/PolyBeam/LoftedPlate）+ Boolean |
| 线构件 | FamilyInstance + StructuralType | Structural Member（焊件） | Beam（统一所有线构件） |
| 板件 | Floor/Roof（系统族） | Sheet Metal / Plate | ContourPlate（统一所有板件） |
| 连接节点 | 不内置（需手动建模或第三方） | 装配体配合（Mate） | Component 系统（端板/底板/角钢等内置组件） |
| 钢筋 | Rebar Bar + Rebar Host | 不支持 | Reinforcement 体系（7 类，最成熟） |
| 型材库 | Family 库（.rfa） | Toolbox + Weldment Profile | Profile Library（17 类，最专业） |
| 布尔切割 | Void Form（空心族） | Combine（添加/减除/共同） | BooleanPart/Fitting/CutPlane（最完整） |
| 装配体 | 不支持（项目即整体） | .sldasm（多零件 + 配合） | Assembly（主件 + 副件，Component 自动构建） |
| 对应 OmniM GenRule | LayerExtrusion + LinearComponent | Primitive + BooleanSubtract | LinearComponent + Primitive + Assembly + BooleanSubtract |

**核心结论**：Tekla 的"Part + Component"双层建模与 OmniM 的"Object + GenRule"双层架构高度对应。Tekla 的 Component 系统是 OmniM `Assembly` GenRule 的工业原型——过程化生成多个子零件，支持输入契约 + 嵌套。Tekla 的 Profile Library 与 Beam 类共同验证了 `LinearComponent`（固定截面 + 路径）是结构工程最核心的 GenRule。Tekla 的 7 类钢筋对象验证了 `LinearComponent`（圆截面 + 任意路径 + 阵列）在混凝土详图中的关键作用。Tekla 的 BooleanPart/Fitting/CutPlane 验证了 `BooleanSubtract` 需同时支持"实体切割"和"半空间平面切割"两种模式。
