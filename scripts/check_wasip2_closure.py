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
edges read as enabled (measured chain:
`tairitsu-web -> tairitsu-packager -> tera -> slug -> wasm-bindgen`), and the
graph goes red for reasons that never occur in a component build. Note `rav1e`
and `tairitsu-web` themselves carry no bindgen edge on this target — only the
optional `packager` subtree does.
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

        # The expectation is derived from the *raw lines* and carries no predicate
        # on package names whatsoever. Two review rounds proved why that matters:
        #   - a helper-based oracle can be pointed at the parser (`x != x`);
        #   - a name-shaped expectation can be mirrored by whatever filter the
        #     parser grows (`not n.startswith(...)` on both sides), which let a
        #     coordinated edit print a green banner with `chrono` really present.
        # Counting only blank and "(annotation)" lines leaves a number that a
        # name filter cannot shrink, so the drop shows up as a count mismatch.
        expected_count = 0
        for raw in text.splitlines():
            body = raw.strip()
            if body and body[0] != "(":
                expected_count += 1

        # Zero hits must never read as green. The ratio is the load-bearing half:
        # a size-conditional parser (correct on tiny inputs, blind on real ones)
        # clears any absolute floor while leaving most lines unaccounted for.
        if expected_count < 5 or len(members) < max(2, expected_count // 2):
            print(
                "wasip2 closure gate could not run: parsed "
                f"{len(members)} of {expected_count} lines from the tree of {crate} "
                "(the extraction is broken, not the closure clean)",
                file=sys.stderr,
            )
            return 2

        # Count equal is not enough either: a parser can drop one name and invent
        # another. Rebuild the expected names here, once, and compare in full.
        expected_names = []
        for raw in text.splitlines():
            body = raw.strip()
            if body and body[0] != "(":
                expected_names.append(body.split()[0])
        if len(members) != expected_count or members != expected_names:
            missing = [n for n in expected_names if n not in members]
            print(
                "wasip2 closure gate could not run: the parse of "
                f"{crate} does not match its own input line-for-line "
                f"({len(members)} vs {expected_count}, missing {missing[:5]})",
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
    """A capture of a real `cargo tree --target wasm32-wasip2 --prefix none` run.

    Kept at real size on purpose: an earlier, 12-line fixture let a reviewer hide
    a size-conditional blindness in `parse_members` with `--selftest` still green.
    A fixture smaller than real output cannot pin a parser against real output.
    Paths are normalised to `/src/...` so the capture is portable.
    """
    return (
        "tairitsu-hooks v0.5.26 (/src/...)\n"
        "tairitsu-vdom v0.5.26 (/src/...)\n"
        "tracing v0.1.44\n"
        "pin-project-lite v0.2.18\n"
        "tracing-attributes v0.1.31 (proc-macro)\n"
        "proc-macro2 v1.0.107\n"
        "unicode-ident v1.0.27\n"
        "quote v1.0.47\n"
        "proc-macro2 v1.0.107 (*)\n"
        "syn v2.0.119\n"
        "proc-macro2 v1.0.107 (*)\n"
        "quote v1.0.47 (*)\n"
        "unicode-ident v1.0.27\n"
        "tracing-core v0.1.36\n"
        "once_cell v1.21.4\n"
        "wit-bindgen v0.57.1\n"
        "bitflags v2.13.2\n"
        "wit-bindgen-rust-macro v0.57.1 (proc-macro)\n"
        "anyhow v1.0.105\n"
        "macro-string v0.2.0\n"
        "proc-macro2 v1.0.107 (*)\n"
        "quote v1.0.47 (*)\n"
        "syn v2.0.119 (*)\n"
        "prettyplease v0.2.37\n"
        "proc-macro2 v1.0.107 (*)\n"
        "syn v2.0.119 (*)\n"
        "proc-macro2 v1.0.107 (*)\n"
        "quote v1.0.47 (*)\n"
        "syn v2.0.119 (*)\n"
        "wit-bindgen-core v0.57.1\n"
        "anyhow v1.0.105\n"
        "heck v0.5.0\n"
        "wit-parser v0.247.0\n"
        "anyhow v1.0.105\n"
        "hashbrown v0.17.2\n"
        "foldhash v0.2.0\n"
        "id-arena v2.3.0\n"
        "indexmap v2.14.2\n"
        "equivalent v1.0.3\n"
        "hashbrown v0.17.2 (*)\n"
        "serde_core v1.0.229\n"
        "log v0.4.35\n"
        "semver v1.0.28\n"
        "serde v1.0.229\n"
        "serde_core v1.0.229\n"
        "serde_derive v1.0.229 (proc-macro)\n"
        "proc-macro2 v1.0.107 (*)\n"
        "quote v1.0.47 (*)\n"
        "syn v3.0.10\n"
        "proc-macro2 v1.0.107 (*)\n"
        "quote v1.0.47 (*)\n"
        "unicode-ident v1.0.27\n"
        "serde_json v1.0.154\n"
        "itoa v1.0.19\n"
        "memchr v2.8.3\n"
        "serde_core v1.0.229\n"
        "zmij v1.0.23\n"
        "unicode-xid v0.2.6\n"
        "wasmparser v0.247.0\n"
        "bitflags v2.13.2\n"
        "hashbrown v0.17.2 (*)\n"
        "indexmap v2.14.2 (*)\n"
        "semver v1.0.28\n"
        "wit-bindgen-rust v0.57.1\n"
        "anyhow v1.0.105\n"
        "heck v0.5.0\n"
        "indexmap v2.14.2 (*)\n"
        "prettyplease v0.2.37 (*)\n"
        "syn v2.0.119 (*)\n"
        "wasm-metadata v0.247.0\n"
        "anyhow v1.0.105\n"
        "indexmap v2.14.2 (*)\n"
        "wasm-encoder v0.247.0\n"
        "leb128fmt v0.1.0\n"
        "wasmparser v0.247.0 (*)\n"
        "wasmparser v0.247.0 (*)\n"
        "wit-bindgen-core v0.57.1 (*)\n"
        "wit-component v0.247.0\n"
        "anyhow v1.0.105\n"
        "bitflags v2.13.2\n"
        "indexmap v2.14.2 (*)\n"
        "log v0.4.35\n"
        "serde v1.0.229 (*)\n"
        "serde_derive v1.0.229 (proc-macro) (*)\n"
        "serde_json v1.0.154 (*)\n"
        "wasm-encoder v0.247.0 (*)\n"
        "wasm-metadata v0.247.0 (*)\n"
        "wasmparser v0.247.0 (*)\n"
        "wit-parser v0.247.0 (*)\n"
        "wit-bindgen v0.57.1 (*)\n"
    )


def _expected_names(tree_text: str) -> list[str]:
    """An independent oracle: what a faithful parser must return.

    Deliberately *not* implemented in terms of `parse_members`. A reviewer turned
    the oracle into `return parse_members(tree_text)` and the identity check
    below became a tautology (`x != x` is never true), which re-opened the whole
    class of name-dropping parsers. Keeping this body written out by hand — regex
    and character classes, no shared helper — is the property the selftest
    enforces by asserting that a *mutated* parse disagrees with the oracle.
    """
    out: list[str] = []
    for raw in tree_text.splitlines():
        body = raw.strip()
        # Blank lines and cargo's own annotations like "(proc-macro)" carry no name.
        if body == "" or body[0] == "(":
            continue
        out.append(body.split()[0])
    return out


def selftest() -> int:
    """Pin the parser by *position*, not by count.

    A count-only assertion is not enough: a reviewer wrote a parser that drops
    `chrono` by *name* when it appears anywhere but the last line, which kept
    every count right, kept `--selftest` green, and reported a clean closure
    while `chrono` was really in the tree. Matching the whole parsed list against
    the whole expected list makes "which line did you drop" the question.
    """
    fixture = _fixture()
    fixture_lines = fixture.count("\n")
    # The guard is the point: a fixture that is smaller than real output makes
    # every "size-dependent" mutation invisible. Kept as a hard assertion so a
    # future edit cannot quietly shrink it.
    if fixture_lines < 80:
        print(
            f"FAIL fixture is only {fixture_lines} lines; it must be comparable to "
            "real cargo tree output (>= 80)",
            file=sys.stderr,
        )
        return 1

    # The oracle must be able to DISAGREE with the parser. A reviewer rewrote
    # `_expected_names` as `return parse_members(tree_text)` and paired it with a
    # name-dropping parser: `members != expected` became `x != x`, the gate
    # printed its green banner with `chrono` really in the tree. Asserting the
    # disagreement (with a deliberately wrong parser) is what makes the oracle
    # load-bearing rather than decorative.
    probe = "alpha v1\nbeta v2\ngamma v3\n"
    if _expected_names(probe) != ["alpha", "beta", "gamma"]:
        print(
            f"FAIL oracle disagrees with the plain reading of {probe!r}: "
            f"{_expected_names(probe)}",
            file=sys.stderr,
        )
        return 1
    if _expected_names(probe) == parse_members(probe[:-len("gamma v3\n")]):
        print(
            "FAIL oracle is tautological: it tracks the parser instead of the input",
            file=sys.stderr,
        )
        return 1

    cases = [
        ("clean fixture", fixture, []),
        ("deep occurrence is found", fixture + "wasm-bindgen v0.2.130\n", ["wasm-bindgen"]),
        ("nested occurrence is found", fixture + "chrono v0.4.45\n", ["chrono"]),
        (
            "duplicate entries dedupe",
            fixture + "js-sys v0.3.107\nweb-sys v0.3.107\njs-sys v0.3.107 (*)\n",
            ["js-sys", "web-sys"],
        ),
        (
            "prefix name does not false-positive",
            fixture + "wasm-bindgen-macro-support v0.2.100\n",
            [],
        ),
        ("annotation lines are not packages", fixture + "(proc-macro)\n", []),
    ]
    # Occurrence at the *head* and *middle* of the tree, not just appended: the
    # reviewer's bypass keyed on "anything but the last line", and in a real tree
    # a first-party crate's direct dependencies sit on line 2.
    head_cases = [
        ("chrono on the second line", "chrono v0.4.45\n" + fixture, ["chrono"]),
    ]
    lines = fixture.splitlines(keepends=True)
    mid_wasm = (
        "".join(lines[: len(lines) // 2])
        + "wasm-bindgen v0.2.130\n"
        + "".join(lines[len(lines) // 2 :])
    )
    head_cases.append(("wasm-bindgen in the middle", mid_wasm, ["wasm-bindgen"]))
    mid = "".join(lines[: len(lines) // 2]) + "web-sys v0.3.107\n" + "".join(lines[len(lines) // 2 :])

    failed = 0
    all_cases = cases + head_cases + [("web-sys in the middle", mid, ["web-sys"])]
    for label, text, expected in all_cases:
        parsed = parse_members(text)
        got = forbidden_in(parsed)
        if got != expected:
            print(f"FAIL {label}: expected {expected}, got {got}", file=sys.stderr)
            failed += 1
            continue
        # Positional equality: every line accounted for, in order, nothing dropped.
        if parsed != _expected_names(text):
            missing = [n for n in _expected_names(text) if n not in parsed]
            print(
                f"FAIL {label}: parsed list does not match line-for-line "
                f"({len(parsed)} vs {len(_expected_names(text))} lines, missing {missing[:5]})",
                file=sys.stderr,
            )
            failed += 1
            continue
        print(f"ok   {label}")
    if failed:
        return 1
    print(f"selftest OK: {len(all_cases)} cases")
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