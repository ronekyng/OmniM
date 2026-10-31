---
title: "OmniM 前端发布就绪度"
status: current
last_updated: 2026-10-31
applies_to: "web/"
evidence: "2026-10-30 依据 REQUIREMENTS_REGISTER 与 FRONTEND_MATURITY 评估门禁状态；并入建模师 UI/UX 复验（G1 体验档）口径区分"
---
# OmniM 前端发布就绪度

> **文档定位。** 本文是门禁看板。它按门禁说明必须满足什么、今天满足什么、什么在阻塞下一道门禁。需求本身不在本文定义，见 `docs/product/`。
>
> **双轨维护义务**：`RELEASE_READINESS.zh-CN.md` 是持续维护的译文。门禁计数与状态标记必须一致。
>
> **更新规则。** 门禁条目只有在有链接证据时才改变状态。口头确认不算证据。

## 1. 目的

商用发布是一个决策，不是一个日期。本看板让决策可审计：每道门禁都有退出标准、证据与责任人。

## 2. 门禁模型

| 门禁 | 名称 | 目的 | 退出责任人 |
|---|---|---|---|
| G0 | 契约就绪 | 编码前先对齐设计与需求 | 产品 + 设计 |
| G1 | Alpha | 主流程可用；无数据丢失与白屏类缺陷 | 前端负责人 |
| G2 | Beta | 产品级界面：主题、i18n、可访问性、性能、反馈 | 产品 |
| G3 | GA | 可支持发布：兼容性、安全、遥测、帮助 | 产品 + 支持 |

每道门禁都有一条硬规则：**不能因为上一道"快完成了"而跳过**。

## 3. G0 契约就绪

| 项 | 必需 | 状态 | 证据 |
|---|---|---|---|
| 产品愿景与非目标 | 是 | 🔀 | [VISION_SCOPE](../product/VISION_SCOPE.zh-CN.md) 已起草，待评审 |
| 画像与旅程 | 是 | 🔀 | [PERSONAS_JOURNEYS](../product/PERSONAS_JOURNEYS.zh-CN.md) 已起草 |
| 含 96 条需求的 PRD | 是 | 🔀 | [PRD_FRONTEND](../product/PRD_FRONTEND.zh-CN.md) 已起草 |
| 需求登记册（RTM） | 是 | 🔀 | [REQUIREMENTS_REGISTER](../product/REQUIREMENTS_REGISTER.zh-CN.md) 已起草 |
| 外壳与信息架构契约 | 是 | 🔀 | [IA_SHELL](../specification/ui/IA_SHELL.zh-CN.md) 已起草 |
| 设计系统契约 | 是 | 🔀 | [DESIGN_SYSTEM](../specification/ui/DESIGN_SYSTEM.zh-CN.md) 已起草 |
| 交互契约 | 是 | 🔀 | [INTERACTION](../specification/ui/INTERACTION.zh-CN.md) 已起草 |
| 反馈/错误契约 | 是 | 🔀 | [FEEDBACK_ERRORS](../specification/ui/FEEDBACK_ERRORS.zh-CN.md) 已起草 |
| 数据完整性契约 | 是 | 🔀 | [DATA_INTEGRITY](../specification/ui/DATA_INTEGRITY.zh-CN.md) 已起草 |
| 非功能契约 | 是 | 🔀 | A11Y、I18N、PERFORMANCE、COMPATIBILITY、SECURITY、TELEMETRY 已起草 |
| 文档规范与检查 | 是 | ✅ | `check_ui_contracts.py`（0 失败）与 `gen_keymap.py --check` 已纳入 CI 文档任务 |

**G0 判定：** 🔀 **草案完成，评审与签字待办。** 契约检查已自动化并通过；剩余条件是 [G0 评审](../process/G0_REVIEW_2026-10-29.zh-CN.md) 记录的 O1 决策。

## 4. G1 Alpha 就绪

G1 的 25 条需求完整清单（从登记册生成，不是挑选子集）。

| 需求 | 状态 | 证据 / 阻塞 |
|---|---|---|
| UI-SHL-01 三层应用结构 | 🔀 | `/projects` + 工作区 + 编辑器；路由矩阵 e2e 待补 |
| UI-SHL-02 七个固定区域 | ✅ | `web/e2e/shell.spec.ts` 断言左/标题/工具栏/视口/右/底/状态七区 |
| UI-SHL-03 属性面板右停靠 | ✅ | 默认右停靠 260–600 px；`layout.spec.ts` 拖宽 + 重载持久化 |
| UI-SHL-04 左停靠标签页 | ✅ | Objects / Grids / 构件库三标签；`layout.spec.ts` 断言 240–400 px |
| UI-SHL-05 底停靠折叠且带标签 | 🔀 | 控制台常驻，无标签 |
| UI-SHL-06 布局持久化与重置 | ✅ | `layout.spec.ts` 拖宽 → 重载恢复 → Reset layout 回 340 px |
| UI-SHL-07 上下文功能区 | 🔀 | 扁平工具栏，未按专业切换 |
| UI-SHL-08 状态栏读数 | ✅ | `web/e2e/shell.spec.ts` 断言坐标/选择/单位/保存；开发者模式断言 FPS |
| UI-SHL-10 标题与面包屑 | 🔀 | 工作区标题/面包屑 + 文档标题；编辑器面包屑待补 |
| UI-DS-02 深浅主题切换 | 🔀 | `data-theme` + `localStorage` 持久化 + OS 默认；标题栏切换；语义令牌迁移（UI-DS-04）与截图证据待补 |
| UI-INT-01 五种选择手势 | 🔀 | 单击/Ctrl/框选已有；Esc 先取消工具 |
| UI-INT-02 拾取状态机 | ✅ | `stateMachine.test.ts` |
| UI-INT-04 输入时不触发快捷键 | ✅ | `web/e2e/shell.spec.ts` 在名称输入框按 Delete，对象未被删除；注册表统一属 UI-INT-03 |
| UI-INT-09 自动命名 | ✅ | `naming.ts` + 4 例单测；接入创建/快速创建/子对象/复制 |
| UI-FBK-01 面板三态 | 🔀 | 属性面板与构件面板已补加载骨架、错误 + 重试、空态；其余面板待补 |
| UI-FBK-02 骨架屏时序 | 🔀 | 400 ms 阈值有单测（`delayedVisibility.test.ts`）；属性/构件面板已接入，逐面板覆盖待补 |
| UI-FBK-05 无原生弹窗 | ✅ | `ConfirmDialog` + `notify`；`check_ui_contracts` dialogs = 0 |
| UI-FBK-06 错误边界 | 🔀 | 应用、工作区与每个面板槽位；抛错面板测试待补 |
| UI-FBK-07 分级提示、错误常驻 | 🔀 | 已分级；错误会自动消失 |
| UI-FBK-08 可关闭且不遮挡 | 🔀 | 可关闭；位置会压住外壳 |
| UI-FBK-09 空状态带操作 | 🔀 | 仅部分面板 |
| UI-DAT-01 项目首页 | 🔀 | `/projects` 含新建/打开/最近/示例；服务端 list/new/open + 允许根目录；E2E 覆盖示例项目进入工作区 |
| UI-DAT-02 显式保存 | 🔀 | 保存按钮 + Ctrl/Cmd+S + 另存为（`/api/project/save-as`）；往返 e2e 待补 |
| UI-DAT-03 脏状态 | 🔀 | `saveState.ts` + 标题/状态栏指示；状态 e2e 待补 |
| UI-DAT-06 未保存拦截 | 🔀 | 脏/失败时 `beforeunload`；应用内导航拦截待补 |

**G1 判定：** 📅 **未就绪。** 25 条中 9 条完成（`UI-SHL-02/03/04/06/08`、`UI-INT-02/04/09`、`UI-FBK-05`），16 条部分实现，0 条缺失。

> **验收记录（2026-10-30，口径区分）。** 建模师 UI/UX 复验（[MODELER_UIUX_REAUDIT_2026-10-30](MODELER_UIUX_REAUDIT_2026-10-30.zh-CN.md)）以**可用原型（G1）档的 UI/UX 维度**正式放行：两个 P0-UX（保存假警报、单位陷阱）已关闭。这不改变上表按 RTM 逐条判定的门禁状态——25 条仍以 `REQUIREMENTS_REGISTER` 的验证证据为准，本看板不因"体验验收通过"而升级门禁。

## 5. G2 Beta 就绪

| 域 | 要求 | 状态 |
|---|---|---|
| 设计系统（12） | 令牌、主题、图标、视口调色板；3 项完备度延至 G3 | 📅 |
| 可访问性（8） | 关键路径 WCAG 2.2 AA；动效与实时播报延至 G3 | 📅 |
| 国际化（7） | 100% 外置且可切换；伪语言延至 G3 | 📅 |
| 性能（8） | 全部预算在 CI 强制 | 📅 |
| 反馈（剩余） | 进度、带码错误、提示 | 🔀 |
| 数据完整性（剩余） | 自动保存、恢复、元信息 | 📅 |
| 交互（剩余） | 命令注册表、弹窗、撤销历史 | 🔀 |

**2026-10-29 延至 G3**（O1 门禁再平衡）：`UI-DS-05`、`UI-DS-07`、`UI-DS-10`、`UI-A11Y-06`、`UI-A11Y-08`、`UI-I18N-05`。理由：G2 是外部试点门禁，完备度层级属于 GA。`UI-DS-08` 保留在 G2，因为 z-index 分层是壳体重写的前置条件且成本低。

**G2 判定：** 📅 **未就绪。**

## 6. G3 GA 就绪

| 域 | 要求 | 状态 |
|---|---|---|
| 完备度层级 | DS-05/07/10、A11Y-06/08、I18N-05（从 G2 移入） | 📅 |
| 兼容性（6） | 浏览器/GPU/DPI 矩阵 | 📅 |
| 安全（5） | 披露、路径根、CSP、同意 | 📅 |
| 遥测与支持（6） | 事件字典、诊断、帮助 | 📅 |
| 数据完整性（最终） | 冲突、依赖、快照 | 📅 |
| 合规产物 | ACR/VPAT 草稿 | 📅 |
| 发布说明与迁移 | 模板与流程见 [RELEASE_CHECKLIST](../process/RELEASE_CHECKLIST.zh-CN.md) | 📅 |
| 支持运行手册 | 模板见 [RELEASE_CHECKLIST](../process/RELEASE_CHECKLIST.zh-CN.md) §4–§5 | 📅 |

**G3 判定：** 📅 **未就绪。**

## 7. 阻塞问题

| # | 问题 | 门禁 | 责任人 | 需求 |
|---|---|---|---|---|
| B6 | 令牌在 731 处被绕过 | G2 | 前端 | UI-DS-04 |
| B7 | 无 i18n 层 | G2 | 前端 | UI-I18N-01 |
| B8 | 无可访问性语义 | G2 | 前端 | UI-A11Y-04 |
| B9 | 持续渲染循环；无实例化 | G2 | 前端 | UI-PRF-03/04 |
| B10 | 无兼容性与安全披露策略 | G3 | 产品 | UI-CMP-01, UI-SEC-01 |

## 8. 证据记录

| 日期 | 门禁 | 事件 | 证据 |
|---|---|---|---|
| 2026-10-29 | G0 | 契约起草（产品 + UI） | 本文档集 |
| 2026-10-29 | — | 基线完成测量 | [FRONTEND_BASELINE](FRONTEND_BASELINE.zh-CN.md) |
| 2026-10-29 | G1（部分） | 外壳/可靠性切片：属性停靠、布局持久化、标题/状态栏、错误边界、共享弹窗、显式保存/另存为、自动命名 | `web/src/__tests__/{layout,saveState,naming}.test.ts`；Rust `persistence::save_as_retargets_the_session_and_rejects_bad_paths` |
| 2026-10-29 | G1（部分） | 项目首页：会话门禁、新建/打开/最近/示例、允许根目录 | `crates/server/tests/projects.rs`；`web/src/__tests__/recentProjects.test.ts` |

## 9. 非目标

1. 公开的发布日期。
2. 市场宣传就绪度。
3. 后端或 IFC 发布门禁（另有看板）。

## 10. 待定问题

| # | 问题 | 责任人 | 需要时间 |
|---|---|---|---|
| Q1 | G1 是对客户可见的里程碑，还是仅内部？ | 产品 | G1 |
| Q2 | G3 是否需要外部安全评审？ | 产品 / 法务 | G3 |
| Q3 | 设计与工程意见不一致时，谁签 G2？ | 产品 | G2 |
