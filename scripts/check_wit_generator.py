#!/usr/bin/env python3
"""Gate: the generator's export-only contract must match the committed world.

`scripts/generate_browser_wit.py` decides which callback interfaces the world
*exports* (the guest implements them, the host calls back in). Interfaces in
`EXPORT_ONLY` additionally must NOT be imported: an import would oblige the host
to implement a callback the guest owns, and the browser host does not.

That decision lives in a Python list, so nothing stopped a later edit from
emptying it and letting the next regeneration quietly re-add the import — the
committed tree would still look fine, and every byte-level gate would stay
green, because all copies are regenerated together.

This check pins the contract from both sides: the list the generator declares,
and the world the repository actually ships (composed + monolithic).

Usage:
    python3 scripts/check_wit_generator.py --check
"""
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
SCRIPTS = REPO_ROOT / "scripts"
GENERATOR = REPO_ROOT / "scripts" / "generate_browser_wit.py"
COMPOSED_WORLD = REPO_ROOT / "packages" / "browser-worlds" / "wit" / "composed" / "browser-full.wit"
SSR_MANIFEST = REPO_ROOT / "packages" / "ssr" / "Cargo.toml"
WEB_MANIFEST = REPO_ROOT / "packages" / "web" / "Cargo.toml"

# tairitsu-ssr's build script reads the monolithic world and emits a file that
# `src/stubs.rs` includes; a packaged crate without either cannot build.
SSR_REQUIRED_INCLUDES = ("build.rs", "wit/browser-full.wit", "wit/composed/**/*")
# tairitsu-web binds `wit/composed` from its embedded copy (build.rs), so the
# packaged crate needs it too.
WEB_REQUIRED_INCLUDES = ("wit/composed/**/*", "build.rs")
MONOLITHIC_WORLD = REPO_ROOT / "packages" / "browser-worlds" / "wit" / "browser-full.wit"

# Changing this set is a contract change: regenerate the WIT, update this pin in
# the same commit, and say why in the PR. That is the point — it must not be a
# silent side effect of editing the generator.
EXPECTED_EXPORT_ONLY = {"web-socket-callbacks"}

LIST_RE = re.compile(r"^(EXPORT[A-Z_]+) = \[(.*?)\]", re.MULTILINE | re.DOTALL)
NAME_RE = re.compile(r'"([a-z0-9-]+)"')



def generator_lists() -> dict[str, set[str]]:
    text = GENERATOR.read_text(encoding="utf-8", errors="replace")
    found: dict[str, set[str]] = {}
    for match in LIST_RE.finditer(text):
        found[match.group(1)] = set(NAME_RE.findall(match.group(2)))
    missing = {"EXPORT_ONLY", "EXPORTED_CALLBACKS"} - set(found)
    if missing:
        raise SystemExit(f"error: could not parse {sorted(missing)} in {GENERATOR.name}")
    return found


def world_membership(path: Path) -> tuple[set[str], set[str]]:
    """Read a world's imports/exports through the same comment-safe parser the
    sync gate uses, so a brace inside a comment cannot truncate this scan either."""
    if str(SCRIPTS) not in sys.path:
        sys.path.insert(0, str(SCRIPTS))
    import sync_wit  # local module in the same directory

    return sync_wit._world_membership(path.read_text(encoding="utf-8", errors="replace"))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="enforce (the only mode)")
    parser.parse_args()

    lists = generator_lists()
    export_only = lists["EXPORT_ONLY"]
    exported = lists["EXPORTED_CALLBACKS"]

    problems: list[str] = []
    if export_only != EXPECTED_EXPORT_ONLY:
        problems.append(
            f"generator EXPORT_ONLY changed: {sorted(export_only)} != pinned "
            f"{sorted(EXPECTED_EXPORT_ONLY)} — regenerate the WIT and update the pin "
            "in the same commit, with a reason"
        )
    if not export_only <= exported:
        problems.append(
            "EXPORT_ONLY is not a subset of EXPORTED_CALLBACKS: "
            f"{sorted(export_only - exported)} would never be exported at all"
        )

    for label, manifest_path, required_items in (
        ("packages/ssr/Cargo.toml", SSR_MANIFEST, SSR_REQUIRED_INCLUDES),
        ("packages/web/Cargo.toml", WEB_MANIFEST, WEB_REQUIRED_INCLUDES),
    ):
        manifest = manifest_path.read_text(encoding="utf-8", errors="replace")
        for required in required_items:
            if f'"{required}"' not in manifest:
                problems.append(
                    f"{label} include list is missing {required} "
                    "(the packaged crate would not build)"
                )

    for label, path in (("composed", COMPOSED_WORLD), ("monolithic", MONOLITHIC_WORLD)):
        imports, exports = world_membership(path)
        also_imported = export_only & imports
        if also_imported:
            problems.append(
                f"{label} world imports an export-only interface: {sorted(also_imported)} "
                "(the host would be forced to implement a guest-owned callback)"
            )
        not_exported = export_only - exports
        if not_exported:
            problems.append(
                f"{label} world does not export an export-only interface: {sorted(not_exported)}"
            )

    if problems:
        print("WIT generator contract gate failed:", file=sys.stderr)
        for problem in problems:
            print(f"  - {problem}", file=sys.stderr)
        return 1

    print(
        f"WIT generator contract OK: {len(export_only)} export-only "
        f"({', '.join(sorted(export_only))}), {len(exported)} exported callbacks"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
