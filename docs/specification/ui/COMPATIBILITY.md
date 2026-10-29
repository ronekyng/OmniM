---
title: "OmniM UI Compatibility Matrix"
status: design
last_updated: 2026-10-29
applies_to: "web/"
evidence: "n/a (design contract)"
---
# OmniM UI Compatibility Matrix

> **Contract scope.** This document fixes supported browsers, WebGL capability gating, display and input matrix, 3D mouse policy, network degradation, and the test procedure. It is the normative source for requirements `UI-CMP-01` … `UI-CMP-06`.
>
> **Dual-track maintenance obligation**: `COMPATIBILITY.zh-CN.md` is a maintained translation. Keep section numbering, matrix rows, and requirement IDs aligned in the same change.
>
> **Status.** Design contract. No compatibility suite exists today.

## 1. Purpose

A commercial engineering application states exactly where it runs. "It works in my browser" is not a support policy.

## 2. Browser Matrix

| Browser | Versions | Tier | Notes |
|---|---|---|---|
| Chrome / Edge (Chromium) | Latest two stable | **Primary** | Reference browser for performance |
| Firefox | Latest two stable | Supported | Pointer lock and shadow quality verified |
| Safari (macOS) | Latest two major | Supported | WebGL2 required; no Windows Safari |
| Safari (iOS/iPadOS) | — | **Unsupported** | Not a target platform |
| Other Chromium builds | Same engine version | Best effort | No support commitment |

Rules:

1. "Latest two" means the current and previous major version at release time.
2. A browser drops out of support only with a release-note deprecation one minor version ahead.
3. Tests run on the primary browser every PR; the full matrix runs per release candidate.

Requirement: UI-CMP-01.

## 3. WebGL Capability

| Check | Action when missing |
|---|---|
| WebGL2 context | Block with a page explaining the requirement and how to enable hardware acceleration |
| `EXT_color_buffer_float` | Disable HDR-ish tone mapping fallback; continue |
| Instanced arrays | Disable instancing path; warn in diagnostics |
| Max texture size < 4096 | Reduce shadow map; continue |
| Software renderer detected | Warn once, allow entry, record a telemetry event |

Rules:

1. Capability detection happens before the first scene build and is cached for the session.
2. The blocking screen must not be a white page; it includes the required capability, the detected GPU string, and a diagnostics export.
3. The app never silently degrades to a 2D-only mode.

Requirement: UI-CMP-02.

## 4. Display and Input

### 4.1 DPI and zoom

| Setting | Requirement |
|---|---|
| DPI 100% / 125% / 150% / 200% | No clipped controls, no overlapping docks |
| Browser zoom 80% / 100% / 125% / 150% / 200% | Layout reflows; canvas resizes correctly |
| Mixed-DPI multi-monitor | Moving the window between monitors preserves layout |
| Color profile | sRGB assumed; no color management for model data |

### 4.2 Minimum size

| Size | Behavior |
|---|---|
| ≥1920×1080 | Full layout |
| 1440×900 | Full layout, reference screenshot size |
| 1280×720 | **Minimum certified**; side docks collapse to icon rails |
| 1024×720 – 1279×720 | Degraded: icon rails only, not certified for support |
| <1024×720 | Blocking notice with the minimum requirement |

Requirement: UI-CMP-03, UI-CMP-05.

## 5. 3D Mouse

| Rule | Value |
|---|---|
| Status | **Not supported in this release**, explicitly declared |
| Detection | If a 3Dconnexion device is present, the About dialog states it is unsupported |
| Future | Support requires a dedicated navigation adapter and a device test matrix |
| Requirement | UI-CMP-04 |

Declaring a non-feature is part of the compatibility contract: users must not discover it by trial.

## 6. Network Degradation

| Condition | UI |
|---|---|
| Server unreachable at start | Blocking screen with "start the local server" instructions and Retry |
| Connection lost mid-session | Banner "disconnected — changes may not be saved"; save disabled; retry loop with backoff |
| Slow response (>5 s) | Progress indicator; no duplicate submissions |
| Partial response failure | Operation-level error with retry; no silent partial model |
| Requirement | UI-CMP-06 |

Rules: no offline editing queue in this release; the user is told the truth instead of being given a false "saved" state.

## 7. Test Procedure

| Layer | Scope | Frequency |
|---|---|---|
| Smoke | Primary browser, critical paths | Every PR |
| Matrix | All supported browsers, critical paths | Release candidate |
| Display matrix | Four DPI values, three viewport sizes, two themes | Release candidate |
| Capability | Forced WebGL2 failure, software renderer | Release candidate |
| Evidence | Screenshot set archived with the release | Release |

## 8. Non-Goals

1. Internet Explorer or legacy Edge.
2. Mobile browsers.
3. Linux distribution-specific browser builds beyond Chromium/Firefox.
4. Remote desktop GPU acceleration guarantees.

## 9. Open Questions

| # | Question | Owner | Needed by |
|---|---|---|---|
| Q1 | Is Safari support required for the first commercial release? | Product | G3 |
| Q2 | Do we support Firefox ESR for enterprise customers? | Product | G3 |
| Q3 | Should 4K at 200% DPI be a tested reference configuration? | Frontend | G3 |
