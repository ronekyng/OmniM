---
title: "OmniM Documentation"
status: current
last_updated: 2026-10-29
applies_to: "repo"
evidence: "verified against the docs tree 2026-10-27"
---
# OmniM Documentation

[English](README.md) | [中文](README.zh-CN.md)

This is the entry point to the OmniM documentation. It is organized by audience and intent: **guides** get you running, **architecture** describes what is actually built, **specification** defines the design contracts, **reference** is for lookup, and **status** tracks where the project really stands.

> **A note on honesty:** documentation in `architecture/` and `status/` describes the **current implementation** as verified against the code. Documents in `specification/` and `RFC/` describe **design intent**, including parts not yet implemented. Where the two differ, the difference is stated explicitly (see [CODE_DEVIATIONS](status/CODE_DEVIATIONS.md)).

## Table of Contents

- [Quick Start](#by-role)
- [By Role](#by-role)
- [By Directory](#by-directory)
- [Document Status Legend](#document-status-legend)

## By Role

### 🧱 I want to model
- [Modeler Handbook](MODELER_HANDBOOK.md) — domain-by-domain task flows, real UI/API steps, and honest gaps ([中文](MODELER_HANDBOOK.zh-CN.md))
- [Capability Matrix](CAPABILITY_MATRIX.md) — per-object capability rows: create, host, connect, measure, IFC, proto status ([中文](CAPABILITY_MATRIX.zh-CN.md))
- [Getting Started](guides/GETTING_STARTED.md) — build, launch, and first interactions ([中文](guides/GETTING_STARTED.zh-CN.md))
- ✅ **Persistence defaults on (2026-10-28):** mutations autosave to `project.omnim` / `$OMNIM_PROJECT`; `OMNIM_EPHEMERAL=1` disables it. Coverage details in [Getting Started](guides/GETTING_STARTED.md).

### 🚀 I want to try it
- [Getting Started](guides/GETTING_STARTED.md) — build the server, run the web UI, first steps ([中文](guides/GETTING_STARTED.zh-CN.md))
- [Building](guides/BUILDING.md) — toolchain, environment variables, scripts, troubleshooting ([中文](guides/BUILDING.zh-CN.md))
- [Branching & Workflow](guides/BRANCHING.md) — branch model, commit convention, merge and release rules ([中文](guides/BRANCHING.zh-CN.md))

### 🧭 I want to understand the system
- [Platform Architecture](architecture/PLATFORM_ARCH.md) — current implementation, layer by layer ([中文](architecture/PLATFORM_ARCH.zh-CN.md))
- [Data Flow](architecture/DATA_FLOW.md) — how a click (or a sentence, or an IFC file) becomes model data ([中文](architecture/DATA_FLOW.zh-CN.md))
- [Architecture Map](guides/ARCHITECTURE_MAP.md) — developer's crate-by-crate map, "where do I add X" recipes ([中文](guides/ARCHITECTURE_MAP.zh-CN.md))
- [Decision Records](architecture/DECISIONS.md) — why slvs, why truck, why JSON files, why protobuf-as-schema ([中文](architecture/DECISIONS.zh-CN.md))

### 📐 I want the design contracts
- [Unified Data Model](specification/DATA_MODEL.md) — the IFC-superset object model ([中文](specification/DATA_MODEL.zh-CN.md))
- [Omcom Specification](specification/OMCOM_SPEC.md) — the unified typed command system (Validator/Executor) ([中文](specification/OMCOM_SPEC.zh-CN.md))
- [AMIR Specification](specification/AMIR_SPEC.md) — natural-language → OmniCommand AI translation layer ([中文](specification/AMIR_SPEC.zh-CN.md))
- [Component Catalog & Series Library](specification/CATALOG_SERIES.md) — 元件库 / 系列库 (PARAGON/SPECON-inspired) ([中文](specification/CATALOG_SERIES.zh-CN.md))
- [Lifecycle Model](specification/LIFECYCLE.md) — nine-phase project lifecycle as data ([中文](specification/LIFECYCLE.zh-CN.md))
- [Domain Mapping](specification/DOMAINS.md) — Building / Civil / Plant / Mechanical object taxonomy ([中文](specification/DOMAINS.zh-CN.md))
- [Assembly Container](specification/ASSEMBLY.md) — container + Component/Part children, delete/transform semantics ([中文](specification/ASSEMBLY.zh-CN.md))
- [IFC Extension Strategy](specification/IFC_EXTENSION.md) — IFC compatibility policy ([中文](specification/IFC_EXTENSION.zh-CN.md))

### 🎨 I want to design or productize the UI

- [Frontend Vision & Scope](product/VISION_SCOPE.md) — positioning, users, non-goals, success metrics ([中文](product/VISION_SCOPE.zh-CN.md))
- [Personas & Journeys](product/PERSONAS_JOURNEYS.md) — three personas and five end-to-end journeys ([中文](product/PERSONAS_JOURNEYS.zh-CN.md))
- [Frontend PRD](product/PRD_FRONTEND.md) — 96 requirements in 11 areas with acceptance criteria ([中文](product/PRD_FRONTEND.zh-CN.md))
- [Requirements Register](product/REQUIREMENTS_REGISTER.md) — the RTM: ID, contract, verification, gate, status ([中文](product/REQUIREMENTS_REGISTER.zh-CN.md))
- [Shell & IA](specification/ui/IA_SHELL.md) · [Design System](specification/ui/DESIGN_SYSTEM.md) · [Interaction](specification/ui/INTERACTION.md) ([中文](specification/ui/IA_SHELL.zh-CN.md))
- [Feedback & Errors](specification/ui/FEEDBACK_ERRORS.md) · [Data Integrity](specification/ui/DATA_INTEGRITY.md) · [A11Y](specification/ui/A11Y.md) · [i18n](specification/ui/I18N_L10N.md) · [Performance](specification/ui/PERFORMANCE.md)
- [Compatibility](specification/ui/COMPATIBILITY.md) · [Security & Privacy](specification/ui/SECURITY_PRIVACY.md) · [Telemetry](specification/ui/TELEMETRY.md) — GA-scope contracts
- [Definition of Done](process/DEFINITION_OF_DONE.md) · [Design Ops](process/DESIGN_OPS.md) · [Localization Workflow](guides/LOCALIZATION_WORKFLOW.md)
- [Release & Support Checklist](process/RELEASE_CHECKLIST.md) — release notes, migration, runbook, known issues ([中文](process/RELEASE_CHECKLIST.zh-CN.md))
- [G0 Review Record (2026-10-29)](process/G0_REVIEW_2026-10-29.md) — independent review findings and dispositions ([中文](process/G0_REVIEW_2026-10-29.zh-CN.md))
- [Web Architecture](architecture/WEB_ARCHITECTURE.md) — what the frontend is today, with measured debt ([中文](architecture/WEB_ARCHITECTURE.zh-CN.md))
- [Frontend Maturity](status/FRONTEND_MATURITY.md) · [Baseline](status/FRONTEND_BASELINE.md) · [Release Readiness](status/RELEASE_READINESS.md) ([中文](status/FRONTEND_MATURITY.zh-CN.md))
- [Keyboard Map](reference/KEYBOARD_MAP.md) · [Error Codes](reference/ERROR_CODES.md) ([中文](reference/KEYBOARD_MAP.zh-CN.md))
- [Frontend Guide](guides/FRONTEND_GUIDE.md) — five recipes: panel, command, dialog, token, string ([中文](guides/FRONTEND_GUIDE.zh-CN.md))
- [RFC-0002: UI Shell & Design System](RFC/RFC-0002-ui-shell-and-design-system.md)

### 🔍 I need an API or a term
- [REST API Reference](reference/API.md) — all server routes with curl examples ([中文](reference/API.zh-CN.md))
- [Omcom Command Reference](reference/OMCOM_REFERENCE.md) ([中文](reference/OMCOM_REFERENCE.zh-CN.md))
- [Glossary](reference/GLOSSARY.md) ([中文](reference/GLOSSARY.zh-CN.md))

### 📊 I want to know where the project really is
- [Roadmap](status/ROADMAP.md) — workstream status, gates and dependencies with code evidence ([中文](status/ROADMAP.zh-CN.md))
- [Format Compatibility](status/FORMAT_COMPAT.md) — honest import/export matrix ([中文](status/FORMAT_COMPAT.zh-CN.md))
- [Code Deviations](status/CODE_DEVIATIONS.md) — ledger of spec-vs-code gaps ([中文](status/CODE_DEVIATIONS.zh-CN.md))
- [Audit & acceptance records (2026-10-30)](status/CODE_AUDIT_2026-10-30.zh-CN.md) — code audit; [modeler docs audit](status/MODELER_DOCS_AUDIT_2026-10-30.zh-CN.md) / [re-audit](status/MODELER_DOCS_REAUDIT_2026-10-30.zh-CN.md); [UI/UX audit](status/MODELER_UIUX_AUDIT_2026-10-30.zh-CN.md) / [re-audit](status/MODELER_UIUX_REAUDIT_2026-10-30.zh-CN.md) (zh-CN snapshots)
- [Changelog](status/CHANGELOG.md) ([中文](status/CHANGELOG.zh-CN.md))

### 💬 I want to shape the design
- [RFC Index](RFC/README.md) — RFC-0001 Omcom Text DSL (Draft), Storage Model proposal (Proposed) ([中文](RFC/README.zh-CN.md))
- [Research Notes](research/README.md) — competitor surveys, IFC 4x3 extracts, GenRule analysis

## By Directory

```
docs/
├── product/         # Why and for whom: vision, personas, PRD, requirement register
├── architecture/    # What IS built: current architecture, data flows, ADRs
├── guides/          # How to run, build, test, and navigate the codebase
├── specification/   # Design contracts: data model, Omcom, AMIR, catalog, domains
│   └── ui/          # UI contracts: shell, design system, interaction, a11y, i18n, perf
├── reference/       # Lookup: REST API, command reference, keyboard map, error codes, glossary
├── status/          # Truth: roadmap, format matrix, frontend maturity, release readiness
├── process/         # Governance: definition of done, design operations
├── RFC/             # Open design proposals (text DSL, storage model, UI shell)
└── research/        # Background research; not part of the contract
```

Two modeler-facing references sit at the docs root: [Modeler Handbook](MODELER_HANDBOOK.md) · [Capability Matrix](CAPABILITY_MATRIX.md).

## Document Status Legend

| Mark | Meaning |
|---|---|
| ✅ | Verified against the implementation at the date of update |
| 🔀 | Partial — some sub-parts implemented, others not; the doc says which |
| 📅 Planned | Design exists, implementation does not |
| 💭 Proposed | Open proposal; see the RFC index |
