---
title: "OmniM Web 架构 — 现状"
status: current
last_updated: 2026-10-29
applies_to: "web/, crates/server"
evidence: "2026-10-29 对照 web/src 审计（ts/tsx/css 共 21,800 行；94 个单元测试通过）"
---
# OmniM Web 架构 — 现状

> **文档定位。** 本文属于*架构*层：前端今天实际是什么样，并已对照代码核实。设计意图见 [UI 架构族](../specification/ui/IA_SHELL.zh-CN.md)，成熟度评分见 [FRONTEND_MATURITY](../status/FRONTEND_MATURITY.zh-CN.md)。
>
> **双轨维护义务**：`WEB_ARCHITECTURE.zh-CN.md` 是持续维护的译文。在同一次变更中保持章节编号与证据行一致。
>
> **诚实规则。** 这里的每个论断都引用文件或实测计数。与目标不一致的地方要写明，不得模糊处理。

## 1. 目的与范围

Web 客户端是一个 React 19 单页应用，由 Vite 提供构建，通过 REST 与 Axum 服务通信，用 three.js 渲染。本文梳理其分层、数据流、渲染管线、状态归属、构建与已知债务。

## 2. 系统上下文

| 组件 | 技术 | 证据 |
|---|---|---|
| 客户端 | React 19、TypeScript 6、Vite 8 | `web/package.json` |
| 路由 | react-router-dom 7 | `web/src/main.tsx`、`web/src/app/App.tsx` |
| 状态 | Zustand 5 + TanStack Query 5 | `web/src/app/store/appStore.ts`、`web/src/shared/queries.ts` |
| 渲染 | three.js 0.185、`@mlightcad/three-viewcube` | `web/src/app/components/ThreeViewer.tsx`、`ViewCube3D.tsx` |
| 传输 | `fetch` 封装 | `web/src/shared/api.ts`（371 行） |
| 服务 | Axum REST，JSON 文件持久化 | `crates/server/src/lib.rs` |
| 构建产物 | `web/dist` 下的静态包 | `web/dist/` |

当前路由：`/projects`（项目首页）、`/`（工作区）、`/component-editor`、`/series-manager`、`/library-manager`。

## 3. 前端分层

```text
web/src/
├── main.tsx                 # Provider：QueryClient、Router、Toast
├── app/
│   ├── App.tsx              # 路由 + 全局快捷键 + 工具栏接线（236 行）
│   ├── layout/AppLayout.tsx # 外壳：侧栏 + 视口 + 右面板
│   ├── panels/              # PanelRegistry、defaultPanels、领域/属性/树面板
│   ├── components/
│   │   ├── ThreeViewer.tsx  # 3D 场景、拾取、预览、剖切、测量（1794 行）
│   │   ├── MainToolbar.tsx  # 创建流程、导入导出、专业操作（460 行）
│   │   ├── OmniPanel.tsx    # Omcom 控制台：高亮、补全、历史（937 行）
│   │   ├── create/          # 创建表单、图层编辑器、目录参数
│   │   └── viewer/three/    # 捕捉、拾取、标记、基准轴网
│   ├── store/appStore.ts    # 界面 + 会话状态（189 行）
│   └── context/             # ThreeViewerContext
├── shared/                  # api、类型、目录、toast、几何、创建状态机
├── sketch/                  # 构件编辑器 + 2D/3D 草图画布（1359 + 1038 行）
├── library/                 # LibraryManagerPage（597 行）
├── series/                  # SeriesManagerPage（648 行）
└── __tests__/               # 5 个 Vitest 文件，79 个测试
```

行数来自 2026-10-29 的 `wc -l`。`ThreeViewer.tsx`、`styles.css`、`ComponentEditorPage.tsx` 的体量集中是主要可维护性风险。

## 4. 数据流

一次创建点击当前走这条路径：

```text
功能区 / 工具栏
  → appStore creationDispatch（状态机）
  → ThreeViewer 中拾取点（射线 + 捕捉）
  → 表单校验（shared/types/creation）
  → shared/api.ts POST /api/genrule | /api/objects | /api/catalog/instantiate
  → 服务端 Validator + Executor + persist
  → 响应
  → refresh() + reloadScene()（全量树 + 全量网格重载）
```

观察：

1. **每次变更都全量重载。** `refresh()` 与 `reloadScene()` 重新拉取整棵树与整份网格；没有增量路径。
2. **两个写入面。** 创建走 `genrule`/`instantiate`，控制台走 `POST /api/omcom`。服务端最终收敛，但客户端分叉。
3. **没有乐观状态。** 界面要等服务端往返后才显示对象。
4. **错误是字符串。** `api.ts` 抛出 `Error(message)`，没有错误码，界面无法把失败映射到恢复动作。

## 5. 渲染管线

| 阶段 | 实现 | 证据 |
|---|---|---|
| 渲染器 | 开启抗锯齿的 `WebGLRenderer` | `ThreeViewer.tsx` |
| 色调映射 | ACES Filmic，曝光 1.2 | 同上 |
| 阴影 | PCF 软阴影，2048²，单个平行光 | 同上 |
| 环境 | `RoomEnvironment` 纹理 | 同上 |
| 场景 | 雾、半球光 + 环境光 + 两盏补光 | 同上 |
| 边线 | 每个网格叠加 `EdgesGeometry` | 同上 |
| 循环 | 持续 `requestAnimationFrame` | 同上 |
| 主题 | 场景颜色硬编码（背景、网格、边线） | 同上 |

**债务：** 空闲时循环仍在运行，场景颜色是字面量而非 [DESIGN_SYSTEM §10](../specification/ui/DESIGN_SYSTEM.zh-CN.md) 中的调色板契约。

## 6. 状态归属

| 状态 | 归属 | 持久化 | 说明 |
|---|---|---|---|
| 服务端数据（树、对象、网格） | TanStack Query / store | 服务端 | 整体重新拉取 |
| 选择 | `appStore` | 无 | 字符串数组 |
| 创建会话 | `appStore`（`creationPhase`、点、表单） | 无 | 镜像 `stateMachine.ts` |
| 视图设置（网格、正交、坐标空间、显示模式） | `appStore` | 无 | 刷新即丢失 |
| 布局（停靠宽度、面板模式） | `AppLayout` 局部状态 | 无 | 刷新即丢失 |
| 主题 | — | — | 未实现 |
| 语言 | — | — | 未实现 |
| 项目路径 / 脏状态 | — | — | 未实现 |

**差距：** 没有 `layoutStore`、没有主题 Provider、没有 i18n Provider、没有项目会话状态。这些是 `UI-SHL-06`、`UI-DS-02`、`UI-I18N-02` 的前置条件。

## 7. 构建与工具链

| 项 | 状态 |
|---|---|
| 开发服务器 | `vite --host 0.0.0.0 --port 5173` |
| 构建 | `vite build` → `web/dist` |
| 类型检查 | `tsc --noEmit` 通过 |
| 单元测试 | Vitest，5 个文件，79 个测试 |
| Lint / 格式化 | **`web/` 未配置** |
| E2E | **无** |
| 视觉回归 | **无** |
| 包体分析 | **无** |

## 8. 已知债务

计数测量于 2026-10-29，构成 [FRONTEND_MATURITY](../status/FRONTEND_MATURITY.zh-CN.md) 的基线。

| # | 债务 | 证据 | 需求 |
|---|---|---|---|
| D1 | 属性面板默认是居中浮动层 | `appStore.ts:89` `panelMode: 'floating'`；`AppLayout` 的 `.floating-panel-overlay` 为 `inset:0` + 居中 | UI-SHL-03 |
| D2 | 没有项目首页、新建/打开/另存为；客户端从不调用 `POST /api/project/save` | 路由表；`saveProject` 引用为零 | UI-DAT-01, UI-DAT-02 |
| D3 | 4,110 行全局样式表中有 712 处硬编码颜色（全部 CSS 731 处） | `web/src/styles.css` | UI-DS-04 |
| D4 | 令牌定义两份且被绕过 | `globals.css` 与 `styles.css` 各有一份完全相同的 `:root` | UI-DS-01 |
| D5 | 63 处 emoji，其中 25 处作为面板图标 | `DomainPanels.tsx` 的 `icon: '▦'` 等 | UI-DS-09 |
| D6 | 52 处 `Date.now()` 调用，多数用于构造对象名 | `wall-${Date.now()}` 模式 | UI-INT-09 |
| D7 | 5 处原生弹窗 | `window.confirm/alert/prompt` | UI-FBK-05 |
| D8 | 没有错误边界 | `ErrorBoundary`/`componentDidCatch` 为零 | UI-FBK-06 |
| D9 | 没有 i18n；31 个文件中 858 段中文内联 | 源码扫描 | UI-I18N-01 |
| D10 | 可访问性接近零 | 1 个 `aria-*`、0 个 `role` | UI-A11Y-04 |
| D11 | 没有实例化、LOD 或列表虚拟化 | `InstancedMesh`/虚拟化引用为零 | UI-PRF-03, UI-PRF-07 |
| D12 | 持续渲染循环 | 无条件 `requestAnimationFrame` | UI-PRF-04 |
| D13 | 过期文档引用已删除文件 | `ARCHITECTURE_MAP` 引用 `OmcomConsole.tsx`、`CatalogBrowser.tsx`、`SeriesBrowser.tsx` | — |

## 9. 扩展点

| 扩展 | 机制 | 契约 |
|---|---|---|
| 新面板 | `PanelRegistry.register` + `defaultPanels` | [IA_SHELL §4](../specification/ui/IA_SHELL.zh-CN.md) |
| 新命令 | 命令注册表（目标） | [INTERACTION §2](../specification/ui/INTERACTION.zh-CN.md) |
| 新弹窗 | `DialogProvider`（目标） | [INTERACTION §7](../specification/ui/INTERACTION.zh-CN.md) |
| 新令牌 | `tokens.css` 语义层 | [DESIGN_SYSTEM §2](../specification/ui/DESIGN_SYSTEM.zh-CN.md) |
| 新语言 | `shared/i18n/<locale>.ts` | [I18N_L10N §3](../specification/ui/I18N_L10N.zh-CN.md) |
| 新测试 | `src/__tests__`（单元）、`tests/e2e`（目标） | [TESTING](../guides/TESTING.zh-CN.md) |

## 10. 非目标

1. 服务端渲染。
2. 微前端拆分。
3. 面板的公开插件 API。
4. 在本产品周期内替换 three.js。

## 11. 待定问题

| # | 问题 | 责任人 |
|---|---|---|
| Q1 | `ThreeViewer.tsx` 应在壳体重写前拆分，还是之后？ | 前端 |
| Q2 | 是否需要拆分全局 store（布局 / 项目 / 会话）？ | 前端 |
| Q3 | 在开始实例化之前，网格端点是否需要增量模式？ | 前端 / 后端 |
