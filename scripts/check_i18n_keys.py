#!/usr/bin/env python3
"""I18N key audit (I18N_L10N §7 / UI-I18N-06).

Hard gates (exit 1):
  * a referenced key that is not in the dictionary (missing keys fail the build),
  * zh-CN / en-US key-set mismatch (zh-CN.ts is the single truth).

Reports only (per contract: "每个版本报告"):
  * dictionary keys never referenced in `web/src` (candidates for removal one
    minor version later),
  * duplicate zh values under different keys (allowed only when the semantics
    differ — a human call, so it stays a report),
  * keys that do not follow the documented `<area>.<component>.<element>`
    naming rule (the legacy prefixes are a recorded deviation).
"""
from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
WEB = ROOT / "web" / "src"
ZH = WEB / "shared" / "i18n" / "zh-CN.ts"
EN = WEB / "shared" / "i18n" / "en-US.ts"

KEY_RE = re.compile(r"'([a-zA-Z0-9_.]+)':")
REF_RE = re.compile(r"\b(?:t|tPlural|hasMessage)\('([a-zA-Z0-9_.]+)'")
TEMPLATE_RE = re.compile(r"`([a-zA-Z0-9_.]+)\.\$\{")
LITERAL_RE = re.compile(r"'([a-z0-9_]+(?:\.[a-z0-9_]+)+)'")

# Non-message strings that share the key shape but are not i18n keys.
NON_KEY_LITERALS = {
    "omnim.nav",  # localStorage key
}

# Documented legacy prefixes that predate the naming rule; they stay report
# lines, not failures, until a deliberate rename lands.
LEGACY_PREFIXES = (
    "panel.", "assay.", "assy.", "layer.", "quick.", "domain.", "left.", "nav.",
    "library.", "mat.", "grid.", "feature.", "avail.", "catp.", "toolbar.",
    "viewer.", "pick.", "i18n.", "crumb.", "storey.",
)

ALLOWED_AREAS = ("common", "shell", "tree", "props", "create", "catalog",
                 "series", "lib", "sketch", "console", "ai", "error", "a11y")


PLURAL_CATEGORIES = ("zero", "one", "two", "few", "many", "other")


def read_keys(path: Path) -> list[str]:
    # Keys may share a line (several per line), so match occurrences anywhere.
    seen: list[str] = []
    for m in KEY_RE.finditer(path.read_text(encoding="utf-8")):
        seen.append(m.group(1))
    return seen


def is_plural_key(key: str) -> bool:
    return key.rsplit(".", 1)[-1] in PLURAL_CATEGORIES


def plural_base(key: str) -> str | None:
    cat = key.rsplit(".", 1)
    return cat[0] if is_plural_key(key) and len(cat) == 2 else None


def zh_values(path: Path) -> dict[str, str]:
    out: dict[str, str] = {}
    for m in re.finditer(r"^  '([a-zA-Z0-9_.]+)': '((?:[^'\\]|\\.)*)'", path.read_text(encoding="utf-8"), re.M):
        out[m.group(1)] = m.group(2)
    return out


def collect_refs() -> tuple[set[str], set[str]]:
    """Return (explicit t()/hasMessage refs, key-shaped literals incl. templates).

    Kernel unit tests are excluded from the explicit set: they deliberately
    probe missing keys and en-only plural categories.
    """
    explicit: set[str] = set()
    literals: set[str] = set()
    for path in WEB.rglob("*"):
        if path.suffix not in (".ts", ".tsx") or "i18n" in path.parts:
            continue
        if path.name.endswith(".test.ts") or path.name.endswith(".test.tsx"):
            literals.update(REF_RE.findall(path.read_text(encoding="utf-8", errors="replace")))
            continue
        text = path.read_text(encoding="utf-8", errors="replace")
        explicit.update(REF_RE.findall(text))
        for prefix in TEMPLATE_RE.findall(text):
            literals.add(prefix)
        for lit in LITERAL_RE.findall(text):
            if lit not in NON_KEY_LITERALS:
                literals.add(lit)
    return explicit, literals


def main() -> int:
    zh = read_keys(ZH)
    en = read_keys(EN)
    zh_set, en_set = set(zh), set(en)
    explicit, literals = collect_refs()

    failures: list[str] = []
    reports: list[str] = []

    # 1. Missing keys: any explicit reference must exist in the dictionary.
    zh_bases = {plural_base(k) for k in zh_set if plural_base(k)}
    missing = sorted(
        key for key in explicit - zh_set
        if key not in zh_bases  # tPlural base refs select a <category> suffix
    )
    if missing:
        for key in missing:
            failures.append(f"missing key referenced in code: {key}")

    # 2. zh/en parity.
    zh_cats = {k for k in zh_set if is_plural_key(k)}
    en_cats = {k for k in en_set if is_plural_key(k)}
    # Plural categories legitimately differ per language (zh: other; en:
    # zero/one/other); the fallback chain covers the rest (I18N §4).
    only_zh = sorted((zh_set - zh_cats) - (en_set - en_cats))
    only_en = sorted((en_set - en_cats) - (zh_set - zh_cats))
    if only_zh or only_en:
        for key in only_zh:
            failures.append(f"key only in zh-CN: {key}")
        for key in only_en:
            failures.append(f"key only in en-US: {key}")

    # 3. Unused keys (report): referenced neither explicitly nor by a literal /
    #    dynamic-template prefix.
    def used(key: str) -> bool:
        if key in explicit:
            return True
        parts = key.split(".")
        for lit in literals:
            if key == lit or key.startswith(lit + "."):
                return True
            lit_parts = lit.split(".")
            n = min(len(parts), len(lit_parts))
            if n >= 2 and parts[:n] == lit_parts[:n] and "${" in lit:
                return True
        return False

    unused = sorted(key for key in zh_set - zh_cats if not used(key))

    # 4. Duplicate zh values (report).
    by_value: dict[str, list[str]] = {}
    for key, value in sorted(zh_values(ZH).items()):
        by_value.setdefault(value, []).append(key)
    dups = {v: ks for v, ks in by_value.items() if len(ks) > 1}

    # 5. Naming rule (report).
    off_names = sorted(
        k for k in zh_set
        if not k.startswith(ALLOWED_AREAS) and not k.startswith(LEGACY_PREFIXES)
    )

    reports.append(f"dictionary: {len(zh_set)} keys (zh), {len(en_set)} (en), {len(explicit)} static refs")
    reports.append(f"unused keys ({len(unused)}; remove one minor version after first reported):")
    reports.extend(f"  - {k}" for k in unused)
    reports.append(f"duplicate zh values ({len(dups)}; allowed only when semantics differ):")
    reports.extend(f"  - {ks} -> '{v[:40]}'" for v, ks in sorted(dups.items()))
    reports.append(f"keys outside the documented area list ({len(off_names)}):")
    reports.extend(f"  - {k}" for k in off_names)

    for line in reports:
        print(line)

    if failures:
        print("\nFAIL: i18n key audit:")
        for line in failures:
            print(f"  - {line}")
        return 1

    print("\nOK: i18n key audit (hard gates passed; reports above are informational)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
