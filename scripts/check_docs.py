#!/usr/bin/env python3
"""Lightweight documentation checks for the OmniM docs tree.

Checks (stdlib only):
  (a) broken relative Markdown links (anchors-only and http(s)/mailto ignored)
  (b) code-fence parity per file (an even number of ``` lines)
  (c) duplicate numbered headings (## N. / ### N.M) within a file
  (d) English/Chinese heading-count and heading-number parity for X.md / X.zh-CN.md

Exit code 1 if any check fails. The known glossary heading-count mismatch is
skipped by default (use ``--strict-parity`` to include it); other known
mismatches can be excluded with ``--skip-parity 'docs/reference/API*'``
(repeatable).
"""

from __future__ import annotations

import argparse
import fnmatch
import re
import sys
from pathlib import Path
from urllib.parse import unquote

HEADING_RE = re.compile(r"^(#{1,6})\s+(.*)$")
NUMBER_RE = re.compile(r"^(#{2,6})\s+(\d+(?:\.\d+)*)\.?(?=\s|$)")
DUP_RE = re.compile(r"^(#{2,3})\s+(\d+(?:\.\d+)*)\.?(?=\s|$)")
FENCE_RE = re.compile(r"^\s*```")
LINK_RE = re.compile(r"!?\[[^\]]*\]\(([^)\n]+)\)")
IGNORED_PREFIXES = ("#", "http://", "https://", "mailto:", "tel:", "data:", "javascript:", "file:")
# Known, pre-existing mismatch: the zh glossary has one extra UI-constraint
# subsection that the English glossary does not. Do not hide new mismatches.
KNOWN_PARITY_SKIPS = ("docs/reference/GLOSSARY*",)


def strip_code(text: str) -> list[str]:
    """Blank out fenced blocks and inline code while preserving line numbers."""
    result: list[str] = []
    in_fence = False
    for line in text.splitlines():
        stripped = line.lstrip()
        if stripped.startswith("```") or stripped.startswith("~~~"):
            in_fence = not in_fence
            result.append("")
            continue
        if in_fence:
            result.append("")
        else:
            result.append(re.sub(r"`[^`]*`", "", line))
    return result


def link_target(raw: str) -> str | None:
    raw = raw.strip()
    if not raw:
        return None
    if raw.startswith("<") and ">" in raw:
        return raw[1:raw.index(">")].strip()
    # Drop an optional Markdown title: [text](path "title")
    return raw.split(None, 1)[0]


def check_links(root: Path, path: Path, lines: list[str], failures: list[str]) -> int:
    count = 0
    for lineno, line in enumerate(lines, 1):
        for match in LINK_RE.finditer(line):
            target = link_target(match.group(1))
            if not target or target.startswith(IGNORED_PREFIXES):
                continue
            if target.startswith("/"):
                continue  # site-absolute, not a relative file link
            target = target.split("#", 1)[0].split("?", 1)[0]
            if not target:
                continue
            target = unquote(target)
            resolved = (path.parent / target).resolve()
            if not resolved.exists():
                failures.append(f"{path.relative_to(root)}:{lineno}: broken link -> {match.group(1).strip()}")
                count += 1
    return count


def check_fences(root: Path, path: Path, text: str, failures: list[str]) -> int:
    count = sum(1 for line in text.splitlines() if FENCE_RE.match(line))
    if count % 2:
        failures.append(f"{path.relative_to(root)}: odd number of ``` fence lines ({count})")
        return 1
    return 0


def check_duplicate_headings(root: Path, path: Path, text: str, failures: list[str]) -> int:
    seen: dict[tuple[str, str], int] = {}
    count = 0
    for lineno, line in enumerate(text.splitlines(), 1):
        match = DUP_RE.match(line)
        if not match:
            continue
        key = (match.group(1), match.group(2))
        if key in seen:
            failures.append(
                f"{path.relative_to(root)}:{lineno}: duplicate heading {match.group(1)} {match.group(2)} "
                f"(first at line {seen[key]})"
            )
            count += 1
        else:
            seen[key] = lineno
    return count


def headings(text: str) -> list[tuple[int, str, str | None]]:
    out: list[tuple[int, str, str | None]] = []
    for line in text.splitlines():
        match = HEADING_RE.match(line)
        if match:
            number = NUMBER_RE.match(line)
            out.append((len(match.group(1)), match.group(2).strip(), number.group(2) if number else None))
    return out


def skipped(rel: str, name: str, patterns: list[str]) -> bool:
    return any(fnmatch.fnmatch(rel, pat) or fnmatch.fnmatch(name, pat) for pat in patterns)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--docs", default="docs", help="docs directory (default: docs)")
    parser.add_argument(
        "--skip-parity",
        action="append",
        default=[],
        metavar="GLOB",
        help="skip en/zh parity for matching paths or filenames (repeatable; comma-separated allowed)",
    )
    parser.add_argument(
        "--strict-parity",
        action="store_true",
        help="also check the known-mismatched glossary pair (no built-in skips)",
    )
    args = parser.parse_args()

    root = Path.cwd()
    docs = (root / args.docs).resolve()
    if not docs.is_dir():
        print(f"error: {docs} is not a directory", file=sys.stderr)
        return 2

    patterns = [] if args.strict_parity else list(KNOWN_PARITY_SKIPS)
    patterns += [part for value in args.skip_parity for part in value.split(",") if part]
    files = sorted(docs.rglob("*.md"))
    failures: list[str] = []
    broken = fences = duplicates = 0

    for path in files:
        text = path.read_text(encoding="utf-8")
        broken += check_links(root, path, strip_code(text), failures)
        fences += check_fences(root, path, text, failures)
        duplicates += check_duplicate_headings(root, path, text, failures)

    parity_failed = 0
    parity_skipped = 0
    pairs = 0
    for zh in files:
        if not zh.name.endswith(".zh-CN.md"):
            continue
        en = zh.with_name(zh.name[: -len(".zh-CN.md")] + ".md")
        if not en.exists():
            continue
        rel_en = en.relative_to(root).as_posix()
        rel_zh = zh.relative_to(root).as_posix()
        if skipped(rel_en, en.name, patterns) or skipped(rel_zh, zh.name, patterns):
            parity_skipped += 1
            continue
        pairs += 1
        en_heads = headings(en.read_text(encoding="utf-8"))
        zh_heads = headings(zh.read_text(encoding="utf-8"))
        if len(en_heads) != len(zh_heads):
            failures.append(
                f"{rel_en} / {zh.name}: heading count mismatch ({len(en_heads)} vs {len(zh_heads)})"
            )
            parity_failed += 1
            continue
        en_nums = [number for _, _, number in en_heads if number]
        zh_nums = [number for _, _, number in zh_heads if number]
        if en_nums != zh_nums:
            failures.append(
                f"{rel_en} / {zh.name}: heading numbering mismatch "
                f"({len(en_nums)} vs {len(zh_nums)} numbered headings)"
            )
            for index, (a, b) in enumerate(zip(en_nums, zh_nums), 1):
                if a != b:
                    failures.append(f"  first difference at numbered heading {index}: en={a} zh={b}")
                    break
            parity_failed += 1

    print(f"docs checked: {len(files)} files")
    print(f"  broken relative links : {broken}")
    print(f"  fence-parity failures : {fences}")
    print(f"  duplicate numbered H  : {duplicates}")
    print(f"  en/zh pairs checked   : {pairs} (skipped: {parity_skipped})")
    if failures:
        print(f"\nFAIL ({len(failures)} issue(s)):")
        for failure in failures:
            print(f"  - {failure}")
        return 1
    print("\nOK: docs checks passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
