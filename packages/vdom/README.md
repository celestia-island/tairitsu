# tairitsu-vdom

Virtual DOM implementation for Tairitsu WASM browser framework.

## Overview

`tairitsu-vdom` provides a lightweight Virtual DOM implementation designed for WebAssembly components using the Component Model. It features efficient diffing, patching, and platform abstraction for browser DOM manipulation.

## Core Components

### VNode

The virtual DOM tree is composed of `VNode` variants:

- **VElement**: Represents HTML elements with attributes, styles, events, and children
- **VText**: Represents text nodes
- **VFragment**: Represents document fragments for grouping children

### Platform Trait

Abstract platform interface for DOM operations. Implementations:

- `WitPlatform`: WIT-backed browser platform (wasm32)
- `BrowserPlatform`: Direct web-sys bindings
- `SsrPlatform`: Server-side rendering platform

## Usage

### Creating Virtual DOM Trees

```rust
use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

let vnode: VNode = rsx! {
    div { class: "container",
        h1 { "Hello, Tairitsu!" }
        p { "A WASM browser framework." }
    }
};
```

### Diffing and Patching

```rust
use tairitsu_vdom::{diff::diff, Platform};

let old: VNode = rsx! { div { "Old content" } };
let new: VNode = rsx! { div { "New content" } };

let patches = diff(Some(&old), &new);
platform.apply_patches(&parent, &patches)?;
```

## Event Handling

Events are typed and include:

- `MouseEvent`: click, mousemove, etc.
- `KeyboardEvent`: keydown, keyup, etc.
- `InputEvent`: Input field changes
- `FocusEvent`: focus, blur
- `TouchEvent`: Touch events (mobile)

## Features

- **Efficient diffing**: O(n) tree comparison algorithm
- **Minimal patches**: Only generates necessary DOM updates
- **Event delegation**: Efficient event handling through delegation
- **Opaque handles**: Type-safe element references via u64 handles
- **Platform agnostic**: Works with multiple backend implementations

## Performance

The VDOM is optimized for:

- Fast diffing with keyed children
- Minimal memory allocation
- Efficient batch operations
- Cache-friendly data structures

## See Also

- [tairitsu-web](../web/): Platform implementations
- [tairitsu-hooks](../hooks/): Reactive state management
- [tairitsu-style](../style/): Styling utilities
