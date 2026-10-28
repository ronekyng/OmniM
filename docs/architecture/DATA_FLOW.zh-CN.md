---
title: "OmniM — 数据流"
status: current
last_updated: 2026-10-27
applies_to: "crates/, web/"
evidence: "audited against crates/server/src/lib.rs 2026-10-27"
---
# OmniM — 数据流

本文描述**当前代码中真实存在的端到端数据流**,依据 `crates/server/src/lib.rs`、
`crates/omcom/src/lib.rs`、`crates/amir/src/generator.rs` 与 `crates/ifc/` 编写。
凡仅处于规划中的机制,均会显式标注**规划中**。

架构总览见 [PLATFORM_ARCH.md](PLATFORM_ARCH.zh-CN.md);本页聚焦**数据如何流动**。

---

## 0. 通用变更规则 —— 这是目标,不是现状

**目标**(ADR-0006)是:**对 `Project` 的每一次写入——无论是鼠标点击、AI 回复还是 REST 调用——
都会变成一条 `OmniCommand`**,然后经过:

```
OmniCommand ──▶ Validator::validate(cmd, &Project)──▶ (收集错误;有错即中止)
            ──▶ Executor::execute(cmd, &mut Project) ─▶ ExecutionResult + 撤销/重做条目
```

**当前代码已符合（2026-10-28）。** 所有变更 handler 都构造 `OmniCommand`（或委托 `Executor`）；
直写端点 42 → 0，每次变更原子落盘，`scripts/check_write_paths.py` 强制执行该不变量。
逐端点矩阵与两处已记录例外（`import_ifc` 单条整项目命令、`.omnimlib` 导入全局库）见
[WRITE_PATHS.zh-CN.md](WRITE_PATHS.zh-CN.md)。

---

## 1. (a) 用户点击 → REST → Executor → 模型变更 → 网格 → three.js 渲染

最常见的流程,比如在工具栏点击"创建墙体":

```
┌─ 浏览器 ───────────────────────────────────────────────────────┐
│ React 组件(如 MainToolbar.tsx)                                 │
│   │ 用户点击                                                    │
│   ▼                                                             │
│ 类型化 REST 客户端 (shared/api.ts) → POST /api/objects  (JSON)   │
│   … Zustand store / React Query 使受影响的查询失效               │
└──────────────────────────────┬──────────────────────────────────┘
                               │ HTTP /api/*
┌──────────────────────────────▼──────────────────────────────────┐
│ omnim-server (Axum)                                             │
│   • 处理器获取 project.write() + executor.write()               │
│   • 就地拼装 OmniCommand{ action, operations:[…] }              │
│   • Executor::execute() → 校验 → 修改 Project                   │
│   • 保存：mutation::persist 原子落盘（save_path 为 Some 时）      │
│   • 返回对象/JSON(或错误状态码)                                 │
└──────────────────────────────┬──────────────────────────────────┘
                               │ 200 OK
       (回到浏览器)             │
                               ▼
      React Query 重新拉取对象树 → 拉取网格
                               │
┌──────────────────────────────▼──────────────────────────────────┐
│ omnim-model                                                      │
│   geometry.rs / brep.rs 用 truck 内核构建实体                     │
│   mesh_gen.rs 剖分 → 三角网格(位置/法线)                         │
└──────────────────────────────┬──────────────────────────────────┘
                               │
┌──────────────────────────────▼──────────────────────────────────┐
│ three.js 渲染器 (web/src/app/components/ThreeViewer.tsx)         │
│   载入网格与对象变换,更新场景图并渲染                             │
└─────────────────────────────────────────────────────────────────┘
```

代码层面的事实:

- **网格是从服务器反复拉取的** —— 几何重建发生在 Rust 侧(`omnim-model` 的 mesh_gen.rs),
  浏览器是纯粹的展示客户端。
- **没有推送通道**:每次变更后 UI 重新拉取状态(调用 resolve 后使相关 query 失效)。
  代码库中不存在 WebSocket。
- 本图只适用于 executor 路径的端点。改名、移动、改父级、连接、赋材质、草图更新/求解、IFC 导入、
  目录实例化、系列编辑现在都走命令(可撤销、会落盘)——见 [WRITE_PATHS.zh-CN.md](WRITE_PATHS.zh-CN.md)。
  IFC 导入是唯一的整项目命令(`Command::ReplaceProject`)。

### Mermaid 版本

```mermaid
sequenceDiagram
    participant U as UI (React)
    participant S as omnim-server (Axum)
    participant O as omnim-omcom
    participant M as omnim-model
    participant R as three.js
    U->>S: POST /api/objects (JSON)
    S->>O: OmniCommand → validate → execute
    O->>M: 修改 Project
    O-->>S: ExecutionResult
    Note over S,M: 快照 → 写入 → 记录 ReplaceProject → 落盘
    S-->>U: 200 JSON
    U->>M: GET /api/mesh /api/tree
    M-->>U: 网格 + 对象树 JSON
    U->>R: 场景更新 + 渲染
```

---

## 2. (b) 自然语言 → AMIR 引擎(LLM 供应商)→ OmniCommand → Validator → Executor

```mermaid
sequenceDiagram
    participant U as AI 对话 (AiChat.tsx)
    participant S as omnim-server /api/ai/*
    participant A as omnim-amir
    participant L as LLM 供应商 (OpenAI/Anthropic/Mock)
    participant O as omnim-omcom
    U->>S: POST 用户消息
    S->>A: 上下文 + 意图请求
    A->>L: HTTPS (reqwest) 对话补全
    L-->>A: 结构化意图 / 命令草稿
    A->>A: generator.rs: 意图 → OmniCommand
    A-->>S: OmniCommand 列表
    S->>O: Validator::validate + Executor::execute
    O-->>S: ExecutionResult(错误经 feedback.rs 反馈给 AI 重试)
    S-->>U: 结果 + 面向用户的摘要
```

代码中实际发生的事:

1. **上下文组装**(`amir::context`):对相关 `Project` 状态(选中对象、目录信息、最近历史)
   做快照,让 LLM 的回答言之有据。
2. **供应商抽象**(`amir::lib`):由配置值 `"openai" | "anthropic"` 选择供应商,
   另有 `Mock` 供应商供离线/测试使用。通信就是 `reqwest` 发起的**普通 HTTPS** ——
   没有 SDK,没有本地模型运行时。
3. **意图 → 命令**(`amir/src/generator.rs`):这份大文件用规则 + 模板把意图映射到具体的
   `OmniCommand` 载荷;`prompt_parser.rs` 处理更松散的自由文本,`templates.rs` 存放响应模板,
   `feedback.rs` 把 Executor 的错误转回可重试的 AI 反馈。
4. **同一道门:** 生成的命令**不会**直接写 `Project`,而是和用户点击走同一个 Validator/Executor
   (见 §0)。这是关键的安全属性 —— 幻觉出来的命令会大张旗鼓地失败,而不是悄然腐蚀状态。
5. **反向描述**(`amir::reverse`):引擎也能把已有 `Project` 描述成自然语言
   (属于摘要/文档生成,不是变更)。

> 上节代码中的 `generator.ts` / `lib` 之类的标注指 `amir/src/` 下相应模块;
> 本项目并未使用 LLM SDK,也绝非仅在浏览器里跑模型。

---

## 3. (c) IFC 导入 / 导出

```mermaid
sequenceDiagram
    participant U as Web UI (库管理 / 导入操作)
    participant S as omnim-server (导入/导出)
    participant I as omnim-ifc (手写 STEP 解析器)
    participant M as omnim-model
    Note over I,M: 导出路径
    U->>S: GET /api/export/ifc
    S->>M: 读取当前 Project
    M-->>S: 对象/几何
    S->>I: 将 Project 序列化为 STEP 文本
    I-->>S: IFC 2x3 / IFC 4 子集文本
    S-->>U: 文件下载

    Note over I,M: 导入路径
    U->>S: POST /api/import/ifc(原始 IFC 文本)
    S->>I: 解析文本(手写,不依赖 IFC crate)
    I-->>S: 解析出的 IFC 记录 + 映射的几何 (geometry.rs)
    S->>M: 把解析出的对象/材料/生命周期直接插入 Project
    M-->>S: Project 被替换(单步撤销,已落盘)
    S-->>U: 导入得到的对象
```

代码事实:

- `omnim-ifc` 包含**手写的解析器与序列化器**(没有使用任何 IFC 生态的 Rust crate —— ADR-0005)。
- 覆盖范围**是局部的** —— 仅覆盖 **IFC 2x3 与 IFC 4** 的一个子集;不支持的实体被跳过,
  而不是整体报错。
- **导入是单条整项目事务。** `import_ifc` 插入解析出的对象/材料/生命周期记录
  (`mode=replace` 时整体覆盖),并记录 `Command::ReplaceProject { before, after }`;undo 恢复导入前项目,
  redo 重放,结果会落盘。见 [WRITE_PATHS.zh-CN.md](WRITE_PATHS.zh-CN.md)。
- 请求体是**原始 IFC 文本**,不是 multipart —— 用 `curl --data-binary @file.ifc`,不是 `curl -F`。
- `fixtures/ifc/` 下的示例 IFC 文件(`simple.ifc`, `test_building.ifc`, `part.ifc` 等)
  就是用来跑通这条流的。
---

## 4. (d) 目录 / 系列实例化

PARAGON 式**组件目录**与 SPECON 式**系列库**位于 `omnim-model`
(`catalog.rs`, `seed_catalog.rs`, `series_store.rs`, `library.rs`)。目录定义由库/种子目录提供;
`Project` 只存系列(`Project.series`)与外部系列引用,**没有** `catalog` 字段。

```mermaid
flowchart TD
    A[种子目录 / 库<br/>seed_catalog.rs] -->|提供给 UI| U[Web UI 目录与系列面板]
    U -->|POST /api/catalog/instantiate| S[omnim-server]
    S -->|InstantiateCatalog 命令<br/>executor + 撤销 + 落盘| M[omnim-model::Project]
    M --> F[特性树 / feature_template.rs<br/>官方几何路径]
    F --> G[geometry.rs / brep.rs → mesh_gen.rs]
    G --> J[(网格 JSON 交给 three.js)]
```

叙述:

1. **目录条目:** 每个目录组件声明一套参数 schema 与默认参数(PARAGON 式参数驱动的零件定义)。
2. **系列:** 一个系列把一个目录组件绑定到一组带尺寸的变体(SPECON 式),用户选择的是
   **系列 + 规格**,而非裸尺寸。
3. **实例化:** `POST /api/catalog/instantiate` 作为 `InstantiateCatalog` 命令经 Executor 执行:
   创建的对象与模板生成的草图被记录在同一条命令里,撤销/重做与落盘都是原子的。
   它创建的仍是 `ObjectType::Component`(不是 `Wall`/`Beam`/…),
   参数取自所选系列记录。
4. **生成:** 新对象的特性树(`GenRule` 生成器,`feature_template.rs`)驱动几何构建
   (`brep.rs`,走 truck 内核)与网格剖分(`mesh_gen.rs`),最终交给 three.js 查看器。

**几何路径分裂(重要)。** 目录特征树 → BRep/网格 是**官方几何路径**。工具栏的 `/api/genrule`
是**遗留/过渡直通路径**:它生成硬编码网格图元,并让 `param_values` 保持为空。因此同一堵墙可能以
`Wall`(genrule)、`Component`(目录)或 IFC 映射类型(导入)出现,撤销/保存行为各不相同。
见 [WRITE_PATHS.zh-CN.md](WRITE_PATHS.zh-CN.md) 与
[CODE_DEVIATIONS.zh-CN.md §6](../status/CODE_DEVIATIONS.zh-CN.md#6-写入路径持久化与几何路径偏差--known2026-10-27)。

---

## 5. (e) `.omnim` JSON 文件的保存 / 加载

```mermaid
flowchart LR
    subgraph 启动
        A[CLI: cargo run / release 二进制<br/>main.rs] -->|读取 OMNIM_PROJECT<br/>默认 project.omnim| B{文件存在?}
        B -->|是| C[Project::load → 反序列化 JSON]
        B -->|否| D[新建空 Project + 预置材料]
    end
    C --> E[SharedProject: Arc RwLock Project]
    D --> E
    E --> F[omnim-server 处理所有 /api 请求]
    F -->|默认 project.omnim 时 save_path=None| X[不持久化任何内容]
    F -->|仅 5 个 handler，且 save_path 为 Some| G[重写 .omnim JSON 文件]
```

机制细节:

- **一进程一文件。** `omnim_server::ServerConfig { project, save_path, ... }`。`main.rs` 在项目路径
  恰好为 `project.omnim` 时把 `save_path` 设为 `None`;只有非默认 `OMNIM_PROJECT` 才得到 `Some`。
- **即使开启也只保存部分变更。** 只有 `update_object`、`add_feature`、`update_feature`、`remove_feature`、
  `rebuild_component` 调用 `Project::save(path)`。创建对象、`/api/genrule`、`/api/omcom`、目录实例化、
  IFC 导入、系列编辑都不会保存。见 [WRITE_PATHS.zh-CN.md](WRITE_PATHS.zh-CN.md)。
- **整体重写,非增量。** 每次保存把**整个 `Project`** 序列化成 JSON 并重写文件。
  没有日志(journal)、没有 WAL、没有检查点。
- **崩溃窗口:** 若进程在"变更成功"与"成功写盘"之间被杀,文件会落后甚至(理论上)截断;
  目前没有原子重命名的落盘模式。
- **无历史。** 一个文件 = 一份快照。撤销/重做只存在于内存中的 `Executor` 状态里。
- **体量风险:** 面对非常大的项目(大规模的 `seed_catalog` 展开),单体文件的整体
  序列化天然是 O(项目大小)/每次变更。这正是规划中数据库持久化方案要解决的问题
  (见 [PLATFORM_ARCH.zh-CN.md §9](PLATFORM_ARCH.zh-CN.md#9-未来方向尚未实现))。

---

## 6. 不存在的流程

为避免误会,以下有人容易默认存在的路径**并未实现**:

- **没有 WebSocket / SSE / 推送。** 浏览器以 React Query 重新拉取替代。
- **没有 LLM 流式输出。** AMIR 的 LLM 回复是整体接收后做命令校验。
- **没有服务端预览渲染** —— 既没有服务端栅格化图片,也没有服务端 GPU 计时所图。
- **没有多文件工程模型**,也没有按对象的文件持久化。只有一个 `.omnim`。
- **没有工作区/分支**意义上的多版本管理(仓库的 git 分支与产品无关)。
