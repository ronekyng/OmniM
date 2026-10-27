# 快速开始

[English](GETTING_STARTED.md) | [简体中文](GETTING_STARTED.zh-CN.md)

本指南带你从克隆仓库到可交互的 OmniM 会话，大约五分钟；同时告诉你接下来去哪里探索。

## 前置条件

| 工具 | 要求 | 说明 |
| --- | --- | --- |
| Rust 工具链 | 任意较新 stable（`rustc` 1.8x+ 即可；工作区使用 **edition 2021**） | 通过 [rustup](https://rustup.rs) 安装。用 `rustc --version` 检查。 |
| Node.js | 建议 **≥ 20**；本机 CI 环境运行 Node 24 | `scripts/dev-web.sh` 在有 nvm 时优先 `nvm use 24`，但任何较新的 LTS 均可。`web/package.json` 中未锁定 `engines` 字段。 |
| npm | 随 Node 安装 | 仅 Web UI 需要。 |
| 可选：`protoc` | 无需单独安装 | `crates/core/build.rs` 使用 `prost-build`，自带 protobuf 代码生成。 |

无需数据库，无需外部服务。服务器把数据持久化到普通 JSON 文件。

## 五分钟快速上手

### 1. 构建并启动服务器

服务器二进制为 **`omnim`**（crate 名 `omnim-server`，位于 `crates/server`）。它启动 Axum HTTP 服务，监听 `0.0.0.0:3000`。

```bash
cargo run -p omnim-server --bin omnim
```

启动时会：

- 如存在则从 `~/.omnim/library.omnimlib` 加载元件库（否则回退到内置种子目录）；
- 从 `./project.omnim`（或 `$OMNIM_PROJECT`）加载项目，文件不存在时新建名为 "OmniM Project" 的项目；
- 在 `/api/*` 下提供 REST 接口。

> 注意：有时习惯在二进制后输入的 `serve` 参数（即 `omnim serve`）目前会被忽略——直接运行 `omnim` 效果相同。

### 2. 启动 Web UI（第二个终端）

```bash
cd web
npm install
npm run dev
```

Vite 开发服务器监听 **http://localhost:5173**，并把 `/api/*` 代理到 `http://localhost:3000`，因此 UI 无需任何 CORS 配置即可与第一步的服务器通信。

用浏览器打开 <http://localhost:5173>。（也可以用 `scripts/dev-server.sh` 和 `scripts/dev-web.sh` 包装这两步——它们具体做什么见 [BUILDING.md](BUILDING.md)。）

### 3. 初步交互

界面加载后，按任意顺序尝试：

- **通过 UI 创建对象。** 用主工具栏 / 创建菜单添加一个组件或对象。它会出现在左侧对象树面板中；若有几何体，还会出现在 3D 视图中。点击树节点可在属性面板查看其属性。
- **试用 Omcom 控制台。** 打开底部命令/控制台面板，输入一条简单命令，例如：

  ```text
  CREATE Object name:"Wall-01"
  ```

  然后试试 `UNDO` 回滚。Omcom 是统一的建模 DSL——UI 与 AI 的一切操作最终都会变成 Omcom 指令。也可以直接打 HTTP 接口：

  ```bash
  curl -X POST localhost:3000/api/omcom -H "Content-Type: application/json" \
    -d '{"command": "UNDO"}'
  ```

- **试用 AI 聊天（免密钥 mock 供应商）。** AMIR 是 AI 指令层（`crates/amir`）。不做任何配置时，它会运行在**mock 供应商**上，无需 API key——打开 AI 聊天面板（`web/src/app/components/AiChat.tsx`），输入类似 *"create a wall named W1"* 的内容。真实使用 OpenAI / Anthropic 需要环境变量密钥——见 [BUILDING.md](BUILDING.md)。
- **导入 IFC 文件。** `fixtures/ifc/` 目录下有几个 `.ifc` 测试文件：`simple.ifc`、`part.ifc`、`part40.ifc`、`test.ifc`、`test_building.ifc`。用 UI 的导入功能，或者：

  ```bash
  curl -X POST localhost:3000/api/import/ifc -F "file=@../../fixtures/ifc/simple.ifc"
  ```

  然后查看 `/api/tree` 或对象树面板，确认导入结果。

## 接下来去哪里

- [BUILDING.md](BUILDING.md) — 工作区布局、环境变量、故障排查。
- [ARCHITECTURE_MAP.md](ARCHITECTURE_MAP.md) — 各 crate 的职责、依赖关系图，以及"要加 X 该改哪里"的路线。
- [TESTING.md](TESTING.md) — Rust 单元测试、服务器 API 集成测试、Web Vitest、`scripts/test-all.sh`。
- `docs/architecture/PLATFORM_ARCH.md` — 长篇平台架构文档。
- `README.md` — 项目愿景、三大核心层（元件库、系列库、Omcom）与路线图。

## 停止服务

- 在各终端按 Ctrl-C，或：

```bash
scripts/stop-server.sh    # 释放 3000 端口（使用 fuser/lsof）
```
