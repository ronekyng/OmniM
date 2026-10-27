# OmniM 开发路线图

**最后更新：2026-10-27**

> ### 当前阶段：第八阶段 — 领域扩展与生产级构件库
>
> 已完成：第一至七阶段（第一阶段为部分完成，见下方说明）。第九阶段提前部分交付（AMIR AI 引擎）。
>
> ### 状态诚实性说明
>
> 本文档早期版本将 Omcom **文本 DSL / nom 解析器**列为已完成的 Phase 1 交付物。这**不准确**：当前实际交付的是统一数据模型、类型化 `OmniCommand` schema 以及完整的类型化命令执行器。Web UI 中的 Omcom 控制台执行的是 **JSON 命令加少量临时文本捷径**——**没有基于 nom 的文本语法解析器**。文本 DSL 目前是草案 RFC：[RFC-0001: Omcom Text DSL](../RFC/README.md)。因此第一阶段标记为 **🔀 部分完成**。

## 状态图例

| 标记 | 含义 |
|------|------|
| ✅ 完成 | 已实现并在代码库中验证 |
| 🔄 进行中 | 活跃开发，部分交付物已落地 |
| 🔀 部分完成 | 主体已交付，存在明确的缺口 |
| 📅 计划中 | 尚未开始 |

---

## Phase 1：核心数据模型 & Omcom DSL (2026 Q1-Q2) — 🔀 部分完成

**目标**：建立基础对象模型、类型化命令/命令执行器层，以及 Omcom 文本 DSL。

### 已交付 ✅

1. **统一数据模型** — `OmniObject`、核心层级（SpatialElement / Element / System）、Property + PropertySet 系统、材质定义、覆盖 4 个领域的 ObjectType 枚举。Protobuf/serde 类型作为内部 schema（`.proto` 文件纯粹作为 serde-JSON 类型声明来源；**未使用 protobuf 运行时/线上编码**）。
   *代码证据：`crates/model/`（含 `crates/model/src/proto/`）、`crates/core/`。*
2. **类型化命令层** — `OmniCommand` 消息类型 + 完整的类型化命令执行器，支持完整撤销/重做。
   *代码证据：`crates/omcom/src/lib.rs`、`crates/server/src/lib.rs` 中的执行器。*
3. **项目生命周期模型**、**行政分区层级**（世界→…→地块，GeoBoundary）、**项目分解结构**（项目群→…→零件）、**关系系统**（包含 / 连接 / 空间关系）。
   *代码证据：`crates/model/src/`。*

### 未交付 📅

- **基于 nom 的 Omcom 文本 DSL**（`crates/omcom-parser`、EBNF 语法、文本→OmniCommand 映射、带用法提示的错误消息）。当前交互式“控制台”仅为 JSON + 少量临时捷径。
  *设计提案：[RFC-0001: Omcom 文本 DSL](../RFC/README.md)（草案）。*

### 完成标准（修订版）
- ✅ 数据模型 + 类型化执行器处理所有对象类型的创建/命令流程
- ✅ 每条已执行命令可撤销/重做
- 📅 文本语法解析所有 `create wall/beam/…` 命令（待 RFC-0001 获得采纳）

---

## Phase 2：约束求解器 & 草图系统 (2026 Q2-Q3) — ✅ 完成

- **草图数据模型**：SketchPlane、SketchEntity（直线/弧/圆/样条）、11 种约束类型、项目中的 SketchStore。
- **约束求解器**：集成 slvs 求解器（crate：`slvs`）；草图→求解→已求解草图管线，自由度分析、过/欠约束检测与报告。
- **Omcom（JSON）草图命令**：`sketch new/add/constr/solve`、`component create`、`feature add/remove`、`EXTRUDE_SKETCH (107)` / `REVOLVE_SKETCH (108)`——均带执行器撤销/重做。
- **草图编辑器前端**：`/sketch-editor` 页面、2D 画布（网格/坐标轴/图元）、Line/Circle/Arc 工具、约束工具栏（11 种）、Solve 按钮、Extrude/Revolve 对话框、与主视图往返导航。

### 代码证据
`crates/sketch/`、`crates/server/src/lib.rs` 中的求解管线（`/api/sketches/*`）、`web/src/app/sketch-editor/`。

---

## Phase 3：特征引擎 & BRep 内核 (2026 Q3-Q4) — ✅ 完成

- **Feature 作为一等实体**：缓存 MeshGeometry、脏标记自动重建、特征依赖排序、回滚与抑制。
- **9 种特征类型**：Extrude、Revolve、Sweep、Loft、Boolean（并/差/交）、Chamfer、Fillet、Pattern、Shell。
- **truck-BRep 内核**（`truck-modeling` / `truck-polymesh`）：从草图拉伸/旋转/扫掠/放样、BRep 实体布尔运算、三角网格细分以供渲染。
- **参数系统**：命名参数存储、meval 表达式引擎、特征参数引用。
- **Component Editor UI（Revit 风格）**：可拖拽排序的 Feature Panel、Feature Ribbon（9 种类型图标）、点击特征→自动建草图→绘制→Done→对话框工作流、AddFeatureDialog、Feature Card 双击编辑、`+ Component` 按钮、对象树“Edit”右键菜单、Rebuild All 按钮。

### 代码证据
`crates/model/`（BRep→网格；旧 `crates/renderer/` 演示 crate 已于 2026-10-27 移除）、`crates/server/src/lib.rs` + `crates/model/` 中的特征引擎、`web/src/app/components/component-editor/`。

---

## Phase 4：元件库 & 系列库 (2027 Q1) — ✅ 完成

借鉴 PDMS PARAGON 的**元件目录**与 SPECON 风格的**系列库**，超前于原计划交付：

- **ComponentDef** 带 ParamDef（长度/角度/整数/选择 + 默认值/取值域/公式）、ConnectorDef（位置/方向表达式、口径、类型码）、Category 层级。
- **Series**：选择器决策树（问题→答案→Selector/SpecComponent）、SpecComponent 参数约束、数值范围匹配（如 DN=50..200）。
- **选择引擎**与**连接点类型码兼容性**（"FAA"、"BW" 等）。
- 目录/系列可存于项目内或外部 `.omnimlib` / `.omnimseries` JSON 文件。

### 代码证据
`crates/model/src/catalog.rs`、`crates/model/src/series_store.rs`。

---

## Phase 5：目录构件建模 (2027 Q2) — ✅ 完成

- **结构**：IPE/HEA/HEB/CHS/SHS/角钢族，参数化截面草图→拉伸。
- **管道**：直管、长半径弯头、三通、大小头、对焊法兰、阀门——带 P-point 连接点与类型码。
- **建筑**：混凝土梁/柱/墙、门、窗模板。
- **系列定义**：CS150、SS300、Concrete-C30、Steel-Q355、Highway-GB。
- **基础设施**：截面库（I/箱形/圆环/矩形/L 形）、ASME B36.10M 壁厚表、FeatureTemplate + FeatureTreeRegistry、实例化管线（ComponentDef→Feature→Sketch→BRep→Mesh）、REST 端点 `POST /api/catalog/instantiate`、`POST /api/catalog/preview`。

### 代码证据
`crates/model/` 中的内置目录种子数据 + `instantiate_from_series()`；`crates/server/src/lib.rs` 中的处理器。

---

## Phase 6：GenRule 重构 & 默认模板 (2027 Q3) — ✅ 完成

- **Library ↔ SeriesStore ↔ Project 三层分离**：Library 为 session 级全局静态（`static LIBRARY: RwLock<Library>`）；SeriesStore 为被 Project 引用的独立模块；Project 持有对象 + 私有 SeriesStore + `external_series_refs`，不再持有 Library 字段。
- **系列关卡** `check_series_gate()`：拒绝不在任何已加载系列可达范围内的构件；未加载系列时为开放模式。
- **GenRule → 目录**：所有 GenRule 生成器（LayerExtrusion、LinearComponent、Assembly、BooleanSubtract、Primitive）经由引用目录 ComponentDef 的特征树创建对象。`POST /api/genrule` 使用 `create_object_from_default_template()` / `instantiate_from_series()`。
- **CatalogRef**：每个实例化对象携带 `component_def_id`、`series_id`、`selector_path`、`param_values`。
- **默认模板注册中心**：`create wall`→Concrete-Wall、`create beam`→Concrete-Beam-Rect 等；按目录取值域做类型参数验证。

### Phase 6b（Library & SeriesStore 基础设施）— ✅ 完成
`.omnimlib` / `.omnimseries` JSON 文件加载/保存、系列解析优先级（项目私有 > 外部文件 > 内置种子）、session 启动时从 `~/.omnim/library.omnimlib` 自动加载。

### 代码证据
`crates/model/src/gen_rule/mod.rs`、`crates/model/src/library.rs`、`crates/model/src/series_store.rs`。

---

## Phase 7：用户界面 (2027 Q4) — ✅ 完成（含一项已知缺口）

- **Project Workspace**：3D 视口（轨道/平移/缩放、选择、剖切、测量、线框/着色/X 光/仅边线视图模式）、草图编辑器、特征树、可用构件面板（系列筛选/参数滑块/拖拽放置）、属性面板（目录引用/材质）、Omcom 控制台（语法高亮/历史/Tab 补全/验证错误）、AI 聊天（自然语言）。
- **Library Manager**（`/library-manager`）：ComponentDef 树、参数定义编辑器、特征树模板分配、3D 预览、`.omnimlib` 导入/导出。
- **SeriesStore Manager**（`/series-manager`）：决策树编辑器、内置种子查看器、外部 `.omnimseries` 加载/保存、拖拽映射 ComponentDef、参数约束编辑器。
- **UI 架构**：Zustand、React Query 数据层、PanelRegistry 可插拔面板、可调整布局、CSS Modules、`ThreeViewerContext`。

### 已知缺口 🔀
交互式 3D 创建仅对 `PICKING_TYPES` 支持 1–2 点点选。N 点多边形/折线点选（slab/roof/floor 边界、featureLine、contourLine 需要）**未实现**——见 [CODE_DEVIATIONS.md](CODE_DEVIATIONS.md) §3/§5。

（早期版本在此列出“应用前命令预览”；该 AI 命令预览/接受工作流**未交付**，仍属 Phase 9 事项，且今天也只是部分存在。）

---

## Phase 8：领域扩展 & 生产级构件库 (2028 Q1-Q2) — 🔄 进行中

**当前状态**：生产级目录数据已大量落地（钢型材按 EN 10365/GB/T 706，管道等级按 ASME B36.10，混凝土模板，带法兰等级的连接点定义）；材料映射到真实标准；构件通过 BRep 正确性检查验证。领域专项工作仍待完成：

1. 📅 建筑：完整 IFC 元素覆盖、MEP 系统、工程量统计
2. 📅 土木：地形拟合与土方、路线/纵断面编辑
3. 📅 工厂：带分支管道布线、P&ID 集成
4. 📅 机械：零件/装配、运动副
5. 📅 前端：领域专属面板、筛选器、领域感知拖拽放置

部分完成标准已达（钢 IPE/HEA/HEB 系列、CS150/SS304 DN 覆盖、BRep 验证）。

---

## Phase 9：AMIR AI 编译 (2028 Q3) — 🔄 部分完成（提前交付，仍有缺口）

**已交付** ✅：AMIR AI 引擎，含 OpenAI / Anthropic / Mock 提供方；自然语言→OmniCommand 翻译；多轮聊天；逆向“描述模型”路径。已作为 Project Workspace 中的 AI 聊天面板集成。

**未交付** 📅：复杂操作的结构化多步规划、自然语言验证错误解释与自动修正建议、置信度评分、人在回路接受/拒绝命令预览、按领域的多智能体系统、标准规范 RAG 查询。

### 代码证据
`crates/amir/`（提供方、translate、chat、describe）、`web/` AI 聊天面板。

---

## Phase 10：云服务器 & 协作 (2029) — 📅 计划中

**今日现实** ✅（本阶段子集，提前完成）：**Axum REST 服务器**（约 100 条路由），**JSON 文件（`.omnim`）持久化**，完整撤销/重做状态历史，目录/系列文件端点。

**未实现** 📅：PostgreSQL + PostGIS 存储、版本历史/审计追踪（目前仅 JSON 文件保存）、WebSocket 实时通知、几何流式传输、客户端缓存层、多用户协作/冲突解决、在线状态、协作 UI。服务器中**没有任何 WebSocket**。

---

## Phase 11：IFC/STEP 格式兼容 (2029) — 🔀 部分完成

- **IFC 2x3 / IFC 4**：导入 + 导出 ✅，通过**手写的解析器/写出器**（`crates/ifc/`）——**实体覆盖不完整**。详见 [FORMAT_COMPAT.md](FORMAT_COMPAT.md)。
- **STEP / DXF / DWG / GIS / glTF / OBJ / FBX**：❌ 未实现（代码中仅有 IFC + JSON 路径）。计划见 [FORMAT_COMPAT.md](FORMAT_COMPAT.md)。

---

## Phase 12：分析与生态 (2029+) — 📅 计划中

尚未开始：结构分析、碰撞检测、成本估算、插件生态。

## 技术债待办

在开发过程中记录的零散事项；按时机捡起，不排阶段：

- **测试隔离重构** — model 测试共享 session 级全局 `static LIBRARY`（见[测试指南 → 已知不稳定测试](../guides/TESTING.zh-CN.md)）。已观察到一次并行测试偶发失败（`test_find_and_rebuild_catalog_instances`）；压力测试未能复现，但随着测试规模扩大，共享全局状态的依赖是真实风险。候选修复方案见测试指南。
- **日志规范化** — 用结构化日志 crate（`tracing` 或 `log`）替换生产路径中零散的 `eprintln!`。当前残留输出仅为错误路径/启动探针。

---

## 非目标（当前范围）

1. **照片级渲染**
2. **离线优先架构**
3. **移动 AR/VR / 移动应用**（不计划、未构建移动应用）
4. **完整 CAM/CNC 能力**
5. 现阶段同样明确：桌面打包（Tauri）、生产 Web 构建中的 WASM 共享内核、nom 文本 DSL 执行（待 RFC-0001）。

---

## 里程碑总结

| 里程碑 | 目标 | 关键能力 | 状态 |
|--------|------|----------|------|
| M1 | 2026 Q1–Q2 | 核心数据模型 + 类型化命令执行器 | ✅（文本 DSL → RFC-0001） |
| M2 | 2026 Q3 | 约束求解器 + 草图系统 | ✅ |
| M3 | 2026 Q4 | 特征引擎 + truck-BRep 内核 | ✅ |
| M4 | 2027 Q1 | 元件目录 + 系列库（PARAGON/SPECON） | ✅ |
| M5 | 2027 Q2 | 目录构件建模（20+ 构件） | ✅ |
| M6 / M6b | 2027 Q3–Q4 | GenRule 重构、默认模板、Library/SeriesStore 基础设施 | ✅ |
| M7 | 2027 Q4 | UI：工作台 + Library Manager + SeriesStore Manager | ✅ |
| M8 | 2028 Q1–Q2 | 领域扩展 + 生产级构件库 | 🔄 |
| M9 | 2028 Q3 | AMIR AI 编译 | 🔀 部分完成 |
| M10 | 2029 | 云存储 + 协作 | 📅（已有 REST+JSON 文件服务器） |
| M11 | 2029 | 格式兼容 | 🔀 部分完成（仅 IFC） |
| M12 | 2029+ | 分析 + 生态 | 📅 |
