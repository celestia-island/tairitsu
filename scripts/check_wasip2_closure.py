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

Threat model — what this gate is for, and what it is not
-------------------------------------------------------
In scope: **dependency regression.** Someone adds `wasm-bindgen` / `chrono` (or
any of the family) back to a manifest, or a transitive edge creeps in, and the
wasip2 delivery closure goes dirty without anything else noticing.

Out of scope: **rewriting this script.** Five review rounds attacked the parser
and every round narrowed the surface (count guard -> line-for-line identity ->
inline expectation -> count derived from blank/annotation lines -> two independent
`cargo tree` runs plus a raw-text scan). The conclusion of those rounds is not
that another layer of in-script paranoia is needed: a machine-readable anchor
checked out *inside the same repository* can be edited by exactly the same
process that edits the parser, so it is theatre. The real anchor is outside the
repository — review, git history, and the fact that disabling a gate is visible
and attributable. This is the same property `comment-gate`, `artifact-drift` and
`pr_merge_gate` have; none of them is expected to defend against their own
rewriting either.

What the double run and the raw-text scan buy, then, is not "cannot be
defeated" but "no *single* edit to the parsing path yields a green verdict while
a forbidden package is present".

Usage:
    python3 scripts/check_wasip2_closure.py --check
    python3 scripts/check_wasip2_closure.py --check --strict
    python3 scripts/check_wasip2_closure.py --selftest

Cross-workspace entries
-----------------------
`DELIVERY_CRATES` holds one workspace first, but the closure that matters is
per delivery path, and one of those paths is hikari's: hikari #1521 moved
`wasm-bindgen` / `web-sys` / `js-sys` into a browser-target table so they leave
hikari's wasip2 closure too. That crate is asserted here through its own
manifest (`--manifest-path`): `examples/website` pulls hikari in at
`branch = "master"`, so resolving from *this* workspace follows this workspace's
lock, and that pin is local state (`Cargo.lock` is gitignored here) rather than
the checkout under review. Measured here, a bare entry does not reach hikari on
this target at all: `cargo tree --target wasm32-wasip2 -p hikari-animation`
prints `warning: nothing to print` and exits 0, which the gate reports as
"could not run" rather than as clean. Resolving through this workspace is
therefore not a usable assertion even when the pin happens to name the commit
under review — it can also miss the browser feature that makes the package's
dependencies resolve at all, and then a clean and a regressed manifest agree.

Locating that workspace must not hard-code an absolute path, so the rule is:
`HIKARI_MANIFEST_DIR` if set, else the `../hikari` sibling of this checkout
(then the worktree spelling described at `hikari_candidates`). If none exists the
entry is reported **NOT CHECKED** on stderr — never folded into the green
banner — and `--strict` turns that into a non-zero exit. The default stays
lenient on purpose: a developer without a hikari checkout is not wrong about
tairitsu, but they are also not told the closure is clean.

Note this gate is a **local / pre-merge** lane: it is reached through
`just wasip2-closure-check` / `just ci`, and no workflow under
`.github/workflows/` runs either, so the verdict is a human step, not a status
check.

Exit codes:
    0  no forbidden package reachable from any delivery crate that was asserted
       (or selftest ok). Under `--strict` this additionally requires that every
       entry could be located, so a clean 0 there means all of them passed
    1  at least one is reachable (the offending chain is printed)
    2  the check could not run (cargo missing, tree failed/empty), or an entry
       was skipped under `--strict` — never green either way
"""
from __future__ import annotations

import argparse
import contextlib
import io
import os
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path

TARGET = "wasm32-wasip2"

# D6's set. `chrono` is the second half of the same ruling.
FORBIDDEN = (
    "wasm-bindgen",
    "wasm-bindgen-futures",
    "js-sys",
    "web-sys",
    "chrono",
)

#: Environment override for the directory holding hikari's workspace
#: `Cargo.toml`; see `resolve_manifest`.
HIKARI_ENV = "HIKARI_MANIFEST_DIR"

#: The tairitsu checkout this script lives in (`<checkout>/scripts/..`). Anchored
#: to the script, not to the process working directory, so `just` at the repo
#: root, a cron job started from `/`, and `--manifest-path <wt>/Cargo.toml` all
#: resolve the same directory.
_REPO_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

#: Directories tried, in order, for the hikari workspace. The first is the plain
#: side-by-side layout (`<workspace>/tairitsu` next to `<workspace>/hikari`).
#: When this checkout sits inside `_worktree/<repo>/<slug>` — the layout
#: `_tools/wt_new.py` creates — the sibling is three levels up plus `hikari`, so
#: that spelling is added only when the path really has that shape; applying it
#: unconditionally would yield `/hikari` for a plain checkout. Nothing is globbed
#: because `<workspace>/_worktree/hikari` is a directory *of* worktrees, not a
#: checkout, and a glob would happily accept it. Every candidate must own a
#: `Cargo.toml` to qualify.
def hikari_candidates() -> tuple[str, ...]:
    """Directories to try, in order, when locating the hikari workspace."""
    candidates = [os.path.join(_REPO_ROOT, "..", "hikari")]
    # <workspace>/_worktree/<repo>/<slug> is one level deeper than a sibling
    # checkout; climbing three levels is only meaningful in that shape.
    if os.path.basename(os.path.dirname(os.path.dirname(_REPO_ROOT))) == "_worktree":
        candidates.append(os.path.join(_REPO_ROOT, "..", "..", "..", "hikari"))
    return tuple(candidates)


@dataclass(frozen=True)
class Delivery:
    """One crate whose wasip2 closure this gate asserts.

    `manifest` is the directory holding the `Cargo.toml` of the workspace the
    crate belongs to. `None` means "this workspace" — the historical behaviour,
    where `cargo tree -p <crate>` resolves against the manifest cargo finds from
    the current directory. A set `manifest` is passed as `--manifest-path`, which
    is what lets one run assert a crate in a *different* workspace: the entry
    has to name the other workspace explicitly, because the resolution cargo
    does from this one follows this workspace's lock, not that checkout's.
    """

    crate: str
    #: A directory bound at edit time. It wins over `HIKARI_MANIFEST_DIR` by
    #: design (an author who hard-codes a path means it), so it is never set on
    #: the entries this file ships — those stay `external` and honour the
    #: environment variable. A `manifest` and `external` together would leave two
    #: competing routes to the same crate, so that combination is refused.
    manifest: str | None = None
    #: True for an entry that belongs to *another* checkout, located by
    #: `resolve_manifest`. Entries of this workspace leave it `False`, which is
    #: what keeps their resolution — and their output — exactly as before.
    external: bool = False
    #: Cargo features to resolve with. Empty means "resolve the default build",
    #: which is what every entry did before this field existed.
    features: tuple[str, ...] = ()

    @property
    def label(self) -> str:
        """The entry as it appears in messages, e.g. `hikari-animation [+wasm]`."""
        return self.crate + (f" [+{','.join(self.features)}]" if self.features else "")


#: `hikari-animation`, bound to the hikari workspace through `resolve_manifest`.
#
# `features = ("wasm",)` is load-bearing, not decoration. hikari's browser-only
# dependencies are `optional = true` under a target table, so they only enter
# the graph when a consumer turns their feature on — and without the feature
# *both* the clean and the regressed manifest resolve to the same bindgen-free
# tree. Measured on a copy with the target table folded into `[dependencies]`:
# default resolution 0 hits (green, wrong); `--features wasm` 6 occurrences of
# the four bindgen packages (red, right). The feature is also what a real
# consumer gets — hikari's `packages/components/Cargo.toml` enables `hikari-animation`'s
# `wasm` feature for the browser build, which is why resolving without it is not
# the question being asked.
_HIKARI_ANIMATION = Delivery("hikari-animation", external=True, features=("wasm",))

# The guest-side packages a browser component links against, measured clean on
# `--target wasm32-wasip2` when this gate landed. Adding one here widens the
# assertion; changing this tuple is a scope decision, so say why in the PR.
#
# Each entry is either a bare crate name (resolved in *this* workspace, the
# behaviour every tairitsu entry has always had) or a `Delivery` carrying a
# `manifest` directory (resolved in that other workspace instead).
DELIVERY_CRATES = (
    "tairitsu-web",
    "tairitsu-hooks",
    "tairitsu-vdom",
    "tairitsu-style",
    "tairitsu-macros",
    # hikari's outlet-B ruling (hikari #1521): `wasm-bindgen` / `web-sys` /
    # `js-sys` left the wasip2 closure by moving to a
    # `cfg(all(target_arch = "wasm32", target_os = "unknown"))` table. Without
    # an entry here nothing would notice a manifest edit that moved them back.
    #
    # It needs its own `--manifest-path` and cannot ride the bare-name path:
    # `examples/website/Cargo.toml` pulls hikari in at `branch = "master"`, and
    # this workspace's lock resolves that to whatever commit it happened to pin
    # (`Cargo.lock` is gitignored here, so the value is local, not a repo fact).
    # A bare entry would therefore assert that commit's closure — or, on a
    # checkout whose resolve does not put hikari in the wasip2 graph at all,
    # print `warning: nothing to print` and trip the empty-tree guard. Measured:
    # `cargo tree --target wasm32-wasip2 -p hikari-animation` from this workspace
    # exits 0 with `warning: nothing to print`, which the gate reports as
    # "could not run" (rc 2) rather than asserting anything. With
    # `--manifest-path` it resolves the checkout next to it and returns a real
    # tree.
    #
    # `hikari-components` is deliberately absent, and not as a deferral. Two
    # measured facts carry that decision:
    #   1. none of the four wasip2 components
    #      (`packages/wasm/{qr,mermaid,scene3d,highlight}`) depends on it — their
    #      `cargo tree --target wasm32-wasip2` roots do not contain it, so it is
    #      outside every wasip2 delivery closure this project actually ships;
    #   2. its `chrono` is unconditional (`packages/components/Cargo.toml`), so it
    #      is in that crate's wasip2 closure (measured: 1 hit) no matter how the
    #      browser edges are gated — that is a T-F-track manifest question, not a
    #      delivery-closure regression, and this gate cannot fix it by asserting.
    # Note hikari's CI does compile the crate for wasip2 as part of
    # `cargo clippy --workspace --all-features --target wasm32-wasip2`; the crate
    # being a workspace member is why. "Shipped to wasip2" is the narrower
    # question this gate asks, and by (1) the answer is no. Asserting it here
    # would pin a permanently red entry over a closure no delivery path consumes,
    # which is the "a permanently red gate is worse than no gate" problem the
    # module docstring already calls out.
    _HIKARI_ANIMATION,
)

# `--prefix none` strips the tree glyphs but keeps the leading indentation, so a
# crate name is whatever non-space token starts a line. This does not depend on
# box-drawing characters at all, which is what made the first scraper blind.
_TOKEN_RE = re.compile(r"(\S+)")


class GateError(RuntimeError):
    """The check could not produce a verdict (exit 2, never green)."""


def _deliveries() -> tuple[Delivery, ...]:
    """`DELIVERY_CRATES` with bare names normalised to `Delivery(crate)`, in order.

    Resolved per call, not once at import, so the hikari directory honours an
    environment value set for the run rather than one snapshotted at import.
    """
    out: list[Delivery] = []
    for entry in DELIVERY_CRATES:
        if isinstance(entry, Delivery):
            if entry.manifest is not None and entry.external:
                raise GateError(
                    f"{entry.crate}: both a fixed manifest and `external` are set; "
                    f"`external` alone locates through {HIKARI_ENV}"
                )
            if entry.manifest is not None and not (Path(entry.manifest) / "Cargo.toml").is_file():
                raise GateError(
                    f"{entry.crate}: manifest directory {entry.manifest} does not exist"
                )
            out.append(entry)
        else:
            out.append(Delivery(entry))
    return tuple(out)


def resolve_manifest(entry: Delivery) -> tuple[Path | None, str]:
    """`(--manifest-path` directory, how it was found). `None` = this workspace.

    Only an entry marked `external` is located this way. An entry of *this*
    workspace always returns `None`, so `HIKARI_MANIFEST_DIR` and the hikari
    sibling never redirect the tairitsu crates to another workspace.

    The second element names the discovery source, because the caller reports it:
    a run that asserts a *sibling* checkout rather than the one set by the
    environment variable is a different fact from one that asserts the
    environment's, and only the caller can say so after the fact. It is one of
    `"workspace"`, `"entry"`, `"env"`, `"sibling"` (with the candidate that
    matched), so the report can never claim a route it did not take.

    Raises `GateError` when the entry is bound to another workspace but that
    workspace cannot be located: "could not find it" is never "it is clean".
    """
    if entry.manifest is not None:
        return Path(entry.manifest), "entry"
    if not entry.external:
        return None, "workspace"
    override = os.environ.get(HIKARI_ENV)
    if override:
        candidate = Path(override)
        if not (candidate / "Cargo.toml").is_file():
            # A malformed override is a configuration error, not a silent fall
            # through to the sibling: the operator asked for a specific checkout.
            raise GateError(
                f"{entry.crate}: {HIKARI_ENV}={override} does not contain a Cargo.toml"
            )
        return candidate, "env"
    tried: list[str] = []
    for sibling in hikari_candidates():
        candidate = Path(os.path.normpath(sibling))
        tried.append(str(candidate))
        if (candidate / "Cargo.toml").is_file():
            return candidate, f"sibling match at {candidate}"
    raise GateError(
        f"{entry.crate}: cannot locate the hikari workspace (tried {HIKARI_ENV} "
        f"(unset) and {', '.join(tried)}); nothing was asserted for this entry"
    )


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


def _tree_invocation(
    cargo: str,
    crate: str,
    manifest: Path | None,
    extra: list[str],
    features: tuple[str, ...] = (),
) -> list[str]:
    """`cargo tree` argv shared by the forward and reverse invocations.

    With a manifest the workspace is named outright; without one the argv is
    byte-for-byte what it has always been, so the five tairitsu entries still
    resolve exactly as before. `features` appends `--features` only when an
    entry asks for it, for the same reason.
    """
    argv = [cargo, "tree", "--target", TARGET, "-p", crate, *extra]
    if manifest is not None:
        argv += ["--manifest-path", str(manifest / "Cargo.toml")]
    if features:
        argv += ["--features", ",".join(features)]
    return argv


def run_tree(
    cargo: str, crate: str, manifest: Path | None = None, features: tuple[str, ...] = ()
) -> str:
    try:
        proc = subprocess.run(
            _tree_invocation(
                cargo, crate, manifest, ["-e", "normal", "--prefix", "none"], features
            ),
            capture_output=True,
            text=True,
        )
    except OSError as exc:
        raise GateError(f"{cargo}: {exc}") from exc
    if proc.returncode != 0:
        raise GateError(f"cargo tree -p {crate} failed:\n{proc.stderr}")
    return proc.stdout or ""


def reverse_chain(
    cargo: str,
    crate: str,
    pkg: str,
    manifest: Path | None = None,
    features: tuple[str, ...] = (),
) -> str:
    """The offending chain, rooted at the delivery crate, for the error report."""
    try:
        proc = subprocess.run(
            _tree_invocation(cargo, crate, manifest, ["-i", pkg], features),
            capture_output=True,
            text=True,
        )
    except OSError:
        return ""
    return (proc.stdout or "").rstrip()


#: Exit code for "this entry was not asserted" — distinct from both 0 (checked
#: and clean) and 1 (checked and dirty). It is what `--strict` returns, and it
#: never appears unless something really was skipped.
EXIT_NOT_CHECKED = 2


def check(cargo: str, strict: bool = False) -> int:
    """Run the gate. `GateError` anywhere means "could not run" (rc 2), never 0."""
    #: (crate, label, package, manifest, features) for each offending edge.
    present: list[tuple[str, str, str, Path | None, tuple[str, ...]]] = []
    skipped: list[tuple[str, str]] = []
    checked: list[str] = []
    #: (crate, resolved directory) for every external entry that was asserted, so
    #: a run against the trunk checkout instead of the branch under review is
    #: visible in the output rather than silent.
    asserted: list[tuple[str, str, str]] = []
    try:
        entries = _deliveries()
    except GateError as exc:
        print(f"wasip2 closure gate could not run: {exc}", file=sys.stderr)
        return 2
    for entry in entries:
        crate = entry.crate
        label = entry.label
        try:
            manifest, source = resolve_manifest(entry)
        except GateError as exc:
            # Not checked is not clean: say so loudly and let `--strict` fail.
            skipped.append((label, str(exc)))
            print(f"wasip2 closure NOT CHECKED: {exc}", file=sys.stderr)
            continue
        try:
            text = run_tree(cargo, crate, manifest, entry.features)
            # Cross-check against a second, independent invocation. A review
            # round showed the whole guard chain is only as good as `text`: one
            # line filtering `text` (or `run_tree`'s return) shrinks both the
            # expectation and the parse together, and they agree by construction.
            # Two separate `cargo tree` runs cannot shrink consistently unless
            # the *sources* agree, so filtering state has to be carried across
            # invocations to survive — which is a much narrower surface.
            second = run_tree(cargo, crate, manifest, entry.features)
        except GateError as exc:
            print(f"wasip2 closure gate could not run: {exc}", file=sys.stderr)
            return 2
        if text != second:
            print(
                f"wasip2 closure gate could not run: two independent `cargo tree` "
                f"runs for {label} disagreed ({len(text.splitlines())} vs "
                f"{len(second.splitlines())} lines)",
                file=sys.stderr,
            )
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
                f"{len(members)} of {expected_count} lines from the tree of {label} "
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
                f"{label} does not match its own input line-for-line "
                f"({len(members)} vs {expected_count}, missing {missing[:5]})",
                file=sys.stderr,
            )
            return 2

        for pkg in forbidden_in(members):
            present.append((crate, label, pkg, manifest, entry.features))

        # Belt and braces, computed straight off both raw texts: a search that
        # does not go through `parse_members` / `forbidden_in` at all. The third
        # run happens ONCE per entry, outside the loop below; hoisting it there
        # by mistake would cost five invocations per clean entry for no extra
        # assurance.
        try:
            third = run_tree(cargo, crate, manifest, entry.features)
        except GateError as exc:
            # "Could not run" must stay rc 2. Letting this third invocation
            # raise would surface a traceback with rc 1, which is the code that
            # means "a forbidden package was found" — the opposite verdict, from
            # the same number.
            print(f"wasip2 closure gate could not run: {exc}", file=sys.stderr)
            return 2
        for pkg in FORBIDDEN:
            needle = pkg + " v"
            raw_hit = any(
                ln.strip().startswith(needle) for ln in text.splitlines()
            ) or any(ln.strip().startswith(needle) for ln in third.splitlines())
            if raw_hit and (crate, pkg) not in [(c, p) for c, _, p, _, _ in present]:
                present.append((crate, label, pkg, manifest, entry.features))

        checked.append(label)
        if manifest is not None and entry.external:
            asserted.append((label, str(manifest), source))

    if present:
        summary = ", ".join(f"{pkg} (via {label})" for _, label, pkg, _, _ in present)
        print(
            f"wasip2 closure gate failed: {summary} reachable on `--target {TARGET}`",
            file=sys.stderr,
        )
        for crate, _label, pkg, manifest, features in present:
            chain = reverse_chain(cargo, crate, pkg, manifest, features)
            if chain:
                print("\n".join(chain.splitlines()[:6]), file=sys.stderr)
        print(
            "D6: a manifest re-added a package the wasip2 delivery closure must "
            "not contain.",
            file=sys.stderr,
        )
        return 1

    if skipped:
        names = ", ".join(crate for crate, _ in skipped)
        print(
            f"wasip2 closure gate NOT fully checked: {len(checked)} of "
            f"{len(checked) + len(skipped)} delivery crates were asserted clean "
            f"on `--target {TARGET}` ({', '.join(checked) or 'none'}); "
            f"{names} could not be located and was NOT checked "
            f"(not checked is not the same as clean)",
            file=sys.stderr,
        )
        if strict:
            print(
                f"wasip2 closure gate failed under --strict: {names} was not asserted",
                file=sys.stderr,
            )
            return EXIT_NOT_CHECKED
        return 0

    print(
        f"wasip2 closure OK: none of {len(FORBIDDEN)} forbidden packages "
        f"({', '.join(FORBIDDEN)}) reachable from {len(checked)} delivery "
        f"crates on `--target {TARGET}`"
    )
    for label, directory, source in asserted:
        hint = (
            f"  ({source}; not {HIKARI_ENV}, so set that to assert a specific "
            "hikari checkout)"
            if source.startswith("sibling")
            else ""
        )
        print(f"    asserted {label} from {directory}{hint}")
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

    # The cross-workspace surface. Each case is a property the `check()` run
    # depends on, asserted without invoking cargo: the argv a workspace entry
    # builds must stay byte-identical to the pre-change argv (that is what keeps
    # the five tairitsu entries' output unchanged), an external entry must name
    # its own manifest, and an unresolvable external entry must raise rather
    # than fall through to "this workspace" — the shape that would silently turn
    # "not checked" into "checked and clean".
    pre_change_argv = [
        "cargo",
        "tree",
        "--target",
        TARGET,
        "-p",
        "tairitsu-web",
        "-e",
        "normal",
        "--prefix",
        "none",
    ]
    checked_argv = _tree_invocation("cargo", "tairitsu-web", None, ["-e", "normal", "--prefix", "none"])
    if checked_argv != pre_change_argv:
        print(
            f"FAIL a workspace entry's argv changed: {checked_argv} != {pre_change_argv}",
            file=sys.stderr,
        )
        return 1
    print("ok   workspace entry argv is unchanged")

    manifest_argv = _tree_invocation(
        "cargo", "hikari-animation", Path("/elsewhere/hikari"), ["-e", "normal", "--prefix", "none"]
    )
    if manifest_argv[-2:] != ["--manifest-path", "/elsewhere/hikari/Cargo.toml"]:
        print(f"FAIL external entry does not pass its manifest: {manifest_argv}", file=sys.stderr)
        return 1
    print("ok   external entry passes --manifest-path <dir>/Cargo.toml")

    # The feature is the difference between a gate and a decoration: without it
    # the reverted-manifest copy resolves to the same bindgen-free tree (probe:
    # 0 occurrences default vs 6 with `--features wasm`), so an entry that lost
    # its features would still print a green banner over a dirty closure.
    external = [d for d in _deliveries() if d.external]
    if not external or not all(d.features for d in external):
        print(
            f"FAIL an external entry resolves without features: {external}",
            file=sys.stderr,
        )
        return 1
    feature_argv = _tree_invocation(
        "cargo", "hikari-animation", Path("/elsewhere/hikari"), ["-e", "normal"], ("wasm",)
    )
    if feature_argv[-2:] != ["--features", "wasm"]:
        print(f"FAIL features are not passed: {feature_argv}", file=sys.stderr)
        return 1
    print("ok   external entry resolves with its features")

    # Asserted through `resolve_manifest` AND through `check()`: the direct call
    # alone does not see the guard removed from a *shipped* entry, and on a
    # machine with no hikari sibling the difference is a `GateError` traceback
    # rather than a named failure.
    probe = Path(tempfile.mkdtemp(prefix="cg-ws-guard-"))
    (probe / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
    saved_for_guard = globals()["DELIVERY_CRATES"]
    try:
        globals()["DELIVERY_CRATES"] = ("tairitsu-web",)
        os.environ[HIKARI_ENV] = str(probe)
        try:
            escaped = _deliveries()
            # Resolve it too: attribute inspection alone cannot see the
            # `external` guard removed, because a workspace entry never sets
            # `manifest` — the guard lives in `resolve_manifest`.
            resolved_ws, resolved_src = resolve_manifest(escaped[0])
        finally:
            os.environ.pop(HIKARI_ENV, None)
    finally:
        globals()["DELIVERY_CRATES"] = saved_for_guard
        shutil.rmtree(probe, ignore_errors=True)
    if escaped[0].manifest is not None or escaped[0].external:
        print(
            f"FAIL a shipped workspace entry picked up a foreign manifest: {escaped[0]!r}",
            file=sys.stderr,
        )
        return 1
    if resolved_ws is not None or resolved_src != "workspace":
        print(
            f"FAIL a shipped workspace entry resolved to {resolved_ws!r} via "
            f"{resolved_src!r}, with {HIKARI_ENV} pointing elsewhere",
            file=sys.stderr,
        )
        return 1

    # A `Delivery` that names its own manifest is the "entry" route, and it must
    # never be reported as a sibling/env discovery. This is layout-independent:
    # unlike resolving a shipped crate (which needs a hikari checkout and would
    # raise on one machine and not another), `manifest=` resolves right here.
    here_dir = Path(tempfile.mkdtemp(prefix="cg-entry-route-"))
    (here_dir / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
    try:
        entry_dir, entry_src = resolve_manifest(
            Delivery("cg-entry-route", manifest=str(here_dir))
        )
    finally:
        shutil.rmtree(here_dir, ignore_errors=True)
    if entry_src != "entry":
        print(
            f"FAIL a manifest-bound entry was reported as {entry_src!r}",
            file=sys.stderr,
        )
        return 1

    # The `env` wire code is pinned too: a route that reports itself as another
    # one is how a "sibling" notice silently becomes an "env" one, and the hint
    # below is only trustworthy because the codes are distinct.
    env_probe = Path(tempfile.mkdtemp(prefix="cg-env-code-"))
    (env_probe / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
    saved_code_env = os.environ.get(HIKARI_ENV)
    try:
        os.environ[HIKARI_ENV] = str(env_probe)
        _, env_code = resolve_manifest(Delivery("hikari-animation", external=True))
    finally:
        if saved_code_env is None:
            os.environ.pop(HIKARI_ENV, None)
        else:
            os.environ[HIKARI_ENV] = saved_code_env
        shutil.rmtree(env_probe, ignore_errors=True)
    if env_code != "env":
        print(f"FAIL an env-supplied directory was reported as {env_code!r}", file=sys.stderr)
        return 1
    saved = os.environ.pop(HIKARI_ENV, None)
    try:
        os.environ[HIKARI_ENV] = "/nonexistent/hikari-by-env"
        try:
            _, source = resolve_manifest(Delivery("hikari-animation", external=True))
        except GateError:
            pass
        else:
            print(
                f"FAIL an external entry pointing at a missing {HIKARI_ENV} resolved anyway",
                file=sys.stderr,
            )
            return 1
        print("ok   unresolvable external entry raises instead of defaulting")
    finally:
        os.environ.pop(HIKARI_ENV, None)
        if saved is not None:
            os.environ[HIKARI_ENV] = saved

    # The provenance line is what makes "asserted a sibling instead of the
    # branch under review" visible, and a hint that names the wrong mechanism is
    # worse than none. Both are pinned by CAPTURING the line, not by grepping the
    # source: a source grep is satisfied by the f-string alone, so a print site
    # that drops the hint would keep it green (a reviewer proved exactly that).
    # `check` is driven here with a fake `cargo` whose `tree` prints a tiny clean
    # listing, so the whole banner path runs without invoking cargo.
    fake_bin = Path(tempfile.mkdtemp(prefix="cg-fake-cargo-"))
    fake_cargo = fake_bin / "cargo"
    fake_cargo.write_text(
        "#!/bin/sh\n"
        "printf 'alpha v1\\nbeta v2\\ngamma v3\\ndelta v4\\nepsilon v5\\n'\n",
        encoding="utf-8",
    )
    fake_cargo.chmod(0o755)
    probe_dir = Path(tempfile.mkdtemp(prefix="cg-probe-hikari-"))
    (probe_dir / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
    saved_entries = DELIVERY_CRATES
    saved_env = os.environ.get(HIKARI_ENV)
    try:
        globals()["DELIVERY_CRATES"] = (Delivery("cg-probe", external=True),)
        os.environ[HIKARI_ENV] = str(probe_dir)
        out, err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            rc_env = check(str(fake_cargo))
        env_stdout = out.getvalue()
        if rc_env != 0 or f"asserted cg-probe from {probe_dir}" not in env_stdout:
            print(
                f"FAIL the asserted directory is not reported (rc={rc_env}): {env_stdout!r}",
                file=sys.stderr,
            )
            return 1
        if "sibling match" in env_stdout or HIKARI_ENV in env_stdout:
            print(
                f"FAIL an env-supplied directory carried the sibling hint: {env_stdout!r}",
                file=sys.stderr,
            )
            return 1

        # Sibling route: no env var, and the resolved directory must announce
        # itself as a sibling match plus name the env var as the way to be
        # explicit. Both halves are what a reader needs to notice they are not
        # asserting the checkout they think they are.
        #
        # The candidate list is PATCHED to the probe directory rather than
        # relying on a real `../hikari` next to this checkout: gated on one being
        # present, this case would silently no-op on a machine without hikari and
        # a suppressed hint would keep `--selftest` green there (a reviewer
        # proved exactly that). Patching also keeps the case honest about what it
        # tests — the sibling *route*, not this machine's layout.
        os.environ.pop(HIKARI_ENV, None)
        saved_candidates = globals()["hikari_candidates"]
        try:
            globals()["hikari_candidates"] = lambda: (str(probe_dir),)
            out, err = io.StringIO(), io.StringIO()
            with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
                rc_sib = check(str(fake_cargo))
        finally:
            globals()["hikari_candidates"] = saved_candidates
        sib_stdout = out.getvalue()
        if rc_sib != 0 or f"asserted cg-probe from {probe_dir}" not in sib_stdout:
            print(
                f"FAIL the sibling route did not assert the probe directory "
                f"(rc={rc_sib}): {sib_stdout!r}",
                file=sys.stderr,
            )
            return 1
        if "sibling match" not in sib_stdout or HIKARI_ENV not in sib_stdout:
            print(
                f"FAIL the sibling route did not name its source: {sib_stdout!r}",
                file=sys.stderr,
            )
            return 1
        print("ok   the asserted hikari directory and its source are reported")

        # `--strict` must turn a skip into a non-zero exit. Driven through the
        # same fake cargo with an entry that cannot be located and no env var,
        # so this is the real surface, not a source grep.
        gone = Path(tempfile.mkdtemp(prefix="cg-gone-"))
        shutil.rmtree(gone)
        os.environ[HIKARI_ENV] = str(gone)
        out, err = io.StringIO(), io.StringIO()
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            rc_skip_lenient = check(str(fake_cargo))
            rc_skip_strict = check(str(fake_cargo), strict=True)
        if rc_skip_lenient != 0:
            print(
                f"FAIL a skip was not lenient by default (rc={rc_skip_lenient})",
                file=sys.stderr,
            )
            return 1
        if rc_skip_strict != EXIT_NOT_CHECKED:
            print(
                f"FAIL --strict did not fail a skipped entry (rc={rc_skip_strict}, "
                f"expected {EXIT_NOT_CHECKED})",
                file=sys.stderr,
            )
            return 1
        print("ok   --strict fails a skipped entry while the default stays lenient")

        # A `GateError` raised while normalising DELIVERY_CRATES must come back
        # as "could not run", not as a traceback (whose rc would collide with
        # "forbidden package found").
        globals()["DELIVERY_CRATES"] = (
            Delivery("cg-missing", manifest=str(gone)),
        )
        out, err = io.StringIO(), io.StringIO()
        escaped = None
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            try:
                rc_missing = check(str(fake_cargo))
            except GateError as exc:
                escaped = exc
        # The diagnostic is printed AFTER the redirect ends: inside it, the
        # message would land in the discarded buffer and the run would exit 1
        # with no `FAIL` line at all.
        if escaped is not None:
            print(
                f"FAIL a missing fixed manifest escaped check() as an exception: {escaped}",
                file=sys.stderr,
            )
            return 1
        if rc_missing != 2:
            print(f"FAIL a missing fixed manifest gave rc={rc_missing}, expected 2", file=sys.stderr)
            return 1
        print("ok   an invalid entry is reported as could-not-run, not a traceback")

    finally:
        globals()["DELIVERY_CRATES"] = saved_entries
        if saved_env is None:
            os.environ.pop(HIKARI_ENV, None)
        else:
            os.environ[HIKARI_ENV] = saved_env
        shutil.rmtree(fake_bin, ignore_errors=True)
        shutil.rmtree(probe_dir, ignore_errors=True)

    # The raw-text belt-and-braces re-run is a THIRD `cargo tree` per entry. Its
    # failure must come back as rc 2; escaping as a `GateError` would exit 1,
    # the code that means "a forbidden package was found". A fake cargo that
    # fails only from its third invocation is the only way to reach that call.
    third_bin = Path(tempfile.mkdtemp(prefix="cg-third-"))
    third_cargo = third_bin / "cargo"
    third_counter = third_bin / "n"
    # The counter lives in the CARGO_HOME-like path, not next to this probe, so
    # it counts calls within one `check()` run. Counting across the whole
    # selftest made this case order-dependent: with a hikari sibling present the
    # earlier cases consume calls first, so "fail from the third invocation"
    # fired inside the wrong case.
    third_cargo.write_text(
        "#!/bin/sh\n"
        f'n=$(cat {third_counter} 2>/dev/null || echo 0); n=$((n+1)); echo $n > {third_counter}\n'
        'case " $* " in *" -i "*) exit 0;; esac\n'
        'if [ "$n" -ge 3 ]; then echo boom >&2; exit 1; fi\n'
        "printf 'alpha v1\\nbeta v2\\ngamma v3\\ndelta v4\\nepsilon v5\\n'\n",
        encoding="utf-8",
    )
    third_cargo.chmod(0o755)
    try:
        globals()["DELIVERY_CRATES"] = ("cg-third",)
        third_counter.write_text("0", encoding="utf-8")
        out, err = io.StringIO(), io.StringIO()
        third_escaped = None
        with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            try:
                rc_third = check(str(third_cargo))
            except GateError as exc:
                third_escaped = exc
        if third_escaped is not None:
            print(
                f"FAIL a failing third `cargo tree` escaped as an exception: {third_escaped}",
                file=sys.stderr,
            )
            return 1
        if rc_third != 2:
            print(
                f"FAIL a failing third `cargo tree` gave rc={rc_third}, expected 2 "
                "(1 would read as a forbidden package)",
                file=sys.stderr,
            )
            return 1
        print("ok   a failing third `cargo tree` is could-not-run, not a finding")
    finally:
        globals()["DELIVERY_CRATES"] = saved_entries
        shutil.rmtree(third_bin, ignore_errors=True)

    # A fixed `manifest` plus `external` is two competing routes; refuse it. The
    # probe directory must be REAL, otherwise the neighbouring "does not exist"
    # guard fires first and the case stays green with this check deleted.
    conflict_dir = Path(tempfile.mkdtemp(prefix="cg-conflict-"))
    (conflict_dir / "Cargo.toml").write_text("[workspace]\n", encoding="utf-8")
    try:
        globals()["DELIVERY_CRATES"] = (
            Delivery("x-conflict", manifest=str(conflict_dir), external=True),
        )
        try:
            _deliveries()
        except GateError as exc:
            if "external" not in str(exc):
                print(
                    f"FAIL a manifest+external entry was refused for the wrong reason: {exc}",
                    file=sys.stderr,
                )
                return 1
        else:
            print("FAIL a manifest+external entry was accepted", file=sys.stderr)
            return 1
        print("ok   a manifest+external entry is refused")
    finally:
        globals()["DELIVERY_CRATES"] = saved_entries
        shutil.rmtree(conflict_dir, ignore_errors=True)

    print(f"selftest OK: {len(all_cases)} parse cases + 9 cross-workspace cases")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="run the gate")
    parser.add_argument("--selftest", action="store_true", help="run the parser selftest")
    parser.add_argument("--cargo", default=None, help="cargo binary (default: from PATH)")
    parser.add_argument(
        "--strict",
        action="store_true",
        help=(
            "treat an entry that could not be located as a failure (exit 2) "
            "instead of reporting it and continuing (exit 0)"
        ),
    )
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
    return check(cargo, strict=args.strict)


if __name__ == "__main__":
    sys.exit(main())