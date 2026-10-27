# Revit 内置构件底层逻辑调研

> 调研日期：2026-07-09
> 来源：Revit 2027 API Documentation、Revit API Developers Guide、revitapidocs.com

---

## 1. 概述

Revit 基于 **族 (Family)** 体系构建所有内置构件，采用 **Element → Family → ElementType → Instance** 四层继承架构。几何生成分为**族文档内创建**（通过 GenericForm 子类）和**项目文档内创建**（通过 GeometryCreationUtilities 工具类）两条路径。

---

## 2. 元素层级体系

### 2.1 核心继承树

```
Element (所有元素基类)
├── Category (类别)
├── Family (族)
│   └── FamilySymbol (族类型)
│       └── FamilyInstance (族实例)
├── Wall (墙)
├── Floor (楼板)
├── Roof (屋顶)
├── HostObjAttributes (主体对象属性)
│   ├── WallType (墙类型)
│   ├── FloorType (楼板类型)
│   └── RoofType (屋顶类型)
├── Structure (结构)
│   └── FamilyInstance (StructuralType = Column/Beam/Brace)
└── MEP (机电)
    ├── Duct (风管)
    ├── Pipe (管道)
    └── CableTray (电缆桥架)
```

### 2.2 Category（类别）系统

Category 是 Revit 元素分类的最顶层，用于过滤和分组：

```csharp
BuiltInCategory
├── OST_Walls              // 墙
├── OST_StructuralColumns  // 结构柱
├── OST_StructuralFraming  // 结构梁/支撑
├── OST_StructuralFoundation // 基础
├── OST_Floors             // 楼板
├── OST_Roofs              // 屋顶
├── OST_Doors              // 门
├── OST_Windows            // 窗
├── OST_PipeCurves         // 管道
├── OST_DuctCurves         // 风管
└── OST_CableTray          // 电缆桥架
```

### 2.3 结构构件分类

结构柱、梁、支撑使用 `FamilyInstance` 类，通过 `StructuralType` 属性区分：

```csharp
public enum StructuralType {
    Column,     // 结构柱
    Beam,       // 结构梁
    Brace,      // 支撑
    Footing,    // 基础
    Stiffener,  // 加劲肋
    BeamSystem, // 梁系统
    Girt,       // 墙梁
    Purlin,     // 檩条
    Truss,      // 桁架
    KickerBrace // 斜撑
}

// 区分梁/柱/支撑
public void GetStructuralType(FamilyInstance instance) {
    switch (instance.StructuralType) {
        case StructuralType.Beam:
            // 梁处理逻辑
            break;
        case StructuralType.Column:
            // 柱处理逻辑
            break;
        case StructuralType.Brace:
            // 支撑处理逻辑
            break;
    }
}
```

---

## 3. 族（Family）体系

### 3.1 族分类

| 族类型 | 说明 | 示例 |
|--------|------|------|
| **系统族 (System Family)** | 内置于 Revit，不可删除 | Wall, Floor, Roof, Duct, Pipe |
| **可载入族 (Loadable Family)** | .rfa 文件，可载入项目 | 门、窗、家具、设备 |
| **内建族 (In-Place Family)** | 在项目中直接创建，不可复用 | 自定义构件 |

### 3.2 族文档内的几何创建

族文档中使用 `FamilyItemFactory` 创建几何体，所有方法返回 `GenericForm` 子类：

```csharp
// FamilyItemFactory 提供的方法
public class FamilyItemFactory {
    // 拉伸 (Extrusion)
    Extrusion NewExtrusion(
        bool isSolid,           // 实体/空心
        CurveArrArray profile,  // 轮廓
        SketchPlane sketchPlane, // 草图平面
        double end              // 拉伸深度
    );

    // 融合 (Blend)
    Blend NewBlend(
        bool isSolid,
        CurveArrArray topProfile,    // 顶部轮廓
        CurveArrArray bottomProfile, // 底部轮廓
        SketchPlane sketchPlane
    );

    // 旋转 (Revolution)
    Revolution NewRevolution(
        bool isSolid,
        CurveArrArray profile,  // 轮廓
        SketchPlane sketchPlane,
        Curve axis              // 旋转轴
    );

    // 放样 (Sweep)
    Sweep NewSweep(
        bool isSolid,
        CurveArray path,          // 路径
        SketchPlane pathPlane,    // 路径平面
        SweepProfile profile,     // 放样轮廓
        int profileLocationCurveIndex,
        ProfilePlaneLocation profilePlaneLocation
    );

    // 放样融合 (SweptBlend)
    SweptBlend NewSweptBlend(
        bool isSolid,
        CurveArray path,
        SketchPlane pathPlane,
        CurveArrArray topProfile,
        CurveArrArray bottomProfile
    );
}
```

### 3.3 拉伸示例

```csharp
// 创建矩形拉伸
private Extrusion CreateExtrusion(Document doc, SketchPlane sketchPlane) {
    // 定义矩形轮廓
    CurveArrArray curves = new CurveArrArray();
    CurveArray profile = new CurveArray();

    XYZ p0 = XYZ.Zero;
    XYZ p1 = new XYZ(10, 0, 0);
    XYZ p2 = new XYZ(10, 10, 0);
    XYZ p3 = new XYZ(0, 10, 0);

    profile.Append(Line.CreateBound(p0, p1));
    profile.Append(Line.CreateBound(p1, p2));
    profile.Append(Line.CreateBound(p2, p3));
    profile.Append(Line.CreateBound(p3, p0));
    curves.Append(profile);

    // 创建实体拉伸
    return doc.FamilyCreate.NewExtrusion(true, curves, sketchPlane, 10);
}
```

### 3.4 放样示例

```csharp
// 创建放样
private Sweep CreateSweep(Document doc, SketchPlane sketchPlane) {
    // 定义卵形轮廓
    CurveArrArray arrarr = new CurveArrArray();
    CurveArray arr = new CurveArray();

    XYZ pnt1 = new XYZ(0, 0, 0);
    XYZ pnt2 = new XYZ(2, 0, 0);
    XYZ pnt3 = new XYZ(1, 1, 0);

    arr.Append(Arc.Create(pnt2, 1.0, 0.0, 180.0, XYZ.BasisX, XYZ.BasisY));
    arr.Append(Arc.Create(pnt1, pnt3, pnt2));
    arrarr.Append(arr);
    SweepProfile profile = doc.Application.Create.NewCurveLoopsProfile(arrarr);

    // 定义路径
    Curve curve = Line.CreateBound(new XYZ(10, 0, 0), new XYZ(0, 10, 0));
    CurveArray curves = new CurveArray();
    curves.Append(curve);

    // 创建放样
    return doc.FamilyCreate.NewSweep(true, curves, sketchPlane, profile, 0, ProfilePlaneLocation.Start);
}
```

---

## 4. 项目文档中的几何创建

### 4.1 GeometryCreationUtilities

项目文档中使用 `GeometryCreationUtilities` 创建 Solid 对象（不直接添加到文档，需通过 DirectShape 包装）：

```csharp
public static class GeometryCreationUtilities {
    // 拉伸
    static Solid CreateExtrusionGeometry(
        IList<CurveLoop> profileLoops,  // 轮廓（闭合曲线环）
        XYZ extrusionDir,               // 拉伸方向
        double extrusionDist            // 拉伸距离
    );

    // 放样
    static Solid CreateSweptGeometry(
        CurveLoop sweepPath,            // 放样路径
        int pathAttachmentCrvIdx,       // 轮廓所在路径段下标
        double pathAttachmentParam,     // 轮廓在路径上的参数化位置
        IList<CurveLoop> profileLoops,  // 轮廓
        SolidOptions solidOptions
    );

    // 旋转
    static Solid CreateRevolvedGeometry(
        Frame coordinateFrame,          // 坐标系
        IList<CurveLoop> profileLoops,  // 轮廓（必须在 XZ 面上）
        double startAngle,              // 起始角度
        double endAngle,                // 终止角度
        SolidOptions solidOptions
    );

    // 放样（Loft）
    static Solid CreateLoftGeometry(
        IList<CurveLoop> profileLoops,  // 多个轮廓
        SolidOptions solidOptions
    );
}
```

### 4.2 布尔运算

```csharp
public static class BooleanOperationsUtils {
    // 布尔运算（不修改原始 Solid）
    static Solid ExecuteBooleanOperation(
        Solid solidA,
        Solid solidB,
        BooleanOperationsType operation  // Union, Difference, Intersect
    );

    // 修改原始 Solid 的布尔运算
    static void ExecuteBooleanOperationModifyingOriginalSolid(
        Solid solidA,
        Solid solidB,
        BooleanOperationsType operation
    );

    // 半空间剪切
    static Solid CutWithHalfSpace(
        Solid solid,
        Plane plane
    );
}
```

### 4.3 DirectShape 包装

将创建的 Solid 添加到项目文档：

```csharp
public class DirectShape {
    // 创建 DirectShape 元素
    static DirectShape CreateElement(
        Document doc,
        ElementId categoryId,  // 所属类别
        string externalId,     // 外部标识
        string applicationId   // 应用标识
    );

    // 设置形状几何
    void SetShape(IList<GeometryObject> shapes);
}
```

---

## 5. 面构件（Wall/Floor/Roof）

### 5.1 墙（Wall）

墙是 Revit 最重要的**面构件**，通过 CurveLoop 生成：

```csharp
// 创建墙
public static Wall Create(
    Document doc,
    Curve curve,             // 墙的中心线路径
    ElementId wallTypeId,    // 墙类型 ID
    ElementId levelId,       // 标高 ID
    double height,           // 墙高度
    double offset,           // 底部偏移
    bool flip,               // 翻转方向
    bool structural          // 是否结构构件
);
```

**墙的几何结构**：

```
Wall
├── Location Curve（定位线）
├── WallType（墙类型）
│   ├── Structure（层结构）
│   │   ├── Layer 1: 砖饰面 (10mm)
│   │   ├── Layer 2: 保温层 (50mm)
│   │   ├── Layer 3: 混凝土 (200mm)
│   │   └── Layer 4: 抹灰 (10mm)
│   ├── Function（功能：Exterior/Interior/Foundation）
│   └── Wrapping（包裹设置）
├── Top Constraint（顶部约束）
├── Bottom Constraint（底部约束）
└── Openings（开洞）
    ├── Door Opening
    └── Window Opening
```

### 5.2 楼板（Floor）

楼板通过封闭轮廓 + 类型定义创建：

```csharp
// 创建楼板
public static Floor Create(
    Document doc,
    IList<CurveLoop> profile,  // 边界轮廓（闭合曲线环）
    ElementId floorTypeId,     // 楼板类型 ID
    ElementId levelId          // 标高 ID
);
```

### 5.3 屋顶（Roof）

```csharp
// 创建拉伸屋顶
public static RoofBase Create(
    Document doc,
    ElementId roofTypeId,
    ElementId levelId,
    ReferenceArray references,  // 参考线
    CurveArray extrusionCurve,  // 拉伸轮廓
    double extrusionStart,      // 拉伸起始
    double extrusionEnd         // 拉伸终止
);
```

---

## 6. 线构件（Beam/Column/Brace）

结构线构件通过定位线和 FamilySymbol 创建：

```csharp
// 创建结构梁
public static FamilyInstance Create(
    Document doc,
    ElementId familySymbolId,  // 族类型 ID
    Curve curve,               // 定位线
    ElementId levelId,         // 标高 ID
    StructuralType structuralType  // 结构类型
);

// 示例：创建梁
FamilyInstance beam = doc.Create.NewFamilyInstance(
    beamLine,                    // 梁的定位线
    beamSymbol,                  // 梁族类型
    level,                       // 标高
    StructuralType.Beam          // 结构类型
);

// 示例：创建柱
FamilyInstance column = doc.Create.NewFamilyInstance(
    columnLocation,              // 柱的位置点
    columnSymbol,                // 柱族类型
    columnLevel,                 // 标高
    StructuralType.Column        // 结构类型
);
```

---

## 7. MEP 构件（Pipe/Duct/CableTray）

MEP 构件使用基于路径的创建方式：

```csharp
// 创建管道
public static Pipe Create(
    Document doc,
    ElementId pipeTypeId,
    ElementId levelId,
    Connector startConnector,  // 起点连接器
    Connector endConnector     // 终点连接器
);

// 创建风管
public static Duct Create(
    Document doc,
    ElementId ductTypeId,
    ElementId levelId,
    Connector startConnector,
    Connector endConnector
);

// 电缆桥架
public static CableTray Create(
    Document doc,
    ElementId cableTrayTypeId,
    ElementId levelId,
    Connector startConnector,
    Connector endConnector
);
```

---

## 8. 参数化与属性系统

### 8.1 ElementType 参数

```csharp
public class ElementType {
    // 获取/设置参数值
    Parameter get_Parameter(BuiltInParameter paramId);
    void SetParameterValue(string paramName, object value);

    // 常见内建参数
    BuiltInParameter.WALL_STRUCTURAL_SIGNIFICANT_THICKNESS
    BuiltInParameter.FLOOR_ATTR_THICKNESS_PARAM
    BuiltInParameter.FAMILY_WIDTH_PARAM
    BuiltInParameter.FAMILY_HEIGHT_PARAM
    BuiltInParameter.SYMBOL_NAME_PARAM
}
```

### 8.2 族参数类型

| 参数类型 | 说明 | 示例 |
|---------|------|------|
| Length | 长度 | 宽度、高度、厚度 |
| Angle | 角度 | 坡度、旋转角度 |
| Number | 数值 | 数量、倍数 |
| Integer | 整数 | 分段数、层数 |
| Material | 材质 | 混凝土、钢材 |
| Yes/No | 布尔 | 是否结构构件 |
| Family Type | 族类型引用 | 连接件类型 |

---

## 9. 对 OmniM 的启示

| Revit 概念 | OmniM 等价概念 | 借鉴点 |
|-----------|---------------|--------|
| Family (族) | ComponentDef | 参数化构件定义 |
| FamilySymbol (族类型) | Series | 同类不同参数的构件变体 |
| FamilyInstance (族实例) | Component | 构件实例化 |
| Wall/Floor (面构件) | LayerExtrusion | 层结构沿路径拉伸 |
| Beam/Column (线构件) | LinearComponent | 路径+截面扫掠 |
| GenericForm (Extrusion/Sweep) | Feature | 特征操作（拉伸/放样/旋转） |
| GeometryCreationUtilities | Feature Engine | 几何创建工具集 |
| BooleanOperationsUtils | BooleanSubtract | 布尔运算 |
| Category (类别) | ObjectType | 对象类型分类 |
| DirectShape | BRep → Mesh | 第三方几何导入 |
| CurveLoop | Sketch | 二维轮廓定义 |
| HostObjAttributes (层结构) | LayerExtrusionGen | 多层材料定义 |

**核心结论**：Revit 的 Element/ElementType/Instance 分层与 OmniM 的 Object/Component/Feature 分层高度对应。Wall 的"定位线 + 层结构 + 开洞" 模式直接对应 OmniM 的 LayerExtrusionGen，Beam 的"定位线 + 截面" 模式对应 LinearComponentGen 的特例（固定截面）。