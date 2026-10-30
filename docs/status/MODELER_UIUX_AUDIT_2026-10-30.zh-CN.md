---
title: "建模师 UI/UX 验收审核报告(第三轮)"
status: history
last_updated: 2026-10-30
applies_to: "web/, docs/status/FRONTEND_MATURITY.md, docs/status/CODE_DEVIATIONS.md"
evidence: "five runtime screenshots cross-checked against web/src (saveState.ts, AssemblyPanel.tsx, styles.css, sketch/*) and docs 2026-10-30"
languages: zh-CN
---
# 建模师 UI/UX 验收审核报告(2026-10-30,基于真实运行截图)

> 本文为单语言（zh-CN）快照，不在 en/zh 双语配对范围内。
>
> **处置进度（2026-10-30，同日，两批）**
>
> **第一批（P0/P2 子集）**：F4——Assembly 面板尺寸/偏移占位符全部标注 (m)、顶部加 mm/m 双单位提示；LayerEditor 厚度字段标注 (mm)（目录 `ComponentParamPanel` 此前已带 unit 后缀）；B2——GridPanel "0 / 0" 机密文案改为语义说明（贯通 ±20 m）。F1——`saveLabel` dirty 态文案改为 "Auto-saved server-side · Ctrl+S to snapshot"、TitleBar dirty 徽标 aria-label 同步，消除"未保存"假警报；`FRONTEND_MATURITY` §5 口径改为"服务端默认落盘、脏状态=快照分级"。A3——`DomainPanels`/`AvailableComponentsPanel`/`AssemblyPanel`/主工具栏**创建 Component**、组件编辑器特征输入草图共 7 处 `-${Date.now()}` 命名替换为 `nextAutoName`（`<prefix>-<nnn>` 契约，UI-INT-09）。
>
> **第二批（剩余可落地项）**：F3——轴网表单新增"贯通整个视图（±20 m）"语义开关（勾选时 From/To 置 0 并禁用，编辑既有贯通轴自动回显），替代 0/0 密码；Assembly 加尺寸与部件名校验（正数/非空，显式 toast）。D2——shape 选项统一为"中文 (English)"。A6——对象树搜索占位符本地化。E1（部分）——`SketchCanvas3D` 背景与网格线从第三套藏蓝改为主壳 graphite token（`#1a1d23`/`#3a3f47`），消除"第三种蓝"；完整 token 迁移仍属 G2。E2——约束工具栏 23 个按钮补齐中文 title/aria-label、Del 按钮补 tooltip、偏移量输入补可见"偏移 (m)"标签。文档立项——`CODE_DEVIATIONS` 增补 7.12（保存语义单一口径）、7.13（同屏单位可读），双语并标注已整改。
>
> **仍未处理**（G2/G3 门禁批次，需独立设计窗口）：E1 完整双主题 token 化、E2 约束图标体系的系统性重做、F2 组件编辑器整体视觉统一、A2 双状态栏冗余、A4 工具栏层级、C3 提示重叠、A11Y（7.10）、i18n（7.9）、硬编码颜色 712 处（7.8）。
>
> 核验：`tsc --noEmit` ✅、vitest 100 ✅、`npm run build` ✅、`check_docs.py` ✅、`check_ui_contracts.py` ✅。待同帧重截验证 F1/F4/F3 消失。
>
> **关闭记录（2026-10-30）**：本报告全部发现已处置或按 G2/G3/结构性分波登记（清单见上"仍未处理"及 `ROADMAP`「审计残余跟踪」），无未入账项。
>
> **复验记录**：[MODELER_UIUX_REAUDIT_2026-10-30](MODELER_UIUX_REAUDIT_2026-10-30.zh-CN.md) 以 G1 体验档正式放行（7/7 发现关闭）。

> **审核角色与视角**:建模师(最终使用者)。本轮聚焦/screen 证据:功能是否满足建模要求、设计是否优雅、操作是否方便。
> **输入**:5 张运行截图(主工作区、轴网面板、创建墙体、Assembly 面板、组件编辑器)+ 前端文档层(`FRONTEND_MATURITY`、`CODE_DEVIATIONS §7/P9`、`IA_SHELL`、`DESIGN_SYSTEM`)+ 代码核证。
> **方法**:截图现象 → 代码定位(`saveState.ts`、`AssemblyPanel.tsx`、`styles.css :root`、`sketch/*`)→ 文档比对,双向验证:文档承诺是否兑现、截图偏差是否已入账。

## 1. 结论

- **功能满足要求:通过**(可用原型档)。主工作流(创建/轴网/楼层/目录/草图-特征-实体/控制台/AI)在截图中均呈现为可完成路径,与《Modeler Handbook》逐条吻合。
- **设计优雅与操作便利:有条件通过(G1 档),距商业验收(G2/G3)有明确距离**——与文档自评(ISO 25010 总分 1.9,G1 目标 2.2,G2 目标 3.0)完全一致。
- **文档与 UI 的一致性:高分**。截图中的每一项强势与弱势均能在文档中找到对应记载(停靠属性面板、自动命名、混排语言、主题债、A11Y=0),没有发现"截图戳穿文档"的虚假陈述。文档体系的求真文化在前端层同样成立。

### 评分(建模师视角,ISO 25010 对齐)

| 维度 | 得分 | 依据 |
|---|---|---|
| 功能满足 | 3.5 / 5 | 12 类建筑构件快捷 chips、轴网 CRUD、楼层视图、目录/系列、草图→BRep 全链可走通 |
| 设计优雅 | 2.5 / 5 | 主壳克制的石墨深色 token 体系统一;但组件编辑器割裂为另一套藏蓝视觉(见 F2) |
| 操作便利 | 3 / 5 | 分步引导 toast、W/E/F/H/G/O 提示条、dock 面板+滑杆参数化好;单位陷阱(F4)与"未保存"假警报(F1)伤害信任 |
| 可交付性 | 2 / 5 | 中英混排、图标语义、时间戳默认名,离出图/交付还有距离 |

## 2. 截图逐帧审核

### 帧 1 — 主工作区
| 编号 | 发现 | 定性 | 证据 |
|---|---|---|---|
| A1 | **"Unsaved changes"常亮与"自动落盘"存在跨层真相矛盾**:UI 是显式保存状态机(`saveState` 状态机:dirty→显式 Save→clean),但服务端默认每次 mutation 自动写盘。建模师看到的"未保存"是**假警报**——磁盘上其实已有该变更;退出拦截 `shouldWarnBeforeUnload` 同样基于 dirty,会误拦 | P0-UX | `saveState.ts:23-63`、`App.tsx:74`;vs `MODELER_HANDBOOK §1.3`/`GETTING_STARTED.md:25-28` |
| A2 | 顶部与底部状态栏**两处**同时显示保存态("Unsaved changes"×2)— 冗余但一致 | P3 | `saveState.ts:48-59` 双渲染位 |
| A3 | 左侧树自动命名 `wall-17932508050609` 为时间戳串,不利于辨识与交付归档(重命名入口有,但默认值观感差) | P2 | `FRONTEND_MATURITY`(auto-naming landed);ObjectTreePanel rename |
| A4 | 工具栏第二行建筑快捷条 + "全部楼层"选择器悬浮于视口顶部左侧,层级归属感弱,与 Create 面板存在入口双轨 | P3 | `StoreyViewSelect.tsx:55` |
| A5 ✅ | 楼层过滤选择器、A1/A2 轴网标签 sprite、视图立方、底部坐标栏(单位 m)、快捷键提示条全部兑现,与文档一致 | 符合 | 同上 |
| A6 | "Objects 5 · Materials 23 · Lifecycle 0" 计数条信息密度好;搜索 "Search by name / type / IFC class…" 混排 | P3 | 帧 1 |

### 帧 2 — 轴网面板
| 编号 | 发现 | 定性 |
|---|---|---|
| B1 ✅ | 轴网 CRUD、Edit/删除、反向提示"0/0 贯通 ±20 m"与文档 P8-S3 一致 | 符合 |
| B2 | "0 / 0 extends the line across the viewport (±20 m)" 是实现机密泄漏式文案(坐标值代替语义),建模师视角应提供"贯通轴线"开关 | P2 |

### 帧 3 — 创建墙体(亮点与陷阱并存)
| 编号 | 发现 | 定性 | 证据 |
|---|---|---|---|
| C1 ✅ | 步骤引导 toast("点击第 1 个点…按 Esc 取消")、面包屑 Create wall、右上停靠属性面板(契约 P9 7.1 已兑现)、height 滑杆 mm(500–10000)、IFC Class 直显 | 符合 | `CreatePanel.tsx`、`CODE_DEVIATIONS P9 update` |
| C2 | 单位语义在同类面板间**相反**且 UI 不承载规则:墙体面板 мм(3000 mm),Assembly 面板 m(见 F4);手册 §1.1 的"双单位口诀"只活在文档里 | **P0-UX** | 帧 3 vs 帧 4、`MODELER_HANDBOOK.md §1.1` |
| C3 | 底部右下提示语("在 3D 视图中点击放置点…")与快捷键提示条重叠,可读性差 | P3 | 帧 3 右下 |

### 帧 4 — Assembly 面板
| 编号 | 发现 | 定性 | 证据 |
|---|---|---|---|
| D1 | 尺寸输入 p1/p2/p3 用占位符"—"无任何语义/单位标注;X/Y/Z 偏移同样;无单位提示文案(代码 grep 证实零处单位说明)。建模师按墙体面板的 mm 习惯输入即成倍缩放 | **P0-UX** | `AssemblyPanel.tsx:74-83`(无 单位/m/mm 文案)、手册 §5.2(m) |
| D2 | "Box 方块" 中英拼接选项文本;部件名自由文本无校验提示 | P2 | `AssemblyPanel.tsx:64` |
| D3 ✅ | 最终动作"创建 Assembly(N 个部件)"存在且带计数,流程可闭环(截图裁切外) | 符合 | `AssemblyPanel.tsx:93` |

### 帧 5 — 组件编辑器
| 编号 | 发现 | 定性 | 证据 |
|---|---|---|---|
| E1 | **双视觉体系并存**:主壳 graphite tokens(`styles.css :root #1a1d23…`)vs 编辑器独立藏蓝配色(内联 hex、`SketchCanvas3D` 背景 `0x1a1a2e`、独立 CSS module)。不止是"硬编码颜色债",是两套并行设计语言 | P1 | `EditorToolbar.tsx:227,264-312`、`SketchCanvas3D.tsx:29`、7.8 |
| E2 | 特性 ribbon 图标+小字尚可认读;约束工具栏全图标无文字、`1` 数字输入框无义项说明、Del 无 tooltip——学习成本高,依赖口头传承 | P1 | `EditorToolbar.tsx` |
| E3 ✅ | 特征树空态引导("暂无 Feature…")、Solve 状态着色、Snap/Const 独立开关、2D-Top/3D 页签、显著 Rebuild All——空态与反馈意识到位 | 符合 | 帧 5、`FeaturePanel.tsx` |

## 3. P0 项的前端文档侧账目核对

- F1(假警报)与 F4(单位陷阱)**未在任何前端文档立案**:`CODE_DEVIATIONS §7` 与 `REQUIREMENTS_REGISTER` 的 DAT/FBK 条目不含"自动保存语义与 UI 保存态不一致"、也不含"GenRule/catalog 面板单位未在 UI 标注"。这是本轮**新增发现**,建议 7.12/7.13 立项。
- 其余(双主题、混排、A11Y、图标)均已在 7.8/7.9/7.10/DS 8%/A11Y 0% 如实记账——**文档没有粉饰,验收可信**。

## 4. 验收判断

| 验收等级 | 结论 | 条件 |
|---|---|---|
| 可用原型(G1) | **通过** | 建议先行关闭 F4(单位标注,预计小时级)与 F1(保存语义对齐:要么 UI 轮询展示"已自动保存 HH:mm:ss",要么明示"显式保存,服务端不留盘") |
| 商业 beta(G2) | 不通过 | F2/F3:主题与语言统一(与 G2 门 DS/I18N 得 8%/0% 互证);E2 图标语义 |
| GA(G3) | 不通过 | A11Y/CMP/SEC/TLM 得 0% 的项,与文档门禁一致,无异议 |

## 5. 整改建议(按建模师风险排序)

1. **F4**:三个 GenRule/catalog 参数面板统一显示单位徽标(mm/m),数据来自各自契约;同屏禁止无单位数字输入。
2. **F1**:引入/UI 呈现服务端自动保存的事实(轮询 `saveState` 或在 dirty 态附带 "auto-saved hh:mm:ss"),统一 FRONTEND_MATURITY"无自动保存"与后端"默认落盘"的口径;退出拦截仅在关闭 `OMNIM_EPHEMERAL` 逃生时触发,或改为"服务端已持久,提示保存快照"。
3. **F2**:抽出组件编辑器的调色板为共享 token(哪怕临时双主题,先消除第三种蓝)。
4. **F3**:轴网表单把"±20 m 贯通"改成语义开关;Assembly 单位/必填校验随 F4 一并做。
5. **A3**:默认名改为序号+类型(`Wall-003`)并可配置模板,时间戳仅作内部 ID。

> 复验方式:同帧重截 5 类界面,核对 F1/F4 是否消失,并跑 `check_docs.py`/`check_ui_contracts.py` 维持门禁绿色。
