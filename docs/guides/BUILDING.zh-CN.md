---
title: "构建 OmniM"
status: current
last_updated: 2026-10-27
applies_to: "repo"
evidence: "commands and paths verified in the repo 2026-10-27"
---
# 构建 OmniM

[English](BUILDING.md) | [简体中文](BUILDING.zh-CN.md)

工作区如何组织、如何构建与运行、出问题时怎么办。

## 工作区布局

一个 Cargo 工作区（`Cargo.toml`，resolver 2），`crates/` 下七个 crate，另有 `web/` 下的 TypeScript Web 应用和 `proto/` 下的 protobuf 定义。各 crate 的职责见 [ARCHITECTURE_MAP.md](ARCHITECTURE_MAP.zh-CN.md)。

```
OmniM/
├── Cargo.toml              # 工作区清单（edition 2021）
├── proto/                  # omnim.proto + omcom.proto（prost，派生 serde 类型）
├── crates/
│   ├── core/               # omnim-core — 共享枚举/领域原语 + proto 代码生成（build.rs）
│   ├── model/              # omnim-model — 统一数据模型、目录、库、持久化
│   ├── sketch/             # omnim-sketch — 草图实体 + 约束求解器
│   ├── omcom/              # omnim-omcom — 类型化命令 Validator/Executor（撤销/重做；无文本解析器）
│   ├── server/             # omnim-server — Axum REST API；二进制名 `omnim`
│   ├── ifc/                # omnim-ifc — IFC 导入导出
│   └── amir/               # AMIR AI 指令层（供应商：mock/openai/anthropic）
├── web/                    # React 19 + Vite + three.js 前端
├── scripts/                # 开发/测试辅助脚本（见下）
└── fixtures/ifc/           # IFC 测试文件
```

## 构建命令

```bash
cargo build                          # 全量 debug 构建
cargo build --release                # 优化构建
cargo build -p omnim-server          # 仅服务器 crate（二进制 omnim 在 target/debug）
cargo check                          # 快速类型检查，不做代码生成/链接

cd web && npm install && npm run build   # Web 生产构建 → web/dist/
```

首次构建最慢（wgpu、truck-* 内核、rustls 的 reqwest）。之后 `cargo check` 很快。

Web 开发服务器（`npm run dev`）在 :5173 提供服务，并把 `/api` 代理到 `http://localhost:3000`——见 `web/vite.config.ts`。生产配置（`npm run build`）使用相对 `./` 基路径。

## 运行与调试

### 服务器

```bash
cargo run -p omnim-server --bin omnim
```

环境变量（读取位置：`crates/server/src/main.rs` 与 `crates/amir/src/lib.rs`）：

| 变量 | 默认值 | 作用 |
| --- | --- | --- |
| `OMNIM_PROJECT` | `project.omnim` | 要加载的项目文件（不存在则创建）。仅当该值**不是**恰好 `project.omnim` 时才启用保存；即便如此也只有 5 个 handler 会落盘（见[持久化](#持久化)）。`cargo run` 时 CWD 是 **crate 目录**（`crates/server/`），指向仓库根目录的文件请用 `../project.omnim` 或绝对路径。 |
| `OMNIM_HOST` | `0.0.0.0` | 监听主机。 |
| `OMNIM_PORT` | `3000` | 监听端口。 |
| `AMIR_PROVIDER` | `mock` | LLM 供应商：`mock`（无需密钥）、`openai`、`anthropic`。 |
| `AMIR_API_KEY` | — | 所选供应商的 API key。 |
| `AMIR_MODEL` | 供应商默认 | 模型名覆盖。 |
| `AMIR_BASE_URL` | 供应商默认 | 自定义/代理 base URL（如自部署的 OpenAI 兼容 API）。 |
| `AMIR_TEMPERATURE` | `0.1` | 采样温度。 |
| `AMIR_MAX_TOKENS` | `1024` | 每次回复的最大 token。 |

供应商细节（精确的 env 回退）在 `crates/amir/src/llm/openai.rs` 与 `anthropic.rs`：未设置 `AMIR_API_KEY` 时，OpenAI 供应商回退到 `OPENAI_API_KEY`，Anthropic 回退到 `ANTHROPIC_API_KEY`。没有可用 key 时调用会失败——无密钥开发请坚持 `AMIR_PROVIDER=mock`。

数据文件：

- **元件库**：存在 `~/.omnim/library.omnimlib` 时用它，否则用内置种子目录（`crates/model/src/seed_catalog.rs`）。若该文件存在但解析失败，服务器打印警告并回退到种子默认值。
- **项目**：`$OMNIM_PROJECT`，读不到就新建。

### 持久化

持久化**默认关闭，即使开启也只覆盖部分端点**。`crates/server/src/main.rs` 与 `crates/server/src/lib.rs` 的实际行为是：

1. `project_file = OMNIM_PROJECT`，缺省为字面量 `project.omnim`。
2. 若 `project_file == "project.omnim"`，则 `save_path = None`；否则 `save_path = Some(project_file)`。
3. 只有 `state.save_path` 为 `Some` **且** handler 属于调用 `Project::save(...)` 的五个之一时才会落盘：`update_object`、`add_feature`、`update_feature`、`remove_feature`、`rebuild_component`。

由此可得：

- 直接 `cargo run -p omnim-server --bin omnim`（不带环境变量）永远不会持久化变更。
- `OMNIM_PROJECT=project.omnim` 仍被视为默认值，**不会**开启保存。
- `scripts/dev-server.sh` 会透传 `OMNIM_PROJECT`，但其默认值为 `project.omnim`——因此默认用该脚本启动同样不持久化。运行前导出其他路径（例如 `OMNIM_PROJECT=./dev-project.omnim scripts/dev-server.sh`）才能开启保存路径。
- 创建对象、`/api/genrule`、`/api/omcom`、目录实例化、IFC 导入、系列编辑**都不会**被保存，即使设置了非默认路径。逐端点状态见 [WRITE_PATHS.zh-CN.md](../architecture/WRITE_PATHS.zh-CN.md)。

调试要点：二进制后面输入的字面 `serve` 参数会被忽略。部分日志是朴素的 `eprintln!`；冷启动会通过 `crates/core/build.rs`（`OUT_DIR`）重新生成 proto 类型。

### Web

```bash
cd web
npm run dev        # :5173，/api 代理 → :3000
npm run build      # dist/
npm run preview    # 预览构建产物
npm run test       # Vitest 单次运行；`npm run test:watch` 为监听模式
```

环境变量包装示例：

```bash
OMNIM_PORT=3100 cargo run -p omnim-server --bin omnim &
cd web && npm run dev   # 将 web/vite.config.ts 中的代理目标改为 :3100
```

## 脚本

脚本在 `scripts/` 下，均以仓库根目录为参照（`cd "$(dirname "$0")/.."`）。

| 脚本 | 作用 | 注意事项 |
| --- | --- | --- |
| `dev-server.sh` | 杀掉占用 `:3000` 的进程（fuser/lsof），`cargo build`，然后以 `OMNIM_PROJECT` 启动 `./target/debug/omnim`。 | 检测到时同时打印 `localhost` 与一条 **WSL2 eth0 IP**——WSL URL 仅在 WSL2 之下有意义。端口随 `OMNIM_PORT` 变化。 |
| `dev-web.sh` | 加载 nvm（`use 24`），缺 `node_modules` 时 `npm install`，然后 `npm run dev`。 | Linux 下 `node` 不可用时退出并给出安装指引。服务端口 :5173。 |
| `stop-server.sh` | 杀掉 `OMNIM_PORT`（默认 3000）上的进程，用 `ss` 报告端口状态。 | 无；可重复执行。 |
| `test-all.sh` | 对整个工作区跑 `cargo test`，然后（有 node 时）在 `web/` 执行 `npm install && npm run build`。 | **不运行 Web 单元测试**——只做 Vite 构建。Web 测试需单独在 `web/` 跑 `npm run test`；见 [TESTING.md](TESTING.md)。 |

## 故障排查

- **3000 端口被占用。** 运行 `scripts/stop-server.sh`，或换端口：`OMNIM_PORT=3100 cargo run -p omnim-server --bin omnim`（再把 Vite 代理指向该端口）。最常见的原因是本项目自己的旧服务器实例——`dev-server.sh` 会替你杀掉它们。
- **AI 聊天没有调用真实 LLM。** 未配置环境变量时，`AmirConfig::from_env()` 把 `AMIR_PROVIDER` 默认为 `mock`，只返回固定结果。使用真实供应商需导出 `AMIR_PROVIDER=openai|anthropic` 加 `AMIR_API_KEY`（或 `OPENAI_API_KEY` / `ANTHROPIC_API_KEY`）。mock 回退是刻意设计，方便无密钥离线演示。
- **项目数据没有持久化/没出现。** 这通常是预期行为，除非同时满足两个条件：(1) `OMNIM_PROJECT` 是非默认路径（绝对路径或不是 `project.omnim` 的值）；(2) 你执行的操作属于五个会保存的 handler（`update_object`、添加/更新/删除特征、重建构件）。创建对象、genrule、`/api/omcom`、目录实例化、IFC 导入、系列编辑都不会保存。见[持久化](#持久化)与 [WRITE_PATHS.zh-CN.md](../architecture/WRITE_PATHS.zh-CN.md)。
- **`~/.omnim/library.omnimlib` 文件损坏。** 解析失败会记录警告并静默使用种子目录；删除或修复该文件即可消除警告。
- **WSL2 网络。** `dev-server.sh` 会打印 WSL2 eth0 IP；若 localhost 转发不生效，Windows 侧请使用该 URL。
- **过时的 `scripts/update_roadmap.py`。** 它是一次性遗留，不属于任何构建/测试流程。忽略即可。
- **Web 构建在 Node 上失败。** `dev-web.sh` 与 README 假设有 nvm；先 `nvm install --lts` 再重试。
