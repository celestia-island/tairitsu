#!/usr/bin/env python3
"""Gate: the wasip2 delivery closure must carry no bindgen family and no chrono.

Project D6 (`_reports/tairitsu-hikari-wasi-browser-20261121/PLAN.md` §3-D6) has
a cheap, first-party end state: the guest-side packages a browser component
links against must not reach wasm-bindgen / js-sys / web-sys /
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

Why `cargo tree` and not `cargo metadata`: metadata resolves the feature union
across every workspace member, so `tairitsu-web`'s *optional* `packager` / `ssr`
edges read as enabled (`-> image -> ravif -> rav1e -> wasm-bindgen`) and the
graph goes red for reasons that never occur in a component build.
`cargo tree -p <crate>` resolves the default build of that one crate, which is
the question being asked.

The tree scraper is the fragile part. A reviewer proved an earlier version could
be made blind by one line (nested entries kept their `│` prefixes and never
matched), so the parse is `--prefix none` plus a path-component-independent token
scan, `--selftest` pins it against captured shapes, and a tree that parses to
almost nothing is reported as "could not run" rather than "clean".

Usage:
    python3 scripts/check_wasip2_closure.py --check
    python3 scripts/check_wasip2_closure.py --selftest

Exit codes:
    0  no forbidden package reachable from any delivery crate (or selftest ok)
    1  at least one is reachable (the offending chain is printed)
    2  the check could not run (cargo missing, tree failed/empty) — never green
"""
from __future__ import annotations

import argparse
import os
import re
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

# `--prefix none` strips the tree glyphs but keeps the leading indentation, so a
# crate name is whatever non-space token starts a line. This does not depend on
# box-drawing characters at all, which is what made the first scraper blind.
_TOKEN_RE = re.compile(r"(\S+)")


class GateError(RuntimeError):
    """The check could not produce a verdict (exit 2, never green)."""


def parse_members(tree_text: str) -> list[str]:
    """Package names present in a `cargo tree --prefix none` listing."""
    names: list[str] = []
    for line in tree_text.splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("("):
            continue
        token = _TOKEN_RE.match(stripped)
        if not token:
            continue
        name = token.group(1)
        # `name v<version>`: a line starting with "v" and a version number is a
        # continuation artifact, never a crate name.
        if re.fullmatch(r"v\d.*", name):
            continue
        names.append(name)
    return names


def forbidden_in(names: list[str]) -> list[str]:
    hits: list[str] = []
    for name in names:
        if name in FORBIDDEN and name not in hits:
            hits.append(name)
    return hits


def run_tree(cargo: str, crate: str) -> str:
    try:
        proc = subprocess.run(
            [
                cargo,
                "tree",
                "--target",
                TARGET,
                "-p",
                crate,
                "-e",
                "normal",
                "--prefix",
                "none",
            ],
            capture_output=True,
            text=True,
        )
    except OSError as exc:
        raise GateError(f"{cargo}: {exc}") from exc
    if proc.returncode != 0:
        raise GateError(f"cargo tree -p {crate} failed:\n{proc.stderr}")
    return proc.stdout or ""


def reverse_chain(cargo: str, crate: str, pkg: str) -> str:
    """The offending chain, rooted at the delivery crate, for the error report."""
    try:
        proc = subprocess.run(
            [cargo, "tree", "--target", TARGET, "-p", crate, "-i", pkg],
            capture_output=True,
            text=True,
        )
    except OSError:
        return ""
    return (proc.stdout or "").rstrip()


def check(cargo: str) -> int:
    present: list[tuple[str, str]] = []
    for crate in DELIVERY_CRATES:
        try:
            text = run_tree(cargo, crate)
        except GateError as exc:
            print(f"wasip2 closure gate could not run: {exc}", file=sys.stderr)
            return 2
        members = parse_members(text)
        # A tree that parses to almost nothing means the extraction is broken,
        # not that the closure is clean — zero hits must never read as green.
        if text.count("\n") < 5 or len(members) < 2:
            print(
                "wasip2 closure gate could not run: no packages parsed from the "
                f"tree of {crate}",
                file=sys.stderr,
            )
            return 2
        for pkg in forbidden_in(members):
            present.append((crate, pkg))

    if present:
        summary = ", ".join(f"{pkg} (via {crate})" for crate, pkg in present)
        print(
            f"wasip2 closure gate failed: {summary} reachable on `--target {TARGET}`",
            file=sys.stderr,
        )
        for crate, pkg in present:
            chain = reverse_chain(cargo, crate, pkg)
            if chain:
                print("\n".join(chain.splitlines()[:6]), file=sys.stderr)
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


def _fixture() -> str:
    """A captured shape, kept verbatim so the parser is pinned to real output."""
    return (
        "tairitsu-hooks v0.5.26 (/src/packages/hooks)\n"
        "tairitsu-vdom v0.5.26 (/src/packages/vdom)\n"
        "anyhow v1.0.100\n"
        "adler2 v2.0.1\n"
        "tracing v0.1.44\n"
        "pin-project-lite v0.2.18\n"
        "tracing-attributes v0.1.31 (proc-macro)\n"
        "proc-macro2 v1.0.107\n"
        "unicode-ident v1.0.27\n"
        "quote v1.0.47\n"
        "syn v2.0.119\n"
        "proc-macro2 v1.0.107 (*)\n"
    )


def selftest() -> int:
    """Pin the parser in both directions against captured output shapes."""
    cases = [
        ("clean fixture", _fixture(), []),
        ("deep occurrence is found", _fixture() + "wasm-bindgen v0.2.130\n", ["wasm-bindgen"]),
        ("nested occurrence is found", _fixture() + "chrono v0.4.45\n", ["chrono"]),
        (
            "duplicate entries dedupe",
            _fixture() + "js-sys v0.3.107\nweb-sys v0.3.107\njs-sys v0.3.107 (*)\n",
            ["js-sys", "web-sys"],
        ),
        (
            "prefix name does not false-positive",
            _fixture() + "wasm-bindgen-macro-support v0.2.100\n",
            [],
        ),
        ("annotation lines are not packages", _fixture() + "(proc-macro)\n", []),
    ]
    failed = 0
    for label, text, expected in cases:
        got = forbidden_in(parse_members(text))
        if got != expected:
            print(f"FAIL {label}: expected {expected}, got {got}", file=sys.stderr)
            failed += 1
        else:
            print(f"ok   {label}")
    if failed:
        return 1
    print(f"selftest OK: {len(cases)} cases")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="run the gate")
    parser.add_argument("--selftest", action="store_true", help="run the parser selftest")
    parser.add_argument("--cargo", default=None, help="cargo binary (default: from PATH)")
    args = parser.parse_args()

    if args.selftest:
        return selftest()

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