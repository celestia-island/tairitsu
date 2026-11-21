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

For a **synchronous** facade (`qrMatrix` is the first), build the surface from
the statically imported module — no await anywhere in the call path:

```ts
// declare-caps.ts — a side-effect module, imported BEFORE the wrapper, because
// ES module imports are hoisted: no statement in this file can run earlier.
//   import { publishCapabilities } from "@celestia-island/tairitsu-host-client";
//   publishCapabilities({ interfaces: [], wasiModules: [] });

import "./declare-caps.js";
import * as wrapper from "./qr/wrapper.js";
import { createSurface } from "@celestia-island/tairitsu-host-client";

const qr = createSurface(wrapper);                       // compute-only
const matrix = qr.call<boolean[][]>("encode", text, { ecc: "m", version: 0 });
```

`createSurface` serves **compute-only** components: a statically imported wrapper
binds its imports before this call, so the door can only *refuse* a capability,
never provide one — a declaration naming an interface or WASI module is rejected
with `sync-door-cannot-provide` (use `loadComponent` for anything else). It also
refuses a wrapper that needs instantiating (`needs-loading`).

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
