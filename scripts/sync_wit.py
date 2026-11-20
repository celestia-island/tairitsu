#!/usr/bin/env python3
"""Sync the composed WIT tree from `browser-worlds` into its publishing copies.

`packages/browser-worlds/wit/composed/` is the single source of truth: it is the
tree `generate_browser_wit.py` emits and the one the `browser-worlds` crate embeds.
`tairitsu-web` and `tairitsu-ssr` must ship the same tree inside their own crates
(crates.io `include` lists it), so they carry **byte-identical copies** — never a
hand-edited fork. Drift between the three used to be invisible until a host
interface silently went missing at instantiation time; `--check` turns that into
a red gate.

Usage:
    python3 scripts/sync_wit.py            # rewrite the copies from the source
    python3 scripts/sync_wit.py --check    # compare only; exit 1 on any drift
"""
from __future__ import annotations

import argparse
import filecmp
import shutil
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

SOURCE = REPO_ROOT / "packages" / "browser-worlds" / "wit" / "composed"
COPY_ROOTS = ("web", "ssr")


def copies() -> list[Path]:
    return [REPO_ROOT / "packages" / name / "wit" / "composed" for name in COPY_ROOTS]


def _relative_files(root: Path) -> set[Path]:
    return {path.relative_to(root) for path in root.rglob("*") if path.is_file()}


def drift(source: Path, dest: Path) -> list[str]:
    """Every path that is missing, extra, or byte-different between the two trees."""
    problems: list[str] = []
    source_files = _relative_files(source)
    dest_files = _relative_files(dest)

    for rel in sorted(source_files - dest_files):
        problems.append(f"missing in {dest.relative_to(REPO_ROOT)}: {rel}")
    for rel in sorted(dest_files - source_files):
        problems.append(f"extra in {dest.relative_to(REPO_ROOT)}: {rel}")
    for rel in sorted(source_files & dest_files):
        if not filecmp.cmp(source / rel, dest / rel, shallow=False):
            problems.append(f"differs: {dest.relative_to(REPO_ROOT)}/{rel}")
    return problems


def sync_one(source: Path, dest: Path) -> int:
    if dest.exists():
        shutil.rmtree(dest)
    shutil.copytree(source, dest)
    return len(_relative_files(dest))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--check",
        action="store_true",
        help="compare only; exit 1 when any copy drifts from the single source",
    )
    args = parser.parse_args()

    if not SOURCE.is_dir():
        print(f"error: single source missing: {SOURCE}", file=sys.stderr)
        return 2

    if args.check:
        problems: list[str] = []
        for dest in copies():
            if not dest.is_dir():
                problems.append(f"missing copy tree: {dest.relative_to(REPO_ROOT)}")
                continue
            problems.extend(drift(SOURCE, dest))
        if problems:
            print("composed WIT drift detected (single source = "
                  f"{SOURCE.relative_to(REPO_ROOT)}):", file=sys.stderr)
            for problem in problems:
                print(f"  - {problem}", file=sys.stderr)
            print("run `python3 scripts/sync_wit.py` to regenerate the copies", file=sys.stderr)
            return 1
        print(f"composed WIT in sync ({len(copies())} copies checked)")
        return 0

    for dest in copies():
        count = sync_one(SOURCE, dest)
        print(f"synced {count} WIT files -> {dest.relative_to(REPO_ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())