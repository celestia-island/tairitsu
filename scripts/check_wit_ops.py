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
machine; every shape they do not model is a blind spot. What keeps the gate
honest is three layers, not one: `EXPECTED_BINDINGS_BY_FILE` plus the base counts
(a bridge the scan stops recognising shows up as a drop in that table), the
per-file floor, and — for the shapes that keep the binding and hide the read —
`unparseable_read_problems`, which reports the SHAPE instead of judging a name it
cannot see. So a NEW bridge must edit the table, and the residual risk is a
bridge that lands in a blind spot AND is hand-added to the table in the same
change. That risk is one commit wide and visible in review; a silent one is not.

Lexical blind spots, and what actually closes each:
  * a RECEIVER other than `globalThis`/`g` (`w.__wasmExports[...]`) — the binding
    pin: the binding disappears, so the per-file count drops and reds;
  * an unterminated string literal, which swallows the rest of the file — same:
    the bindings it eats are a drop in the pin;
  * `//` inside a regular-expression literal — CONDITIONAL, and the condition is
    measured rather than assumed. `/a\/\/b/` does NOT truncate: the two `\/`
    sequences never form `//`, so a read written after it on the same line is
    still seen (pinned in `media-query.test.mjs`). `/[//]/` DOES: `strip_comments`
    reads the `//` inside the character class as a comment and drops the rest of
    that line, so a read there is not seen — and the binding is untouched, so the
    pin does not fire either. The truncation is line-bounded (a read on the next
    line is still seen). Left as a residual rather than "fixed": deciding whether
    a `/` opens a regex literal or divides needs expression context (`a / b / c`),
    and any state machine that guesses would trade this silence for a different
    one (the wrong guess eats a real comment's line instead);
  * a backtick nested in a template's `${ … }` — NOT a bypass: the scanner
    handles it byte-for-byte (the template branch re-syncs), so it is not listed
    as a risk any more;
  * a property name the source does not spell as a name — CLOSED fail-closed,
    five spellings, all reported by `unparseable_read_problems` (the declared set
    cannot supply a verdict about a name the source never spells as a name, so the
    SHAPE is the finding):
      - destructured: `const { onX } = guest` in any of its ordinary TS spellings
        (plain, `: any` / `: Record<string, () => void>` annotation, `= guest as
        any` / `satisfies T`, `= guest!`), `const [onX] = guest` (a dead read —
        object destructured by index), and `({ onX }) => …` in a file that binds
        the interface;
      - subscripted with a template (`` guest[`${k}`] ``);
      - subscripted with any computed key — variable, call, index, or a
        concatenation that starts with a quote (`guest["on" + dyn]`);
      - read off a parenthesised EXPRESSION receiver (`(0, guest)?.onX`,
        `(flag || guest)?.onX`);
      - read behind a cast whose core is a member (`(guest.sub as any)?.onX`, and
        the nastier `(guest.onMutate as any)?.onWrong`, where the member is
        declared and the read after the cast would otherwise red nothing);
      - and a RE-EXPORT of the guest is reported at the declaration, because every
        read written through it is invisible to the read scan: a plain copy, an
        arrow that returns it (`() => guest`), a fallback whose operand can be it
        (`guest || {}`), or a spread copy (`{ ...guest }`).
    Three residuals inside this class: a destructuring pattern nested more than
    one level (`const { a: { onX } } = guest`) is not matched — 0 occurrences, and
    a second level on a flat object of functions is not a shape a bridge can
    produce; the PARAMETER form is anchored on the interfaces the file binds, so
    `({ onX }) => …` naming an op of an interface this file does not bridge is not
    reported (anchoring it on the whole world reded on unrelated code — `item` is
    a `dom-token-list` op); and a guest object handed in as a function PARAMETER
    (`function f(cb) { cb?.onX }`), returned by a NAMED function
    (`function getGuest() { return guest; }; getGuest()?.onX`) or reached through a
    member is not a source, because only file-level bindings and the six
    initialiser spellings above can be proven — 0 occurrences. A read taken off a
    CALL's result, plain or optional (`identity(guest)?.onX`,
    `identity?.(guest)?.onX`), is deliberately not judged: it is a property of the
    call's return value, not of the guest, and it is pinned green.

Read that split as the rule for this gate: the binding pin guards that a bridge
EXISTS, the lexical scan guards that a READ is SEEN, and a shape neither can see
is a finding. Neither substitutes for the other, so a shape that keeps the
binding and hides the read is a finding even when every count looks right.

Usage:
    python3 scripts/check_wit_ops.py --check      # the gate
    python3 scripts/check_wit_ops.py --list       # dispatch pairs, for diffing
    python3 scripts/check_wit_ops.py --selftest   # the lexical layer, on shapes
"""
from __future__ import annotations

import argparse
import bisect
import os
import re
import sys
from dataclasses import dataclass
from functools import lru_cache
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


@lru_cache(maxsize=1)
def world_declarations() -> dict[str, list[str]]:
    """Every `interface → declared funcs` pair in the composed WIT.

    One reader for the whole world, so `declared_funcs` (one interface) and the
    guest-export scans (all of them) cannot drift into two parsers. The
    declarations come from the SINGLE SOURCE (`browser-worlds`); the copy under
    `packages/web/wit/composed` is a generated artifact gated by
    `sync_wit.py --check`. Reading only one side is deliberate: a union over both
    would let an op that exists in just one copy satisfy the gate, so a forgotten
    sync plus an op added to one side would read green.
    """
    declarations: dict[str, list[str]] = {}
    for path in sorted(WIT_COMPOSED.glob("*.wit")):
        text = path.read_text(encoding="utf-8", errors="replace")
        for match in WIT_INTERFACE_RE.finditer(text):
            body = text[match.end():]
            declarations.setdefault(
                match.group(1), WIT_FUNC_RE.findall(body[: body.find("\n}")])
            )
    return declarations


def declared_funcs(iface: str) -> list[str]:
    """The `func` names `interface <iface>` declares (see `world_declarations`)."""
    return world_declarations().get(iface, [])


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
_WRAPPED_RE = re.compile(r"[(]\s*(?P<core>[A-Za-z_$][\w$]*)\s*[)]")


# Keywords that take a parenthesised OPERAND rather than being called.
_OPERAND_KEYWORDS = frozenset(
    {
        "if", "for", "while", "switch", "catch", "with", "return", "typeof",
        "void", "delete", "new", "in", "of", "instanceof", "await", "yield",
        "throw", "case", "do", "else", "function", "as", "satisfies",
    }
)


def _in_call_position(line: str, start: int) -> bool:
    """Is the `(` at `start` a CALL's argument list rather than a wrapper?

    Decided by the previous non-space character: an identifier, `)` or `]` means
    "call of the thing before it". Both users of this test — the cast reducer and
    the wrapper stripper — REPLACE the group with its core, and doing that in call
    position fuses the argument into whatever follows:
    `callbacks.onGenericEvent(evtType)` became `callbacks.onGenericEventevtType`
    and the gate reported a property that exists nowhere in the source (measured,
    a false rejection of correct code). A cast wrapper always follows `=`, `(`,
    `,`, an operator, a keyword or a line start, so nothing legitimate is lost.
    """
    head = line[:start].rstrip()
    if not head:
        return False
    if head.endswith("?.") or head.endswith("?"):
        # OPTIONAL CALL `f?.(x)` / `o.m?.(x)`: the `?.` is the optional-call
        # marker, not an access, and missing it turned `identity?.(guest)?.onX`
        # into `identity?.guest?.onX` — a property the source never writes.
        head = head.rstrip(".").rstrip("?").rstrip()
        if not head:
            return False
    if head[-1] in ")]":
        return True
    if not (head[-1].isalnum() or head[-1] in "_$"):
        return False
    # A KEYWORD before the `(` is an operand slot, not a callee: `void (0, guest)`,
    # `return (guest as any)`, `if (x)`. Missing that made `void (…)` count as a
    # call, so the expression-receiver rule skipped the very shape it exists for
    # (measured) — and it also left `return (guest as any)?.onX` unreduced.
    word = re.search(r"([A-Za-z_$][\w$]*)$", head)
    return word is not None and word.group(1) not in _OPERAND_KEYWORDS


def _strip_wrapper(match: re.Match[str]) -> str:
    """Drop one wrapping paren pair, unless it is a call's argument list."""
    if _in_call_position(match.string, match.start()):
        return match.group(0)
    return match.group("core")
# A read of the guest object, in the two spellings the sources use and the one
# they would get if somebody edited the bridge by hand:
#
#   guest?.onMutate        — optional member access
#   guest.onMutate         — plain member access (NO `?`). This is what
#                            `eventTarget.ts` actually ships for all five of its
#                            event bridges (`callbacks.onMouseEvent` &c), and the
#                            member branch used to require `?.` — so those five
#                            reads were never judged, and a wrong name there would
#                            have read GREEN with every count intact. The `\??` is
#                            that hole, measured on the tree, not hypothetical.
#   guest["on_mutate"]     — computed access (both `?.[` and `[`), which is how a
#                            "quick" fix puts a wrong name back without tripping
#                            anything else.
_ACCESS_RE = re.compile(
    r"(?<![A-Za-z0-9_$])(?P<local>[A-Za-z_$][A-Za-z0-9_$]*)"
    r"(?:\??\.\s*(?P<prop>[A-Za-z_$][A-Za-z0-9_$]*)"
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
    plain wrapping groups are dropped as well. Each of those three shapes has an
    end-to-end control in `media-query.test.mjs`: a wrong name behind it reds and
    the right name stays green.

    A cast whose core is NOT a bare identifier — `(guest.sub as any)?.onX` — is
    deliberately NOT reduced: there is no single local to put back, and guessing
    which identifier in the core is the receiver is how a scan starts inventing
    reads. It is not silent either: the member read inside the cast
    (`guest.sub`) is judged like any other read, so such a cast reds as an
    undeclared member name. Loud and slightly misnamed beats green and wrong.
    """
    previous = None
    while previous != line:
        previous = line
        # Walk INNERMOST groups only (depth 1 → 0), then re-scan: `((guest as any))`
        # has no depth-1 group that starts with the cast, so an outer-first loop
        # would never see it.
        for start, index in _group_spans(line):
            if _in_call_position(line, start):
                # A cast ARGUMENT (`f(guest as any)`): reducing it would fuse the
                # argument into the callee's name exactly like the wrapper case.
                continue
            inner = line[start + 1 : index]
            # `as T` and `satisfies T` are both type-level operators that sit
            # between the local and its access. Reducing only `as` leaves
            # `(guest satisfies any)?.on_mutate` with a type name where the
            # receiver should be — the read then vanishes and nothing notices
            # (the binding pin guards EXISTENCE, not that a read was seen).
            # `!` (a non-null assertion) sits between the local and the cast
            # operator and changes nothing about which local it is:
            # `(guest! as any)?.onX` reduces like `(guest as any)?.onX`.
            core = re.fullmatch(
                r"\s*(?P<core>[A-Za-z_$][\w$]*)\s*!?\s+(?:as|satisfies)\s+.*", inner, re.S
            )
            if core:
                line = line[:start] + core.group("core") + line[index + 1 :]
                break
        line = _WRAPPED_RE.sub(_strip_wrapper, line)
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


# Reads whose property name is NOT an identifier in the source, so `_ACCESS_RE`
# has nothing to compare against the declared set:
#
#   const { onMutate } = guest;        // destructured: a property KEY
#   guest?.[`${name}`]                 // computed: run-time TEXT
#
# Both keep the binding (so the per-file pin is satisfied) and hide the read (so
# the lexical scan sees nothing): measured on the tree as it stood, the two
# shapes occur ZERO times across the 33 scanned files, and a wrong read written
# in either form read green. They are therefore reported, not judged — the
# declared set cannot supply a verdict about a name that is not written as a
# name. That is the fail-closed half of this gate: `EXPECTED_BINDINGS_BY_FILE`
# guards that a bridge EXISTS, the scan guards that a read is SEEN, and a shape
# neither can see is a finding rather than a smaller report.
# The pattern may carry a type annotation (`const { onX }: any = guest`) — a
# destructuring declaration with an explicit type is ordinary TS, and requiring
# `}` to be followed immediately by `=` read it as "no destructuring at all".
# The annotation may itself contain `=` — a function type (`: Record<string, () =>
# void>`) — so the separator is "`=` that is not part of `=>`", not "the next `=`".
_DESTRUCTURE_DECL_RE = re.compile(
    r"(?:const|let|var)\s*\{(?P<names>[^{}]*)\}\s*(?::(?:[^=]|=(?=>))*)?=\s*"
    r"(?P<rhs>[^\n;]*)"
)
# `(obj) => …` / `({ onMutate }) => …` — the guest object is the CALLER's
# argument, so there is no initialiser to anchor on and the declared set is the
# only handle: a parameter pattern that names a declared op is reported.
_DESTRUCTURE_PARAM_RE = re.compile(r"\(\s*\{(?P<names>[^{}]*)\}\s*(?::[^)]*)?\)\s*=>")
# `const [onX] = guest` — array destructuring of the namespace binds by INDEX, so
# the guest, being an object, yields `undefined`: a dead read with no name to judge
# and nothing else reds (measured rc 0). Same family as the object pattern above,
# reported for the same reason. Separate pattern because the delimiter pair differs
# and one regex for both would mis-match `[{a}, {b}]` shapes.
_DESTRUCTURE_ARRAY_RE = re.compile(
    r"(?:const|let|var)\s*\[(?P<names>[^\[\]]*)\]\s*(?::[^=]*)?=\s*(?P<rhs>[^\n;]*)"
)
_TEMPLATE_KEY_READ_RE = re.compile(
    r"(?<![A-Za-z0-9_$])(?P<local>[A-Za-z_$][A-Za-z0-9_$]*)\s*\??\.?\s*\[\s*`"
)
# The same read written with a variable key: `guest[key]`. A QUOTED key
# (`guest["onMutate"]`) is not this shape — `_ACCESS_RE` judges it — and a numeric
# index is not a guest export, so only an identifier subscript lands here.
# A subscript whose key is NOT one quoted identifier: `guest[key]`,
# `guest[keys.pop()]`, `guest[k[0]]`, `guest["on" + dyn]`. The last one is why the
# rule keys on "not a simple quoted identifier" instead of "unquoted": a
# concatenation that STARTS with a quote fell between the quoted-key scan (which
# wants the whole bracket to be one literal) and this rule (which wanted no quote
# at all), so it was judged by neither and reported by neither — measured silent.
# A backtick key is excluded: the template rule above reports it, and one read
# must not produce two findings.
_BRACKET_KEY_READ_RE = re.compile(
    r"(?<![A-Za-z0-9_$])(?P<local>[A-Za-z_$][A-Za-z0-9_$]*)\s*\??\.?\s*\[\s*"
    r"(?P<key>(?![\`])[^\]\n]+?)\s*\]"
)
# The one bracket form `_ACCESS_RE` JUDGES (and therefore must not be reported
# here): `local["onMutate"]`.
_JUDGED_BRACKET_KEY = re.compile(r"[\"']([A-Za-z_$][A-Za-z0-9_$]*)[\"']")
# A cast whose CORE is a member of the guest: `(guest.sub as any)`,
# `(guest["onMutate"] as any)`, `(guest.sub as Fn<(a: T) => U>)`. `normalise_casts`
# reduces only a single-identifier core, so the cast stays and whatever is read
# after it (`?.onX`) is invisible — including the case where the member itself IS
# a declared export, where nothing else reds. Reported like the other two shapes.
# A receiver that is a PARENTHESISED EXPRESSION ending in the guest local:
# `(0, guest)?.onX`, `(flag || guest)?.onX`, `(a ?? guest)?.onX`. `_ACCESS_RE` needs
# the local immediately followed by `?.`/`.`/`[`, so the `)` in between makes the
# read invisible; and it is not a cast wrapper, so nothing reduces it. Reported,
# not collapsed: collapsing the group to its tail identifier would claim the value
# IS the guest, which `a || guest` does not promise.
_EXPR_RECEIVER_RE = re.compile(
    r"\((?P<inner>[^()\n]*?)(?P<local>[A-Za-z_$][A-Za-z0-9_$]*)\s*\)\s*(?:\?\.|\.|\[)"
)
# `const g = guest;` — an alias of a guest local. Reads through the alias are
# invisible to the read scan (that scan keys on the locals `guest_bindings`
# proves), and nothing else notices, so the ALIAS ITSELF is the finding.
_ALIAS_RE = re.compile(
    r"(?:const|let|var)\s+(?P<name>[A-Za-z_$][\w$]*)\s*(?::[^=]*)?=\s*(?P<rhs>[^\n;]*)"
)
# `[^\n(]` (not `[^\n()]`): one redundant pair of parentheses around the member
# (`((guest.onX) as any)?.onY`) is ordinary formatting, and refusing to cross the
# `)` made the whole shape invisible again. The read-after requirement is applied
# separately (`_casts_before_a_read`) — a cast that IS the value
# (`(guest.onX as unknown as MutationCallback)`) is not a hidden read, and
# reporting it was a false positive on correct code.
_MEMBER_CAST_RE = re.compile(
    r"\(\s*(?P<local>[A-Za-z_$][A-Za-z0-9_$]*)\s*[.\[][^\n(]*?\s+(?:as|satisfies)\s+"
)


def _destructured_names(pattern: str) -> list[str]:
    """The property names a pattern binds, from the text BETWEEN the braces.

    `" onMutate: fn, ...rest "` → `["onMutate", "...rest"]`: the key is what the
    guest is asked for (the local binding after `:` is the caller's business), a
    `...rest` keeps its dots so the report says what it is, and a computed key
    (`[expr]`) is returned as written rather than guessed at.

    Splitting on `,` is text, not parsing: a default value that itself contains a
    comma (`onX = pick(a, b)`) splits too, so each part is cut at its first `:`/`=`
    and any fragment that follows an unclosed `(` is dropped instead of being
    reported as a key.
    """
    names: list[str] = []
    depth = 0
    for part in pattern.split(","):
        # A `,` inside a default value's call (`onX = pick(a, b)`) is not a key
        # separator: skip fragments while an earlier part left a paren open.
        head = re.split(r"[:=]", part.strip(), maxsplit=1)[0].strip() if depth == 0 else ""
        depth += part.count("(") - part.count(")")
        if not head:
            continue
        match = re.match(r"(?P<rest>\.\.\.)?\s*(?P<name>[A-Za-z_$][\w$]*)", head)
        if match:
            names.append(f"{match.group('rest') or ''}{match.group('name')}")
        elif head.startswith("["):
            names.append(head)
    return names


def _bare_guest_source(reduced: str, sources: list[str]) -> str | None:
    """The proved source a whole expression IS (or a zero-argument call of one)."""
    if re.match(r"(?:globalThis\s*\.\s*|g\s*\.\s*)?__wasmExports\b", reduced):
        return "__wasmExports"
    for source in sources:
        if re.match(
            re.escape(source) + r"\s*\(\s*\)\s*(?=$|[;,)]|\?\?)", reduced
        ) or re.match(re.escape(source) + r"\s*(?=$|[;,)]|\?\?)", reduced):
            return source
    return None


def _guest_source(rhs: str, sources: list[str], _depth: int = 0) -> str | None:
    """The guest source an initialiser hands over, or None.

    Six forms, and they all end up holding the guest namespace at run time — so
    all six have to be recognised or an alias written any of those ways slips
    past the alias rule entirely (measured: the last three read rc 0 while the
    plain copy was reported):

    * `= guest`, `= guest ?? {}`, `= callbacks()` — a proved source;
    * `= globalThis.__wasmExports?.["<key>"]` — the direct lookup;
    * `= guest as any` / `satisfies T` / `guest!` — assertions, stripped first;
    * `= guest || {}` / `= {} || guest` — a fallback whose operand can be it;
    * `= () => guest` — an arrow that returns it;
    * `= { ...guest }` — a spread copy carrying every op it spreads.

    `sources` (built in `unparseable_read_problems`) holds the locals
    `guest_bindings` resolved plus the helper functions whose key is a CALLBACK
    interface the composed WIT declares, so a helper returning a WASI namespace
    is not a source. A NAMED function that returns the guest
    (`function getGuest() { return guest; }`) is not a source either: its call
    looks exactly like any other call, and resolving that needs a call graph —
    listed as a residual instead.
    """
    if _depth > 3:
        return None
    reduced = normalise_casts(rhs).strip()
    # The assertion spellings sit between the source and the end of the
    # initialiser and change nothing about WHICH object was handed over:
    #   const { onX } = guest as any;   const { onX } = guest satisfies T;
    #   const { onX } = guest!;
    reduced = re.sub(r"\s+(?:as|satisfies)\s+.*$", "", reduced).strip()
    reduced = re.sub(r"!\s*$", "", reduced).strip()
    bare = _bare_guest_source(reduced, sources)
    if bare is not None:
        return bare
    # `() => guest` — the arrow's body is the value.
    arrow = re.match(r"(?:async\s+)?(?:\([^()]*\)|[A-Za-z_$][\w$]*)\s*=>\s*(?P<body>.+)$", reduced, re.S)
    if arrow:
        found = _guest_source(arrow.group("body"), sources, _depth + 1)
        if found is not None:
            return found
    # `guest || {}` — either operand can be the guest at run time.
    operands = re.split(r"\s*(?:\|\||\?\?)\s*", reduced)
    if len(operands) > 1:
        for operand in operands:
            found = _guest_source(operand, sources, _depth + 1)
            if found is not None:
                return found
    # `{ ...guest }` — an object literal that spreads it.
    spread = re.fullmatch(r"\{\s*(?P<body>.*?)\s*\}", reduced, re.S)
    if spread:
        for piece in spread.group("body").split(","):
            piece = piece.strip()
            if not piece.startswith("..."):
                continue
            found = _guest_source(piece[3:], sources, _depth + 1)
            if found is not None:
                return found
    return None


def _casts_before_a_read(line: str, cast_start: int, cast_end: int) -> bool:
    """Is something READ after the cast that starts at `cast_start`?

    A cast is only interesting to this gate when it sits BETWEEN a guest local and
    a further access (`(guest.onX as any)?.onWrong`): that is what puts the read
    out of the lexical scan's reach. A cast that IS the value
    (`const cb = (guest.onMutate as unknown as MutationCallback);`) hides nothing
    — the member read is already judged on its own — and reporting it was a false
    positive on correct code (measured).
    """
    # The group that ENCLOSES the cast, not the first `(` of the match: one
    # redundant pair around the member (`((guest.onX) as any)?.onY`) makes those
    # two different parens, and keying on the match's own `(` found the inner one
    # and then looked for a read in ` as any)…`.
    for start, end in _group_spans(line):
        if start <= cast_start and end >= cast_end:
            return re.match(r"\s*(?:\?\.|\.|\[)", line[end + 1 :]) is not None
    # An unbalanced line: no closing paren to inspect. Judging it is the slot the
    # binding pin owns, so stay silent here rather than guess.
    return False


def unparseable_read_problems() -> tuple[list[str], int]:
    """Direction 3c: guest reads written in a shape the member-access scan cannot see.

    Five shapes, each hiding a read behind syntax instead of a name:

    * DESTRUCTURING from a guest source — `const { onX } = guest` binds a property
      KEY, with or without a type annotation (`: any`), an assertion
      (`= guest as any` / `satisfies T`) or a non-null assertion (`= guest!`). The
      initialiser is the anchor, so EVERY name is reported, right or wrong: a
      misspelled key is exactly the defect this gate exists for and is not in the
      declared set. (Not matched: a pattern nested more than one level —
      `const { a: { onX } } = guest`, `const [{ onX }] = [guest]`. The guest
      namespace is a flat object of functions, so a second level is not a shape a
      bridge can produce; 0 occurrences, and the report would fire on the outer
      key anyway if the pattern were one level shallower.)
    * DESTRUCTURING in a parameter list — `({ onX }) => …`. There is no
      initialiser, so the anchor is the guest interfaces THIS FILE binds:
      `guest_bindings` names them, their declared funcs give the camel names, and
      a pattern naming one of those is reported. Anchoring on the whole world
      instead would report `({ item }) => …` in any file with a bridge, because
      `item` is a `dom-token-list` op — measured, and exactly the false positive
      this narrowing removes.
    * a TEMPLATE-LITERAL subscript — `guest[`${k}`]`: the key is run-time text.
    * a bare-IDENTIFIER subscript — `guest[key]`: a quoted key (`guest["onX"]`) is
      judged by `_ACCESS_RE` and is NOT this shape.
    * a CAST whose core is a member of the guest — `(guest.sub as any)?.onX`.
      `normalise_casts` cannot reduce it (there is no single local to put back),
      and if the member happens to be a declared export (`(guest.onMutate as
      any)?.onWrong`) nothing else reds at all.

    The template key, the variable key and the member cast are per line, with the
    same cast reduction the member scan uses:
    the "quick fix" spellings (`(guest as any)?.[`on_mutate`]`) separate the local
    from its subscript otherwise, and reducing a single line cannot move a line
    number, so the reported line stays the real one.

    Returns the findings and how many shapes were reported. Zero findings on a
    tree that is otherwise clean is the normal state, so this has no floor: the
    report is what closes the hole, and `media-query.test.mjs` pins every shape by
    injecting one and requiring this gate to exit 1.
    """
    problems: list[str] = []
    reported = 0
    declarations = world_declarations()
    for path in callback_scan_files():
        text = strip_comments(path.read_text(encoding="utf-8", errors="replace"))
        bindings = guest_bindings(text)
        if not bindings:
            continue
        starts = [0]
        for offset, char in enumerate(text):
            if char == "\n":
                starts.append(offset + 1)
        # The guest sources this file can destructure or subscript. A helper name
        # counts only when its key is a CALLBACK interface the composed WIT
        # declares: `wasi:cli/exit@0.2.0` matches the key SHAPE (ns/iface@ver), so
        # shape alone let `function wasiStatus()` be treated as a guest source and
        # reported `const { status } = wasiStatus()` as a destructured guest read
        # (measured). `guest_bindings` keeps its shape-only filter on purpose —
        # naming an UNDECLARED interface there is its own finding — but a source
        # this scan reports against has to be a real guest namespace.
        helpers = {
            match.group("name")
            for match in _HELPER_DECL_RE.finditer(text)
            if (keyed := _CALLBACK_KEY_RE.match(match.group("key")))
            and keyed.group("iface") in declarations
        }
        sources = sorted(
            {local for local, _key, _offset in bindings} | helpers, key=len, reverse=True
        )
        # The camel names a destructured PARAMETER may use: the declared funcs of
        # the guest interfaces this file binds.
        param_names = {
            camel(name)
            for _local, key, _offset in bindings
            for name in declarations.get(_CALLBACK_KEY_RE.match(key).group("iface"), [])
        }

        binding_locals = {local for local, _key, _offset in bindings}
        for match in _ALIAS_RE.finditer(text):
            name = match.group("name")
            if name in binding_locals:
                # The binding itself (`const guest = …__wasmExports[…]`) — every
                # bridge has one, and they are not aliases of anything.
                continue
            if _guest_source(match.group("rhs"), sources) is None:
                continue
            reported += 1
            problems.append(
                f"callback site {path.name}:{bisect.bisect_right(starts, match.start())} "
                f"re-exports the guest as `{name}` — every read written through that local is "
                "invisible to this gate (the read scan keys on the locals it can prove ARE the "
                "guest namespace), so the re-export itself is reported; read the guest local "
                "directly instead of copying, spreading, falling back to or returning it"
            )

        for match in _DESTRUCTURE_DECL_RE.finditer(text):
            source = _guest_source(match.group("rhs"), sources)
            if source is None:
                continue
            names = ", ".join(_destructured_names(match.group("names"))) or "(empty)"
            reported += 1
            problems.append(
                f"callback site {path.name}:{bisect.bisect_right(starts, match.start())} "
                f"destructures `{names}` out of `{source}` — a destructured guest export is a "
                "read this gate cannot check (the property is a key, not a member access), so "
                f"it is reported instead of skipped; read `{source}.<name>` and let the scan "
                "judge the name (if a local object merely shares this name, rename that local — "
                "scope analysis is out of reach here, so the shape is reported either way)"
            )

        for match in _DESTRUCTURE_ARRAY_RE.finditer(text):
            source = _guest_source(match.group("rhs"), sources)
            if source is None:
                continue
            names = ", ".join(_destructured_names(match.group("names"))) or "(empty)"
            reported += 1
            problems.append(
                f"callback site {path.name}:{bisect.bisect_right(starts, match.start())} "
                f"array-destructures `{names}` out of `{source}` — array destructuring binds by "
                "INDEX, and the guest namespace is an object, so every binding is `undefined` and "
                "no property is read at all; read the property you meant as a member access"
            )

        for match in _DESTRUCTURE_PARAM_RE.finditer(text):
            named = [
                name
                for name in _destructured_names(match.group("names"))
                if name in param_names
            ]
            if not named:
                continue
            reported += 1
            problems.append(
                f"callback site {path.name}:{bisect.bisect_right(starts, match.start())} "
                f"destructures `{', '.join(named)}` in a parameter list — a guest export taken "
                "as a destructured parameter is a read this gate cannot check (no initialiser "
                "names the guest object), so it is reported instead of skipped; take the object "
                "and read the member"
            )

        for index, line in enumerate(text.split("\n")):
            reduced = normalise_casts(line)
            for match in _TEMPLATE_KEY_READ_RE.finditer(reduced):
                local = match.group("local")
                if local not in sources:
                    continue
                reported += 1
                problems.append(
                    f"callback site {path.name}:{index + 1} reads "
                    f"`{local}[\u0060\u2026\u0060]` with a template-literal key — the property "
                    "name is run-time text, not an identifier, so the read cannot be judged; it "
                    "is reported instead of skipped. Spell the property as a member access"
                )
            for match in _BRACKET_KEY_READ_RE.finditer(reduced):
                local = match.group("local")
                if local not in sources:
                    continue
                key = match.group("key").strip()
                if _JUDGED_BRACKET_KEY.fullmatch(key):
                    # `local["onMutate"]`: `_ACCESS_RE` judges it, and this rule
                    # must not report what another rule already judged.
                    continue
                reported += 1
                problems.append(
                    f"callback site {path.name}:{index + 1} reads "
                    f"`{local}[{key}]` with a computed key — the property name is computed "
                    "(a variable, a call, an index or a concatenation), not an identifier the "
                    "declaration can be compared against, so the read cannot be judged; it is "
                    "reported instead of skipped"
                )
            for match in _EXPR_RECEIVER_RE.finditer(reduced):
                local = match.group("local")
                if local not in sources:
                    continue
                if _in_call_position(reduced, match.start()):
                    # `f(guest)?.onX` reads a property of the CALL's result.
                    continue
                reported += 1
                problems.append(
                    f"callback site {path.name}:{index + 1} reads a property of an expression "
                    f"that ends in `{local}` — the receiver is an expression, not the guest "
                    "object itself, so the property cannot be attributed to the declaration; "
                    f"put the guest in its own local and read `{local}.<name>`"
                )
            for match in _MEMBER_CAST_RE.finditer(reduced):
                local = match.group("local")
                if local not in sources:
                    continue
                if not _casts_before_a_read(reduced, match.start(), match.end()):
                    continue
                reported += 1
                problems.append(
                    f"callback site {path.name}:{index + 1} casts a member of `{local}` with "
                    "`as`/`satisfies` — a cast whose core is a member expression is not "
                    "reducible to a local, so whatever is read after the cast is invisible to "
                    "this gate; drop the cast, or read the member the guest exports"
                )
    return problems, reported


def _read_line_index(
    lines: list[str],
    kept_positions: list[int],
    binding_index: int,
    window_lines: int,
    local: str,
    read_re: "re.Pattern[str]",
    prop: str,
) -> int:
    """The raw line index of the read of `local.<prop>` inside the binding's window.

    The window runs to the end of the enclosing block, which can be 80 lines past
    the binding (`eventTarget.ts`'s five event bridges sit at 68–112 with their
    binding on 26). A fixed 12-line probe found nothing there, so every finding
    quoted the BINDING's line instead of the read's — a finding that sends the
    reader to the wrong code, diagnosed by round 2 as a second un-widened copy of
    the same rule. Scanning the window's own lines keeps the line mapping exact
    (`normalise_casts` is applied per LINE, so a reduction can never move a line
    number), and it runs only when a finding is being reported, so the green path
    pays nothing. Falls back to the binding's line when the window has no read at
    all — which is the "never uses it" finding's case.
    """
    for offset in range(window_lines + 1):
        probe_index = binding_index + offset
        if probe_index >= len(lines):
            break
        for match in read_re.finditer(normalise_casts(lines[probe_index])):
            if match.group("local") != local:
                continue
            # The read being REPORTED, not merely the first read of the local:
            # `eventTarget.ts` reads `callbacks` at 68, 86, 102, 107 and 112, and
            # keying on the local alone quoted 68 for a defect on 86.
            if (match.group("prop") or match.group("bracket")) != prop:
                continue
            return kept_positions[probe_index]
    return kept_positions[binding_index]


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
    # back empty and a wrong name reads green. (Both dot spellings: until
    # `_ACCESS_RE` admitted the plain `.`, `eventTarget.ts`'s five event bridges —
    # which use it — were reads the scan never saw, and the counts around them
    # looked right.)
    # Two read notations, both real in these sources:
    #   guest?.onMutate        — member access (the plain-dot spelling is judged
    #                            by the same branch);
    #   (guest as any)?.["on_mutate"] — computed access, which is how a "quick"
    #                                    fix would put a wrong name back without
    #                                    tripping anything else. Both are checked
    #                                    against the same declared set.
    # Not covered HERE (a JS tokenizer, not this gate): destructuring
    # (`const { onX } = guest`) and template-literal prop names. Both are the
    # property of `unparseable_read_problems`, which reports the shape instead of
    # letting it read green — this scan is the one that JUDGES a name, so a name
    # the source does not spell as a name is somebody else's question.
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
                # Lazily, and only on a finding: the line of the first read of this
                # local inside the window, which for `eventTarget.ts` is 40-86 lines
                # below the binding.
                read_line_index = _read_line_index(
                    lines,
                    kept_positions,
                    filtered_index,
                    window.count("\n"),
                    local,
                    read_re,
                    prop,
                )
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


def findings() -> tuple[list[Finding], int, int, int, int, int, int]:
    """(findings, wired interfaces, declared ops, dispatched ops, callback lookups,
    bound callbacks, guest reads in an unparseable shape)."""
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
    # 3c: reads the two scripts above CANNOT see (a destructured key, a computed
    #     key). Reported, never judged: they keep the binding and hide the read,
    #     which is the one combination the pin plus the scan cannot catch.
    unparseable_findings, unparseable_total = unparseable_read_problems()
    for problem in unparseable_findings:
        results.append(Finding("callback-mismatch", problem, ""))

    return (
        results,
        len(interfaces),
        declared_total,
        dispatched_total,
        callback_total,
        bound_total,
        unparseable_total,
    )


def selftest() -> int:
    """Pin the LEXICAL layer against captured shapes; reads no repo file.

    Why this exists next to the suite pins: the rules that JUDGE a name have a
    floor (the base counts) and a set pin (`EXPECTED_BINDINGS_BY_FILE`), but the
    rule that REPORTS a shape can only ever print 0 on a clean tree — a detector
    that stopped matching reads exactly like a tree with no such shape. So the
    shapes and their current verdicts are captured here; `wit-ops-check` runs
    `--check` alone, so the lane that executes this is the suite
    (`media-query.test.mjs` spawns it) — say so rather than implying the recipe
    runs it.

    Every expectation below is a MEASURED behaviour, not a wish. Two of them pin
    a residual: `[//]` truncating its line, and `strip_comments` eating nothing
    else. If one flips, the module docstring's blind-spot list is where to look
    first — that is the contract this function executes.
    """
    failures: list[str] = []

    def expect(label: str, actual: object, wanted: object) -> None:
        if actual != wanted:
            failures.append(f"{label}: got {actual!r}, wanted {wanted!r}")

    # strip_comments: the three `//` shapes, and one that is not a comment.
    expect("a line comment is dropped", strip_comments("a; // gone\nb;"), "a; \nb;")
    expect("a URL inside a string survives", strip_comments('x("ws://h");'), 'x("ws://h");')
    expect(
        "escaped slashes never form `//`",
        strip_comments("const r = /a\\/\\/b/; read();"),
        "const r = /a\\/\\/b/; read();",
    )
    # RESIDUAL: `[//]` swallows the rest of its line. Pinned, not wished away.
    # (The scan copies the `/` that opens the literal and stops at the `//` inside
    # the class, so what survives is the literal's first two characters.)
    expect("[//] truncates its line", strip_comments("const r = /[//]/; read();"), "const r = /[")
    # …and the truncation stops at the newline, so the next line is judged.
    expect(
        "[//] truncation is line-bounded",
        strip_comments("const r = /[//]/;\nread();"),
        "const r = /[\nread();",
    )

    # normalise_casts: the three cast shapes the docstring names, plus `satisfies`.
    for shape in (
        "((guest as any))",
        "(guest as Record<string, any>)",
        "(guest as Fn<(a: T) => U>)",
        "(guest satisfies any)",
    ):
        expect(f"{shape} reduces", normalise_casts(f"{shape}?.onMutate"), "guest?.onMutate")
    # …and a member core is deliberately NOT reduced (the read inside it is judged
    # instead, which is what makes it fail-closed rather than silent).
    expect(
        "a member core is left untouched (its inner read is judged instead)",
        normalise_casts("(guest.sub as any)?.onMutate"),
        "(guest.sub as any)?.onMutate",
    )

    # The read scan: both dot spellings and both bracket spellings are reads.
    def reads(text: str) -> list[tuple[str, str]]:
        return [
            (match.group("local"), match.group("prop") or match.group("bracket"))
            for match in _ACCESS_RE.finditer(normalise_casts(text))
        ]

    expect("optional member read", reads("guest?.onMutate"), [("guest", "onMutate")])
    expect("plain member read", reads("guest.onMutate"), [("guest", "onMutate")])
    expect("quoted bracket read", reads('guest["onMutate"]'), [("guest", "onMutate")])
    expect(
        "cast + quoted bracket read",
        reads('(guest as any)?.["onMutate"]'),
        [("guest", "onMutate")],
    )
    expect("a template subscript is NOT a read", reads("guest[`onMutate`]"), [])

    # guest_bindings: the three ways the namespace is reached.
    direct = 'const guest = globalThis.__wasmExports?.["tairitsu-browser:full/event-callbacks@0.2.0"];'
    indirect = (
        "function callbacks() {\n"
        '  return globalThis.__wasmExports?.["tairitsu-browser:full/media-query-list-callbacks@0.2.0"];\n'
        "}\nconst guest = callbacks();"
    )
    key = "tairitsu-browser:full/event-callbacks@0.2.0"
    expect("direct binding", [(local, kind) for local, kind, _o in guest_bindings(direct)], [("guest", key)])
    expect(
        "helper binding",
        [(local, kind) for local, kind, _o in guest_bindings(indirect)],
        [("guest", "tairitsu-browser:full/media-query-list-callbacks@0.2.0")],
    )
    expect("a non-callback key is not a binding", guest_bindings('const g = globalThis.__wasmExports?.["wasi/x"];'), [])

    # The fail-closed shape rules, piece by piece (the walk over files is pinned by
    # `media-query.test.mjs`; these are the pieces it is built from).
    expect("destructuring from the guest", _guest_source("guest;", ["guest"]), "guest")
    expect("destructuring from a guest-returning helper", _guest_source("callbacks()", ["callbacks"]), "callbacks")
    expect("destructuring from a direct lookup", _guest_source('globalThis.__wasmExports["k"];', []), "__wasmExports")
    expect("an unrelated initialiser", _guest_source("globalThis.__elementHandles.get(x)", ["guest"]), None)
    # The three assertion spellings: each sat between the pattern and the object
    # and each read as "no destructuring from the guest" before the rule learned
    # about them.
    expect("initialiser with `as`", _guest_source("guest as any", ["guest"]), "guest")
    expect("initialiser with `satisfies`", _guest_source("guest satisfies T", ["guest"]), "guest")
    expect("initialiser with `!`", _guest_source("guest!", ["guest"]), "guest")
    expect(
        "a typed pattern still matches",
        bool(_DESTRUCTURE_DECL_RE.search("const { onX }: any = guest;")),
        True,
    )
    expect(
        "a default value with a comma keeps one key",
        _destructured_names(" onMutate = f(1, 2) "),
        ["onMutate"],
    )
    expect(
        "a default value's call does not become a key",
        _destructured_names(" onMutato = pick(a, b) "),
        ["onMutato"],
    )
    expect("a variable subscript is a shape", bool(_BRACKET_KEY_READ_RE.search("guest[key]")), True)
    expect(
        "a quote-starting concatenation is a shape",
        bool(_BRACKET_KEY_READ_RE.search('guest["on" + dyn]')),
        True,
    )
    expect(
        "a simple quoted subscript is NOT this shape (it is judged)",
        _JUDGED_BRACKET_KEY.fullmatch('"onMutate"') is not None,
        True,
    )
    expect(
        "an expression receiver is a shape",
        bool(_EXPR_RECEIVER_RE.search("(0, guest)?.onX")),
        True,
    )
    # Call position: `f(guest)` is a CALL's argument list, `void (guest)` is an
    # operand slot. Getting this wrong fused the argument into the property name
    # and rejected correct code.
    expect("a call's parens are call position", _in_call_position("f(guest)", 1), True)
    expect("a keyword's parens are not", _in_call_position("void (guest)", 5), False)
    expect(
        "a call keeps its argument",
        normalise_casts("callbacks.onGenericEvent(evtType)"),
        "callbacks.onGenericEvent(evtType)",
    )
    expect(
        "a cast ARGUMENT keeps its call too",
        normalise_casts("foo(guest as any)?.onMutato"),
        "foo(guest as any)?.onMutato",
    )
    expect(
        "an alias RHS is a guest source",
        _guest_source("guest", ["guest"]),
        "guest",
    )
    expect(
        "a property read is NOT an alias RHS",
        _guest_source("guest?.onMutate", ["guest"]),
        None,
    )
    # The six spellings that hand the namespace over at run time. Each one read
    # rc 0 while the plain copy was reported, so they are pinned together.
    for form in ("guest || {}", "{} ?? guest", "() => guest", "async () => guest", "{ ...guest }", "{ ...guest, x: 1 }"):
        expect(f"an alias written as {form}", _guest_source(form, ["guest"]), "guest")
    for form in ("() => other", "other || {}", "{ ...other }"):
        expect(f"a NON-alias written as {form}", _guest_source(form, ["guest"]), None)
    expect(
        "an optional call keeps its arguments",
        normalise_casts("identity?.(guest)?.onMutato"),
        "identity?.(guest)?.onMutato",
    )
    expect(
        "array destructuring from the guest is a shape",
        bool(_DESTRUCTURE_ARRAY_RE.search("const [onX] = guest;")),
        True,
    )
    expect(
        "a member-core cast is a shape",
        bool(_MEMBER_CAST_RE.search("(guest.onMutate as any)?.onMutato")),
        True,
    )
    expect(
        "a reducible cast is NOT a member-core cast",
        bool(_MEMBER_CAST_RE.search("(guest as any)?.onMutate")),
        False,
    )
    expect(
        "a pattern that names a declared op",
        [name for name in _destructured_names(" onMutate: fn, ...rest ") if name == "onMutate"],
        ["onMutate"],
    )
    expect("a quoted subscript is not a template shape", bool(_TEMPLATE_KEY_READ_RE.search('guest["onMutate"]')), False)
    expect(
        "a cast-wrapped template subscript is a shape",
        bool(_TEMPLATE_KEY_READ_RE.search(normalise_casts("(guest as any)?.[`${k}`]"))),
        True,
    )
    expect(
        "an arrow parameter is a shape",
        bool(_DESTRUCTURE_PARAM_RE.search("(({ onMutate }) => onMutate)")),
        True,
    )

    if failures:
        print(f"selftest FAILED ({len(failures)} of the captured shapes disagree):", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print("selftest ok: the lexical layer still produces every captured verdict")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument("--check", action="store_true", help="enforce")
    group.add_argument("--list", action="store_true", help="print the dispatch pairs")
    group.add_argument(
        "--selftest", action="store_true", help="pin the lexical layer against captured shapes"
    )
    args = parser.parse_args()

    if args.selftest:
        return selftest()

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
        unparseable_total,
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
        f"{unparseable_total} guest read{'' if unparseable_total == 1 else 's'} in a shape "
        "the scan cannot judge (destructuring / computed key / member cast), "
        f"callback scan excluded {', '.join(excluded_scan_files()) or 'nothing'} "
        f"(plus {len(skipped_scan_files()) - len(excluded_scan_files())} runtime files that "
        "never touch the guest namespace)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
