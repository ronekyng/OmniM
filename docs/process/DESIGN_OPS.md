---
title: "OmniM Design Operations"
status: design
last_updated: 2026-10-29
applies_to: "web/"
evidence: "n/a (process contract)"
---
# OmniM Design Operations

> **Document role.** Governance for the frontend design system and UI contracts: who decides, what needs review, how components and tokens evolve, and how changes stay traceable.
>
> **Dual-track maintenance obligation**: `DESIGN_OPS.zh-CN.md` is a maintained translation. Roles, change classes, and lifecycle states must match.
>
> **Status.** Design contract. The project is small; roles may be held by the same person, but the responsibilities stay explicit.

## 1. Purpose

Without governance, a design system becomes a suggestion. This document makes changes reviewable without turning a small team into a committee.

## 2. Roles and RACI

| Activity | Product | Design | Frontend | QA | Docs |
|---|---|---|---|---|---|
| Requirement definition | **A/R** | C | C | I | I |
| Shell/IA contract | A | **R** | C | I | C |
| Design system contract | I | **A/R** | C | I | C |
| Interaction contract | C | **A/R** | C | C | I |
| Implementation | I | C | **A/R** | C | I |
| Verification evidence | I | C | R | **A** | I |
| Release gate sign-off | **A** | C | R | R | C |
| Documentation | C | C | C | I | **A/R** |

R = responsible, A = accountable, C = consulted, I = informed.

## 3. Change Classes

| Class | Examples | Review | Notice |
|---|---|---|---|
| **Editorial** | Typo, wording, link fix | Docs owner | None |
| **Minor** | New component state, new token in an existing role, new e2e test | Design + Frontend | Changelog |
| **Major** | New contract section, new shell region, theme model change | Design review + RFC if cross-cutting | RFC or ADR |
| **Breaking** | Removing a component, renaming a token, changing a shortcut | RFC + migration note + deprecation window | Release notes |

Rule: when in doubt, classify upward. A "minor" change that alters muscle memory is major.

## 4. Design Review

| Trigger | Required artifacts | Decision window |
|---|---|---|
| New panel or shell region | Wireframe, states table, screenshots | 2 working days |
| New component | State matrix, tokens used, a11y notes | 2 working days |
| Token change | Before/after screenshots, contrast check | 1 working day |
| Navigation or shortcut change | KEYBOARD_MAP diff, migration note | 2 working days |

Review output is recorded as: approved, approved with conditions, or rejected. Conditions have owners and dates.

## 5. Component Lifecycle

```text
proposed → alpha → stable → deprecated → removed
```

| State | Meaning | Allowed in production |
|---|---|---|
| proposed | Documented only | No |
| alpha | Implemented, API may change | Behind a flag only |
| stable | Full state matrix, tested, documented | Yes |
| deprecated | Replacement exists; removal scheduled | Yes, with warning |
| removed | Deleted after the deprecation window | — |

A component may not jump from proposed to stable without an alpha period in at least one wave.

## 6. Token and Contract Versioning

| Artifact | Versioning | Breaking change |
|---|---|---|
| `tokens.css` | Semver comment header | Renaming or removing a token requires a deprecation alias for one minor version |
| UI contracts (`specification/ui/*`) | Document version in front matter | Requires an RFC |
| Component API | Semver in the component module | Major bump with a codemod or migration note |
| Requirement IDs | Permanent | Never renamed; superseded instead |

## 7. Decision Records

| Decision type | Record |
|---|---|
| Cross-cutting architecture | ADR in [DECISIONS](../architecture/DECISIONS.md) |
| Open design alternatives | RFC in [RFC](../RFC/README.md) |
| Requirement trade-off | `DEFINITION_OF_DONE` exception + release note |
| Visual exception | Inline comment with owner and expiry, plus a debt entry |

## 8. Cadence and Metrics

| Activity | Cadence |
|---|---|
| Contract review | Per wave |
| Maturity rescore | Per gate |
| Visual baseline refresh | Per release candidate |
| Accessibility audit | Per gate |
| Design debt triage | Monthly |

Metrics tracked: requirement coverage, maturity score, hardcoded-color count, string externalization, component state completeness, a11y critical count.

## 9. Non-Goals

1. A separate design-system team.
2. A public component marketplace.
3. Approval workflows inside the repository tooling.

## 10. Open Questions

| # | Question | Owner | Needed by |
|---|---|---|---|
| Q1 | Should design review happen before or after implementation for small changes? | Design | G1 |
| Q2 | Who owns the visual regression baseline approvals? | Design / Frontend | G2 |
| Q3 | Do we need a written UX research cadence for the pilot? | Product | G2 |
