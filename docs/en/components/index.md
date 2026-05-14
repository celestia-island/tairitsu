# Layered Package Overview

The Tairitsu workspace is organized into four layers. Each layer builds on the one below it.

## Layer 1: Foundations (no browser dependency, pure Rust)

| Crate | Path | Description |
|:--|:--|:--|
| `tairitsu` | `packages/runtime` | Core runtime: Container, Image, Registry, WIT binding, dynamic invocation (RON + binary canonical ABI). The server-side engine. |
| `tairitsu-macros` | `packages/macros` | Procedural macros: `rsx!`, `scss!`, `include_scss!`, `svg!`, `component`, `define_props`, `wit_interface!`, `wit_world!` |
| `tairitsu-vdom` | `packages/vdom` | Platform-agnostic Virtual DOM: VNode, VElement, diffing, patching, 34+ typed events, reactive signals, scheduler, safe SVG |
| `tairitsu-style` | `packages/style` | Type-safe CSS property builders from W3C CSS data, serde-compatible, length/color/typed values |
| `tairitsu-hooks` | `packages/hooks` | Hooks system: Signal, Effect, Memo, Suspense/Resource, Context, StateMachine, Animation, Interval, Store |

**Dependency graph** (Layer 1):
```
tairitsu (runtime)     tairitsu-vdom (VDOM)
                              ↑
                         tairitsu-hooks (signals, effects, suspense)
                              ↑
tairitsu-macros ──────────────┘ (rsx!, scss!, component)
tairitsu-style (CSS type system)
```

## Layer 2: Protocol & Platform

| Crate | Path | Description |
|:--|:--|:--|
| `tairitsu-web` | `packages/web` | Umbrella crate: browser platform, router, i18n, SSR integration, navigation, batch DOM operations |
| `tairitsu-ssr` | `packages/ssr` | Server-Side Rendering: HTML renderer, streaming, HMR, fast refresh, error overlay, data fetcher, linker |
| `tairitsu-browser-worlds` | `packages/browser-worlds` | WIT world definitions: 0.1.x (hand-written) + 0.2.x (auto-generated from WebIDL). Covers DOM, Events, Fetch, Canvas, CSS, HTML, Storage, WebSocket, WebRTC, Workers, etc. |
| `tairitsu-browser-wit-resolver` | `packages/browser-wit-resolver` | WIT package resolution: version resolution, cloud fetching, local caching |

**Dependency graph** (Layer 2):
```
tairitsu-web ──→ vdom, hooks, macros, style
     ├──→ tairitsu-ssr ──→ tairitsu (runtime), browser-worlds
     └──→ browser-wit-resolver
```

## Layer 3: Tooling & Delivery

| Crate | Path | Description |
|:--|:--|:--|
| `tairitsu-packager` | `packages/packager` | CLI binary (`tairitsu`): dev server (axum + HMR), build pipeline (WASM optimization, CSS minification, sourcemaps), visual regression, desktop packaging, VTty terminal, debug API, test runner |
| `tairitsu-mcp` | `packages/mcp` | MCP server for AI coding assistants: browser automation + terminal access |
| `tairitsu-e2e` | `packages/e2e` | End-to-end testing: WebDriver + chromiumoxide, form validation, async ops, state management, events |
| `tairitsu-browser-test` | `packages/browser-test` | Browser testing via headless Chromium |
| `browser-glue` | `packages/browser-glue` | TypeScript runtime glue: bridges WIT canonical ABI ↔ DOM APIs. 39+ per-domain internal modules. |

## Layer 4: Examples & Demos

| Crate | Path | Purpose |
|:--|:--|:--|
| `wit-native-macro` | `examples/wit-native-macro` | **Recommended** approach: define WIT interfaces with `wit_interface!` macro, zero boilerplate |
| `wit-native-simple` | `examples/wit-native-simple` | Trait-based WIT interface, maximum control |
| `wit-dynamic-advanced` | `examples/wit-dynamic-advanced` | Dynamic WASM invocation: RON + binary paths, runtime function discovery |
| `wit-compile-time` | `examples/wit-compile-time` | Static WIT binding with wasmtime bindgen |
| `wit-runtime` | `examples/wit-runtime` | Runtime WIT loading, plugin-like system |
| `vtty-graphics-demo` | `examples/vtty-graphics-demo` | Terminal graphics demo using VTty |
| `website` | `examples/website` | Full documentation website demo |

## Choosing Your Approach

| Approach | Type Safety | Performance | Flexibility | Best For |
|:--|:--|:--|:--|:--|
| **Macro** (`wit_interface!`) | Full | Best | Medium | Most use cases |
| **Trait** (`wit-native-simple`) | Full | Best | High | Complex interface hierarchies |
| **Dynamic RON/Binary** | Runtime | Best (binary) | Highest | Plugin systems, hot-reload |
| **Compile-time** | Full | Best | Low | Fixed, known interfaces |
| **Runtime** | Partial | Good | High | Plugin discovery |
