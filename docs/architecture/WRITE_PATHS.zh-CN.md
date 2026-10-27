---
title: "OmniM 写入路径 — 静态扫描快照（2026-10-27）"
status: current
last_updated: 2026-10-27
applies_to: "crates/, web/"
evidence: "audited against crates/server/src/lib.rs 2026-10-27"
---
# OmniM 写入路径 — 静态扫描快照（2026-10-27）

**快照日期：** 2026-10-27
**依据：** 对 `crates/server/src/lib.rs` 中 `async fn` handler 函数体的静态扫描（检查 `.project.write()`、`executor`、`.save(`），并与路由表交叉核对。数字复现自 [2026-10-27 文档审计](../status/DOC_AUDIT_2026-10-27.zh-CN.md)附录 A。

> **决策背景。** [ADR-0006](DECISIONS.zh-CN.md) 仍然是**已接受的目标架构**：所有变更都应构造 `OmniCommand`，经 `Validator` + `Executor`，并进入撤销栈。当前代码**不满足**该目标。本页记录差距，不是新决策。
>
> **几何路径。** 官方几何路径是**特征树 / BRep** 管线（`/api/catalog/instantiate`、`/api/components/*`）。`/api/genrule` 直接生成硬编码网格图元的直通路径属于**遗留/过渡**方案，尽管工具栏仍可触达。

| 指标 | 数量 |
|---|---|
| `/api/*` 路由（去重后） | **78** |
| 持有 `project.write()` 的 handler | **55** |
| 其中函数体出现 `executor` 的 | **13** |
| 其中调用 `.save(` 的 | **5** |
| 直接写 Project、无 executor 的 handler（约） | **~42** |

> **方法说明。** 这是**静态文本扫描**，不是运行时追踪。它可能把只读但获取写锁的 handler 计入（如 `section_handler`），也可能漏掉通过 helper 间接保存/记录撤销的路径。55/13/5 的量级才是证据；逐行条目仅作示例。代码变更后需重新扫描。

---

## (a) Executor + Validator + 撤销路径

这些 handler 构造 `OmniCommand`，经 `Validator` + `Executor` 执行，因此可撤销。**它们都不保存项目文件。**

| 端点 | Handler | 撤销 | 保存 |
|---|---|---|---|
| `POST /api/objects` | `create_object` | ✅ | ❌ |
| `DELETE /api/objects/{id}` | `delete_object` | ✅ | ❌ |
| `POST /api/genrule` | `genrule_create` | ✅ | ❌ |
| `POST /api/omcom` | `execute_omcom` | ✅ | ❌ |
| `POST /api/undo` | `undo` | 历史操作 | ❌ |
| `POST /api/redo` | `redo` | 历史操作 | ❌ |
| `POST /api/sketches`、`DELETE /api/sketches/{id}` | `create_sketch`、`delete_sketch` | ✅ | ❌ |
| `POST /api/sketches/{id}/entities`、`POST …/constraints`、`DELETE …/entities/{idx}`、`DELETE …/constraints/{idx}` | `add_sketch_entity`、`add_sketch_constraint`、`delete_sketch_entity`、`remove_sketch_constraint` | ✅ | ❌ |

说明：

- `POST /api/omcom` 接收完整 `OmniCommand` 信封，**不**接受 `{"command":"UNDO"}`；撤销是 `POST /api/undo`。
- `POST /api/genrule` 仍在生成硬编码网格图元（`build_wall` / `build_slab` / `build_road` / `build_pipe`，且 `param_values` 为空），没有驱动目录特征树。它是遗留/过渡路径。

## (b) 直接写 `Project` —— 无撤销

这些 handler 获取 `state.project.write()` 并直接修改聚合根，**不会**产生撤销条目。

| 端点 | Handler | 撤销 | 保存 |
|---|---|---|---|
| `PUT /api/objects/{id}`（改名/描述/标签） | `update_object` | ❌ | ✅ |
| `PUT /api/objects/{id}/prop` | `set_property` | ❌ | ❌ |
| `PUT /api/objects/{id}/properties` | `update_properties` | ❌ | ❌ |
| `PUT /api/objects/{id}/transform` | `transform_object` | ❌ | ❌ |
| `PUT /api/objects/{id}/parent` | `set_parent` | ❌ | ❌ |
| `POST /api/objects/{from}/connect/{to}` | `connect` | ❌ | ❌ |
| `POST /api/materials` | `create_material` | ❌ | ❌ |
| `PUT /api/objects/{id}/material` | `assign_material` | ❌ | ❌ |
| `POST /api/measurements`、`DELETE /api/measurements/{id}` | `create_measurement`、`delete_measurement` | ❌ | ❌ |
| `POST /api/lifecycle`、`POST /api/lifecycle/{id}/link/{obj_id}` | `create_lifecycle`、`link_record` | ❌ | ❌ |
| `POST /api/administrative`、`DELETE /api/administrative/{id}` | `create_administrative`、`delete_administrative` | ❌ | ❌ |
| `POST /api/breakdown`、`DELETE /api/breakdown/{id}`、`POST …/link-obj`、`POST …/link-admin` | `create_breakdown`、`delete_breakdown`、`link_breakdown_object`、`link_breakdown_admin` | ❌ | ❌ |
| `PUT /api/sketches/{id}`、`POST /api/sketches/{id}/solve`、`PUT /api/sketches/{id}/entities/{idx}` | `update_sketch`、`solve_sketch`、`update_sketch_entity` | ❌ | ❌ |
| `POST /api/sketches/{id}/extrude`、`POST /api/sketches/{id}/revolve` | `extrude_sketch_handler`、`revolve_sketch_handler` | ❌ | ❌ |
| `POST /api/components`、`DELETE /api/components/{id}` | `create_component_handler`、`delete_component` | ❌ | ❌ |
| `POST /api/components/{id}/features` | `add_feature_handler` | ❌ | ✅ |
| `PUT /api/components/{id}/features/{fid}` | `update_feature_handler` | ❌ | ✅ |
| `DELETE /api/components/{id}/features/{fid}` | `remove_feature_handler` | ❌ | ✅ |
| `POST /api/components/{id}/rebuild` | `rebuild_component_handler` | ❌ | ✅ |
| `POST /api/import/ifc` | `import_ifc` | ❌ | ❌ |
| `POST /api/catalog/instantiate` | `instantiate_from_catalog` | ❌ | ❌ |
| `POST /api/series`、`PUT /api/series/{id}`、`DELETE /api/series/{id}`、`POST …/selectors`、`POST …/answers`、`POST …/spec`、`POST …/constraints`、`PUT …/constraints/{idx}`、`DELETE …/constraints/{idx}` | `create_series_handler`、`update_series_handler`、`delete_series_handler`、`add_selector_handler`、`add_answer_handler`、`add_spec_component_handler`、`add_param_constraint_handler`、`update_param_constraint_handler`、`delete_param_constraint_handler` | ❌ | ❌ |

`POST /api/import/ifc` 值得单独强调：它解析原始 IFC 文本，并把得到的对象/材料/生命周期记录直接插入 `Project`（`mode=replace` 时整体覆盖项目）。它**不**转换成 `OmniCommand`，因此导入无法撤销，也不会保存。见 [ADR-0005](DECISIONS.zh-CN.md#adr-0005-手写-ifc-解析器而非-ifc-crate) 的实现说明。

## (c) 保存行为

只有**同时**满足两个条件时 handler 才落盘：

1. `state.save_path` 为 `Some` —— 仅当 `OMNIM_PROJECT` 被设为字面量 `project.omnim` 以外的值（`crates/server/src/main.rs`）。
2. handler 属于调用 `Project::save(...)` 的五个之一：
   - `update_object`
   - `add_feature_handler`
   - `update_feature_handler`
   - `remove_feature_handler`
   - `rebuild_component_handler`

(a)(b) 两组中的其他所有端点都只留在内存。默认启动时完全没有任何持久化。`scripts/dev-server.sh` 会透传 `OMNIM_PROJECT`，但它的默认值（`project.omnim`）仍映射为 `save_path = None`；需导出非默认路径才能开启保存。

| 用户动作 | 撤销 | 默认是否保存 |
|---|---|---|
| 创建对象 / genrule / `/api/omcom` | ✅ | ❌ |
| 对象改名 | ❌ | ✅（仅非默认 `OMNIM_PROJECT`） |
| 移动、改父级、连接、赋材质 | ❌ | ❌ |
| 特征添加/更新/删除/重建 | ❌ | ✅（仅非默认 `OMNIM_PROJECT`） |
| IFC 导入 | ❌ | ❌ |
| 目录实例化 | ❌ | ❌ |
| 系列编辑 | ❌ | ❌ |

---

## 恢复单一写入路径

目标仍是 ADR-0006：所有变更走同一条命令管线，具备撤销与明确的持久化策略。这是路线图中的跟踪项——请链接 [ROADMAP.zh-CN.md](../status/ROADMAP.zh-CN.md)，不要在本页重述计划。在其落地之前，这些偏差记录于 [CODE_DEVIATIONS.zh-CN.md §6](../status/CODE_DEVIATIONS.zh-CN.md#6-写入路径持久化与几何路径偏差--known2026-10-27)。
