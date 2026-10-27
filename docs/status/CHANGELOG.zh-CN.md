# 变更日志（Changelog）

本文件记录 OmniM 项目的所有重要变更。仅包含阶段性实质性摘要——这不是提交日志。

格式基于 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.1.0/)；本项目暂不使用语义化版本，条目按路线图阶段组织。

**最后更新：2026-10-27**

## Keep a Changelog 图例

- **新增（Added）** — 新能力
- **变更（Changed）** — 对既有行为的更改
- **修复（Fixed）** — 缺陷修复 / 已解决偏差
- **未实现（已知缺口）** — 已计划但缺失的能力，为诚实起见明确列出

---

## [2026-10-27] — 文档重组与死代码清理

### 新增
- `docs/` 重组为 `architecture/`、`guides/`、`specification/`、`reference/`、`status/`、`RFC/`（每篇中英双语），新增文档门户（`docs/README.md`）。
- 新参考文档：全量 REST API 参考（`reference/API.md`，对照 `crates/server/src/lib.rs` 审计约 100 条路由）。
- 新规范：元件库与系列库（`specification/CATALOG_SERIES.md`）、数据流、架构决策记录（6 条种子 ADR）。
- 新指南：快速开始、构建、架构地图、测试。
- RFC-0001：Omcom 文本 DSL（草案）——把原来的"文本 DSL"愿景归位为未实现的设计提案。
- 首份端到端 CHANGELOG（本文件）。

### 变更
- `PLATFORM_ARCH.md` 由云优先愿景式改写为对当前实现的如实描述（REST + JSON 文件持久化，无 WebSocket/DB/WASM/移动端）。
- `FORMAT_COMPAT.md` 修正：IFC 导入/导出为部分支持（手写解析器）；DXF/DWG/STEP-CAD/GIS/glTF 未实现。
- `ROADMAP.md` 状态修正（Phase 1 的文本 DSL 声明改述为 RFC-0001；每阶段附代码佐证）。
- `DATA_MODEL.md` 对照代码审计：修正约 25 处过时/不实声明（如已实现的 Torus/Dish 图元、真实 FeatureType 枚举、"规范-代码"分歧标注 🗓 计划中）。
- `OMCOM_REFERENCE.md` 重写：原为虚构文本 DSL 的参考；现忠实记录真实的类型化 OmniCommand 系统（proto/omcom.proto + crates/omcom Validator/Executor）。

### 删除
- `crates/renderer` —— 无人依赖的 wgpu/winit 演示 crate，从 workspace 移除；渲染由浏览器内 three.js 承担，原生查看器如需重启走 RFC。
- 未用依赖：`prost-types`（core）、`tower`（server）、`omnim-omcom`（amir）。
- 已声明未实现的 Action 枚举值（62/80/81/82 等）在 `proto/omcom.proto` 中标注为保留（编号保留、不复用）。
- 死前端组件：`CatalogBrowser.tsx`、`SeriesBrowser.tsx`、`OmcomConsole.tsx`、`ViewCube.tsx`（已被 OmniPanel / ViewCube3D 取代）。
- 仓库垃圾：web 一次性脚本（`fix_delim*.sh`、`test.sh`、误提交的 `web/dist/`）、过时的 `scripts/update_roadmap.py`、调试 `eprintln!`/`console.log`。
- 仓库根目录测试文件迁移至 `fixtures/ifc/`（文档同步）；`logo.png` 移至 `docs/img/` 并嵌入中英 README。

### 修复
- 整个 workspace 编译警告清零（未用变量/导入、多余 `mut`）。
- 文档内部链接全部校验（0 坏链）。

### 未实现（已知缺口）
- 不变：仅 REST（无 WebSocket/实时协作）、JSON 文件持久化（无数据库/版本历史）、无 WASM 共享内核、无桌面/移动打包（见 `PLATFORM_ARCH` 未来方向章节）。

---

## [Phase 8 — 进行中]（目标 2028 Q1-Q2，已启动）

### 新增
- 生产级目录数据：钢型材（IPE/HEA/HEB/CHS/SHS，按 EN 10365、GB/T 706）、管道等级（CS150、SS300 / ASME B36.10）、混凝土墙/梁/柱/板模板、带法兰等级的连接点定义。
- 材料映射到真实标准（EN 10025、GB/T 1591、ASTM A36）；构件模板经 BRep 正确性验证。

### 未实现（已知缺口）
- 领域专项操作（MEP、土方/路线、管道布线、运动副）、工程量统计、领域专属 UI 面板。

### 修复
- 见 [CODE_DEVIATIONS.md §1/§2](CODE_DEVIATIONS.md)：4 类构件分类法、目录毫米 vs. 米缩放——**均于 2026-08-11 RESOLVED**。

---

## [Phase 7 — 用户界面]（2027 Q4）— 完成

### 新增
- Project Workspace：three.js 视口（轨道/平移/缩放、选择、剖切、测量、线框/着色/X 光/仅边线视图模式）、特征树、可用构件面板（系列筛选/滑块/拖拽放置）、属性面板、Omcom 控制台（语法高亮/历史/Tab 补全）、AI 聊天面板。
- Library Manager（`/library-manager`）：ComponentDef 树、参数编辑器、特征模板分配、3D 预览、`.omnimlib` 导入/导出。
- SeriesStore Manager（`/series-manager`）：决策树编辑器、种子查看器、`.omnimseries` 加载/保存、拖拽映射、约束编辑器。
- UI 架构：Zustand、React Query、PanelRegistry、可调整布局、CSS Modules。

### 未实现（已知缺口）
- 场景内 N 点多边形/折线拾取（见 CODE_DEVIATIONS §3/§5）；AI 命令预览/接受工作流（延至 Phase 9，部分存在）。

---

## [Phase 6 / 6b — GenRule 重构与 Library 基础设施]（2027 Q3-Q4）— 完成

### 新增
- Library ↔ SeriesStore ↔ Project 三层分离；Library 为 session 级全局静态；项目私有 SeriesStore + `external_series_refs`。
- 系列关卡（`check_series_gate`）、GenRule→目录/系列实例化、实例上的 `CatalogRef`、默认模板注册中心、按目录取值域的参数验证。
- `.omnimlib` / `.omnimseries` 外部文件格式、系列解析优先级、session 自动加载。

### 变更
- `generate_geometry()` 弃用，由特征引擎取代；`POST /api/genrule` 现通过目录/系列系统创建对象。

---

## [Phase 5 — 目录构件建模]（2027 Q2）— 完成

### 新增
- 覆盖结构（IPE/HEA/HEB/CHS/SHS/角钢）、管道（直管、弯头、三通、大小头、法兰、阀门）、建筑（混凝土/门/窗）族别的 20+ 参数化目录构件；系列定义 CS150、SS300、Concrete-C30、Steel-Q355、Highway-GB。
- 截面库、ASME B36.10M 壁厚表、FeatureTemplate + FeatureTreeRegistry、实例化管线、REST `/api/catalog/instantiate` 与 `/api/catalog/preview`。

---

## [Phase 4 — 元件目录与系列库]（2027 Q1）— 完成

### 新增
- PARAGON 风格元件目录（ComponentDef/ParamDef/ConnectorDef、Category 层级）与 SPECON 风格系列库（选择器决策树、SpecComponent 约束、数值范围匹配）、选择引擎、连接点类型码兼容性、外部文件存储。

---

## [Phase 3 — 特征引擎与 BRep 内核]（2026 Q3-Q4）— 完成

### 新增
- Feature 作为一等实体（缓存 MeshGeometry、脏跟踪、重建管线、回滚/抑制）；9 种特征类型（Extrude、Revolve、Sweep、Loft、Boolean、Chamfer、Fillet、Pattern、Shell）。
- truck-BRep 集成（实体拉伸/旋转、沿路径扫掠、放样、BRep 布尔运算）、细分网格、带 meval 表达式的参数系统。
- Revit 风格 Component Editor UI（feature 面板/工具带、自动建草图工作流、拖拽排序、Rebuild All）。

---

## [Phase 2 — 约束求解器与草图]（2026 Q2-Q3）— 完成

### 新增
- 草图数据模型（SketchPlane、图元、11 种约束类型、SketchStore）；基于 slvs 的约束求解器，带自由度分析与过/欠约束报告；草图/特征/构件命令带撤销/重做；独立 Sketch Editor 网页；`EXTRUDE_SKETCH` / `REVOLVE_SKETCH` 生成 3D 网格。

---

## [Phase 1 — 核心数据模型与命令层 — 部分完成]（2026 Q1-Q2）

### 新增
- 统一数据模型（OmniObject 层级、属性/属性集、材质、覆盖 4 领域的 ObjectType）、项目生命周期与分解结构、行政分区层级、关系系统。
- `OmniCommand` 类型化 schema，纯作 serde-JSON 类型使用（无 protobuf 线上运行时）以及完整的类型化命令执行器，带完整撤销/重做。

### 未实现（已知缺口）
- **Omcom 文本 DSL / nom 解析器**——Web UI 控制台执行 JSON 命令加少量临时捷径。文本 DSL 为草案 RFC：[RFC-0001](../RFC/README.md)。

---

## 基线（当前运行时）

- Axum REST 服务器（约 100 条路由）+ JSON 文件（`.omnim`）持久化；**无 WebSocket/实时协作，无数据库/版本历史/审计追踪**。
- AMIR AI 引擎（OpenAI/Anthropic/Mock 提供方）：自然语言→OmniCommand 翻译、聊天、逆向模型描述。
- 手写 IFC（2x3/4）导入/导出，**实体覆盖不完整**；无 STEP/DXF/GIS/glTF 支持。
- Web UI：基于 three.js 的工作台、草图编辑器、构件编辑器、系列/库管理器、AI 聊天、Omcom 控制台。
