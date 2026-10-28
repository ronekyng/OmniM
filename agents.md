执行新任务务必先写文档，文档没有的内容，不能直接写代码。
无论是写文档还是写代码，都要先给出详细方案，具体执行写入操作前，必须重新核对方案。

## Git 工作流（硬性规则，详见 docs/guides/BRANCHING.zh-CN.md）

1. 长期分支只有 `main` 与 `develop`（staging）；任何任务先从 `develop` 拉短期分支，命名 `<type>/<phase>-<slug>`（如 `feat/p8-npoint-pick`），禁止向 `main`/`develop` 直接 push 或 force push。
2. 引入新规范/架构变更：先走 `docs/*` 分支合入 `develop`，再开 `feat/*` 实现并在提交正文引用文档路径；提交信息一律 Conventional Commits（`<type>(<scope>): <摘要>`，scope 用 crate 或 area 名）。
3. 任务分支以 **squash** 合入 `develop` 后立即删除；`main` 只接收 `release/*` / `hotfix/*` 的 merge commit，且每次 `main` 变动后必须立即把 `main` 回合并进 `develop`；标签用 `phase-8`、`phase-8.1`、`phase-8-rc.1` 这类阶段式 annotated tag（本项目不使用 SemVer）。合并前必须 rebase 最新并跑 `scripts/test-all.sh` 与 `python scripts/check_docs.py`。
