# AVEVA E3D 仪表/电气内置组件逻辑研究

> 调研日期：2026-08-07
> 来源：AVEVA E3D 2.1 Documentation (help.aveva.com)、AVEVA Instrumentation 产品手册、AVEVA Electrical 培训课程、AVEVA E3D Design Brochure、ANSI/ISA-5.1-2009 标准、AVEVA Engineering Electrical/Instrumentation User 培训大纲

---

## 1. 平台概述

### 1.1 产品线历史与定位

AVEVA 的仪表与电气产品线源自两条历史脉络，最终在 AVEVA 统一工程（Unified Engineering）框架下与 E3D 三维设计平台集成：

| 产品 | 前身 | 定位 | 现归属 |
|------|------|------|--------|
| **AVEVA Instrumentation** | Intergraph INtools（6.0 前）→ SmartPlant Instrumentation / SPI（7.0 起） | 仪表与系统工程设计、回路/接线/电缆文档 | AVEVA Unified Engineering |
| **AVEVA Electrical** | Intergraph SmartPlant Electrical (SPEL) | 高/中/低压电气网络设计、负荷/电缆/盘柜文档 | AVEVA Unified Engineering |
| **AVEVA E3D Design** | AVEVA PDMS | 多专业三维物理设计（含电气桥架/电缆布线） | AVEVA Unified Engineering |

INtools 早期由以色列公司开发，被 Intergraph 收购后纳入 SmartPlant 系列（即 SPI），Intergraph PPM（过程与电力）部门被 Hexagon 收购后，AVEVA 又将部分工程产品线整合。当前 AVEVA Instrumentation 与 AVEVA Electrical 共享同一 Microsoft SQL 多用户数据库，并可与 E3D 进行双向数据交换。

### 1.2 逻辑视图与物理视图的双架构

AVEVA 工厂设计体系的核心是**逻辑（schematic）与物理（physical）双视图分离**，二者通过数据库交叉引用（cross-reference）关联：

```
┌─────────────────────────────────────────────────────────────────┐
│                    逻辑视图 (Logical / Schematic)               │
│   AVEVA Instrumentation / AVEVA Electrical / AVEVA P&ID         │
│   ─────────────────────────────────────────────────────         │
│   · 仪表索引 (Instrument Index)、回路 (Loop)、位号 (Tag)         │
│   · P&ID 流程图、回路图 (Loop Diagram)、接线图 (Termination)     │
│   · 电缆清册 (Cable Schedule)、端子排 (Terminal Strip)          │
│   · 信号 (Signal)、I/O 模块、跨接 (Cross Patching)              │
│   · 单线图 (SLD)、负荷表 (Load List)、配电板 (Distribution Board)│
└──────────────────────────────┬──────────────────────────────────┘
                               │  双向集成 (Cable data 共享 + 长度回传)
                               │  Engineering DB / Schematic DB ↔ Model DB
┌──────────────────────────────┴──────────────────────────────────┐
│                    物理视图 (Physical / 3D)                     │
│              AVEVA E3D Design (MODEL 数据库)                    │
│   ─────────────────────────────────────────────────────         │
│   · 设备 (Equipment) 物理放置：泵/容器/换热器/仪表外壳          │
│   · 电缆桥架 (Cable Tray) 系统布线：直段/弯头/三通/变径         │
│   · 导管 (Conduit) 走向                                          │
│   · 电缆 (Cable) 在桥架内的自动路由                              │
│   · 支吊架 (Support / Hanger) 物理安装                           │
└─────────────────────────────────────────────────────────────────┘
```

**关键数据流**（基于 E3D Brochure 与 AVEVA Engineering 培训大纲）：

1. **逻辑 → 物理**：AVEVA Instrumentation/Electrical 中的电缆定义（起点设备、终点设备、信号类型、芯数）传递至 E3D，E3D 根据可配置规则（桥架填充率、信号/动力电缆分隔、路径选择）自动将电缆路由进桥架。
2. **物理 → 逻辑**：E3D 计算出的精确电缆长度回传至 Instrumentation/Electrical，生成完整的电缆清册（Cable Schedule）。
3. **回路一致性**：逻辑侧的回路（Loop）与位号（Tag）贯穿两视图，P&ID 上的仪表符号与 E3D 中放置的物理仪表共享同一 Tag。

### 1.3 E3D 数据库层级中的双视图落点

E3D 项目数据库类型（基于 help.aveva.com E3D 2.1 Getting Started）明确分离了逻辑与物理数据：

| 数据库 | 内容 | 视图归属 |
|--------|------|----------|
| **MODEL** | 三维几何：设备/管道/结构/桥架/支吊架的完整三维表示 | 物理 |
| **PADD (DRAW)** | 图纸的图形与标注信息 | 物理出图 |
| **ISOD** | 管道轴测图（spool drawing） | 物理出图 |
| **SCHEMATIC** | 2D 原理图模型的数据与连通性，及与原理图图纸的交叉引用 | 逻辑 |
| **ENGINEERING** | 常规工程数据（仪表索引、电缆清册等） | 逻辑 |
| **CATALOGUE** | 标准元件目录：尺寸、允许连接、螺栓、规范（压力/温度） | 共享参考 |
| **LEXICON** | 用户自定义属性 (UDA) 与自定义元素类型 (UDET)，例如把 EQUI 重命名为 :PUMP / :VESSEL | 共享参考 |
| **PROPERTIES** | 材料属性（应力分析等补充数据） | 共享参考 |

### 1.4 AVEVA Instrumentation 的四模块结构

AVEVA Instrumentation 由四个集成模块组成，共享同一 SQL 数据库（基于 AVEVA 官方产品手册与培训大纲）：

| 模块 | 职责 | 主要交付物 |
|------|------|-----------|
| **Instrument Engineer** | 仪表与回路索引数据录入、变更跟踪、数据表 (Datasheet)、报告/文档 | 仪表索引、数据表、仪表清单（含 DCS/PLC/I/O 信息） |
| **Wiring Manager** | 设备/端子布局定义、电缆与互连、接线管理、电缆块图 | 电缆清册、端子图、电缆块图、接线图 |
| **Designer** | 自动生成 AutoCAD 图纸：回路图、勾连图 (Hook-up)、安装详图、BOM | 回路图、勾连图、BOM |
| **Browser / Calculation / Binder** | 浏览/过滤/批量更新、仪表尺寸计算、文档装订 | EDE 视图、计算报告、规范装订册 |

**核心实体**：Plant（工厂）/System（系统）→ Loop（回路）→ Instrument（仪表，含 Tag）→ Signal（信号）→ Cable（电缆）→ Termination（端子）→ Equipment/Panel/JB（设备/盘柜/接线盒）。

### 1.5 AVEVA Electrical 的模块结构

AVEVA Electrical 与 Instrumentation 模块结构对称（基于 AVEVA Engineering 15.6 Electrical 培训课程）：

| 模块 | 职责 | 主要交付物 |
|------|------|-----------|
| **Engineer** | 电气负荷 (Load)、电源 (Supply)、配电室 (Compartment)、负荷分配至电源/配电板 | 负荷表、电源表、单线图 (SLD/KOL) |
| **Wiring Manager** | 电缆创建、端子连接、电缆卷筒 (Drumming)、电缆穿越 (Transit)、自动接线规则 | 电缆清册、端子报告、电缆块图 |
| **Designer** | 自动生成 AutoCAD 单线图与原理图 | 单线图、原理图、安装详图 |

**核心实体**：Plant/System → Electrical Supply → Distribution Board / MCC / Panel → Load（电机/照明/小动力）→ Cable → Termination。AVEVA Electrical 还可与 ETAP 双向同步进行电气系统分析计算。

---

## 2. 仪表组件 (Instrumentation)

### 2.1 仪表 Instrument (通用)

#### 2.1.1 逻辑表示

仪表在逻辑视图中的核心标识是 **ISA-5.1 位号 (Tag)**，结构为：

```
[功能字母] - [回路号] [后缀]
   │          │        │
   │          │        └─ 同回路同类型多台时的区分 (A/B/C...)
   │          └─ 唯一回路编号 (如 1001)
   └─ 首字母(测量变量) + 后续字母(功能)
```

| 字母位 | 含义 | 示例 |
|--------|------|------|
| 首字母 | 测量/触发变量 | F=流量, T=温度, P=压力, L=液位, A=分析 |
| 后续字母 | 执行功能 | T=变送, I=指示, C=控制, V=阀门, E=传感元件, S=开关, A=报警 |
| 回路号 | 控制回路编号 | 101, 1001 |
| 后缀 | 冗余/并行实例 | A (主), B (备) |

**常见组合**：`FT-101A`（流量变送器，回路 101，A 路）、`PIC-202`（压力指示控制器，回路 202）、`TE-303`（温度元件，回路 303）、`FCV-405`（流量调节阀，回路 405）。

仪表在逻辑视图（P&ID / 回路图）中以**标准符号**表示（ISA-5.1）：
- 圆圈 (balloon) 表示单台仪表，圆圈内写位号
- 圆圈加横线表示主控室 (control room) 安装
- 圆圈加双横线表示现场盘 (field panel) 安装
- 无横线圆圈表示现场安装
- 正方形套圆圈表示 DCS 仪表
- 信号线类型区分：虚线=电信号，双虚线=气动信号，实线带双斜杠=毛细管

#### 2.1.2 物理表示

物理侧（E3D）中，仪表是 **EQUI/SUBE 容器内由基本体组合而成的参数化设备**，与 `aveva_e3d_primitives.md` 第 3 节描述的基本体拼装体系一致。典型现场仪表由以下部分组成：

```
Instrument (EQUI)
├── SUBE: Housing (外壳)        ← Cylinder/Box 基本体
├── SUBE: Display (显示头)      ← Box/Cylinder 基本体 (LCD 窗口)
├── SUBE: Process Connection    ← Nozzle/Cylinder (过程接管)
├── SUBE: Electrical Connection ← Nozzle/Cylinder (电缆进线口)
└── SUBE: Mounting Bracket      ← Box (安装支架)
```

AVEVA E3D 提供标准参数化设备模板（`AVEVA_STD_EQUIP`），通过字母参数（A/B/C/D...）驱动基本体尺寸，类似泵模板中的 `A=Baseplate Length, B=Baseplate Width, C=Baseplate Height...` 模式。

#### 2.1.3 关键参数

| 参数类别 | 属性 |
|----------|------|
| 标识 | Tag, Loop Number, P&ID Reference, Instrument Type |
| 过程连接 | 接管尺寸 (DN), 法兰等级 (PN/Class), 过程连接类型 (螺纹/法兰/卡套) |
| 电气连接 | 供电电压, 信号类型 (4-20mA HART / Foundation Fieldbus / Profibus) |
| 安装 | 安装方式 (现场/盘装/导轨), 防护等级 (IP), 防爆等级 (Ex) |
| 物理 | 外壳尺寸 (H×W×D), 重量, 材质 |

### 2.2 传感器 Sensor

#### 2.2.1 逻辑表示

传感器在 ISA-5.1 中用首字母 + `E`（Element，传感元件）表示：
- `TE` — Temperature Element（温度元件，热电偶/RTD）
- `FE` — Flow Element（流量元件，孔板/文丘里/涡街）
- `LE` — Level Element（液位元件）
- `PE` / `PSE` — Pressure Element / Pressure Safety Element

传感器是回路的"一次仪表"（primary element），将物理量转换为可测信号。在 P&ID 上通常画在工艺管线或设备上，通过信号线连接到变送器 (`TT`/`FT`/`LT`/`PT`)。

#### 2.2.2 物理表示

传感器本身几何通常较简单，多为**单一基本体**或其与保护套件的组合：

| 传感器类型 | 几何构成 |
|-----------|----------|
| 温度元件 (RTD/TC) | Cylinder (保护套管 thermowell) + Cylinder (传感元件) — 插入式 |
| 孔板 (Orifice Plate) | Cylinder (法兰组件) + 薄 Box (孔板) — 法兰间夹装 |
| 文丘里管 | Cone/Cylinder 拼接 (收缩段+喉部+扩散段) |
| 涡街/电磁流量计 | Cylinder (测量管段) + Box (变送器外壳) — 法兰连接 |
| 压力变送器本体 | Box/Cylinder (外壳) + Nozzle (过程接口) |

#### 2.2.3 GenRule 归属

传感器几何上多为单一基本体或少量基本体的简单组合（无复杂装配关系），按 GenRule 分类应归 **Primitive**（基本体+布尔）。但流量计等带变送器外壳的复合形态应归 **Assembly**。

### 2.3 变送器 Transmitter

#### 2.3.1 逻辑表示

变送器 ISA 位号以 `T`（Transmit）结尾：`PT`（压力）、`FT`（流量）、`LT`（液位）、`TT`（温度）、`AT`（分析）。其逻辑职责是将传感器信号转换为标准化远传信号（4-20mA / 数字总线）。

#### 2.3.2 物理表示（壳体 + 电子 + 显示 + 接线）

变送器是典型的**多子件装配体**，对应 OmniM 的 Assembly GenRule：

```
Transmitter (EQUI, Assembly)
├── SUBE: Housing (外壳)         ← Cylinder/Box  — 主承压/防护壳体
│   参数: DIA (直径), HEIGHT (高度), MATERIAL (材质), IP (防护等级)
├── SUBE: Electronics Module     ← Box  — 内置电子板（几何上常简化或省略）
├── SUBE: Display Head (显示头)  ← Box/Cylinder  — LCD/LED 就地显示
│   参数: DISPLAY_SIZE, ORIENTATION
├── SUBE: Process Connection     ← Nozzle  — 过程接口（法兰/螺纹）
│   参数: DN, PN, CONNECTION_TYPE
├── SUBE: Cable Gland (电缆格兰) ← Cylinder  — 电缆进线密封
└── SUBE: Terminal Block (端子排)← Box (内部) — 接线端子
```

各子件通过 P-Point（特征点）定位组装（参见 `aveva_e3d_primitives.md` 3.2 节的特征点系统），例如显示头 POSITION 到外壳的 P1（顶部中心），过程接口 POSITION 到外壳的 P2（底部中心）。

#### 2.3.3 关键参数

| 类别 | 参数 |
|------|------|
| 测量 | 测量范围 (LRV/URV), 量程比, 精度 |
| 过程 | 接管 DN/PN, 材质 (湿件) |
| 电气 | 供电 (24VDC), 信号 (4-20mA HART/FF/Profibus), 负载电阻 |
| 物理 | 外壳尺寸, 显示类型, 防护 IP66/67, 防爆 Ex ia/Ex d |
| 安装 | 直接安装/支架安装/导轨安装, 阀门定位安装 |

### 2.4 调节阀 ControlValve (含执行机构/定位器)

#### 2.4.1 逻辑表示

调节阀 ISA 位号以 `V` 或 `CV` 结尾：`FCV`（流量调节阀）、`PCV`（压力调节阀）、`LCV`（液位调节阀）、`TCV`（温度调节阀）。在 P&ID 上，调节阀符号是**阀门本体符号 + 执行机构符号**的组合（ISA-5.1）：

| 执行机构类型 | 符号特征 |
|-------------|----------|
| 气动薄膜 (Pneumatic Diaphragm) | 阀门上方半圆形/拱形 |
| 气动活塞 (Piston) | 阀门上方方形带活塞符号 |
| 电动 (Motor) | 圆圈内含 M |
| 电磁 (Solenoid) | 方框带 S |
| 手动 (Handwheel) | 阀杆顶部水平横线 |
| 弹簧返回 (Spring Return) | 倾斜弹簧线 |

**故障安全位置**用箭头标注：FO (Fail Open)、FC (Fail Close)、FL (Fail Last)。

#### 2.4.2 物理表示（阀体 + 执行机构 + 定位器 装配）

调节阀是典型的**多件装配体**（control valve assembly），由四个主要部分组成：

```
ControlValve (EQUI, Assembly)
├── SUBE: Valve Body (阀体)        ← 阀门本体（globe/ball/butterfly/diaphragm）
│   ├── Cylinder/Cone (阀体铸件)
│   ├── Nozzle × 2 (进出口接管)
│   └── Box (阀盖 bonnet)
├── SUBE: Actuator (执行机构)      ← 驱动阀杆的装置
│   ├── Cylinder (气室/活塞缸)
│   ├── Box (弹簧腔)
│   └── Cylinder (阀杆 stem)
├── SUBE: Positioner (定位器)      ← 高精度定位反馈装置
│   ├── Box (定位器外壳)
│   └── Cylinder (反馈连杆)
└── SUBE: Accessories (附件)
    ├── Solenoid Valve (电磁阀)
    ├── Limit Switch (限位开关)
    ├── Air Filter Regulator (过滤减压阀)
    └── Booster (放大器)
```

各子件通过阀杆 (stem) 轴线对齐，执行机构位于阀体上方，定位器侧装于执行机构。AVEVA E3D 中此类设备通过 `AVEVA_STD_EQUIP` 模板或自定义参数化模板生成。

#### 2.4.3 关键参数

| 类别 | 参数 |
|------|------|
| 阀体 | 阀型 (Globe/Ball/Butterfly/Diaphragm), 通径 (DN), 压力等级 (PN/Class), 流通能力 (Cv/Kvs), 阀体材质, 内件材质 |
| 执行机构 | 类型 (Pneumatic/Electric/Hydraulic), 尺寸, 弹簧范围, 气源压力 |
| 定位器 | 输入信号 (4-20mA/HART), 输出特性 (线性/等百分比), 反馈类型 |
| 故障安全 | Fail Action (FO/FC/FL) |
| 附件 | 电磁阀电压, 限位开关类型, 是否本安 |

### 2.5 接线盒 JunctionBox

#### 2.5.1 逻辑表示

接线盒在 AVEVA Instrumentation 的 Wiring Manager 模块中作为**端子聚合点**管理：现场仪表的多根电缆汇入 JB，通过端子排 (Terminal Strip) 转接为多芯电缆继续上传至控制室/盘柜。JB 在 Reference Explorer 中定义，可被多个回路共用。

#### 2.5.2 物理表示

```
JunctionBox (EQUI, Assembly)
├── SUBE: Enclosure (外壳)         ← Box  — 防护壳体
│   参数: H × W × D, MATERIAL, IP Rating, Ex Rating
├── SUBE: Cover (盖板)             ← Box  — 门/盖
├── SUBE: DIN Rail × N (导轨)      ← Box  — 端子安装导轨
├── SUBE: Terminal Strip (端子排)  ← 端子阵列
│   └── Terminal × N: Cylinder/Box
├── SUBE: Cable Glands (格兰)      ← Cylinder × N  — 电缆进线密封
└── SUBE: Mounting Plate (安装板)  ← Box  — 内部安装板
```

#### 2.5.3 关键参数

| 参数 | 说明 |
|------|------|
| 端子数 (Terminal Count) | 决定导轨数量与外壳尺寸 |
| 进线路数 (Cable Entries) | 格兰孔数量 |
| 防护等级 | IP56/65/66/67 |
| 防爆等级 | Ex e/Ex d/Ex ia |
| 材质 | 不锈钢/铝合金/玻璃钢 |
| 安装方式 | 壁挂/柱装/支架 |

---

## 3. 电气组件 (Electrical)

### 3.1 电缆桥架 CableTray

#### 3.1.1 与 `aveva_e3d_primitives.md` 的差异说明

`aveva_e3d_primitives.md` 第 2.1 节将 Cable Tray 列为 E3D 的顶层管理容器（与 EQUI/STRU/PIPE 并列的 `CABLE TRAY` 容器，下设 BRANCH 与 COMPONENT），描述的是**数据层级**。本节聚焦**电气工程视角**的桥架组件类型、几何与电气参数。

#### 3.1.2 组件类型与几何

E3D 桥架目录（Cablofil / Vantrunk / Legrand / Salamandre / Allied ARC / Oglaend 等厂商规范）覆盖以下元件类型（基于 E3D Brochure 与培训资料）：

| 组件类型 | 英文 | 几何特征 | GenRule 归属 |
|----------|------|----------|-------------|
| 直段 | Straight | U/C 截面沿直线路径扫掠 | LayerExtrusion（多层：本体+盖板）或 LinearComponent |
| 弯头 | Bend/Elbow | U 截面沿圆弧路径扫掠，含水平/垂直弯 | LinearComponent（弧线路径） |
| 三通 | Tee | 主路直段 + 支路直段拼接，含等径/变径 | Assembly（多 LinearComponent 拼装） |
| 十字 | Cross | 主路 + 双支路 | Assembly |
| 变径 | Reducer | 截面沿路径线性变化（宽变窄/高变低） | LinearComponent（变截面插值） |
| 立柱/竖弯 | Riser | 垂直方向过渡 | LinearComponent |
| 盖板 | Cover | 桥架盖，与本体平行 | LayerExtrusion 附加层 |

#### 3.1.3 桥架系统层级

```
CABLE TRAY (桥架管理容器)
└── BRANCH (分支)
    ├── Head (起始点)
    ├── COMPONENT: Straight (直段)        ← 固定长度或自动填充
    ├── COMPONENT: Bend (弯头)            ← 角度参数 (30°/45°/90°)
    ├── COMPONENT: Tee (三通)
    ├── COMPONENT: Reducer (变径)
    ├── COMPONENT: Cross (十字)
    ├── COMPONENT: Riser (立柱)
    └── Tail (终止点)
```

E3D 提供 **Quick Cable Tray Routing**：定义起止点后自动插入弯头并填充直段，类似管道 BRANCH 的自动布管逻辑。固定长度直段（Fixed-Length Trays）可替换隐含直段以保证设计一致性。

#### 3.1.4 关键电气参数

| 参数 | 说明 |
|------|------|
| 宽度 × 高度 (W × H) | 桥架截面尺寸，如 300×100, 600×150 |
| 填充率 (Fill Factor) | 电缆截面积占桥架有效面积百分比，用于自动路由规则 |
| 材质 | 镀锌钢/不锈钢/铝合金/玻璃钢 (FRP) |
| 类型 | 梯式 (Ladder) / 槽式 (Trough/Solid bottom) / 网格式 (Wire mesh) |
| 盖板 | 有盖/无盖 |
| 荷载等级 | 安全工作荷载 (SWL) |

### 3.2 导管 Conduit

#### 3.2.1 几何与逻辑

导管（Conduit）是单截面线性构件，用于保护穿管电缆。E3D 中导管规范（Conduit Specs）涵盖 Allied GRC、Allied ARC 等厂商。导管系统结构与桥架类似：BRANCH 下串联直段、弯头、接头，但截面为**封闭圆形**而非 U/C 形。

```
Conduit (BRANCH)
├── COMPONENT: Straight (直管段)    ← Cylinder 沿路径
├── COMPONENT: Elbow (弯头)         ← 弧形 Cylinder
├── COMPONENT: Coupling (接头)      ← 短 Cylinder
├── COMPONENT: Connector (连接器)
├── COMPONENT: Gland (格兰)
└── COMPONENT: Box (接线盒/拉线盒)
```

#### 3.2.2 GenRule 归属

导管为**单截面沿中心线扫掠**的典型线性构件，归 **LinearComponent**（固定截面模式）。与桥架（U/C 开口截面、可能多层）不同，导管是单一封闭截面，不需要 LayerExtrusion 的多层能力。

#### 3.2.3 关键参数

| 参数 | 说明 |
|------|------|
| 公称直径 (DN/Trade Size) | 1/2", 3/4", 1", 2" 等 |
| 材质 | 镀锌钢管 (GRC)、不锈钢、PVC、EMT、铝合金 |
| 弯曲半径 | 最小弯曲半径（电缆类型相关） |
| 防护等级 | IP / NEMA 等级 |
| 填充率 | 单根电缆/多根电缆填充率限制（NEC/IEC 标准） |

### 3.3 盘柜 Panel

#### 3.3.1 逻辑表示

盘柜在 AVEVA Electrical 中作为 **Distribution Board** 或 **Panel** 管理，是负荷分配与保护的核心节点。每个盘柜有电气标识号（如 `M1`~`M99` 表示 LV MCC/开关柜，`H1`~`H99` 表示 HV 开关柜），并管理若干出线回路 (Power Circuit, PC)。

盘柜电气属性（Panel Schedule）包含：主断路器额定值/类型、各支路断路器额定值/类型、相别、负荷、馈线电缆截面、备用回路数等。

#### 3.3.2 物理表示（柜体 + 母线 + 断路器 装配）

```
Panel (EQUI, Assembly)
├── SUBE: Enclosure (柜体)          ← Box  — 主体外壳
│   参数: H × W × D, MATERIAL, IP, Mounting (Surface/Flush)
├── SUBE: Door (门板)               ← Box  — 含观察窗/锁
├── SUBE: Main Breaker (主断路器)   ← Box + Cylinder (操作手柄)
├── SUBE: Busbar (母线)             ← Box (长条形) × N  — 三相+中性线
├── SUBE: Branch Breakers (支路断路器) ← Box × N
│   ├── MCB / MCCB / ELCB / RCCB
│   └── 排列于导轨上
├── SUBE: DIN Rail (导轨)           ← Box × N
├── SUBE: Terminal Block (端子排)   ← Box × N
├── SUBE: Cable Entry (电缆进线)    ← Cylinder × N
└── SUBE: Internal Wiring (内部接线) ← 逻辑数据，几何上常省略
```

#### 3.3.3 GenRule 归属与开洞

盘柜本体是 **Assembly**（多子件组合）。盘柜面板上的元件开孔（断路器窗口、操作手柄孔、电缆格兰孔）通过 **BooleanSubtract**（负实体）从面板 Box 中挖出，对应 E3D 的 Negative Primitive 机制（参见 `aveva_e3d_primitives.md` 3.3 节）。

#### 3.3.4 关键参数

| 类别 | 参数 |
|------|------|
| 电气 | 额定电压 (230/400, 110/220...), 主断路器额定电流, 频率 (50/60Hz), 相数 |
| 回路 | 回路数 (ways), 各回路断路器额定/类型, 备用回路数, 空位 |
| 负荷 | 各相连接负荷, 总连接负荷, 需用系数, 相平衡度 |
| 物理 | 尺寸 H×W×D, 安装方式 (Surface/Recessed/Semi-flush), 馈线进线方式 (Top/Bottom) |
| 防护 | IP 等级, NEMA 等级, 防爆 |

### 3.4 马达控制中心 MotorControlCenter (MCC)

#### 3.4.1 逻辑表示

MCC 是集中控制多台电机的盘柜组合，在 AVEVA Electrical 中作为带多个出线回路（每个回路控制一台电机）的特殊 Distribution Board。MCC 的电气标识号遵循 `M1`~`M99` 规则，每个电机回路编号为 `M101`、`M102`...（前缀 M + 设备号 + 顺序号）。

MCC 内部包含三类接线（IEC 61439 / IEC 60947）：
- **动力接线 (Power Wiring)**：主断路器 → 母线 → 各馈线 → 电机
- **控制接线 (Control Wiring)**：起停指令、联锁、报警、远程控制
- **辅助接线 (Auxiliary Wiring)**：指示灯、仪表、信号

#### 3.4.2 物理表示（柜组 + 隔室 + 单元 装配）

MCC 由多个并排的**柜体 (Cubicle)** 组成，每个柜体内分**隔室 (Compartment)**，每个隔室装一个**单元 (Unit)** 控制一台电机：

```
MotorControlCenter (EQUI, Assembly)
├── SUBE: Cubicle A (柜 A)
│   ├── SUBE: Enclosure (柜体)        ← Box
│   ├── SUBE: Incoming Compartment    ← Assembly (进线隔室)
│   │   └── Main Breaker + Busbar
│   ├── SUBE: Unit Compartment 1      ← Assembly (电机控制单元)
│   │   ├── Breaker (断路器)
│   │   ├── Contactor (接触器)
│   │   ├── Overload Relay (热继电器)
│   │   ├── Control Transformer (控制变压器)
│   │   └── Terminal Block (端子排)
│   ├── SUBE: Unit Compartment 2
│   └── ...
├── SUBE: Cubicle B (柜 B)
│   └── ...
├── SUBE: Busbar System (母线系统)    ← Box × N (贯穿各柜)
└── SUBE: Cable Alley (电缆通道)      ← Box (柜后/柜底电缆槽)
```

#### 3.4.3 关键参数

| 类别 | 参数 |
|------|------|
| 系统 | 额定电压, 短路电流, 母线额定电流, 防护等级 |
| 结构 | 柜数, 每柜单元数, 隔室形式 (Form 1~4 per IEC 61439) |
| 单元 | 电机功率, 接触器规格, 热继电器整定范围, 启动方式 (DOL/Star-Delta/VFD/Soft Starter) |
| 接线 | 控制电压, 联锁逻辑, 远程/就地模式 |
| 物理 | 总尺寸, 柜宽, 前后维护类型 |

### 3.5 电缆 Cable

#### 3.5.1 逻辑表示

电缆在 AVEVA Instrumentation/Electrical 的 Wiring Manager 中以**数据对象**形式存在，描述端到端连接：

```
Cable Object
├── Cable Tag (电缆编号)
├── From Equipment / Terminal (起点设备/端子)
├── To Equipment / Terminal (终点设备/端子)
├── Cable Type (电缆型号)
│   ├── 按用途: Signal / Power / Control / Communication / Wire
│   └── 按芯数: Single Core / Multicore
├── Core Count (芯数)
├── Cross Section (截面 mm² / AWG)
├── Voltage Grade (电压等级)
├── Length (长度, 由 E3D 回传)
└── Routing Status (是否已布线至桥架)
```

电缆编号规则示例（基于电力行业惯例）：`1101` = `M1`(设备号) + `01`(回路号)；多芯并缆加后缀 `/1`, `/2`；单芯加相别 `/L1-1`。

#### 3.5.2 物理表示

物理侧（E3D）中，电缆是**沿中心线扫掠的线构件**，路径由桥架/导管系统约束：

```
Cable (LinearComponent)
├── Centerline (中心线路径)
│   └── 沿桥架/导管/直接敷设的路径序列
├── Cross Section (截面)
│   ├── Single Core: 单圆截面 (Cylinder 沿路径)
│   └── Multicore: 多圆截面组合 (导体 + 屏蔽 + 护套)
└── Layered Structure (多层结构)
    ├── Conductor (导体) — 铜芯/铝芯
    ├── Insulation (绝缘层)
    ├── Screen/Shield (屏蔽层) — 可选
    ├── Armor (铠装) — 可选
    └── Outer Sheath (外护套)
```

E3D 的自动电缆路由（Automatic Cable Routing）按预配置规则将电缆路由进桥架：桥架填充率、信号/动力电缆分隔、路径选择。电缆定义从 AVEVA Instrumentation/Electrical 导入，路由后长度回传生成完整 Cable Schedule。

#### 3.5.3 GenRule 归属

电缆为典型 **LinearComponent**（截面沿中心线扫掠）。多芯电缆的多层结构（导体+绝缘+屏蔽+护套）也契合 LayerExtrusion 的多层概念，但因其为线构件，应作为 LinearComponent 的多层截面变体处理（OmniM `linear_component.rs` 已支持 `CrossSectionAtStation` 多层截面插值）。

---

## 4. 逻辑-物理双视图架构 (Logical-Physical Dual View)

### 4.1 回路 Loop

#### 4.1.1 回路定义

回路（Loop）是仪表工程的核心逻辑单元，由**共享同一回路号**的所有仪表组成：从传感器（测量变量）→ 变送器 → 控制器 → 最终控制元件（调节阀）。例如回路 `101` 可能包含 `FE-101`（孔板）、`FT-101`（流量变送器）、`FIC-101`（流量控制器）、`FCV-101`（流量调节阀）。

回路号是回路内所有仪表的**逻辑聚合键**。同一回路的仪表可能物理上分布在工厂的不同区域：传感器与调节阀在现场，控制器在控制室，JB 在中间走线。

#### 4.1.2 回路图 Loop Diagram

回路图（Loop Diagram / Loop Sheet）是回路的全息文档，记录从现场仪表到控制室 I/O 卡件的完整连接：
- 现场仪表端子 → JB 端子 → 控制室机柜端子 → DCS/PLC I/O 通道
- 信号类型、供电、接地、本安栅
- 电缆编号、芯线颜色、端子号

AVEVA Instrumentation 的 Designer 模块自动从 Wiring Manager 的数据生成回路图，使用宏 (Macro) 控制布局。

#### 4.1.3 几何属性

**回路本身无独立几何**，它是逻辑关联关系。在物理视图中，回路通过其成员仪表的 Tag 与物理放置的设备关联。OmniM 中应将 Loop 实现为**逻辑关联对象**（无几何，仅持有成员引用列表），而非几何对象。

### 4.2 工艺管线 ProcessLine

#### 4.2.1 逻辑表示

工艺管线（Process Line）在 P&ID 上以线段表示，编码信息密集：
```
Line Number 格式示例: 12"-3A1-CS-SCH40-HT1-INS
                       │   │   │   │    │    │
                       │   │   │   │    │    └─ 保温 (Insulation)
                       │   │   │   │    └─ 热伴 (Heat Tracing)
                       │   │   │   └─ 壁厚等级 (Schedule)
                       │   │   └─ 材质 (Carbon Steel)
                       │   └─ 管道等级 (Spec: 3A1)
                       └─ 公称直径 (DN300)
```

#### 4.2.2 物理表示

工艺管线在 E3D 中是 PIPE 容器下的 BRANCH 结构（参见 `aveva_e3d_primitives.md` 4.1 节），由直管段 + 弯头 + 阀门 + 法兰等元件序列组成。物理管线与逻辑管线通过 Line Number 关联。

#### 4.2.3 GenRule 归属

工艺管线作为**截面沿中心线扫掠的线构件**，归 **LinearComponent**（元件序列模式）。OmniM 当前 `linear_component.rs` 已支持多层截面插值，但"元件序列"模式（E3D BRANCH 风格的元件串联）需在 `LinearComponentInput` 中增加 `ComponentSequenceItem`（参见 `genrule_design_analysis.md` 2.1 节建议）。

### 4.3 标签 Tag 与位号体系

#### 4.3.1 Tag 在双视图中的贯穿

Tag 是贯穿逻辑与物理视图的**唯一标识符**：
- **逻辑视图**：P&ID 上的仪表符号、回路图、仪表索引、数据表中均用 Tag 标识
- **物理视图**：E3D 中放置的设备元素的 NAME 或 UDA (User Defined Attribute) 持有 Tag
- **交叉引用**：SCHEMATIC 数据库与 MODEL 数据库通过 Tag + Loop 进行交叉引用

#### 4.3.2 位号体系层次

```
Plant (工厂)
└── Unit/Area (单元/区域)        ← 可选数字前缀 (如 12-FT-101)
    └── Loop (回路)              ← 回路号 (如 101)
        └── Instrument (仪表)    ← 功能字母 + 后缀 (如 FT-101A)
            └── Sub-Components    ← 阀体/执行机构/定位器等
```

#### 4.3.3 与 P&ID 的集成

AVEVA P&ID 是逻辑视图的图形入口。P&ID 中绘制的每个仪表符号、管线、阀门都关联到工程数据库（ENGINEERING DB）。E3D 通过 SCHEMATIC DB 的交叉引用，从 P&ID 数据驱动 3D 设计：可基于 P&ID 创建 3D 管道，验证 3D 设计与 P&ID 的一致性。

**OmniM 启示**：物理对象必须支持携带逻辑属性（Tag, Loop, P&ID Ref, Line Number）作为 UDA，且这些属性可用于跨视图的双向追溯查询。

---

## 5. 其他工厂组件 (Other Plant Components)

### 5.1 过滤器 Filter

#### 5.1.1 逻辑表示

过滤器（Filter / Strainer）属于工艺设备，在 P&ID 上以特定符号表示（临时/永久、Y 型/T 型/篮式）。逻辑上属于 PIPE 系统的一个 COMPONENT，关联到工艺管线 (ProcessLine)。

#### 5.1.2 物理表示

过滤器是参数化设备，几何上由**基本体组合**构成：

```
Filter (EQUI, Assembly)
├── SUBE: Body (壳体)              ← Cylinder/Box  — 主过滤室
├── SUBE: Inlet/Outlet Nozzle       ← Nozzle × 2  — 进出口接管
├── SUBE: Cover (盖板)              ← Box/DISH  — 便于清理滤芯
├── SUBE: Filter Element (滤芯)     ← Cylinder  — 内部过滤元件
├── SUBE: Drain (排净口)            ← Nozzle
└── SUBE: Vent (放空口)             ← Nozzle
```

复杂过滤器（如多滤芯并联、自清洗过滤器）由更多子件组合，仍为 Assembly。简单 Y 型过滤器可简化为 **Primitive**（基本体拼装）。

### 5.2 支架 Support

#### 5.2.1 类型

E3D 支吊架系统涵盖管道支吊架与设备支座，支持多家厂商规范目录（基于 E3D Brochure）：

| 厂商 | 类型 |
|------|------|
| Lisega | 弹簧支吊架、恒力支吊架 |
| Pipe Supports Limited | 管架 |
| Carpenter & Patterson | 弹簧/恒力 |
| Anvil (Grinnell) | 管架配件 |
| OBO Bettermann | 电缆桥架支撑系统 |

E3D 支吊架目录规模：24 Specs, 331,407 Components, 172 Shapes, 74 Fittings，600 Support Frameworks, 1,517 Support Hangers。

#### 5.2.2 几何表示

支架是**标准件装配体**，由目录中预定义的子件拼装：

```
PipeSupport (EQUI, Assembly)
├── SUBE: Attachment to Structure    ← Box  — 与结构连接件
├── SUBE: Hanger Rod (吊杆)          ← Cylinder  — 线性构件
├── SUBE: Spring Can (弹簧罐)        ← Cylinder + 内部弹簧
├── SUBE: Clevis / U-bolt (U 型卡)   ← 弯曲 Cylinder
├── SUBE: Shoe (管托)                ← Box
├── SUBE: Guide (导向)               ← Box + 滑动面
├── SUBE: Anchor (固定支架)          ← Box  — 限制所有自由度
└── SUBE: Baseplate (底板)           ← Box
```

E3D 提供 **Automatic Drawing Production** 自动生成支吊架详图。OBO Bettermann 的 E3D 集成提供 `OBO Support Creator` 与 `OBO Single Bracket Creator` 应用，可在 E3D 标准界面内创建支撑构造。

#### 5.2.3 GenRule 归属

支架归 **Assembly**（标准件目录拼装）。其中个别子件（吊杆、管托底板）可作为 Primitive 或 LinearComponent（吊杆为圆柱线构件）的特例。支架子件通过 Catalog 驱动（参见 `genrule_design_analysis.md` 1.2 节"Primitive 由 Catalog 驱动"）。

---

## 6. 对 OmniM 的启示 (GenRule 映射表)

### 6.1 完整映射表

| 组件 | 几何主类 (GenRuleHint) | 子组件策略 | 逻辑附加属性 |
|------|----------|------------|----------|
| **Instrument (通用)** | Assembly | Primitive 组合 (壳体 + 接管 + 显示头) | Tag, Loop, P&ID Ref, Instrument Type, Signal Type |
| **Sensor (TE/FE/LE/PE)** | Primitive | 单/少基本体 (Cylinder + Nozzle) | Tag, Loop, Process Variable, P&ID Ref |
| **Sensor (流量计)** | Assembly | 测量管段 + 变送器外壳 | Tag, Loop, Flow Range |
| **Transmitter** | Assembly | 壳体 + 显示头 + 过程接口 + 电缆格兰 + 端子排 | Tag, Loop, Signal Type (4-20mA/HART/FF), Range |
| **ControlValve** | Assembly | 阀体 + 执行机构 + 定位器 + 附件 (电磁阀/限位开关) | Tag, Loop, Fail Action, Cv, Actuator Type |
| **JunctionBox** | Assembly | 外壳 + 端子排 + 格兰 + 导轨 | Tag, Terminal Count, Ex Rating, IP |
| **CableTray 直段** | LayerExtrusion | U/C 多层截面 (本体+盖板) 沿直线路径 | Width, Height, Fill Factor, Spec |
| **CableTray 弯头** | LinearComponent | U 截面沿弧线路径 | Bend Angle, Radius |
| **CableTray 三通/十字** | Assembly | 多 LinearComponent 拼装 | Branch Sizes |
| **CableTray 变径** | LinearComponent | 变截面插值 | From W×H, To W×H |
| **Conduit** | LinearComponent | 圆形封闭截面沿中心线 | DN, Material, Fill Factor |
| **Panel** | Assembly | 柜体 + 母线 + 断路器 + 端子排 | Panel Tag, Voltage, Main Breaker, Ways |
| **Panel 面板开孔** | BooleanSubtract | 面板 Box 减去元件孔洞 | — |
| **MCC** | Assembly | 多柜体 + 隔室 + 单元 (断路器+接触器+热继) | MCC Tag, Voltage, Short Circuit, Form Type |
| **Cable (单芯)** | LinearComponent | 圆截面沿中心线 | Cable Tag, From, To, Core Count, Section |
| **Cable (多芯)** | LinearComponent (多层截面) | 多圆截面 + 屏蔽 + 护套 沿中心线 | + Voltage Grade, Armor Type |
| **Loop (回路)** | 无几何 (逻辑对象) | — | Loop Number, Member Tags, P&ID Ref |
| **ProcessLine** | LinearComponent (元件序列模式) | 直管段 + 弯头 + 阀门序列 | Line Number, DN, Spec, Material, Insulation |
| **Tag** | 无几何 (属性) | — | Tag String, ISA Letters, Loop, Location |
| **Filter** | Assembly (复杂) / Primitive (简单) | 壳体 + 接管 + 滤芯 + 盖板 | Tag, Line Number, Filter Type |
| **Support** | Assembly | 目录标准件拼装 (吊杆 + 弹簧罐 + U 卡 + 管托) | Support Type, Load, Vendor |

### 6.2 与 OmniM 现有 ObjectType → GenRuleHint 映射的对齐

参照 `genrule_design_analysis.md` 1.3 节现有映射（`Tank/Pump/Valve → Primitive`, `Ahu/Chiller/Reactor → Assembly`），本调研建议扩展：

| 建议新增 ObjectType | GenRuleHint | 依据 |
|---------------------|-------------|------|
| Instrument / Transmitter / JunctionBox / Panel / MCC / Support / ControlValve | Assembly | 多子件装配（壳体+内部件+接口） |
| Sensor (简单) | Primitive | 单/少基本体 |
| Cable / Conduit / ProcessLine | LinearComponent | 截面沿中心线扫掠 |
| CableTray 直段 | LayerExtrusion | 多层 U/C 截面沿路径 |
| CableTray 弯头/变径 | LinearComponent | 弧线/变截面扫掠 |
| CableTray 三通/十字 | Assembly | 多线构件拼装 |
| Loop / Tag | (无 GenRule，逻辑对象) | 仅逻辑关联，无几何 |

**注**：当前代码中 `Valve → Primitive`（`genrule_design_analysis.md` 1.3 节），但调节阀因含执行机构/定位器，应细分为 `ManualValve → Primitive` 与 `ControlValve → Assembly`。

---

## 7. 关键发现与建议

### 7.1 核心发现

1. **逻辑-物理双视图是 AVEVA 体系的根特征**：SCHEMATIC/ENGINEERING 数据库（逻辑）与 MODEL 数据库（物理）分离，通过 Tag/Loop/Line Number 交叉引用。OmniM 必须在物理对象上支持逻辑属性挂载，并支持跨视图追溯查询。Loop 与 Tag 本身无几何，是纯逻辑关联对象。

2. **仪表/盘柜/MCC/调节阀 = Assembly of primitives**：变送器（壳体+显示+接口）、调节阀（阀体+执行机构+定位器）、JB（壳体+端子排+格兰）、Panel（柜体+母线+断路器）、MCC（多柜+多隔室+多单元）均为多子件装配体，对应 OmniM 的 Assembly GenRule 与 `assembly.rs` 引擎。

3. **电缆/导管/工艺管线 = LinearComponent**：单截面沿中心线扫掠的线构件。电缆多芯结构可借 `linear_component.rs` 已有的多层截面插值（`CrossSectionAtStation`）。工艺管线需"元件序列"模式（BRANCH 风格），这是当前代码缺失项（见 `genrule_design_analysis.md` 2.1 节）。

4. **桥架直段 = LayerExtrusion**：U/C 多层截面（本体+盖板+可选分隔层）沿路径拉伸，契合 `layer_extrusion.rs`。但桥架弯头/变径/三通分别归 LinearComponent（弧线/变截面）与 Assembly（拼装），需在 Catalog 模板中区分处理。

5. **盘柜面板开孔 = BooleanSubtract**：Panel/MCC 面板上的元件窗口、格兰孔通过负实体（E3D Negative Primitive）从面板挖出，直接对应 `boolean_subtract.rs`。这是 BooleanSubtract 在电气领域的典型应用，补充了现有 Door/Window（墙开洞）场景。

6. **Tag/Loop 贯穿双视图**：ISA-5.1 位号体系（`FT-101A` = 功能字母 + 回路号 + 后缀）是逻辑侧的主键，物理对象必须携带 Tag/Loop 作为 UDA。OmniM 的 `ConnectorDef` 已有 `connection_type_code` 与 `role` 字段（`genrule_design_analysis.md` 2.5 节），可承载此类逻辑编码。

7. **集成数据流是双向的**：逻辑侧（Instrumentation/Electrical）提供电缆定义→物理侧（E3D）路由并计算长度→回传逻辑侧生成 Cable Schedule。OmniM 若做电气模块，需设计双向同步机制，而非单向生成。

### 7.2 对 OmniM 的具体建议

#### 高优先级

1. **扩展 ObjectType 枚举**：增加 `Instrument`, `Transmitter`, `Sensor`, `ControlValve`, `JunctionBox`, `Panel`, `Mcc`, `Cable`, `Conduit`, `CableTray`, `CableTrayFitting`, `Loop`, `Support`, `Filter` 等电气/仪表域类型，并在 `primary_gen_rule()` 中补全映射（参照 6.2 节）。

2. **引入逻辑对象（无几何）**：Loop 与 Tag 不应通过 GenRule 生成几何，而应作为 `LogicalObject` 单独存储，持有成员物理对象引用列表。这是 OmniM 当前架构（以几何对象为中心）需要补充的维度。

3. **逻辑属性 UDA 机制**：所有物理对象支持挂载 `tag`, `loop_number`, `pid_reference`, `line_number`, `cable_from`, `cable_to` 等逻辑属性，用于跨视图追溯。可复用 E3D 的 LEXICON（UDA/UDET）思路。

#### 中优先级

4. **LinearComponent 元件序列模式**：在 `LinearComponentInput` 增加 `ComponentSequenceItem` 序列字段，支持 E3D BRANCH 风格的"直段+弯头+阀门"串联（工艺管线、桥架分支、导管分支）。这是 `genrule_design_analysis.md` 2.1 节已识别的缺口。

5. **Assembly 域便捷函数**：在 `assembly.rs` 增加 `build_transmitter()`, `build_control_valve()`, `build_panel()`, `build_mcc()`, `build_junction_box()` 等电气域便捷函数，与现有 `build_ahu()`, `build_pump()`, `build_reactor()` 并列。

6. **BooleanSubtract 电气场景**：`boolean_subtract.rs` 当前主要用于 Door/Window 墙开洞，应扩展 Panel 面板开孔、MCC 隔室分隔、JB 格兰孔等电气场景的测试与便捷函数。

#### 低优先级

7. **双向同步机制**：若 OmniM 发展为支持逻辑-物理双视图，需设计逻辑侧（电缆清册/回路图数据）与物理侧（3D 路由/长度）的双向同步协议，参考 AVEVA 的 Cable data 共享 + 长度回传模式。

8. **ISA-5.1 Tag 解析器**：实现 Tag 字符串（如 `FT-101A`）的解析与校验，支持首字母/后续字母/回路号/后缀的拆分，作为逻辑属性系统的输入校验。

### 7.3 核心结论

AVEVA 仪表/电气体系的本质是**"逻辑数据驱动 + 物理装配表达"的双层架构**：
- 逻辑层（Instrumentation/Electrical + SCHEMATIC/ENGINEERING DB）管理 Tag/Loop/Cable/Termination 等连接关系与工程数据，无几何；
- 物理层（E3D + MODEL DB）用**基本体拼装**（Assembly）表达仪表/盘柜/MCC/调节阀，用**截面扫掠**（LinearComponent/LayerExtrusion）表达电缆/导管/桥架/管线，用**布尔差**（BooleanSubtract）表达面板开孔。

这与 OmniM 的 GenRule 五类完全对应：Assembly 承担电气/仪表设备的主体表达，LinearComponent 承担线缆/导管/管线，LayerExtrusion 承担多层桥架，BooleanSubtract 承担盘柜开孔，Primitive 承担简单传感元件。OmniM 需补充的是**逻辑对象层**（Loop/Tag 无几何关联），这是当前以几何对象为中心的架构尚未覆盖的维度。
