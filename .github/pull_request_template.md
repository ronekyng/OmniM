<!--
目标分支：任务 → develop；发布/热修 → main（见 docs/guides/BRANCHING.zh-CN.md）
-->

## 概述

<!-- 一句话说明本 PR 做了什么、为什么。 -->

## 关联文档

<!-- 文档先行：新规范/架构变更必须有已合入或同一 PR 内的文档。链接路径，不用 #编号。 -->

- `docs/...`

## 自查清单

- [ ] `scripts/test-all.sh` 本地通过
- [ ] `python scripts/check_docs.py` 本地通过
- [ ] 文档已更新（或在史上合并的文档中已覆盖）
- [ ] 实质性变更已更新 `docs/status/CHANGELOG(.zh-CN).md`
- [ ] 已 rebase 到目标分支最新（`git fetch && git rebase origin/<base>`）
- [ ] （仅 release/hotfix 合入 main 时）`main` 已回合并至 `develop`
