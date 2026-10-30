---
title: "装配容器与子件（Assembly Container & Parts）"
status: design
last_updated: 2026-10-30
applies_to: "proto/, crates/model, crates/omcom, crates/server, web/"
evidence: "Phase-8 装配升级的设计契约；同一变更集内增量实现"
---
# 装配容器与子件（Assembly Container & Parts）

> **为什么有这份文档。** 2026-10-30 的 UI/UX 审核与代码审计都指出旧 `Assembly` GenRule 的问题：把基本体网格合并成一个对象后**结构全部丢失**——没有子件、没有 BOM、不能单独选择。本契约把用户侧流程改为**装配容器 + 普通对象子件**（带 `catalog_ref` 的 `Component` 实例，或基本体 `Part`），同时保留旧合并网格路径的可读性。

## 1. 对象模型

```
Assembly（ObjectType::Assembly，401）         ← 容器，自身无几何
├── Component（ObjectType::Component，500）   ← 目录子件：catalog_ref + 烘焙几何
├── Component                                 ← …
└── Part（ObjectType::Part）                  ← 基本体子件：烘焙网格，无 catalog_ref
```

- **容器**：名称、可选 `parent_id`（场地/建筑/楼层/其它容器）、`geometry = None`、属性/标签。它是选择、分组与导出单位。
- **子件**：通过 `parent_id` + 空间 `Containment` 挂在容器下的直接子对象。目录子件走与 `POST /api/catalog/instantiate` 相同的实例化辅助函数（参数 mm、放置 m、特征树几何、`catalog_ref`）；基本体子件是 box/cylinder/sphere/cone 网格，按请求位姿**烘焙**。
- **放置**：v1 把位姿**烘焙**进子件几何（平移 + Z 旋转）。逐对象 `placement` 字段是后续工作；在此之前移动容器会把同一变换级联到直接子件（§4）。
- **嵌套**：容器可挂在任何对象下；子件是叶子。嵌套装配 v1 不做。

## 2. 命令契约 — `AssemblyCreate`

```
Action: ASSEMBLY_CREATE (130)
Operation: CreateAssemblyOperation {
  string name = 1;
  string parent_id = 2;          // 可选容器父级
  bool skip_series_gate = 3;     // 语义同 InstantiateCatalog
  repeated AssemblyPart parts = 4;
}
AssemblyPart { string name = 1; oneof source {
  ComponentPart component = 2;   // component_def_id, params, position, rotation_z(rad)
  PrimitivePart primitive = 3;   // shape, p1..p3, position, rotation_z(rad)
} }
```

校验（`Validator` 与 executor 双侧）：
- 子件数 `1..=64`；名称非空；`component_def_id` 必须存在于目录；
- 基本体 `shape ∈ {box, cylinder, sphere, cone}` 且尺寸 > 0；
- 可选 `parent_id` 必须存在；
- 未设置 `skip_series_gate` 时应用系列门禁（与目录实例化同一可达性规则）。

执行：
1. 先插入容器对象（类型 `Assembly`）；
2. 逐个创建子件（目录辅助函数 / 基本体网格），设置 `parent_id`、添加 containment 边；
3. 记录**一条** `Command::CreateBatch { snapshots, sketches }`，覆盖容器、全部子件与实例化产生的草图 → 单步撤销；
4. 任一失败：清除本次已创建的对象/草图并返回 `success: false`（不留下半成品）。

`created_ids` = 容器在前，子件按请求顺序。

## 3. 删除语义

- `DELETE /api/objects/{id}?cascade_children=true`（UI 对容器自动携带）→ `DELETE_OBJECT_SUBTREE` 动作：删除全部后代，快照 containment/connection 边，剥离 lifecycle/breakdown 引用，**一次撤销**恢复整棵子树（`DeleteCascade` 增加 `deleted_children`）。
- 不带标志时保持既有"子件提升"行为，删除 Site/Building 不会连带丢失内容。
- 仍引用对象的楼层删除继续被拒绝。

## 4. 变换语义

对带子件的装配容器调用 `PUT /api/objects/{id}/transform` 时，同一矩阵（平移/旋转/缩放，pivot 语义不变）作用于每个**直接子件**的几何，并记录一条 `Command::TransformAssembly` 覆盖全部子件（`Command::Batch` 包裹 `ModifyObject`）——一次撤销恢复所有子件。旧合并网格装配（无子件）保持叶子对象行为。

## 5. IFC 导出

子件是普通对象，按各自映射导出（Wall → `IfcWall`、Component → proxy…）。容器沿用现有回退（`IfcBuildingElementProxy`），携带 Name/Description；专用 `IfcElementAssembly` 映射属后续工作。

## 6. UI 流程（AssemblyPanel）

1. 命名装配体；可选父级（复用现有父级选择器）。
2. 添加子件，二选一：
   - **目录构件** — 选目录构件、编辑参数（mm）、设置位置（m）与 Z 旋转（度）；或
   - **基本体** — box/cylinder/sphere/cone + 尺寸/偏移（m）。
3. 列表显示来源徽标、尺寸与位姿；空名称/零尺寸基本体被拒绝。
4. "创建 Assembly" 调 `POST /api/assemblies`；子件出现在对象树容器下；删除容器移除整棵子树。

`POST /api/genrule` 的 `type: "assembly"` 作为**遗留**合并网格路径保留，供脚本/向后兼容调用与内建设备类型（`Stair`、`AHU`、`Chiller`、`Transformer`、`Reactor`）复用 `AssemblyGen`。

## 7. 范围外（已登记）

- 逐子件 live placement 字段与子件非破坏移动。
- 基于 `catalog_ref` 子件的 BOM/工程量端点。
- 运动副 / 约束装配（ROADMAP Phase 8）。
- 嵌套装配与子件替换工作流。
