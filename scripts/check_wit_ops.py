#!/usr/bin/env python3
"""Op-level host-surface gate: every reachable dispatched op has a host implementation.

`check_wit_surface.py` answers the INTERFACE-level question ("is
`@tairitsu-glue/<iface>` registered?") and was blind to the hole that made this
module necessary: `window` was wired, and the host's `window` object exported
three ops — while `window.match-media` had no implementation at all. The
interface count said 25 wired / 8 allowed and read green; a guest calling
`match-media` got a silent `false`, so `prefers-reduced-motion` never suppressed
animation (hikari `animation/src/prefers_reduced_motion.rs` →
`AnimationProvider` → `enabled: props.enabled && !reduced_motion`).

Interface-level granularity cannot see a missing op inside an interface that is
wired. This gate adds that granularity, on the surface that actually dispatches.

WHAT "HAS AN IMPLEMENTATION" MEANS HERE (measured, not assumed)
================================================================
The runtime installs one table, `globalThis.__TAIRITSU_GLUE__.INTERFACES`, built
from `src/runtime/registry.ts`. Two mechanisms reference an op of a wired
interface:

1. **dispatch** — the emitted ESM module (`scripts/emit-glue-esm.mjs` feeds the
   packager's `<iface>.js`) calls `glue.INTERFACES[spec][key]`. If that key is
   absent the guest gets a `TypeError` — loud. This gate requires every op the
   emitted module dispatches to exist as a runtime export.
   (`packages/browser-glue/test/media-query.test.mjs` drives this module
   against fixtures, including a green-baseline guard and both mutation
   directions.)
2. **guest import** — a component built against `tairitsu-web` names every op of
   every interface it imports, so jco resolves all of them at instantiate. An op
   that only `src/glue/*.ts` exports (the generator output) and that the runtime
   table does NOT re-export is an import the page satisfies with `undefined`,
   and the guest's first call to it traps. Those ops are the honest remaining
   gap; they are listed with a reason in
   `packages/browser-glue/wit-op-allow.tsv`.
3. **guest export (the callback direction)** — the runtime also calls INTO the
   guest, through `globalThis.__wasmExports["<ns>/<iface>@<ver>"]?.<prop>(…)`.
   That namespace is the transpiled wrapper's export object, so `<prop>` must be
   the identifier jco generates for a WIT func — the camelCase of its kebab name
   (`on-mutate` → `onMutate`, `on-resize` → `onResize`; verified with
   `jco types`, and `timer-callbacks`' `onTimeout` is the precedent that shipped
   working). Directions 1 and 2 read the INTERFACES table and are structurally
   blind to this one: a wrong property name here does not throw, it evaluates to
   `undefined`, so the notification is dropped silently. That is why the
   `mutation` observer never told a guest anything.

   The jco camelCase must be at least one candidate. A snake_case fallback is
   allowed only as the second candidate of the same `??`, and only when it is
   that func's OWN snake spelling (`on-timeout` → `onTimeout ?? on_timeout`).
   `on_mutate` is not a spelling of `on-mutate` under any convention, so no
   bridge carries it. The value shape is
   NOT checked here (that needs the record types); the resize / mutation bridges
   carry their own assertions in
   `packages/browser-glue/test/callback-bridge.test.mjs`.

So the gate's two directions are deliberately different, because the two
mechanisms are:

* **dispatch** (the emitted modules derived from the runtime table) — anything
  not in the runtime table is unreachable, so no allowance list applies;
* **import** (the WIT declaration) — may be allowed, one line per op with a
  reason, and every allowance must name a WIT op that exists and is not
  implemented (an allowance for a wired op fails, so this list only shrinks).

Like `check_wit_surface.py`, this file is importable so the gate and the
browser-glue test suite run the same logic instead of two drifting copies.

THIS GATE'S SAFETY IS NOT LEXICAL COMPLETENESS
==============================================
Everything below reads source with regular expressions and one scanner state
machine; every shape they do not model is a blind spot (three of them are listed
just below, plus one shape that is not). What keeps the gate honest is `EXPECTED_BINDINGS_BY_FILE` plus the
base counts: a bridge the scan stops recognising shows up as a drop in that
table. So a NEW bridge must edit the table, and the residual risk is a
bridge that lands in a blind spot AND is hand-added to the table in the same
change. That risk is one commit wide and visible in review; a silent one is not.

Lexical blind spots, and what actually closes each:
  * a RECEIVER other than `globalThis`/`g` (`w.__wasmExports[...]`) — the binding
    pin: the binding disappears, so the per-file count drops and reds;
  * an unterminated string literal, which swallows the rest of the file — same:
    the bindings it eats are a drop in the pin;
  * `//` inside a regular-expression literal — CONDITIONAL: the pin only helps
    when the truncation lands on a line that carries a binding; elsewhere this is
    a read the scan does not see, and nothing reds;
  * a backtick nested in a template's `${ … }` — NOT a bypass: the scanner
    handles it byte-for-byte (the template branch re-syncs), so it is not listed
    as a risk any more.

Read that split as the rule for this gate: the binding pin guards that a bridge
EXISTS, the lexical scan guards that a READ is SEEN. Neither substitutes for the
other, so a shape that keeps the binding and hides the read is a finding even
when every count looks right.

Usage:
    python3 scripts/check_wit_ops.py --check   # the gate
    python3 scripts/check_wit_ops.py --list    # dispatch pairs, for diffing
"""
from __future__ import annotations

import argparse
import bisect
import os
import re
import sys
from dataclasses import dataclass
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
GLUE_PKG = REPO_ROOT / "packages" / "browser-glue"
REGISTRY = GLUE_PKG / "src" / "runtime" / "registry.ts"
RUNTIME_DIR = REGISTRY.parent
EMITTER = GLUE_PKG / "scripts" / "emit-glue-esm.mjs"
OP_ALLOW = GLUE_PKG / "wit-op-allow.tsv"
# `observers` is wired by the registry yet declared by no interface: not an op
# gap, a name gap between the runtime and the WIT. It gets its own inventory so
# the two failure modes stay distinguishable in review.
STRUCT_ALLOW = GLUE_PKG / "wit-interface-allow.tsv"
# Single source of truth for the composed world (`packages/web/wit/composed` is a
# generated copy, gated byte-for-byte by `sync_wit.py --check`).
WIT_COMPOSED = REPO_ROOT / "packages" / "browser-worlds" / "wit" / "composed"

# Floors, not targets (same rationale as check_wit_surface.py): an empty scan
# must read as "the scan broke", never as "nothing to check".
MIN_INTERFACES = 20
MIN_OPS = 50
MIN_DISPATCHED = 20
# Direction 3 (guest exports) has its own floor: a scan that stops matching must
# read as "the scan broke", exactly like the other two.
MIN_CALLBACK_SITES = 10

IMPORT_RE = re.compile(
    r"import\s*\{\s*([A-Za-z_$][A-Za-z0-9_$]*)\s*\}\s*from\s*\"\./([A-Za-z0-9_$]+)\""
)
REGISTRY_KEY_RE = re.compile(
    r'"@tairitsu-glue/([a-z0-9-]+)"\s*:\s*([A-Za-z_$][A-Za-z0-9_$]*)'
)
EMITTER_ENTRY_RE = re.compile(r'^\s*"?([a-z0-9-]+)"?\s*:\s*"([A-Za-z0-9_$]+)"', re.MULTILINE)
WIT_INTERFACE_RE = re.compile(r"^interface\s+([a-z0-9-]+)\s*\{", re.MULTILINE)
WIT_FUNC_RE = re.compile(r"^\s+([a-z0-9][a-z0-9-]*)\s*:\s*func\s*\(", re.MULTILINE)
# `__wasmExports["tairitsu-browser:full/<iface>@0.2.0"]?.<prop>` — the guest
# export the runtime calls. Deliberately anchored on `__wasmExports` (the
# transpiled wrapper's namespace installed by `setWasmExports`) so a lookup of
# some other object does not count as a bridge.
# Two notations, because the sources use both:
#   const guest = globalThis.__wasmExports?.["<ns>/<iface>@<ver>"];   // bound
#   <ns>/<iface>@<ver>"]?.<prop>(…)                                   // inline
# The key is matched on its own and parsed in a second step (`_CALLBACK_KEY_RE`)
# rather than with one megapattern: the first single-pattern draft matched one
# site and missed the rest (its bracket class swallowed the following text), and
# the MIN_CALLBACK_SITES floor is what turned that into a red gate instead of a
# quietly smaller report.
_CALLBACK_KEYED = re.compile(
    r'__wasmExports\s*\??\.?\[\s*"(?P<key>[^"]+)"\s*\]',
)
# The same lookup written with a template: `__wasmExports[`<ns>/<iface>@0.2.0`]`.
# Ignoring this form lets a bridge be rewritten into it and vanish from the
# report entirely — the floor only measures what the scan already sees, so a
# shrinking BASE has to be caught here, not there.
_CALLBACK_KEYED_TEMPLATE = re.compile(
    r"__wasmExports\s*\??\.?\[\s*`(?P<key>[^`$]+)`\s*\]",
)
# How far past a bound lookup the property reads may sit (lines). The bridges
# read and call within a handful; the bound-callback scan below uses it.
_WINDOW_LINES = 400

_CALLBACK_KEY_RE = re.compile(r"^(?P<ns>.*)/(?P<iface>[a-z0-9-]+)@(?P<ver>[0-9.]+)$")
# Right after the closing `]` the inline dispatch continues with `?.<prop>(` /
# `.<prop>(` — `\s*` only, never a newline: a newline means the lookup was bound
# to a local (`const exp = …`) and the call goes through that local.
# `\s*` may carry ONE newline: the inline form shipped at `origin/master` wraps
# the lookup across lines (`__wasmExports[\n  "…@0.2.0"\n]?.on_mutation?.(…)`),
# and a same-line-only scanner is blind to exactly the shape the defect shipped
# in — measured, it read green on a restored copy of it.
_DISPATCH_PROP_RE = re.compile(
    r"(?:(?:\s*\n?\s*)(?:\?\.|\.)(?:\s*\n?\s*))(?P<prop>[A-Za-z_$][A-Za-z0-9_$]*)(?:[^\n]*\()"
)


STRING_QUOTES = ('"', "'", "`")


def strip_comments(text: str) -> str:
    """Drop comments, skipping over string literals (their contents stay intact).

    Not "split on `//`": that eats a URL (`"ws://host"`) and leaves the rest of
    the line looking like code — the truncation shrinks what the scans see. A
    one-pass state machine is enough (it does not parse JS; it tracks the four
    things that change the meaning of a character: `"`, `'`, a backtick, `//`
    and `/* */`). The output keeps every newline and never grows, and every
    literal is copied verbatim, so indices and reported line numbers stay put.

    `normalise_casts` needs the same lexical boundary (`(guest as "a)b")`), which
    is why both read this one output.
    """
    out: list[str] = []
    index = 0
    length = len(text)
    while index < length:
        char = text[index]
        following = text[index + 1] if index + 1 < length else ""
        if char == "/" and following == "/":
            while index < length and text[index] != "\n":
                index += 1
            continue
        if char == "/" and following == "*":
            index += 2
            while index < length and not (text[index] == "*" and text[index + 1 : index + 2] == "/"):
                if text[index] == "\n":
                    out.append("\n")
                index += 1
            index += 2
            continue
        if char in STRING_QUOTES:
            # Copy the literal VERBATIM: this function's callers parse module
            # specifiers, interface keys and registry names out of exactly these
            # strings. Only the `//` rule was wrong (it ate `"ws://…"`); the fix
            # is to know where a string ENDS, not to erase it.
            quote = char
            out.append(quote)
            index += 1
            while index < length:
                if text[index] == "\\" and index + 1 < length:
                    out.append(text[index : index + 2])
                    index += 2
                    continue
                if text[index] == quote:
                    break
                out.append(text[index])
                index += 1
            if index < length:
                out.append(quote)
                index += 1
            continue
        out.append(char)
        index += 1
    return "".join(out)


def camel(kebab: str) -> str:
    """`match-media` → `matchMedia`; `get-onchange` → `getOnchange`.

    This is the exact mapping `emit-glue-esm.mjs` produces (named export names
    must be valid ESM identifiers) and the one the guest generator uses.
    """
    head, *rest = kebab.split("-")
    return head + "".join(part[:1].upper() + part[1:] for part in rest)


def wired_interfaces() -> dict[str, str]:
    """`@tairitsu-glue/<iface>` → the runtime module that defines its exports."""
    text = strip_comments(REGISTRY.read_text(encoding="utf-8", errors="replace"))
    start = re.search(r"export const INTERFACES\s*=\s*\{", text)
    if start is None:
        raise SystemExit("error: could not locate `export const INTERFACES = {` in registry.ts")
    body, sep, _rest = text[start.end():].partition("\n};")
    if not sep:
        raise SystemExit("error: could not find the end of the INTERFACES object in registry.ts")
    bindings = {symbol: module for symbol, module in IMPORT_RE.findall(text)}
    interfaces: dict[str, str] = {}
    for iface, symbol in REGISTRY_KEY_RE.findall(body):
        module = bindings.get(symbol)
        if module is None:
            raise SystemExit(
                f"error: registry maps @tairitsu-glue/{iface} to `{symbol}`, which this "
                "module cannot trace to an import — the registry shape changed"
            )
        interfaces[iface] = module
    return interfaces


def emitted_dispatchers() -> dict[str, str]:
    """The emitter's interface → runtime module map (what gets emitted as ESM)."""
    text = EMITTER.read_text(encoding="utf-8", errors="replace")
    start = text.find("const EXPORT_SOURCES = {")
    if start < 0:
        raise SystemExit("error: could not locate EXPORT_SOURCES in emit-glue-esm.mjs")
    body, sep, _rest = text[start:].partition("\n};")
    if not sep:
        raise SystemExit("error: could not find the end of EXPORT_SOURCES")
    return {iface: module for iface, module in EMITTER_ENTRY_RE.findall(body)}


def module_exports(module: str) -> set[str]:
    """Top-level keys of `export const <module>_exports = { … }`, read from source.

    Ported from `packages/browser-glue/scripts/emit-glue-esm.mjs::moduleExports`
    (the emitter is what actually ships these keys as named exports), and kept
    deliberately line-for-line equivalent because three cheaper rules were
    tried first and each leaked parameter names into the key set:

    * "identifier at depth 1" reads every parameter (`self`, `value`, …);
    * "…and it starts a line" fixes one-line parameter lists and breaks on a
      wrapped one (`setSelectionRange(\n self: bigint,`);
    * "…and it is followed by `(`/`:`/`,`" still reads `bigint` out of a
      multi-line type intersection (`child: Node & \n { someProp } & Something,`).

    The emitter's rule — a key is the pending text at depth 1 that ends a
    statement at paren depth 0 — handles all three, and
    `packages/browser-glue/test/media-query.test.mjs` runs this module over a
    fixture copy of the whole tree, so a scan that silently stops matching
    turns that suite red instead of shrinking the report.
    """
    path = RUNTIME_DIR / f"{module}.ts"
    if not path.is_file():
        raise SystemExit(f"error: registry names module {module!r} but {path} does not exist")
    source = path.read_text(encoding="utf-8", errors="replace")
    marker = f"export const {module}_exports = {{"
    start = source.find(marker)
    if start < 0:
        raise SystemExit(f"error: {path.name} does not export {module}_exports")

    keys: set[str] = set()

    def flush(pending: str) -> None:
        match = re.match(r"^[\"']?([A-Za-z_$][A-Za-z0-9_$]*)", pending.strip())
        if match:
            keys.add(match.group(1))

    depth = 0
    parens = 0
    pending = ""
    expect_key = True
    i = start + len(marker) - 1
    while i < len(source):
        char = source[i]
        nxt = source[i + 1] if i + 1 < len(source) else ""
        if char == "/" and nxt == "/":
            while i < len(source) and source[i] != "\n":
                i += 1
            continue
        if char == "/" and nxt == "*":
            i += 2
            while i < len(source) and not (source[i] == "*" and source[i + 1] == "/"):
                i += 1
            i += 2
            continue
        if char in "\"'`":
            quote = char
            i += 1
            while i < len(source):
                if source[i] == "\\":
                    i += 2
                    continue
                if source[i] == quote:
                    break
                i += 1
            i += 1
            continue
        # Paren depth is updated FIRST and on every character: parens track the
        # parameter list / call arguments, and a top-level `,` only ends a key
        # when we are outside one. (Gating the paren update inside the depth-1
        # branch is the bug that made every key read as `<name><first param>`:
        # the `(` after a method name was consumed by the `:`/`(` branch and
        # parens never left 0… then never came back.)
        if char == "(":
            parens += 1
            if depth == 1:
                expect_key = False
            i += 1
            continue
        if char == ")":
            parens -= 1
            i += 1
            continue
        if char == "{":
            opened_object = depth == 0
            depth += 1
            if opened_object:
                pending = ""
                expect_key = True
            i += 1
            continue
        if char == "}":
            depth -= 1
            if depth == 0:
                break
            i += 1
            continue
        if depth == 1:
            if char in (",", "\n") and parens <= 0:
                flush(pending)
                pending = ""
                expect_key = True
                i += 1
                continue
            if expect_key:
                pending += char
            if char == ":":
                expect_key = False
        i += 1
    flush(pending)
    return keys


def declared_funcs(iface: str) -> list[str]:
    """The `func` names `interface <iface>` declares in the composed WIT.

    The declarations come from the SINGLE SOURCE (`browser-worlds`); the copy
    under `packages/web/wit/composed` is a generated artifact gated by
    `sync_wit.py --check`. Reading only one side is deliberate: a union over
    both would let an op that exists in just one copy satisfy the gate, so a
    forgotten sync plus an op added to one side would read green.
    """
    for path in sorted(WIT_COMPOSED.glob("*.wit")):
        text = path.read_text(encoding="utf-8", errors="replace")
        for match in WIT_INTERFACE_RE.finditer(text):
            if match.group(1) != iface:
                continue
            body = text[match.end():]
            return WIT_FUNC_RE.findall(body[: body.find("\n}")])
    return []


# Files that mention the namespace without reading a guest export: the ambient
# declaration and the installer.
#
# WHAT AN EXCLUSION MEANS: the file is outside EVERY callback scan — it cannot
# contribute a binding, and a bridge added to it would be invisible. That is why
# the gate prints the list on every run (`excluded_scan_files`) instead of
# trusting a reader to look here: the cost of the exception stays in view.
# There is no third state — a file is scanned or it is excluded, and both scans
# and the per-file floor read the same list (`callback_scan_files`).
CALLBACK_SCAN_EXCLUDED = ("global.d.ts", "wasmExports.ts")


def skipped_scan_files() -> list[str]:
    """Runtime files the callback scan does not read, and why — printed by the gate.

    The ONLY function that walks every runtime file on purpose: reporting what was
    skipped requires seeing what was skipped. A silent exclusion list is where a
    bridge gets hidden, so the gate names every file the scan skipped on every run.
    """
    skipped = []
    for path in sorted(RUNTIME_DIR.glob("*.ts")):
        if path.name in CALLBACK_SCAN_EXCLUDED:
            skipped.append(path.name)
            continue
        text = strip_comments(path.read_text(encoding="utf-8", errors="replace"))
        if "__wasmExports" not in text:
            skipped.append(path.name)
    return skipped


def excluded_scan_files() -> list[str]:
    """The exclusion list, as actually present on disk.

    A name that no longer exists (a rename) is dropped here — the exclusion is
    then simply dead, which is what an exclusion of a missing file should mean.
    The gate prints both this list and how many runtime files never touch the
    namespace, so "excluded" never reads as "does not exist".
    """
    present = {path.name for path in RUNTIME_DIR.glob("*.ts")}
    return [name for name in CALLBACK_SCAN_EXCLUDED if name in present]


def callback_scan_files() -> list[Path]:
    """The runtime sources both callback scans read — ONE list, so an exclusion
    really takes a file out of the scan instead of only out of one of them.

    (It did not, once: the per-file floor honoured the list and direction 3b did
    not, so excluding `platformHelpers.ts` left `88 (47 bound)` untouched and the
    gate read green while the docstring promised the opposite.)
    """
    return [
        path
        for path in sorted(RUNTIME_DIR.glob("*.ts"))
        if path.name not in CALLBACK_SCAN_EXCLUDED
    ]


# The per-file binding IDENTITY, pinned in the gate (not only in a test): `just
# ci` runs `wit-ops-check` on its own, so a pin that lives in the test suite does
# not guard the gate anyone actually runs. A file losing a binding to a refactor
# the scan does not understand (`Reflect.get`, a renamed local) must be a RED
# GATE with a message saying which file and how many it lost — not a slightly
# smaller report.
EXPECTED_BINDINGS_BY_FILE = {
    "eventTarget.ts": 1,
    "mediaQueryList.ts": 1,
    "platformHelpers.ts": 45,
}


def binding_identity_problems(counted_by_file: dict[str, int]) -> list[str]:
    """The per-file binding counts must match `EXPECTED_BINDINGS_BY_FILE`."""
    problems: list[str] = []
    for name, expected in sorted(EXPECTED_BINDINGS_BY_FILE.items()):
        actual = counted_by_file.get(name, 0)
        if actual == expected:
            continue
        problems.append(
            f"{name}: expected {expected} guest-object bindings, the scan found {actual} — a "
            "bridge stopped being recognised (a refactor the scan does not understand) or a "
            "new one was added; update EXPECTED_BINDINGS_BY_FILE with the reason"
        )
    for name, actual in sorted(counted_by_file.items()):
        if actual and name not in EXPECTED_BINDINGS_BY_FILE:
            problems.append(
                f"{name}: {actual} guest-object bindings in a file this list does not cover — "
                "add it to EXPECTED_BINDINGS_BY_FILE with the reason"
            )
    return problems


def per_file_binding_problems(counted_by_file: dict[str, int]) -> list[str]:
    """Every runtime file that TOUCHES `__wasmExports` must yield a binding.

    The scan recognises three shapes (direct, bound, helper-returned), each by
    regex. An ordinary refactor — an arrow-function helper, a guarded helper body,
    a renamed local — can make a whole file produce ZERO bindings while the global
    floor stays green at 47 (measured: `mediaQueryList.ts` drops out and a wrong
    read inside it reads clean again). Worse, a file could keep a handful of
    bindings and lose the rest.

    So the floor is judged on the SAME numbers direction 3b used
    (`counted_by_file` comes from `bound_bridge_problems`), per file: zero
    bindings in a file that mentions the namespace is a red gate. It notices that
    the SHAPE CHANGED; it does not try to parse arrow functions or nested braces,
    which would drag this regex gate toward a JS parser.
    """
    problems: list[str] = []
    for path in callback_scan_files():
        text = strip_comments(path.read_text(encoding="utf-8", errors="replace"))
        if "__wasmExports" not in text:
            continue
        if counted_by_file.get(path.name, 0) > 0:
            continue
        problems.append(
            f"{path.name} references `__wasmExports` but the scan produced 0 bindings — "
            "the bridge shape changed (an arrow-function helper, a guarded helper body, a "
            "renamed local) or the scanner broke; check guest_bindings"
        )
    return problems


def _all_callback_lookups(text: str) -> list[re.Match[str]]:
    """Every `__wasmExports[...]` lookup, string-keyed or template-keyed."""
    found = [*_CALLBACK_KEYED.finditer(text), *_CALLBACK_KEYED_TEMPLATE.finditer(text)]
    return sorted(found, key=lambda match: match.start())


def callback_sites() -> tuple[list[tuple[str, int, str, str, str]], int]:
    """The guest-export dispatch sites, plus how many lookups were scanned.

    Returns `([(module, line, interface, version, property), …], lookups)`.

    Two notations, and this pass names only the first:

    * **inline** — `<key>"]?.<prop>(…`, on one line (`"<key>"` or `` `<key>` ``).
      Named here, with the property;
    * **bound** — `const guest = …["<key>"];` … `guest?.<prop> ?? guest?.<snake>`.
      NOT named here: the call goes through a local, so the property sits away
      from the lookup. `bound_bridge_problems` names those sites, which is why
      the number of lookups (both notations) is returned: the floor guards the
      BASE, so a rewrite that shrinks what the scan sees is red even when the
      remaining sites are all clean.

    In the tree as it stands there are no inline lookups and 88 lookups in total
    (47 of them bound), so
    the inline branch of `callback_problems` is a guard for the shape the original
    defect was written in rather than a live producer of findings — kept, and
    pinned by `media-query.test.mjs`'s "WRAPPED inline shape" control.

    An interface whose key is not `<ns>/<iface>@<ver>` is skipped — this check is
    about the browser world's callbacks.
    """
    sites: list[tuple[str, int, str, str, str]] = []
    lookups = 0
    for path in callback_scan_files():
        source = path.read_text(encoding="utf-8", errors="replace")
        text = strip_comments(source)
        starts = [0]
        for offset, char in enumerate(text):
            if char == "\n":
                starts.append(offset + 1)
        for lookup in _all_callback_lookups(text):
            keyed = _CALLBACK_KEY_RE.match(lookup.group("key"))
            if keyed is None:
                continue
            lookups += 1
            line_number = bisect.bisect_right(starts, lookup.start())
            bracket_end = text.find("]", lookup.end("key"))
            rest = _DISPATCH_PROP_RE.match(text, bracket_end + 1)
            if rest is None:
                continue
            sites.append(
                (
                    path.name,
                    line_number,
                    keyed.group("iface"),
                    keyed.group("ver"),
                    rest.group("prop"),
                )
            )
    return sites, lookups


# ONE definition per shape, used by every scan: `bound_binding_files` and
# `bound_bridge_problems` used to carry separate copies of `bound_re`, which is
# the "same shape implemented twice" drift this file has been bitten by before.
_BOUND_RE = re.compile(
    r"(?<![.A-Za-z0-9_$])const\s+(?P<local>[A-Za-z_$][A-Za-z0-9_$]*)\s*=\s*\n?"
    r"\s*(?:globalThis|g)\.__wasmExports\??\.?\[\s*[\"`](?P<key>[^\"`$]+)[\"`]\s*\]"
)
# A helper that hands the guest namespace back:
#   function callbacks() { return globalThis.__wasmExports?.["<key>"]; }
# matched as a UNIT so the helper NAME and the KEY stay paired.
_HELPER_DECL_RE = re.compile(
    r"function\s+(?P<name>[A-Za-z_$][\w$]*)\s*\([^)]*\)[^{]*\{[^}]{0,400}?"
    r"__wasmExports\s*\??\.?\[\s*[\"`](?P<key>[^\"`$]+)[\"`]",
    re.DOTALL,
)
_HELPER_CALL_RE = re.compile(
    r"const\s+(?P<local>[A-Za-z_$][\w$]*)\s*=\s*(?P<helper>[A-Za-z_$][\w$]*)\(\)"
)


def guest_bindings(text: str) -> list[tuple[str, str, int]]:
    """`(local, key, offset)` for every way a runtime file reaches the guest.

    Three shapes, all resolved here so no two scans can drift apart:

    * `const guest = globalThis.__wasmExports?.["<key>"]` — direct;
    * `const guest = helper()` where `function helper() { … __wasmExports["<key>"] … }`
      — the helper NAME and its KEY must be the SAME pair (a call to one helper
      and a declaration of another used to count as a hit);
    * both tolerate a template-literal key.

    A key that is not a `<ns>/<iface>@<ver>` callback key is skipped: this
    direction is about the browser world, not about WASI.
    """
    bindings: list[tuple[str, str, int]] = []
    for match in _BOUND_RE.finditer(text):
        bindings.append((match.group("local"), match.group("key"), match.start()))
    helpers = {
        match.group("name"): match.group("key") for match in _HELPER_DECL_RE.finditer(text)
    }
    for call in _HELPER_CALL_RE.finditer(text):
        key = helpers.get(call.group("helper"))
        if key is not None:
            bindings.append((call.group("local"), key, call.start()))
    return [
        (local, key, offset)
        for local, key, offset in bindings
        if _CALLBACK_KEY_RE.match(key) is not None
    ]


# `(local as T)` — and `((local as any))`, which leaves `(local)` behind: strip
# plain wrapping parentheses too, or a double cast reduces to `(local)` and the
# access pattern (which starts at the local) never sees it.
_WRAPPED_RE = re.compile(r"\(\s*(?P<core>[A-Za-z_$][\w$]*)\s*\)")
_ACCESS_RE = re.compile(
    r"(?<![A-Za-z0-9_$])(?P<local>[A-Za-z_$][A-Za-z0-9_$]*)"
    r"(?:\?\.\s*(?P<prop>[A-Za-z_$][A-Za-z0-9_$]*)"
    r"|\??\.?\[\s*[\"'](?P<bracket>[A-Za-z_$][A-Za-z0-9_$]*)[\"']\s*\])"
)


def parens_balanced(line: str) -> bool:
    """`(`/`)` counts match, ignoring what is inside a string literal."""
    depth = 0
    quote = None
    index = 0
    while index < len(line):
        char = line[index]
        if quote:
            if char == "\\" and index + 1 < len(line):
                index += 2
                continue
            if char == quote:
                quote = None
            index += 1
            continue
        if char in STRING_QUOTES:
            quote = char
        elif char == "(":
            depth += 1
        elif char == ")":
            depth -= 1
            if depth < 0:
                return False
        index += 1
    return depth == 0 and quote is None


def normalise_casts(line: str) -> str:
    """Reduce a cast expression to the local at its core: `(local as T)` → `local`.

    Rewriting every type syntax in a regex is a losing game — a white-list misses
    `,`, nesting and function types, and each miss is a new blind spot (`(guest as
    Record<string, any>)`, `((guest as any))`, `(guest as Fn<(a: T) => U>)`). What
    actually separates the cast from the access around it is PARENTHESIS DEPTH, so
    the innermost balanced group is popped while it starts with `<local> as`, and
    plain wrapping groups are dropped as well.
    """
    previous = None
    while previous != line:
        previous = line
        # Walk INNERMOST groups only (depth 1 → 0), then re-scan: `((guest as any))`
        # has no depth-1 group that starts with the cast, so an outer-first loop
        # would never see it.
        for start, index in _group_spans(line):
            inner = line[start + 1 : index]
            # `as T` and `satisfies T` are both type-level operators that sit
            # between the local and its access. Reducing only `as` leaves
            # `(guest satisfies any)?.on_mutate` with a type name where the
            # receiver should be — the read then vanishes and nothing notices
            # (the binding pin guards EXISTENCE, not that a read was seen).
            core = re.fullmatch(
                r"\s*(?P<core>[A-Za-z_$][\w$]*)\s+(?:as|satisfies)\s+.*", inner, re.S
            )
            if core:
                line = line[:start] + core.group("core") + line[index + 1 :]
                break
        line = _WRAPPED_RE.sub(lambda match: match.group("core"), line)
    return line


def _group_spans(line: str) -> list[tuple[int, int]]:
    """`(start, end)` of every balanced parenthesis group, closing paren included.

    Brackets INSIDE a string literal do not count: `(guest as "a)b")` has one
    group, not one group cut in half. The mask keeps index positions, so the
    spans apply to the original line.
    """
    masked = []
    quote = None
    index = 0
    while index < len(line):
        char = line[index]
        if quote:
            if char == "\\" and index + 1 < len(line):
                masked.append("  ")
                index += 2
                continue
            if char == quote:
                quote = None
            masked.append(" ")
            index += 1
            continue
        if char in STRING_QUOTES:
            quote = char
            masked.append(" ")
            index += 1
            continue
        masked.append(char)
        index += 1
    spans: list[tuple[int, int]] = []
    stack: list[int] = []
    for position, char in enumerate("".join(masked)):
        if char == "(":
            stack.append(position)
        elif char == ")" and stack:
            spans.append((stack.pop(), position))
    return spans


def bound_bridge_problems(
    declared_by_iface: dict[str, list[str]], counted_by_file: dict[str, int] | None = None
) -> tuple[list[str], int]:
    """Direction 3b: a lookup bound to a local must be CALLED through that local.

    The inline form is named by `callback_sites`; the bound form
    (`const guest = …["<key>"];` … `guest?.<prop>(…)`) is not, so without this the
    whole `platformHelpers.ts` rewrite could be reverted to `guest?.on_wrong` and
    stay green. Every binding comes from `guest_bindings` — the one place that
    knows the three shapes. A second copy of that knowledge is how the file-level
    floor and this scan drifted apart. `counted_by_file`, when passed, receives
    the per-file binding counts so the floor judges on THIS scan's numbers.
    """
    problems: list[str] = []
    bindings = 0
    # `<local>?.<prop>` / `<local>.<prop>` ONLY: a looser pattern scans for a bare
    # identifier, and a preceding `??` supplies one, so the property list comes
    # back empty and a wrong name reads green.
    # Two read notations, both real in these sources:
    #   guest?.onMutate        — member access;
    #   (guest as any)?.["on_mutate"] — computed access, which is how a "quick"
    #                                    fix would put a wrong name back without
    #                                    tripping anything else. Both are checked
    #                                    against the same declared set.
    # Not covered (a JS tokenizer, not this gate): destructuring
    # (`const { onX } = guest`) and template-literal prop names.
    # `(guest as any)?.["on_mutate"]` is the shape a "quick" fix produces: a cast
    # sits between the local and the access, so the optional `as <Type>` group is
    # not decoration — without it the bracket branch never matches this form.
    read_re = _ACCESS_RE
    for path in callback_scan_files():
        # Whole-line comments are dropped BEFORE the window is cut: a comment that
        # mentions a bad spelling is not a read of it.
        source = path.read_text(encoding="utf-8", errors="replace")
        stripped_source_lines = strip_comments(source).split("\n")
        kept_positions = [
            index
            for index, line in enumerate(stripped_source_lines)
            if line.strip() and not line.strip().startswith("//")
        ]
        lines = [stripped_source_lines[index] for index in kept_positions]
        text = "\n".join(lines)
        file_bindings = guest_bindings(text)
        if counted_by_file is not None:
            counted_by_file[path.name] = len(file_bindings)
        # A window ends at the enclosing block, so it can end mid-nesting; balance
        # is judged on the FILE. If a file does not balance, every binding in it
        # is reported rather than silently judged on a broken frame.
        file_balanced = parens_balanced(text)
        for local, key_text, offset in file_bindings:
            keyed = _CALLBACK_KEY_RE.match(key_text)
            if keyed is None:
                continue
            bindings += 1
            iface = keyed.group("iface")
            filtered_index = text[:offset].count("\n")
            previous = lines[filtered_index - 1].strip() if filtered_index > 0 else ""
            wrapped = bool(re.fullmatch(r"const\s+[A-Za-z_$][\w$]*\s*=", previous))
            # The property may be read several lines below a wrapped lookup (a doc
            # comment sits between them), so the read line is the first line at or
            # after the binding that mentions the local.
            if not file_balanced:
                problems.append(
                    f"callback site {path.name}:{kept_positions[filtered_index] + 1} binds "
                    f"`{local}` to `{iface}@{keyed.group('ver')}` but the file's parentheses do "
                    "not balance — the read scan cannot judge it, so it is reported rather "
                    "than skipped"
                )
                continue
            read_line_index = kept_positions[filtered_index]
            for probe_index in range(filtered_index, min(filtered_index + 12, len(lines))):
                if re.search(
                    r"(?<![A-Za-z0-9_$])" + re.escape(local) + r"\s*\?\.", lines[probe_index]
                ):
                    read_line_index = kept_positions[probe_index]
                    break
            declaration_line_index = (
                kept_positions[filtered_index - 1] if wrapped else kept_positions[filtered_index]
            )
            declared = declared_by_iface.get(iface) if iface in declared_by_iface else declared_funcs(iface)
            if not declared:
                # The inline pass only sees the INLINE form, so a bound lookup
                # into an undeclared interface would otherwise be lost.
                if iface not in callback_allowances():
                    problems.append(
                        f"callback site {path.name}:{declaration_line_index + 1} binds a guest export of "
                        f"`{iface}@{keyed.group('ver')}`, which the composed WIT does not "
                        "declare — no guest can receive it"
                    )
                continue
            accepted = {camel(name) for name in declared}
            # The window runs to the end of the enclosing block (the binding's
            # indentation minus four), not a fixed line count: the first draft
            # missed `eventTarget.ts`'s five dispatches 30–80 lines below.
            base_indent = len(lines[filtered_index]) - len(lines[filtered_index].lstrip())
            tail = text[offset:].split("\n")
            window = tail[0] + "\n"
            for probe in tail[1 : 1 + _WINDOW_LINES]:
                if _BOUND_RE.search(probe) or _HELPER_CALL_RE.search(probe):
                    break
                indent = len(probe) - len(probe.lstrip())
                if probe.strip() and probe.strip().startswith("}") and (
                    indent <= base_indent - 4 or probe == "};"
                ):
                    break
                window += probe + "\n"

            for read in read_re.finditer(normalise_casts(window)):
                if read.group("local") != local:
                    continue
                prop = read.group("prop") or read.group("bracket")
                bracket_read = read.group("bracket") is not None
                if prop in accepted:
                    continue
                # NO fallback is accepted: jco emits exactly one identifier per
                # func, so a second candidate can never run. (Four rounds of the
                # rule taught that the hard way — neighbour check, spelling check,
                # alias table, chain parse — before the alias table turned out to
                # be empty for every declared name.)
                shown = f'{local}["{prop}"]' if bracket_read else f"{local}.{prop}"
                problems.append(
                    f"callback site {path.name}:{read_line_index + 1} binds `{local}` to "
                    f"`{iface}@{keyed.group('ver')}` and reads `{shown}`, which the "
                    f"guest does not export (declared: {', '.join(declared)}) — the callback "
                    "is dropped with no error"
                )
            # "Used": a binding that is stored and never mentioned stays dead weight.
            names = {local}
            for assigned in re.finditer(
                r"const\s+(?P<name>[A-Za-z_$][A-Za-z0-9_$]*)\s*=\s*[^;]*\b"
                + re.escape(local)
                + r"\b[^;]*;",
                window,
            ):
                names.add(assigned.group("name"))
            named = (
                r"(?<![.A-Za-z0-9_$])(?:"
                + "|".join(re.escape(name) for name in sorted(names))
                + r")"
            )
            stored = tail[0].strip() != ""
            used = (
                bool(re.search(named + r"\s*(?:[,(]|\.|\?\.)", window))
                if stored
                else bool(re.search(named, window))
            )
            if not used:
                problems.append(
                    f"callback site {path.name}:{read_line_index + 1} binds `{local}` to "
                    f"`{iface}@{keyed.group('ver')}` and never uses it — the bridge is dead weight"
                )
    return problems, bindings


def callback_problems() -> tuple[list[str], int]:
    """Direction 3: the property a host→guest callback site reads must exist.

    A site passes when the property is the jco camelCase of one of the
    interface's declared funcs, or that func's snake spelling. An interface the
    composed WIT does not declare at all is its own finding: it names a bridge
    nothing on the other side can implement (the historical defect here was a
    `resize-callbacks` namespace that does not exist, reading `on_resize`, while
    the real interface is `window-resize-callbacks` / `on-window-resize`).
    """
    problems: list[str] = []
    sites, lookups = callback_sites()
    for module, line, iface, ver, prop in sites:
        declared = declared_funcs(iface)
        if not declared:
            # Same discipline as the op allow-list: a line in
            # `wit-interface-allow.tsv` is the only way to say "known, not fixed",
            # so the exception is visible in review instead of implicit.
            if iface not in callback_allowances():
                problems.append(
                    f"callback site {module}:{line} dispatches into `{iface}@{ver}`, which the "
                    "composed WIT does not declare — no guest can receive it, and no "
                    "wit-interface-allow.tsv line explains it. The real interface may have a "
                    "different name (check `window-resize-callbacks`)"
                )
            continue
        accepted = {camel(name) for name in declared}
        if prop not in accepted:
            problems.append(
                f"callback site {module}:{line} reads `{prop}` from `{iface}@{ver}`, which the "
                f"guest does not export (declared: {', '.join(declared)}; jco emits "
                f"{camel(declared[0])}, …) — the callback is dropped with no error"
            )
    return problems, lookups


def copy_problems() -> list[str]:
    """The composed WIT copies must agree, or this gate is reading one side only.

    `check_wit_ops.py` trusts the single source. If the generated copy drifted,
    the numbers this gate prints no longer describe the world `tairitsu-web` is
    compiled against (build.rs prefers the embedded copy), so the drift is a
    finding here rather than somebody else's gate to catch.
    """
    mirror = REPO_ROOT / "packages" / "web" / "wit" / "composed"
    if not mirror.is_dir():
        return [f"the composed WIT mirror {mirror} is missing — cannot cross-check"]
    problems = []
    source_files = sorted(p.name for p in WIT_COMPOSED.glob("*.wit"))
    mirror_files = sorted(p.name for p in mirror.glob("*.wit"))
    for name in sorted(set(source_files) ^ set(mirror_files)):
        problems.append(f"composed WIT copies disagree on file {name} (run sync_wit.py)")
    for name in source_files:
        if name not in mirror_files:
            continue
        a = (WIT_COMPOSED / name).read_text(encoding="utf-8", errors="replace")
        b = (mirror / name).read_text(encoding="utf-8", errors="replace")
        if a != b:
            problems.append(f"composed WIT copies differ in {name} (run sync_wit.py --check)")
    return problems


def op_allowances() -> dict[str, str]:
    """`<iface>.<kebab-op>` → reason, from `wit-op-allow.tsv` (both directions)."""
    entries: dict[str, str] = {}
    if not OP_ALLOW.exists():
        return entries
    for raw in OP_ALLOW.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        parts = raw.split("\t")
        if len(parts) < 2 or not parts[1].strip():
            raise SystemExit(f"error: op allowance line lacks a reason: {raw!r}")
        key = parts[0].strip()
        if "." not in key:
            raise SystemExit(f"error: op allowance must be <iface>.<op>: {raw!r}")
        entries[key] = parts[1].strip()
    return entries


CALLBACK_ALLOW = GLUE_PKG / "wit-callback-allow.tsv"


def callback_allowances() -> dict[str, str]:
    """`<kebab-interface>` → reason, for guest exports the WIT does not declare.

    Its own file next to the other two: "the interface is not declared",
    "the op is not implemented for a wired interface" and "this bridge calls an
    interface no world declares" are three different defects with three
    different fixes, and folding any two together hides the third.
    """
    entries: dict[str, str] = {}
    if not CALLBACK_ALLOW.exists():
        return entries
    for raw in CALLBACK_ALLOW.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        parts = raw.split("\t")
        if len(parts) < 2 or not parts[1].strip():
            raise SystemExit(f"error: callback allowance line lacks a reason: {raw!r}")
        entries[parts[0].strip()] = parts[1].strip()
    return entries


def structural_allowances() -> dict[str, str]:
    """`<wired-but-undeclared interface>` → reason, from `wit-interface-allow.tsv`."""
    entries: dict[str, str] = {}
    if not STRUCT_ALLOW.exists():
        return entries
    for raw in STRUCT_ALLOW.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        parts = raw.split("\t")
        if len(parts) < 2 or not parts[1].strip():
            raise SystemExit(f"error: structural allowance line lacks a reason: {raw!r}")
        entries[parts[0].strip()] = parts[1].strip()
    return entries


@dataclass(frozen=True)
class Finding:
    kind: str
    key: str
    detail: str

    def __str__(self) -> str:
        if self.kind == "unreachable":
            return (
                f"{self.key}: the interface's emitted module exports `{self.detail}` but the "
                "runtime INTERFACES table has no such key, so a guest call through the "
                "emitted module throws TypeError"
            )
        if self.kind == "unimplemented":
            return (
                f"{self.key}: declared by WIT for a wired interface, the runtime does not "
                f"export `{self.detail}`, and there is no allowance — a guest importing it "
                "traps on first call"
            )
        if self.kind == "invented":
            return (
                f"{self.key}: the runtime exports `{self.detail}` but the WIT interface "
                f"declares no such func (an invented op is as bad as a missing one)"
            )
        if self.kind == "declared-not-table":
            return (
                f"{self.key}: declared by WIT but the runtime module {self.detail!r} does not "
                "export it — the emitted module would build a reference to a missing export"
            )
        if self.kind == "stale-allowance":
            return f"stale allowance: {self.key} is allowed but {self.detail}"
        if self.kind == "callback-mismatch":
            # The detail is the whole sentence (it names the file:line and the
            # declared set), so it is returned as-is.
            return self.key
        if self.kind == "no-declaration":
            return (
                f"{self.key}: wired by the registry but the composed WIT declares no such "
                "interface — the declaration scan and the wiring disagree"
            )
        return f"{self.kind}: {self.key} ({self.detail})"


def findings() -> tuple[list[Finding], int, int, int, int, int]:
    """(findings, wired interfaces, declared ops, dispatched ops, callback lookups, bound callbacks)."""
    interfaces = wired_interfaces()
    emitters = emitted_dispatchers()
    allow = op_allowances()
    struct_allow = structural_allowances()

    results: list[Finding] = []
    declared_total = 0
    dispatched_total = 0
    covered: set[str] = set()

    for iface, module in sorted(interfaces.items()):
        declared = declared_funcs(iface)
        if not declared:
            if iface not in struct_allow:
                results.append(Finding("no-declaration", iface, module))
            continue
        try:
            exported = module_exports(module)
        except SystemExit as error:
            results.append(Finding("declared-not-table", iface, str(error)))
            continue
        declared_total += len(declared)
        declared_camel = {camel(name): name for name in declared}

        # 1. dispatch: whichever runtime module the emitter ships for this
        #    interface must exist in the table (it is the module the registry
        #    names once both agree — and they must agree).
        emitter_module = emitters.get(iface)
        if emitter_module is not None and emitter_module != module:
            results.append(
                Finding(
                    "unreachable",
                    f"{iface}.*",
                    f"{emitter_module}_exports",
                )
            )
            continue
        for key in sorted(exported):
            if key not in declared_camel:
                # The runtime really exports this; the WIT does not declare it.
                # Guest-unreachable, but still listed one by one (see the file).
                if f"{iface}.{key}" not in allow:
                    results.append(Finding("invented", f"{iface}.{key}", key))
                continue
            dispatched_total += 1
            covered.add(f"{iface}.{declared_camel[key]}")

        # 2. import: every WIT op of a wired interface resolves at instantiate.
        for name in declared:
            op = f"{iface}.{name}"
            if op in covered:
                continue
            if op in allow:
                continue
            results.append(Finding("unimplemented", op, camel(name)))

    for name in sorted(callback_allowances()):
        # The line must still describe a real bridge AND a really undeclared
        # interface, or it has served its purpose and must shrink.
        if declared_funcs(name):
            results.append(
                Finding(
                    "stale-allowance",
                    f"callback-allow:{name}",
                    "the composed WIT declares this interface now — drop the line",
                )
            )
        elif not any(
            # A BOUND lookup into an undeclared interface is reported by
            # `bound_bridge_problems`, never by `callback_sites` (which only names
            # the inline form) — so the staleness test has to look at the raw
            # lookups, or a legitimate allowance for a bound site reads as stale.
            name == _CALLBACK_KEY_RE.match(lookup.group("key")).group("iface")
            for path in callback_scan_files()
            for lookup in _all_callback_lookups(
                strip_comments(path.read_text(encoding="utf-8", errors="replace"))
            )
            if _CALLBACK_KEY_RE.match(lookup.group("key"))
        ):
            results.append(
                Finding(
                    "stale-allowance",
                    f"callback-allow:{name}",
                    "no callback site dispatches into it any more",
                )
            )

    for name in sorted(struct_allow):
        if name not in interfaces:
            results.append(
                Finding("stale-allowance", name, "not wired by the registry any more")
            )
        elif declared_funcs(name):
            results.append(
                Finding("stale-allowance", name, "the composed WIT declares it now — drop the line")
            )

    # Allowances must describe a real, still-unwired WIT op.
    wired = set(interfaces)
    for key in sorted(allow):
        iface, _, op = key.partition(".")
        if iface not in wired:
            results.append(
                Finding("stale-allowance", key, f"interface {iface!r} is not wired at all")
            )
        elif op not in declared_funcs(iface):
            # Either an invented-export allowance (key IS a runtime export) or a
            # line naming an op that does not exist at all.
            if op not in module_exports(interfaces[iface]):
                results.append(
                    Finding(
                        "stale-allowance",
                        key,
                        f"neither a WIT func nor a runtime export of {iface!r}",
                    )
                )
        elif key in covered:
            results.append(
                Finding("stale-allowance", key, "the op IS implemented now — drop the line")
            )

    # 3. guest exports: the host→guest callback property must be a name jco
    #    emits. Same list, because a wrong name there is the same class of hole
    #    (a call the page silently swallows) seen from the other side.
    callback_findings, callback_total = callback_problems()
    for problem in callback_findings:
        results.append(Finding("callback-mismatch", problem, ""))
    declared_by_iface = {iface: declared_funcs(iface) for iface in interfaces}
    counted_by_file: dict[str, int] = {}
    bound_findings, bound_total = bound_bridge_problems(declared_by_iface, counted_by_file)
    for problem in per_file_binding_problems(counted_by_file):
        results.append(Finding("callback-mismatch", problem, ""))
    for problem in binding_identity_problems(counted_by_file):
        results.append(Finding("callback-mismatch", problem, ""))
    # The per-file binding identity, on request: a pin compares this map, so a
    # bridge that stops being recognised shows up as a missing entry rather than
    # as "one fewer binding somewhere".
    if os.environ.get("TAIRITSU_WIT_OPS_BINDINGS"):
        for name in sorted(counted_by_file):
            if counted_by_file[name]:
                print(f"BINDINGS {name}:{counted_by_file[name]}")
    for problem in bound_findings:
        results.append(Finding("callback-mismatch", problem, ""))

    return results, len(interfaces), declared_total, dispatched_total, callback_total, bound_total


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--check", action="store_true", help="enforce")
    group.add_argument("--list", action="store_true", help="print the dispatch pairs")
    args = parser.parse_args()

    if args.list:
        for iface, module in sorted(wired_interfaces().items()):
            for key in sorted(module_exports(module)):
                print(f"{iface}\t{key}\t{module}")
        return 0

    (
        mismatches,
        interface_count,
        declared_total,
        dispatched_total,
        callback_total,
        bound_total,
    ) = findings()
    problems = [str(f) for f in mismatches]
    problems.extend(copy_problems())

    # A gate that passes on an empty set passes when it is broken.
    if interface_count < MIN_INTERFACES:
        problems.append(
            f"registry scan found only {interface_count} wired interfaces "
            f"(< {MIN_INTERFACES}); the scan stopped matching — check REGISTRY_KEY_RE"
        )
    if declared_total < MIN_OPS:
        problems.append(
            f"WIT scan found only {declared_total} declared ops (< {MIN_OPS}); the scan "
            f"stopped matching — check WIT_FUNC_RE and {WIT_COMPOSED}"
        )
    if dispatched_total < MIN_DISPATCHED:
        problems.append(
            f"dispatch scan found only {dispatched_total} dispatched ops "
            f"(< {MIN_DISPATCHED}); check module_exports and the INTERFACES block"
        )
    if callback_total < MIN_CALLBACK_SITES:
        problems.append(
            f"callback scan found only {callback_total} guest-export lookups "
            f"(< {MIN_CALLBACK_SITES}); the scan stopped matching — check _CALLBACK_KEYED"
        )
    if bound_total < MIN_CALLBACK_SITES:
        problems.append(
            f"bound-callback scan found only {bound_total} bound guest exports "
            f"(< {MIN_CALLBACK_SITES}); check bound_bridge_problems"
        )

    if problems:
        print("WIT host-surface op gate failed:", file=sys.stderr)
        for problem in problems:
            print(f"  - {problem}", file=sys.stderr)
        return 1

    allow = op_allowances()
    wired = wired_interfaces()
    unimplemented = 0
    invented = 0
    for key in allow:
        iface, _, op = key.partition(".")
        if iface in wired and op in declared_funcs(iface):
            unimplemented += 1
        else:
            invented += 1
    print(
        f"WIT host surface ops OK: {interface_count} wired interfaces, "
        f"{declared_total} declared ops "
        f"({declared_total - unimplemented} implemented, {unimplemented} allowed), "
        f"{dispatched_total} dispatched, {invented} runtime-only exports listed, "
        f"{callback_total} guest-export lookups ({bound_total} bound) named correctly, "
        f"callback scan excluded {', '.join(excluded_scan_files()) or 'nothing'} "
        f"(plus {len(skipped_scan_files()) - len(excluded_scan_files())} runtime files that "
        "never touch the guest namespace)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
