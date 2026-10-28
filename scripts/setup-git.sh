#!/usr/bin/env bash
# One-time (idempotent) local git setup for OmniM, matching docs/guides/BRANCHING.md
set -euo pipefail

cd "$(dirname "$0")/.."

echo "[omnim] configuring local git..."

# Never pull in unexpected merge commits
git config --local pull.ff only
# More readable conflict hunks
git config --local merge.conflictstyle zdiff3
# Reuse recorded conflict resolutions across rebases
git config --local rerere.enabled true
# Install repo-local hooks
git config --local core.hooksPath .githooks
# Commit template
git config --local commit.template .gitmessage
# Remove stale VS Code merge-base config pointing at origin/main
git config --local --unset branch.develop.vscode-merge-base 2>/dev/null || true

chmod +x .githooks/commit-msg .githooks/pre-push

echo "[omnim] done. Shared branches (main/develop) may only be updated via PR;"
echo "[omnim] see docs/guides/BRANCHING.zh-CN.md"
