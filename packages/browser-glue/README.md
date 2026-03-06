# @tairitsu/browser-glue

TypeScript/JavaScript adaptor glue for Tairitsu browser WIT worlds.

This package bridges the WIT world interfaces (defined in
`packages/browser-worlds`) and the actual browser DOM/W3C APIs. It is
intended to be loaded by a WASM host (e.g. the Tairitsu packager dev server)
to satisfy the host-side imports declared in the WIT worlds.

## Package contents

| Module | WIT world satisfied |
|--------|---------------------|
| `dom-glue.ts` | `tairitsu-browser:dom` — node, document, window, style |
| `events-glue.ts` | `tairitsu-browser:events` — event-target (imports), event-callbacks (exports) |
| `fetch-glue.ts` | `tairitsu-browser:fetch` — fetch-api, async-fetch |
| `canvas-glue.ts` | `tairitsu-browser:canvas` — canvas2d |

## Build

```bash
npm install
npm run build          # compile TypeScript → dist/ with SWC
npm run typecheck      # type-check without emitting
```

## Status (Phase 0)

Initial stubs are wired to real browser APIs where straightforward.
See `PLAN.md` for the full Phase 2/3 implementation roadmap.
