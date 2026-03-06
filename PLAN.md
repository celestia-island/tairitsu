# Tairitsu — WIT-First Browser Interface Architecture

## Initiative Overview

**Status**: 🚧 In Progress  
**Last Updated**: 2026-03-06  
**Goal**: Decouple browser/W3C API bindings from `wasm-bindgen` version lockstep by using WIT worlds as the protocol framework. The build tooling (`tairitsu` CLI / `build.rs`) resolves versioned WIT packages, fetches declarations from the cloud, caches them locally under `target/tairitsu-wit`, and supports fully-offline builds from cache.

---

## Architecture Summary

```
packages/
├── browser-wit-resolver/   🚧 WIT version resolution, cloud fetch, local cache
├── browser-worlds/         🚧 WIT world definitions
│   └── wit/
│       ├── *.wit               # Phase 0: hand-written baseline (0.1.x)
│       └── generated/*.wit     # Phase A: auto-generated from W3C WebIDL (0.2.x)
├── browser-glue/           🚧 TypeScript/JS adaptor glue (SWC-built)
├── packager/               ✅ CLI extended with `wit` subcommand (fetch / verify)
├── runtime/                ✅ Core WASM component runtime
├── web/                    ✅ Web platform implementation (wasm-bindgen today)
└── …                       (other existing packages unchanged)

scripts/
├── fetch_webidl.py         ✅ Download WebIDL specs from w3c/webref into cache
├── generate_browser_wit.py ✅ Parse WebIDL + generate WIT files grouped by domain
├── gen_wit_from_webidl.py  ✅ Orchestrator: run fetch → generate pipeline
└── download_wasi_adapters.py  (pre-existing)

target/tairitsu-wit/                    (git-ignored)
├── webidl-cache/<spec>.idl             W3C/WHATWG WebIDL source cache
└── <namespace>/<name>/<version>/       Tairitsu WIT package cache
      ├── manifest.json
      └── *.wit
```

**Key insight**: `wasm-bindgen`/`web-sys` are retained as a _compatibility shim_ for the current release. The new WIT worlds define the canonical interface surface. Over time, code generation from WIT worlds replaces `web-sys` direct usage.

---

## Phased Work Plan

### Phase 0 — Foundation ✅
- [x] Replace `PLAN.md` with this document
- [x] Create `packages/browser-wit-resolver` crate — resolver, cache, fetch stub
- [x] Create `packages/browser-worlds` crate — initial WIT world files (dom, events, fetch, canvas, browser-full)
- [x] Create `packages/browser-glue` JS/TS package — SWC setup, TypeScript stubs
- [x] Extend `packages/packager` CLI with `wit` subcommand (`fetch`, `verify`, `list`)
- [x] Update root `Cargo.toml` workspace members

### Phase A — W3C WebIDL Source Pipeline ✅
> _Goal: establish a reproducible, data-driven WIT generation pipeline from authoritative W3C/WHATWG specifications._

- [x] Identify and confirm accessible W3C WebIDL data source: `w3c/webref` (`ed/idl/*.idl` — confirmed accessible via raw GitHub CDN)
- [x] Create `scripts/fetch_webidl.py` — downloads 50 W3C/WHATWG spec files from `w3c/webref`, caches under `target/tairitsu-wit/webidl-cache/`
- [x] Create `scripts/generate_browser_wit.py` — parses WebIDL with custom Python parser, converts to WIT using opaque-handle pattern, groups interfaces by domain, outputs `packages/browser-worlds/wit/generated/*.wit`
- [x] Create `scripts/gen_wit_from_webidl.py` — orchestrates fetch + generate in a single command
- [x] Wire into `justfile` build flow with 8 new recipes (`wit-gen`, `wit-fetch-idl`, `wit-gen-wit`, `wit-fetch-force`, `wit-stats`, `wit-sources`, `wit-list-specs`, `wit-dry-run`)
- [x] Update `PLAN.md`

**Phase A results**: 422 WebIDL interfaces parsed across 18 domains → 18 WIT files generated

| Domain | WIT file | Interfaces | Source spec(s) |
|--------|----------|-----------|----------------|
| `dom` | `generated/dom.wit` | 34 | dom, fullscreen |
| `events` | `generated/events.wit` | 15 | uievents, pointerevents, touch-events, clipboard-apis |
| `html` | `generated/html.wit` | 182 | html |
| `css` | `generated/css.wit` | 36 | cssom, cssom-view, css-animations, css-transitions |
| `fetch` | `generated/fetch.wit` | 25 | fetch, streams, xhr |
| `url` | `generated/url.wit` | 2 | url |
| `storage` | `generated/storage.wit` | 2 | storage |
| `websocket` | `generated/websocket.wit` | 2 | websockets |
| `workers` | `generated/workers.wit` | 12 | service-workers |
| `crypto` | `generated/crypto.wit` | 3 | webcrypto |
| `canvas` | `generated/canvas.wit` | 18 | webgl1, webcodecs |
| `media` | `generated/media.wit` | 12 | mediacapture-streams, media-capabilities, mediasession |
| `webrtc` | `generated/webrtc.wit` | 20 | webrtc |
| `observers` | `generated/observers.wit` | 5 | intersection-observer, resize-observer |
| `performance` | `generated/performance.wit` | 11 | performance-timeline, hr-time, navigation-timing, resource-timing, user-timing |
| `notifications` | `generated/notifications.wit` | 4 | notifications |
| `permissions` | `generated/permissions.wit` | 2 | permissions |
| `device` | `generated/device.wit` | 12 | geolocation, screen-orientation, battery-status, gamepad |

**Data source**: `https://github.com/w3c/webref` (MIT / W3C Software License)  
**Specs not yet fetched** (add to extend coverage): `fileapi`, `mediacapture-output`, `webmidi`, `payment-request`, `credential-management`, `wasm-js-api`, `speech-api`, `screen-capture`

### Phase 1 — Resolver & Cache (next)
- [ ] Implement real HTTP fetch in `browser-wit-resolver::fetch` (reqwest, with timeout)
- [ ] Implement cache integrity check (SHA-256 of WIT content vs. manifest)
- [ ] Add offline-mode detection: if network unavailable, fall back to cache; hard error if cache also absent
- [ ] Integrate resolver into `packages/packager` `build.rs` hook
- [ ] Add `TAIRITSU_WIT_REGISTRY` environment variable override for private registries

### Phase 2 — WIT World Coverage Expansion
- [ ] Expand hand-written `dom.wit` to align with Phase A generated coverage
- [ ] Expand `events.wit` to match all `Event` subtypes from Phase A
- [ ] Add `storage.wit`, `workers.wit`, `websocket.wit` to hand-written baseline
- [ ] Add `streams.wit` (WHATWG Streams)
- [ ] Reach ≥ 90% parity with `wasm-bindgen-cli` surface (Phase A provides the target list)
- [ ] Add missing specs: `fileapi`, `speech-api`, `screen-capture`, `payment-request`, `wasm-js-api`

### Phase 3 — Glue Code Generation
- [ ] Build Rust-side WIT→Rust binding generator (extend `browser-wit-resolver`)
- [ ] Build TS-side WIT→TypeScript stub generator in `browser-glue`
- [ ] Wire generated bindings into `tairitsu-web` as an alternative to `web-sys`
- [ ] CI job that validates generated bindings compile against real browser environments (wasm-pack + headless browser)

### Phase 4 — Migration & Compatibility
- [ ] Deprecation path: feature flag `wit-bindings` in `tairitsu-web`
- [ ] Provide migration guide from `web-sys` to WIT-generated bindings
- [ ] Ensure `wasm-bindgen` version can be bumped independently of WIT world version
- [ ] Document versioning strategy (see below)

---

## WebIDL → WIT Pipeline

### Data Source

The primary source for browser interface definitions is **[w3c/webref](https://github.com/w3c/webref)**, a W3C-maintained repository that:
- Automatically crawls W3C and WHATWG editor's drafts daily via the [Reffy](https://github.com/nicehash/reffy) tool
- Provides machine-readable WebIDL under `ed/idl/<spec>.idl`
- Covers: DOM, HTML, Fetch, Streams, WebSockets, WebGL, WebRTC, Web Crypto, CSS OM, Service Workers, Media Capture, and 40+ more specs
- Licensed: MIT / W3C Software License

**Secondary / future sources** considered:
- `@webref/idl` npm package — same data, npm distribution
- MDN browser-compat-data — for browser compatibility information (Phase 3+)
- W3C Bikeshed / WebIDL validator — for validation only

### Type Mapping

WebIDL primitive types → WIT types:

| WebIDL | WIT |
|--------|-----|
| `boolean` | `bool` |
| `long` / `unsigned long` | `s32` / `u32` |
| `long long` / `unsigned long long` | `s64` / `u64` |
| `double` / `float` | `f64` / `f32` |
| `DOMString` / `USVString` | `string` |
| `undefined` / `void` | _(no return type)_ |
| `T?` (nullable) | `option<T>` |
| `sequence<T>` | `list<T>` |
| `Promise<T>` | `u64` (async handle) |
| `(A or B)` union | first non-undefined member type |
| Any interface type | `u64` (opaque handle) |

### Justfile Recipes

```
just wit-gen          # Full pipeline: fetch W3C WebIDL + generate WIT
just wit-fetch-idl    # Only download WebIDL spec files (cached in target/)
just wit-gen-wit      # Only generate WIT from existing cache
just wit-fetch-force  # Re-download all specs (skip cache)
just wit-stats        # Show interface count per domain
just wit-sources      # Show data source information
just wit-list-specs   # List all target specs + cache status
just wit-dry-run      # Preview what the pipeline would do
```

---

## Versioning Strategy

WIT world packages are versioned independently of the Tairitsu crate version:

| Package | Version | Status |
|---------|---------|--------|
| `tairitsu-browser:dom@0.1.0` | Hand-written baseline | ✅ Phase 0 |
| `tairitsu-browser:events@0.1.0` | Hand-written baseline | ✅ Phase 0 |
| `tairitsu-browser:fetch@0.1.0` | Hand-written baseline | ✅ Phase 0 |
| `tairitsu-browser:canvas@0.1.0` | Hand-written baseline | ✅ Phase 0 |
| `tairitsu-browser:full@0.1.0` | Union world (hand-written) | ✅ Phase 0 |
| `tairitsu-browser-gen:<domain>@0.2.0` | Auto-generated from WebIDL | ✅ Phase A |

Consumers pin a world version in their `Cargo.toml` (or `tairitsu.toml`):

```toml
[tairitsu.browser-worlds]
version = "0.1.0"
```

The resolver maps this to a URL pattern:

```
https://wit.tairitsu.dev/<namespace>/<name>/<version>/<file>.wit
```

(During development / offline mode: served from the embedded fallback in `browser-worlds/wit/`.)

---

## Cache Behaviour

### WIT package cache (`target/tairitsu-wit/`)

| Scenario | Behaviour |
|----------|-----------|
| First fetch (online) | Download → write `target/tairitsu-wit/<ns>/<name>/<ver>/` |
| Subsequent build (cache hit) | Read from cache, skip network |
| Offline + cache hit | Read from cache |
| Offline + cache miss | Hard error with actionable message |
| `--offline` flag | Force cache-only mode |
| `TAIRITSU_WIT_REGISTRY` set | Use custom URL base |

Cache entries include a `manifest.json` with content hashes for integrity verification.

### WebIDL source cache (`target/tairitsu-wit/webidl-cache/`)

| Scenario | Behaviour |
|----------|-----------|
| First run (online) | Download each `<spec>.idl` from w3c/webref |
| Subsequent runs | Use cached file (skip network) |
| `just wit-fetch-force` | Re-download all specs |
| No internet / CI without cache | Script errors clearly; commit generated `.wit` files to avoid this |

---

## Compatibility Strategy

1. **Short term**: `tairitsu-web` keeps `wasm-bindgen`/`web-sys` as default. WIT worlds are additive.
2. **Medium term**: Feature flag `wit-bindings` uses generated WIT bindings instead of `web-sys`.
3. **Long term**: `web-sys` dependency removed; WIT worlds are the sole browser API surface.

`wasm-bindgen-cli` interface surface is the coverage target (≥ 90%) but we are not bound to its versioning.

---

## Risks

| Risk | Mitigation |
|------|------------|
| WIT world API churn | Version pinning + compatibility shims |
| Network unavailability in CI | Embedded fallback WIT in `browser-worlds` crate; commit generated files |
| W3C webref URL changes | Script uses versioned GitHub raw URLs; easy to update |
| WebIDL parser gaps | Unrecognized patterns emit `u64` (safe fallback); improve incrementally |
| Large WIT surface area | Incremental by design; Phase A already achieves 422 interfaces |
| SWC/TS build fragility | Lock SWC version; test in CI |
| Adoption friction | Keep `web-sys` path working until Phase 4 |

---

## Deliverables per Phase

| Phase | Deliverables | Status |
|-------|-------------|--------|
| Phase 0 | `packages/browser-wit-resolver/`, `packages/browser-worlds/` (0.1.x), `packages/browser-glue/`, packager `wit` subcommand | ✅ |
| Phase A | `scripts/fetch_webidl.py`, `scripts/generate_browser_wit.py`, `scripts/gen_wit_from_webidl.py`, `packages/browser-worlds/wit/generated/` (0.2.x, 18 domains, 422 interfaces), justfile `wit-*` recipes | ✅ |
| Phase 1 | Real HTTP fetch in resolver, SHA-256 cache integrity, `build.rs` hook | 🔜 |
| Phase 2 | Expanded hand-written WIT coverage (≥ 90% target), missing spec integration | 🔜 |
| Phase 3 | Rust + TS binding generators, `tairitsu-web` integration, CI validation | 🔜 |
| Phase 4 | Migration guide, `web-sys` deprecation, versioning docs | 🔜 |

---

## Prior Project Status (Archived)

The following phases were completed before this initiative:

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 1: Core vdom | ✅ Done | vdom, reactive system, Diff/Patch |
| Phase 2: Web backend | ✅ Done | WebPlatform, DOM ops, event management |
| Phase 3: Macro system | ✅ Done | rsx!, component, WIT macros |
| Phase 4: Hooks | ✅ Done | use_state/signal/effect/style/context/ref/animation |
| Phase 6: E2E test | ✅ Done (80%) | Test framework complete |
| Phase 7: Packager | ✅ Done (40%) | WASM build, HTML generation |

---

*Plan owner: Tairitsu contributors — update inline as work progresses.*  
*Last updated: 2026-03-06*


