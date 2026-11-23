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

   The jco camelCase is required. A snake_case fallback
   (`guest?.onMutate ?? guest?.on_mutation`) is allowed only as the second
   candidate of the same `??`, so a wrapper built with the other convention (or
   a future jco that renames again) keeps working while a lone wrong name — the
   shape that silently drops every callback — still fails. The value shape is
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

Usage:
    python3 scripts/check_wit_ops.py --check   # the gate
    python3 scripts/check_wit_ops.py --list    # dispatch pairs, for diffing
"""
from __future__ import annotations

import argparse
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
# rather than with one megapattern: the single-pattern version silently dropped
# 41 of 42 sites (its bracket class swallowed the rest of the line), and the
# MIN_CALLBACK_SITES floor is what turned that into a red gate instead of a
# quietly smaller report.
_CALLBACK_KEYED = re.compile(
    r'__wasmExports\s*\??\.?\[\s*"(?P<key>[^"]+)"\s*\]',
)
# How far past a bound lookup the property reads may sit (lines). The bridges
# read and call within a handful; the bound-callback scan below uses it.
_WINDOW_LINES = 120

_CALLBACK_KEY_RE = re.compile(r"^(?P<ns>.*)/(?P<iface>[a-z0-9-]+)@(?P<ver>[0-9.]+)$")
# Right after the closing `]` the inline dispatch continues with `?.<prop>(` /
# `.<prop>(` — `\s*` only, never a newline: a newline means the lookup was bound
# to a local (`const exp = …`) and the call goes through that local.
_DISPATCH_PROP_RE = re.compile(
    r"(?:\s*(?:\?\.|\.)\s*)(?P<prop>[A-Za-z_$][A-Za-z0-9_$]*)(?:[^\n]*\()"
)


def strip_comments(text: str) -> str:
    """Drop `//` and `/* … */` so a commented-out wiring is not read as wired."""
    without_block = re.sub(r"/\*.*?\*/", "", text, flags=re.DOTALL)
    return "\n".join(line.split("//", 1)[0] for line in without_block.splitlines())


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


def snake(kebab: str) -> str:
    """`on-mutate` → `on_mutate`. The fallback spelling a site may use.

    `packages/browser-glue/src/runtime/platformHelpers.ts` reads
    `guest?.onTimeout ?? guest?.on_timeout`: the first is what jco emits, the
    second keeps a wrapper built to the WIT spelling working. Accepting both is
    what keeps this check from forbidding a deliberate belt-and-braces read —
    and what makes it catch the defect that motivated it, since
    `on_mutation` is neither (`on-mutate` camelCases to `onMutate`, never to
    `on_mutation`).
    """
    return kebab.replace("-", "_")


def callback_sites() -> tuple[list[tuple[str, int, str, str, str]], int]:
    """The guest-export dispatch sites, plus how many lookups were scanned.

    Returns `([(module, line, interface, version, property), …], lookups)`.

    Two notations, both real in these sources:

    * **inline** — `<key>"]?.<prop>(…`, on one line. Named here;
    * **bound** — `const guest = …["<key>"];` … `guest?.<prop> ?? guest?.<snake>`.
      NOT named: the call goes through a local, so the property is several lines
      away from the lookup and belongs to whichever local. Those sites are
      covered behaviourally instead (`packages/browser-glue/test/callback-bridge.test.mjs`
      drives every bridge through its real DOM path), which is why the count of
      bound lookups is returned: the gate asserts the scan still SEES them, so a
      broken scan cannot hide behind "everything is inline".

    An interface whose key is not `<ns>/<iface>@<ver>` is skipped — this check is
    about the browser world's callbacks.
    """
    sites: list[tuple[str, int, str, str, str]] = []
    lookups = 0
    for path in sorted(RUNTIME_DIR.glob("*.ts")):
        source = path.read_text(encoding="utf-8", errors="replace")
        for line_number, line in enumerate(strip_comments(source).split("\n"), start=1):
            for lookup in _CALLBACK_KEYED.finditer(line):
                keyed = _CALLBACK_KEY_RE.match(lookup.group("key"))
                if keyed is None:
                    continue
                lookups += 1
                rest = _DISPATCH_PROP_RE.match(line, lookup.end())
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


_MEMBER_RE = re.compile(
    r"(?<![A-Za-z0-9_$])[A-Za-z_$][\w$]*\??\.(?P<prop>[A-Za-z_$][\w$]*)"
)


def _statement_around(text: str, position: int) -> str:
    """The `;`-delimited statement containing `position` (bounded to 400 chars)."""
    start = max(text.rfind(";", max(0, position - 400), position), text.rfind("\n", max(0, position - 400), position))
    end = text.find(";", position)
    if end < 0:
        end = min(len(text), position + 200)
    return text[start + 1 : end]


def bound_bridge_problems(declared_by_iface: dict[str, list[str]]) -> tuple[list[str], int]:
    """Direction 3b: a lookup bound to a local must be CALLED through that local.

    The inline form is named by `callback_sites`; the bound form
    (`const guest = …["<key>"];` … `guest?.<prop>(…)`) is not, so without this the
    whole `platformHelpers.ts` rewrite could be reverted to `guest?.on_wrong` and
    stay green. For every bound local: each property read off it must be an
    accepted spelling of the interface's declared funcs, and the local (or the
    slot it was stored into) must actually be used.
    """
    problems: list[str] = []
    bindings = 0
    # Accepts a WRAPPED binding (`const guest =\n  globalThis.__wasmExports?.[…]`),
    # which is how the longer interface names fit the line budget.
    # `(?<!\.)` keeps `guest?.onMutate` and, crucially, `(guest?.onMutate ?? guest?.
    # on_mutation)?.` — the shape the inline dispatch is rewritten into — out.
    bound_re = re.compile(
        r"(?<![.A-Za-z0-9_$])const\s+(?P<local>[A-Za-z_$][A-Za-z0-9_$]*)\s*=\s*\n?"
        r"\s*(?:globalThis|g)\.__wasmExports\??\.?\[\s*\"(?P<key>[^\"]+)\"\s*\]"
    )
    # `<local>?.<prop>` / `<local>.<prop>` ONLY. A looser pattern scans for a bare
    # identifier, and a preceding `??` supplies one (`?? guest?.onX`), so the
    # property list comes back empty and a wrong name reads green.
    read_re = re.compile(
        r"(?<![A-Za-z0-9_$])(?P<local>[A-Za-z_$][A-Za-z0-9_$]*)(?:\?\.|\.)\s*"
        r"(?P<prop>[A-Za-z_$][A-Za-z0-9_$]*)"
    )
    for path in sorted(RUNTIME_DIR.glob("*.ts")):
        # Whole-line comments are dropped BEFORE the window is cut: a comment that
        # mentions a bad spelling is not a read of it.
        source = path.read_text(encoding="utf-8", errors="replace")
        kept = [
            line
            for line in strip_comments(source).split("\n")
            if line.strip() and not line.strip().startswith("//")
        ]
        text = "\n".join(kept)
        lines = text.split("\n")
        for binding in bound_re.finditer(text):
            keyed = _CALLBACK_KEY_RE.match(binding.group("key"))
            if keyed is None:
                continue
            bindings += 1
            iface = keyed.group("iface")
            declared = declared_by_iface[iface] if iface in declared_by_iface else declared_funcs(iface)
            if not declared:
                continue  # reported by the inline pass as an unknown interface
            # The jco camelCase only. A snake spelling is a legitimate
            # `??` FALLBACK, never a candidate on its own — resolving it to the
            # accepted set would make a lone wrong name read green.
            accepted = {camel(name) for name in declared}
            local = binding.group("local")
            line_index = text[: binding.start()].count("\n")
            # The window: the rest of the binding's line plus the following lines,
            # stopping at the next binding or at the end of the method. A bounded
            # count on purpose — a dedented brace is not a reliable stop once whole
            # comment lines are removed — and an overrun can only add reads of the
            # SAME local, which are that bridge's own.
            tail = text[binding.end() :].split("\n")
            window = tail[0] + "\n"
            for probe in tail[1 : 1 + _WINDOW_LINES]:
                if bound_re.search(probe) or probe.rstrip() == "  },":
                    break
                window += probe + "\n"
            # (local, prop) pairs this binding's window reads. A guarded read can be
            # split from its local by the pre-pass, which removes whole COMMENT
            # lines: `const guest = …;` then a five-line comment then
            # `guest?.onScrollEvent`. A bare property on a line whose only other
            # content is a local (or a local assigned from it) is the same read.
            reads = [(read.group("local"), read.group("prop")) for read in read_re.finditer(window)]
            # Where each read sits, to tell a first candidate from a guarded one.
            spans = [(read.group("local"), read.group("prop"), read.end()) for read in read_re.finditer(window)]
            aliases = {local} | {
                assigned.group("name")
                for assigned in re.finditer(
                    r"const\s+(?P<name>[A-Za-z_$][A-Za-z0-9_$]*)\s*=\s*[^;]*\b"
                    + re.escape(local)
                    + r"\b[^;]*;",
                    window,
                )
            }
            for probe in tail[1 : 1 + _WINDOW_LINES]:
                stripped = strip_comments(probe).strip()
                if not stripped or bound_re.search(probe):
                    continue
                for head in aliases:
                    if head not in stripped:
                        continue
                    rest = stripped.replace(head, "", 1).lstrip().lstrip("?.").lstrip()
                    if re.fullmatch(r"[A-Za-z_$][A-Za-z0-9_$]*", rest):
                        reads.append((local, rest))
                        spans.append((local, rest, -1))
                        break
            for owner, prop, position_of in spans:
                if owner != local:
                    continue
                if prop in accepted:
                    continue
                # A fallback (`a ?? b`) is allowed ONLY when one side of the pair
                # is a name the guest really exports. For `a` the partner is
                # whatever the `??` chain leads to; for `b` it is whatever comes
                # before it. Both are resolved by slicing the local's own pair on
                # `??` and comparing the member accesses — no anchored regex over a
                # dangling prefix (that form does not behave like the same regex
                # inside a longer text: measured, twice).
                statement = _statement_around(window, position_of)
                if "??" in statement:
                    names_in_sides = {match.group("prop") for match in _MEMBER_RE.finditer(statement)}
                    has_partner = bool(names_in_sides & accepted)
                else:
                    has_partner = False
                if has_partner:
                    continue
                problems.append(
                    f"callback site {path.name}:{line_index + 1} binds `{local}` to "
                    f"`{keyed.group('iface')}@{keyed.group('ver')}` and reads `{local}.{prop}`, "
                    f"which the guest does not export (declared: {', '.join(declared)}) and which "
                    "no other candidate on that local repairs — the callback is dropped with no "
                    "error"
                )
            # "Used" = the local (or a local assigned from it) is invoked, passed
            # on, or read from (`callbacks.onMouseEvent(…)` is a read of
            # `callbacks`). A binding that is stored and never mentioned stays dead.
            names = {local}
            for assigned in re.finditer(
                r"const\s+(?P<name>[A-Za-z_$][A-Za-z0-9_$]*)\s*=\s*[^;]*\b"
                + re.escape(local)
                + r"\b[^;]*;",
                window,
            ):
                names.add(assigned.group("name"))
            named = r"(?<![.A-Za-z0-9_$])(?:" + "|".join(re.escape(n) for n in sorted(names)) + r")"
            # A binding with NOTHING after it on its line (`if (…) { if (guest) {`)
            # has its uses inside the block, so a bare mention is enough there.
            stored = tail[0].strip() != ""
            used = (
                bool(re.search(named + r"\s*(?:[,(]|\.|\?\.)", window))
                if stored
                else bool(re.search(named, window))
            )
            if not used:
                problems.append(
                    f"callback site {path.name}:{line_index + 1} binds `{local}` to "
                    f"`{keyed.group('iface')}@{keyed.group('ver')}` and never uses it — the "
                    "bridge is dead weight"
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
        elif not any(site[2] == name for site in callback_sites()[0]):
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
    bound_findings, bound_total = bound_bridge_problems(declared_by_iface)
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
        f"{callback_total} guest-export lookups ({bound_total} bound) named correctly"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
