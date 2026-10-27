# AVEVA E3D / PDMS 内置构件底层逻辑调研

> 调研日期：2026-07-09
> 来源：AVEVA E3D 2.1 Documentation、PDMS Design Module Guide、AVEVA E3D Equipment Modeling Guide

---

## 1. 概述

AVEVA E3D（原 PDMS，Plant Design Management System）是面向工厂设计的全专业三维设计平台。其核心设计哲学是**基本体拼装 (Primitive Assembly)**——所有设备由基本体（Primitives）像搭积木一样组合而成，管道、结构、电气等专业使用各自的元件库。

---

## 2. 数据库层级结构

### 2.1 元素层次树

```
WORLD (世界)
└── SITE (场地)
    └── ZONE (区域)
        ├── EQUIPMENT (EQUI) — 设备
        │   ├── SUB-EQUIPMENT (SUBE) — 子设备
        │   │   ├── PRIMITIVE — 基本体
        │   │   └── PRIMITIVE
        │   └── PRIMITIVE — 基本体
        │       ├── CYLI (圆柱)
        │       ├── BOX (立方体)
        │       └── NOZZ (管嘴)
        ├── STRUCTURES (STRU) — 结构管理
        │   └── FRAMEWORK (FRMW) — 框架
        │       ├── SUB-FRAMEWORK (SBFR)
        │       │   ├── GENSEC (通用型钢)
        │       │   └── PANE (板)
        │       └── GENSEC
        ├── PIPE (管道管理)
        │   └── BRANCH (BRAN) — 分支
        │       ├── COMPONENT: ELBO (弯头)
        │       ├── COMPONENT: FLAN (法兰)
        │       ├── COMPONENT: VALV (阀门)
        │       └── COMPONENT: TEE (三通)
        └── CABLE TRAY (桥架管理)
            └── BRANCH
                └── COMPONENT: 桥架元件
```

### 2.2 元素类型编码

| 编码 | 全称 | 说明 |
|------|------|------|
| EQUI | Equipment | 设备，Primitive 的容器 |
| SUBE | Sub-Equipment | 子设备，设备的分组容器 |
| STRU | Structures | 结构管理容器 |
| FRMW | Framework | 结构框架 |
| SBFR | Sub-Framework | 子框架 |
| GENSEC | General Section | 通用型钢（I 型、H 型等） |
| PANE | Panel | 板 |
| PIPE | Pipe | 管道管理容器 |
| BRAN | Branch | 管道分支 |
| NOZZ | Nozzle | 管嘴（特殊基本体） |

---

## 3. 基本体系统 (Primitives)

### 3.1 基本体类型

基本体是 E3D 最基本的几何构建单元，每种基本体有独立的参数系统和特征点：

| 基本体 | 缩写 | 说明 | 关键参数 |
|--------|------|------|---------|
| **Box** | BOX | 立方体 | X, Y, Z 三个方向尺寸 |
| **Cylinder** | CYLI | 圆柱 | 直径 (DIA)、高度 (HEIGHT) |
| **Cone** | CONE | 圆台 | 顶径 (TDIA)、底径 (BDIA)、高度 (HEIGHT) |
| **Snout** | SNOUT | 偏心圆台 | 顶径、底径、高度、偏心距 |
| **Pyramid** | PYRA | 角锥（棱台） | 顶部 X/Y、底部 X/Y、高度 |
| **Dish** | DISH | 碟形封头 | 直径、深度、类型 |
| **Dome** | DOME | 球冠 | 直径、高度 |
| **Sphere** | SPHR | 球体 | 直径 |
| **Torus** | TORUS | 圆环 | 主半径、次半径 |
| **Ctubend** | CTUB | 斜切圆柱 | 直径、高度、斜切角度 |
| **Ptube** | PTUB | 棱柱管 | 截面多边形、高度 |
| **Extrusion** | EXTR | 拉伸体 | 轮廓曲线、拉伸距离 |
| **Revolution** | REVO | 旋转体 | 轮廓曲线、旋转轴、角度 |
| **Nozzle** | NOZZ | 管嘴 | 公称直径、壁厚、法兰类型 |

### 3.2 特征点系统 (P-Points)

每个基本体有预定义的特征点，用于定位和连接：

```
Cylinder 的特征点:
  P0: 圆柱体中心（原点）
  P1: 顶部中心
  P2: 底部中心
  P3: 顶部圆周 0° 方向
  P4: 顶部圆周 90° 方向
  P5: 底部圆周 0° 方向
  P6: 底部圆周 90° 方向

Box 的特征点:
  P0: 盒子中心
  P1-P8: 八个顶点
  P9-P14: 六个面的中心

Nozzle 的特征点:
  P0: 管嘴根部中心
  P1: 管嘴端面中心
  P2: 法兰面中心
```

### 3.3 负实体 (Negative Primitive)

基本体可以是实体或负实体，负实体用于在已有基本体上挖孔：

```pml
! 创建正圆柱体（实体）
NEW CYLI
DIA 1000
HEIGHT 2000
! 创建负圆柱体（挖孔）
NEW CYLI
NEGATIVE
DIA 500
HEIGHT 2000
! 定位负实体到正实体中心
POSITION P0 OF CYLI_1
```

### 3.4 设备建模示例

```pml
! 创建立式储罐设备
! 1. 创建设备
NEW EQUI TANK-101
NAME "储罐 T-101"

! 2. 创建筒体（圆柱）
NEW CYLI
DIA 3000
HEIGHT 5000
NAME "Shell"

! 3. 创建顶部封头（碟形封头）
NEW DISH
DIA 3000
DEPTH 750
POSITION P1 OF Shell
NAME "Top Head"

! 4. 创建底部封头
NEW DISH
DIA 3000
DEPTH 750
POSITION P2 OF Shell
NAME "Bottom Head"

! 5. 创建管嘴
NEW NOZZ ! 默认连接在 P1（顶部中心）
DIA 400
THICKNESS 10
! 从 P0 偏移到指定位置
POSITION P0 OF Top Head
NAME "Inlet Nozzle"

! 6. 负实体创建人孔
NEW CYLI
NEGATIVE
DIA 600
HEIGHT 500
POSITION X 1500 Y 0 Z 2500
NAME "Manhole Cut"
```

---

## 4. 管道系统 (Piping)

### 4.1 管道层级

```
PIPE (管道管理容器)
└── BRANCH (分支)
    ├── BRAN 起始点 (Head)
    ├── COMPONENT: 弯头 (ELBO)
    ├── COMPONENT: 管道段 (PIPE SEGMENT)
    ├── COMPONENT: 三通 (TEE)
    ├── COMPONENT: 阀门 (VALV)
    ├── COMPONENT: 法兰 (FLAN)
    ├── COMPONENT: 垫片 (GASKET)
    ├── COMPONENT: 变径 (REDUCER)
    └── BRAN 终止点 (Tail)
```

### 4.2 管道元件类型

| 编码 | 元件类型 | 说明 |
|------|---------|------|
| ELBO | 弯头 | 90°/45° 弯头 |
| BEND | 弯管 | 大半径弯管 |
| TEE | 三通 | 等径/异径三通 |
| VALV | 阀门 | 闸阀、截止阀、球阀、蝶阀等 |
| FLAN | 法兰 | 各种标准法兰 |
| GASK | 垫片 | 法兰垫片 |
| REDUCER | 变径 | 同心/偏心变径 |
| VENT | 放空 | 放空管件 |
| DRAIN | 排净 | 排净管件 |
| CAP | 管帽 | 管道封头 |
| OLET | 接头 | 焊接支管台 |

### 4.3 管道规范系统

```
Piping Specification (管道等级)
├── Header (头部信息)
│   ├── Spec Name: 3A1
│   ├── Pressure Class: PN16
│   └── Temperature Range: -20°C ~ 300°C
├── Materials (材料)
│   ├── Carbon Steel A106B
│   └── Stainless Steel 304L
├── Components (元件列表)
│   ├── Pipe: 3A1-PIPE-CS-SCH40
│   ├── Elbow: 3A1-ELBOW-LR-CS-SCH40
│   ├── Flange: 3A1-FLANGE-WN-CS-150RF
│   └── Valve: 3A1-GATE-VALVE-CS-150RF
└── Constraints (约束)
    ├── Min Wall Thickness
    └── Max Pressure
```

---

## 5. 结构系统 (Structures)

### 5.1 结构构件类型

| 编码 | 构件类型 | 说明 |
|------|---------|------|
| GENSEC | 通用型钢 | I/H/U/L/T 型钢等，通过截面规格选择 |
| PANE | 板 | 钢板、平台板 |
| GRATING | 格栅板 | 钢格栅 |
| STAIR | 楼梯 | 钢楼梯 |
| HANDR | 扶手 | 栏杆扶手 |
| LADDER | 梯子 | 直梯/斜梯 |

### 5.2 型钢截面系统

GENSEC 通过**截面规格 (Section Specification)** 引用标准型钢库：

```pml
! 创建梁
NEW GENSEC
NAME "BEAM-01"
! 选择 I 型钢截面
SECTION "IPE300"
! 定位
POSITION X 0 Y 0 Z 0
DIRECTION X 1 Y 0 Z 0
LENGTH 6000
! 旋转
ROTATION Y 90
```

### 5.3 板系统

```pml
! 创建钢板
NEW PANE
NAME "PLATFORM-01"
THICKNESS 10
! 定义轮廓
PROFILE "1000,2000,3000,2000,4000,0,0,0"
! 定位
POSITION X 5000 Y 2000 Z 10000
```

---

## 6. 目录与规范系统 (Catalog & Spec)

### 6.1 目录层次

```
Catalogue (目录)
├── Component Catalogue (元件目录)
│   ├── Piping Components (管道元件)
│   │   ├── Elbows (弯头)
│   │   ├── Flanges (法兰)
│   │   └── Valves (阀门)
│   ├── Structural Components (结构元件)
│   │   ├── Steel Sections (型钢截面)
│   │   └── Plates (板)
│   └── Equipment Templates (设备模板)
│       ├── Pumps (泵)
│       ├── Tanks (储罐)
│       └── Heat Exchangers (换热器)
├── Piping Specification (管道等级)
│   ├── Spec 3A1 (碳钢)
│   ├── Spec 3B1 (不锈钢)
│   └── Spec 3C1 (合金钢)
└── Design Templates (设计模板)
    ├── Standard Equipment (标准设备)
    ├── Supports (支吊架)
    └── Steel Connections (节点连接)
```

### 6.2 参数化设备模板

E3D 提供标准设备模板，通过参数驱动生成：

```pml
! 使用标准泵模板
NEW EQUI PUMP-101
TEMPLATE "AVEVA_STD_EQUIP.PUMP_Centrifugal"
! 设置模板参数
PARAMETER A = 1800   ! 底板长度
PARAMETER B = 600    ! 底板宽度
PARAMETER C = 100    ! 底板厚度
PARAMETER D = 300    ! 底座到中心高度
PARAMETER E = 250    ! 出口管嘴高度
PARAMETER F = 200    ! 进口管嘴高度
```

---

## 7. 设计模板系统 (Design Templates)

### 7.1 模板定义

设计模板是参数化基本体的组合，可快速生成标准设备：

```pml
! 定义参数化立式容器模板
DEFINE TEMPLATE "Vertical_Vessel"
    PARAMETER DIA = 1000      ! 直径
    PARAMETER HEIGHT = 3000   ! 筒体高度
    PARAMETER HEAD_DEPTH = 250  ! 封头深度
    PARAMETER WALL_THK = 10    ! 壁厚

    ! 筒体
    CYLI
        DIA = DIA
        HEIGHT = HEIGHT
        WALL_THK = WALL_THK

    ! 上封头
    DISH
        DIA = DIA
        DEPTH = HEAD_DEPTH
        AT = P1 OF CYLI_1

    ! 下封头
    DISH
        DIA = DIA
        DEPTH = HEAD_DEPTH
        AT = P2 OF CYLI_1
ENDTEMPLATE
```

---

## 8. 对 OmniM 的启示

| E3D 概念 | OmniM 等价概念 | 借鉴点 |
|---------|---------------|--------|
| EQUI/SUBE (设备容器) | OmniObject | 对象层级容器 |
| Primitive (基本体) | GeometryPrimitive | 基本几何体（Box/Cylinder/Cone） |
| Negative Primitive | BooleanSubtract | 布尔差集运算 |
| P-Point (特征点) | Connector | 连接点系统 |
| NOZZ (管嘴) | Connector with type | 带类型码的连接器 |
| GENSEC (型钢) | LinearComponent | 固定截面线构件 |
| PANE (板) | LayerExtrusion | 面构件 |
| Piping Specification | Series | 等级库/系列库 |
| Component Catalogue | CatalogStore | 元件目录 |
| Design Template | FeatureTreeTemplate | 特征树模板 |
| Section Specification | MaterialProfileSet | 型钢截面规格 |
| BRANCH (分支) | LinearComponent | 路径+截面+元件序列 |

**核心结论**：E3D 的"基本体拼装 + 目录规范"体系为 OmniM 的 PrimitiveGen 和 AssemblyGen 提供了直接参考。其 P-Point 连接点系统与 OmniM 的 Connector 系统高度一致。管道等级库 (Spec) 的概念对应 OmniM 的 Series/SeriesStore。设备模板对应 OmniM 的 FeatureTreeTemplate。