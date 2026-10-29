---
title: "OmniM UI Data Integrity and Project Lifecycle"
status: design
last_updated: 2026-10-29
applies_to: "web/, crates/server"
evidence: "n/a (design contract)"
---
# OmniM UI Data Integrity and Project Lifecycle

> **Contract scope.** This document fixes the project home, open/recent, metadata, save state machine, autosave, crash recovery, conflict handling, snapshots, and the minimal server contract. It is the normative source for requirements `UI-DAT-01` … `UI-DAT-10`.
>
> **Dual-track maintenance obligation**: `DATA_INTEGRITY.zh-CN.md` is a maintained translation. Keep section numbering, endpoint names, and requirement IDs aligned in the same change.
>
> **Status.** Design contract. The client now calls `POST /api/project/save` (explicit save, Ctrl/Cmd+S) and `POST /api/project/save-as`; the project home route plus open/new endpoints remain unimplemented. See [CODE_DEVIATIONS §7](../../status/CODE_DEVIATIONS.md).

## 1. Purpose

Modeling work is the user's asset. Data integrity is therefore a product feature with explicit states, not an implementation detail. The contract guarantees three things:

1. The user always knows which project is open and whether it is saved.
2. No operation can silently discard work.
3. Any interruption has a bounded, visible recovery path.

## 2. Project Model in the UI

### 2.1 Project home

| Element | Rule |
|---|---|
| Route | `/projects` |
| Content | New project, Open file, Recent (max 10), Sample projects |
| Recent row | Name, path, last opened, size, missing-dependency badge |
| Empty state | Explains that no project is open and offers the sample project |
| Behavior | The workspace is not reachable until a project is created or opened |
| Requirement | UI-DAT-01 |

**Invariant:** the application never loads a project implicitly from a working directory. A file may be offered as "last session", but the user must confirm.

### 2.2 Open and recent

| Rule | Value |
|---|---|
| Open | File picker restricted to allowed roots (see [SECURITY_PRIVACY](SECURITY_PRIVACY.md) §4) |
| Recent storage | Local user settings, not the project file |
| Missing file | Row stays with an error badge and a "locate…" action |
| Concurrent open | Single project per session in this release |
| Requirement | UI-DAT-01 |

### 2.3 Metadata

Editable before the first save and afterwards through Project settings.

| Field | Type | Default | Persisted |
|---|---|---|---|
| Name | string, unique per root | "Untitled" | Yes |
| Description | string | empty | Yes |
| Unit system | `metric` \| `imperial` (display units; stored geometry stays metric) | `metric` | Yes |
| Schema version | integer | current | Yes (read-only) |
| Author / organization | string | empty | Yes |
| Created / modified | timestamps | now | Yes (read-only) |

Requirement: UI-DAT-07.

### 2.4 Dependency report

On open, the client compares required catalog and series references against what the server can resolve.

| Condition | UI |
|---|---|
| All resolved | Silent |
| Missing series | Banner: "3 series missing — components may fall back to mesh" with a list |
| Missing catalog definition | Banner: "2 component definitions unresolved" with affected object count |
| External file missing | Banner with "locate…" and "continue read-only" |
| Requirement | UI-DAT-09 |

**Rule:** the report is informational before editing and becomes blocking only for operations that need the missing definition.

## 3. Save State Machine

### 3.1 States

```text
clean ──mutation──▶ dirty ──save──▶ saving ──ok──▶ clean
  ▲                   ▲                 │
  │                   └────fail─────────┘
  └──────────────── save as ────────────┘
```

| State | Status bar | Title | Allowed actions |
|---|---|---|---|
| `clean` | `Saved HH:MM:SS` | project name | everything |
| `dirty` | `Unsaved changes` | `• project name` | everything; close prompts |
| `saving` | `Saving…` | project name | read-only operations |
| `failed` | `Save failed` | `• project name` | retry, save as, export |
| `readonly` | `Read-only — external change` | `🔒 project name` | view only |
| Requirement | UI-DAT-02, UI-DAT-03 |

### 3.2 Dirty tracking and last-saved time

- A mutation marks the session dirty; a successful save records `last_saved_at` and clears it.
- View changes, selection, panel layout, and theme do **not** mark dirty.
- Autosave does not clear dirty (dirty means "not in the user's target file").
- The timestamp is shown in local time with seconds.
- Requirement: UI-DAT-03.

### 3.3 Unsaved-changes interception

Triggered by: close tab, open another project, new project, reload, navigate away.

| Choice | Behavior |
|---|---|
| Save | Save, then continue; on failure stay and show error |
| Discard | Continue; the project is closed without writing |
| Cancel | Abort the navigation, keep the session |

Rules:

1. The dialog names the project and the last save time.
2. Browser `beforeunload` is used only while `dirty`, and never blocks an explicit in-app navigation.
3. Keyboard: `Esc` = cancel, `Enter` = save (the safe default).
4. Requirement: UI-DAT-06.

### 3.4 Save failure

| Cause | UI |
|---|---|
| Server unreachable | Error dialog + retry + save as; state stays `dirty` |
| Permission denied | Error with path and recovery; offer save as |
| Disk full | Error with size hint; offer save as to another location |
| Serialization error | Error code + diagnostics export; never partial write |

**Invariant:** a failed save never clears dirty and never reports success.

## 4. Autosave

| Rule | Value |
|---|---|
| Interval | 60 s of inactivity after a mutation, or 5 minutes of continuous editing |
| Target | Recovery file outside the project file (e.g. `~/.omnim/recovery/<project-hash>/`) |
| Blocking | Must not block input; uses the same atomic write as explicit save |
| Failure | Logged, surfaced as a warning banner once per session, never silently ignored |
| Retention | Keep the last 5 snapshots, prune older |
| Requirement | UI-DAT-04 |

Autosave never writes to the user's project path. That path is written only by explicit save, save-as, or the server's configured autosave policy — never both.

## 5. Crash Recovery

| Step | Behavior |
|---|---|
| Detect | On project open, look for recovery snapshots newer than the project's mtime |
| Prompt | "Recover unsaved work from HH:MM (N minutes)?" with Compare, Recover, Discard |
| Recover | Load the snapshot into the session, mark dirty, keep the original file untouched |
| Discard | Delete the snapshot after confirmation |
| Guarantee | At most 5 minutes of work lost, provided the session ran ≥1 minute |
| Requirement | UI-DAT-05 |

**Verification:** kill the server and browser mid-edit, restart, confirm the recovery prompt and the object count.

## 6. External Modification and Conflicts

| Signal | Detection | UI |
|---|---|---|
| mtime changed | Stat comparison on focus and before save | Read-only banner with Reload / Save as |
| Content hash changed | Hash on open and before save | Same, with a stronger warning |
| File deleted | Stat missing | Read-only banner with "Save as" |
| Requirement | UI-DAT-08 |

Rules:

1. Never overwrite an externally modified file automatically.
2. Reload discards the session's unsaved changes only after explicit confirmation.
3. If both sides changed, offer save-as instead of a merge (no merge UI in this release).

## 7. Destructive Operation Snapshots

| Operation | Snapshot | Restore point |
|---|---|---|
| Delete object(s) | Yes, implicit via undo | Undo |
| Import with replace | Yes, named snapshot | Project settings → Snapshots |
| Bulk property update | Yes, if > 20 objects | Snapshots |
| Feature rebuild failure | Yes, pre-rebuild | Snapshots |

Rules:

1. Snapshots are stored server-side next to the project (`.omnim.snapshots/`).
2. Each snapshot records a label, timestamp, object count, and the command id that created it.
3. Restoring a snapshot is itself undoable in-session and always offers "save as" first.
4. Requirement: UI-DAT-10.

## 8. Server Contract (minimal)

These endpoints are **required** by the UI. They do not exist yet except `GET /api/project` and `POST /api/project/save`.

| Method | Path | Purpose | Notes |
|---|---|---|---|
| GET | `/api/project` | Project state | Adds `path`, `dirty`, `last_saved_at`, `schema_version` |
| POST | `/api/project/new` | Create and switch | Body: name, root, optional template |
| POST | `/api/project/open` | Open by path | Path restricted to allowed roots |
| POST | `/api/project/save` | Explicit save | Existing; returns path and timestamp |
| POST | `/api/project/save-as` | Save to a new path | Returns new path; updates session |
| GET | `/api/project/recent` | Recent list | Server-side list with existence check |
| GET | `/api/project/autosaves` | Recovery snapshots | For the current project only |
| POST | `/api/project/restore` | Restore a snapshot | Body: snapshot id |

Rules:

1. `save_path` becomes mutable session state; switching projects must not leak undo history.
2. Every endpoint returns a typed error with an `OMNI-*` code.
3. New/open must be idempotent on retry after a network failure.
4. Undo/redo state is reset on project switch.

## 9. Non-Goals

1. Multi-document interface (several projects open at once).
2. Cloud sync or conflict merge.
3. Version control integration.
4. Automatic background upload of recovery files.

## 10. Open Questions

| # | Question | Owner | Needed by |
|---|---|---|---|
| Q1 | Should autosave write a recovery file or append to a journal? | Backend | G2 |
| Q2 | Do snapshots belong to the project artifact or to the user profile? | Product / Backend | G3 |
| Q3 | Should "save as" preserve the undo history? | Frontend | G2 |
| Q4 | Is a 60 s autosave interval acceptable for very large projects (save > 5 s)? | Frontend | G2 |
