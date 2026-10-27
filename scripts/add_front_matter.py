#!/usr/bin/env python3
"""Add YAML front matter to docs Markdown files that lack it.

Idempotent: a file whose first non-blank line is ``---`` is left untouched.
Stdlib only. Run from the repository root (or pass --docs).
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

DEFAULT_DATE = "2026-10-27"

STATUS_BY_DIR = {
    "RFC": "design",
    "architecture": "current",
    "guides": "current",
    "reference": "reference",
    "research": "research",
    "specification": "design",
    "status": "current",
}

APPLIES_TO = {
    "": "repo",
    "RFC": "n/a (proposal)",
    "architecture": "crates/, web/",
    "guides": "repo",
    "reference": "crates/server, proto",
    "research": "n/a (research)",
    "specification": "proto/, crates/model",
    "status": "repo",
}

EVIDENCE = {
    "": "verified against the docs tree 2026-10-27",
    "RFC": "n/a (proposal)",
    "architecture": "audited against crates/server/src/lib.rs 2026-10-27",
    "guides": "commands and paths verified in the repo 2026-10-27",
    "reference": "audited against crates/server/src/lib.rs route table 2026-10-27",
    "research": "n/a (background research)",
    "specification": "n/a (design contract)",
    "status": "audited against code 2026-10-27",
}

DATE_RE = re.compile(r"(20\d{2}-\d{2}-\d{2})")


def rel_dir(docs: Path, path: Path) -> str:
    try:
        rel = path.relative_to(docs)
    except ValueError:
        return ""
    return "" if len(rel.parts) == 1 else rel.parts[0]


def title_of(text: str) -> str | None:
    for line in text.splitlines():
        match = re.match(r"^#\s+(.+?)\s*$", line)
        if match:
            return match.group(1).strip()
    return None


def status_of(docs: Path, path: Path) -> str:
    directory = rel_dir(docs, path)
    if directory == "status" and (path.name.startswith("CHANGELOG") or path.name.startswith("DOC_AUDIT")):
        return "history"
    return STATUS_BY_DIR.get(directory, "current")


def date_of(text: str) -> str:
    # Prefer an explicit "last updated"/"更新" date near the top; otherwise the
    # first full date in the first 20 lines, which is usually the doc's own date.
    head = "\n".join(text.splitlines()[:20])
    for line in head.splitlines():
        if any(key in line.lower() for key in ("last updated", "更新", "snapshot date", "审计日期")):
            match = DATE_RE.search(line)
            if match:
                return match.group(1)
    match = DATE_RE.search(head)
    return match.group(1) if match else DEFAULT_DATE


def front_matter(docs: Path, path: Path, text: str) -> str:
    directory = rel_dir(docs, path)
    title = title_of(text)
    if title is None:
        raise ValueError("no H1 heading")
    title = json.dumps(title, ensure_ascii=False)
    applies = json.dumps(APPLIES_TO.get(directory, "repo"), ensure_ascii=False)
    evidence = json.dumps(EVIDENCE.get(directory, EVIDENCE[""]), ensure_ascii=False)
    return (
        "---\n"
        f"title: {title}\n"
        f"status: {status_of(docs, path)}\n"
        f"last_updated: {date_of(text)}\n"
        f"applies_to: {applies}\n"
        f"evidence: {evidence}\n"
        "---\n"
    )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--docs", default="docs", help="docs directory (default: docs)")
    parser.add_argument("--dry-run", action="store_true", help="report without writing")
    args = parser.parse_args()

    docs = Path(args.docs).resolve()
    if not docs.is_dir():
        print(f"error: {docs} is not a directory", file=sys.stderr)
        return 2

    added, skipped, errors = [], [], []
    for path in sorted(docs.rglob("*.md")):
        text = path.read_text(encoding="utf-8")
        if text.lstrip("\ufeff").lstrip().startswith("---"):
            skipped.append(path)
            continue
        try:
            block = front_matter(docs, path, text)
        except ValueError as exc:
            errors.append((path, str(exc)))
            continue
        if not text.endswith("\n"):
            text += "\n"
        if not args.dry_run:
            path.write_text(block + text, encoding="utf-8")
        added.append(path.relative_to(docs.parent))

    for path in added:
        print(f"added  {path}")
    for path, reason in errors:
        print(f"skip   {path.relative_to(docs.parent)} ({reason})")
    print(f"\nfront matter added: {len(added)}; already present: {len(skipped)}; errors: {len(errors)}")
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
