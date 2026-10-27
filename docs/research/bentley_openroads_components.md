# Bentley OpenBridge Designer / OpenRoads / OpenTunnel Designer 内置构件逻辑研究

> 调研日期：2026-08-07
> 来源：Bentley 官方文档（docs.bentley.com LiveContent）、Bentley Communities 知识库、TxDOT/FDOT/FLH/Oregon DOT 公开培训手册、Virtuosity 产品资料
> 目标：分析 Bentley 基础设施产品线的内置构件底层几何逻辑，映射到 OmniM 的 5 类 GenRule（Primitive / LayerExtrusion / LinearComponent / Assembly / BooleanSubtract）
> 状态：完成

---

## 1. 平台概述

### 1.1 Bentley 土木/桥梁/隧道产品线分工

Bentley 面向基础设施的 BIM/CAD 由三个互补的 CONNECT Edition 产品构成，全部基于同一 **MicroStation PowerPlatform + Civil Framework (OpenRoads Technology)** 内核：

| 产品 | 定位 | 关键能力 |
|------|------|---------|
| **OpenRoads Designer (ORD)** | 道路与场地详细设计 | 几何线形、地形模型、走廊 (Corridor)、模板库、土方、排水、场地平整 |
| **OpenBridge Designer (OBD)** | 桥梁建模与分析（含 OBM + LEAP Bridge Concrete/Steel + RM Bridge） | 参数化桥梁上部/下部结构、BrIM 模型、与 LEAP/RM 互操作 |
| **OpenTunnel Designer (OTD)** | 隧道建模与设计 | 传统钻爆隧道 + TBM 盾构隧道、衬砌、支护、横通道、竖井、洞口 |

三产品共用：**Alignment（线形）+ Profile（纵断面）+ Corridor（走廊）+ Terrain（地形）+ Civil Cell（civil 单元）+ Feature Definition（特征定义）** 这一套数据模型，因此一座桥/隧洞可以参照引用同一条道路线形与地形，参数化联动更新。

### 1.2 四种通用建模范式

Bentley 在三产品中复用以下四类参数化机制，与 OmniM 的 GenRule 高度对应：

| 范式 | Bentley 实现 | 几何本质 | 对应 OmniM GenRule |
|------|------------|---------|------------------|
| **走廊模板 (Corridor Template)** | ORD/OBD/OTD 通用，模板库 `.itl`/`PierLib.xml`/Tunnel Catalog | 2D 横断面（点+组件+端部条件）沿 3D 路径扫掠 | LinearComponent |
| **线性模板 (Linear Template)** | 单模板沿单条特征线扫掠（无超高、无端部条件例外） | 单截面沿路径扫掠 | LinearComponent（固定截面） |
| **曲面模板 (Surface Template)** | 将模板投射到地形模型上，分层（路面结构层） | 多层 2D 截面在地形上铺装 | LayerExtrusion |
| **Civil Cell（civil 单元）** | 预封装的几何规则集合（交叉口、匝道、ADA 坡道、pond 控制线） | 几何 + 模板 + 地形 + 约束的复合体 | Assembly |
| **参数化单元 (Parametric Cell / Functional Component)** | MicroStation 约束 + 变量驱动的 3D 实体（自定义桥墩/桥台） | 单一参数化实体 | Primitive |
| **实体布尔 (Solid Modeling)** | MicroStation 实体建模工具（剪切、合并、trim） | 体素布尔 | BooleanSubtract |

### 1.3 Civil Model 数据容器

ORD/OBD/OTD 都通过 Project Explorer > Civil Model 管理同一套对象层级：

```
Civil Model
├── Linear Elements（线性特征：边线、中线、特征线）
├── Point Elements（点特征：检查井、桩位）
├── 3D Linear（由平面线形 + 纵断面组合的 3D 元素）
├── Terrain Models（地形模型：TIN）
├── Corridors（走廊：模板沿路径扫掠的主体）
├── Superelevation（超高）
├── Civil Objects（civil 对象：标注、命名边界等）
├── Civil Cells（封装的规则化复合构件）
└── Reference Models（参照模型：桥梁/隧道/管线互参照）
```

---

## 2. 桥梁上部结构 (Bridge Superstructure)

OpenBridge Modeler (OBM) 是 OBD 的建模核心。其上部结构以 **Supportline（支承线）+ Bridge Unit（桥跨单元）+ Beam Layout（布梁）+ Beam Definition（梁定义）+ Deck Template（桥面板模板）** 为骨架，全部参数化驱动。

### 2.1 主梁 Girder (I / U / Box)

#### 2.1.1 Beam Definition 对话框

OBM 通过 `Place Beam Group` 工具调出 Beam Definition 对话框，按 Bridge Unit 中的 Beam Layout 逐根定义主梁。截面来源分两大类：

| 桥型 | Beam Sections 选项 | 截面来源 | 是否可拼接 (Splice) |
|------|------------------|---------|------------------|
| **Beam Slab (P/S 或 RC 混凝土梁)** | LEAP Concrete | LEAP Bridge Concrete 标准预制截面库（AASHTO I~VI、Bulb-Tee、U-Beam、箱梁、T-Beam 等） | ❌ 不可拼接 |
| | Custom | OBM 自带预制截面库 | ✅ 可拼接 |
| **Beam Slab (钢梁)** | Rolled Shapes | Section Property Catalog 中 AISC 宽翼缘型钢 | ✅ 可拼接 |
| | Built-Up | 用户自定义板梁（腹板 + 翼缘 + 盖板） | ✅ 可拼接，支持变截面 |
| **Box / Tub Girder** | Place Tub Girders 工具 | 箱梁由 Web + Top Flange + Bottom Flange 表格分段定义 | ✅ 可拼接，支持变截面 |

#### 2.1.2 Beam Rows：分段与拼接

Beam Definition 表格中每一行代表梁沿跨长的一段：

- **单行**：整跨使用同一截面（prismatic，柱状）
- **多行**：通过 `Insert Row After` 增加行，每行指定 `Location Type`（SupportLine / Head / Tail / RatioBySpan）+ 相对位置 + 截面，实现拼接、材料变化或截面变化
- **Different End Template**：勾选后可指定段末截面，OBM 在段长内**线性放样 (loft)** 两截面间的实体；起末模板的点数与点起点位置必须一致

#### 2.1.3 Built-Up 钢板梁的变截面规则

Built-Up 钢梁由 Web / Top Flange / Bottom Flange / Cover Plate 四组子件独立定义，每件均有独立 `Variation` 方法：

| Variation 选项 | 几何含义 |
|---------------|---------|
| None | 等高/等宽（柱状） |
| Linear | 起值→末值线性变化 |
| Parabolic Up | 抛物线向上变化 |
| Parabolic Down | 抛物线向下变化 |
| Circular Up | 圆曲线向上变化 |
| Circular Down | 圆曲线向下变化 |

Cover Plate（盖板）不可沿跨长变宽，但可通过多行实现厚度/宽度跳跃（拼接）。

#### 2.1.4 Box / Tub Girder 几何参数

Place Box Beam 对话框中箱梁由 Web（腹板）与 Flanges（顶/底翼缘）独立定义：

- **Web**：Thickness、Start Height、End Height、Top Offset、Bottom Offset（控制腹板相对箱梁中心线的横向偏移，定义非对称箱室）、Variation（同上六种）
- **Top/Bottom Flange**：Width、Thickness、Offset（顶翼缘可不对称）、Variation
- **Haunch**：最小加腋高度（CL of beam 处测量），左右延伸、边缘类型（Chamfered / Normal），外腹梁可"Follow Deck Edge"自动延伸至桥面板边缘以模拟加厚桥面板边缘

#### 2.1.5 Beam Haunch（加腋）

加腋是从主梁顶到桥面板底的可变混凝土填充：

- **混凝土梁**：在梁起、末端（或支承线/跨中）输入加腋厚度；可"Compute"自动计算（输入最小间隙 + 残余预拱度）
- **钢梁**：从顶翼缘底到桥面板底测量；最小加腋 + 左右延伸 + 边缘倒角/圆角

#### 2.1.6 GenRule 映射

OBM 主梁本质为 **沿跨长（路径）扫掠截面** 的线构件，支持固定截面、变截面（六种 Variation）、分段拼接、端部模板放样。完全对应：

> **Girder → LinearComponent**
> - 路径 = Bridge Unit 沿线形 + Profile 的 3D 跨长
> - 截面 = LEAP/Custom/Rolled/Built-Up/Box 模板
> - VariationRule = None/Linear/Parabolic Up|Down/Circular Up|Down
> - Splice = 多个 `CrossSectionAtStation` 段
> - Different End Template = 段内两站间线性插值（loft）

### 2.2 桥面板 Deck

#### 2.2.1 Deck Template（桥面板模板）

`Utilities > Libraries > Decks` 管理 Deck Template 库，每个模板是桥面板的横断面（含行车道宽度、路缘、路拱、厚度、加厚边缘等约束变量，如 `RT_Width_Lane1`、`LT_Width_Lane1`）。模板可保存为 `.DGN` 文件，导入 RM Bridge 做结构分析。

`Place Deck` 工具选取起末支承线 → 添加 Variable Constraint → 应用模板沿桥跨生成实体。Deck 可"Follow Deck Edge"贴合任意线形/几何，支持变宽度 overhang。

#### 2.2.2 Deck Breakbacks

OBM 10.11+ 支持 Deck Breakbacks（桥面板断点），允许桥面板在跨内不连续，适应伸缩缝位置或斜桥分段。

#### 2.2.3 Haunch 与桥面板一体化

桥面板实体自动从主梁顶（或加腋顶）延伸到桥面板顶面，形成含加腋的复合实体。Haunch Material 在 10.11+ 单独可指定。

#### 2.2.4 GenRule 映射

桥面板是 **多层 2D 横断面沿桥跨路径扫掠** 的对象，但与单纯 LayerExtrusion 不同的是它沿 bridge alignment（曲线桥时为曲线路径）扫掠：

> **Deck → LinearComponent（多层截面）** — 曲线桥时
> **Deck → LayerExtrusion** — 直线桥且桥面板为平面板时（厚度方向多层）
>
> Deck Template 的约束变量（Lane 宽度、Overhang、厚度）映射到 OmniM 的 `CrossSectionLayer` + 表达式参数。

### 2.3 支座 Bearing

#### 2.3.1 Bearing 库与放置

OBM 内置 Bearing 库（`Utilities > Libraries > Bearings`，含中国专项轴承/支座 Cells），类型包括：

- Elastomeric Pad（弹性垫层支座）
- Pot Bearing（盆式支座）
- Disc Bearing（球形/圆盘支座）
- Mechanical Bearing（机械支座，固定/导向）

放置时选取支承线 → 指定 Transverse Offsets（多个偏移用分号分隔，支持一排多支座）→ 选取 Bearing 模板 → 关联到 Beam 的 Bearing Line。Bearing 几何由模板参数化定义（平面尺寸 + 高度 + 锚固件），通常为单一参数化实体。

#### 2.3.2 Bearing Seat（支座垫石）

OBM 自动生成 Bearing Seat（垫石/支承垫），10.11+ 支持按斜桥/倒角/圆角调整垫石几何，并支持 Stepped Cap（阶梯式帽梁）以适配不同支座高度。

#### 2.3.3 GenRule 映射

> **Bearing → Primitive** — 单一参数化实体（Box/Cylinder 组合 + 锚固孔）
> **Bearing → Assembly** — 复杂支座（含上下座板、锚栓、PTFE 板等多部件）时为子件组合

### 2.4 伸缩缝 ExpansionJoint

OBM 的 Expansion Joint 库（`Utilities > Libraries`）支持常见类型：

- Strip Seal（带状密封）
- Modular Joint（模数式伸缩缝，多向）
- Finger Plate（齿形板）
- Compression Seal（压缩密封）

伸缩缝通常放置在 Bridge Unit 的 breakback 位置或支承线处，由钢梁/型材 + 锚固件 + 密封件组成。几何由多个型材沿横向排列 + 锚固到桥面板构成。

#### GenRule 映射

> **ExpansionJoint → Assembly** — 多个型材子件 + 锚固子件 + 密封子件组合
> 简单单型材伸缩缝可降级为 Primitive。

---

## 3. 桥梁下部结构 (Bridge Substructure)

OBM 下部结构以 **Substructure Template（PierLib.xml 中的桥墩/桥台模板）+ Pier Config（桥墩配置）+ Functional Components（功能组件）** 为核心。

### 3.1 桥墩 Pier

#### 3.1.1 Pier Config 对话框

OBM 在 Substructure 模块 > Geometry 选项卡 > Pier Config 中定义桥墩，主要参数：

| 字段 | 选项 |
|------|------|
| Pier Type | Single Column / Multi Column / Wall / Hammer Head |
| Cap Shape | Tapered（变截面悬臂）/ Rectangle / Inverted T / Variable |
| Column Shape | Round / Rectangular / Custom（用户模板） |
| Footing | Spread Footing / Pile Cap / Drilled Shaft（含 Bell End） |

#### 3.1.2 Cap（帽梁）变截面

Cap 是横向（沿桥宽方向）的线构件：

- **Tapered**：两端变窄悬臂
- **Rectangle**：矩形等截面
- **Inverted T**：倒 T 形（可 follow crown slope 跟随桥面横坡）
- **Variable**：顶部保持水平，深度可 Linear / Parabolic / Circular 变化；10.11+ 增加倒角/圆角两种 edge modification

变截面 Cap 数据可传递到 RM Bridge / LEAP Bridge 做分析。

#### 3.1.3 Column（墩柱）堆叠

OBM 10.11+ 支持用户自定义 Column 模板，可 **"stack"（堆叠）** 不同模板截面组成复杂柱身（如圆→方→圆过渡），堆叠段可线性或抛物线变化。Drilled Shaft 支持 Bell End（扩底）。

#### 3.1.4 Footing（承台/基础）

- Isolated Footing / Pile Cap / Spread Footing
- 支持旋转角度（10.11+ 单独旋转单个承台）
- 支持斜承台（sloped footing）——适用于坡地或斜交桥
- User Defined Footing Templates（10.11+ 用户自定义承台模板）
- Piles：可从 H-Pile 标准列表选择，支持 batter（倾斜桩）以 ratio 格式定义，10.11+ 支持单独旋转桩

#### 3.1.5 Functional Component（参数化桥墩）

对于非标桥墩，OBM 提供 Functional Component 工作流（基于 MicroStation Parametric Modeling）：

1. 在 2D 草图中绘制构造线（垂直参考、水平参考、坡度线）
2. 应用 2D 约束（Fix、Coincident、Parallel、Distance、Angle 等）
3. 定义 Parameters（变量）
4. 建立 Intrinsic Variables（OBM 内置变量，如桥面坡度、支承线高程）与参数的关系
5. 用 3D 约束定义实体（Solid by Extrude/Sweep/Revolve）
6. 保存为 Parametric Cell → 在 OBM 中通过 Supportline 放置并自动适配桥面参数

#### 3.1.6 GenRule 映射

桥墩是典型的 **多子件组合**，每个子件几何本质不同：

> **Pier → Assembly**，子件 GenRule 如下：
> - Cap → LinearComponent（横向变截面线构件，Variation = Linear/Parabolic/Circular）
> - Column → LinearComponent（柱状/堆叠变截面）或 Primitive（单一柱状实体）
> - Footing → Primitive（Box 基本体 + 可选斜面）
> - Pile → Primitive（Cylinder，沿 batter 方向）
> - 非标 Functional Component → Primitive（参数化单元）

### 3.2 桥台 Abutment

#### 3.2.1 Abutment 组成

OBM 桥台由以下部件组合：

| 部件 | 几何参数 |
|------|---------|
| Stem（台身） | 厚度、高度、可分段 |
| Backwall（背墙） | 10.11+ 顶面可 follow deck slope 跟随桥面坡度 |
| Footing（承台） | 同 Pier Footing |
| Wingwall（翼墙） | 独立左右翼墙参数；模板库管理；可含 footing + piles；水平/垂直偏移裁剪顶/底端 → 形成 cantilever 或"elephant ear"翼墙；可按填土坡度自动计算翼墙长度 |
| Cheek Wall（侧墙） | 10.11+ 可倒角/圆角 |
| Beam Seat（梁座） | 同支座垫石 |
| Approach Slab（搭板） | 10.11+ 高程报告 |
| Longitudinal Offset（纵向偏移） | 桥台可沿桥纵轴向偏移 |

#### 3.2.2 Wingwall Template 库

翼墙模板库管理横断面，翼墙沿其自身路径（通常与桥轴线成角度）扫掠生成，可独立参数化。

#### 3.2.3 GenRule 映射

> **Abutment → Assembly**，子件 GenRule：
> - Stem → LinearComponent（截面沿台身高扫掠，可变截面）
> - Backwall → LinearComponent
> - Footing → Primitive（Box）
> - Wingwall → LinearComponent（翼墙模板沿翼墙路径扫掠，端部按填土坡裁剪 = VariationRule 与外部约束）
> - Cheek Wall → Primitive
> - Approach Slab → LayerExtrusion（多层板）或 LinearComponent

---

## 4. 隧道结构 (Tunnel Structure)

OpenTunnel Designer (OTD) 建立在 OpenRoads 技术之上，使用 **Tunnel Corridor + Tunnel Unit + Tunnel Template Catalog** 三层架构。模板库分类：Tunnels（Conventional / TBM）、Portal、Shafts、Rooms、Excavation、Reinforcement、Auxiliaries、Materials。

### 4.1 衬砌 Lining

#### 4.1.1 Conventional Tunnel Lining（传统钻爆隧道衬砌）

OTD 中传统隧道的衬砌：

- 从 Tunnel Templates Catalog 导入隧道横断面（含理论开挖线、衬砌内/外轮廓、支护层）
- 添加 Tunnel Corridor 沿 alignment → 默认创建一个 Tunnel Unit
- 在 Tunnel Unit 内指定 Start/End Reference Line → 衬砌截面沿 corridor 路径扫掠生成实体

**关键限制**：当前版本**不支持横断面之间的放样 (lofting)**。若沿隧道长度需变化截面，必须建模为多个 Tunnel Unit，每个 Unit 使用自己的横断面（相邻 Unit 可用不同截面）。这对应 OmniM LinearComponent 的"分段变截面"模式。

#### 4.1.2 Lining 的多层结构

衬砌横断面可包含多层：开挖轮廓（extrados）+ 初期支护（shotcrete）+ 二次衬砌（cast-in-place concrete）+ 内饰层。每层在模板中作为独立 component 定义，扫掠时各层独立生成实体。

#### 4.1.3 Construction Stages（施工阶段）

OTD 支持为衬砌和支护定义施工阶段，10.x+ Construction Scheduler 支持在已有阶段之间插入新阶段。每个阶段的衬砌实体可独立计算工程量。

#### 4.1.4 GenRule 映射

> **Lining → LinearComponent（多层截面）**
> - 路径 = Tunnel Corridor 沿 alignment 的 3D 路径
> - 截面 = Tunnel Template（多层 component：开挖/支护/衬砌）
> - 变截面 = 多个 Tunnel Unit 各自截面（当前无 loft，是分段变截面特例）
> - Construction Stage = 时序属性（非几何）

### 4.2 洞口 Portal

#### 4.2.1 Portal Template

OTD Portal Catalog 中每个洞口模板由两组闭合形状定义：

- **Closed Shape**：洞口仰坡/护面的闭合轮廓（正面投影）
- **Excavation**：洞口开挖体（向山体延伸的实体）

放置时选取隧道 alignment 的起/终点 + 地形模型 → 模板自动适配地形生成洞口实体（含仰坡、挡墙、截水沟等）。

#### 4.2.2 GenRule 映射

> **Portal → Assembly** — 多部件（仰坡面、挡墙、截水沟、开挖体）组合
> 简单单部件洞口可降级为 Primitive（参数化单元）。
> 洞口开挖体与地形的差集 → BooleanSubtract。

### 4.3 竖井 Shaft

#### 4.3.1 Shaft Template 参数

OTD Shaft Catalog 中每个竖井模板由四个核心参数定义：

- **Diameter**（直径，圆形竖井）
- **Thickness**（井壁厚度）
- **Length**（竖井深度）
- **Slab Thickness**（底板厚度）

竖井本质是 **沿垂直路径扫掠环形截面** 的线构件，底板为附加实体。

#### 4.3.2 GenRule 映射

> **Shaft → LinearComponent** — 环形截面沿垂直路径扫掠 + 底板 Primitive
> 或 **Shaft → Primitive**（Cylinder + Shell，简单等截面竖井）

### 4.4 管片 Segment (TBM UTR)

#### 4.4.1 TBM Universal Tapered Ring (UTR) 工作流

OTD 的 TBM UTR 是盾构隧道衬砌环的参数化建模工具：

1. 导入/创建/参照 alignment
2. 添加 Tunnel Corridor，类型选 TBM
3. 添加 Reference Line（环起/终点）
4. 从 Tunnel Templates Catalog 导入隧道横断面
5. 运行 Place TBM UTR 命令

#### 4.4.2 UTR 参数

| 参数 | 含义 |
|------|------|
| Tunnel Template | 用于 extrusion 的隧道横断面 |
| Start/End Station Offset | 相对 Reference Line 的偏移 |
| Ring Name Prefix / Ring Start Number | 环命名与编号 |
| Taper | 环的最大/最小宽度与平均宽度之差（锥度） |
| Number of Dowels | 连接相邻环管片的销钉数（用于计算销钉节距角 dowel pitch angle） |
| First Ring Position | 首环位置（1 = 按模板定义；其余按 dowel pitch angle 旋转重排） |
| Excluded Ring Positions | 不允许与首环组合的位置列表 |

每个 Ring 由若干 Segment（管片）组成，Segment 之间通过 dowel 连接，环间通过锥度方向调整实现曲线拟合（Universal Tapered Ring 的"通用环"特性）。

#### 4.4.3 环展开图与定位图

OTD 2025+ 自动生成 TBM UTR 的 Unfolded View Drawings（环展开图）与 Ring Position Drawings（环定位图）。

#### 4.4.4 GenRule 映射

> **Segment → Primitive** — 单个管片为参数化实体（带销钉孔、密封槽）
> **Ring → Assembly** — 由若干 Segment 子件 + Dowel 子件组合
> **TBM Lining 全长 → Assembly**（环序列）— 每环为子件，锥度方向 = VariationRule

### 4.5 横通道 CrossPassage

#### 4.5.1 横通道建模流程

OTD 2025+ 引入横通道专项工具：

1. **Create Transverse Alignments**（横向线形）——基于 OpenRoads 的同名工具，简化横通道线形创建
2. 在横通道 alignment 上添加独立 Tunnel Corridor（横通道截面通常小于主隧道）
3. **Cross Passage Intersection**——将主隧道实体与横通道实体合并（boolean union），形成连通体
4. **Tunnel Trimming**——将次隧道（横通道）实体在主隧道 extrados 处修剪；主隧道实体不被修改

#### 4.5.2 限制

早期版本无专用工具，需用 MicroStation 实体建模手动修改，且修改后若隧道实体更新则手动修改会被覆盖（2025+ 工具已解决此问题）。

#### 4.5.3 GenRule 映射

> **CrossPassage → Assembly**，含：
> - 横通道实体 → LinearComponent（小截面沿横通道 alignment 扫掠）
> - 与主隧道的连通 → BooleanSubtract（从主隧道开挖体减去横通道开挖体）/ BooleanUnion（合并衬砌）
> - Trim 到主隧道 extrados → BooleanSubtract

---

## 5. 水工结构 (Hydraulic Structures)

Bentley 水工/排水由 **OpenRoads Designer 的 Drainage & Utilities 工作流 + OpenFlows StormCAD/CivilStorm/SewerGEMS** 协同完成；大型水工（渠道/调蓄池/坝）多用 ORD 的 Linear Template + Surface Template + Terrain 实现。

### 5.1 渠道 Channel

#### 5.1.1 实现方式

渠道在 ORD 中典型建模：

- **Corridor Template**：定义渠道横断面（梯形/矩形/U 形，含边坡、底宽、水深、超高）
- **Linear Template**：单模板沿渠道中心线扫掠（适用于等截面渠道）
- **End Condition**：渠道边坡顶部 daylight 到现有地形（同道路边坡）

渠道底纵坡由 Profile 控制，横坡由模板控制。

#### 5.1.2 GenRule 映射

> **Channel → LinearComponent** — 横断面沿渠道 alignment 扫掠
> 含多层（衬砌、护坡、底板）时 → LinearComponent（多层截面）

### 5.2 排水 Drainage

#### 5.2.1 Drainage & Utilities 工作流

ORD 内置排水管网建模，可与 OpenFlows StormCAD/CivilStorm/SewerGEMS 互操作：

- **Pipe Network**：节点（Manhole / Catch Basin / Inlet / Junction）+ 管段（Circular / Elliptical / Box Culvert）
- **Hydraulics & Hydrology**：稳态/非稳态、HEC-22 入口容量、SWMM 暴雨、1D/2D 地表洪水分析
- **结构**：从 2D 工件与外部数据建模 → 平面/纵断面/横断面出图 + BIM

#### 5.2.2 几何构成

```
Drainage Network
├── Pipes（管段）
│   ├── Shape: Circular / Elliptical / Box / Arch
│   ├── Start/End Invert Elevation
│   ├── Slope / Length / Diameter
│   └── Material / Manning's n
├── Structures（节点结构）
│   ├── Manhole（检查井）
│   ├── Catch Basin（雨水口）
│   ├── Junction Box（交汇井）
│   └── Headwall（端墙）
└── Rules（规则）
    ├── Min Cover / Min Slope / Max Depth
    └── HEC-22 / SWMM 计算
```

#### 5.2.3 GenRule 映射

> **Drainage → Assembly**，子件：
> - Pipe → LinearComponent（截面沿管段路径扫掠）
> - Structure → Primitive（参数化井/口实体，含 Connectors 与管道对接）
> - 网络拓扑 = OmniM Series + Connector 关系系统

### 5.3 调蓄池 RetentionBasin

#### 5.3.1 Pond Design 工作流（FDOT/FHWA 标准）

ORD 中调蓄池/滞洪池的标准建模流程：

1. **创建 Pond Design File**
2. **创建 Pond Control Line**（池中心控制线，含平面线形 + 纵断面，定义池底/水位/顶高）
3. **Place Pond Design Linear Template** —— 沿 Pond Control Line 扫掠池断面模板（含内边坡、外边坡、底宽、超高、freeboard）
4. **Manipulate Pond Dimensions / Elevations** —— 调整池几何
5. **Create Pond Design Surface** —— 由 Linear Template 生成 Pond 设计地形（Design Terrain）
6. **Create Cut Fill Volumes** —— 与 Existing Terrain 计算挖填方
7. **Stormwater Facilities Sheet** —— 出图

Pond 本质是 **沿控制线扫掠断面模板 + 与地形布尔求挖填**。

#### 5.3.2 GenRule 映射

> **RetentionBasin → LinearComponent**（沿 Pond Control Line 扫掠池断面）
> + **BooleanSubtract**（与 Existing Terrain 求挖填体）

### 5.4 坝 Dam

#### 5.4.1 实现方式

ORD 中土坝/堆石坝典型建模：

- **Corridor Template**：坝体横断面（含上下游边坡、马道、坝顶宽度、防渗心墙/斜墙多层结构）
- **Dam Centerline Alignment + Profile**：沿河谷的坝轴线 + 坝顶纵断面
- **End Condition**：上下游坝脚 daylight 到河谷地形
- **变截面**：沿坝长不同桩号用不同 Template Drop（坝高随河谷变化）→ 多段 Template Drop 实现

混凝土坝（拱坝/重力坝）多用 Parametric Cell 或 MicroStation 实体建模。

#### 5.4.2 GenRule 映射

> **Dam → LinearComponent**（变截面线构件）
> - 路径 = 坝轴线
> - 截面 = 坝体横断面（多层：心墙/坝壳/护坡）
> - 变截面 = 多段 Template Drop（不同坝高）
> - 坝脚与地形相交 = End Condition / VariationRule（外部约束求解）

---

## 6. 土方与地形 (Earthwork & Terrain)

### 6.1 地形 Terrain

#### 6.1.1 Terrain Model 数据结构

ORD Terrain Model 是 **不规则三角网 (TIN)**：

```
Terrain Model
├── Points（点数据：x, y, z）
├── Break Lines（断裂线：三角形边不可跨越）
├── Boundary（外边界：约束三角网范围）
├── Void / Break Void（空洞/带高程空洞）
└── Triangles（三角形：由点与断裂线自动三角化）
```

#### 6.1.2 创建来源

- Survey 原始测量数据（_least squares / compass / crandall / transit 平差）
- ASCII / LandXML / 点云 / 等高线
- Corridor 输出（Create Terrain Model from Corridor）
- 平面（Plane Terrain，如指定高程的水平地形，用于"Terrain to Plane"体积计算）

#### 6.1.3 Feature Definition 的 Volume Option

每个 Terrain Feature Definition 有 `Volume Option` 属性，决定其在土方计算中的角色：

| Volume Option | 含义 |
|--------------|------|
| Existing | 现有地面（被减面） |
| Design | 设计面（被加面） |
| Subgrade | 路基面（独立计方） |
| None | 不参与体积计算 |

#### 6.1.4 GenRule 映射

地形不是单一实体几何，而是 **TIN 曲面**，不属于 5 类 GenRule 的任一类。OmniM 应将其作为独立 `TerrainModel` 对象类型，与 GenRule 并列。

> **Terrain → TerrainModel（独立类型，非 GenRule）**
> 在 GenRule 体系中作为外部约束目标（如 LinearComponent 的 End Condition 目标）。

### 6.2 挖方/填方 CutVolume / FillVolume

#### 6.2.1 Create Cut Fill Volumes 工具

`Terrain > Analysis > Volumes > Create Cut Fill Volumes`：

- 输入：Existing Surface（Existing Volume Option）+ Design Surface（Design/Subgrade Volume Option）
- 输出：3D mesh solid，带体积属性（cut volume / fill volume / surface area）
- 算法：从 Design Surface 向 Existing Surface 垂直拉伸边界，形成闭合实体并计算体积（master units 立方）
- 设计面可不连续 → 所有片合并为一个 mesh solid

#### 6.2.2 三种体积计算方法对比

| 方法 | 工具 | 是否考虑 clipping boundary | 是否含 cut/fill | 适用场景 |
|------|------|--------------------------|---------------|---------|
| Element Component Quantities | Analysis > Element Component Quantities | ✅ | ❌ | 走廊组件工程量（不含土方） |
| Corridor Component Quantities | Corridor Modeling > Component Quantities | ❌ | ✅ | 快速估算（不裁剪） |
| End Area Volumes | Corridor Modeling > End Area Volume | ✅ | ✅ | **推荐**，按横断面平均端面积法 |
| Analyze Volumes (Terrain to Terrain) | Civil Analysis > Analyze Volume | N/A | ✅ | 三角棱柱体积法，无桩号 |
| Analyze Volumes (Terrain to Plane) | Civil Analysis > Analyze Volume | N/A | ✅ | 洪水位/堆料体积 |
| Quantities Report by Named Boundary | Drawing Production > Named Boundary | ✅ | ✅ | 按命名边界出报告，含曲率修正 |

#### 6.2.3 GenRule 映射

> **CutVolume / FillVolume → BooleanSubtract**
> - Cut = Existing Terrain − Design Surface（差集）
> - Fill = Design Surface − Existing Terrain（差集）
> - 几何本质为两 TIN 曲面之间的布尔运算，生成 3D mesh solid

### 6.3 边坡 Slope

#### 6.3.1 End Condition（端部条件）

ORD 模板中的 End Condition 是边坡/Daylight 子装配：

- 从路面边缘向外延伸边坡（如 1V:6H 填方、1V:1H 挖方）
- **Target**：Active Terrain Model 或指定命名地形
- **Daylight Point**：边坡与地形相交点（求解器自动求交）
- **End Condition Exception**：在特定桩号区间覆盖默认边坡（如挡墙段、桥台段、特殊地质段）
- **Priority**：多组端部条件可设优先级，按序尝试直到成功 daylight

#### 6.3.2 Slope 作为独立构件

在场地平整中，边坡也可作为独立的 Linear Template（沿场地边界线扫掠边坡断面），或通过 Surface Template 在地形上铺护坡层。

#### 6.3.3 GenRule 映射

> **Slope → LinearComponent**（走廊模板的 End Condition 组件）
> - 截面 = 边坡断面（坡比、马道、支护）
> - 路径 = 道路/场地边缘线
> - VariationRule = End Condition Exception（区间变截面）
> - 外部约束 = Terrain Target（daylight 求交）

---

## 7. 对 OmniM 的启示（GenRule 映射表）

| 工程类别 | 构件 | Bentley 平台/方法 | GenRule 映射 | 关键映射理由 |
|--------|------|----------------|------------|-----------|
| 桥梁上部 | Girder (I/U/Box) | OBM Beam Definition（LEAP/Custom/Rolled/Built-Up/Box），Beam Rows + Different End Template | **LinearComponent** | 沿跨长扫掠截面；6 种 Variation（None/Linear/Parabolic±/Circular±）；多行 Splice = 分段变截面；Different End Template = 段内 loft |
| 桥梁上部 | Deck | OBM Deck Template（横断面 + 约束变量）沿桥跨放置 | **LinearComponent**（多层截面）/ LayerExtrusion（直线板） | 多层 2D 截面沿桥跨路径扫掠；曲线桥时路径为曲线 |
| 桥梁上部 | Bearing | OBM Bearing 库（Elastomeric/Pot/Disc/Mechanical） | **Primitive** / Assembly | 单一参数化实体；复杂多部件支座为 Assembly |
| 桥梁上部 | ExpansionJoint | OBM Expansion Joint 库（Strip Seal/Modular/Finger/Compression） | **Assembly** | 多型材 + 锚固 + 密封件组合 |
| 桥梁下部 | Pier | OBM Substructure Template + Pier Config（Cap/Column/Footing）+ Functional Component | **Assembly** | Cap=LinearComponent（变截面）；Column=LinearComponent/Primitive；Footing=Primitive；Pile=Primitive；非标=Primitive(参数化单元) |
| 桥梁下部 | Abutment | OBM Abutment Template（Stem/Backwall/Footing/Wingwall/CheekWall/ApproachSlab） | **Assembly** | Stem/Backwall/Wingwall=LinearComponent；Footing=Primitive；ApproachSlab=LayerExtrusion |
| 隧道 | Lining | OTD Tunnel Template（多层 component）沿 Tunnel Corridor 扫掠 | **LinearComponent**（多层截面） | 多层（开挖/支护/衬砌）截面沿 alignment 扫掠；变截面=多 Tunnel Unit 分段 |
| 隧道 | Portal | OTD Portal Catalog（Closed Shape + Excavation） | **Assembly** / Primitive | 多部件（仰坡/挡墙/截水沟/开挖体）；开挖体与地形=BooleanSubtract |
| 隧道 | Shaft | OTD Shaft Catalog（Diameter/Thickness/Length/Slab） | **LinearComponent** / Primitive | 环形截面沿垂直路径扫掠 + 底板 |
| 隧道 | Segment (TBM UTR) | OTD TBM UTR（Taper/Dowels/First Ring Position） | **Primitive**（单管片） | 参数化实体（带销钉孔/密封槽） |
| 隧道 | Ring (TBM) | OTD TBM UTR Ring | **Assembly** | 多 Segment + Dowel 组合 |
| 隧道 | CrossPassage | OTD Transverse Alignment + Cross Passage Intersection + Tunnel Trimming | **Assembly** + BooleanSubtract | 横通道=LinearComponent；与主隧道连通=BooleanSubtract/Union |
| 水工 | Channel | ORD Corridor/Linear Template（梯形/矩形/U 形） | **LinearComponent** | 横断面沿渠道 alignment 扫掠；End Condition daylight 到地形 |
| 水工 | Drainage | ORD Drainage & Utilities + OpenFlows | **Assembly** | Pipe=LinearComponent；Structure=Primitive；网络拓扑=Series+Connector |
| 水工 | RetentionBasin | ORD Pond Control Line + Linear Template + Pond Surface + Cut Fill | **LinearComponent** + BooleanSubtract | 池断面沿控制线扫掠；与地形求挖填 |
| 水工 | Dam | ORD Corridor Template（多段 Template Drop，含心墙多层） | **LinearComponent** | 坝体断面沿坝轴线扫掠；变截面=多段 Template Drop；坝脚=End Condition |
| 土方 | Terrain | ORD Terrain Model（TIN：Points/Breaklines/Boundary/Void） | **TerrainModel（非 GenRule）** | TIN 曲面，作为外部约束目标，与 GenRule 并列 |
| 土方 | CutVolume/FillVolume | ORD Create Cut Fill Volumes（Existing vs Design） | **BooleanSubtract** | 两 TIN 曲面间布尔差，生成 3D mesh solid |
| 土方 | Slope | ORD Template End Condition（Daylight 到 Terrain） | **LinearComponent** | 边坡断面沿边缘线扫掠；End Condition Exception=VariationRule；Target=外部约束 |

### 7.1 跨平台 GenRule 验证结论

本次 Bentley 调研与既有 Civil 3D / Revit / E3D 调研交叉验证 5 类 GenRule 的完备性：

| GenRule | Bentley 对应 | 验证结论 |
|---------|------------|---------|
| Primitive | OBM Bearing / Functional Component / Shaft（简单） | ✅ 参数化单元需扩展 ParametricOperation（Cone/Sphere/Torus/Sweep/Revolve/Loft） |
| LayerExtrusion | ORD Surface Template / OBM Deck（直线桥）/ Approach Slab | ✅ 多层 2D 截面 + 厚度方向 |
| LinearComponent | OBM Girder/Cap/Stem/Wingwall；OTD Lining/Shaft/Segment；ORD Corridor/Channel/Dam/Slope | ✅ **核心 GenRule**，覆盖最广；变截面（VariationRule）支持 6 种插值方法 |
| Assembly | OBM Pier/Abutment/ExpansionJoint；OTD Ring/CrossPassage/Portal；ORD Drainage/RetentionBasin | ✅ 子件 + Transform 嵌套 |
| BooleanSubtract | ORD Cut/Fill Volumes；OTD Cross Passage Intersection/Trimming；Portal 开挖 | ✅ 两实体/曲面间布尔差 |

---

## 8. 关键发现与建议

### 8.1 LinearComponent 是基础设施领域的核心 GenRule

Bentley 三产品中 80% 以上的构件本质都是 **横截面沿路径扫掠** ：桥跨梁/帽梁/台身/翼墙/隧道衬砌/竖井/渠道/坝/边坡/管道。OmniM 的 `linear_component.rs` 已支持变截面插值（`CrossSectionAtStation` + `interpolate_cross_section_layer()`）和 Frenet 框架，方向正确。

**建议**：
1. **扩展 VariationRule 插值方法**：当前仅线性插值，应增加 Parabolic Up/Down、Circular Up/Down（OBM Built-Up 钢梁的 6 种 Variation 直接对应）
2. **支持分段变截面**：OBM 的 Beam Rows + Different End Template、OTD 的多 Tunnel Unit 都是"分段不同截面"模式，对应 OmniM `LinearComponentInput` 的 `ComponentSequenceItem`（genrule_design_analysis.md 中已建议但未实现）
3. **外部约束求解（Target Mapping）**：ORD End Condition 的 Daylight、OTD 衬砌到地质边界、Pond 到地形，都是"截面端点求交到外部曲面"——OmniM 需在 LinearComponent 中增加 `ExternalTarget` 约束

### 8.2 模板库（Template Library）是工程级配置的关键

Bentley 全部参数化构件都由模板库驱动：`.itl`（ORD Corridor Template）、`PierLib.xml`（OBM 桥墩/桥台）、Tunnel Catalog（OTD 隧道/portal/shaft/room）、Beam Section Template Library。模板 = 可复用的横断面定义 + 约束变量 + 命名规则。

**建议**：OmniM 的 `CatalogStore` + `ComponentDef` + `FeatureTreeTemplate` 三层架构应承载类似模板库能力，每个 ComponentDef 关联：
- 横断面定义（点表 + 约束表达式）
- 参数表（变量 + 默认值 + 范围）
- Feature Definition 元数据（symbology、level、命名规则）

### 8.3 Civil Cell = Assembly GenRule 的工程化封装

ORD Civil Cell 将"几何 + 模板 + 地形 + 约束规则"打包为可复用单元（交叉口、匝道、ADA 坡道、Pond），通过 Reference Elements 自动适配新位置。这正是 OmniM Assembly GenRule 应具备的能力：子件带 Transform + 表达式驱动的位置/方向 + 外部参照绑定。

**建议**：OmniM Assembly 应支持"参数化参照点"——子件位置可表达式引用外部对象（如桥轴线、支承线、地形点），实现 Civil Cell 式的"放置即适配"。

### 8.4 Functional Component / Parametric Cell 范式

OBM Functional Component 工作流（2D 约束 → 3D 实体 → 参数化单元 → 在桥梁上下文中放置）验证了 OmniM 的"Primitive + 参数化约束"路径。当前 OmniM `ParametricGeometry` + `ParametricOperation` 仅支持 Box/Cylinder，需扩展到 Cone/Sphere/Torus/Sweep/Revolve/Loft，并支持 2D/3D 约束求解。

### 8.5 地形/土方需独立于 5 类 GenRule

Bentley Terrain Model 是 TIN 曲面，不属于扫掠/放样/布尔中的任何一类；Cut/Fill Volume 是两曲面间布尔差（生成 mesh solid）。OmniM 应将 **TerrainModel 作为独立 ObjectType**（与 5 类 GenRule 并列），并新增 **BooleanSubtract 的曲面-曲面变体** 用于土方计算。

### 8.6 变截面"无 loft"是工程现实

OTD 明确说明当前版本不支持横断面之间的 lofting，只能用多 Tunnel Unit 分段（每个 Unit 内等截面）。这与 OmniM 当前 `LinearComponentInput` 的 `CrossSectionAtStation` + 段内插值实现一致。说明 **段内插值 + 段间跳跃** 是工程上更稳健的变截面策略，OmniM 应保持此设计而非追求全局光滑 loft。

### 8.7 需要新增/调整的 OmniM 设计项

| 项目 | 优先级 | 来源 | 说明 |
|------|--------|------|------|
| VariationRule 扩展 6 种插值 | 高 | OBM Built-Up Variation | Linear/Parabolic±/Circular± |
| ComponentSequenceItem（分段变截面） | 高 | OBM Beam Rows + OTD Tunnel Unit | LinearComponentInput 增加 |
| ExternalTarget 约束 | 高 | ORD End Condition Daylight | 截面端点求交到 Terrain/特征线 |
| TerrainModel ObjectType | 高 | ORD Terrain Model | 与 GenRule 并列的独立类型 |
| 曲面-曲面 BooleanSubtract | 中 | ORD Cut Fill Volumes | 土方计算 |
| ParametricOperation 扩展 | 中 | OBM Functional Component | Cone/Sphere/Torus/Sweep/Revolve/Loft |
| 表达式驱动的 Assembly 参照点 | 中 | ORD Civil Cell / OBM Intrinsic Variables | 子件位置引用外部对象 |
| Connector 跨平台编码映射 | 中 | OBM Supportline/Bearing Line | Civil3D Point Code ↔ OBM Supportline ↔ OmniM Connector |
| Construction Stage 时序属性 | 低 | OTD Construction Scheduler | 非几何属性，用于 4D |
| TBM UTR 锥度/销钉节距角 | 低 | OTD TBM UTR | 环序列专用 VariationRule |

### 8.8 核心结论

Bentley 三产品（ORD/OBD/OTD）共用 OpenRoads 技术栈，其建模范式高度统一：**横断面模板 + 线形路径 + 端部条件 + 外部地形目标**。这与 OmniM 的 LinearComponent + VariationRule + ExternalTarget 设计完全同构，进一步验证了 OmniM GenRule 体系的工业正确性。

与 Civil 3D（Subassembly Composer 流程图）、Revit（Family 类型）、E3D（Primitive 数据库）相比，Bentley 的 **走廊模板 + Civil Cell** 范式最接近 OmniM 的 LinearComponent + Assembly 组合，是基础设施领域（桥梁/隧道/道路/水工）的首选参考实现。
