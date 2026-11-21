# @celestia-island/tairitsu-host-client

Load a tairitsu component from a **library** — not a page — and get a
synchronous call surface back.

The page lane (`boot.js` + `component-wrapper-loader.js`) is emitted into a dist
by the packager. A library has no dist: it ships a component artifact and needs
to call it. This package is that bridge, and it keeps the guarantees the page
lane established:

| Guarantee | Where it comes from |
|---|---|
| fail-closed load | W3: a wrapper that cannot be imported or instantiated throws; never a partial surface |
| capability enforcement | W6: the imports are built from the declaration; a declared-but-unprovided interface/wasi module throws **before** instantiate |
| the W4 lifecycle | the surface exposes `stop()` / `whenIdle()` / `stopped` instead of hiding them |
| synchronous calls | `loadComponent()` is async; `component.call()` is sync |

The **bundler-safe** route imports the wrapper statically (so Vite/Rollup can
analyse it) and publishes the declaration *before* that import — the emitted
glue modules check it when they are evaluated:

```ts
import { loadComponent } from "@celestia-island/tairitsu-host-client";
import * as wrapper from "./qr/wrapper.js"; // static: bundlers see it

const qr = await loadComponent({ module: wrapper, capabilities: { interfaces: [], wasiModules: [] } });
```

For a dynamic load, `loadComponent` orders the publish itself:

```ts
import { loadComponent } from "@celestia-island/tairitsu-host-client";

const qr = await loadComponent({
  wrapper: new URL("./qr/wrapper.js", import.meta.url),
  capabilities: { interfaces: [], wasiModules: [] },
});

const matrix = qr.call<boolean[][]>("encode", text, { ecc: "m", version: 0 });
```

Both instantiate ABI shapes are accepted (the wrapper's own
`instantiate(imports)` and the jco-style `instantiate(resolveCoreModule,
imports)`), because both ship in the wild.

## What it does not do

It does not mount anything into the DOM and it does not run a lifecycle entry:
that is the page lane's job (`boot.js`). This package is for compute-only
components and for libraries that need a callable surface.
