---
title: "OmniM 界面遥测与可支持性"
status: design
last_updated: 2026-10-29
applies_to: "web/, crates/server"
evidence: "n/a (design contract)"
---
# OmniM 界面遥测与可支持性

> **契约范围。** 本文固定事件字典、漏斗、错误事件、性能指标、诊断导出与帮助入口。它是需求 `UI-TLM-01` … `UI-TLM-06` 的规范来源。
>
> **双轨维护义务**：`TELEMETRY.zh-CN.md` 是持续维护的译文。在同一次变更中保持章节编号、事件名与需求 ID 一致。
>
> **状态。** 设计契约。目前没有遥测管线；若没有预算建设，§6 的降级方案就是交付形态。

## 1. 目的

遥测用来在不打扰用户的情况下回答三个问题：产品能不能用、用户卡在哪、先修什么。它不是监视，也不能替代可用性测试。

**诚实规则：** 不要记录没有任何收集端接收的事件。若没有端点，本文只规定本地缓冲与导出路径。

## 2. 事件字典

| 规则 | 取值 |
|---|---|
| 命名 | `omnim.<area>.<object>.<action>`，多词用下划线 |
| 版本 | 每个事件带 `schema_version` 整数 |
| 属性 | 有类型、可枚举、在本文档登记 |
| PII | 禁止：不含名称、路径、几何或自由文本 |
| 稳定性 | 重命名需要版本升级与过渡期 |

基线事件：

| 事件 | 触发 | 属性 |
|---|---|---|
| `omnim.app.start` | 客户端启动 | app_version、locale、theme |
| `omnim.project.create` | 新建项目落定 | template_id、unit_system |
| `omnim.project.open` | 项目打开 | source（recent/file/sample）、load_ms、object_count_bucket |
| `omnim.project.save` | 显式保存 | result、duration_ms、object_count_bucket |
| `omnim.project.recover` | 显示恢复提示 | offered（布尔）、accepted（布尔） |
| `omnim.object.create` | 创建成功 | object_type、method（ribbon/palette/console） |
| `omnim.object.delete` | 删除确认 | object_type、child_count_bucket |
| `omnim.selection.change` | 选择稳定 500 ms | count_bucket、source（viewport/tree） |
| `omnim.panel.switch` | 停靠标签切换 | dock、tab_id |
| `omnim.theme.change` | 切换主题 | from、to |
| `omnim.locale.change` | 切换语言 | from、to |
| `omnim.error.shown` | 任一错误码出现 | code、severity、area |
| `omnim.diagnostics.export` | 导出诊断包 | trigger（user/error_boundary） |

需求：UI-TLM-01。

## 3. 漏斗

**上手漏斗：** `app.start` → `project.create|open` → `object.create` → `project.save` → `project.open（重开）`。

| 步骤 | 成功定义 | 目标 |
|---|---|---|
| 启动 | 外壳可交互 | ≥98% 会话 |
| 项目就绪 | 已打开项目 | ≥95% |
| 首个对象 | 至少创建一个对象 | ≥80% 新用户 |
| 首次保存 | 显式保存成功 | ≥75% |
| 重开 | 7 天内再次打开同一项目 | ≥40% |

规则：漏斗事件带 `session_id`（随机、不持久）且无用户标识；流失按步骤分析，绝不按人分析。

需求：UI-TLM-02。

## 4. 错误事件

| 规则 | 取值 |
|---|---|
| 必填字段 | `code`、`severity`、`area`、`operation`、`result`、`app_version` |
| 禁止 | 对象名、文件路径、几何、遥测中的堆栈 |
| 关联 | `correlation_id` 与本地诊断日志一致 |
| 采样 | 阻断 100%，可恢复 25%，校验 0% |
| 需求 | UI-TLM-03 |

## 5. 性能指标

| 指标 | 聚合 | 维度 |
|---|---|---|
| 首屏 / 工作区就绪 | p50、p95 | browser、gpu_class、object_count_bucket |
| 交互延迟 | p50、p95 | command_id |
| 帧率 | p50、p05 | object_count_bucket、scene_state |
| 内存 | p50、p95 | object_count_bucket |
| 保存耗时 | p50、p95 | project_size_bucket |

规则：可能识别具体项目时使用分桶而非精确计数；不上报逐帧事件。

需求：UI-TLM-04。

## 6. 诊断导出

| 项 | 说明 |
|---|---|
| 触发 | 用户在帮助/关于中操作，或错误边界提供入口 |
| 格式 | `.zip`，含 manifest、日志、命令、环境 |
| 内容 | 见 [FEEDBACK_ERRORS](FEEDBACK_ERRORS.zh-CN.md) §9 |
| 脱敏 | 路径缩减为 `<root>/…`；对象名替换为 `<object>` |
| 保留 | 由用户控制；绝不自动上传 |
| 降级 | 没有收集端时，本导出就是支持通道 |

需求：UI-TLM-05。

## 7. 帮助与支持入口

| 入口 | 内容 | 快捷键 |
|---|---|---|
| 快捷键参考 | 从命令注册表生成 | `F1` |
| 上下文帮助 | 每个面板与每条禁用命令的简短说明 | 聚焦时按 `?` |
| 示例项目 | 至少一个专业示例，离线可用 | 项目首页 |
| 关于 | 版本、构建、服务版本、GPU、数据目录、许可 | 帮助菜单 |
| 诊断 | 导出诊断包、复制摘要 | 帮助菜单 |
| 发布说明 | 应用内链接到 CHANGELOG | 关于 |

规则：所有帮助入口必须离线可用；不得有死链；帮助截图必须与当前主题一致。

需求：UI-TLM-06。

## 8. 非目标

1. 会话录制或热力图。
2. 用户账号或跨设备身份。
3. 实时崩溃告警。
4. A/B 实验框架。

## 9. 待定问题

| # | 问题 | 责任人 | 需要时间 |
|---|---|---|---|
| Q1 | 是否有收集端，还是首个版本只做导出？ | 产品 / 后端 | G3 |
| Q2 | 许可协议是否要求单独的遥测同意对话框？ | 法务 | G3 |
| Q3 | 当项目名称本身敏感时，哪些指标可以安全上报？ | 产品 / 安全 | G3 |
