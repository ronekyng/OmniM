---
title: "OmniM 前端开发者指南"
status: current
last_updated: 2026-10-29
applies_to: "web/"
evidence: "2026-10-29 对照 web/src 结构审计；recipe 引用目标契约"
---
# OmniM 前端开发者指南

> **文档定位。** 改动前端的实操指南：东西在哪、最常用的五条 recipe。它补充 [ARCHITECTURE_MAP](ARCHITECTURE_MAP.zh-CN.md)，后者映射 crate。
>
> **双轨维护义务**：`FRONTEND_GUIDE.zh-CN.md` 是持续维护的译文。章节编号与代码路径必须一致。
>
> **诚实规则。** 标记 📅 的 recipe 描述目标结构，在所引契约实现前无法直接照做。

## 1. 目的

前端有 20,000 行、三个编辑界面和一个 1,794 行的渲染器。本文缩短从"我要改 X"到"文件在这里、检查项是这些"的路径。

## 2. 东西在哪

| 关注点 | 位置 |
|---|---|
| 路由 | `web/src/app/App.tsx` |
| 外壳 | `web/src/app/layout/AppLayout.tsx`（📅 将由 `app/shell/` 取代） |
| 面板注册 | `web/src/app/panels/PanelRegistry.tsx`、`defaultPanels.ts` |
| 会话/界面状态 | `web/src/app/store/appStore.ts` |
| 3D 场景与拾取 | `web/src/app/components/ThreeViewer.tsx`、`viewer/three/` |
| 创建表单 | `web/src/app/components/create/` |
| 控制台 | `web/src/app/components/OmniPanel.tsx` |
| 草图编辑器 | `web/src/sketch/` |
| 图库/系列页面 | `web/src/library/`、`web/src/series/` |
| REST 客户端 | `web/src/shared/api.ts` |
| 类型 | `web/src/shared/types.ts`、`shared/types/creation.ts` |
| i18n（📅） | `web/src/shared/i18n/` |
| 令牌（📅） | `web/src/styles/tokens.css` |
| 单元测试 | `web/src/__tests__/` |

## 3. 加一个面板

1. 在 `web/src/app/panels/` 或 `components/` 定义组件。
2. 在 `defaultPanels.ts` 注册，提供 `id`、`position`、`title`、`order`、`visible()`。
3. 若需要标签图标或默认宽度，扩展 `PanelTypes.ts`（📅 契约：[IA_SHELL](../specification/ui/IA_SHELL.zh-CN.md) §4）。
4. 加空状态与错误态（[FEEDBACK_ERRORS](../specification/ui/FEEDBACK_ERRORS.zh-CN.md) §3）。
5. 把截图加入视觉基线，并在需求登记册补一行。

**规则：** 取数据的面板不得自带全局 Provider；数据访问统一走 `shared/api.ts`。

## 4. 加一个命令

1. 在注册表加入命令：`id`、`labelKey`、`group`、`shortcut?`、`enabled(ctx)`、`run(ctx)`（📅）。
2. 接到功能区组与菜单。
3. 快捷键有变化时重新生成 [KEYBOARD_MAP](../reference/KEYBOARD_MAP.zh-CN.md)。
4. 把命令加入命令面板覆盖测试。
5. 需求：`UI-INT-03`——不在注册表里的快捷键属于缺陷。

## 5. 加一个弹窗

1. 使用共享弹窗族（`ConfirmDialog`、`PromptDialog`、表单外壳）（📅）。
2. 提供 `aria-labelledby`、`aria-describedby` 与初始焦点目标。
3. `Esc` 关闭，`Enter` 确认，焦点归还触发者。
4. 破坏性弹窗说明影响与可逆性（[INTERACTION](../specification/ui/INTERACTION.zh-CN.md) §7.3）。
5. 绝不使用 `window.confirm`；`check_ui_contracts.py --strict --dialogs` 会拦截。

## 6. 加一个令牌

1. 只有在没有合适原始令牌时才新增原始令牌。
2. 在 `tokens.css` 为每个主题添加语义令牌（📅）。
3. 若取值是组件专用，再映射组件令牌。
4. 在双主题下验证对比度。
5. 绝不在组件中硬编码取值；`check_ui_contracts.py --strict --tokens` 会拦截。

## 7. 加一个字符串

1. 在同一次变更中把键加入 `en-US` 与 `zh-CN` 词典（📅）。
2. 遵循 `area.component.element` 命名（[I18N_L10N](../specification/ui/I18N_L10N.zh-CN.md) §7）。
3. 复数与数字使用 ICU。
4. 若术语是产品新词，先加入 [GLOSSARY](../reference/GLOSSARY.zh-CN.md)。

## 8. 加一个测试

| 层次 | 位置 | 最低要求 |
|---|---|---|
| 单元 | `web/src/__tests__/` | 纯逻辑、状态机、格式化 |
| 组件 | 与组件同目录（📅） | 组件族的渲染与交互 |
| E2E | `web/tests/e2e/`（📅） | 一条用户可见路径 |
| 视觉 | `web/tests/visual/`（📅） | 任何影响布局的变更 |
| 可访问性 | `web/tests/a11y/`（📅） | 变更面、axe 规则 |
| 性能 | `web/tests/perf/`（📅） | 变更的热路径 |

**规则：** 无法因所述原因失败的测试不是证据。

## 9. 排查

| 现象 | 首先检查 |
|---|---|
| 场景空白 | `reloadScene()` token、`/api/mesh` 响应、WebGL 上下文 |
| 选择不更新 | `appStore.selectedIds`、树可见性 |
| 创建卡住 | `creationPhase`、`creationPoints`、表单校验错误 |
| 面板空白 | 错误边界日志、`/api/*` 响应、Provider 顺序 |
| 主题不生效 | `<html>` 上的 `data-theme`、令牌文件导入顺序 |
| 界面显示键名 | 当前语言的缺失键、回退链 |

## 10. 非目标

1. React 或 three.js 教程。
2. 后端开发 recipe（见 [ARCHITECTURE_MAP](ARCHITECTURE_MAP.zh-CN.md)）。
3. API 参考的重复（见 [API](../reference/API.zh-CN.md)）。

## 11. 待定问题

| # | 问题 | 责任人 |
|---|---|---|
| Q1 | 壳体重写前是否应先拆分 `ThreeViewer` 为相机、场景、交互、覆盖层？ | 前端 |
| Q2 | 继续用 Zustand，还是转向命令 store 模型？ | 前端 |
| Q3 | 组件测试用 Testing Library 还是 Playwright 组件模式？ | 前端 |
