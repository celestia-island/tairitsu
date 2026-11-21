# Capability authorization (W6)

A component gets **exactly** the glue interfaces and WASI shim modules it
declares, and nothing else. This file is the contract; the code that enforces it
lives in three places:

| Where | What it does |
|---|---|
| `packages/browser-glue/scripts/emit-glue-esm.mjs` | emits the conservative **default** set (`dist/glue-esm/default-capabilities.json`) from the registry, and makes every emitted interface module check the declaration before handing out a function |
| `packages/packager/src/wasm/mod.rs::write_capabilities` | writes `tairitsu-capabilities.json` into the dist: the component's own declaration, or the default |
| `packages/packager/src/wasm/boot.template.js` | fetches that file, injects only the declared interfaces/shim modules, and reports a mismatch as `capability-denied` |

## The declaration

`tairitsu-capabilities.json` beside the component's manifest:

```json
{
  "interfaces": ["document", "element-css-inline-style"],
  "wasiModules": []
}
```

* `interfaces` — glue interface names, with or without the
  `@tairitsu-glue/` / `tairitsu-browser:full/` prefix and without the version
  suffix (`document`, `element-css-inline-style`, …).
* `wasiModules` — a subset of `cli`, `clocks`, `filesystem`, `io`, `random`,
  `sockets`. Anything else is a build/boot error, exactly like an unknown
  interface. An **empty list means nothing from WASI is injected and the shim
  modules are not even fetched** — how a pure-function world is served without
  the `wasi:cli/clocks` stub at runtime. (Removing those stubs from the *artifact* is a
  separate, build-side job — see "Not in this cut" below.)

A component without a declaration gets the default set: every interface the glue
registry provides (25 today) plus all six shim modules. The default is **emitted
from the registry** (`registryInterfaceNames()`), and the test in
`packages/browser-glue/test/instance.test.mjs` re-emits into a fresh directory
and compares the emitted list with the registry's keys, the shim list with the
six modules, and the count with 25 — so a hand edit of the emitter's map, a
dropped interface or a new one without a matching registry entry all fail. The
data file itself is a build artifact; it is not shipped as a source of truth.

## Denial semantics

Both directions are errors, and both are reported before anything runs:

| Situation | Result |
|---|---|
| declared, but the host cannot provide it | the boot throws *"the component declares capabilities the host does not provide: …"* **before** instantiate |
| imported by the wrapper, but not declared | the emitted glue module throws *"capability denied: … is not declared"* while the wrapper is being imported |
| declared == imported | the component boots normally |

Either failure sets `data-tairitsu-error="capability-denied"` (not the generic
`instantiate-failed`), so a page can tell "the component asked for something it
was not granted" apart from "the wasm would not load".

## When the declaration is not loaded

The emitted interface modules check the declaration at import time and **fail
open** when `globalThis.__TAIRITSU_CAPABILITIES__` is unset: "no declaration
loaded yet" is not the same as "not granted". That state is only reachable when a
page imports a `/glue/<interface>.js` module directly (the URL is public): the
boot always loads the declaration and publishes it *before* the wrapper — and
therefore the glue modules — is imported. A component cannot trigger it; a page
that does so is the host acting on its own behalf, which is the boundary this
grant draws.

## Scope of the grant

The declaration is **per dist**, not per guest instance. A page boots one
component (see the single-guest contract in W4); the capability list therefore
describes that component. A host that wants two differently-authorized
components on one page needs the per-call instance binding listed below.

## Worker lane (W7)

A page can opt into running the component in a dedicated Worker
(`<html data-tairitsu-worker>`); the default is unchanged (main thread).

* The worker host has **no DOM**, so it can provide no glue interface and no WASI
  module: a capability declaration that names any of them is refused there with
  `capability-denied`. Components that touch the DOM stay on the main thread —
  that is the boundary, not a gap.
* `globalThis.__TAIRITSU_WORKER__.terminate()` is the hard kill: it drops the
  Worker, disposes the page's W4 instance and reports whether anything was
  running. It is the only way to stop a component that never yields; the
  cooperative stop still exists for components that do.
* **The facade is async in this lane.** Post-message is asynchronous by nature,
  and a synchronous facade would need `Atomics.wait`, which blocks the main
  thread — contradicting the reason the lane exists. Callers get an async entry
  (`boot` resolves on `ready`/`error`); the four hikari components are pure
  compute and fit it without changes.

## Not in this cut

* **W4 residue ①** — the `elementReverse` reverse index keeps a page-level store
  hook. It is harmless under the single-guest contract; a second live instance
  could still feed the first one's index. **Not touched by W6.**
* **W4 residue ②** — per-call instance binding. W6 does not add it; the
  single-active-guest contract stands, and the capability list is per dist. This
  remains the input for the multi-instance work.
* **W4 residue ③** — the state bags' dispose clearing is exercised by hand but
  has no dedicated assertion. **Not touched by W6.**
* **Artifact-level WASI stub stripping** — an empty `wasiModules` list stops the
  *boot* from injecting the shim, which is what a page controls. Stripping the
  unused `wasi:cli/clocks` stub out of the compiled artifact is a rustc/linker
  concern; doing it needs a post-link tool we do not have, so it stays a
  follow-up rather than a half-measure here.