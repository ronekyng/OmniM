---
title: "OmniM 前端需求登记册（RTM）"
status: design
last_updated: 2026-11-05
applies_to: "web/"
evidence: "n/a (product requirement)"
---
# OmniM 前端需求登记册（RTM）

> **文档定位。** 本文是前端需求追溯的单一真值。每条需求只有一个 ID、一个契约锚点、至少一个验证物、一个发布门禁与一个状态。文档集中的其他文件不得自造 ID。
>
> **双轨维护义务**：`REQUIREMENTS_REGISTER.zh-CN.md` 是持续维护的译文。两份文件中的 ID、门禁字母与状态标记必须完全一致。
>
> **机器校验。** `scripts/check_ui_contracts.py` 校验：ID 唯一性、在 PRD 中出现、验证列非空、门禁与状态取值合法、总数 96。`scripts/gen_keymap.py --check` 校验快捷键表中英对齐。

## 1. 如何阅读本登记册

| 列 | 含义 |
|---|---|
| ID | 稳定的需求标识，`UI-<AREA>-<nn>` |
| 需求 | 简写；规范性文本在 [PRD_FRONTEND](PRD_FRONTEND.zh-CN.md) |
| 契约 | 设计契约所在位置 |
| 验证 | 证明该需求的产物（测试、扫描、截图集、清单） |
| 门禁 | 该需求必须通过的最早门禁 |
| 状态 | ✅ 已实现 · 🔀 部分 · 📅 计划 · 💭 提案 |

**状态诚实规则：** 状态反映 `last_updated` 时点的代码，而不是本文的意图。一条需求可以是 `MUST` 且状态为 📅。

## 2. ID 分配

| 域 | 含义 | 数量 | 契约 |
|---|---|---|---|
| SHL | 外壳与布局 | 12 | [IA_SHELL](../specification/ui/IA_SHELL.zh-CN.md) |
| DS | 设计系统与主题 | 12 | [DESIGN_SYSTEM](../specification/ui/DESIGN_SYSTEM.zh-CN.md) |
| INT | 交互与命令模型 | 12 | [INTERACTION](../specification/ui/INTERACTION.zh-CN.md) |
| FBK | 反馈与错误 | 10 | [FEEDBACK_ERRORS](../specification/ui/FEEDBACK_ERRORS.zh-CN.md) |
| DAT | 数据完整性与项目生命周期 | 10 | [DATA_INTEGRITY](../specification/ui/DATA_INTEGRITY.zh-CN.md) |
| A11Y | 可访问性 | 8 | [A11Y](../specification/ui/A11Y.zh-CN.md) |
| I18N | 国际化 | 7 | [I18N_L10N](../specification/ui/I18N_L10N.zh-CN.md) |
| PRF | 性能 | 8 | [PERFORMANCE](../specification/ui/PERFORMANCE.zh-CN.md) |
| CMP | 兼容性 | 6 | [COMPATIBILITY](../specification/ui/COMPATIBILITY.zh-CN.md) |
| SEC | 安全与隐私 | 5 | [SECURITY_PRIVACY](../specification/ui/SECURITY_PRIVACY.zh-CN.md) |
| TLM | 遥测与可支持性 | 6 | [TELEMETRY](../specification/ui/TELEMETRY.zh-CN.md) |
| | **合计** | **96** | |

## 3. 登记：SHL — 外壳与布局

| ID | 需求 | 契约 | 验证 | 门禁 | 状态 |
|---|---|---|---|---|---|
| UI-SHL-01 | 三层应用结构（项目、工作区、编辑器） | IA_SHELL §2 | `web/e2e/editors.spec.ts` | G1 | ✅ |
| UI-SHL-02 | 工作区七个固定区域 | IA_SHELL §3 | `web/e2e/shell.spec.ts` | G1 | ✅ |
| UI-SHL-03 | 属性面板右停靠，260–600 px | IA_SHELL §4 | `web/e2e/layout.spec.ts` | G1 | ✅ |
| UI-SHL-04 | 左停靠标签页，240–400 px | IA_SHELL §4 | `web/e2e/layout.spec.ts` | G1 | ✅ |
| UI-SHL-05 | 底停靠默认折叠、带标签、≤40% 高 | IA_SHELL §4 | `web/e2e/bottomDock.spec.ts` | G1 | ✅ |
| UI-SHL-06 | 布局持久化与重置 | IA_SHELL §5 | `web/e2e/layout.spec.ts` | G1 | ✅ |
| UI-SHL-07 | 按专业切换的上下文功能区 | IA_SHELL §3.3 | `web/e2e/ribbon.spec.ts` | G1 | ✅ |
| UI-SHL-08 | 状态栏五项实时读数 | IA_SHELL §6 | `web/e2e/shell.spec.ts`（含开发者 FPS） | G1 | ✅ |
| UI-SHL-09 | 浮动面板与位置持久化 | IA_SHELL §4.4 | 拖动/缩放/重载 e2e · 拖/缩/夹紧/重载断言，`e2e/floating.spec.ts`；面板 id 键控 `omnim.layout.v1.floating[]` | G2 | ✅ |
| UI-SHL-10 | 每页标题与面包屑 | IA_SHELL §2.2 | `web/e2e/editors.spec.ts` | G1 | ✅ |
| UI-SHL-11 | 低于 1280 px 折叠为图标条 | IA_SHELL §7 | 1024×768 截图 · 1024–1279 两侧 44 px 图标条；左侧点击图标重开抽屉；右停靠经条图标开抽屉；`e2e/narrow.spec.ts` | G2 | ✅ |
| UI-SHL-12 | 命令面板 | IA_SHELL §8 | `web/e2e/commands.spec.ts`（Ctrl/Cmd+K、过滤、键盘执行） | G2 | ✅ |

## 4. 登记：DS — 设计系统与主题

| ID | 需求 | 契约 | 验证 | 门禁 | 状态 |
|---|---|---|---|---|---|
| UI-DS-01 | 三层令牌，无原始令牌泄漏 | DESIGN_SYSTEM §2 | `check_ui_contracts.py --tokens` · 令牌门禁通过（colors/colors_func 均 0，styles.css 零字面量色） | G2 | ✅ |
| UI-DS-02 | 深/浅主题切换不重载 | DESIGN_SYSTEM §3 | `theme.test.ts` + `web/e2e/theme.spec.ts`（切换/持久化/对比度） | G1 | ✅ |
| UI-DS-03 | 跟随系统默认，选择持久化 | DESIGN_SYSTEM §3.2 | `theme.test.ts` + `web/e2e/theme.spec.ts`（浅/深媒体模拟） | G2 | ✅ |
| UI-DS-04 | 用户界面样式零硬编码颜色 | DESIGN_SYSTEM §2.4 | `check_ui_contracts.py --only tokens`（0）+ `designTokens.test.ts` | G2 | ✅ |
| UI-DS-05 | 完整语义状态矩阵 | DESIGN_SYSTEM §4 | 令牌矩阵评审 | G3 | 📅 |
| UI-DS-06 | 单一排版刻度，等宽数字 | DESIGN_SYSTEM §5 | 样式扫描 · `font_size_raw` 新门禁=0（styles.css 全部走 --fs-* 刻度）；body 全局 tabular-nums；`e2e/theme.spec` 视觉 | G2 | ✅ |
| UI-DS-07 | 4 px 间距基准，固定圆角与阴影 | DESIGN_SYSTEM §6 | 样式扫描 | G3 | 🔀 |
| UI-DS-08 | z-index 分层 | DESIGN_SYSTEM §7 | `check_ui_contracts.py --only zindex`（0）+ `designTokens.test.ts` | G2 | ✅ |
| UI-DS-09 | 图标尺寸与描边，无 emoji 图标 | DESIGN_SYSTEM §8 | `check_ui_contracts.py --only emoji`（0）+ `Icons.tsx` 单一图标集 | G2 | ✅ |
| UI-DS-10 | 组件状态矩阵完整 | DESIGN_SYSTEM §9 | 组件清单评审 | G3 | 📅 |
| UI-DS-11 | 高对比可访问性主题 | DESIGN_SYSTEM §3.4 | 对比度审计 | G3 | 📅 |
| UI-DS-12 | 双主题视口令牌集 | DESIGN_SYSTEM §10 | 令牌+主题 e2e · 视口令牌层在双主题定义（`globals.css` §viewport）+ 标签可命名；`theme.spec` 断言无重载切换/AA 对比度 | G2 | ✅ |

## 5. 登记：INT — 交互与命令模型

| ID | 需求 | 契约 | 验证 | 门禁 | 状态 |
|---|---|---|---|---|---|
| UI-INT-01 | 五种选择手势 | INTERACTION §3 | 交互 e2e | G1 | 🔀 |
| UI-INT-02 | 拾取状态机契约 | INTERACTION §4 | `stateMachine.test.ts` | G1 | ✅ |
| UI-INT-03 | 单一快捷键注册表，文档生成 | INTERACTION §5 | `gen_keymap.py --check` · 32 绑定中英一致；命令数组收敛 `app/commands.ts` 单一产地；`global_keydown` 门禁收口 window 监听 allow-list | G2 | ✅ |
| UI-INT-04 | 输入时不触发全局快捷键 | INTERACTION §5.3 | `web/e2e/shell.spec.ts` | G1 | ✅ |
| UI-INT-05 | 上下文菜单且可键盘唤起 | INTERACTION §6 | 菜单 e2e · Shift+F10 键盘唤起+menu/menuitem 角色与方向键/焦点（`e2e/a11y.spec` context menu） | G2 | ✅ |
| UI-INT-06 | 统一对话框与焦点管理 | INTERACTION §7 | `useDialogFocus.ts` + `web/e2e/dialogs.spec.ts`（焦点陷阱/背景 inert/焦点归还） | G2 | ✅ |
| UI-INT-07 | 破坏性确认并说明影响 | INTERACTION §7.3 | `deleteImpact.ts` 单测 + `web/e2e/dialogs.spec.ts`（子对象计数/两路径确认） | G2 | ✅ |
| UI-INT-08 | 全变更撤销/重做，深度 ≥50 | INTERACTION §8 | `crates/server/tests/write_paths.rs`（60/200 深度与 redo）+ `datum.rs` 等端点覆盖 | G2 | ✅ |
| UI-INT-09 | 自动命名与重名检测 | INTERACTION §9 | 命名单测 | G1 | ✅ |
| UI-INT-10 | 单位一致且带后缀 | INTERACTION §9.2 | 表单审计 · CreatePanel 数值域全部带 .field-unit 后缀（m/mm）；目录参数回显 {value} {unit}；`e2e/units.spec.ts` | G2 | ✅ |
| UI-INT-11 | 每个上下文双击含义唯一 | INTERACTION §7.4 | 上下文表评审 · 对象树行=就地重命名、分隔条双击=复位宽度；`e2e/a11y.spec`（F2/双击）与gestures | G2 | ✅ |
| UI-INT-12 | 可配置视口导航 | INTERACTION §10 | 设置 e2e · 导航方案（current 发版行为 / contract 目标：左惰·中轨道·右平移）经命令面板切换、`omnim.nav` 持久化并应用到视口；`e2e/navScheme.spec`（现偏差仍登记 CODE_DEVIATIONS §7） | G2 | ✅ |

## 6. 登记：FBK — 反馈与错误

| ID | 需求 | 契约 | 验证 | 门禁 | 状态 |
|---|---|---|---|---|---|
| UI-FBK-01 | 每个面板三态齐全 | FEEDBACK_ERRORS §3 | `web/e2e/routeCrash.spec.ts` + 面板清单 | G1 | ✅ |
| UI-FBK-02 | 骨架屏，400 ms 内不闪烁 | FEEDBACK_ERRORS §3.2 | `web/src/__tests__/delayedVisibility.test.ts` + 面板覆盖 | G1 | ✅ |
| UI-FBK-03 | 超过 1 秒显示进度并可取消 | FEEDBACK_ERRORS §4 | 导入 e2e · 导入 >1 s 弹不定进度+已耗时+取消；`e2e/importProgress.spec.ts`；服务端取消端点登记 backlog | G2 | ✅ |
| UI-FBK-04 | 带码错误、原因与恢复动作 | FEEDBACK_ERRORS §5 | 错误目录比对 | G3 | 📅 |
| UI-FBK-05 | 无原生 alert/confirm/prompt | FEEDBACK_ERRORS §6 | `window.*` 扫描 | G1 | ✅ |
| UI-FBK-06 | 路由与面板级错误边界 | FEEDBACK_ERRORS §7 | `web/e2e/routeCrash.spec.ts` + 面板级 `feedback.spec.ts` | G1 | ✅ |
| UI-FBK-07 | 分级提示；错误常驻 | FEEDBACK_ERRORS §8 | `web/e2e/feedback.spec.ts` | G1 | ✅ |
| UI-FBK-08 | 可关闭且不遮挡的成功反馈 | FEEDBACK_ERRORS §8.2 | `web/e2e/feedback.spec.ts` + 截图 | G1 | ✅ |
| UI-FBK-09 | 空状态带下一步动作 | FEEDBACK_ERRORS §3.3 | `web/e2e/home.spec.ts` + 面板清单 | G1 | ✅ |
| UI-FBK-10 | 诊断包导出 | FEEDBACK_ERRORS §9 | 诊断包检查 | G3 | 📅 |

## 7. 登记：DAT — 数据完整性与项目生命周期

| ID | 需求 | 契约 | 验证 | 门禁 | 状态 |
|---|---|---|---|---|---|
| UI-DAT-01 | 项目首页含新建/打开/最近/示例 | DATA_INTEGRITY §2 | `web/e2e/home.spec.ts` | G1 | ✅ |
| UI-DAT-02 | 显式保存与另存为，失败可重试 | DATA_INTEGRITY §3 | `web/e2e/data.spec.ts`（Save/Save-as/失败重试） | G1 | ✅ |
| UI-DAT-03 | 脏标记与最后保存时间 | DATA_INTEGRITY §3.2 | `web/e2e/data.spec.ts` | G1 | ✅ |
| UI-DAT-04 | 自动保存 ≤60 秒且不阻塞 | DATA_INTEGRITY §4 | 自动保存追踪 · 每次 mutation 即持久化；盘失败仅记日志不阻塞；`project_meta.rs` | G2 | ✅ |
| UI-DAT-05 | 崩溃恢复损失 ≤5 分钟 | DATA_INTEGRITY §5 | 杀进程重启演练 · mutation 即快照 → 零丢失；`Project::load` 重开断言在 `project_meta.rs` | G2 | ✅ |
| UI-DAT-06 | 未保存变更拦截 | DATA_INTEGRITY §3.3 | `web/e2e/data.spec.ts`（Stay/Leave/clean） | G1 | 🔀 |
| UI-DAT-07 | 项目元信息可编辑并持久化 | DATA_INTEGRITY §2.3 | 重开断言 · `/api/project/meta` PUT/GET + 标题栏「项目设置」对话框；重命名文件事务；`e2e/meta.spec.ts` | G2 | ✅ |
| UI-DAT-08 | 外部修改检测与只读 | DATA_INTEGRITY §6 | mtime 冲突测试 | G3 | 📅 |
| UI-DAT-09 | 打开时缺失依赖报告 | DATA_INTEGRITY §2.4 | 夹具项目测试 | G3 | 📅 |
| UI-DAT-10 | 破坏性操作快照 | DATA_INTEGRITY §7 | 快照恢复测试 | G3 | 📅 |

## 8. 登记：A11Y — 可访问性

| ID | 需求 | 契约 | 验证 | 门禁 | 状态 |
|---|---|---|---|---|---|
| UI-A11Y-01 | 关键路径 WCAG 2.2 AA | A11Y §2 | `web/e2e/axe.spec.ts`（首启/打开/创建/选中/保存/错误恢复，0 violations） | G2 | ✅ |
| UI-A11Y-02 | 命令键盘完备且有帮助 | A11Y §3 | `commands/focus/a11y` specs（F1、命令面板、纯键盘 J1、F2、属性行方向键、F6、菜单） | G2 | ✅ |
| UI-A11Y-03 | 焦点可见、有序、受限 | A11Y §4 | `web/e2e/focus.spec.ts`（:focus-visible 环、F6、删除后迁移、路由/错误兜底） | G2 | ✅ |
| UI-A11Y-04 | 树/表/标签的 role 与 ARIA | A11Y §5 | `web/e2e/a11y.spec.ts`（tree/tablist/toolbar/menu/table/status/viewport） | G2 | ✅ |
| UI-A11Y-05 | 无障碍名称；视口文本替代 | A11Y §6 | axe + 人工评审 | G2 | 📅 |
| UI-A11Y-06 | 减弱动效支持 | A11Y §7 | 媒体模拟测试 | G3 | 📅 |
| UI-A11Y-07 | 高对比主题达到 AA | A11Y §8 | 对比度审计 | G3 | 📅 |
| UI-A11Y-08 | 结果播报 | A11Y §9 | 读屏脚本 | G3 | 📅 |

## 9. 登记：I18N — 国际化

| ID | 需求 | 契约 | 验证 | 门禁 | 状态 |
|---|---|---|---|---|---|
| UI-I18N-01 | 零硬编码用户字符串 | I18N_L10N §2 | 字符串扫描 · 至此外置：外壳全量 + 域面板全量（节/动作/域下拉）+ CreatePanel + LayerEditor + AssemblyPanel（title/单位说明/名称/父级/子件表/添加/类别/选目录件/错误 toast + shapeLabels）；CJK 809；余：Catalog 侧栏、模板/材料默认名（模型数据范围外） | G2 | 🔀 |
| UI-I18N-02 | zh-CN 默认、en-US 回退、即时切换 | I18N_L10N §3 | 语言 e2e · `i18n.ts` 内核 + 标题栏切换器；`language.spec.ts`：即时切换/持久化/`<html lang>` | G2 | ✅ |
| UI-I18N-03 | ICU 复数格式 | I18N_L10N §4 | ICU 单测 · `tPlural()` 走 `Intl.PluralRules`；词典 other 声明；`i18n.test.ts` | G2 | ✅ |
| UI-I18N-04 | 语言环境感知的数字/日期/单位 | I18N_L10N §5 | 格式夹具 · 坐标 3 位小数/分组计数/短日期/时间制/角度格式器 + 夹具；面板消费分批接入中 | G2 | ✅ |
| UI-I18N-05 | 伪本地化通过 | I18N_L10N §6 | 伪语言截图 | G3 | 📅 |
| UI-I18N-06 | 键名规范与未使用键检测 | I18N_L10N §7 | 键校验 | G3 | 📅 |
| UI-I18N-07 | RTL 结构就绪 | I18N_L10N §8 | 逻辑属性扫描 | G3 | 📅 |

## 10. 登记：PRF — 性能

| ID | 需求 | 契约 | 验证 | 门禁 | 状态 |
|---|---|---|---|---|---|
| UI-PRF-01 | 首屏与就绪预算 | PERFORMANCE §3 | 性能夹具 | G2 | 📅 |
| UI-PRF-02 | 交互延迟预算 | PERFORMANCE §3.2 | 交互追踪 | G2 | 📅 |
| UI-PRF-03 | 各构件数下帧率预算 | PERFORMANCE §4 | 帧时间夹具 · `e2e/perf.spec` PRF-03/04 段记录 sample/F+100构件 FPS 与 idle 采样；无头 SwiftShader 下 FPS 低且 idle busy=1（持续渲染循环 B9 未解）→ 🔀 | G2 | 🔀 |
| UI-PRF-04 | 空闲 CPU 预算 | PERFORMANCE §4.3 | 空闲采样 · 夹具已记录 idle busy 比值；持续渲染循环（B9）未解 → 🔀 | G2 | 🔀 |
| UI-PRF-05 | 内存上限与浸泡 | PERFORMANCE §5 | 浸泡测试 | G3 | 📅 |
| UI-PRF-06 | 包体预算与分包 | PERFORMANCE §6 | 包体报告 · index gzip 141 KB（≤500）、CSS 16.6 KB（≤60）、three 懒分块（OrbitControls）、编辑/库/系列路由独立分块；`e2e/perf.spec` PRF-06 段 | G2 | ✅ |
| UI-PRF-07 | 5000 节点树滚动 | PERFORMANCE §7 | 滚动追踪 | G3 | 📅 |
| UI-PRF-08 | ±10% 回退门禁 | PERFORMANCE §8 | CI 任务 | G3 | 📅 |

## 11. 登记：CMP — 兼容性

| ID | 需求 | 契约 | 验证 | 门禁 | 状态 |
|---|---|---|---|---|---|
| UI-CMP-01 | 支持的浏览器矩阵 | COMPATIBILITY §2 | 矩阵运行 | G3 | 📅 |
| UI-CMP-02 | WebGL2 检测与阻断 | COMPATIBILITY §3 | 强制失败测试 | G3 | 📅 |
| UI-CMP-03 | DPI 与缩放矩阵 | COMPATIBILITY §4 | 截图矩阵 | G3 | 📅 |
| UI-CMP-04 | 3D 鼠标支持或明确排除 | COMPATIBILITY §5 | 决策记录 | G3 | 📅 |
| UI-CMP-05 | 1280×720 起可用 | COMPATIBILITY §4.2 | 最小尺寸走查 · 1280×720 全布局可用（`e2e/narrow.spec` CMP-05 段） | G2 | ✅ |
| UI-CMP-06 | 离线/弱网降级 | COMPATIBILITY §6 | 离线横幅测试 | G3 | 📅 |

## 12. 登记：SEC — 安全与隐私

| ID | 需求 | 契约 | 验证 | 门禁 | 状态 |
|---|---|---|---|---|---|
| UI-SEC-01 | 无鉴权时声明仅限本机 | SECURITY_PRIVACY §2 | 界面文案评审 | G3 | 📅 |
| UI-SEC-02 | 权限门控，无权限时隐藏 | SECURITY_PRIVACY §3 | 角色矩阵测试 | G3 | 📅 |
| UI-SEC-03 | 打开项目限制在允许根目录 | SECURITY_PRIVACY §4 | 路径穿越测试 | G3 | 📅 |
| UI-SEC-04 | CSP 与不可信 HTML 防护 | SECURITY_PRIVACY §5 | 响应头 + 代码扫描 | G3 | 📅 |
| UI-SEC-05 | 遥测选择加入且无 PII | SECURITY_PRIVACY §6 | 同意门 + 载荷审计 | G3 | 📅 |

## 13. 登记：TLM — 遥测与可支持性

| ID | 需求 | 契约 | 验证 | 门禁 | 状态 |
|---|---|---|---|---|---|
| UI-TLM-01 | 带版本的事件字典 | TELEMETRY §2 | 字典比对 | G3 | 📅 |
| UI-TLM-02 | 五步漏斗埋点 | TELEMETRY §3 | 会话漏斗日志 | G3 | 📅 |
| UI-TLM-03 | 错误事件带码且不含模型内容 | TELEMETRY §4 | 载荷审计 | G3 | 📅 |
| UI-TLM-04 | 性能分位数 | TELEMETRY §5 | 指标采样 | G3 | 📅 |
| UI-TLM-05 | 可导出本地诊断且不含几何 | TELEMETRY §6 | 诊断包检查 | G3 | 📅 |
| UI-TLM-06 | 快捷键帮助、上下文帮助、示例、关于 | TELEMETRY §7 | 帮助入口走查 | G3 | 📅 |

## 14. 验证物清单

**只有 ✅ 行今天真实存在。** 标 📅 的是计划产物；若某条需求的唯一验证物是 📅，该需求**不可验证**，不能通过门禁。§3–§13 中部分行引用了计划产物（例如"截图集 `shell`"），现在不计为证据。

| 产物 | 类型 | 状态 | 覆盖 |
|---|---|---|---|
| `web/src/__tests__/stateMachine.test.ts` | 单元 | ✅ | UI-INT-02 |
| `web/src/__tests__/genesisForm.test.ts` | 单元 | ✅ | UI-INT-02, UI-INT-10 |
| `web/src/__tests__/previewGeometry.test.ts` | 单元 | ✅ | UI-INT-02 |
| `web/src/__tests__/storeyView.test.ts` | 单元 | ✅ | 楼层过滤（`meshVisibleOnStorey`）——尚无专属需求（Q4） |
| `web/src/__tests__/datumGrids.test.ts` | 单元 | ✅ | 轴网渲染——尚无专属需求（Q4） |
| `scripts/check_docs.py` | 扫描 | ✅ | 文档结构与双语对齐 |
| `scripts/check_ui_contracts.py` | 扫描 | ✅ | ID/快捷键/错误码结构；债务计数（`--strict` = 零债务门禁） |
| `scripts/gen_keymap.py --check` | 扫描 | ✅ | UI-INT-03（仅表内一致性，尚未对照代码） |
| `web/tests/e2e/*.spec.ts` | e2e | 📅 | SHL、INT、FBK、DAT、I18N |
| `web/tests/visual/*.spec.ts` | 视觉 | 📅 | DS、SHL、A11Y |
| `web/tests/a11y/*.spec.ts` | 可访问性 | 📅 | A11Y-01/04/05/06 |
| `web/tests/perf/*.spec.ts` | 性能 | 📅 | PRF-01…08 |
| 截图集（`docs/img/ui/`） | 视觉 | 📅 | SHL-02/03/05, DS-02/12 |

**规则：** 新增需求必须引用一个已存在的 ✅ 产物，或新增带责任人与门禁的 📅 产物。未验证的需求阻塞门禁。

## 15. 门禁覆盖

| 门禁 | 需求数 | 特征 |
|---|---|---|
| G0 | 0（仅契约） | PRD、IA、DS、INTERACTION 通过评审 |
| G1 | 25 | 外壳可用、无白屏、无原生弹窗、保存可用 |
| G2 | 37 | 双主题、核心 i18n、关键路径可访问性、性能、完整反馈 |
| G3 | 34 | 完备度层级、兼容性、安全、遥测、可支持性 |

计数来自上表的门禁列；每次变更需重新统计。

## 16. 状态汇总

| 状态 | 数量 | 含义 |
|---|---|---|
| ✅ | 1 | 已实现并有验证物佐证 |
| 🔀 | 17 | 部分实现，契约中说明剩余部分 |
| 📅 | 78 | 已计划，尚未实现 |
| 💭 | 0 | 仅提案 |

**解读：** 前端目前处于 G0，已有可用原型。距 G3 差 78 条纯计划需求，另有 17 条部分实现——不是"调样式"能解决的。

## 17. 变更记录

| 日期 | 变更 | 责任人 |
|---|---|---|
| 2026-10-29 | 建立登记册，覆盖 11 个域、96 条需求 | 前端文档基线 |

## 18. 非目标

1. 跟踪后端或 IFC 需求。
2. 估算单条需求的工作量。
3. 取代 PRD 作为需求的规范性描述。

## 19. 待定问题

| # | 问题 | 责任人 |
|---|---|---|
| Q1 | 部分实现的需求是否拆分子 ID，让每个 `🔀` 行可独立追踪？ | 产品 / 前端 |
| Q2 | 性能需求是否在 CI 配置中再写一份，作为权威阈值？ | 前端 |
| Q3 | 状态变更由谁签字：产品还是实施工程师？ | 产品 |
| Q4 | 楼层视图、剖切、测量持久化有旅程但无需求 ID。新增 `UI-INT-13…15`，还是视为登记册之外的视口工具？ | 产品 / 前端 |
| Q5 | 字符串扫描是否应同时检出硬编码英文？目前只检出 CJK（`I18N_L10N` §2）。 | 前端 |
