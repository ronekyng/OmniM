---
title: "代码与文档模型的偏差（Code-Level Deviations）"
status: current
last_updated: 2026-10-27
applies_to: "repo"
evidence: "audited against code 2026-10-27"
---
# 代码与文档模型的偏差（Code-Level Deviations）

> 本文档为 [CODE_DEVIATIONS.md](CODE_DEVIATIONS.md) 的简体中文对照版本，章节与表格与英文原版完全一致。

本文档列出代码库与文档化概念模型存在偏差之处。标记为 **RESOLVED** 的条目已在所示日期修复；其余条目为已知缺口，将在未来的代码开发阶段修复。

## 1. 4 类构件分类（面 / 线 / 点 / 体）— **RESOLVED**（2026-08-11）

**文档模型**（见 `DATA_MODEL.md §1`、`GLOSSARY.md §17`、`DOMAINS.md §7.1`、`PLATFORM_ARCH.md §3.1`）：

| 类别 | 对象 | GenRule |
|-------|---------|---------|
| **Surface**（面构件） | Wall, Slab, Roof, Floor | LayerExtrusion |
| **Linear**（线构件） | Beam, Column, Pipe, Road, Duct, CableTray | LinearComponent |
| **Point**（点构件） | PointCloud, TerrainModel, FeatureLine, ContourLine | 非 GenRule |
| **Body**（体构件） | Tank, Pump, Valve, Gear, Equipment | Primitive / Assembly |

**代码现状**：4 类分类法**现已实现**，为 `MainToolbar.tsx` 与 `ObjectPropertiesPanel.tsx` 中的 `SURFACE_TYPES` / `LINEAR_TYPES` / `POINT_TYPES` / `BODY_TYPES`。

---

## 2. 目录单位：毫米 vs. 米制视口 — **RESOLVED**（2026-08-11）

**文档模型**：3D 视口、网格、相机及旧版 GenRule 几何均使用**米**（如旧版墙为 5×0.2×3 m）。

**代码现状**：目录 `ComponentDef` 参数以**毫米**编写（`unit: "mm"`，如 `length=5000`），特征树模板直接用这些毫米值生成几何（截面尺寸硬编码为 mm，拉伸深度由 mm 参数解析）。修复之前，`/api/mesh` 与 `/api/catalog/preview` 返回原始 mm 坐标，导致 5000 mm 的墙在米制视口中渲染为 **5000 米**的墙——巨大且消失在场景雾中（`THREE.Fog` near=100/far=600）。

**修复**：渲染路径现按 `1/1000` 缩放目录构件网格：
- `crates/model/src/geometry.rs` — 新增 `scale_mesh(mesh, factor)` 辅助函数
- `crates/server/src/lib.rs` `get_mesh` — 带 `catalog_ref` 的对象网格被缩放为米（本身已生成米制几何的旧版 GenRule 对象不受影响）
- `crates/server/src/lib.rs` `preview_catalog_component` — 预览网格缩放为米

存储的模型保持毫米（与 `ComponentDef` 参数一致）；仅显示/渲染路径转换为米。

---

## 3. 创建工作流 — 完整偏差矩阵

### 3.1 SURFACE TYPES（面构件）—— 主管道已解决 2026-10-27；内核残留另行跟踪

| 类型 | 正确工作流（来自研究） | 代码行为（当前） | 偏差 |
|------|-------------------------------|------------------------|-----------|
| **Wall** | 折线路径（平面内 2+ 点）+ 高度 + WallType 层 | 分层路径：最少 2 点（N 可继续），点 → `boundary_points`；MainToolbar 以 `path[]` 提交 `layered` genrule → 服务端 `build_layered_path`（含 `base_z` 标高与转角 miter） | ⚠️ 管道已通；残留：不规则坡向 z 取首点 |
| **Slab** | 闭合边界（平面内 N 点多边形）+ FloorType 层 | polygon 模式（最少 3 点），回首点或双击/回车闭合 → `boundary[]` → `build_layered_slab_path`（`base_z` 应用 + 有效性校验） | ✅ 一致 |
| **Roof** | 拉伸轮廓 + 范围（2 点）或 足迹草图（N 点） | 足迹模式与 slab 一样 ✅；挤出模式（2 点）→ `build_layered_roof_profile`（真实剖面、宽度生效、`base_z` 应用） | ✅ 一致 |
| **Floor** | 闭合边界（平面内 N 点多边形）+ FloorType 层 | 同 slab（polygon → `boundary[]`） | ✅ 一致 |

**历史根因（已修）**：`neededClicksForType` 曾把所有 SURFACE_TYPES 归为一类（返回 2），`MainToolbar.tsx` 忽略第二点后的所有点。由拾取模式状态机（`defaultPickModeForType` 按类型、`CreationForm.boundary_points`/`is_closed`、`closeToPoint` 闭合判定）修复，genrule 现消费 `path[]`/`boundary[]`。

### 3.2 LINEAR TYPES（线构件）

| 类型 | 正确工作流（来自研究） | 代码行为 | 偏差 |
|------|-------------------------------|--------------|-----------|
| **Beam** | 2 点（起点/终点） | 2 次点击，使用起点/终点 ✅ | ✅ 一致 |
| **Column** | Revit 中为 1 点（定位）；Tekla 中为 2 点（底/顶） | 2 次点击，按线构件处理 | ⚠️ 轻微——两种皆有效；Revit 风格的单点模式会更直观 |
| **Pipe** | 2 点（连接点对连接点） | 2 次点击 ✅ | ✅ 一致 |
| **Duct** | 2 点（连接点对连接点） | 2 次点击 ✅ | ✅ 一致 |
| **Road** | 路线 + 纵断面 + 装配（多步 corridor 工作流） | 折线点经 `Road.alignment` 消费：沿折线逐段铺装滚放（`build_road_alignment`） | ✅ 管道一致；完整 corridor 工作流仍属 Phase 8 |
| **Bridge** | 多步 subassembly corridor / 装配 | 仍只用首末点 | ⚠️ 待 Phase 8 corridor |
| **Tunnel** | 多步 corridor 装配 | 仍只用首末点 | ⚠️ 待 Phase 8 corridor |

### 3.3 POINT TYPES（点构件）

| 类型 | 正确工作流（来自研究） | 代码行为 | 偏差 |
|------|-------------------------------|--------------|-----------|
| **PointCloud** | 1 次点击（放置） | 1 次点击 ✅ | ✅ 一致 |
| **TerrainModel** | 1 次点击（放置） | 1 次点击 ✅ | ✅ 一致 |
| **FeatureLine** | N 点折线绘制 | polyline 模式（最少 2 点，N 可继续）→ `line` genrule `alignment[]` → 沿折线的细导管 | ✅ 一致 |
| **ContourLine** | N 点折线绘制 | 同 featureLine | ✅ 一致 |

### 3.4 BODY TYPES（体构件）

| 类型 | 正确工作流（来自研究） | 代码行为 | 偏差 |
|------|-------------------------------|--------------|-----------|
| **Tank** | 1 次点击（放置） | 1 次点击 ✅ | ✅ 一致 |
| **Pump** | 1 次点击（放置，基于连接点） | 1 次点击 ✅ | ✅ 一致 |
| **Valve** | 1 次点击（放置，inline） | 1 次点击 ✅ | ✅ 一致 |
| **Gear** | 1 次点击（放置） | 1 次点击 ✅ | ✅ 一致 |
| **Equipment** | 1 次点击（放置） | 1 次点击 ✅ | ✅ 一致 |
| **Part** | 1 次点击（放置） | 1 次点击 ✅ | ✅ 一致 |
| **Assembly** | 1 次点击（放置） | 1 次点击 ✅ | ✅ 一致 |

---

## 4. Rust 后端（crates/）— 正确

Rust 后端（`crates/model/src/gen_rule/mod.rs`）正确映射：
- Wall → `LayerExtrusion`（第 76-77 行）
- Beam/Column → `LinearComponent`（第 80 行）
- Road/Bridge/Pipe/Duct → `LinearComponent`（第 83-90 行）

Rust 后端未发现偏差。

---

## 5. 拾取基础设施缺口 — **大部分已解决** (2026-10-27)

**状态**：端到端 N 点拾取管道已实现并验证；上文 §3.1/§3.3 中"只用第 1 点/未实现"的行已在原文位置更正。仍然属实的内容：

### `web/src/app/components/viewer/three/Picking.ts`
- ~~`pickingHint` 仅支持 2 点/1 点两态~~ **已解决**：`pickingHint` 处理 polyline/polygon 完成态（带回点中文提示、按模式与点数四态）
- ~~单点地面拾取、无多点多边形模式~~ **已解决**：`PickMode`（polyline/polygon/segment/place）状态机驱动多点收集；每次拾取应用 `snapPoint` 吸附

### `web/src/shared/types/creation.ts`
- ~~对所有 SURFACE_TYPES 返回 2~~ **已解决**：`defaultPickModeForType` 按 wall→polyline(≥2)、slab/floor/roof→polygon(≥3)、beam/pipe/duct→segment、其余→place 区分
- ~~仅处理 1/2 点~~ **已解决**：N 点序列化进 `CreationForm.boundary_points` + `is_closed`
- ~~缺少 `boundary_points` 字段~~ **已解决**

### `web/src/app/components/ThreeViewer.tsx`
- ~~仅 1 点→光标虚线预览；点击时把整条折线在同一 marker 组反复重画（生命周期混乱）~~ **已解决 (2026-10-27 预览重构)**：拆分为"已确认视觉"（点标记 + 链线/填充，由 effect 依 `creationPoints` 同步）与"逐帧 hover 组"（虚线、整条多段墙 ghost、多边形环+填充），每次指针移动重建；hover 终点与点击走同一 `snapPoint` 吸附管线

### `web/src/app/components/viewer/three/Markers.ts`
- ~~预览只画最后一段的灰色厚板；层厚换算大 20 倍（`unit=0.02`）；`MeshBasicMaterial` 无光照；转角无 miter~~ **已解决 (2026-10-27 预览重构)**：`addLayeredWallPreview` 以服务端同款 mm→m 换算（`0.001`）构建整条逐段 ghost，转角 miter 与内核一致（`e=(t/2)·tan(φ/2)`，封顶），带光照半透明材质（`opacity 0.55`、关深度写入）；无效多边形底面红色预览（`polygonPreviewError`，镜像服务端 `validate_boundary`）

### `web/src/app/components/MainToolbar.tsx`
- ~~SURFACE_TYPES 落入 else 分支只用单点~~ **已解决**：LAYERED_TYPES 以 `path[]`（墙折线）/`boundary[]`（slab/roof/floor 底面）提交 `layered` genrule；LINE_GENRULE_TYPES 以 N 点 `alignment[]` 提交 `line`
- ~~未写 `boundary_points`~~ **已解决**（表单携带 boundary_points，genruleCreate 消费之）
- 残留内核缺口（已跟踪至内核工作）：`to_p2` 丢弃 z（已改为 base_z 保留首点标高）；roof 挤出模式（已改为真实剖面）；road 对齐（已消费 `alignment`）；转角 miter（已实现，限 90°/45° 精确）

### 测试
- `stateMachine.test.ts` 覆盖 idle/picking/configuring 全流程含多点累积；polygon 闭合与 span 过滤用例已随残留项补上（multi-point picking describe 块，6 用例）。

---

## 6. 写入路径、持久化与几何路径偏差 — **Known**（2026-10-27）

**状态：Known — 2026-10-27。** 依据：[DOC_AUDIT_2026-10-27](DOC_AUDIT_2026-10-27.zh-CN.md)附录 A 对 `crates/server/src/lib.rs` 的静态扫描，并做了代码抽查。逐端点矩阵见 [WRITE_PATHS.zh-CN.md](../architecture/WRITE_PATHS.zh-CN.md)。以下各项均未解决。面向建模者的逐对象能力视图见 [CAPABILITY_MATRIX.zh-CN.md](../CAPABILITY_MATRIX.zh-CN.md)。

### 6.1 单一写入路径被违反（审计 P0-2）

> **2026-10-28 更新——已修复。** Wave 1 已迁移所有变更 handler：切片 2–4 覆盖对象/属性/材质、草图/构件/目录定义、系列/生命周期/管理区/分解/测量；切片 5 覆盖目录实例化、IFC 导入与 `load_series`。`scripts/check_write_paths.py` 现报告 **0** 个白名单外 `project.write()`，并接入 CI。剩余例外记录于 [WRITE_PATHS](../architecture/WRITE_PATHS.zh-CN.md)：`section_handler`（仅缓存刷新）、`import_ifc`（单条 `Command::ReplaceProject` 事务）、`POST /api/catalog/import`（会话级 `LIBRARY`，非 `Project`）。下方数字为修复前审计快照。

**文档模型**（ADR-0006、PLATFORM_ARCH §1、DATA_FLOW §0）：所有变更都经 `Validator` + `Executor`，并进入撤销栈。

**代码现状：** 持有 `project.write()` 的 **55** 个 handler 中，只有 **13** 个出现 executor；约 **42** 个直接修改 `Project`，不产生撤销条目。例如：`update_object`、`transform_object`、`set_parent`、`connect`、`create_material`/`assign_material`、`update_sketch`/`solve_sketch`/`update_sketch_entity`、`import_ifc`、`instantiate_from_catalog`，以及全部系列编辑 handler。改名、赋材质、连接管道、编辑系列、导入 IFC、放置目录构件都无法撤销。

### 6.2 “自动保存”不成立；默认启动完全不保存（审计 P0-3）

> **2026-10-28 更新——已修复。** 默认落盘：`main.rs` 设 `save_path = Some(project_file)`，除非 `OMNIM_EPHEMERAL=1`；`Project::save` 原子写（tmp + rename）；**所有已迁移 handler** 都经 `mutation::persist` 落盘，包括对象、草图、构件、目录定义、系列、生命周期、测量、管理区/分解、实例化与 IFC 导入。`POST /api/project/save` 可按需落盘。唯一不写项目的变更路径是 `.omnimlib` 导入到全局 `LIBRARY`。下方内容为修复前快照。

**文档模型**：变更会自动持久化。

**代码现状：** 只有 **5** 个 handler 调用 `Project::save(...)`：`update_object`、`add_feature`、`update_feature`、`remove_feature`、`rebuild_component`。`POST /api/objects`、`/api/genrule`、`/api/omcom`、`/api/catalog/instantiate`、`/api/import/ifc`，以及所有 `/api/series/*` 编辑都不会保存。此外，`main.rs` 在项目路径为默认 `project.omnim` 时把 `save_path` 设为 `None`，因此默认启动完全不落盘。`scripts/dev-server.sh` 会透传 `OMNIM_PROJECT`，但默认值为 `project.omnim`，所以默认用该脚本启动同样不持久化。

### 6.3 几何路径分裂（审计 P0-4）

**文档模型**：目录/系列驱动几何生成。

**代码现状（2026-10-28 更新——部分修复）：** **layered** 族已收敛——路径驱动的
wall/slab/floor/roof 创建走特征树（`PathSweep` 步骤 → `gen_rule::path_sweep`，与遗留
构建器同一内核，带逐位 parity 测试），产出**领域对象类型**并携带 `catalog_ref`/`param_values`。
剩余分裂：矩形参数快速创建（`wall`/`slab` 无路径变体）、屋顶两点坡向、目录中基于
extrude 的 pipe/road 模板与 `LinearComponent` 引擎。审计时的三条并行路径：

- `/api/genrule`（工具栏）：硬编码 `build_wall`/`build_slab`/`build_road`/`build_pipe` 网格，`param_values` 为空，对象类型为 `Wall`/`Slab`/`Road`/`Pipe`。
- `/api/catalog/instantiate`（构件面板）：特征树 → BRep/网格（官方路径），但创建 `ObjectType::Component`（不是 `Wall`/`Beam`/…），无撤销、不保存。
- `/api/import/ifc`：导入网格，映射类型，无撤销、不保存。

官方几何路径是特征树/BRep 管线；genrule 直通属于遗留/过渡方案。见 [DATA_FLOW.zh-CN.md §4](../architecture/DATA_FLOW.zh-CN.md#4-d-目录--系列实例化)。

> **2026-10-29 更新（P8）。** 管道路由改为经 `LinearComponent` 内核沿拾取的 N 点折线扫掠（`gen_rule::pipe_route`），不再只做直管；路由记录在对象上（`route_points`/`pipe_radius`/`route_length`）。仍是网格化 Linear 族路径（无特征树模板），该分裂明确保留为过渡。

### 6.4 系列门禁、材质、连接弱于文档（审计 P0-5）

**文档模型**：系列门禁强制使用批准构件、材质与连接兼容性。

> **2026-10-28 更新（Wave 4）——大部分已修复。** 系列门禁现在会定位可达构件的
> `SpecComponent`，强制其 `ParamConstraint`（`fixed` 相等、`min`/`max` 边界 → 400 含理由），
> 并应用 `SpecComponent.material`（id → 名称忽略大小写；解析失败 → 400）。
> `connect()` 拒绝自连、重复连接与未知连接类型，支持 `?connection_type=`，并对目录
> 实例检查连接器 `connection_type_code` 兼容性。`ConnectorDef` 新增 `up_x/y/z_expr`
> （法兰朝向）。**剩余**：连接时的解析值朝向校验、`step` 强制、`from_point`/`to_point`
> 填充。下方为修复前快照。


**代码现状：**

- `check_series_gate()` 只检查构件是否可从某个系列可达（`crates/model/src/project.rs:88-103`），不校验材质、参数范围或连接等级。
- `instantiate_from_series()` 只填充 `ParamConstraint.default_value`，不强制 `min/max/fixed`，也**不应用** `SpecComponent.material`（`crates/model/src/project.rs:368-386`）。
- `Project::connect()` 只检查两个对象是否存在，不校验连接点、类型码、口径或压力等级，也不写入 `from_point`/`to_point`（`crates/model/src/project.rs:105-121`）。
- `ConnectorDef` 只有 `direction`，没有 `up`/基准轴，无法确定法兰螺栓孔或阀杆朝向（`proto/omnim.proto:506-518`）。

---
