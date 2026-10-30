---
title: "建模师验收复验记录（2026-10-30 第二轮）"
status: history
last_updated: 2026-10-30
applies_to: "docs/, web/, crates/server"
evidence: "逐条复核 MODELER_DOCS_AUDIT_2026-10-30 P1/P2 处置并对照代码 2026-10-30"
languages: zh-CN
---
# 建模师验收复验记录(2026-10-30,针对首轮审计报告的整改)

> **复验方式**:逐条对照首轮报告(`MODELER_DOCS_AUDIT_2026-10-30.zh-CN.md`)P1-1~P1-6、P2-1~P2-3,重跑 `check_docs.py`,并对整改后文档的**新增断言**反向核查代码证据。

## 1. 结论:**正式通过(Acceptance Pass)**

首轮 6 项 P1 全部关闭,3 项 P2 关闭 2 项;整改未引入新的失实断言。残余 3 项记为 P3,不阻塞验收。

## 2. 逐项处置核验

| 首轮项 | 处置 | 复验证据 | 结论 |
|---|---|---|---|
| P1-1 撤销章节三源矛盾 | 手册 §1.3/§1.2 重写 | `MODELER_HANDBOOK.md:21,34`(transform "is undoable";undo 覆盖端点全列)与 `WRITE_PATHS.md`/代码 `lib.rs:715,975` 一致 | ✅ 关闭 |
| P1-2 项目主页过时 | §1.5 更新为 /projects 跳转 | `MODELER_HANDBOOK.md:42`;`App.tsx:98` 无会话时 `navigate('/projects',{replace:true})` | ✅ 关闭 |
| P1-3 storey 标高过时 | §2.6 重写 | `MODELER_HANDBOOK.md:95`(elevation→`IfcBuildingStorey.Elevation`、P8-S2/S4、残余仍注明)与 `CAPABILITY_MATRIX.md:106` 一致 | ✅ 关闭 |
| P1-4 Save 按钮/快捷键 | 手册 §6.1 + 键表更新 | `MODELER_HANDBOOK.md:164`(标题栏 Save/Save as)↔ `TitleBar.tsx:73-76` 实存;`KEYBOARD_MAP.md:29` Ctrl/Cmd+S 标 ✅ 并在 :42 注明 Ctrl+Shift+S 行为 | ✅ 关闭 |
| P1-5 按钮名 Assy | 改为 Assembly | `MODELER_HANDBOOK.md:151` ↔ `MainToolbar.tsx:416` | ✅ 关闭 |
| P1-6 门禁 FAIL | front matter 补齐 | `CODE_AUDIT_2026-10-30.zh-CN.md:1-8` 有 front matter;`check_docs.py` → `OK: docs checks passed`(0 failure) | ✅ 关闭 |
| P2-1 快速开始自相矛盾警告 | 遗留首行删除 | `GETTING_STARTED.md:23-28` 表述单一(默认开启+逃生口+两例外) | ✅ 关闭 |
| P2-2 双语缺对 | 部分处置 | `CODE_AUDIT` 加 `languages: zh-CN` 声明;`DOC_AUDIT_2026-10-27` 仍仅中文且**无**豁免声明 | ⚠️ 部分,转 P3 |
| P2-3 research 目录过期 | 归档声明 | `research/README.md:3` `status: archived` + 目录级"非契约文档"说明(gca 36dcd1d) | ✅ 关闭 |

## 3. 新增断言正面核查(防止修复引入新失实)

- **Save as 全链路**:`TitleBar Save as` → `appStore.saveProjectAs` → `api.ts:47` `POST /api/project/save-as` ↔ `lib.rs:74,283` 路由与 handler 实存。✔
- **/projects 无会话重定向**:`App.tsx:98`。✔
- **路由总数不变**:86 去重路径,与 API.md 口径一致。✔

## 4. 残余项(转 P3,不阻塞)

| ID | 事项 | 说明 |
|---|---|---|
| R1 | `MODELER_HANDBOOK.md:34`/`CAPABILITY_MATRIX.md:117` "每端点可撤销"对 `POST /api/catalog/import` 的-WRITE_PATHS 例外表(❌ undo)存在轻微过度声明 | `GETTING_STARTED.md:28` 已正确列明两例外,建议手册同步补一句 |
| R2 | `DOC_AUDIT_2026-10-27.zh-CN.md` 无双语豁免声明 | 补一行说明或在下批补 EN |
| R3 | `check_docs.py --strict-parity` 下 `GLOSSARY.md`/`.zh-CN.md` 标题数 74 vs 75 内建豁免项 | 已知既有不匹配,非本轮回归,建议择期清零以解除永久豁免 |

## 5. 签注

- 复验脚本状态:`check_docs.py → OK`(默认门禁 0 failure,56 对双语检查/0 断链/0 围栏失败)。
- 整改批次:`8b049c2`(P1 全清+P2 闭环)、`36dcd1d`(research 归档/双语声明/写路径防呆)。
- 复验性质:基于首轮审报告的独立复核,未自评分数;建议下次全量评审时按 G0 §6 惯例重打分。
