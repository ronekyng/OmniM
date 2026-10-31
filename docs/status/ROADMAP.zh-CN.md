---
title: "OmniM 开发路线图"
status: current
last_updated: 2026-10-31
applies_to: "repo"
evidence: "对照代码、CHANGELOG 与门禁看板复核；每条状态附证据链接"
---
# OmniM 开发路线图

> 本文回答三个问题：**今天有什么、下一步做什么、什么在阻塞**。
>
> - **日期不是承诺。** 本文不按日历排期，按依赖与门禁排序；历史交付日期只保留在 [CHANGELOG](CHANGELOG.zh-CN.md)。
> - **单一真值。** 能力现状以 `status/` 与 `architecture/` 为准，设计意图以 `specification/` 与 `RFC/` 为准；两者不一致时必须在 [CODE_DEVIATIONS](CODE_DEVIATIONS.zh-CN.md) 显式登记。
> - **编号是标签，不是顺序。** `7b`、`8`、`P8-S1` 是可被其他文档引用的稳定标签，不代表必须串行，也不代表日历先后。

## 1. 阅读约定

### 1.1 状态标记

| 标记 | 含义 |
|---|---|
| ✅ 完成 | 已实现，且有代码或测试证据 |
| 🔄 进行中 | 活跃开发，部分切片已交付 |
| 🔀 部分完成 | 主体已交付，存在点名的缺口 |
| 📅 计划中 | 未开始，或仅有意向 |

### 1.2 三组容易混淆的词

| 词 | 指什么 | 不是什么 |
|---|---|---|
| 批次 / 阶段 | 交付流标签（1–12、7b） | 日历季度，也不是必须串行的序号 |
| 门禁 | G0–G3，发布质量闸门 | 功能批次 |
| 切片 | 批次内一次可验收的交付（如 P8-S1） | 独立阶段 |

## 2. 当前状态快照

| 工作流 | 批次 | 状态 | 今天真实可用 | 明确缺口 | 证据 |
|---|---|---|---|---|---|
| 内核与几何 | 1–6 | ✅ 主体（批次 1 🔀） | 统一数据模型、类型化命令与撤销/重做、草图 + slvs、特征 + truck-BRep、目录/系列、GenRule 与默认模板 | 基于 nom 的 Omcom 文本 DSL（[RFC-0001](../RFC/README.zh-CN.md)） | `crates/model/`、`crates/omcom/`、[CAPABILITY_MATRIX](../CAPABILITY_MATRIX.zh-CN.md) |
| 工作台原型 | 7 | ✅ | 视口、草图编辑器、特征树、目录面板、属性面板、Omcom 控制台、AI 聊天、Library/Series Manager | 仅为功能原型，不是产品级界面；AI 命令预览归批次 9 | `web/src/`、[CHANGELOG](CHANGELOG.zh-CN.md) |
| 前端产品化 | 7b | 🔄 | 项目首页、停靠外壳、保存/另存为、错误边界、共享弹窗、自动命名 | G1 未通过；G2/G3 未开始；无 e2e、视觉与性能基线 | [RELEASE_READINESS](RELEASE_READINESS.zh-CN.md) |
| 领域扩展 | 8 | 🔄 | 生产级目录数据；楼层标高/轴网/楼层视图/洞口；装配容器；管道路由与连接器 | 建筑完整 IFC/MEP/算量、土木地形路线、工厂分支管道、机械 BOM；领域前端 | [CAPABILITY_MATRIX](../CAPABILITY_MATRIX.zh-CN.md) |
| AI 编译 | 9 | 🔀 | AMIR 提供方、自然语言→命令、多轮聊天、反向描述模型 | 多步规划、校验解释、置信度、人在回路预览、多智能体、RAG | `crates/amir/` |
| 格式互操作 | 11 | 🔀 | IFC 2x3 / IFC 4 子集导入导出（手写解析器/写出器） | 实体覆盖不完整；STEP/DXF/DWG/GIS/glTF/OBJ/FBX 未实现 | [FORMAT_COMPAT](FORMAT_COMPAT.zh-CN.md) |
| 云与协作 | 10 | 🔀（仅子集） | Axum REST、JSON 文件持久化、全变更端点撤销/重做 | PostgreSQL/PostGIS、版本历史、WebSocket、多用户协作 | [API](../reference/API.zh-CN.md)、[WRITE_PATHS](../architecture/WRITE_PATHS.zh-CN.md) |
| 分析与生态 | 12 | 📅 | 无 | 结构分析、碰撞检测、工程量、成本、插件生态 | — |

## 3. 已交付批次（1–7）

### 3.1 内核与几何（1–6）

- **批次 1 🔀**：统一数据模型、类型化 `OmniCommand` 与执行器、生命周期/行政区划/分解结构/关系系统。缺口只有一项：文本 DSL 仍是草案 RFC。
- **批次 2 ✅**：草图数据模型、11 类约束、slvs 求解器、草图编辑器、Extrude/Revolve 管线。
- **批次 3 ✅**：特征引擎（9 类特征）、truck-BRep、参数系统、组件编辑器。
- **批次 4 ✅**：ComponentDef/ParamDef/ConnectorDef、Series 选择器、`.omnimlib` / `.omnimseries`。
- **批次 5 ✅**：钢型材、管道、混凝土建筑构件目录与实例化管线。
- **批次 6 ✅**：GenRule 重构、Library/SeriesStore/Project 分离、系列门禁、CatalogRef、默认模板。

证据入口：`crates/model/`、`crates/sketch/`、`crates/omcom/`、[ARCHITECTURE_MAP](../guides/ARCHITECTURE_MAP.zh-CN.md)。

### 3.2 工作台原型（7）✅

Project Workspace、草图编辑器、特征树、目录面板、属性面板、Omcom 控制台、AI 聊天，以及 Library/Series Manager 均已可用。

**边界说明：** 批次 7 的 ✅ 只表示“功能原型可走通主流程”，不表示产品级界面。产品化由批次 7b 承担，两者不可互相替代。

## 4. 主线 A：前端产品化（7b）🔄

### 4.1 出口定义

| 出口 | 包含 | 判定 |
|---|---|---|
| 工程出口 | G1 + G2 | 前端工程可闭环，全部需求有证据 |
| 发布出口 | G3 | 含兼容性、安全、遥测与外部评审 |

> 当前口径：**7b 的工程完成线定在 G2**，G3 归发布前。G3 含外部安全评审与 ACR/VPAT，不是纯代码工作。此口径需产品签字确认（见 §10 的 Q2、Q4）。

### 4.2 门禁看板

| 门禁 | 范围 | 状态 | 今天 | 退出条件 |
|---|---|---|---|---|
| G0 | 契约与需求 | 🔀 | 草案完成，签字栏待填 | 产品/设计/前端三方签字 |
| G1 | Alpha | 📅 未通过 | 25 条中 10 ✅ / 15 🔀 / 0 📅 | 25 条全部 ✅ 且证据可点开 |
| G2 | Beta | 📅 未开始 | 0 / 37 | 设计系统、i18n、A11Y、性能全部达标 |
| G3 | GA | 📅 未开始 | 0 / 34 | 兼容/安全/遥测/ACR/发布手册 + 外部评审 |

完整清单：[REQUIREMENTS_REGISTER](../product/REQUIREMENTS_REGISTER.zh-CN.md)；判定规则：[RELEASE_READINESS](RELEASE_READINESS.zh-CN.md)。

### 4.3 执行顺序（按依赖，不按日期）

| 步骤 | 内容 | 完成判据 |
|---|---|---|
| 0 治理 | G0 签字；关闭 O4/O6；确认 §4.1 出口口径；发布冻结令 | 决策有记录，冻结范围有清单 |
| 1 测试基建 | Playwright、截图基线、axe、性能参考机、CI 任务 | CI 在 PR 上运行；基线可比较 |
| 2 G1 收口 | 先补缺失功能（主题 Provider、骨架屏），再补外壳/面板/数据完整性，最后逐条补证据 | 25 / 25 |
| 3 G2 收口 | 语义令牌、i18n、可访问性、性能预算、剩余反馈与数据完整性 | 37 / 37 |
| 4 G3 预备 | 兼容矩阵、安全披露、遥测字典、ACR/VPAT 草稿、发布与支持手册 | 34 / 34 + 外部评审结论 |

### 4.4 波次与需求域

| 波次 | 范围 | 需求域 | 目标门禁 |
|---|---|---|---|
| A | 外壳、停靠、状态栏、双主题、图标与组件体系 | SHL、DS | G1 → G2 |
| B | 项目生命周期、脏状态、错误边界、弹窗、自动命名 | DAT、FBK、INT | G1 |
| C | 命令注册表、反馈三态、i18n、可访问性、撤销历史 | INT、I18N、A11Y | G2 |
| D | 按需渲染、实例化、虚拟化、e2e、视觉回归、CI 门禁 | PRF、CMP、SEC、TLM | G2 → G3 |

## 5. 主线 B：领域扩展（8）🔄

### 5.1 已交付切片

- 生产级目录数据：钢型材（EN 10365 / GB/T 706）、管道等级（ASME B36.10）、混凝土模板、真实材料标准、BRep 正确性校验。
- 建筑基准：楼层 `elevation`、轴网面板、`bottom_level_id`/`top_level_id` 单步级联、楼层视图与剖切联动。
- 独立洞口对象（`IfcOpeningElement`）。
- 装配容器（`POST /api/assemblies`，容器 + Component/Part 子件，整树删除一次撤销）。
- 管道路由：N 点折线扫掠、弯头占位、实例连接器与口径/类型兼容校验。

证据：[CAPABILITY_MATRIX](../CAPABILITY_MATRIX.zh-CN.md)、[CHANGELOG](CHANGELOG.zh-CN.md)。

### 5.2 未完成项

| 项 | 状态 | 前置 | 说明 |
|---|---|---|---|
| 8-1 建筑：完整 IFC 元素、MEP、工程量 | 📅 | 内核 ✅ | 门窗为宿主填充（门扇 + 框），仍缺框类型、窗台/过梁与门窗表；宿主不重剪 |
| 8-2 土木：地形拟合、土方、路线/纵断面 | 📅 | 内核 ✅ | 桥/隧仍只用首末点 |
| 8-3 工厂：分支管道、P&ID | 📅 | 管道路由 ✅ | 分支、管件、特征模板未覆盖 |
| 8-4 机械：BOM、算量、运动副 | 🔀 | 装配容器 ✅ | 容器已交付，BOM/运动副未开始 |
| 8-5 前端：领域面板、筛选器、领域感知拖拽 | 📅 | **7b G2** | 必须等外壳与设计系统定稿 |

### 5.3 与 7b 的依赖

[ADR-0008](../architecture/DECISIONS.zh-CN.md) 明确规定“壳体重写必须先于面板美化”；[ADR-0009](../architecture/DECISIONS.zh-CN.md) 的面板注册表就是为领域面板预留的扩展点。因此：

- **8-5 在 7b G2 之前不启动。**
- 8-5 落地时必须走 `PanelRegistration`、语义令牌、`t(key)` 与统一命令注册表；否则按新债处理。
- 8-1 至 8-4 属于后端/几何域，是否与 7b 并行见 §7.2。

## 6. 其他工作流（9–12）

- **9 AI 编译 🔀**：引擎、提供方、翻译与聊天已交付；缺多步规划、校验解释、置信度、人在回路与 RAG。
- **11 格式互操作 🔀**：只有 IFC 子集；其余格式未实现，矩阵见 [FORMAT_COMPAT](FORMAT_COMPAT.zh-CN.md)。
- **10 云与协作 🔀（仅子集）**：REST + JSON 文件已交付；数据库、版本历史、WebSocket、协作均未实现。
- **12 分析与生态 📅**：结构分析、碰撞、算量、成本、插件均未开始。

## 7. 依赖、并行与冻结规则

### 7.1 依赖关系

- 内核（1–6）→ 9、11、8-1…8-4。
- 7b（外壳、设计系统、命令注册表、i18n）→ 8-5 领域前端。
- 7b → 10 的协作 UI。
- 8-5 不得早于 7b G2；其余领域后端项与 7b 无代码依赖。

### 7.2 并行规则

1. **同一文件域同一时间只有一个流写入。** `web/src/` 归 7b；`crates/` 归后端流。
2. 跨流改动必须先落契约，再改代码。
3. 新增 UI 必须使用面板注册表、语义令牌、`t(key)`、统一命令注册表；绕过即记为新债。
4. 8-1 至 8-4 的后端工作是否允许在 7b 冻结期内推进，见 §10 的 Q7；默认冻结。

### 7.3 当前冻结与例外

| 冻结 | 例外 |
|---|---|
| 8-1 至 8-4 的新功能 | 已交付能力的回归修复 |
| 8-5 领域前端 | — |
| 新对象类型、新 REST 路由 | 安全或数据丢失类修复 |

**已关闭的回归修复（不计入 8-1 新功能）：**

- 门几何：门/窗改为门扇/框面板，按墙体 `path_points` 定位；不再用世界 AABB 当墙厚，也不再漂在原点。见 [CODE_DEVIATIONS §6.5](CODE_DEVIATIONS.zh-CN.md)。
- 楼层创建：`storey` 不再附带 box 几何，成为纯标高容器；服务端同样丢弃容器几何。

两项均已带测试（`crates/server/tests/opening.rs`、`datum.rs`）。

## 8. 当前非目标

1. 照片级渲染。
2. 离线优先架构。
3. 移动 AR/VR 与移动应用。
4. 完整 CAM/CNC。
5. 桌面打包（Tauri）。
6. 生产 Web 构建中的 WASM 共享内核。

文本 DSL 不属于非目标；它是批次 1 的未交付项，等待 [RFC-0001](../RFC/README.zh-CN.md) 定稿。

## 9. 技术债账本

| 项 | 归属 | 证据 |
|---|---|---|
| 硬编码颜色未走语义令牌 | G2 | [CODE_DEVIATIONS](CODE_DEVIATIONS.zh-CN.md) 7.8 |
| 界面文案未外置（31 文件） | G2 | CODE_DEVIATIONS 7.9 |
| 可访问性语义缺失 | G2 | CODE_DEVIATIONS 7.10 |
| `App.tsx` 全量 store 订阅、`ThreeViewer` 未拆分、react-query 未启用 | G2 | [CODE_AUDIT](CODE_AUDIT_2026-10-30.zh-CN.md) P2/P3 |
| 错误码保留但未发射 | G2 | [ERROR_CODES](../reference/ERROR_CODES.zh-CN.md) |
| 性能预算未进 CI（TLM 0%） | G2 | [MODELER_UIUX_REAUDIT](MODELER_UIUX_REAUDIT_2026-10-30.zh-CN.md) §5 |
| 无 e2e / 视觉 / 性能基线 | G1 前置 | 本文 §4.3 |
| server token 中间件、`TimeoutLayer`、persist 移出写锁、列表/读取全量克隆 | 发布前或后端并行 | CODE_AUDIT P1/P2 |
| omcom 单文件拆分、executor `Result` 化、`project.rs` 拆分、共享 `IndexedStore` | 长期 | CODE_AUDIT §5 |
| 真三角网格 CSG 替代 AABB 近似 | 长期 | CODE_AUDIT |
| clippy 警告基线未清零，`-D warnings` 未启用 | 质量基线（G2 前） | CODE_AUDIT 验收记录 |
| `sketch` 约束 `let _ =`、`resolve_param` 未知参数 | 长期 | CODE_AUDIT |
| 测试共享全局 `LIBRARY` | 长期 | [TESTING](../guides/TESTING.zh-CN.md) |
| `eprintln!` 未换结构化日志 | 长期 | CODE_AUDIT |
| 门几何与楼层创建回归 | 当前例外修复 | 本文 §7.3 |

## 10. 待定问题

| # | 问题 | 责任人 | 影响 |
|---|---|---|---|
| Q1 | G1 是对外里程碑还是仅内部？ | 产品 | G1 |
| Q2 | G3 是否需要外部安全评审？ | 产品 / 法务 | G3 |
| Q3 | 设计与工程不一致时谁签 G2？ | 产品 | G2 |
| Q4 | 7b 工程出口是否确认为 G1+G2、G3 归发布？ | 产品 | 7b / G2 |
| Q5 | 楼层视图、剖切、测量、8-5 领域面板是否分配需求 ID？ | 产品 / 前端 | G1 / G2 |
| Q6 | 英文硬编码界面字符串是否纳入扫描？ | 前端 | G2 |
| Q7 | 冻结期内 8-1 至 8-4 的后端是否允许独立推进？ | 技术负责人 | 7b / 8 |

## 11. 维护规则

1. **本文件只放当前状态。** 历史过程写入 [CHANGELOG](CHANGELOG.zh-CN.md)，不在此复制。
2. **每条 ✅ 必须有证据链接；每条 📅 必须挂在某个批次或门禁上。** 无验收物的条目不得出现在路线图。
3. **正文不写日历日期。** 只保留 front matter 的 `last_updated`；历史日期由 CHANGELOG 承担。
4. **状态变更必须在同一次提交中更新**：本文件快照、门禁看板、CHANGELOG。
5. **双语结构必须一致。** `check_docs.py` 校验标题数量与编号；英文版是同步维护的译文，不是独立文档。
6. **批次标签保持稳定。** 重命名必须附映射表，避免其他文档断链。
