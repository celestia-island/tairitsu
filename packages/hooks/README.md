# tairitsu-hooks

Reactive state management hooks for Tairitsu components.

## Overview

`tairitsu-hooks` provides a React/Dioxus-inspired hook system for managing component state and side effects in Tairitsu applications.

## Available Hooks

### State Management

#### `use_state`

Simple state management with getter and setter.

```rust
use tairitsu_hooks::use_state;

let (count, set_count) = use_state(0);
*count.borrow() += 1;
set_count(42);
```

#### `use_signal`

Reactive signal with automatic change tracking.

```rust
use tairitsu_hooks::use_signal;

let signal = use_signal(|| 0);
assert_eq!(signal.get(), 0);
signal.set(42); // Triggers re-render
```

#### `use_ref`

Persistent reference that survives across renders.

```rust
use tairitsu_hooks::use_ref;

let counter = use_ref(0);
*counter.current_mut() += 1;
```

### Side Effects

#### `use_effect`

Run side effects on mount and when dependencies change.

```rust
use tairitsu_hooks::use_effect;

use_effect(move || {
    // Setup
    let element = get_element();
    setup_listeners(element);

    // Cleanup
    || {
        remove_listeners(element);
    }
});
```

#### `use_callback`

Memoize callbacks to prevent unnecessary recreations.

```rust
use tairitsu_hooks::use_callback;

let callback = use_callback(
    move || Rc::new(|| println!("Hello")),
    (), // Dependencies
);
```

### Performance

#### `use_memo`

Memoize expensive computations.

```rust
use tairitsu_hooks::use_memo;

let memo = use_memo(|| expensive_computation());
```

### Context

#### `use_context` / `provide_context`

Dependency injection pattern.

```rust
use tairitsu_hooks::{provide_context, use_context};

provide_context("user_data".to_string());
let data = use_context::<String>();
```

### Global State

#### `use_store`

Global state management with subscriptions.

```rust
use tairitsu_hooks::use_store;

store!(APP_STORE, AppState {
    count: 0,
    user: None,
});

let state = use_store!(APP_STORE());
state.count = 42;

// Subscribe to changes
let handle = state.subscribe(|s| {
    println!("State changed: {:?}", s);
});
```

## Patterns

### Component State

For local component state, use `use_signal`:

```rust
pub fn Counter() -> VNode {
    let (count, set_count) = use_signal(|| 0);

    rsx! {
        button {
            onclick: move |_| {
                let current = count.get();
                set_count.set(current + 1);
            },
            "Count: {count.get()}"
        }
    }
}
```

### Derived State

Use `use_memo` for computed values:

```rust
let (width, set_width) = use_signal(|| 10);
let (height, set_height) = use_signal(|| 20);

let area = use_memo(move || {
    width.get() * height.get()
});
```

### Global State

For app-wide state, use `use_store`:

```rust
store!(THEME_STORE, ThemeState {
    mode: "light".to_string(),
});

pub fn ThemeButton() -> VNode {
    let theme = use_store!(THEME_STORE());

    rsx! {
        button {
            onclick: move |_| {
                theme.mode = if theme.mode == "light" {
                    "dark".to_string()
                } else {
                    "light".to_string()
                };
            },
            "Toggle theme"
        }
    }
}
```

## Performance Tips

1. **Prefer `use_signal` over `use_state`** for reactive updates
2. **Use `use_memo`** for expensive computations
3. **Use `use_callback`** for event handlers passed to children
4. **Avoid deep nesting** of reactive signals

## See Also

- [tairitsu-vdom](../vdom/): Virtual DOM integration
- [tairitsu-web](../web/): Platform implementations
- [Reactivity Guide](../../docs/en-US/guides/reactivity.md): Detailed patterns
