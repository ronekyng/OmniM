---
title: "OmniM Release and Support Checklist"
status: design
last_updated: 2026-10-29
applies_to: "web/, crates/server"
evidence: "n/a (process contract)"
---
# OmniM Release and Support Checklist

> **Document role.** Fills the G3 gap between "the build passes" and "a customer can install, migrate, and get help". Defines the release-note format, migration and rollback procedure, support runbook, and known-issues register.
>
> **Dual-track maintenance obligation**: `RELEASE_CHECKLIST.zh-CN.md` is a maintained translation. Section numbering and checklist items must match.
>
> **Status.** Design contract. No release has been executed under this checklist yet.

## 1. Purpose and Scope

A commercial release needs four artifacts beyond the binary: what changed, how to move from the previous version, what to do when it breaks, and what is knowingly broken. This document defines all four and the sign-off that proves they exist.

Out of scope: marketing collateral, pricing, and store listings.

## 2. Release Notes Template

Published with every build; stored in the release record and linked from the About dialog.

```markdown
# OmniM <version> (<date>)

## Highlights
- <one to three user-visible outcomes>

## Added
- <feature> (<requirement IDs>)

## Changed
- <behavior change> — <who is affected>

## Fixed
- <defect> (<issue link>)

## Known issues
- <issue> — <workaround>

## Compatibility
- Project schema: <version>; readable by <app versions>
- Server API: <compatible client versions>
- Browsers: <matrix reference>

## Upgrade notes
- <steps, or "none">
```

Rules:

1. Every entry names a user-visible outcome, not a file.
2. Requirement IDs are included for traceability.
3. Breaking changes appear under **Upgrade notes**, never buried in **Changed**.
4. Known issues link to the register in §5.

## 3. Migration and Rollback

| Case | Requirement |
|---|---|
| Project schema unchanged | Release note states "no migration" |
| Project schema added fields | Forward-compatible read; old app refuses newer schema with `OMNI-IO-0006` |
| Project schema changed semantics | A migration command or an explicit refusal; never silent reinterpretation |
| Server API changed | Client and server version compatibility table in the release note |
| Rollback | Previous build must open files written by the new build, or the note states the downgrade path |

Rules:

1. A schema bump without a migration path is a release blocker.
2. Before migration, the app writes a backup next to the project (`<name>.omnim.bak`).
3. Migration is idempotent and reports the number of objects touched.
4. Rollback after migration requires the backup; the UI says so before migrating.

## 4. Support Runbook

| Scenario | First action | Escalation |
|---|---|---|
| App will not start | Check server reachability, port, and `OMNIM_PROJECT` | Collect diagnostics bundle |
| Model fails to load | Check schema version, dependency report, recovery snapshots | Ask for the project plus dependency list |
| Save fails | Check path roots, permissions, disk space, external modification | Collect diagnostics + server log |
| Crash or white screen | Ask for diagnostics bundle and last actions | Reproduce with the fixture project |
| Rendering broken | Check WebGL2 support and GPU driver | Compatibility matrix entry |
| Performance regression | Ask for object count, browser, GPU class | Compare against the baseline harness |

Rules:

1. Support never asks for a project file unless the user consents and the data is not confidential.
2. Every ticket records app version, server version, OS, browser, GPU, and error code.
3. The runbook is tested once per release with a synthetic ticket.

## 5. Known Issues Register

| Field | Rule |
|---|---|
| ID | `KI-<year>-<nn>` |
| Summary | One sentence, user terms |
| Impact | Workaround exists / no workaround |
| Affected versions | Range |
| Workaround | Concrete steps or "none" |
| Target fix | Version or "unscheduled" |
| Owner | Named person |

Rules: the register is published with the release; a closed issue moves to the changelog; the register is reviewed at every gate.

## 6. Version and Compatibility Statement

| Statement | Source |
|---|---|
| App version and build hash | Build metadata |
| Project schema version | `Project.schema_version` |
| Server API compatibility | Client/server table |
| Browser and GPU matrix | [COMPATIBILITY](../specification/ui/COMPATIBILITY.md) §2 |
| Accessibility conformance | [A11Y](../specification/ui/A11Y.md) §10 |

The About dialog shows the first four; the release note shows all five.

## 7. Sign-off Checklist

- [ ] Release notes written from the template.
- [ ] Migration path tested forward and backward.
- [ ] Rollback tested with the backup file.
- [ ] Support runbook scenario executed once.
- [ ] Known-issues register published.
- [ ] Compatibility statement updated.
- [ ] Diagnostics export verified to contain no geometry.
- [ ] CHANGELOG linked.
- [ ] Gate owner signed ([RELEASE_READINESS](../status/RELEASE_READINESS.md)).

## 8. Non-Goals

1. Automated app-store distribution.
2. In-app update delivery.
3. Customer-specific release branches.

## 9. Open Questions

| # | Question | Owner | Needed by |
|---|---|---|---|
| Q1 | Who owns the known-issues register between releases? | Product / Support | G3 |
| Q2 | Is a migration tool in scope for the first commercial release? | Product / Backend | G3 |
| Q3 | Does support require a signed data-handling agreement before receiving a project file? | Legal | G3 |
