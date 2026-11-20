#!/usr/bin/env python3
"""Sync the composed WIT tree from `browser-worlds` into its publishing copies.

`packages/browser-worlds/wit/composed/` is the single source of truth: it is what
`generate_browser_wit.py` emits (and `just wit-gen` now chains this script, so a
regeneration cannot leave the copies behind) and the tree the `browser-worlds`
crate embeds.
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
import re
import shutil
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

SOURCE = REPO_ROOT / "packages" / "browser-worlds" / "wit" / "composed"
COPY_ROOTS = ("web", "ssr")

# A floor, not a target: it stops an emptied or half-copied source tree from
# being propagated over the copies in write mode.
MIN_SOURCE_FILES = 25

# `packages/ssr/build.rs` parses the MONOLITHIC file (not the composed tree) to
# emit its host stubs, and silently degrades to "no stubs" when the file is
# missing — so that copy is part of the single-source contract too.
MONOLITHIC_SOURCE = REPO_ROOT / "packages" / "browser-worlds" / "wit" / "browser-full.wit"
MONOLITHIC_DEST = REPO_ROOT / "packages" / "ssr" / "wit" / "browser-full.wit"


def copies() -> list[Path]:
    return [REPO_ROOT / "packages" / name / "wit" / "composed" for name in COPY_ROOTS]


def _relative_files(root: Path) -> set[Path]:
    return {path.relative_to(root) for path in root.rglob("*") if path.is_file()}


INTERFACE_RE = re.compile(r"^interface ([a-z0-9-]+)", re.MULTILINE)
WORLD_START_RE = re.compile(r"^world ([a-z0-9-]+) \{", re.MULTILINE)
WORLD_MEMBER_RE = re.compile(r"^\s+(import|export) ([a-z0-9-]+)\s*;", re.MULTILINE)


def _world_membership(text: str) -> tuple[set[str], set[str]]:
    """(imported, exported) interface names of the last world block in `text`."""
    match = None
    for match in WORLD_START_RE.finditer(text):
        pass
    if match is None:
        return set(), set()
    tail = text[match.end():]
    depth = 1
    body_chars: list[str] = []
    for ch in tail:
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
            if depth == 0:
                break
        body_chars.append(ch)
    body = "".join(body_chars)
    imports = {name for kind, name in WORLD_MEMBER_RE.findall(body) if kind == "import"}
    exports = {name for kind, name in WORLD_MEMBER_RE.findall(body) if kind == "export"}
    return imports, exports


def _interface_names(text: str) -> set[str]:
    return set(INTERFACE_RE.findall(text))


def monolithic_parity() -> list[str]:
    if not MONOLITHIC_SOURCE.is_file():
        return [f"missing single source: {MONOLITHIC_SOURCE.relative_to(REPO_ROOT)}"]
    """The monolithic world and the composed tree must declare the same interfaces.

    ssr parses the monolithic file to emit host stubs but binds `wit/composed`
    for its types; if the two disagree, the stubs and the bindings describe
    different worlds — and neither byte-gate notices, because each pair stays
    internally consistent.
    """
    monolithic = _interface_names(MONOLITHIC_SOURCE.read_text(encoding="utf-8", errors="replace"))
    composed: set[str] = set()
    for path in SOURCE.glob("*.wit"):
        if path.name == "browser-full.wit":
            continue
        composed |= _interface_names(path.read_text(encoding="utf-8", errors="replace"))

    only_monolithic = sorted(monolithic - composed)
    only_composed = sorted(composed - monolithic)
    problems: list[str] = []
    if only_monolithic:
        problems.append(f"declared only in the monolithic world: {', '.join(only_monolithic)}")
    if only_composed:
        problems.append(f"declared only in the composed tree: {', '.join(only_composed)}")

    # Same interface set is not enough: a world that imports what the other one
    # exports describes a different host obligation (the guest owns the callback
    # in one, the host must provide it in the other).
    m_imports, m_exports = _world_membership(
        MONOLITHIC_SOURCE.read_text(encoding="utf-8", errors="replace")
    )
    c_imports, c_exports = _world_membership(
        (SOURCE / "browser-full.wit").read_text(encoding="utf-8", errors="replace")
    )
    for label, mono, comp in (("import", m_imports, c_imports), ("export", m_exports, c_exports)):
        if mono - comp:
            problems.append(
                f"{label}ed only by the monolithic world: {', '.join(sorted(mono - comp))}"
            )
        if comp - mono:
            problems.append(
                f"{label}ed only by the composed world: {', '.join(sorted(comp - mono))}"
            )
    return problems


def monolithic_drift() -> list[str]:
    if not MONOLITHIC_SOURCE.is_file():
        return [f"missing single source: {MONOLITHIC_SOURCE.relative_to(REPO_ROOT)}"]
    if not MONOLITHIC_DEST.is_file():
        return [f"missing copy: {MONOLITHIC_DEST.relative_to(REPO_ROOT)}"]
    if not filecmp.cmp(MONOLITHIC_SOURCE, MONOLITHIC_DEST, shallow=False):
        return [f"differs: {MONOLITHIC_DEST.relative_to(REPO_ROOT)}"]
    return []


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
    """Stage next to the target, then swap — a failure never leaves a half tree."""
    staging = dest.with_name(dest.name + ".sync-tmp")
    if staging.exists():
        shutil.rmtree(staging)
    shutil.copytree(source, staging)
    if dest.exists():
        shutil.rmtree(dest)
    staging.rename(dest)
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
        problems.extend(monolithic_drift())
        problems.extend(monolithic_parity())
        if problems:
            print("composed WIT drift detected (single source = "
                  f"{SOURCE.relative_to(REPO_ROOT)}):", file=sys.stderr)
            for problem in problems:
                print(f"  - {problem}", file=sys.stderr)
            print("run `python3 scripts/sync_wit.py` to regenerate the copies", file=sys.stderr)
            return 1
        print(f"composed WIT in sync ({len(copies())} copies checked)")
        return 0

    source_files = _relative_files(SOURCE)
    if len(source_files) < MIN_SOURCE_FILES:
        print(
            f"error: single source has only {len(source_files)} files "
            f"(< {MIN_SOURCE_FILES}) — refusing to overwrite the copies",
            file=sys.stderr,
        )
        return 1

    for dest in copies():
        count = sync_one(SOURCE, dest)
        print(f"synced {count} WIT files -> {dest.relative_to(REPO_ROOT)}")
    shutil.copyfile(MONOLITHIC_SOURCE, MONOLITHIC_DEST)
    print(f"synced monolithic world -> {MONOLITHIC_DEST.relative_to(REPO_ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
