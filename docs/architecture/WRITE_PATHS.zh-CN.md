---
title: "OmniM 写入路径 — 现状（2026-10-28）"
status: current
last_updated: 2026-10-28
applies_to: "crates/server, crates/omcom, web/"
evidence: "由 scripts/check_write_paths.py 与 cargo test --workspace 于 2026-10-28 验证"
---
# OmniM 写入路径 — 现状（2026-10-28）

[English](WRITE_PATHS.md)

**状态：单一写入路径已恢复（Wave 1，2026-10-28）。** 所有变更端点都构造
`OmniCommand`，经 `Validator` + `Executor` 执行，进入撤销/重做栈，并原子落盘。
下列数字可用 `python3 scripts/check_write_paths.py` 复现；端点级撤销覆盖位于
`crates/server/tests/write_paths.rs` 与 `crates/server/tests/instantiate_import.rs`。

> **决策背景。** [ADR-0006](DECISIONS.zh-CN.md) 已**实现**，不再只是目标：
> handler 调用 `mutation::apply`（或持锁后委托 `Executor`），不再就地修改 `Project`。
>
> **几何路径。** 正式路径是特征树管线。Wave 3（2026-10-28）起 genrule 的
> **layered** 族已收敛：路径驱动的 wall/slab/floor/roof 创建会实例化
> `PathSweep` 特征树，并产出**领域对象类型**（`Wall`，而非 `Component`）与
> `catalog_ref`。仍属过渡：矩形参数快速创建（`wall`/`slab` 无路径变体）、
> 屋顶两点坡向、以及目录中基于 extrude 的 pipe/road 模板。

| 指标 | 数值 |
|---|---|
| `/api/*` 路由（按路径去重） | **79** |
| `crates/server/src/lib.rs` 扫描到的 handler | **105** |
| 调用 `mutation::apply` 的 handler | **55** |
| 持锁但委托 `Executor` 的 handler | **7**（`create_object`、`delete_object`、`execute_omcom`、`genrule_create`、`undo`、`redo`、`import_ifc`） |
| 白名单外的直写 `project.write()` | **0** |
| `save_project` 之外的直连 `Project::save()` | **0** |
| 默认持久化 | **开启** —— `$OMNIM_PROJECT`（缺省 `project.omnim`）；`OMNIM_EPHEMERAL=1` 关闭 |

## 一次变更的流转

```
handler → mutation::apply(action, ops)
        → Validator::validate        （非法状态直接拒绝，不产生任何改动）
        → Executor::execute          （修改 Project，记录撤销命令）
        → mutation::persist          （save_path 为 Some 时原子落盘）
```

锁顺序固定：先 `project`，后 `executor`。`mutation.rs` 是唯一同时获取两把锁的地方；
handler 不允许手写这对锁。

## 已收编的端点族

以下全部可撤销，且默认落盘。

| 端点族 | 端点 | 撤销 | 保存 | 切片 |
|---|---|---|---|---|
| 对象 / 属性 / 材质 | 创建/删除/更新对象、设属性、批量属性、变换、改父级、连接、创建/赋材质 | ✅ | ✅ | 1–2 |
| 草图 | 创建/删除草图、图元增改删、约束增删、求解、extrude/revolve | ✅ | ✅ | 1–3 |
| 构件 | 创建/删除实例、特征增改删、重建 | ✅ | ✅ | 3 |
| 目录定义 | 定义创建/更新/删除、参数增改删、连接器添加、特征树绑定、发布 | ✅ | ✅ | 3 |
| 系列 | 系列创建/更新/删除、选择器、答案、规格构件、参数约束增改删、保存/加载 | ✅ | ✅ | 4 |
| 生命周期 | 创建记录、记录关联对象 | ✅ | ✅ | 4 |
| 测量 | 创建/删除 | ✅ | ✅ | 4 |
| 轴网基准（P8-S1） | `POST/GET /api/grids`、`PUT/DELETE /api/grids/{id}` | ✅ | ✅ | P8-S1 |
| 管理区 / 分解 | 区划创建/删除、节点创建/删除、关联对象、关联区划 | ✅ | ✅ | 4 |
| 目录实例化 | `/api/catalog/instantiate`（对象 + 模板生成的草图作为一条命令） | ✅ | ✅ | 5 |
| IFC 导入 | `/api/import/ifc`（整项目事务，merge 或 replace） | ✅ | ✅ | 5 |
| 显式保存 | `POST /api/project/save` | — | ✅ | 1 |

## 有意保留的例外

| 路径 | 为什么不是普通逐实体命令 | 撤销 | 保存 |
|---|---|---|---|
| `section_handler` | 取写锁只为在读取剖切线前刷新特征缓存，没有用户可见变更 | — | — |
| `import_ifc` | 批量导入整体替换聚合，记录为 `Command::ReplaceProject { before, after }`——单步撤销，显式的内存取舍 | ✅（单步） | ✅ |
| `POST /api/catalog/import`（`import_catalog_handler`） | 把 `.omnimlib` 导入**会话级全局 `LIBRARY`**，不进入 `Project`，因此没有项目级撤销条目 | ❌ | ❌（库文件，非项目） |
| `preview_catalog_component` | 在一次性 `Project` 上构建实例；不取写锁、不污染实际模型 | — | — |
| `/api/genrule` 几何 | 命令/撤销/落盘都正确，但几何仍是硬编码网格而非特征树驱动（Wave 3） | ✅ | ✅ |

## 持久化

- **默认开启**：`main.rs` 设 `save_path = Some(project_file)`，除非
  `OMNIM_EPHEMERAL=1|true|yes`（此时完全不写文件）。
- **原子写**：`Project::save` 先写 `{path}.tmp` 再 rename 覆盖目标；崩溃不会留下半截文件。
- **失败策略**：保存失败只记录日志（`Auto-save failed: ...`），不会让请求失败——
  内存变更与撤销条目此时已经成立。
- **显式保存**：`POST /api/project/save` 按需落盘并返回路径；持久化被禁用时返回 400。
- **对象基准字段（P8-S1）**：`elevation`/`level_id` 走常规创建/更新对象命令；
  `ModifyObject` 的撤销快照现覆盖**全部**可修改字段（这同时关闭了 Wave 1 的缺口——
  `ifc_class`/`tags` 编辑此前不在还原范围），redo 重放 after 镜像。`POST /api/objects`
  校验失败返回 **400 与校验理由**（此前是无提示的 500）。

## 验证

```bash
python3 scripts/check_write_paths.py   # 0 直写，退出码 0
cargo test --workspace                 # 含端点级撤销测试
python3 scripts/check_docs.py          # 链接/围栏/编号检查
```

## 历史

修复前的审计快照（55 个 handler 持写锁、13 个提及 executor、5 个保存、约 42 个直写）
保留在 [2026-10-27 审计报告](../status/DOC_AUDIT_2026-10-27.zh-CN.md) 与
[CODE_DEVIATIONS §6](../status/CODE_DEVIATIONS.zh-CN.md#6-写入路径持久化与几何路径偏差--known-2026-10-27)。
