#!/usr/bin/env python3
"""Validate allowed PR base/head pairs (docs/guides/BRANCHING.md).

Rules:
  base=develop  -> head must be a short-lived task branch or release candidate work:
                   feat/ fix/ docs/ refactor/ test/ chore/ agent/<tool>/ perf/ build/
  base=main     -> head must be  release/  or  hotfix/
Anything else (e.g. feat/* -> main) fails with exit 1.

Runs inside GitHub Actions; also usable locally:
    GITHUB_BASE_REF=develop GITHUB_HEAD_REF=feat/p8-x python scripts/ci/branch_guard.py
"""
from __future__ import annotations

import os
import re
import sys

TASK_RE = re.compile(
    r"^(feat|fix|docs|refactor|test|chore|agent|perf|build)/[a-z0-9][a-z0-9._/-]*$"
)
RELEASE_RE = re.compile(r"^release/(phase-[0-9]+|test-.*)$")
HOTFIX_RE = re.compile(r"^hotfix/(phase-[0-9]+(\.[0-9]+)?)-[a-z0-9][a-z0-9.-]*$")


def eval_pair(base: str, head: str) -> int:
    if not base or not head:
        print("branch-guard: 缺少 GITHUB_BASE_REF / GITHUB_HEAD_REF，跳过校验")
        return 0

    if head in ("main", "develop"):
        print(f"branch-guard: PR head 不能是长期分支 (head={head})")
        return 1

    if base == "develop":
        if TASK_RE.match(head):
            print(f"branch-guard: OK — {head} -> develop")
            return 0
        print(
            "branch-guard: 到 develop 的 PR 必须来自 docs/ feat/ fix/ refactor/ test/"
            " chore/ agent/<tool>/ perf/ build/ 分支 (docs/guides/BRANCHING.zh-CN.md)"
        )
        return 1

    if base == "main":
        if RELEASE_RE.match(head) or HOTFIX_RE.match(head):
            print(f"branch-guard: OK — {head} -> main")
            return 0
        print(
            "branch-guard: 到 main 的 PR 必须来自 release/ 或 hotfix/，"
            "其余任务请先合入 develop (docs/guides/BRANCHING.zh-CN.md)"
        )
        return 1

    # base 是 release/* 等：允许 fix/docs/test/chore 收冻结期改动
    if base.startswith("release/"):
        if re.match(r"^(fix|docs|test|chore)/", head):
            print(f"branch-guard: OK — {head} -> {base}")
            return 0

    print(f"branch-guard: 不允许的 PR 组合 base={base!r} head={head!r}")
    return 1


def main() -> int:
    return eval_pair(os.environ.get("GITHUB_BASE_REF", ""), os.environ.get("GITHUB_HEAD_REF", ""))


if __name__ == "__main__":
    sys.exit(main())
