---
title: "无障碍人工评审套件（UI-A11Y-05）"
status: current
last_updated: 2026-11-27
applies_to: "web/"
evidence: "checklist + captured aria snapshots for the reviewer sign-off"
---
# 无障碍人工评审套件（UI-A11Y-05）

> 机器部分已由 `e2e/axe.spec.ts`（WCAG 2.2 AA 关键路径 0 严重违规）与 `e2e/a11y.spec.ts`（名称/角色/键盘）覆盖。本套件补足**真实读屏软件**无法自动化的最后一段：朗读顺序、措辞可理解性、视口文本替代是否足够。

## 1. 环境
| 项 | 要求 |
|---|---|
| 浏览器 | Chrome/Edge 最新稳定版 |
| 读屏 | Windows：NVDA 2026.x；macOS：VoiceOver（Safari/Chrome）；Linux 可选 Orca |
| 数据 | 项目首页 → Open sample project（示例工程）；再准备一个含 3 个对象的工程 |
| 参照 | `docs/specification/ui/A11Y.zh-CN.md` §6（名称与替代方案） |

## 2. 评审脚本（逐项勾选）
| # | 场景 | 操作 | 期望朗读 | 结果 |
|---|---|---|---|---|
| 1 | 项目首页 | Tab 遍历卡片与按钮 | 卡片名称、路径、修改时间；按钮有明确动作词 | ☐ |
| 2 | 工作区进入 | 打开示例工程 | 标题栏区域名 + 当前项目名；无“按钮 按钮”类重复 | ☐ |
| 3 | 左停靠 | 对象树上下键移动 | tree/treeitem 角色、层级、选中态；Shift+单击播报范围结果 | ☐ |
| 4 | 视口 | 聚焦 `3D viewport` | 朗读对象数、相机模式、楼层视图（`#viewport-description`） | ☐ |
| 5 | 场景描述 | 命令面板 → Describe scene | 通知区播报 `场景描述：N 个对象…` | ☐ |
| 6 | 属性面板 | 树中选中对象后 Tab | 表格角色、列数、行标签/值成对朗读 | ☐ |
| 7 | 创建表单 | 打开创建面板并填写 | 每个数值框：可见标签 + 单位；错误提示可感知 | ☐ |
| 8 | 离场守卫 | 弄脏后点返回 | 对话框标题/项目名/上次保存时间；Esc=取消、Enter=保存 | ☐ |
| 9 | 通知 | 保存成功/失败 | `role=status` 区域及时播报，不打断输入 | ☐ |
| 10 | 语言切换 | 切到 en-US 再复测 2–9 | 全部名称/描述随语言变化，无残留中文 | ☐ |

## 3. 记录与签收
1. 失败项记录：场景编号、读屏/浏览器版本、实际朗读文本、期望文本、截图或录音。
2. 修复后回归对应编号；全表勾选后在 [REQUIREMENTS_REGISTER](../product/REQUIREMENTS_REGISTER.zh-CN.md) `UI-A11Y-05` 证据列填入评审人/日期/构建版本。
3. 机器快照：`web/e2e/a11ySnapshot.spec.ts` 会把关键表面的 aria 树摘要写入 `/tmp/opencode/a11y-snapshots.json`，供评审前对照。

## 4. 通过标准
- 10/10 场景通过，或剩余项记录为带日期的已知问题且不影响关键路径。
- `UI-A11Y-05` 由评审人签字后方可置 ✅；机器检查通过不等于读屏体验通过。
