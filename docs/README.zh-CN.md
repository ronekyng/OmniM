# OmniM 文档

[English](README.md) | [中文](README.zh-CN.md)

这里是 OmniM 文档的总入口。文档按读者与用途分层组织：**guides（指南）** 帮你跑起来，**architecture（架构）** 描述实际建成的东西，**specification（规范）** 定义设计契约，**reference（参考）** 用于查阅，**status（状态）** 跟踪项目真实进展。

> **关于诚实性的说明：** `architecture/` 与 `status/` 中的文档描述的是**与代码核对过的当前实现**；`specification/` 与 `RFC/` 中的文档描述的是**设计意图**，其中部分内容尚未实现。两者不一致之处都会显式标注（见 [代码偏差记录](status/CODE_DEVIATIONS.zh-CN.md)）。

## 目录

- [按角色查找](#按角色查找)
- [按目录查找](#按目录查找)
- [文档状态标记](#文档状态标记)

## 按角色查找

### 🚀 我想上手试用
- [快速开始](guides/GETTING_STARTED.zh-CN.md) — 编译服务端、启动 Web 界面、迈出第一步
- [构建指南](guides/BUILDING.zh-CN.md) — 工具链、环境变量、脚本、故障排查

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

### 🔍 我要查 API 或术语
- [REST API 参考](reference/API.zh-CN.md) — 全部服务端路由与 curl 示例
- [Omcom 命令参考](reference/OMCOM_REFERENCE.zh-CN.md)
- [术语表](reference/GLOSSARY.zh-CN.md)

### 📊 我想知道项目真实进度
- [路线图](status/ROADMAP.zh-CN.md) — 各阶段状态，附代码佐证
- [格式兼容性](status/FORMAT_COMPAT.zh-CN.md) — 如实的导入/导出支持矩阵
- [代码偏差记录](status/CODE_DEVIATIONS.zh-CN.md)
- [变更日志](status/CHANGELOG.zh-CN.md)

### 💬 我想参与设计
- [RFC 索引](RFC/README.zh-CN.md) — RFC-0001 Omcom 文本 DSL（草案）、存储模型提案（未采纳）
- [调研笔记](research/README.md) — 竞品调研、IFC 4x3 规范摘录、GenRule 分析

## 按目录查找

```
docs/
├── architecture/    # 已建成：当前架构、数据流、架构决策记录
├── guides/          # 如何运行、构建、测试、在代码库中导航
├── specification/   # 设计契约：数据模型、Omcom、AMIR、目录与系列库、领域
├── reference/       # 查阅：REST API、命令参考、术语表
├── status/          # 实况：路线图、格式矩阵、偏差记录、变更日志
├── RFC/             # 开放设计提案（文本 DSL、存储模型）
└── research/        # 背景调研；不属于契约的一部分
```

## 文档状态标记

| 标记 | 含义 |
|---|---|
| ✅ | 已与实现核对（以各文档标注的更新日期为准） |
| 🔀 | 部分实现 —— 文档内会说明哪些子项已实现、哪些没有 |
| 📅 计划中 | 有设计、无实现 |
| 💭 提案中 | 开放提案；见 RFC 索引 |
