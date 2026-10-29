#!/usr/bin/env python3
"""Enforce the single write path (ADR-0006).

Every mutating endpoint must go through `mutation::apply` / `Executor`; only
explicitly allow-listed read-only handlers may take `project.write()`, and only
explicit persistence handlers (`save_project`, `save_project_as`) may call
`Project::save` directly.

Scans `crates/server/src/lib.rs` (top-level `async fn` bodies) and reports:

  1. handlers holding `project.write()` that are not in `READ_ONLY_ALLOWED`
  2. handlers calling `.save(` other than `save_project` / `save_project_as`
  3. handlers writing the global `LIBRARY` other than `LIBRARY_ALLOWED`

Exit code 1 when any violation is found, 0 otherwise. Intended for CI and for
the Wave-1 migration check (`python3 scripts/check_write_paths.py`).
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

LIB_RS = Path(__file__).resolve().parent.parent / "crates/server/src/lib.rs"

# Handlers that legitimately take the project write lock without mutating it.
# Keep this list short and justify each entry — it is the audit surface.
READ_ONLY_ALLOWED = {
    # Rebuilds feature caches before reading section lines; behaviour is a
    # cache refresh, not a user mutation.
    "section_handler": "rebuilds component caches before reading section lines",
}

# Handlers that hold the project write lock but delegate the mutation itself to
# the Executor (single write path), so the lock is expected.
EXECUTOR_LOCK_ALLOWED = {
    "create_object": "builds a CreateObject command and runs Executor",
    "delete_object": "builds a DeleteObject command and runs Executor",
    "execute_omcom": "runs a client-supplied OmniCommand through Executor",
    "genrule_create": "legacy genrule path; records undo via Executor::record_create",
    "undo": "pop-and-apply of the Executor undo stack",
    "redo": "pop-and-apply of the Executor redo stack",
    "import_ifc": "records a whole-project replace command on the Executor (bulk import)",
}

DIRECT_WRITE_ALLOWED = READ_ONLY_ALLOWED | EXECUTOR_LOCK_ALLOWED

# Handlers that mutate the session-global LIBRARY. Catalog definition edits go
# through commands (Slice 3); bulk file import is a whole-library operation that
# does not touch Project state and therefore has no project undo entry.
LIBRARY_ALLOWED = {
    "import_catalog_handler": "bulk .omnimlib import into the session-global library",
}

SAVE_ALLOWED = {"save_project", "save_project_as"}

CALL_RE = re.compile(r"^async fn (\w+)\s*\(", re.MULTILINE)


def handler_bodies(src: str) -> dict[str, str]:
    """Split the file into top-level `async fn` bodies (rustfmt: body ends at `}` in column 0)."""
    lines = src.splitlines()
    starts: list[tuple[int, str]] = []
    for i, line in enumerate(lines):
        m = CALL_RE.match(line)
        if m:
            starts.append((i, m.group(1)))
    bodies: dict[str, str] = {}
    for n, (start, name) in enumerate(starts):
        end = len(lines)
        for j in range(start + 1, len(lines)):
            if lines[j].rstrip() == "}":
                end = j + 1
                break
            # A new top-level item begins before the previous body closed means
            # the previous was a one-liner; rustfmt keeps `}` on its own line.
        bodies[name] = "\n".join(lines[start:end])
    return bodies


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--lib", default=str(LIB_RS), help="path to crates/server/src/lib.rs")
    ap.add_argument("--list-allowed", action="store_true", help="print the allow-lists and exit")
    args = ap.parse_args()

    if args.list_allowed:
        print("read-only / executor-delegating write-lock holders:")
        for name, reason in sorted(EXECUTOR_LOCK_ALLOWED.items()):
            print(f"  {name}: {reason}")
        for name, reason in sorted(READ_ONLY_ALLOWED.items()):
            print(f"  {name}: {reason}")
        print("library writers:")
        for name, reason in sorted(LIBRARY_ALLOWED.items()):
            print(f"  {name}: {reason}")
        return 0

    src = Path(args.lib).read_text(encoding="utf-8")
    bodies = handler_bodies(src)

    write_offenders: list[str] = []
    save_offenders: list[str] = []
    library_offenders: list[str] = []

    for name, body in bodies.items():
        if "project.write()" in body and name not in DIRECT_WRITE_ALLOWED:
            write_offenders.append(name)
        if ".save(" in body and name not in SAVE_ALLOWED:
            save_offenders.append(name)
        if "LIBRARY.write()" in body and name not in LIBRARY_ALLOWED:
            library_offenders.append(name)

    total = sum(len(x) for x in (write_offenders, save_offenders, library_offenders))
    print(f"scanned {len(bodies)} handlers in {Path(args.lib).name}")
    for label, names in (
        ("direct project.write()", write_offenders),
        ("direct Project::save()", save_offenders),
        ("direct LIBRARY.write()", library_offenders),
    ):
        if names:
            print(f"  {label}: {len(names)}")
            for n in names:
                print(f"    - {n}")
        else:
            print(f"  {label}: 0")

    if total:
        print(
            "\nFAIL: non-allowlisted write paths found. Route them through "
            "mutation::apply (see docs/architecture/WRITE_PATHS.md) or justify "
            "them in the allow-lists in this script."
        )
        return 1
    print("\nOK: single write path enforced.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
