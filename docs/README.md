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

### 🚀 I want to try it
- [Getting Started](guides/GETTING_STARTED.md) — build the server, run the web UI, first steps ([中文](guides/GETTING_STARTED.zh-CN.md))
- [Building](guides/BUILDING.md) — toolchain, environment variables, scripts, troubleshooting ([中文](guides/BUILDING.zh-CN.md))

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
- [IFC Extension Strategy](specification/IFC_EXTENSION.md) — IFC compatibility policy ([中文](specification/IFC_EXTENSION.zh-CN.md))

### 🔍 I need an API or a term
- [REST API Reference](reference/API.md) — all server routes with curl examples ([中文](reference/API.zh-CN.md))
- [Omcom Command Reference](reference/OMCOM_REFERENCE.md) ([中文](reference/OMCOM_REFERENCE.zh-CN.md))
- [Glossary](reference/GLOSSARY.md) ([中文](reference/GLOSSARY.zh-CN.md))

### 📊 I want to know where the project really is
- [Roadmap](status/ROADMAP.md) — phase status with code evidence ([中文](status/ROADMAP.zh-CN.md))
- [Format Compatibility](status/FORMAT_COMPAT.md) — honest import/export matrix ([中文](status/FORMAT_COMPAT.zh-CN.md))
- [Code Deviations](status/CODE_DEVIATIONS.md) — ledger of spec-vs-code gaps ([中文](status/CODE_DEVIATIONS.zh-CN.md))
- [Changelog](status/CHANGELOG.md) ([中文](status/CHANGELOG.zh-CN.md))

### 💬 I want to shape the design
- [RFC Index](RFC/README.md) — RFC-0001 Omcom Text DSL (Draft), Storage Model proposal (Proposed) ([中文](RFC/README.zh-CN.md))
- [Research Notes](research/README.md) — competitor surveys, IFC 4x3 extracts, GenRule analysis

## By Directory

```
docs/
├── architecture/    # What IS built: current architecture, data flows, ADRs
├── guides/          # How to run, build, test, and navigate the codebase
├── specification/   # Design contracts: data model, Omcom, AMIR, catalog, domains
├── reference/       # Lookup: REST API, command reference, glossary
├── status/          # Truth: roadmap, format matrix, deviations, changelog
├── RFC/             # Open design proposals (text DSL, storage model)
└── research/        # Background research; not part of the contract
```

## Document Status Legend

| Mark | Meaning |
|---|---|
| ✅ | Verified against the implementation at the date of update |
| 🔀 | Partial — some sub-parts implemented, others not; the doc says which |
| 📅 Planned | Design exists, implementation does not |
| 💭 Proposed | Open proposal; see the RFC index |
