---
title: "建模师视角文档体系验收审核报告"
status: history
last_updated: 2026-10-30
applies_to: "docs/, web/, crates/server, crates/model, crates/ifc"
evidence: "audited against crates/server/src/lib.rs, web/src, scripts/check_docs.py 2026-10-30"
---
# 建模师视角文档体系验收审核报告（2026-10-30）

> 本文为单语言（zh-CN）快照，不在 en/zh 双语配对范围内（对应本报告 P2-2）。
>
> **整改进度（2026-10-30，同日闭环）**：P1-1（英版手册 §1.3 撤销声明与中版/WRITE_PATHS 对齐）、P1-2（§1.5 改为 `/projects` 已落地，双语）、P1-3（§2.6 更正标高/级联/楼层视图现状，双语）、P1-4（`KEYBOARD_MAP` 双语 Ctrl+S → ✅ 并注明 `Ctrl+Shift+S` 现行为；§6.1 改为标题栏 Save/Save as 已有，双语）、P1-5（§5.2 Assy → Assembly，双语）、P1-6（本报告族 `CODE_AUDIT_2026-10-30.zh-CN.md` 补 front matter）；P2-1（英版 GETTING_STARTED 删除矛盾遗留行）；P2-2（两份 zh-only 审计报告加双语范围声明；DOC_AUDIT_2026-10-27 同法补充）；P2-3（`docs/research` 全部标注 `status: archived`，README 顶部注明非契约文档性质）。核验：`check_docs.py` `OK (0 failures)`、`gen_keymap --check` ✅、`check_ui_contracts.py` ✅、`check_write_paths.py` ✅（并已新增"扫描不到 handler 即 FAIL"防呆）。P3 项（UI 内近似几何提示、错误码启用里程碑标注）记入产品后续批次。

> **审计角色**：建模师（文档的最终使用者——用文档完成任务、交付工程模型的人），按软件验收标准评审文档的**设计、功能、使用、操作**四个维度。
> **审计对象**：`docs/` 全部 130 篇 Markdown（重点：`MODELER_HANDBOOK`、`CAPABILITY_MATRIX`、`guides/`、`reference/`、`status/`）、根 `README`。
> **审计方法**：把文档中可验证断言逐条对照代码（`crates/server`、`web/src`）；跑 `python3 scripts/check_docs.py`；检查链接、双语配对、front matter 时效；并对照既有审计记录（`DOC_AUDIT_2026-10-27`、`G0_REVIEW_2026-10-29`）核对遗留项闭环情况。
> **报告性质**：验收审核快照；行号以审计时工作树为准。

## 1. 总体结论：**有条件通过（Conditional Pass）**

文档体系的设计水平与诚实度在同类项目中属上乘：审计维度齐全、断言大多有 evidence 行可溯源、链接零断链、API 路由数与代码逐条吻合。**但存在一个会在验收现场直接误导建模者的"用户手册滞后"问题簇**（§3 P1-1~P1-5),以及一项**文档门禁自身未通过的实质缺陷**（§3 P1-6）。修复后可转正式通过。

### 记分卡

| 维度 | 得分 | 说明 |
|---|---|---|
| 设计（体系架构） | 4.5 / 5 | 按受众/意图组织的目录 + 源真层级声明，结构优异 |
| 功能（内容真实性） | 4 / 5 | 抽查断言与代码吻合度高，但手册有 4 处滞后、1 处快捷键表失实 |
| 使用（可上手性） | 4.5 / 5 | 五分钟快速开始、双单位（mm/m）规则、快捷键均可执行 |
| 操作（运维/门禁） | 3.5 / 5 | `check_docs.py` 当前 FAIL(1)；双语配对缺 2 对 |
| 可维护性 | 4.5 / 5 | front matter 全覆盖(129/130)，last_updated 2026-10-27 起占 92% |

## 2. 符合项（验收亮点，均有代码证据）

| 维度 | 符合项 | 证据 |
|---|---|---|
| 设计 | 按受众（建模/试用/理解系统/契约/UI/查询/现状）组织入口；"architecture+status=实现了什么，specification+RFC=设计意图"的源真层级明确 | `docs/README.md:12-14, 16-100` |
| 设计 | 744 条内部链接零断链；`check_docs.py` 门禁覆盖链接/围栏/编号/front matter | 门禁输出：broken 0；对比 G0 整改前 28 对文档日期过期问题已闭环 |
| 功能 | API 参考路由数与代码**完全一致**：文档自称 86 去重/87 注册/1 路径双注册，实测 `crates/server` 恰为此数 | `grep .route( crates/server/src/lib.rs` = 87；去重 = 86 |
| 功能 | 能力矩阵对缺失功能不粉饰：Clash/算量/出图全层 ❌、IFC 导出代理限制、系列门仅可达性、proto 155 型中 104 型仅规划 | `CAPABILITY_MATRIX.md:83,98,113-120` |
| 功能 | 手册操作流程抽查 10 项中 8 项与代码一致：建筑→墙体菜单、屋顶两模式、门窗必墙提示、工厂→管道、`/api/genrule`、catalog 实例化路径 | `MainToolbar.tsx:438`、`CreatePanel.tsx:164-182,404`、`DomainPanels.tsx:59-108`、`api.ts:90` |
| 功能 | 错误码表诚实声明"全部为保留码、现版本不发射"，实测 crates/web 中 0 处出现，与声明吻合 | `ERROR_CODES.md §Honesty rule` |
| 使用 | 双单位规则（catalog=mm、视口/genrule=m）明确且与代码行为一致；吸附阈值 0.5 m/网格 1.0 m 与实现一致 | `MODELER_HANDBOOK.md §1.1/1.4`、`Snapping.ts:30-31` |
| 使用 | 快捷键抽查 13 项中 12 项与 `web/src` 绑定一致（W/E 移旋、F/H、G/O/X、Del 组合、Enter/Backspace/Esc 同步、`.`,`/`)；控制台/AI 面板/草图键一致 | `App.tsx:108-155`、`OmniPanel.tsx:416-444`、`ComponentEditorPage.tsx:199-219` |
| 操作 | G0 审阅记录化：P0×6/P1×16/P2×15 逐条处置并留证据；17 项 P1 已闭环、签发条件明确；残余项 O1–O6 有 owner/gate | `G0_REVIEW_2026-10-29.md:38-110` |
| 操作 | 需求 → 门 → 验证的三向追踪矩阵（RTM）、放行清单、DoD 就绪,验收链路文档齐备 | `product/REQUIREMENTS_REGISTER.md`、`process/RELEASE_CHECKLIST.md`、`process/DEFINITION_OF_DONE.md` |

## 3. 问题清单（按验收影响排序）

### P1（必须在验收前整改）

| # | 问题 | 证据 | 影响 |
|---|---|---|---|
| P1-1 | **用户手册撤销章节与能力矩阵/写入路径/代码三者矛盾**：手册 §1.3 称 transform/属性/连接/材质/IFC 导入"不可撤销"；而 `WRITE_PATHS.md` 与 `CAPABILITY_MATRIX.md:117` 均称"全部可撤销"，代码亦走 Executor 记 undo | `MODELER_HANDBOOK.md:34` vs `WRITE_PATHS.md:53,75`; `lib.rs:715,975` | 建模师会误以为移动物件后必须手动备份，实际已有 undo 保护——**直接误导操作习惯** |
| P1-2 | 手册 §1.5"项目主页未实现"已过时：`/projects` 页已上线,`CODE_DEVIATIONS §7.2` 已标 resolved | `MODELER_HANDBOOK.md:42`; `App.tsx:189`、`ProjectsPage.tsx` | 与状态文档自相矛盾，削弱"文档即真"承诺 |
| P1-3 | 手册 §2.6"BuildingStorey 只有名字、没有标高系统"过时：能力矩阵已载 storey 标高输入（P8-S2）与楼层视图（P8-S4）已实现 | `MODELER_HANDBOOK.md:95` vs `CAPABILITY_MATRIX.md:106,114` | 建模师会放弃本可用的标高/楼层工作流 |
| P1-4 | 手册 §6.1"Web UI 无保存按钮"过时：Ctrl/Cmd+S 已绑定保存；且 `KEYBOARD_MAP.md` 仍把 Ctrl+S 标 📅，与本身已实现的 `App.tsx:108-111` 及 `CODE_DEVIATIONS §7.3 resolved` 冲突 | `MODELER_HANDBOOK.md:164`; `KEYBOARD_MAP.md`; `App.tsx:108-111` | 快捷键表三处来源不一致,验收现场可复现争议 |
| P1-5 | 手册 §5.2 按钮名不符：手册写 **Assy**，实为 **Assembly** | `MODELER_HANDBOOK.md:151`; `MainToolbar.tsx:416` | 步骤无法照做 |
| P1-6 | **文档门禁自身 FAIL**：`python3 scripts/check_docs.py` 报 `docs/status/CODE_AUDIT_2026-10-30.zh-CN.md` 缺 front matter（1 failure） | 门禁输出 `FAIL (1 issue(s))` | 门禁未绿即构成验收阻塞项,需补齐 front matter 或豁免 |

### P2（建议整改，不阻塞验收）

| # | 问题 | 证据 | 建议 |
|---|---|---|---|
| P2-1 | 快速开始的持久化警告块**首行与正文自相矛盾**（首行"默认不落盘"，紧跟"2026-10-28 起默认开启"） | `guides/GETTING_STARTED.md:25-29` | 删除矛盾的遗留首行 |
| P2-2 | 双语配对缺 2 对：`DOC_AUDIT_2026-10-27`（仅中文）与 `CODE_AUDIT_2026-10-30`（仅中文）；README 英文入口无法链接到英文审计 | `ls docs/status`；`CHANGELOG.md:262` 引用 | 补英文版或在前言注明"双语范围外" |
| P2-3 | `docs/research/` 13 篇 last_updated 停留在 2026-07/08（3 个月未维护） | front matter 统计 | front matter 加 `status: archived` 或在 README 标注"非契约文档" |

### P3（备忘）

- 能力矩阵/手册对"移动后几何降为 Mesh、参数不再驱动"的表述一致且诚实，但建议在 Create 面板 UI 内提示,而不仅靠文档（超出文档范围，转产品项）。
- 错误码体系（OMNI-UI/API/NET/IO/GEO/CAT/IFC/AUTH/SYS）作为验收契约保留合理，属"文档先于实现"的正确用法,建议在 ROADMAP 标注其启用里程碑。

## 4. 维度小结

- **设计**：目录划分、受众路由、源真层级、front matter 规范、check 脚本——体系设计成熟;RFC/process/architecture/status 层级使"承诺 vs 现状"可判。✔
- **功能**：references 与代码高度吻合(路由数、错误码、链接)；唯 `MODELER_HANDBOOK` 似乎在 2026-10-28/29 两波整改(Wave 3/4、P8)后**未同步更新**,形成"权威手册反而滞后于求证文档"的倒挂。⚠
- **使用**：五分钟上手路径、双单位口诀、控制台语法、快捷键表基本可用;P1-4/P1-5 修复后可用率达 100%。
- **操作**：CI 已纳入 UI checks 与文档门禁,但门禁当前不绿；双语与审计文档归档管理需按 P2 处理。

## 5. 验收建议

1. **先行整改 P1-1~P1-6**（预计半天工作量：改 4 处手册章节 + 1 处快捷键表 + 1 处快速开始遗留行 + 1 篇 front matter）,重跑 `check_docs.py` 至 0 failure。
2. P2 项随下一文档批次处理；P3 记入 ROADMAP。
3. 建议把"手册同步检查"纳入现有 check 脚本候选：凡 `CODE_DEVIATIONS` 状态翻转的条目,自动 grep `MODELER_HANDBOOK`/`KEYBOARD_MAP`/`GETTING_STARTED` 是否残留旧表述，防止本次"手册倒挂"再次发生。

> 复评方式：限期整改后按同方法重跑断言抽查与门禁,由独立复审评分（沿用 G0 §6 惯例,不自评）。
