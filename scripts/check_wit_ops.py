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

IMPORT_RE = re.compile(
    r"import\s*\{\s*([A-Za-z_$][A-Za-z0-9_$]*)\s*\}\s*from\s*\"\./([A-Za-z0-9_$]+)\""
)
REGISTRY_KEY_RE = re.compile(
    r'"@tairitsu-glue/([a-z0-9-]+)"\s*:\s*([A-Za-z_$][A-Za-z0-9_$]*)'
)
EMITTER_ENTRY_RE = re.compile(r'^\s*"?([a-z0-9-]+)"?\s*:\s*"([A-Za-z0-9_$]+)"', re.MULTILINE)
WIT_INTERFACE_RE = re.compile(r"^interface\s+([a-z0-9-]+)\s*\{", re.MULTILINE)
WIT_FUNC_RE = re.compile(r"^\s+([a-z0-9][a-z0-9-]*)\s*:\s*func\s*\(", re.MULTILINE)


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
        if self.kind == "no-declaration":
            return (
                f"{self.key}: wired by the registry but the composed WIT declares no such "
                "interface — the declaration scan and the wiring disagree"
            )
        return f"{self.kind}: {self.key} ({self.detail})"


def findings() -> tuple[list[Finding], int, int, int]:
    """(findings, wired interface count, declared op count, dispatched op count)."""
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

    return results, len(interfaces), declared_total, dispatched_total


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

    mismatches, interface_count, declared_total, dispatched_total = findings()
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
        f"{dispatched_total} dispatched, {invented} runtime-only exports listed"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
