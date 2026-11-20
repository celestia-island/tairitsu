#!/usr/bin/env python3
"""Gate: every host interface `tairitsu-web` references is either wired or allowed.

`tairitsu-web` compiles against the composed `browser-full` world, which declares
hundreds of interfaces; the browser host implementation in
`packages/browser-glue/src/runtime/registry.ts` wires a much smaller, hand-curated
set. Nothing used to connect the two, so an interface the guest imports could
simply have no host implementation — the component then failed at instantiation
(or trapped on first call) with no gate anywhere in CI.

This script derives the three sets mechanically:

* **referenced** — interfaces named by `bindings::tairitsu_browser::full::<iface>`
  anywhere in `packages/web/src/**/*.rs`;
* **wired** — the `"@tairitsu-glue/<iface>"` keys of `INTERFACES` in
  `packages/browser-glue/src/runtime/registry.ts`;
* **allowed** — the explicit allowance list in
  `packages/browser-glue/wit-surface-allow.tsv` (one `iface<TAB>reason` per line).

It fails when a referenced interface is neither wired nor allowed, when an
allowance no longer matches a reference (stale entry), or when an allowance names
an interface that is already wired (the allowance has served its purpose).

Usage:
    python3 scripts/check_wit_surface.py --check
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
WEB_SRC = REPO_ROOT / "packages" / "web" / "src"
REGISTRY = REPO_ROOT / "packages" / "browser-glue" / "src" / "runtime" / "registry.ts"
ALLOW = REPO_ROOT / "packages" / "browser-glue" / "wit-surface-allow.tsv"
WORLD = REPO_ROOT / "packages" / "browser-worlds" / "wit" / "composed" / "browser-full.wit"

REFERENCE_RE = re.compile(r"bindings::tairitsu_browser::full::([a-z0-9_]+)")
WIRED_RE = re.compile(r'"@tairitsu-glue/([a-z0-9-]+)"\s*:')
WORLD_IMPORT_RE = re.compile(r"^\s*import\s+([a-z0-9-]+)\s*;", re.MULTILINE)


def kebab(name: str) -> str:
    return name.replace("_", "-")


def referenced() -> set[str]:
    found: set[str] = set()
    for path in WEB_SRC.rglob("*.rs"):
        text = path.read_text(encoding="utf-8", errors="replace")
        found.update(kebab(name) for name in REFERENCE_RE.findall(text))
    return found


def wired() -> set[str]:
    text = REGISTRY.read_text(encoding="utf-8", errors="replace")
    block = text.split("export const INTERFACES", 1)
    if len(block) != 2:
        raise SystemExit("error: could not locate `export const INTERFACES` in registry.ts")
    return set(WIRED_RE.findall(block[1]))


def world_imports() -> set[str]:
    text = WORLD.read_text(encoding="utf-8", errors="replace")
    return set(WORLD_IMPORT_RE.findall(text))


def allowed() -> dict[str, str]:
    entries: dict[str, str] = {}
    if not ALLOW.exists():
        return entries
    for raw in ALLOW.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        parts = raw.split("\t")
        if len(parts) < 2 or not parts[1].strip():
            raise SystemExit(f"error: allowance line lacks a reason: {raw!r}")
        entries[parts[0].strip()] = parts[1].strip()
    return entries


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="enforce (the only mode)")
    parser.parse_args()

    ref = referenced()
    wrd = wired()
    allow = allowed()
    world = world_imports()

    problems: list[str] = []

    for name in sorted(ref - wrd - set(allow)):
        problems.append(
            f"referenced but neither wired nor allowed: {name} "
            f"(add a host implementation in browser-glue or an entry in "
            f"{ALLOW.relative_to(REPO_ROOT)})"
        )
    for name in sorted(set(allow) - ref):
        problems.append(
            f"stale allowance: {name} is allowed but no longer referenced by tairitsu-web"
        )
    for name in sorted(set(allow) & wrd):
        problems.append(
            f"needless allowance: {name} is already wired by browser-glue"
        )
    for name in sorted(ref - world):
        problems.append(
            f"referenced interface is not declared by the world: {name} "
            f"(check {WORLD.relative_to(REPO_ROOT)})"
        )

    if problems:
        print("WIT host-surface gate failed:", file=sys.stderr)
        for problem in problems:
            print(f"  - {problem}", file=sys.stderr)
        return 1

    print(
        f"WIT host surface OK: {len(ref)} referenced, {len(wrd)} wired, "
        f"{len(allow)} allowed"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())