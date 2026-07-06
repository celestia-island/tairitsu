# System Overview

Tairitsu is a full-stack framework powered by the WASM Component Model. A single WASM component can run on the server (via the Container runtime), in the browser (via the VDOM runtime), or at the edge — all through the same WIT interface definitions.

## The Four Layers

```mermaid
graph TD
    subgraph L4["4. Tooling Layer"]
        T1["packager, dev server, MCP, visual diff, scripts"]
    end
    subgraph L3["3. Platform Layer"]
        P1["WitPlatform (WIT bindings)"]
        P2["WebPlatform (web-sys)"]
        P3["browser-glue (TypeScript ↔ WIT bridging)"]
    end
    subgraph L2["2. Runtime Layer"]
        R1["Container / Registry / Image lifecycle"]
        R2["WIT binding, dynamic invocation (RON + binary)"]
    end
    subgraph L1["1. Interface Layer"]
        I1["WIT world definitions, browser-worlds"]
        I2["W3C WebIDL → WIT code generation pipeline"]
    end
    L1 --> L2 --> L3 --> L4
```

## How a Request Flows

### Browser (Client Path)

```mermaid
graph TD
    A["User clicks a button"] --> B["DOM event fires"]
    B --> C["browser-glue captures event,<br/>converts to WIT ABI"]
    C --> D["WASM component receives typed event<br/>(MouseEvent / KeyboardEvent / ...)"]
    D --> E["Signal updates → VDOM diff → Patch operations"]
    E --> F["Patch applied via DomOps → DOM updated"]
```

### Server (SSR Path)

```mermaid
graph TD
    A["HTTP request arrives"] --> B["axum dev server or<br/>standalone wasmtime host"]
    B --> C["Container instantiates WASM component"]
    C --> D["Component renders VNode tree via WIT calls"]
    D --> E["SSR engine serializes to HTML string"]
    E --> F["Streaming response sent to client"]
```

## Core Design Decisions

### Why Component Model instead of wasm-bindgen?

| wasm-bindgen path | WIT path (Tairitsu) |
|:--|:--|
| Rust → wasm-bindgen → JS shim → browser | Rust → WIT → canonical ABI → browser (eventually native) |
| Tight coupling to JS runtime | Language-agnostic WIT interface |
| No server-side reuse | Same component works in any wasmtime host |
| Mature, stable ecosystem (Leptos, Dioxus, Yew) | Emerging, future-facing |

Tairitsu bets on the Component Model becoming the standard for browser-wasm interop, eliminating the need for wasm-bindgen's JS glue layer.

### Why Docker-like Image/Container/Registry?

WASM components need lifecycle management just like containers:

- **Image** = compiled `.wasm` binary + metadata (like a Docker image)
- **Container** = running instance with host-provided WIT imports (like a Docker container)
- **Registry** = collection of images and active containers (like Docker daemon)

This model enables:
- Hot-reload during development (swap Image, keep Container)
- Versioned deployment (tag images, roll back)
- Multi-tenant isolation (separate containers, shared host)
- Dynamic invocation (call into running components at runtime)

## Next Steps

- [Runtime & Container Model](runtime.md) — deep dive into Container/Image/Registry
- [VDOM & Rendering](vdom.md) — how the browser-side VDOM works
- [WIT Pipeline](wit-pipeline.md) — W3C WebIDL → WIT generation
- [Web Backends](web-backends.md) — dual WitPlatform / WebPlatform strategy
- [Browser Glue](browser-glue.md) — TypeScript bridging layer
