# OmniM RFC

RFC（Request for Comments）文档用于讨论 OmniM 的重大设计变更。

## 索引

| RFC | 标题 | 状态 |
| --- | ----- | ------ |
| [RFC-0001](RFC-0001-omcom-text-dsl.md) | Omcom 文本 DSL（基于 nom 的文本语法） | **Draft（草案）** — Omcom 文本语言的权威设计；产品内控制台目前仅执行 JSON 命令 |
| [STORAGE_MODEL.md](STORAGE_MODEL.md) | 存储模型设计提案 | **Proposed / 未采纳** — 从仓库根 `docs/` 目录移入此处作为设计提案；未落地实现（当前持久化为 JSON 文件 `.omnim`） |

## 适用范围

以下变更需要 RFC：

- 统一数据模型
- AMIR 指令 schema
- 几何表示
- 存储模型
- 同步协议
- 权限、安全与加密模型
- 导入/导出兼容性策略
- 领域扩展规则

## RFC 状态

| 状态 | 说明 |
| --- | --- |
| Draft | 讨论中 |
| Accepted | 已采纳，可开始实现 |
| Rejected | 已否决 |
| Superseded | 被更新的 RFC 取代 |
| Proposed / 未采纳 | 已写成提案，尚未被实现采纳 |

## 文件命名

```text
0001-title.md
0002-title.md
```

## RFC 模板

```markdown
# RFC 0001: 标题

## Summary（摘要）

## Motivation（动机）

## Detailed Design（详细设计）

## Compatibility（兼容性）

## Alternatives（备选方案）

## Open Questions（待解决问题）
```
