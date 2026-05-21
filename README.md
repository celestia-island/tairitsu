<div align="center"><img src="./docs/logo_x256.png" width="120" /></div>
<h1 align="center">Tairitsu</h1>
<div align="center">
 <strong>Full-stack framework powered by the WASM Component Model</strong>
</div>

<br />

<div align="center">
  <a href="https://github.com/celestia-island/tairitsu/actions">
    <img src="https://img.shields.io/github/actions/workflow/status/celestia-island/tairitsu/clippy.yml?branch=master" alt="CI" />
  </a>
  <a href="LICENSE">
    <img src="https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg" alt="License" />
  </a>
  <a href="https://github.com/casey/just">
    <img src="https://img.shields.io/badge/built%20with-just-blue" alt="Built with just" />
  </a>
</div>

<div align="center">
  <h3>
    <a href="#quick-start">Quick Start</a>
    <span> | </span>
    <a href="docs/en/system/overview.md">Architecture</a>
    <span> | </span>
    <a href="docs/en/guides/index.md">Documentation</a>
  </h3>
</div>

<br/>

> Write components once, run them anywhere — server, browser, edge. All communication typed via WIT. The same WASM component talks to a server database, renders HTML via SSR, and handles browser DOM events — all through a single WIT interface definition.

## Why Tairitsu?

Tairitsu unifies two worlds that are typically separate:

| Traditional stack | Tairitsu |
|:--|:--|
| Backend: Node/Go/Rust server | Container runtime (native wasmtime) |
| Frontend: React/Vue/Svelte | VDOM + hooks — compiled to `wasm32-wasip2` |
| Protocol: REST/GraphQL/JSON | WIT interfaces: typed at compile time, zero serialization |
| Build: Webpack/Vite/Trunk | `tairitsu dev` / `tairitsu build` — one tool for both targets |
| Deploy: Docker + CDN | `tairitsu deploy` — push to Registry, serve anywhere |

## Features

- **WIT-native**: No wasm-bindgen. Full Component Model with type-safe interface definitions
- **Docker-like runtime**: Image → Container → Registry lifecycle for WASM components
- **VDOM + hooks**: React-like DX with `rsx!` macro, signals, effects, suspense
- **Compile-time CSS**: `scss!` macro with class hashing (CSS Modules) and source maps
- **Safe SVG**: `svg!` macro with XSS sanitization at compile time
- **SSR + streaming**: Server-side rendering with HMR, fast refresh, error overlay
- **Dual backends**: `WitPlatform` (WIT bindings) and `WebPlatform` (web-sys) side by side
- **W3C coverage**: 50+ WebIDL specs → generated WIT interfaces → TypeScript glue
- **Visual regression**: Pixel diff testing with Chromium automation
- **MCP server**: AI agent integration via Model Context Protocol
- **Desktop packaging**: Bundle as native app (Wry/Tao)

## Quick Start

### Prerequisites

- Rust (stable) — [rustup.rs](https://rustup.rs)
- `just` (command runner) — `cargo install just`
- Python 3 — for WIT generation scripts
- Node.js — for browser-glue TypeScript runtime

### Installation

```bash
cargo install tairitsu-packager
```

### Build & Run

```bash
just build      # Build workspace
just test       # Run tests
just fmt        # Format code
just clippy     # Lint
just install-packager  # Install CLI globally
```

### Example

```rust
use tairitsu_macros::{component, rsx};
use tairitsu_vdom::{VNode, Signal};

#[component]
fn Counter(initial: i32) -> VNode {
    let count = Signal::new(initial);

    rsx! {
        div {
            button {
                onclick: move |_| count.set(count.get() + 1),
                "Clicked: "
                ..txt(&count.get().to_string())
            }
        }
    }
}
```

Same component, two execution paths:
- **Browser**: `tairitsu dev` — renders with real DOM, handles click events
- **Server**: `tairitsu serve` — renders to HTML string via SSR

## Workspace Packages

| Package | Description |
| --- | --- |
| `tairitsu` | Core runtime: Container, Image, Registry, WIT binding, dynamic invocation |
| `tairitsu-macros` | Procedural macros: `rsx!`, `scss!`, `svg!`, `component`, `wit_interface!` |
| `tairitsu-vdom` | Platform-agnostic Virtual DOM: VNode, diffing, patching, 34+ typed events |
| `tairitsu-hooks` | Hooks system: Signal, Effect, Memo, Suspense, Context, Animation, Interval |
| `tairitsu-style` | Type-safe CSS property builders from W3C CSS data |
| `tairitsu-web` | Umbrella crate: browser platform, router, i18n, SSR integration |
| `tairitsu-ssr` | Server-Side Rendering: HTML renderer, streaming, HMR, fast refresh |
| `tairitsu-browser-worlds` | WIT world definitions from W3C WebIDL (DOM, Events, Fetch, Canvas, etc.) |
| `tairitsu-browser-wit-resolver` | WIT package resolution: version resolution, cloud fetching, caching |
| `tairitsu-packager` | CLI binary: dev server, build pipeline, visual regression, desktop packaging |
| `tairitsu-mcp` | MCP server for AI coding assistants |
| `tairitsu-e2e` | End-to-end testing: WebDriver + chromiumoxide |

## Documentation

- [System Overview](docs/en/system/overview.md)
- [Quick Start Guide](docs/en/guides/quick-start.md)
- [Getting Started Tutorial](docs/en/guides/getting-started.md)
- [Migration Guide](docs/en/guides/migration.md)
- Multilingual docs under `docs/` (en, zhs, zht, ja, ko, es, fr, ru, ar)

## License

Tairitsu is dual-licensed under MIT OR Apache-2.0.

## Name

"Tairitsu" (対立) means "conflict" or "opposition" in Japanese, from the rhythm game [Arcaea](https://arcaea.lowiro.com/).
