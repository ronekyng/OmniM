---
title: "建模师 UI/UX 验收复验记录(第四轮)"
status: history
last_updated: 2026-10-30
applies_to: "web/, docs/status/CODE_DEVIATIONS.md"
evidence: "逐项复核 MODELER_UIUX_AUDIT_2026-10-30 P0-P2 处置;check_docs/check_ui_contracts/tsc/vitest 2026-10-30"
languages: zh-CN
---
# 建模师 UI/UX 验收复验记录(2026-10-30,针对第三轮 UI/UX 报告的整改)

> **复验方式**:逐条对照第三轮报告(`MODELER_UIUX_AUDIT_2026-10-30.zh-CN.md`)的 P0×2 / P1×2 / P2×3 发现,代码定位核证 + 门禁与测试套件全量重跑。

## 1. 结论:**UI/UX 层复验通过**

首轮 UI/UX 报告全部 7 项发现已处置或立项;**两个 P0-UX(假警报、单位陷阱)均已修复并有同日台账留痕**;门禁与前端测试全绿。

## 2. 逐项处置核验

| 首轮项 | 级别 | 处置 | 复验证据 | 结论 |
|---|---|---|---|---|
| F1 "Unsaved changes" 假警报 | P0-UX | 保存语义统一为"服务端自动落盘 + Ctrl+S 快照"双层叙事 | `saveState.ts:57` dirty 文案改为 "Auto-saved server-side · Ctrl+S to snapshot";`TitleBar.tsx:61` aria 同步;`CODE_DEVIATIONS 7.12` 当日 Resolved 留痕 | ✅ 关闭 |
| F4 参数面板单位陷阱 | P0-UX | 同屏单位标注 | `AssemblyPanel.tsx:36-92`:面板顶部 mm/m 口诀 + 输入占位符 `${a} (m)`/"X 偏移 (m)";`LayerEditor.tsx:192,210` 厚度 "(mm)" 徽标;`CODE_DEVIATIONS 7.13` 当日 Resolved 留痕 | ✅ 关闭 |
| F2 组件编辑器"第三种蓝" | P1 | 调色并轨 | `SketchCanvas3D.tsx:31` 背景 `0x1a1a2e` → `0x1a1d23`(主壳 token) | ✅ 关闭 |
| E2 约束工具栏图标无语义 | P1 | 补齐 tooltip | `EditorToolbar.tsx` 16 处 `title=`,约束带快捷键提示(title 含 `(key)`) | ✅ 关闭 |
| B2 "0/0 贯通"实现术语外泄 | P2 | 语义开关 | `GridPanel.tsx:153-162` "贯通整个视图(±20 m)" checkbox + :193 帮助文案 | ✅ 关闭 |
| A3 时间戳自动命名 | P2 | 序号命名 | `shared/naming.ts` `nextAutoName()`:`<prefix>-<nnn>` 三位序号、按前缀复用释放号、注释明示"前缀不本地化为交换标识符"(UI-INT-09) | ✅ 关闭 |
| D2 "Box 方块"拼接文本 | P2 | 中文优先并注英 | `AssemblyPanel.tsx:73` "方块 (Box)" | ✅ 关闭 |

## 3. 门禁与测试(全量重跑)

| 检查 | 结果 |
|---|---|
| `scripts/check_docs.py` | OK(0 断链 / 0 front-matter 失败 / 56 对双语) |
| `scripts/check_ui_contracts.py` | OK,0 failures(colors 720 / func 57 / cjk 884 如实记账) |
| `web` `tsc --noEmit` | 0 错误 |
| `web` `vitest run` | **10 文件 / 100 用例全部通过**(356 ms) |

## 4. 文档侧闭环

- `CODE_DEVIATIONS.md:190` P10 addendum 登记 7.12/7.13 当日立项并关闭,原文档叙述未被抹改(审美上乘的台账做法);
- `FRONTEND_MATURITY` §5 保存语义改述为"快照分层"叙事(由 7.12 引用);
- 整改批次:`455fd15`(P0)、`d9cce51`(剩余项)、`a32defa`(立账)、`0daa7a3`(周期收口)。

## 5. 残余项(全部为 G2+ 范围,不阻塞当前验收档位)

| 项 | 状态 |
|---|---|
| F3 中英混排(858→884 CJK 计数上升为新增控件所致) | 7.9 开放,G2 门 |
| 双主题完全归一(widget仍在 token 之外少量独立色) | 7.8 开放,G2 门 |
| A11Y、错误码呈现、性能预算 | A11Y 0%、TLM 0%,G2/G3 门 |

## 6. 签注

以"可用原型(G1)档"的 UI/UX 验收**正式放行**:建模师主线工作流无假警报、无单位陷阱,界面语言/主题/语义在可接受限度内。G2 条目的距离已由文档如实标定,无需复审第三者再行判定。

> 复验性质:基于第三轮报告的独立代码级复核,非自评;建议 G2 门前按 ISO 25010 重打分(FRONTEND_MATURITY §6)。
