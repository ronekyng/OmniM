---
title: "OmniM UI Error Codes"
status: design
last_updated: 2026-10-29
applies_to: "web/, crates/server"
evidence: "n/a (design contract); current code throws plain strings, no codes exist yet"
---
# OmniM UI Error Codes

> **Document role.** The stable catalogue that joins UI copy, server responses, diagnostics, and telemetry. Format and message rules are defined in [FEEDBACK_ERRORS](../specification/ui/FEEDBACK_ERRORS.md) §5.
>
> **Dual-track maintenance obligation**: `ERROR_CODES.zh-CN.md` is a maintained translation. Codes, areas, and severities must match exactly.
>
> **Honesty rule.** No code in this table is emitted by the current code base. Codes are reserved so that implementations do not invent incompatible numbering.

## 1. Code Format

```text
OMNI-<AREA>-<NNNN>
  UI   client-side interaction and state
  API  server response rejected by validation
  NET  transport and connectivity
  IO   file system, save, load
  GEO  geometry validation and kernel
  CAT  catalog and series resolution
  IFC  import/export
  AUTH authentication and permission
  SYS  unexpected internal failure
```

Rules: codes are permanent; retired codes remain reserved; a code maps to exactly one message key and one severity.

## 2. Severity and Surface

| Severity | Surface | Auto-dismiss |
|---|---|---|
| Blocking | Dialog | No |
| Degraded | Banner | No |
| Recoverable | Inline + error toast | No |
| Validation | Field-level inline | On edit |
| Unexpected | Error boundary | No |

## 3. Client Interaction (UI)

| Code | Meaning | Severity | Recovery |
|---|---|---|---|
| OMNI-UI-0001 | Operation blocked because another operation is running | Recoverable | Wait or cancel |
| OMNI-UI-0002 | Invalid numeric input | Validation | Correct the field |
| OMNI-UI-0003 | Value outside allowed range | Validation | Show the range |
| OMNI-UI-0004 | Selection required before this command | Recoverable | Select an object |
| OMNI-UI-0005 | Command unavailable in the current discipline | Recoverable | Switch discipline |
| OMNI-UI-0006 | Picking session cancelled | Validation | Restart the tool |
| OMNI-UI-0007 | Too few points to complete the shape | Validation | Add points |
| OMNI-UI-0008 | Panel failed to render | Unexpected | Retry panel |
| OMNI-UI-0009 | Layout state was corrupt and reset | Degraded | Informational |
| OMNI-UI-0010 | Clipboard unavailable in this browser context | Recoverable | Use the menu |

## 4. Server Validation (API)

| Code | Meaning | Severity | Recovery |
|---|---|---|---|
| OMNI-API-0001 | Request body failed validation | Recoverable | Show field errors |
| OMNI-API-0002 | Unknown object id | Recoverable | Refresh the tree |
| OMNI-API-0003 | Operation not allowed for this object type | Recoverable | Explain the rule |
| OMNI-API-0004 | Series gate rejected the component | Recoverable | Choose a valid series |
| OMNI-API-0005 | Connection type incompatible | Recoverable | Choose a compatible connector |
| OMNI-API-0006 | Undo stack empty | Recoverable | Informational |
| OMNI-API-0007 | Concurrent modification detected | Degraded | Reload or save as |

## 5. Network (NET)

| Code | Meaning | Severity | Recovery |
|---|---|---|---|
| OMNI-NET-0001 | Server unreachable | Blocking | Start the server, retry |
| OMNI-NET-0002 | Request timed out | Recoverable | Retry |
| OMNI-NET-0003 | Connection lost mid-session | Degraded | Reconnect loop |
| OMNI-NET-0004 | Response was not valid JSON | Unexpected | Diagnostics |
| OMNI-NET-0005 | Request rejected by CORS/Origin check | Blocking | Explain local-only policy |

## 6. File and Project (IO)

| Code | Meaning | Severity | Recovery |
|---|---|---|---|
| OMNI-IO-0001 | Project file not found | Recoverable | Locate, recent list |
| OMNI-IO-0002 | Permission denied | Recoverable | Save as |
| OMNI-IO-0003 | Disk full | Blocking | Save as elsewhere |
| OMNI-IO-0004 | Save failed on the server | Recoverable | Retry, save as |
| OMNI-IO-0005 | Path outside allowed roots | Blocking | Choose a project in the root |
| OMNI-IO-0006 | Project schema version is newer than the app | Blocking | Update the app |
| OMNI-IO-0007 | Project file is corrupted | Blocking | Open recovery snapshot |
| OMNI-IO-0008 | External modification detected | Degraded | Reload, save as |

## 7. Geometry (GEO)

| Code | Meaning | Severity | Recovery |
|---|---|---|---|
| OMNI-GEO-0001 | Degenerate path (zero length) | Validation | Re-pick |
| OMNI-GEO-0002 | Self-intersecting boundary | Validation | Re-pick |
| OMNI-GEO-0003 | Fewer than three points for a region | Validation | Add points |
| OMNI-GEO-0004 | Collinear boundary | Validation | Re-pick |
| OMNI-GEO-0005 | Extrusion produced no volume | Recoverable | Adjust parameters |
| OMNI-GEO-0006 | Boolean operation failed | Recoverable | Adjust operands |
| OMNI-GEO-0007 | Feature rebuild failed | Recoverable | Inspect the feature tree |

## 8. Catalog and Series (CAT)

| Code | Meaning | Severity | Recovery |
|---|---|---|---|
| OMNI-CAT-0001 | Component definition not found | Recoverable | Check the library |
| OMNI-CAT-0002 | Parameter outside the definition's domain | Validation | Show bounds |
| OMNI-CAT-0003 | Series selector has no matching spec | Recoverable | Change answers |
| OMNI-CAT-0004 | Library import failed validation | Recoverable | Show the failing entry |
| OMNI-CAT-0005 | Duplicate component id | Validation | Rename |

## 9. IFC (IFC)

| Code | Meaning | Severity | Recovery |
|---|---|---|---|
| OMNI-IFC-0001 | Unsupported IFC schema version | Blocking | Convert to IFC2x3/IFC4 |
| OMNI-IFC-0002 | Malformed file | Blocking | Validate externally |
| OMNI-IFC-0003 | Unsupported entity skipped | Degraded | List skipped types |
| OMNI-IFC-0004 | Geometry could not be converted | Degraded | Import as proxy |
| OMNI-IFC-0005 | Export produced unsupported mapping | Degraded | List affected objects |

## 10. Authentication and System (AUTH, SYS)

| Code | Meaning | Severity | Recovery |
|---|---|---|---|
| OMNI-AUTH-0001 | Operation requires a higher role | Recoverable | Hidden by default |
| OMNI-AUTH-0002 | Session not authenticated | Blocking | Local-only notice |
| OMNI-SYS-0001 | Unexpected internal error | Unexpected | Diagnostics, reload |
| OMNI-SYS-0002 | Renderer context lost | Blocking | Reload scene |
| OMNI-SYS-0003 | Out of memory | Blocking | Reduce scene, reload |

## 11. Maintenance Rules

1. A new code requires a row here **and** a message key in every locale.
2. `scripts/check_ui_contracts.py --errors` scans source and docs; unknown codes fail the build.
3. Messages never include object names or paths in telemetry; the UI may include the object name.
4. Codes are not localized; the surrounding message is.
5. A code's meaning never changes after release; create a new code instead.

## 12. Non-Goals

1. HTTP-status-only error reporting.
2. Stack traces shown to end users.
3. Per-customer error code ranges.

## 13. Open Questions

| # | Question | Owner | Needed by |
|---|---|---|---|
| Q1 | Should server crates share this numbering space? | Backend | G2 |
| Q2 | Do we need a 5-digit range for future geometry sub-codes? | Frontend | G3 |
| Q3 | Is `OMNI-SYS-0003` detectable reliably in the browser? | Frontend | G3 |
