# tairitsu-web

Web platform implementations and browser integration for Tairitsu.

## Overview

`tairitsu-web` provides platform implementations for running Tairitsu applications in web browsers. It supports both WIT-backed (Component Model) and direct web-sys backends.

## Platform Implementations

### WitPlatform

WIT-backed platform using the WebAssembly Component Model.

**Features:**
- Type-safe WIT bindings
- Opaque handle pattern (u64)
- Efficient handle caching
- Batch DOM operations

**Target:** `wasm32-wasip2`

```rust
use tairitsu_web::WitPlatform;

let platform = WitPlatform::new()?;
platform.mount_vnode_to_app(&vnode)?;
```

### BrowserPlatform

Direct web-sys bindings for native-feeling browser interaction.

**Features:**
- Direct DOM access via web-sys
- Event handling
- CSS manipulation
- Canvas/WebGL support

**Target:** `wasm32-unknown-unknown`

## Modules

### wit_platform

WIT-backed platform implementation with:
- Opaque handles (`WitElement`, `WitEvent`)
- Handle caching for performance
- Event callbacks
- Style operations

### handle_cache

Performance optimization for caching DOM handles:
- Style handle caching
- Cache statistics
- Thread-local storage

### batch_ops

Batch DOM operations for performance:
- Group style updates
- Group attribute updates
- Reduce WIT round-trips

### router

File-system based routing:
- Static routes
- Dynamic routes (`:id`)
- Wildcard routes (`*`)
- Middleware support

### i18n

Internationalization support:
- Multi-language resources
- Context-based translation
- Pluralization

## Feature Flags

- `wit-bindings`: Enable WIT platform (default for wasm32-wasip2)
- `browser`: Enable direct browser platform
- `ssr`: Enable SSR platform
- `router`: Enable routing module
- `i18n`: Enable i18n module

## Usage

### Basic Setup

```rust
use tairitsu_web::WitPlatform;
use tairitsu_vdom::VNode;

#[no_mangle]
pub extern "C" fn run() {
    let platform = WitPlatform::new().unwrap();
    let vnode = render_app();
    platform.mount_vnode_to_app(&vnode).unwrap();
}
```

### Event Handling

```rust
use tairitsu_vdom::MouseEvent;

button {
    onclick: |e: MouseEvent| {
        println!("Clicked at: {}, {}", e.client_x, e.client_y);
    },
    "Click me"
}
```

### Styling

```rust
use tairitsu_style::{Classes, Style};

div {
    class: Classes::from(["container", "active"]),
    style: Style::new()
        .background("blue")
        .color("white")
        .padding("10px"),
    "Styled content"
}
```

## Performance

The web package includes several performance optimizations:

1. **Handle Caching**: Reduces WIT calls for repeated style operations
2. **Batch Operations**: Groups DOM updates for efficiency
3. **Opaque Handles**: Lightweight u64 references instead of complex objects
4. **Event Delegation**: Efficient event handling

## See Also

- [tairitsu-vdom](../vdom/): Virtual DOM implementation
- [tairitsu-hooks](../hooks/): Reactive state management
- [tairitsu-browser-worlds](../browser-worlds/): WIT definitions
- [tairitsu-browser-glue](../../browser-glue/): Browser runtime host
