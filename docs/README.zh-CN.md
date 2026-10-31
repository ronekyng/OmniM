---
title: "OmniM 文档"
status: current
last_updated: 2026-10-29
applies_to: "repo"
evidence: "verified against the docs tree 2026-10-27"
---
# OmniM 文档

[English](README.md) | [中文](README.zh-CN.md)

这里是 OmniM 文档的总入口。文档按读者与用途分层组织：**guides（指南）** 帮你跑起来，**architecture（架构）** 描述实际建成的东西，**specification（规范）** 定义设计契约，**reference（参考）** 用于查阅，**status（状态）** 跟踪项目真实进展。

> **关于诚实性的说明：** `architecture/` 与 `status/` 中的文档描述的是**与代码核对过的当前实现**；`specification/` 与 `RFC/` 中的文档描述的是**设计意图**，其中部分内容尚未实现。两者不一致之处都会显式标注（见 [代码偏差记录](status/CODE_DEVIATIONS.zh-CN.md)）。

## 目录

- [按角色查找](#按角色查找)
- [按目录查找](#按目录查找)
- [文档状态标记](#文档状态标记)

## 按角色查找

### 🧱 我想建模
- [建模者手册](MODELER_HANDBOOK.zh-CN.md) — 按领域组织的任务流程、真实 UI/API 步骤与如实缺口（[English](MODELER_HANDBOOK.md)）
- [能力矩阵](CAPABILITY_MATRIX.zh-CN.md) — 逐对象能力行：创建、宿主、连接、测量、IFC、proto 状态（[English](CAPABILITY_MATRIX.md)）
- [快速开始](guides/GETTING_STARTED.zh-CN.md) — 编译、启动与首次交互（[English](guides/GETTING_STARTED.md)）
- ✅ **持久化已默认开启（2026-10-28）**：变更自动写入 `project.omnim` / `$OMNIM_PROJECT`；`OMNIM_EPHEMERAL=1` 可关闭。覆盖范围见[快速开始](guides/GETTING_STARTED.zh-CN.md)的说明。

### 🚀 我想上手试用
- [快速开始](guides/GETTING_STARTED.zh-CN.md) — 编译服务端、启动 Web 界面、迈出第一步
- [构建指南](guides/BUILDING.zh-CN.md) — 工具链、环境变量、脚本、故障排查
- [Git 分支与工作流](guides/BRANCHING.zh-CN.md) — 分支模型、提交规范、合并与发布规则

### 🧭 我想理解系统
- [平台架构](architecture/PLATFORM_ARCH.zh-CN.md) — 当前实现的分层架构
- [数据流](architecture/DATA_FLOW.zh-CN.md) — 一次点击（或一句话、一个 IFC 文件）如何变成模型数据
- [架构地图](guides/ARCHITECTURE_MAP.zh-CN.md) — 面向开发者的逐 crate 地图、"想加 X 改哪里"速查
- [架构决策记录](architecture/DECISIONS.zh-CN.md) — 为什么选 slvs、truck、JSON 文件、protobuf-as-schema

### 📐 我需要设计契约
- [统一数据模型](specification/DATA_MODEL.zh-CN.md) — IFC 超集对象模型
- [Omcom 规范](specification/OMCOM_SPEC.zh-CN.md) — 统一类型化命令系统（Validator/Executor）
- [AMIR 规范](specification/AMIR_SPEC.zh-CN.md) — 自然语言 → OmniCommand 的 AI 翻译层
- [元件库与系列库](specification/CATALOG_SERIES.zh-CN.md) — PARAGON/SPECON 风格的目录与等级体系
- [生命周期模型](specification/LIFECYCLE.zh-CN.md) — 九阶段工程生命周期数据映射
- [领域映射](specification/DOMAINS.zh-CN.md) — 建筑/土木/工厂/机械对象分类树
- [IFC 扩展策略](specification/IFC_EXTENSION.zh-CN.md) — IFC 兼容与扩展策略

### 🎨 我想设计或产品化界面

- [前端愿景与范围](product/VISION_SCOPE.zh-CN.md) — 定位、用户、非目标与成功指标（[English](product/VISION_SCOPE.md)）
- [画像与旅程](product/PERSONAS_JOURNEYS.zh-CN.md) — 三类画像与五条端到端旅程（[English](product/PERSONAS_JOURNEYS.md)）
- [前端 PRD](product/PRD_FRONTEND.zh-CN.md) — 11 个域、96 条带验收标准的需求（[English](product/PRD_FRONTEND.md)）
- [需求登记册](product/REQUIREMENTS_REGISTER.zh-CN.md) — RTM：ID、契约、验证、门禁、状态（[English](product/REQUIREMENTS_REGISTER.md)）
- [外壳与信息架构](specification/ui/IA_SHELL.zh-CN.md) · [设计系统](specification/ui/DESIGN_SYSTEM.zh-CN.md) · [交互](specification/ui/INTERACTION.zh-CN.md)
- [反馈与错误](specification/ui/FEEDBACK_ERRORS.zh-CN.md) · [数据完整性](specification/ui/DATA_INTEGRITY.zh-CN.md) · [可访问性](specification/ui/A11Y.zh-CN.md) · [国际化](specification/ui/I18N_L10N.zh-CN.md) · [性能](specification/ui/PERFORMANCE.zh-CN.md)
- [兼容性](specification/ui/COMPATIBILITY.zh-CN.md) · [安全与隐私](specification/ui/SECURITY_PRIVACY.zh-CN.md) · [遥测](specification/ui/TELEMETRY.zh-CN.md) — GA 范围契约
- [完成定义](process/DEFINITION_OF_DONE.zh-CN.md) · [设计运营](process/DESIGN_OPS.zh-CN.md) · [本地化流程](guides/LOCALIZATION_WORKFLOW.zh-CN.md)
- [发布与支持清单](process/RELEASE_CHECKLIST.zh-CN.md) — 发布说明、迁移、运行手册、已知问题（[English](process/RELEASE_CHECKLIST.md)）
- [G0 评审记录（2026-10-29）](process/G0_REVIEW_2026-10-29.zh-CN.md) — 独立评审发现与处置（[English](process/G0_REVIEW_2026-10-29.md)）
- [Web 架构现状](architecture/WEB_ARCHITECTURE.zh-CN.md) — 前端今天是什么样，附实测债务（[English](architecture/WEB_ARCHITECTURE.md)）
- [前端成熟度](status/FRONTEND_MATURITY.zh-CN.md) · [基线](status/FRONTEND_BASELINE.zh-CN.md) · [发布就绪度](status/RELEASE_READINESS.zh-CN.md)
- [快捷键表](reference/KEYBOARD_MAP.zh-CN.md) · [错误码](reference/ERROR_CODES.zh-CN.md)
- [前端开发者指南](guides/FRONTEND_GUIDE.zh-CN.md) — 面板、命令、弹窗、令牌、字符串五条 recipe
- [RFC-0002：界面外壳与设计系统](RFC/RFC-0002-ui-shell-and-design-system.zh-CN.md)

### 🔍 我要查 API 或术语
- [REST API 参考](reference/API.zh-CN.md) — 全部服务端路由与 curl 示例
- [Omcom 命令参考](reference/OMCOM_REFERENCE.zh-CN.md)
- [术语表](reference/GLOSSARY.zh-CN.md)

### 📊 我想知道项目真实进度
- [路线图](status/ROADMAP.zh-CN.md) — 工作流状态、门禁与依赖，附代码佐证
- [格式兼容性](status/FORMAT_COMPAT.zh-CN.md) — 如实的导入/导出支持矩阵
- [代码偏差记录](status/CODE_DEVIATIONS.zh-CN.md)
- [变更日志](status/CHANGELOG.zh-CN.md)

### 💬 我想参与设计
- [RFC 索引](RFC/README.zh-CN.md) — RFC-0001 Omcom 文本 DSL（草案）、存储模型提案（未采纳）
- [调研笔记](research/README.md) — 竞品调研、IFC 4x3 规范摘录、GenRule 分析

## 按目录查找

```
docs/
├── product/         # 为什么做、给谁做：愿景、画像、PRD、需求登记册
├── architecture/    # 已建成：当前架构、数据流、架构决策记录
├── guides/          # 如何运行、构建、测试、在代码库中导航
├── specification/   # 设计契约：数据模型、Omcom、AMIR、目录与系列库、领域
│   └── ui/          # 界面契约：外壳、设计系统、交互、可访问性、国际化、性能
├── reference/       # 查阅：REST API、命令参考、快捷键表、错误码、术语表
├── status/          # 实况：路线图、格式矩阵、前端成熟度、发布就绪度
├── process/         # 治理：完成定义、设计运营
├── RFC/             # 开放设计提案（文本 DSL、存储模型、界面外壳）
└── research/        # 背景调研；不属于契约的一部分
```

两份面向建模者的参考文档位于 docs 根目录：[建模者手册](MODELER_HANDBOOK.zh-CN.md) · [能力矩阵](CAPABILITY_MATRIX.zh-CN.md)。

## 文档状态标记

| 标记 | 含义 |
|---|---|
| ✅ | 已与实现核对（以各文档标注的更新日期为准） |
| 🔀 | 部分实现 —— 文档内会说明哪些子项已实现、哪些没有 |
| 📅 计划中 | 有设计、无实现 |
| 💭 提案中 | 开放提案；见 RFC 索引 |
