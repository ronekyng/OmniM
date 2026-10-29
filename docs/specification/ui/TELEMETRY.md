---
title: "OmniM UI Telemetry and Supportability"
status: design
last_updated: 2026-10-29
applies_to: "web/, crates/server"
evidence: "n/a (design contract)"
---
# OmniM UI Telemetry and Supportability

> **Contract scope.** This document fixes the event dictionary, funnels, error events, performance metrics, diagnostics export, and help surfaces. It is the normative source for requirements `UI-TLM-01` … `UI-TLM-06`.
>
> **Dual-track maintenance obligation**: `TELEMETRY.zh-CN.md` is a maintained translation. Keep section numbering, event names, and requirement IDs aligned in the same change.
>
> **Status.** Design contract. There is no telemetry pipeline today; if none is funded, the fallback in §6 is the delivery model.

## 1. Purpose

Telemetry exists to answer three questions without asking users: does the product work, where do users fail, and what should we fix first. It is not surveillance and it is not a substitute for usability testing.

**Honesty rule:** do not document events that no collector receives. If there is no endpoint, this document specifies the local buffer and export path only.

## 2. Event Dictionary

| Rule | Value |
|---|---|
| Naming | `omnim.<area>.<object>.<action>`, lower_snake for multiword |
| Version | `schema_version` integer on every event |
| Properties | Typed, enumerated, documented here |
| PII | Prohibited: no names, paths, geometry, or free text |
| Stability | Renaming requires a schema version bump and a transition period |

Baseline events:

| Event | Trigger | Properties |
|---|---|---|
| `omnim.app.start` | Client boot | app_version, locale, theme |
| `omnim.project.create` | New project committed | template_id, unit_system |
| `omnim.project.open` | Project opened | source (recent/file/sample), load_ms, object_count_bucket |
| `omnim.project.save` | Explicit save | result, duration_ms, object_count_bucket |
| `omnim.project.recover` | Recovery prompt shown | offered (bool), accepted (bool) |
| `omnim.object.create` | Creation succeeded | object_type, method (ribbon/palette/console) |
| `omnim.object.delete` | Deletion confirmed | object_type, child_count_bucket |
| `omnim.selection.change` | Selection settled 500 ms | count_bucket, source (viewport/tree) |
| `omnim.panel.switch` | Dock tab change | dock, tab_id |
| `omnim.theme.change` | Theme switched | from, to |
| `omnim.locale.change` | Locale switched | from, to |
| `omnim.error.shown` | Any coded error surfaced | code, severity, area |
| `omnim.diagnostics.export` | Bundle exported | trigger (user/error_boundary) |

Requirement: UI-TLM-01.

## 3. Funnels

**Onboarding funnel:** `app.start` → `project.create|open` → `object.create` → `project.save` → `project.open (reopen)`.

| Step | Success definition | Target |
|---|---|---|
| Start | App shell interactive | ≥98% of sessions |
| Project ready | A project is open | ≥95% |
| First object | At least one object created | ≥80% of new users |
| First save | Explicit save succeeded | ≥75% |
| Reopen | Same project opened again within 7 days | ≥40% |

Rules: funnel events carry a `session_id` (random, non-persistent) and no user identifier; drop-off is analyzed by step, never by person.

Requirement: UI-TLM-02.

## 4. Error Events

| Rule | Value |
|---|---|
| Required fields | `code`, `severity`, `area`, `operation`, `result`, `app_version` |
| Prohibited | Object names, file paths, geometry, stack traces in telemetry |
| Correlation | `correlation_id` shared with the local diagnostic log |
| Sampling | 100% for blocking, 25% for recoverable, 0% for validation |
| Requirement | UI-TLM-03 |

## 5. Performance Metrics

| Metric | Aggregation | Dimensions |
|---|---|---|
| First paint / workspace ready | p50, p95 | browser, gpu_class, object_count_bucket |
| Interaction latency | p50, p95 | command_id |
| Frame rate | p50, p05 | object_count_bucket, scene_state |
| Memory | p50, p95 | object_count_bucket |
| Save duration | p50, p95 | project_size_bucket |

Rules: buckets, never exact counts, when the count could identify a project; no per-frame events.

Requirement: UI-TLM-04.

## 6. Diagnostics Export

| Item | Detail |
|---|---|
| Trigger | User action in Help/About or an error-boundary action |
| Format | `.zip` with manifest, logs, commands, environment |
| Contents | See [FEEDBACK_ERRORS](FEEDBACK_ERRORS.md) §9 |
| Redaction | Paths reduced to `<root>/…`; object names replaced with `<object>` |
| Retention | User-controlled; nothing uploaded automatically |
| Fallback | This export is the support channel when no collector exists |

Requirement: UI-TLM-05.

## 7. Help and Support Surfaces

| Surface | Content | Shortcut |
|---|---|---|
| Shortcut reference | Generated from the command registry | `F1` |
| Contextual help | Short explanation per panel and per disabled command | `?` on focus |
| Sample projects | At least one discipline sample, offline | Project home |
| About | Version, build, server version, GPU, data directory, licenses | Help menu |
| Diagnostics | Export bundle, copy summary | Help menu |
| Release notes | In-app link to CHANGELOG | About |

Rules: every help surface must work offline; no dead links; screenshots in help match the current theme.

Requirement: UI-TLM-06.

## 8. Non-Goals

1. Session recording or heatmaps.
2. User accounts or cross-device identity.
3. Real-time crash alerting.
4. A/B experimentation framework.

## 9. Open Questions

| # | Question | Owner | Needed by |
|---|---|---|---|
| Q1 | Is there a collector endpoint, or is this export-only for the first release? | Product / Backend | G3 |
| Q2 | Does the license agreement require a separate telemetry consent dialog? | Legal | G3 |
| Q3 | Which metrics are safe to report when the project name itself is sensitive? | Product / Security | G3 |
