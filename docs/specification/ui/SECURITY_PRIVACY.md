---
title: "OmniM UI Security and Privacy"
status: design
last_updated: 2026-10-29
applies_to: "web/, crates/server"
evidence: "n/a (design contract)"
---
# OmniM UI Security and Privacy

> **Contract scope.** This document fixes the threat model, local-only disclosure, permission gating, path boundaries, client hardening, and telemetry consent. It is the normative source for requirements `UI-SEC-01` … `UI-SEC-05`.
>
> **Dual-track maintenance obligation**: `SECURITY_PRIVACY.zh-CN.md` is a maintained translation. Keep section numbering, rules, and requirement IDs aligned in the same change.
>
> **Status.** Design contract. The current server has no authentication; the UI does not disclose that.

## 1. Purpose and Threat Model

OmniM runs a local HTTP server and a browser client. The realistic threats are:

| Threat | Vector | Mitigation |
|---|---|---|
| Accidental LAN exposure | Server bound to `0.0.0.0` with no auth | UI disclosure + documented safe default |
| Malicious local page | CSRF against the local API | Origin check, no permissive CORS, CSP |
| Path traversal via open/save | Crafted path in a project request | Allowed-root validation server-side |
| Untrusted content injection | Object names, IFC strings, console output | No `dangerouslySetInnerHTML`; escape everywhere |
| Data exfiltration | Telemetry or diagnostics | Opt-in, no geometry, no names |
| Supply chain | npm dependency | Lockfile, audit, pinned versions |

## 2. Local-Only Disclosure

| Surface | Message |
|---|---|
| First run | Banner: "OmniM runs a local server with no authentication. Do not expose it to a network." |
| About dialog | Server bind address, authentication status, data directory |
| Settings → Security | Same information with a "copy diagnostics" action |
| Requirement | UI-SEC-01 |

Rules:

1. If the server detects a non-loopback bind, the UI must show a persistent warning banner.
2. The product must not offer "share link", "invite", or "publish" actions in this release.
3. The documentation and the UI must agree on this statement.

## 3. Permission Gating

Roles are reserved but the model matters now, because retrofitting permissions into every command is expensive.

| Role | Capabilities |
|---|---|
| Viewer | Read, select, measure, section, export |
| Modeler | Viewer + create, edit, delete, save |
| Administrator | Modeler + catalog/series edits, project settings, diagnostics |

Rules:

1. Disabled-by-permission commands are **hidden**, not shown disabled, to avoid leaking capability detail.
2. The command registry carries a `requiredRole` field; the UI filters before rendering.
3. Server-side enforcement is mandatory for every mutating endpoint; UI gating is usability, not security.
4. Requirement: UI-SEC-02.

## 4. Path Boundaries

| Rule | Value |
|---|---|
| Allowed roots | Configured via `OMNIM_PROJECTS_DIR` (default `~/.omnim/projects`) |
| Open | Path must resolve inside an allowed root after symlink resolution |
| Save as | Same rule; the UI shows the resolved path before writing |
| Traversal | `..`, absolute paths outside roots, and UNC paths rejected with `OMNI-IO-*` |
| Symlinks | Resolved; a symlink escaping the root is rejected |
| Recent list | Only paths that still resolve inside a root |
| Requirement | UI-SEC-03 |

## 5. Client Hardening

| Control | Rule |
|---|---|
| CSP | `default-src 'self'; img-src 'self' data: blob:; style-src 'self' 'unsafe-inline'; script-src 'self'` in production |
| Untrusted HTML | `dangerouslySetInnerHTML` prohibited for user content; console highlighting must operate on escaped text |
| URL handling | No `javascript:` links; external links use `rel="noopener noreferrer"` |
| File URLs | Object URLs revoked after use |
| Dependencies | Lockfile committed; audit in CI; no post-install scripts without review |
| Secrets | No API keys in the client bundle; AMIR keys stay server-side |
| Requirement | UI-SEC-04 |

## 6. Telemetry Consent and PII

| Rule | Value |
|---|---|
| Default | Off |
| Consent | Explicit, reversible, with a plain-language explanation of what is collected |
| Collected | Event names, timings, error codes, app/server versions, GPU class |
| Not collected | Geometry, object names, file paths, user names, project content |
| Storage | Local buffer until consent; discarded if declined |
| Requirement | UI-SEC-05 |

Rules:

1. Consent is separate from the license terms; declining must not degrade functionality.
2. Diagnostics export is a user-initiated action and is not telemetry.
3. Any new event property requires a privacy review recorded in [TELEMETRY](TELEMETRY.md) §2.

## 7. Review Checklist

| Item | Checked at |
|---|---|
| New endpoint validates path and role | PR review |
| New string rendered from user data is escaped | PR review |
| New dependency audited | PR review |
| CSP still passes with the new asset type | Release candidate |
| Local-only banner still accurate | Release |
| Telemetry event privacy-reviewed | Release |

## 8. Non-Goals

1. Multi-user authentication and SSO.
2. Encryption at rest for project files.
3. Role management UI beyond three fixed roles.
4. Client-side license enforcement.

## 9. Open Questions

| # | Question | Owner | Needed by |
|---|---|---|---|
| Q1 | Should the server default to `127.0.0.1` instead of `0.0.0.0`? | Backend / Product | G3 |
| Q2 | Is a token-based local auth (printed at startup) acceptable UX? | Product | G3 |
| Q3 | Do enterprise customers require an air-gapped mode statement? | Product | G3 |
