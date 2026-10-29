---
title: "OmniM Frontend Vision & Scope"
status: design
last_updated: 2026-10-29
applies_to: "web/"
evidence: "n/a (product requirement)"
---
# OmniM Frontend Vision & Scope

> **Document role.** This is the product layer of the frontend documentation set. It answers *why* the UI exists, *for whom*, and *what counts as done*. It does not describe implementation details; those live in `docs/specification/ui/`.
>
> **Dual-track maintenance obligation**: `VISION_SCOPE.zh-CN.md` is a maintained translation of this document. When editing this English document, update the Chinese twin in the same change: keep section numbering, tables, and requirement IDs aligned.
>
> **Honesty rule.** Anything in this document is a **target**, not a claim about current code. Current implementation is described in [WEB_ARCHITECTURE](../architecture/WEB_ARCHITECTURE.md) and scored in [FRONTEND_MATURITY](../status/FRONTEND_MATURITY.md).

## 1. Purpose

OmniM's modeling core is already deep: typed commands, undo/redo, a feature engine, a BRep kernel, a catalog/series system, IFC import/export, and an AI translation layer. The gap is **productization of the frontend**: the UI can execute operations, but it does not yet behave like a commercial engineering application.

This document fixes the target: what "commercial grade" means for the OmniM frontend, which users it must serve, and which outcomes are measurable.

## 2. Product Positioning

> **OmniM is a local-first, multi-discipline engineering modeler for small teams: model, standardize, and exchange building, civil, plant, and mechanical data in one application, with reviewable commands and honest format limits.**

Positioning constraints:

- **Local-first.** A single local server plus a browser client. No cloud dependency, no mandatory account.
- **Multi-discipline.** Building, civil, plant, and mechanical share one data model; the UI must not pretend otherwise.
- **Command-reviewable.** Every mutation is an Omcom command, undoable, and inspectable.
- **Honest about limits.** The UI states what is unsupported instead of hiding it.

## 3. Target Users

| Persona | Primary goal | Frequency | Reference |
|---|---|---|---|
| **P1 Discipline modeler** | Turn intent into geometry and save it safely | Daily | [PERSONAS_JOURNEYS §1.1](PERSONAS_JOURNEYS.md) |
| **P2 Discipline lead / reviewer** | Inspect, measure, section, and validate someone else's model | Weekly | [PERSONAS_JOURNEYS §1.2](PERSONAS_JOURNEYS.md) |
| **P3 Catalog & standards administrator** | Maintain component definitions, series trees, units, and language packs | Monthly | [PERSONAS_JOURNEYS §1.3](PERSONAS_JOURNEYS.md) |

Explicitly **not** target users in this cycle: consumers, mobile-only users, and multi-tenant cloud collaborators.

## 4. In-Scope Workflows

| ID | Workflow | Persona | Success condition |
|---|---|---|---|
| W1 | First run → new project → create → save → reopen | P1 | Model survives a restart with zero data loss |
| W2 | Place a catalog component through a series decision tree | P1 | Correct `ComponentDef` instantiated with valid parameters |
| W3 | Inspect a model: select, read properties, measure, section | P2 | Reviewer can answer "what is this and how big is it" without the console |
| W4 | Recover from an interrupted or failed operation | P1 | No silent data loss; every failure has a recovery action |
| W5 | Maintain catalog/series and switch language/units | P3 | Definitions editable; UI fully localized |

## 5. Explicit Non-Goals

1. **Photorealistic rendering.** Visual quality targets are "readable engineering viewport", not marketing imagery.
2. **Cloud collaboration and multi-tenant accounts.** A single-user local server is the current delivery model.
3. **Mobile and touch-first UI.** Desktop-class pointer and keyboard workflows only.
4. **Design-tool parity.** No Figma-style freeform canvas for the shell; the UI is a docked engineering workspace.
5. **Full WCAG AAA.** The target is WCAG 2.2 AA on critical paths, with a high-contrast theme as an accessibility aid.
6. **Plugin UI marketplace.** The panel registry is internal extensibility, not a public plugin SDK.

These non-goals are binding: proposals that violate them require an RFC.

## 6. Success Metrics

Product-level outcomes, measured per release gate. Detailed budgets live in [PERFORMANCE](../specification/ui/PERFORMANCE.md).

| Metric | Target | Measured by |
|---|---|---|
| Time to first saved model (new user, sample project) | ≤ 10 minutes | Moderated task test |
| Save/reopen data loss | 0 objects | Automated round-trip e2e |
| Crash-to-recovery data loss | ≤ 5 minutes of work | Recovery drill |
| First meaningful paint (reference machine) | ≤ 2.5 s P95 | Performance harness |
| Critical-path WCAG 2.2 AA conformance | 100% | axe + manual keyboard pass |
| User-visible strings externalized | 100% | String scan |
| Supported browser matrix pass rate | 100% | Compatibility suite |

## 7. Document Map & Traceability

```text
VISION_SCOPE (this file)
  └── PERSONAS_JOURNEYS ............ journeys J1–J5
        └── PRD_FRONTEND ........... requirements R1–R11 (IDs UI-*)
              ├── REQUIREMENTS_REGISTER ... single source of truth
              ├── specification/ui/* ...... contracts
              ├── status/FRONTEND_MATURITY  scores and coverage
              └── status/RELEASE_READINESS  gate evidence
```

**Rule:** a requirement that is not in [REQUIREMENTS_REGISTER](REQUIREMENTS_REGISTER.md) does not exist. A requirement that has no verification entry cannot pass a gate.

## 8. Open Questions

| # | Question | Owner | Needed by |
|---|---|---|---|
| Q1 | Is desktop packaging (Tauri/Electron) in scope for the first commercial release? | Product | G0 |
| Q2 | Is there a telemetry collector, or is local diagnostic export the only support channel? | Product / Backend | G3 |
| Q3 | Which discipline is the launch discipline for the reference workflow? | Product | G1 |
| Q4 | Does "commercial" require licensing/entitlement UX in the first release? | Product | G3 |
