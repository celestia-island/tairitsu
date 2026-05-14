# Tairitsu

**Full-stack framework powered by the WASM Component Model.** Write components once, run them anywhere — server, browser, edge. All communication typed via WIT.

---

## Why Tairitsu?

Tairitsu unifies two worlds that are typically separate:

| Traditional stack | Tairitsu |
|:--|:--|
| Backend: Node/Go/Rust server | Container runtime (native wasmtime) |
| Frontend: React/Vue/Svelte | VDOM + hooks — compiled to `wasm32-wasip2` |
| Protocol: REST/GraphQL/JSON | WIT interfaces: typed at compile time, zero serialization |
| Build: Webpack/Vite/Trunk | `tairitsu dev` / `tairitsu build` — one tool for both targets |
| Deploy: Docker + CDN | `tairitsu deploy` — push to Registry, serve anywhere |

**The same WASM component** talks to a server database, renders HTML via SSR, and handles browser DOM events — all through a single WIT interface definition.

---

## Three Faces of Tairitsu

```
┌──────────────────────────────────────────────────────────┐
│                                                          │
│  📦 tairitsu-core    Server runtime                      │
│     Container / Registry / WIT bindings / Dynamic calls  │
│      docker-like WASM lifecycle                        │
│                                                          │
│  🎨 tairitsu-web     Client framework + SSR              │
│     VDOM / hooks / rsx! / scss! / Suspense / i18n        │
│      React-like, compiled to wasm32-wasip2             │
│                                                          │
│  🔧 tairitsu-cli     Build / dev / deploy toolchain       │
│     Dev server / HMR / visual diff / MCP / VTty           │
│      Vite-like, but understands WIT                    │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

---

## Quick Start

```bash
# Install the CLI
cargo install tairitsu-packager

# Create a project
tairitsu new my-app
cd my-app

# Start dev server (hot reload, browser VDOM)
tairitsu dev

# Build for production
tairitsu build
```

Open `http://localhost:3000`.

---

## A Component in 30 Seconds

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

---

## Architecture

```
┌──────────────┐    WIT interface     ┌──────────────┐
│  Host (Rust)  │◀══════════════════▶│ Guest (WASM)  │
│               │                     │               │
│  Container    │    typed calls      │  Component    │
│  Registry     │    zero-ser         │  VDOM tree    │
│  Platform     │                     │  Event logic  │
└──────────────┘                     └──────────────┘
```

---

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

---

## Documentation

- [Guides](docs/en/guides/index.md) — tutorials, quick start, migration
- [System Architecture](docs/en/system/overview.md) — runtime model, WIT pipeline, backends
- [Package Map](docs/en/components/index.md) — workspace crate hierarchy

---

## Compared to...

|  | Tairitsu | Leptos | Dioxus | wasmCloud |
|:--|:--|:--|:--|:--|
| Paradigm | VDOM | Fine-grained reactive | VDOM | Actor/Lattice |
| WASM model | Component Model | wasm-bindgen | wasm-bindgen | Component Model |
| Server runtime | Built-in Container | Server functions | Server functions | Full platform |
| SSR | Streaming + HMR | In-order + streaming | Static + hydration | N/A |
| Browser binding | WIT (WebIDL→WIT) | web-sys | web-sys | N/A |
| Deploy target | Any wasmtime host | WASM server | WASM server | Distributed cluster |
| CSS | scss! macro | Stylers (Rust) | Stylers (Rust) | N/A |

---

## License

MIT OR Apache-2.0
