---
title: "Git Branching & Workflow"
status: current
last_updated: 2026-10-28
applies_to: "repo"
evidence: "policy for the OmniM repository; branch model confirmed by the maintainer 2026-10-28"
---
# Git Branching & Workflow

[English](BRANCHING.md) | [中文](BRANCHING.zh-CN.md)

This guide defines how branches, commits, merges, releases, and CI work in this
repository. Every contributor (human or AI agent) must follow it. The rules here
are also mirrored in `agents.md`.

## Branch Model

Two long-lived branches plus short-lived task branches:

| Branch   | Role                                                                 | Lifetime     |
| -------- | -------------------------------------------------------------------- | ------------ |
| `main`   | Release baseline. Always buildable, testable, demonstrable.          | permanent    |
| `develop`| Integration / staging. Leads `main`. Pushed to `origin/develop`.     | permanent    |
| short-lived branches | one task per branch                                      | ≤ 1–2 days   |

```
main       ──────●──────────────●────────────●   (tag phase-8) (tag phase-8.1)
            \                  /  \         /
release      \     ●────●────●     \   ●──●──●        (freeze: fix/docs only)
              \   /                 \ /
develop        ●──●──●──●──●─────────●────────────    (staging, squash merges)
                  ↑ feat/fix/docs branches, deleted after merge
```

## Branch Naming

`<type>/<phase>-<kebab-slug>` — English lowercase slug; Chinese is allowed only
in commit bodies and documentation.

| Prefix                             | Starts from  | Merges into          | Example                        |
| ---------------------------------- | ------------ | -------------------- | ------------------------------ |
| `docs/`                            | `develop`    | `develop`            | `docs/p8-branching-policy`     |
| `feat/`                            | `develop`    | `develop`            | `feat/p8-npoint-pick`          |
| `fix/`                             | `develop`    | `develop` / `release`| `fix/p8-pick-offset`           |
| `refactor/`, `test/`, `chore/`     | `develop`    | `develop`            | `chore/ci-branch-guard`        |
| `agent/<tool>/`                    | `develop`    | `develop`            | `agent/opencode/p8-preview`    |
| `release/`                         | `develop`    | `main`               | `release/phase-8`              |
| `hotfix/`                          | `main` tag   | `main`               | `hotfix/phase-8.1-server-panic`|

### Docs-first rule

`agents.md` requires documentation before code:

- For new specifications or architecture changes: open a `docs/*` branch first,
  merge it into `develop`, then open a `feat/*` branch whose commits reference
  the merged doc path.
- For implementation-only tasks, the `feat/*` branch may contain both a docs
  commit and code commits, but the merged commit body must keep doc links.

## Commit Convention

```
<type>(<scope>): <summary>

Why this change (Chinese is fine), linked docs:
docs/specification/XXX.md §N
```

- `type`: `feat fix docs refactor test chore perf build ci revert`
- `scope`: `core model sketch omcom server ifc amir web proto docs scripts`
- Breaking change: `feat(proto)!:` with a migration note in the body
- Optional footers: `Refs: docs/status/ROADMAP.zh-CN.md`, `Phase: 8`
- One concern per commit; keep docs and code commits separate (unless the same
  small `feat/*` branch carries both).
- Enforced locally by `.githooks/commit-msg` (install via
  `scripts/setup-git.sh`).

## Merge Policy

| Scenario                  | Method                              | Reason                                        |
| ------------------------- | ----------------------------------- | --------------------------------------------- |
| short-lived → `develop`   | **Squash**, delete the source branch| `develop` stays linear: one task, one commit  |
| `develop` → `release/*`   | branch off, no merge                | isolation of the freeze window                |
| `release/*` → `main`      | **Merge commit** (`--no-ff`)        | visible release boundary on `main`            |
| `hotfix/*` → `main`       | **Merge commit** (`--no-ff`)        | same                                          |
| **`main` → `develop`**    | **Merge commit**, immediately after every release/hotfix | prevents drift (the one hard rule) |

Rules:

- No `rebase` / `force push` on shared branches; shared branches only advance.
- Before opening a PR: `git fetch && git rebase origin/<base>`, then run
  `scripts/test-all.sh` and `python scripts/check_docs.py`.
- Never rebase `main` or `develop`.

## Releases & Tags

The project does **not** use SemVer (see
`status/CHANGELOG.zh-CN.md`); changelog entries are organized by roadmap phase.
Tags follow the same scheme: annotated tags on `main`.

| Tag             | Where              | Meaning                       |
| --------------- | ------------------ | ----------------------------- |
| `phase-7`       | `main`             | baseline of phases 1–7        |
| `phase-8` …     | `main`             | phase release                 |
| `phase-8.1` …   | `main`             | post-release patch            |
| `phase-8-rc.1`  | `release/*` branch | release candidate             |

`Cargo.toml` / `web/package.json` versions are **not** bumped per tag; the app
version `0.1.0` is only updated when binaries are actually published.

### Phase release

```bash
git switch develop && git pull --ff-only
git switch -c release/phase-8
# freeze: only fix/docs allowed; update CHANGELOG + ROADMAP
scripts/test-all.sh && python scripts/check_docs.py
git tag -a phase-8-rc.1 -m "Phase 8 release candidate 1"
git push -u origin release/phase-8 --follow-tags
# PR: release/phase-8 → main, merged with a merge commit
git switch main && git pull --ff-only
git tag -a phase-8 -m "Phase 8: domain expansion"
git push origin phase-8
git switch develop && git merge --no-ff main -m "chore: back-merge main (phase-8)" && git push
```

### Hotfix

```bash
git switch -c hotfix/phase-8.1-server-panic phase-8
# fix + tests → PR to main (merge commit) → tag phase-8.1 → back-merge into develop
```

## Branch Protection (GitHub)

Both `main` and `develop` are protected:

| Rule                                | main                                 | develop |
| ----------------------------------- | ------------------------------------ | ------- |
| no direct push                      | ✅                                   | ✅      |
| no force push / no delete           | ✅                                   | ✅      |
| PR required                         | ✅                                   | ✅      |
| required checks green & up to date  | ✅                                   | ✅      |
| merge methods                       | Merge commit                         | Squash  |
| require linear history              | ❌ (release merge commits allowed)   | ✅      |

Repository settings: enable **Squash** and **Merge commit**; disable **Rebase
merge**. Allowed PR base/head pairs are validated by
`scripts/ci/branch_guard.py` in CI (e.g. `feat/* → main` fails).

If the repository is private on the free plan and protection rules are
unavailable, fall back to the local `pre-push` hook plus the PR checklist.

## CI

`.github/workflows/ci.yml` runs on `pull_request` and pushes to `main` /
`develop`:

1. `cargo fmt --check`
2. `cargo clippy` (warnings not denied yet; tightened once the backlog is clean)
3. `cargo test`
4. `web`: `npm ci && npm run build`
5. `python scripts/check_docs.py`
6. `scripts/ci/branch_guard.py` (PR only)

## Local Setup

```bash
scripts/setup-git.sh   # pull.ff=only, zdiff3, rerere, hooksPath, commit template
```

Installs:

- `.githooks/commit-msg` — Conventional Commits check
- `.githooks/pre-push` — blocks direct pushes to `main`/`develop`
- `.gitmessage` — commit template

## PR Checklist

`.github/pull_request_template.md` mirrors these items:

- [ ] Documentation updated (or a link to the already-merged doc)
- [ ] `scripts/test-all.sh` passes locally
- [ ] `python scripts/check_docs.py` passes
- [ ] CHANGELOG entry updated for substantive changes
- [ ] Correct target branch (`develop` for tasks, `main` for release/hotfix)
- [ ] After merging release/hotfix into `main`: back-merge into `develop`
