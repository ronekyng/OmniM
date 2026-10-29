---
title: "OmniM Localization Workflow"
status: design
last_updated: 2026-10-29
applies_to: "web/"
evidence: "n/a (process contract)"
---
# OmniM Localization Workflow

> **Document role.** The operational procedure behind [I18N_L10N](../specification/ui/I18N_L10N.md): who extracts, who translates, how review works, and what blocks a release.
>
> **Dual-track maintenance obligation**: `LOCALIZATION_WORKFLOW.zh-CN.md` is a maintained translation. Steps, roles, and checks must match.
>
> **Status.** Design contract. There is no string catalogue or translation pipeline in the code base yet.

## 1. Purpose

Translation fails when it is a last-minute export. This workflow makes localization a continuous, checked activity with clear handoffs.

## 2. Roles

| Role | Responsibility |
|---|---|
| String author (Frontend) | Adds keys, English source, context notes |
| Terminology owner (Product / discipline lead) | Approves new terms in the glossary |
| Translator | Produces `zh-CN` values with domain accuracy |
| Reviewer | Verifies meaning and UI fit in context |
| Release owner | Confirms locale coverage before the gate |

One person may hold several roles; the handoff still happens in writing.

## 3. String Lifecycle

```text
author (en-US) → entry in change (zh-CN) → context screenshot → review → merge
      │                                                              │
      └── later change: update both locales in the same commit ───────┘
```

| Stage | Artifact | Rule |
|---|---|---|
| Authoring | `en-US` entry with a context note | No literal strings in components |
| Draft translation | `zh-CN` entry | Same key, same placeholders |
| Context | Screenshot of the changed screen | Both locales when layout differs |
| Review | Reviewer sign-off | Terminology consistency checked |
| Merge | One commit containing both locales | Missing counterpart fails CI |

## 4. Translation Handoff

| Field | Required |
|---|---|
| Key | As authored |
| English source | Full sentence, not a fragment |
| Context note | Where it appears and what it means |
| Placeholders | Listed with type and example |
| Screenshot | Optional but preferred for new surfaces |
| Glossary terms | Linked when the string contains a domain term |

Handoff format: a diff of the `en-US` catalogue plus the context table above. No spreadsheets in the repository.

## 5. Terminology

1. A new domain term is added to [GLOSSARY](../reference/GLOSSARY.md) (both languages) **before** it is used in the UI.
2. The same concept uses one term across panels; synonyms are rejected in review.
3. Unit symbols are not translated (`m`, `mm`, `°`).
4. Object type prefixes in auto-names stay ASCII by contract ([INTERACTION](../specification/ui/INTERACTION.md) §9.1).

## 6. Review

| Check | Blocking |
|---|---|
| Key parity between locales | Yes |
| Placeholder parity and types | Yes |
| Pseudo-locale layout pass | Yes at G2 |
| Terminology matches glossary | Yes |
| Tone matches the product voice (direct, no marketing language) | Yes |
| Screenshot review of changed surfaces | Yes for new surfaces |

## 7. Release Checklist

- [ ] All keys present in `zh-CN` and `en-US`.
- [ ] No unused keys above the agreed threshold.
- [ ] Pseudo-locale screenshots reviewed.
- [ ] Locale switch works without reload.
- [ ] `<html lang>` updates correctly.
- [ ] Numbers, dates, and units follow the locale.
- [ ] CHANGELOG notes locale coverage and any new terms.

## 8. Tooling

| Tool | Purpose | Status |
|---|---|---|
| `check_ui_contracts.py --strings` | Detect CJK literals outside catalogues | 📅 |
| Key parity test | Fail on missing keys | 📅 |
| Pseudo-locale build | `en-XA` visual pass | 📅 |
| Unused-key report | Hygiene | 📅 |
| Screenshot diff | Context review | 📅 |

## 9. Non-Goals

1. Translation management system integration.
2. Machine translation as a shipping path.
3. Localized screenshots in the repository for every string.

## 10. Open Questions

| # | Question | Owner | Needed by |
|---|---|---|---|
| Q1 | Who is the terminology owner for civil and plant domains? | Product | G2 |
| Q2 | Do we translate the AI assistant's system prompts? | Product | G2 |
| Q3 | Is a third locale needed before GA? | Product | G3 |
