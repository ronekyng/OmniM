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

### 3.1 SURFACE TYPES（面构件）

| 类型 | 正确工作流（来自研究） | 代码行为 | 偏差 |
|------|-------------------------------|--------------|-----------|
| **Wall** | 折线路径（平面内 2+ 点）+ 高度 + WallType 层 | `neededClicksForType` 返回 2，点已收集，但 `MainToolbar.tsx` 在 `else` 分支仅使用第 1 点（`px, py, pz`）（第 196 行） | ❌ 折线路径未实现；第 2 次点击被浪费 |
| **Slab** | 闭合边界（平面内 N 点多边形）+ FloorType 层 | 同墙——2 次点击，仅用第 1 点 | ❌ 需要 N 点闭合多边形 + 类型化的 floor 定义 |
| **Roof** | 拉伸轮廓 + 范围（2 点）或 足迹草图（N 点） | 同墙——2 次点击，仅用第 1 点 | ❌ 需要轮廓/范围模式或足迹多边形 |
| **Floor** | 闭合边界（平面内 N 点多边形）+ FloorType 层 | 同墙——2 次点击，仅用第 1 点 | ❌ 需要 N 点闭合多边形 + 类型化的 floor 定义 |

**根本原因**：`neededClicksForType`（`web/src/shared/types/creation.ts:168`）把所有 SURFACE_TYPES 归为一类（返回 2）。`applyPointsToForm`（第 115 行）填充 `startX/Y/Z` / `endX/Y/Z`，但 `MainToolbar.tsx` 对非 LINEAR 类型忽略这些（第 196 行只用单一位置）。拾取提示（`Picking.ts:10`）只显示“点击点 1 / 2”——不支持 N 点多边形创建。

### 3.2 LINEAR TYPES（线构件）

| 类型 | 正确工作流（来自研究） | 代码行为 | 偏差 |
|------|-------------------------------|--------------|-----------|
| **Beam** | 2 点（起点/终点） | 2 次点击，使用起点/终点 ✅ | ✅ 一致 |
| **Column** | Revit 中为 1 点（定位）；Tekla 中为 2 点（底/顶） | 2 次点击，按线构件处理 | ⚠️ 轻微——两种皆有效；Revit 风格的单点模式会更直观 |
| **Pipe** | 2 点（连接点对连接点） | 2 次点击 ✅ | ✅ 一致 |
| **Duct** | 2 点（连接点对连接点） | 2 次点击 ✅ | ✅ 一致 |
| **Road** | 路线 + 纵断面 + 装配（多步 corridor 工作流） | 2 次点击，简化路径 | ⚠️ 预期——完整 corridor 工作流尚未实现（见 ROADMAP.md） |
| **Bridge** | 多步 subassembly corridor / 装配 | 2 次点击，简化路径 | ⚠️ 同 road |
| **Tunnel** | 多步 corridor 装配 | 2 次点击，简化路径 | ⚠️ 同 road |

### 3.3 POINT TYPES（点构件）

| 类型 | 正确工作流（来自研究） | 代码行为 | 偏差 |
|------|-------------------------------|--------------|-----------|
| **PointCloud** | 1 次点击（放置） | 1 次点击 ✅ | ✅ 一致 |
| **TerrainModel** | 1 次点击（放置） | 1 次点击 ✅ | ✅ 一致 |
| **FeatureLine** | N 点折线绘制 | 1 次点击 | ❌ 需要 N 点折线拾取模式 |
| **ContourLine** | N 点折线绘制 | 1 次点击 | ❌ 同 featureLine |

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

## 5. 拾取基础设施缺口

### `web/src/app/components/viewer/three/Picking.ts`
- `pickingHint` 仅支持 `needed === 2` 或 `needed === 1`——无 N 点多边形提示
- `pickGroundPoint` 拾取单个地面平面点——无多点多边形模式

### `web/src/shared/types/creation.ts`
- `neededClicksForType` 对所有 SURFACE_TYPES 返回 2——应区分 wall（2+ 路径）与 slab/roof/floor（N 点闭合多边形）
- `applyPointsToForm` 仅处理 1 或 2 个点——无 N 点多边形序列化
- `CreationForm` 缺少用于闭合多边形存储的 `boundary_points` 字段

### `web/src/app/components/ThreeViewer.tsx`
- 预览线（第 834-860 行）对 1 点→光标仅绘制虚线——N 点拾取无多边形预览
- `addPreviewLine` 绘制顺序线段——对 N 点正确，但无“闭合多边形”视觉反馈

### `web/src/app/components/MainToolbar.tsx`
- `SURFACE_TYPES` 对象落入 `else` 分支（第 196 行）使用单一位置——完全忽略第 2 点
- 未为 slab/roof/floor 写入 `boundary_points` 属性
- 层属性（`height`、`core_thickness`、`finish_thickness`）仅为 wall/slab 硬编码（第 237 行）——缺失 roof 和 floor
