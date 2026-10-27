# OmniM — 当前架构

> **范围说明:** 本文档描述 OmniM **当前代码库中真实落地的架构**。
> 设计阶段的云端优先愿景(多租户、共享云工作区、企业级生命周期服务)位于
> [`docs/specification/`](../specification/) 与 [`docs/RFC/`](../RFC/)。凡本文标注"规划中"或"尚未实现"的内容,
> 均**未在代码中存在**。

---

## 1. 概述与设计目标

OmniM 是一个 **AI 原生的工程生命周期平台**:它以**本地优先的桌面级应用形态**起步,
当前的实际交付形态是**一个 Web SPA + 一个 Rust REST 服务器**,而工程领域(BIM/PLM/CAD)
建模完全实现在 Rust workspace crate 中。

用户本次实现能够做到:

- 构建、编辑工程对象的**项目对象树**(属性、材料、标签、关系、测量);
- 用真实的**约束求解器**编写二维草图,并从草图生成轮廓;
- 用 BRep 内核建模实体,并生成用于显示的三角网格;
- 从 PARAGON 式参数驱动的**组件目录**与 SPECON 式**系列库**实例化零部件;
- 通过覆盖部分 **IFC 2x3 / IFC 4** 子集的 STEP 文件实现导入/导出;
- 用**自然语言驱动模型**,由 AI 意图引擎生成结构化命令。

代码中可见的、具体的设计目标:

1. **协议定义先行,线上传输用 JSON。** Protobuf(`.proto` 文件)只用于定义消息 schema,
   运行时全部走 **Serde JSON** —— 没有 gRPC,没有运行时 protobuf 编码(见 §6)。
2. **一套引擎,多端前端。** 引擎是一组纯 Rust crate;浏览器 UI 是其中一个消费方,
   原生 wgpu 查看器是另一个(独立的)消费方。
3. **所有变更走同一条门。** 不论是 REST 处理器还是 AI 生成的命令,最终都产生
   `OmniCommand`,由**同一个 `Validator` + `Executor`** 校验并执行,并支持完整的撤销/重做。
4. **不引入隐性基础设施。** 没有数据库、没有 WebSocket、没有消息队列。应用的全部状态
   就是一个在内存中的 `Project` 值,持久化为单个 JSON 文件。

---

## 2. 分层结构(实际实现)

```
┌────────────────────────────────────────────────────────────────────┐
│                        Web SPA  (web/, Vite 构建)                   │
│  React 19 + TypeScript · react-router v7 · Zustand · React Query    │
│  three.js 查看器 · 草图编辑器 · 目录/系列/库管理                       │
│  AI 对话 · Omcom 控制台        ── 类型化 REST 客户端 (shared/api.ts) │
└───────────────────────────────────┬────────────────────────────────┘
                        HTTP/JSON   │  /api/*   (Vite 开发代理 → :3000)
┌───────────────────────────────────▼────────────────────────────────┐
│                     omnim-server  (Axum)                            │
│  ~100 个 REST 路由 · AppState = Arc<RwLock<Project>>                │
│  构造 OmniCommand · 变更后自动保存 JSON 文件                          │
│  为 web/ 构建产物提供静态文件服务                                     │
└───────┬───────────────────┬────────────────────┬───────────────────┘
        │                   │                    │
┌───────▼───────┐   ┌───────▼────────┐   ┌───────▼────────┐
│ omnim-omcom   │   │  omnim-amir    │   │   omnim-ifc    │
│ Validator +   │   │  AI 意图引擎   │   │  手写的 STEP/  │
│ Executor,     │   │  (LLM HTTP)    │   │  IFC 解析器    │
│ 撤销/重做      │   │                │   │                │
└───────┬───────┘   └───────┬────────┘   └───────┬────────┘
        │                   │                    │
        └───────────────────▼────────────────────┘
                            │
┌───────────────────────────▼────────────────────────────────────────┐
│   omnim-model (Project 聚合根、目录/系列、特性树、                    │
│   truck BRep 内核、网格生成、生命周期/分解 等)                        │
│─── omnim-sketch (slvs 约束求解器、草图实体、轮廓)                     │
│─── omnim-core  (prost 生成的 proto 类型,Serde JSON)                 │
└────────────────────────────────────────────────────────────────────┘
        │
        ▼
┌────────────────────────────────────────────────────────────────────┐
│  持久化: 单个 .omnim JSON 文件                                      │
│  (默认 project.omnim;可用 OMNIM_PROJECT 覆盖)                       │
│  无数据库、无版本历史存储                                            │
└────────────────────────────────────────────────────────────────────┘

(omnim-renderer: 已于 2026-10-27 删除 —— 原生 wgpu/winit 查看器,
 无人依赖,作为孤儿 crate 移除;渲染统一由浏览器 three.js 承担)
```

---

## 3. Crate 架构与职责

workspace 在根 `Cargo.toml` 中声明所有 crate。依赖 DAG:

```
omnim-core
    ▲
omnim-sketch
    ▲
omnim-model
    ▲             │
    ├── omnim-omcom ──┐
    ├── omnim-ifc  ───┼──► omnim-server
    └── omnim-amir ───┘
            (sketch 也参与 server 的草图相关端点)

omnim-renderer: 2026-10-27 已删除(原本仅依赖 omnim-core + omnim-model,
                是未接入 server/web 的孤儿原生查看器)
```

| Crate | 大致代码行数(`src/`) | 职责 |
|---|---|---|
| `omnim-core` | 很小 | 由 `proto/omnim.proto` 和 `proto/omcom.proto` 经 prost 生成的 Rust 类型。类型仅用 Serde **JSON** 序列化;没有 gRPC、没有运行时 protobuf。作为所有 crate 共享的薄基础层。 |
| `omnim-sketch` | ~1,350 | 二维草图领域:草图实体(位于工作平面上的点、线、圆、弧)、轮廓(profil),以及基于 `slvs`(SolveSpace)库的约束求解(`solver.rs`)。 |
| `omnim-model` | **最大 crate(约 7,700)** | `Project` 聚合根与整个工程领域:对象树、属性(`property_def.rs`)、特性树 + `GenRule` 生成器(`feature.rs`, `feature_template.rs`)、PARAGON 式组件目录(`catalog.rs`, `seed_catalog.rs`)、SPECON 式系列库(`series_store.rs`, `library.rs`)、基于 truck 的 BRep 建模(`brep.rs`)、三角网格生成(`mesh_gen.rs`)、材料、标签、关系、测量、生命周期记录、行政(document/versioning 记录)、分解(product-breakdown)、以及 **JSON `.omnim` 文件持久化**(`project.rs`)。 |
| `omnim-omcom` | ~3,650 | 命令层。`Validator::validate(&OmniCommand, &Project)` 返回错误列表;`Executor::execute(&OmniCommand, &mut Project)` 返回 `ExecutionResult`,内含**完整的撤销/重做栈与事务分组**。项目的一切合法变更都是一条 `OmniCommand`。 |
| `omnim-ifc` | ~1,590 | **手写的** STEP/IFC 文件解析器(不依赖任何 IFC crate),支持导入与导出,**部分覆盖** IFC 2x3 与 IFC 4;含 IFC 几何映射。 |
| `omnim-amir` | ~2,300 | AI 意图引擎("AMIR"):自然语言 → 意图 → `OmniCommand` 生成(`generator.rs`,约 1,018 行)、提示词解析、上下文构建、**反向描述**(把已有模型描述成自然语言,`reverse.rs`)、模板、反馈。通过 reqwest 的 HTTP 调用 LLM 供应商:**OpenAI / Anthropic / Mock**。 |
| `omnim-server` | ~3,700 | Axum HTTP 服务器。`/api/*` 下约 100 个 REST 路由,`AppState` = `Arc<RwLock<Project>>` + `Arc<RwLock<Executor>>` + 保存路径。为请求构造 `OmniCommand`,变更后自动保存 JSON 项目文件,并为 `web/` 构建产物提供静态文件服务。默认监听 `0.0.0.0:3000`(可用 `OMNIM_HOST`/`OMNIM_PORT` 覆盖)。**没有 WebSocket、没有数据库。** |
*`omnim-renderer`(自包含的原生 wgpu/winit 查看器)已于 2026-10-27 作为无人依赖的死底料从 workspace 删除;如需原生查看器将以 RFC 形式重新立项。*

> 说明:`omnim-core` 提供清单类型(共享枚举、`Version` 等);具体的 `Project` 与其持久化
> 完全位于 `omnim-model` 中。

### 体量说明

上表行数是示意值(基于对 `crates/*/src/*.rs` 的最近一次统计);模型 crate 约占引擎代码
三分之一,这印证该项目的主心骨是**领域模型**而非基础设施。

---

## 4. 关键数据流:HTTP 请求 → 变更 → JSON 保存

以一条典型变更路径(如 `PUT /api/objects/:id` 重命名对象)为例:

```
浏览器(React) ──PUT /api/objects/{id}──▶  Axum 路由处理器
                                               │  获取 state.project.write()
                                               │  获取 state.executor.write()
                                               │  构造 OmniCommand{action, operations...}
                                               ▼
                                       omnim-omcom Executor::execute()
                                               │  先经 Validator 校验
                                               │  修改 omnim-model::Project
                                               │  记录撤销/重做条目
                                               ▼
                                     内存中的 Project 已更新
                                               ▼
                                若 state.save_path 为 Some:
                                   Project::save(path) → 重写 .omnim JSON 文件
                                               ▼
                                  返回 200/4xx JSON 响应给浏览器
```

处理器代码中的约定:

- 读路由获取 `state.project.read().await`;变更路由同时获取 `project.write()` 和 `executor.write()`。
- 大多数变更路由**不会**直接构造裸的操作对象;而是就地拼装 `OmniCommand`(来自 `omcom.proto` 的
  action 枚举 + 操作列表)再交给 Executor,以确保撤销栈对所有写入一致。
- 自动保存失败只会写 stderr,不会让请求失败(`save_path` 在使用默认工程文件时为 `None`,
  即运行中的服务器可以刻意不持久化)。

完整的端到端流程(点击→渲染、自然语言→命令、IFC 进出、目录实例化、保存/加载)见
[DATA_FLOW.md](DATA_FLOW.md)。

---

## 5. 渲染策略

OmniM 刻意拥有**两个互不共享代码的独立渲染器**:

| 渲染器 | 技术 | 用于何处 | 数据来源 |
|---|---|---|---|
| 浏览器 3D/2D 查看器 | `web/src/app/components/ThreeViewer.tsx`(约 1,750 行)中的 `three.js`(+ 视图立方体辅助),二维草图画布在 `web/src/sketch` | 主 Web SPA | 通过 REST 端点(如 `/api/mesh`、单对象 mesh 端点)拉取三角网格,纯客户端渲染 |
| 原生查看器 | —(2026-10-27 删除:`omnim-renderer` 作为孤儿 crate 被移除;重启方案走 RFC) | — | — |

两者的关键共性:**服务器只通过 REST 暴露几何;渲染完全在客户端、与服务器代码无关。**
两个渲染器都不消费服务端栅格化;不存在服务端 GPU 流水线。

---

## 6. Protobuf 仅作类型 Schema(线上用 JSON)

本项目的一个独特之处:`.proto` 文件是**消息 schema 的唯一权威来源**,但线上传输完全不使用
protobuf 二进制编码。

- `proto/omnim.proto` 与 `proto/omcom.proto` 定义了所有领域/命令类型。
- `omnim-core` 用 `prost` 生成 Rust 结构体并附带 Serde 派生。
- 客户端(TS)在 `web/src/shared/types.ts` 中镜像这些类型;REST 层始终用 **HTTP + JSON** 交换数据。
- 另外一部分 serde 派生的领域类型直接放在 `omnim-model`/`omnim-sketch`(不在 proto 中),
  主要是那些携带大量 Rust 原生数据、与 protobuf 枚举相互映射不适配的类型。

带来的效果:schema 演进与跨语言契约仍集中在一处 `.proto`,同时避免了 gRPC 编帧、
每种客户端语言都要跑代码生成、以及二进制排查工具等运维负担 —— 代价是负载更大
(JSON 相比二进制 protobuf),并且失去 proto3 的跨语言二进制互操作保证。

当真正的多用户/云后端到来时(见 §8),引入真实的 gRPC 或二进制传输层是自然演进 ——
类型体系已经就位。

---

## 7. 持久化模型

**当前:单个 JSON 文件 + 单个内存中的工程。**

- 构造 `omnim_server::ServerConfig { project, save_path, ... }` 决定是否开启自动保存。
- 变更调用 `Project::save(path)`,整体重写 `.omnim` 文件;启动时读回同一文件。
- `.omnim` 文件(仓库根 `project.omnim`, `fixtures/my-project.omnim`)就是落盘示例。

**这种模型的局限**(这些是现状,而非计划移除的东西):

- **单进程、单工程。** 只有一个 `Project`,两个浏览器标签页共享同一份内存状态。
- **无多写者安全性。** `Arc<RwLock<Project>>` 在**单进程内**正确地串行化写入,
  但两个服务器实例无法共享一个文件;文件不是事务性的,中途崩溃可能损坏。
- **无历史。** 撤销/重做栈只在内存;JSON 文件只存当前状态。
- **无分支/合并。** 不存在面向模型的"版本管理 Git"。

这些限制将 OmniM 当前约束在**本地/桌面级**部署形态上。

---

## 8. 安全 / 并发模型

- **并发:** 单一的 `tokio::sync::RwLock<Project>` 守护全部状态。代码避免在只读到
  持有段内做内可变写;变更路由获取 `write()` 并同时获取 `executor.write()` 以写入
  撤销历史。这正确,但完全是单进程的 —— 不是分布式锁,不提供跨服务器协调。
- **安全:** Axum 层**没有任何认证或授权** —— 能到达端口的请求都会被接受。这在本地开发/演示
  服务器上是有意为之;它被列为已知的、计划填补的空缺(见 §9)。
- **信任边界:** 由于 LLM 生成的命令与用户驱动的命令走**同一个** `Validator`,AI 路径无法
  绕过领域校验 —— 这是一个重要的结构安全性(见 ADR-0006)。
- **CORS / 出站流量:** 默认绑定 `0.0.0.0:3000`(个人机器上合理,但在现阶段会把 LAN 面暴露出来);
  `omnim-amir` 通过 `reqwest` 对外发起 HTTPS 调用 LLM 供应商。

---

## 9. 未来方向(尚未实现)

以下均为**设计意图**,当前代码中都不存在;由 `docs/specification/` 与 `docs/RFC/` 跟踪:

| 方向 | 状态 | 预期承载位置 |
|---|---|---|
| 云端 / 多用户工作区,实时协作编辑(WebSocket 或类似 CRDT 的同步) | **规划中** | `docs/specification/` |
| 数据库支撑的持久化,带版本历史 / 快照存储(替代或并存于 JSON 文件) | **规划中** | `docs/specification/` |
| 引擎 crate 的 WebAssembly 构建,使浏览器 SPA 可在进程内执行同一份 Rust 领域逻辑(不再依赖服务器往返) | **规划中** | `docs/RFC/` |
| 桌面打包(如 Tauri 外壳)与移动端应用 | **规划中** | `docs/RFC/` |
| 服务边界演进:按工作区划分服务器、服务间 gRPC | **规划中** | `docs/RFC/` |

今天阅读 `crates/` 与 `web/` 下的代码时,**不应**假定上述任何一项已经存在。
