#!/usr/bin/env python3
"""Gate: the wasip2 delivery closure must carry no bindgen family and no chrono.

Project D6 (`_reports/tairitsu-hikari-wasi-browser-20261121/PLAN.md` §3-D6) has
a cheap, first-party end state: the guest-side packages a browser component
links against must not contain wasm-bindgen / js-sys / web-sys /
wasm-bindgen-futures / chrono on `--target wasm32-wasip2`.

Why this needs a gate and not a sentence in the docs: `Cargo.lock` is
target-independent, so a crate that only depends on `web-sys` under
`cfg(target_os = "unknown")` still writes that edge into the lock. A manifest
edit that adds `wasm-bindgen = "0.2"` therefore shows up as nothing at all in the
lock, compiles fine for `wasm32-wasip2`, and leaves every existing byte-level
check green. Measured before this gate existed: injecting exactly that line into
`packages/hooks/Cargo.toml` produced zero failures anywhere.

Scope is the **delivery closure**, not the whole workspace. `--workspace` is not
clean today and cannot be: `examples/website` pulls
`hikari-components -> hikari-animation`, and `tairitsu-packager -> webbrowser ->
web-sys`. Neither is a shipped browser delivery path, so asserting them here
would make the gate red for reasons it does not own — and a permanently red gate
is worse than no gate.

Polarity, copied from akivili's `tf-lock-assert`: `cargo tree -i <pkg>` exits 0
(printing the reverse dependencies) when the package **is** in the graph, so a
zero exit here is the failure.

Usage:
    python3 scripts/check_wasip2_closure.py --check

Exit codes:
    0  no forbidden package is reachable from any delivery crate
    1  at least one is reachable (reverse-dependency paths are printed)
    2  the check could not run (cargo missing, tree failed) — never silently green
"""
from __future__ import annotations

import argparse
import os
import shutil
import subprocess
import sys

TARGET = "wasm32-wasip2"

# D6's set. `chrono` is the second half of the same ruling.
FORBIDDEN = (
    "wasm-bindgen",
    "wasm-bindgen-futures",
    "js-sys",
    "web-sys",
    "chrono",
)

# The guest-side packages a browser component links against, measured clean on
# `--target wasm32-wasip2` when this gate landed. Adding one here widens the
# assertion; changing this tuple is a scope decision, so say why in the PR.
DELIVERY_CRATES = (
    "tairitsu-web",
    "tairitsu-hooks",
    "tairitsu-vdom",
    "tairitsu-style",
    "tairitsu-macros",
)

# `cargo tree` prints this when the package is simply not reachable from the
# selected root. Absence is only believed when it is corroborated by a clean
# forward tree — an extractor that reads 0 hits as "absent" without a positive
# control is the classic zero-hit-read-as-green trap.
ABSENT_MARKER = "nothing to print"

_TREE_GLYPHS = ("├── ", "└── ", "│   ", "    ", "├", "└")


def run_tree(cargo: str, args: list[str]) -> tuple[int, str]:
    try:
        proc = subprocess.run(
            [cargo, "tree", "--target", TARGET, *args],
            capture_output=True,
            text=True,
        )
    except OSError as exc:
        # A missing/unrunnable cargo must surface as "could not run" (2), not as
        # a traceback and not as a clean tree.
        raise RuntimeError(f"{cargo}: {exc}") from exc
    return proc.returncode, (proc.stdout or "") + (proc.stderr or "")


def forward_hits(cargo: str, crate: str) -> list[str]:
    """Forbidden package names reachable from `crate`'s forward wasip2 tree."""
    rc, output = run_tree(cargo, ["-p", crate, "-e", "normal"])
    if rc != 0:
        raise RuntimeError(f"cargo tree -p {crate} failed:\n{output}")
    hits: list[str] = []
    for line in output.splitlines():
        name = line
        for glyph in _TREE_GLYPHS:
            name = name.replace(glyph, " ", 1)
        name = name.split(" v")[0].strip()
        if name in FORBIDDEN and name not in hits:
            hits.append(name)
    return hits


def reverse_deps(cargo: str, pkg: str, crate: str) -> tuple[bool, str]:
    """Reverse-dependency paths for `pkg`, rooted at the delivery crate.

    Rooted, not whole-workspace: an unrooted `-i` reports every path in the
    workspace (including the example and the packager), which buries the one
    path that actually broke the assertion. Rooting at `crate` prints exactly
    the chain the gate is complaining about.
    """
    rc, output = run_tree(cargo, ["-p", crate, "-i", pkg])
    if rc != 0:
        return False, output
    return ABSENT_MARKER not in output, output


def check(cargo: str) -> int:
    present: list[str] = []
    failures: list[str] = []
    for crate in DELIVERY_CRATES:
        try:
            hits = forward_hits(cargo, crate)
        except RuntimeError as exc:
            print(f"wasip2 closure gate could not run: {exc}", file=sys.stderr)
            return 2
        for pkg in hits:
            present.append(f"{pkg} (via {crate})")
            found, output = reverse_deps(cargo, pkg, crate)
            if found:
                failures.append(output.rstrip())

    if present:
        print(
            f"wasip2 closure gate failed: {', '.join(present)} reachable on "
            f"`--target {TARGET}`",
            file=sys.stderr,
        )
        for text in failures:
            print("\n".join(text.splitlines()[:6]), file=sys.stderr)
        print(
            "D6: a manifest re-added a package the wasip2 delivery closure must "
            "not contain.",
            file=sys.stderr,
        )
        return 1

    print(
        f"wasip2 closure OK: none of {len(FORBIDDEN)} forbidden packages "
        f"({', '.join(FORBIDDEN)}) reachable from {len(DELIVERY_CRATES)} delivery "
        f"crates on `--target {TARGET}`"
    )
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="run the gate")
    parser.add_argument("--cargo", default=None, help="cargo binary (default: from PATH)")
    args = parser.parse_args()

    cargo = args.cargo or shutil.which("cargo")
    if not cargo:
        print("wasip2 closure gate could not run: cargo not found", file=sys.stderr)
        return 2
    if not (os.path.isfile(cargo) and os.access(cargo, os.X_OK)):
        print(f"wasip2 closure gate could not run: {cargo} is not executable", file=sys.stderr)
        return 2
    return check(cargo)


if __name__ == "__main__":
    sys.exit(main())