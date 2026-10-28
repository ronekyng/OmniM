---
title: "Git 分支与工作流"
status: current
last_updated: 2026-10-28
applies_to: "repo"
evidence: "OmniM 仓库的流程规范；分支模型由维护者于 2026-10-28 确认"
---
# Git 分支与工作流

[English](BRANCHING.md) | [中文](BRANCHING.zh-CN.md)

本指南定义本仓库的分支、提交、合并、发布与 CI 规则。所有参与者（无论人还是
AI agent）都必须遵守；这些规则同时已写入 `agents.md`。

## 分支模型

两条长期分支加若干短期任务分支：

| 分支 | 角色 | 生命周期 |
| --- | --- | --- |
| `main` | 发布基线，永远可构建、可测试、可演示 | 永久 |
| `develop` | 集成 / staging，领先 `main`，已推送到 `origin/develop` | 永久 |
| 短期分支 | 一个任务一条 | ≤ 1–2 天 |

```
main       ──────●──────────────●────────────●   (tag phase-8) (tag phase-8.1)
            \                  /  \         /
release      \     ●────●────●     \   ●──●──●        （冻结：只收 fix/docs）
              \   /                 \ /
develop        ●──●──●──●──●─────────●────────────    （staging，squash 合入）
                  ↑ feat/fix/docs 分支，合并后即删
```

## 分支命名

`<type>/<phase>-<kebab-slug>` —— slug 用英文小写；中文只出现在提交正文与
文档里。

| 前缀 | 起点 | 合入 | 示例 |
| --- | --- | --- | --- |
| `docs/` | `develop` | `develop` | `docs/p8-branching-policy` |
| `feat/` | `develop` | `develop` | `feat/p8-npoint-pick` |
| `fix/` | `develop` | `develop` / `release` | `fix/p8-pick-offset` |
| `refactor/`、`test/`、`chore/` | `develop` | `develop` | `chore/ci-branch-guard` |
| `agent/<tool>/` | `develop` | `develop` | `agent/opencode/p8-preview` |
| `release/` | `develop` | `main` | `release/phase-8` |
| `hotfix/` | `main` 的 tag | `main` | `hotfix/phase-8.1-server-panic` |

### 文档先行规则

`agents.md` 要求先文档后代码：

- 引入新规范或架构变更时：先开 `docs/*` 分支合入 `develop`，再开 `feat/*`
  分支，并在提交正文引用已合入的文档路径。
- 纯实现类任务：允许在同一个 `feat/*` 分支内既有文档提交又有代码提交，但
  合并后的提交正文必须保留文档链接。

## 提交规范

```
<type>(<scope>): <摘要>

为什么这样改（中文即可），关联文档：
docs/specification/XXX.md §N
```

- `type`：`feat fix docs refactor test chore perf build ci revert`
- `scope`：`core model sketch omcom server ifc amir web proto docs scripts`
- 破坏性变更：`feat(proto)!:`，正文写迁移办法
- 可选页脚：`Refs: docs/status/ROADMAP.zh-CN.md`、`Phase: 8`
- 一个提交只做一件事；文档与代码拆开提交（小的 `feat/*` 分支允许两者同分支）
- 由 `.githooks/commit-msg` 在本地强制（`scripts/setup-git.sh` 安装）

## 合并策略

| 场景 | 方式 | 理由 |
| --- | --- | --- |
| 短期分支 → `develop` | **Squash**，删除源分支 | `develop` 保持线性：一条任务一条提交 |
| `develop` → `release/*` | 新建分支，不合并 | 冻结窗口隔离 |
| `release/*` → `main` | **Merge commit**（`--no-ff`） | 在 `main` 上保留发布边界 |
| `hotfix/*` → `main` | **Merge commit**（`--no-ff`） | 同上 |
| **`main` → `develop`** | **Merge commit**，每次发版/热修后立即执行 | 防止双长期分支漂移（唯一硬性纪律） |

规则：

- 共享分支禁止 `rebase` / `force push`；共享分支只前进。
- 发 PR 前：`git fetch && git rebase origin/<base>`，然后跑
  `scripts/test-all.sh` 与 `python scripts/check_docs.py`。
- 永远不要 rebase `main` 或 `develop`。

## 发布与标签

本项目**不使用语义化版本**（见 `status/CHANGELOG.zh-CN.md`），变更日志按
路线图阶段组织。标签采用同一方案：全部为 `main` 上的 annotated tag。

| 标签 | 位置 | 含义 |
| --- | --- | --- |
| `phase-7` | `main` | 第 1–7 阶段的基线 |
| `phase-8` … | `main` | 阶段发布 |
| `phase-8.1` … | `main` | 发布后补丁 |
| `phase-8-rc.1` | `release/*` 分支 | 候选发布 |

`Cargo.toml` / `web/package.json` 的版本号**不随标签变动**；应用版本
`0.1.0` 只在真正发布二进制时才统一更新。

### 阶段发布

```bash
git switch develop && git pull --ff-only
git switch -c release/phase-8
# 冻结期：只允许 fix/docs；更新 CHANGELOG + ROADMAP
scripts/test-all.sh && python scripts/check_docs.py
git tag -a phase-8-rc.1 -m "Phase 8 release candidate 1"
git push -u origin release/phase-8 --follow-tags
# PR: release/phase-8 → main，用 merge commit 合并
git switch main && git pull --ff-only
git tag -a phase-8 -m "Phase 8: 领域扩展"
git push origin phase-8
git switch develop && git merge --no-ff main -m "chore: 回合并 main (phase-8)" && git push
```

### 热修

```bash
git switch -c hotfix/phase-8.1-server-panic phase-8
# 修复 + 测试 → PR 到 main（merge commit）→ tag phase-8.1 → 回合并 develop
```

## 分支保护（GitHub）

`main` 与 `develop` 均开启保护：

| 规则 | main | develop |
| --- | --- | --- |
| 禁止直接 push | ✅ | ✅ |
| 禁止 force push / 删除 | ✅ | ✅ |
| 必须 PR | ✅ | ✅ |
| 检查全绿且分支最新 | ✅ | ✅ |
| 合并方式 | Merge commit | Squash |
| 线性历史要求 | ❌（允许 release 的 merge commit） | ✅ |

仓库级设置：启用 **Squash** 与 **Merge commit**，禁用 **Rebase merge**。
允许的 PR base/head 组合由 CI 里的 `scripts/ci/branch_guard.py` 校验
（如 `feat/* → main` 会失败）。

若仓库为私有免费计划、保护规则不可用，则退化为本地 `pre-push` 钩子加
PR 自查清单。

## CI

`.github/workflows/ci.yml` 在 `pull_request` 及推送 `main` / `develop` 时
运行：

1. `cargo fmt --check`
2. `cargo clippy`（暂不拒绝告警，待存量告警清零后收紧）
3. `cargo test`
4. `web`：`npm ci && npm run build`
5. `python scripts/check_docs.py`
6. `scripts/ci/branch_guard.py`（仅 PR）

## 本地配置

```bash
scripts/setup-git.sh   # pull.ff=only、zdiff3、rerere、hooksPath、提交模板
```

安装内容：

- `.githooks/commit-msg` —— Conventional Commits 校验
- `.githooks/pre-push` —— 禁止直接推送 `main` / `develop`
- `.gitmessage` —— 提交信息模板

## PR 自查清单

`.github/pull_request_template.md` 与下列条目一致：

- [ ] 文档已更新（或给出已合入文档的链接）
- [ ] 本地通过 `scripts/test-all.sh`
- [ ] 本地通过 `python scripts/check_docs.py`
- [ ] 实质性变更已更新 CHANGELOG
- [ ] 目标分支正确（任务 → `develop`；发布/热修 → `main`）
- [ ] 若 release/hotfix 已合入 `main`：已回合并至 `develop`
