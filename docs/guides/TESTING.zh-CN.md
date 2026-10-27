# 测试指南

[English](TESTING.md) | [简体中文](TESTING.zh-CN.md)

如何在 OmniM 中运行和编写测试：Rust 单元 + 集成测试、服务器 API 集成套件、Web 单元测试。

## 概览

| 层 | 框架 | 命令 | 测试位置 |
| --- | --- | --- | --- |
| Rust（单 crate） | `cargo test` | `cargo test -p <crate>` | `crates/*/src` 内的 `#[cfg(test)] mod tests` |
| Rust（全工作区） | `cargo test` | `cargo test` | 所有 crate |
| 服务器 REST API | `cargo test` | `cargo test -p omnim-server` | `crates/server/tests/api_integration.rs` |
| IFC | `cargo test` | `cargo test -p omnim-ifc` | `crates/ifc/tests/ifc_roundtrip.rs` |
| Web | Vitest | `npm run test`（在 `web/` 下） | `web/src/__tests__/*.test.ts` |

`scripts/test-all.sh` 会运行工作区 `cargo test`，然后（有 Node 时）执行一次 **web 构建**（不是 web 测试）——注意这个差异，见下文。

## Rust 测试

### 各 crate 单元测试

大多数 crate 把单元测试一起放在被测文件里：

```bash
cargo test -p omnim-omcom        # crates/omcom/src/lib.rs 的 DSL 解析/执行测试
cargo test -p omnim-model        # model/store/catalog/material 测试
cargo test -p omnim-sketch       # 约束求解器测试（基于 slvs）
cargo test -p omnim-amir         # prompt 解析等；真实供应商网络测试缺 AMIR_API_KEY 时跳过
```

（工作区成员名。`omnim-amir` 的真实供应商测试会检查 `AMIR_API_KEY`，否则打印 "Skipping real provider test"——无密钥环境只会走 mock 供应商。）

### 整个工作区

```bash
cargo test
```

包含下述集成套件。首次运行编译所有内容；之后是增量运行。

### 服务器 API 集成测试

`crates/server/tests/api_integration.rs`（约 1,400 行）是最大的一组。它通过 `axum` + `tower` 在内存共享项目上启动服务器的 axum 路由（不走网络），端到端地驱动真实 REST 流程：对象 CRUD、树/统计、属性、变换、材料、测量、生命周期、管理层级、分解、omcom 执行、撤销重做、草图/求解/拉伸、目录与系列管理、mock 供应商下的 AMIR translate/chat、IFC 导入导出。IFC 测试文件从仓库根目录的 `.ifc` 文件读取。

```bash
cargo test -p omnim-server --test api_integration
cargo test -p omnim-server --test api_integration amir   # 按名称过滤
```

### IFC 往返测试

```bash
cargo test -p omnim-ifc --test ifc_roundtrip
```

几何经过 IFC 写出后再重新导入，比较结果的保真度。

### `.ifc` 测试文件

示例 IFC 文件在**仓库根目录**，被测试引用，也可通过 `/api/import/ifc` 手动使用：

- `simple.ifc` — 最小构件
- `part.ifc`、`part40.ifc` — 机械零件样本
- `test.ifc` — 通用冒烟测试文件
- `test_building.ifc` — 完整楼层的建筑样本，适合看对象树/层级

它们放在根目录（而非 `crates/ifc/tests/` 下），因为服务器集成测试以项目根目录为基准读取它们；新增测试文件时注意路径。

## Web 测试

使用 Vitest，在 `web/` 目录下执行：

```bash
cd web
npm run test          # = `vitest run`，非监听模式
npm run test:watch    # 或：npx vitest
```

现有测试在 `web/src/__tests__/`（`stateMachine.test.ts` 覆盖视图器所用的创建工具/交互状态机）。Vitest 默认匹配 `__tests__/*.test.ts` 之类的文件。

注意：**`scripts/test-all.sh` 不运行 Web 单元测试**——它只跑 `cargo test`，再对 web 执行 `npm install && npm run build`。要全部覆盖，需另加 `cd web && npm run test`。

## 已知不稳定测试（Flaky）

**`omnim-model` → `test_find_and_rebuild_catalog_instances`**（crates/model/src/project.rs）在一次全量并行 `cargo test --workspace` 运行中失败过一次（断言 `should find 2 wall instances`，实际得到 1），随后无法复现（压力验证：单库运行 15 次 + workspace 运行 6 次，全部通过）。在此记录，避免这一观察丢失。

- **根因（评估）：** 测试共享 session 级全局 `static LIBRARY: RwLock<Option<Library>>`（由 `setup_library()` 以 `call_once` 初始化）。cargo 在同一测试二进制内默认多线程并发跑测试，读取全局库的测试（如 `create_object_from_default_template` → `instantiate_catalog_component_impl`）会与读/写全局状态的其它测试竞争——`Once` 只保证"只初始化一次"，并不提供按测试隔离。
- **排查方法：** 用 `cargo test -p omnim-model --lib -- --test-threads=1` 重跑该二进制；若隔离运行必过而并行运行偶发失败，即是上述全局状态竞争。
- **候选修复（做测试隔离重构时择一）：** per-test Library 注入（构造 `Project` 时传入库，而不是读静态全局）、将依赖全局的测试串行化、或每测试进程 `OnceLock` 快照。
- server 集成测试不受影响（独立进程 + 每次运行独立端口隔离）。

**新测试防 flake 惯例：** 不要从测试里写全局静态；像现有 store/`Project` 测试那样在局部构造状态。

## 如何添加测试

**Rust 单元测试** — 在你改动的模块里加 `#[cfg(test)] mod tests`，沿用现有风格（小的 setup + 断言，不依赖外部文件）。

**服务器 API 测试** — 扩展 `crates/server/tests/api_integration.rs`。构造 JSON body，按文件中已有的 `oneshot` 客户端模式打共享路由，并断言状态码 + 解码后的 serde JSON。每个端点/边界情况写一个聚焦的测试；mock 供应商的 AMIR 测试按设计不需密钥。优先使用现有 `.ifc` 文件，不要新建过大的新文件——若确需新增，命名清晰（`*_test.ifc`）并注意只有仓库根路径会被读取。

**有状态的草图/模型测试** — 直接构造 `Project`（参考现有 model/store 测试），不必经由网络栈。

**Web 测试** — 放在 `web/src/__tests__/` 下，命名 `<feature>.test.ts`，测 store/状态机逻辑而非 three.js 渲染本身（渲染难以无头断言）。保持纯逻辑：zustand store 与 reducer 式函数很适合 Vitest。

## 快速全量验证

```bash
cargo test                                   # 全部 Rust
cd web && npm run test                       # 全部 Web
scripts/test-all.sh                          # Rust + web 构建（不含 web 测试）
```
