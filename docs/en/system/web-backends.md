# Web Backend: One Path — WIT Contract + JS Host Implementation

`packages/web` has exactly **one** web backend path. The "dual backend" described by earlier
docs — `web` (`wasm-bindgen` / `web-sys`) alongside `wit-bindings` — no longer exists: the
`web` side had no consumers, and its dependencies and `cfg` branches are retired.

## What this path is

| Item | Value |
| --- | --- |
| Compile target | `wasm32-wasip2` (WASI 0.2 / 0.3 Component Model) |
| Contract | WIT worlds `tairitsu-browser:*` (`packages/browser-worlds/wit/`) |
| Guest-side bindings | `wit-bindgen` (`wit-bindings` feature) |
| Browser-side implementation | JS host (`packages/browser-glue/`) |
| Guest browser capabilities | Component Model host, via host-injected imports |

The entry point is `WitPlatform` (`packages/web/src/wit_platform.rs`, `wit-bindings` feature).
`BrowserPlatform` (`packages/web/src/browser.rs`) is a placeholder implementation — immediate
callbacks, default rects — and is not a web-sys backend.

## Why only this path remains

1. **The Component Model changes how capabilities are delivered.** `wasm32-unknown-unknown` +
   `web-sys` compiles browser capabilities directly into the guest's import section, so neither
   *who the host is* nor *which capabilities were granted* can be declared. A WIT contract states
   the capability surface as reviewable, versionable import interfaces, and the host authorizes
   them per instance.
2. **Build-time transpilation, not runtime self-hosting.** jco transpiles a component into core
   wasm plus a JS canonical-ABI runtime at build time; browsers have **no** native
   `WebAssembly.Component`. "Contract in WIT, implementation in JS" is therefore the only
   workable shape today — not a trade-off.
3. **Host responsibilities are made explicit.** Instantiation, capability authorization, async
   scheduling and lifecycle (including hard termination) belong to the tairitsu host; the guest
   only describes what it needs.

## World layers

WIT worlds are split into packages named `tairitsu-browser:{domain}` under
`packages/browser-worlds/wit/` (`browser-full.wit` plus the per-domain packages generated into
`generated/`). That directory is authoritative for the package list.

## Build and packaging

`packages/packager` provides the `tairitsu build` CLI, which supports **only the `component`
target** (`wasm32-wasip2` + browser-glue path). Any other target fails with
`Unknown build target … Only 'component' is supported.`

Component build steps:

1. Verify the `wasm32-wasip2` toolchain is installed
2. `cargo build --target wasm32-wasip2 --lib`
3. Copy the `.wasm` component into `output_dir`
4. Copy the browser-glue runtime into `output_dir/browser-glue/`
5. Generate the host `index.html` (loads the component through browser-glue)

> **Prerequisite**: run `npm run build` in `packages/browser-glue/` first, to compile the
> TypeScript glue code.

## Recommendations

- Write new projects directly against `wit-bindings` and the WIT worlds; there is no legacy path
  that needs "stay on `web` first, migrate later".
- When you need a new browser capability, declare the interface on the WIT side and implement it
  in the JS host — do not pull a DOM binding library into the guest.
- Async API contract shapes follow the current declarations in `packages/browser-worlds/wit/`;
  this page does not restate individual function signatures.