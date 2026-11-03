---
title: "OmniM 界面设计系统"
status: design
last_updated: 2026-11-02
applies_to: "web/"
evidence: "n/a (design contract)"
---
# OmniM 界面设计系统

> **契约范围。** 本文固定令牌、主题、排版、间距、z-index、图标、组件状态与视口配色。它是需求 `UI-DS-01` … `UI-DS-12` 的规范来源。
>
> **双轨维护义务**：`DESIGN_SYSTEM.zh-CN.md` 是持续维护的译文。在同一次变更中保持章节编号、表格、令牌名与需求 ID 一致。
>
> **状态。** 设计契约。现有样式表早于这些令牌；迁移进度见 [FRONTEND_MATURITY](../../status/FRONTEND_MATURITY.zh-CN.md)。

## 1. 目的与原则

1. **令牌优先。** 每个视觉值都有名称、有主题、可评审。
2. **两套主题，一个结构。** 深色与浅色只差语义取值。
3. **工程密度。** 正文 12–13 px，行高 24–32 px，不用装饰性留白。
4. **状态完整。** 状态不全的组件不得入库。
5. **不允许内联例外。** 一次性取值就是设计系统缺陷。

## 2. 令牌架构

### 2.1 三层

| 层 | 前缀 | 示例 | 使用方 |
|---|---|---|---|
| 原始 | `--omni-color-*`、`--omni-space-*` | `--omni-color-blue-500: #4a90d9` | 仅语义层 |
| 语义 | `--surface-*`、`--text-*`、`--border-*`、`--accent-*` | `--surface-panel` | 组件 |
| 组件 | `--btn-*`、`--tree-*`、`--viewport-*` | `--btn-bg-hover` | 单个组件族 |

**规则：** 组件必须消费语义或组件令牌。组件文件中出现原始令牌引用即缺陷，由 `scripts/check_ui_contracts.py --tokens` 检出。

### 2.2 命名

```text
--<层>-<角色>-<变体>-<状态>
  角色:  surface | text | border | accent | status | shadow
  变体:  panel | elevated | sunken | muted | strong | subtle
  状态:  default | hover | active | disabled | selected | focus
```

名称使用语义角色，不用视觉描述（用 `--surface-panel`，不用 `--gray-800-panel`）。

### 2.3 主题与 `data-theme`

主题应用在 `<html data-theme="dark|light|hc">` 上。所有语义令牌在一个地方按主题重定义：

```css
:root, [data-theme="dark"] { color-scheme: dark;  --surface-app: #1a1d23; }
[data-theme="light"]      { color-scheme: light; --surface-app: #f4f6f9; }
[data-theme="hc"]         { color-scheme: dark;  --surface-app: #000000; }
```

除 3D 场景（读取 §10 的视口调色板）外，组件不得在 JavaScript 中判断主题。

### 2.4 强制检查

| 检查 | 规则 | 门禁 |
|---|---|---|
| 硬编码颜色 | 令牌文件之外零 hex/rgb/hsl；由 `--strict --tokens` 强制 | UI-DS-04 |
| 令牌层级 | 组件样式中零原始令牌引用 | UI-DS-01（组件令牌就绪前扫描为 📅） |
| 刻度外取值 | 字号、间距、圆角、阴影必须来自既定集合 | UI-DS-06, UI-DS-07 |
| z-index | 只能来自 §7 的刻度 | UI-DS-08 |

## 3. 主题

### 3.1 深色主题

主用工程主题。中性低彩度表面，一个蓝色强调色，状态色只用于表达含义。

| 角色 | 取值 |
|---|---|
| 应用背景 | `#1a1d23` |
| 面板表面 | `#14161c` |
| 抬高表面 | `#1e2128` |
| 下沉表面 | `#121418` |
| 边框默认 | `#2a2d35` |
| 边框强调 | `#3a3d4a` |
| 主文本 | `#e0e0e0` |
| 次文本 | `#ccc` |
| 弱化文本 | `#888` |
| 禁用文本 | `#555` |
| 强调色 | `#4a90d9` |
| 强调悬停 | `#5ba0e9` |
| 强调按下 | `#4fc3f7` |
| 焦点环 | `#4fc3f7` |

### 3.2 浅色主题与系统偏好

浅色是**工作主题**，不是打印主题：在明亮办公显示器上保持稳定对比。

| 角色 | 取值 |
|---|---|
| 应用背景 | `#f4f6f9` |
| 面板表面 | `#e9edf2` |
| 抬高表面 | `#ffffff` |
| 下沉表面 | `#eef1f5` |
| 边框默认 | `#d5dae2` |
| 边框强调 | `#b8c0cb` |
| 主文本 | `#1c222b` |
| 次文本 | `#374151` |
| 弱化文本 | `#5b6575` |
| 禁用文本 | `#98a2b3` |
| 强调色 | `#2f6fb3` |
| 强调悬停 | `#3b7fc7` |
| 强调按下 | `#1f5f9f` |
| 焦点环 | `#1f5f9f` |

规则：

1. 首次启动遵循 `prefers-color-scheme`。
2. 显式选择存入 `localStorage`（`omnim.theme`），优先于系统。
3. 主题切换在一帧内生效：不重载，除调色板外不重建场景。
4. 需求：UI-DS-02, UI-DS-03。

### 3.3 语义令牌映射

| 语义令牌 | 深色 | 浅色 |
|---|---|---|
| `--surface-app` | `#1a1d23` | `#f4f6f9` |
| `--surface-panel` | `#14161c` | `#e9edf2` |
| `--surface-elevated` | `#1e2128` | `#ffffff` |
| `--surface-sunken` | `#121418` | `#eef1f5` |
| `--border-default` | `#2a2d35` | `#d5dae2` |
| `--text-primary` | `#e0e0e0` | `#1c222b` |
| `--text-muted` | `#888` | `#5b6575` |
| `--accent-default` | `#4a90d9` | `#2f6fb3` |
| `--status-danger` | `#f44336` | `#c62828` |
| `--status-warning` | `#ff9800` | `#a15c00` |
| `--status-success` | `#81c784` | `#2e7d32` |
| `--status-info` | `#4a90d9` | `#2f6fb3` |

取值权威来源是 `web/src/globals.css`：本表与令牌文件必须一致；组件样式不得再出现字面量颜色（`scripts/check_ui_contracts.py --only tokens` 为 0）。

### 3.4 高对比

第三套映射（`data-theme="hc"`）用于可访问性，不是品牌主题：纯黑背景、纯白文本、饱和强调色、非文本对比 ≥3:1、无阴影、2 px 焦点环。

需求：UI-DS-11, UI-A11Y-07。

## 4. 语义状态矩阵

每个交互角色必须定义六态。缺态将阻止组件注册。

| 角色 | 默认 | 悬停 | 按下 | 选中 | 禁用 | 聚焦 |
|---|---|---|---|---|---|---|
| 表面 | `--surface-panel` | `--surface-elevated` | `--surface-sunken` | `--accent-subtle` | `--surface-panel` @60% | 环 |
| 边框 | `--border-default` | `--border-strong` | `--accent-default` | `--accent-default` | `--border-default` @50% | 环 |
| 文本 | `--text-primary` | `--text-primary` | `--text-primary` | `--accent-default` | `--text-disabled` | — |
| 强调 | `--accent-default` | `--accent-hover` | `--accent-active` | `--accent-active` | `--accent-default` @40% | 环 |
| 危险 | `--status-danger` | 提亮 8% | 压暗 8% | `--status-danger` | 40% | 环 |

禁用元素还必须设置 `aria-disabled="true"`，且不得接收会改变状态的指针事件。

## 5. 排版

| 令牌 | 字号 | 行高 | 用途 |
|---|---|---|---|
| `--text-xs` | 11 px | 16 px | 密集元信息、状态栏 |
| `--text-sm` | 12 px | 18 px | 面板标签、表格单元 |
| `--text-md` | 13 px | 20 px | 正文、输入、菜单 |
| `--text-lg` | 15 px | 22 px | 区块标题 |
| `--text-xl` | 20 px | 28 px | 页面标题 |
| `--text-2xl` | 28 px | 36 px | 项目首页主标题 |

规则：

1. 正文 12–13 px；禁用 10 px。
2. 数值界面（坐标、尺寸、帧率）使用 `font-variant-numeric: tabular-nums`。
3. 等宽字体只用于控制台、ID 与代码，不用于标签。
4. 字体栈：`Inter, "Segoe UI", system-ui, -apple-system, sans-serif`；中文回退 `"Noto Sans SC", "PingFang SC", "Microsoft YaHei"`。
5. 需求：UI-DS-06。

## 6. 间距、圆角、阴影

| 刻度 | 取值 |
|---|---|
| 间距 | 4、8、12、16、24、32 px（`--space-1…6`） |
| 行高 | 紧凑 24 px、默认 28 px、舒适 32 px |
| 圆角 | 2、4、6、8 px（`--radius-1…4`） |
| 阴影 | sm `0 1px 2px rgba(0,0,0,.30)`、md `0 6px 16px rgba(0,0,0,.35)`、lg `0 16px 40px rgba(0,0,0,.45)` |

阴影只用于抬高表面（浮层、弹窗、浮动面板）。停靠面板用边框，不用阴影。需求：UI-DS-07。

## 7. Z-Index 分层

| 层 | 取值 | 令牌 | 用途 |
|---|---|---|---|
| 0 | 0 | `--z-viewport` | 视口画布 |
| 1 | 10–19 | `--z-overlay`、`--z-local`、`--z-local-raised` | 视口覆盖层（视图立方、工具条、提示）与局部堆叠 |
| 2 | 20–29 | `--z-dock`、`--z-resize` | 停靠与分隔条 |
| 3 | 30–39 | `--z-shell`、`--z-menu` | 附着在外壳上的浮层、菜单、提示 |
| 4 | 40–49 | `--z-dialog` | 弹窗遮罩与弹窗 |
| 5 | 60–69 | `--z-toast` | 提示消息与实时播报 |
| 6 | 90–99 | `--z-debug` | 调试覆盖层（绝不进生产） |

任何组件都不得使用这些区间之外的 z-index；组件样式只能通过 `var(--z-*)` 取值。需求：UI-DS-08。

## 8. 图标

| 规则 | 取值 |
|---|---|
| 尺寸 | 16 px（内联）、20 px（工具栏）、24 px（功能区） |
| 描边 | 1.5 px，圆头圆角 |
| 颜色 | 仅 `currentColor` |
| 网格 | 16 px 画板，1 px 内边距，2 px 圆角 |
| 禁止 | emoji、位图、混用描边粗细 |
| 可访问性 | 纯图标按钮必须有可访问名称 |

图标集位于 `web/src/app/components/icons/`，由单一索引导出。需求：UI-DS-09, UI-A11Y-05。

## 9. 组件契约

组件只有声明以下全部内容才允许注册：

| 项 | 要求 |
|---|---|
| 状态 | 默认、悬停、按下、选中、禁用、聚焦、加载、错误 |
| 尺寸 | sm / md（lg 仅用于弹窗与页面级操作） |
| 令牌 | 只使用语义令牌 |
| 键盘 | 可聚焦；适用时 Enter/Space 可操作 |
| 可访问性 | role、可访问名称、禁用时 `aria-disabled` |
| 文档 | 组件清单中的一行 |

G2 的最低组件清单：Button、IconButton、SplitButton、Select、Combobox、NumberField、TextField、Checkbox、Radio、Switch、Slider、Tabs、Tree、Table、Panel、Tooltip、Popover、ContextMenu、Dialog、ConfirmDialog、Toast、Skeleton、EmptyState、ProgressBar、Banner、StatusBar、Badge。

需求：UI-DS-10。

## 10. 视口调色板

3D 场景在两套主题下都必须清晰可读。场景颜色是令牌，不是渲染代码里的字面量。

| 场景角色 | 深色 | 浅色 |
|---|---|---|
| 背景 | `#232830` | `#eef1f5` |
| 主轴网 | `#4a5566` | `#b6bfca` |
| 次轴网 | `#333b47` | `#d3d9e2` |
| 边线 | `#11141a` | `#8a929e` |
| 选中 | `#4fc3f7` | `#1565c0` |
| 悬停 | `#7fd4ff` | `#1a6fd4` |
| 剖切面 | `#e0a030` | `#a86400` |
| 测量 | `#4caf72` | `#2e7d32` |
| 幽灵预览 | `#4fc3f7` @35% | `#1565c0` @30% |

规则：

1. 渲染器从当前主题派生的 `ViewerPalette` 对象取值；渲染代码中不得出现主题判断。
2. 选中与悬停相对背景必须保持 3:1 可辨识度。
3. 需求：UI-DS-12, UI-A11Y-01。

## 11. 治理

| 规则 | 取值 |
|---|---|
| 令牌变更 | 需要设计评审条目，并提升 `tokens.css` 版本 |
| 新组件 | 需要完整状态矩阵与清单行 |
| 废弃 | 标记废弃一个次要版本后才可移除 |
| 例外 | 内联样式必须带到期日与责任人的注释 |
| 自动检查 | `check_ui_contracts.py`、视觉回归、对比度审计 |

## 12. 非目标

1. 用户自定义主题或品牌皮肤。
2. 按项目换肤。
3. 打印样式表。
4. 在仓库外发布组件库。

## 13. 待定问题

| # | 问题 | 责任人 | 需要时间 |
|---|---|---|---|
| Q1 | 高对比应作为主题，还是适配系统 `forced-colors`？ | 设计 / 可访问性 | G3 |
| Q2 | 区块标题用 15 px 够不够，还是应改 16 px？ | 设计 | G2 |
| Q3 | 是否需要为 1080p 笔记本提供紧凑密度模式？ | 产品 | G2 |
