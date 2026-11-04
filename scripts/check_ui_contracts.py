#!/usr/bin/env python3
"""Contract checks for the OmniM frontend documentation set.

Default mode checks structure (requirement IDs, keymap conflicts, unknown error
codes). ``--strict`` additionally fails when contract debt is above the
thresholds recorded in docs/status/FRONTEND_BASELINE.md.

Usage:
    python3 scripts/check_ui_contracts.py
    python3 scripts/check_ui_contracts.py --strict
    python3 scripts/check_ui_contracts.py --json
    python3 scripts/check_ui_contracts.py --only ids,keymap,errors

Checks
  ids      every UI-* ID in the register is unique, appears in the PRD and in
           a verification column, and area counts match the declared total
  tokens   hardcoded colours in user-facing styles (strict only)
  zindex   z-index values outside the DESIGN_SYSTEM §7 scale
  emoji    emoji used as icons in tsx (strict only)
  dialogs  window.alert/confirm/prompt call sites (strict only)
  strings  CJK literals outside the i18n catalogue (strict only)
  keymap   duplicate shortcut within one scope in KEYBOARD_MAP.md
  errors   OMNI-* codes used in source but absent from ERROR_CODES.md

Exit code 1 when any selected check fails.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
WEB = ROOT / "web" / "src"
DOCS = ROOT / "docs"
REGISTER = DOCS / "product" / "REQUIREMENTS_REGISTER.md"
PRD = DOCS / "product" / "PRD_FRONTEND.md"
ERROR_CODES = DOCS / "reference" / "ERROR_CODES.md"
KEYMAP = DOCS / "reference" / "KEYBOARD_MAP.md"

ID_RE = re.compile(r"\bUI-(SHL|DS|INT|FBK|DAT|A11Y|I18N|PRF|CMP|SEC|TLM)-(\d{2})\b")
CODE_RE = re.compile(r"\bOMNI-(UI|API|NET|IO|GEO|CAT|IFC|AUTH|SYS)-(\d{4})\b")
HEX_RE = re.compile(r"#[0-9a-fA-F]{3,8}\b")
NAMED_COLOR_RE = re.compile(
    r":\s*(?:white|black|red|green|blue|gray|grey|silver|maroon|navy|teal|olive|purple|orange|yellow)\s*(?:!important\s*)?;"
)
EMOJI_RE = re.compile(
    "[\U0001F300-\U0001FAFF\U00002600-\U000027BF\U0001F000-\U0001F2FF"
    "\U000025A0-\U000025FF\U00002B00-\U00002BFF]"
)
CJK_RE = re.compile(r"[\u4e00-\u9fff]")
DIALOG_RE = re.compile(r"window\.(alert|confirm|prompt)\s*\(")

AREAS = ["SHL", "DS", "INT", "FBK", "DAT", "A11Y", "I18N", "PRF", "CMP", "SEC", "TLM"]
STATUS = {"✅", "🔀", "📅", "💭"}
GATES = {"G0", "G1", "G2", "G3", "—"}

# Measured baseline on 2026-11-02. ``--baseline`` fails on regression above
# these values; ``--strict`` requires zero debt (the G2 gate).
DEBT_BASELINE = {
    "colors": 0,
    "font_size_raw": 0,
    "global_keydown": 2,
    "colors_func": 0,
    "emoji": 0,
    "dialogs": 0,
    "cjk": 899,
    "zindex": 0,
}


def rel(path: Path) -> str:
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def iter_files(suffixes: tuple[str, ...]) -> list[Path]:
    if not WEB.exists():
        return []
    out = []
    for suffix in suffixes:
        out.extend(WEB.rglob(f"*{suffix}"))
    return sorted(p for p in out if p.is_file())


def register_rows() -> list[list[str]]:
    if not REGISTER.exists():
        return []
    rows = []
    for line in REGISTER.read_text(encoding="utf-8").splitlines():
        if not line.strip().startswith("| UI-"):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) >= 6:
            rows.append(cells)
    return rows


def check_ids() -> list[str]:
    failures: list[str] = []
    rows = register_rows()
    if not rows:
        return [f"{rel(REGISTER)}: no requirement rows found"]

    seen: dict[str, int] = {}
    per_area: dict[str, int] = {a: 0 for a in AREAS}
    prd = PRD.read_text(encoding="utf-8") if PRD.exists() else ""

    for cells in rows:
        req_id = cells[0]
        match = ID_RE.fullmatch(req_id)
        if not match:
            failures.append(f"{rel(REGISTER)}: malformed id {req_id!r}")
            continue
        area = match.group(1)
        per_area[area] = per_area.get(area, 0) + 1
        seen[req_id] = seen.get(req_id, 0) + 1
        if cells[3] in ("", "-", "—"):
            failures.append(f"{req_id}: empty verification column")
        if cells[4] not in GATES:
            failures.append(f"{req_id}: invalid gate {cells[4]!r}")
        if cells[5] not in STATUS:
            failures.append(f"{req_id}: invalid status {cells[5]!r}")
        if req_id not in prd:
            failures.append(f"{req_id}: missing from PRD_FRONTEND")

    for req_id, count in seen.items():
        if count > 1:
            failures.append(f"{req_id}: appears {count} times in the register")

    total = len(rows)
    if total != 96:
        failures.append(f"register row count is {total}, contract declares 96")
    return failures


FUNC_COLOR_RE = re.compile(r"\b(?:rgba?|hsla?)\s*\(")
Z_INDEX_RE = re.compile(r"z-index\s*:\s*([^;]+);")
Z_INDEX_BANDS = ((0, 0), (10, 19), (20, 29), (30, 39), (40, 49), (60, 69), (90, 99))


def in_z_band(value: int) -> bool:
    return any(low <= value <= high for low, high in Z_INDEX_BANDS)


def check_zindex() -> tuple[list[str], dict[str, int]]:
    """DESIGN_SYSTEM §7: every z-index is a --z-* token within the scale."""
    failures: list[str] = []
    for path in iter_files((".css",)):
        for raw in Z_INDEX_RE.findall(path.read_text(encoding="utf-8", errors="ignore")):
            value = raw.strip()
            if value.startswith("var(--z-"):
                continue
            if value in ("auto", "inherit", "initial", "unset", "revert"):
                continue
            if path == TOKEN_FILE:
                try:
                    numeric = int(value)
                except ValueError:
                    failures.append(f"{rel(path)}: z-index {value!r} is not a number")
                    continue
                if not in_z_band(numeric):
                    failures.append(f"{rel(path)}: z-index {numeric} is outside the §7 scale")
                continue
            failures.append(f"{rel(path)}: z-index {value!r} must use a --z-* token")
    for path in iter_files((".tsx",)):
        text = path.read_text(encoding="utf-8", errors="ignore")
        for match in re.finditer(r"zIndex\s*:\s*(\d+)", text):
            failures.append(f"{rel(path)}: inline zIndex {match.group(1)} must use a --z-* token")
    return failures, {"zindex": len(failures)}


TOKEN_FILE = WEB / "globals.css"


def check_font_size() -> tuple[list[str], dict[str, int]]:
    """DS-06 (DESIGN_SYSTEM §5): single typography scale — `styles.css` may
    only set `font-size` via `var(--fs-*)` tokens (raw px is a miss)."""
    import re
    css = (Path(__file__).resolve().parent.parent / "web/src/styles.css").read_text(encoding="utf-8", errors="ignore")
    raw = [m.group(0) for m in re.finditer(r"font-size:\s*[^;]+;", css) if "var(--fs-" not in m.group(0)]
    return [], {"font_size_raw": len(raw), "_font_detail": raw[:8]}


def check_tokens() -> tuple[list[str], dict[str, int]]:
    """Hardcoded colours outside the token file (hex + rgb/hsl)."""
    hex_total = 0
    func_total = 0
    for path in iter_files((".css",)):
        # DESIGN_SYSTEM §2.4: literals are allowed only in the token source.
        if "tokens" in path.name or path == TOKEN_FILE:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        hex_total += len(HEX_RE.findall(text)) + len(NAMED_COLOR_RE.findall(text))
        func_total += len(FUNC_COLOR_RE.findall(text))
    return [], {"colors": hex_total, "colors_func": func_total}


def check_emoji() -> tuple[list[str], dict[str, int]]:
    total = 0
    hits = []
    for path in iter_files((".tsx",)):
        text = path.read_text(encoding="utf-8", errors="ignore")
        found = EMOJI_RE.findall(text)
        if found:
            total += len(found)
            hits.append(f"{rel(path)}: {len(found)}")
    return [], {"emoji": total, "_emoji_detail": hits}  # type: ignore[dict-item]


def check_dialogs() -> tuple[list[str], dict[str, int]]:
    total = 0
    detail = []
    for path in iter_files((".tsx", ".ts")):
        text = path.read_text(encoding="utf-8", errors="ignore")
        n = len(DIALOG_RE.findall(text))
        if n:
            total += n
            detail.append(f"{rel(path)}: {n}")
    return [], {"dialogs": total, "_dialogs_detail": detail}  # type: ignore[dict-item]


# Global keyboard-listener allow-list (INTERACTION §5.1, INT-03): the
# workspace registry consumer (dispatch in App) is the only window-level
# registration. React's scoped `onKeyDown` props don't count.
GLOBAL_KEYDOWN_ALLOWED = {
    # INTERACTION §5.1: the workspace dispatch reads the single registry in
    # `commands.ts` — only allowed general-purpose window listener.
    "src/app/App.tsx": "workspace command dispatch (single registry consumer)",
    # Dialog contract: the focus trap lives only while a dialog is open.
    "src/app/components/useDialogFocus.ts": "dialog focus trap (capture; removed with the dialog)",
}


def check_global_keydown() -> tuple[list[str], dict[str, int]]:
    import re
    offenders = []
    for path in iter_files((".tsx", ".ts")):
        rel_key = rel(path)
        if rel_key.startswith("web/"):
            rel_key = rel_key[len("web/"):]
        else:
            rel_key = rel_key[len("web/"):] if "/web/" in rel_key else rel_key
        if rel_key.startswith("test") or "__tests__" in rel_key or rel_key.startswith("node_modules"):
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        n = len(re.findall(r"window[.]addEventListener\(.(?:keydown).", text))
        if n and rel_key not in GLOBAL_KEYDOWN_ALLOWED:
            offenders.append(rel_key)
    detail = ", ".join(offenders)
    result: list[str] = []
    if offenders:
        result.append(
            f"{len(offenders)} file(s) register window key listeners outside the allow-list ({detail})"
        )
    return result, {"global_keydown": len(offenders), "_global_keydown_detail": detail}


def startswith_i18n(path: Path) -> bool:
    """True for the i18n catalogue (`i18n/` dir) and the kernel (`i18n.ts`)."""
    return any(part == "i18n" or part.startswith("i18n.") for part in path.parts)


def check_strings() -> tuple[list[str], dict[str, int]]:
    """CJK runs (not characters) in ts/tsx outside the i18n catalogue."""
    total = 0
    files = 0
    run_re = re.compile(r"[\u4e00-\u9fff]+")
    for path in iter_files((".tsx", ".ts")):
        # i18n catalogue: directory `i18n/` plus the kernel file `i18n.ts`.
        if startswith_i18n(path):
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        n = len(run_re.findall(text))
        if n:
            total += n
            files += 1
    return [], {"cjk": total, "_cjk_files": files}  # type: ignore[dict-item]


def check_keymap() -> list[str]:
    failures: list[str] = []
    if not KEYMAP.exists():
        return [f"{rel(KEYMAP)}: missing"]

    rows: list[tuple[str, str, str]] = []
    section = "unknown"
    for line in KEYMAP.read_text(encoding="utf-8").splitlines():
        heading = re.match(r"^##\s+\d+\.\s+(.*)$", line.strip())
        if heading:
            section = heading.group(1)
            continue
        if not line.strip().startswith("| `"):
            continue
        cells = [c.strip() for c in line.strip().strip("|").split("|")]
        if len(cells) < 3:
            continue
        keys = cells[0].strip("`")
        command = cells[1]
        rows.append((keys, command, section))

    by_scope: dict[str, dict[str, str]] = {}
    commands: dict[str, str] = {}
    for keys, command, scope in rows:
        scope_key = scope.lower()
        bucket = by_scope.setdefault(scope_key, {})
        if keys in bucket:
            failures.append(f"keymap: {keys} bound twice in scope {scope}")
        bucket[keys] = command
        if command.lower() not in ("action", "命令", "动作") and command in commands:
            failures.append(f"keymap: command {command!r} bound more than once")
        commands[command] = keys
    if len(rows) < 20:
        failures.append(f"keymap: only {len(rows)} binding rows parsed")
    return failures


def check_errors() -> list[str]:
    failures: list[str] = []
    catalogue = ERROR_CODES.read_text(encoding="utf-8") if ERROR_CODES.exists() else ""
    if not catalogue:
        return [f"{rel(ERROR_CODES)}: missing"]

    paths = iter_files((".tsx", ".ts")) + sorted((ROOT / "crates").rglob("*.rs"))
    for path in paths:
        text = path.read_text(encoding="utf-8", errors="ignore")
        for match in CODE_RE.finditer(text):
            code = match.group(0)
            if code not in catalogue:
                failures.append(f"{rel(path)}: {code} not in ERROR_CODES.md")
    return failures


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--strict", action="store_true", help="zero-debt gate: fail on any debt")
    parser.add_argument("--baseline", action="store_true", help="fail when debt exceeds the 2026-10-29 baseline")
    parser.add_argument("--json", action="store_true", help="machine-readable output")
    parser.add_argument("--only", default="", help="comma list: ids,tokens,zindex,emoji,dialogs,strings,keymap,errors")
    parser.add_argument("--ids", action="store_true", help="alias for --only ids")
    parser.add_argument("--tokens", action="store_true", help="alias for --only tokens")
    parser.add_argument("--zindex", action="store_true", help="alias for --only zindex")
    parser.add_argument("--emoji", action="store_true", help="alias for --only emoji")
    parser.add_argument("--dialogs", action="store_true", help="alias for --only dialogs")
    parser.add_argument("--strings", action="store_true", help="alias for --only strings")
    parser.add_argument("--keymap", action="store_true", help="alias for --only keymap")
    parser.add_argument("--errors", action="store_true", help="alias for --only errors")
    args = parser.parse_args()

    wanted = {p.strip() for p in args.only.split(",") if p.strip()}
    for flag in ("ids", "tokens", "zindex", "emoji", "dialogs", "strings", "keymap", "errors"):
        if getattr(args, flag):
            wanted.add(flag)
    failures: list[str] = []
    metrics: dict[str, object] = {}

    def run(name: str, check) -> None:
        if wanted and name not in wanted:
            return
        result = check()
        if isinstance(result, tuple):
            found, data = result
            failures.extend(found)
            metrics.update(data)
        else:
            failures.extend(result)

    run("ids", check_ids)
    run("keymap", check_keymap)
    run("errors", check_errors)
    run("tokens", check_tokens)
    run("font_size", check_font_size)
    run("global_keydown", check_global_keydown)
    run("zindex", check_zindex)
    run("emoji", check_emoji)
    run("dialogs", check_dialogs)
    run("strings", check_strings)

    debt_failures: list[str] = []
    for key, value in metrics.items():
        if key.startswith("_") or not isinstance(value, int):
            continue
        if args.strict and value > 0:
            debt_failures.append(f"debt {key}={value} (strict requires 0)")
        elif args.baseline and value > DEBT_BASELINE.get(key, 0):
            debt_failures.append(
                f"debt {key}={value} exceeds baseline {DEBT_BASELINE.get(key, 0)}"
            )

    all_failures = failures + debt_failures
    if args.json:
        print(json.dumps({"ok": not all_failures, "failures": all_failures,
                          "metrics": {k: v for k, v in metrics.items() if not k.startswith("_")}},
                         ensure_ascii=False, indent=2))
        return 1 if all_failures else 0

    print("UI contract checks")
    for key in ("colors", "colors_func", "zindex", "emoji", "dialogs", "cjk"):
        if key in metrics:
            print(f"  {key:11s}: {metrics[key]}")
    print(f"  failures: {len(all_failures)}")
    if all_failures:
        print()
        for failure in all_failures[:40]:
            print(f"  - {failure}")
        if len(all_failures) > 40:
            print(f"  ... and {len(all_failures) - 40} more")
        return 1
    print("\nOK: contract checks passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
