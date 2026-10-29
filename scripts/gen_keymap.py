#!/usr/bin/env python3
"""Validate the keyboard map against itself and across locales.

The command registry is the eventual source of truth. Until it lands, this
script keeps the two language versions of KEYBOARD_MAP in sync and catches
duplicate bindings inside a section.

Usage:
    python3 scripts/gen_keymap.py --check
    python3 scripts/gen_keymap.py --emit            # JSON snapshot
    python3 scripts/gen_keymap.py --emit --write    # write keymap.json

Exit code 1 when a check fails.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DOCS = ROOT / "docs" / "reference"
EN = DOCS / "KEYBOARD_MAP.md"
ZH = DOCS / "KEYBOARD_MAP.zh-CN.md"
JSON_OUT = DOCS / "keymap.json"

HEADING_RE = re.compile(r"^##\s+(\d+)\.\s+(.*)$")
ROW_RE = re.compile(r"^\|\s*`([^`]+)`\s*\|\s*([^|]+?)\s*\|(.*)$")
VALID_STATUS = {"✅", "🔀", "📅", "💭"}


def parse(path: Path) -> tuple[list[dict[str, str]], list[str]]:
    failures: list[str] = []
    rows: list[dict[str, str]] = []
    section = "unknown"
    if not path.exists():
        return rows, [f"missing file: {path}"]

    for line in path.read_text(encoding="utf-8").splitlines():
        heading = HEADING_RE.match(line.strip())
        if heading:
            section = heading.group(1)
            continue
        match = ROW_RE.match(line.strip())
        if not match:
            continue
        keys, command, rest = match.groups()
        cells = [c.strip() for c in rest.strip().strip("|").split("|")]
        status = cells[-1] if cells else ""
        if status and status not in VALID_STATUS and status not in {"状态"}:
            failures.append(f"{path.name}:{section}: invalid status {status!r} for {keys}")
        rows.append({
            "section": section,
            "keys": keys.strip(),
            "command": command.strip(),
            "scope": cells[0] if len(cells) >= 2 else "",
            "status": status,
        })

    seen: dict[tuple[str, str], str] = {}
    for row in rows:
        key = (row["section"], row["keys"])
        if key in seen:
            failures.append(
                f"{path.name}:{row['section']}: duplicate binding {row['keys']!r} "
                f"({seen[key]!r} and {row['command']!r})"
            )
        seen[key] = row["command"]
    return rows, failures


def check() -> int:
    en, en_fail = parse(EN)
    zh, zh_fail = parse(ZH)
    failures = en_fail + zh_fail

    if len(en) != len(zh):
        failures.append(f"row count mismatch: en={len(en)} zh={len(zh)}")
    else:
        for i, (a, b) in enumerate(zip(en, zh), 1):
            if a["section"] != b["section"] or a["keys"] != b["keys"]:
                failures.append(
                    f"row {i} divergence: en=({a['section']},{a['keys']}) "
                    f"zh=({b['section']},{b['keys']})"
                )

    if len(en) < 20:
        failures.append(f"only {len(en)} bindings parsed; expected at least 20")

    if failures:
        print(f"KEYBOARD_MAP check FAILED ({len(failures)} issue(s)):")
        for failure in failures:
            print(f"  - {failure}")
        return 1
    print(f"OK: {len(en)} bindings, en/zh aligned, no duplicate scope bindings.")
    return 0


def emit(write: bool) -> int:
    en, _ = parse(EN)
    payload = {"version": 1, "locale": "en", "bindings": en}
    text = json.dumps(payload, ensure_ascii=False, indent=2) + "\n"
    if write:
        JSON_OUT.write_text(text, encoding="utf-8")
        print(f"wrote {JSON_OUT.relative_to(ROOT)} ({len(en)} bindings)")
    else:
        print(text, end="")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="validate the map and locale parity")
    parser.add_argument("--emit", action="store_true", help="print a JSON snapshot")
    parser.add_argument("--write", action="store_true", help="with --emit, write keymap.json")
    args = parser.parse_args()

    if not any((args.check, args.emit)):
        args.check = True
    if args.check and check() != 0:
        return 1
    if args.emit:
        return emit(args.write)
    return 0


if __name__ == "__main__":
    sys.exit(main())
