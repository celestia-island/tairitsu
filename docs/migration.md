# Migration guide: `web` feature → `wit-bindings` feature

This guide explains how to migrate a Tairitsu application from the `web` feature
(which uses `wasm-bindgen` / `web-sys`) to the `wit-bindings` feature (which uses
the `tairitsu-browser:full` WIT world and the WebAssembly Component Model).

---

## When to use which backend

| Situation | Recommended feature |
|---|---|
| Target `wasm32-unknown-unknown`, existing JS bundler (webpack, Vite, …) | `web` |
| Target `wasm32-wasip2`, Component Model host (browser-glue or custom) | `wit-bindings` |
| Native unit tests that must instantiate a `WebPlatform` stub | `web` |
| Maximum portability / future WIT ecosystem | `wit-bindings` |

Both features are **mutually exclusive** in practice: you choose one build target
per binary.  Your application logic that depends only on `Platform` works unchanged
with either backend.

---

## Step-by-step migration

### 1. Change the Cargo dependency feature flags

```toml
# Before (wasm-bindgen path)
[dependencies]
tairitsu-web = { version = "0.1", features = ["web"] }

# After (WIT component-model path)
[dependencies]
tairitsu-web = { version = "0.1", features = ["wit-bindings"] }
```

### 2. Change the build target

```sh
# Before
cargo build --target wasm32-unknown-unknown

# After
cargo build --target wasm32-wasip2
```

The `wit-bindings` feature links against WIT-generated `extern "C"` trampolines
that expect the Component Model ABI, so the binary **must** target `wasm32-wasip2`
(or another Component Model-capable target).  Attempting to run it without a
matching host will result in a link error.

### 3. Change the platform instantiation

```rust
// Before — `web` feature
use tairitsu_web::WebPlatform;

let platform = WebPlatform::new(); // infallible

// After — `wit-bindings` feature
use tairitsu_web::WitPlatform;

let platform = WitPlatform::new()?; // returns Err on non-wasm32
```

`WitPlatform::new()` returns `anyhow::Result<WitPlatform>`.  On `wasm32` targets
the call always succeeds; on native hosts it returns `Err` (useful for conditional
code paths in libraries that support both backends at compile time).

### 4. Change the element / event types

The concrete `Element` and `Event` associated types change, but because all
application code should operate through the `Platform` trait and the
`ElementHandle` / `EventHandle` / `EventData` traits, no further changes are
required in most cases.

```rust
// Works unchanged for both backends
fn mount<P: Platform>(platform: &P, root: &P::Element) {
    let div = platform.create_element("div");
    platform.append_child(root, &div);
}
```

If you have code that directly holds `WebElement` or depends on `web-sys` types
(e.g. `HtmlElement`), you will need to abstract that behind a trait or gate it
with a `#[cfg(feature = "web")]` block.

### 5. Update the host / runtime

The `tairitsu-browser:full` WIT world must be satisfied by the host environment.
In a browser context, use `packages/browser-glue` as the JavaScript/TypeScript
host implementation.  Alternatively, implement the WIT interfaces in a custom
host (e.g. for server-side rendering or testing).

---

## Compatibility notes

- **Shared application logic**: Any code that only uses the `Platform` trait,
  `vdom`, `hooks`, and `rsx!` macros needs **no changes**.
- **Feature flags at the library level**: If you are writing a library crate that
  wants to support both backends, gate the backend-specific code:
  ```rust
  #[cfg(feature = "web")]
  fn init_web() -> impl tairitsu_vdom::Platform { tairitsu_web::WebPlatform::new() }

  #[cfg(feature = "wit-bindings")]
  fn init_wit() -> anyhow::Result<impl tairitsu_vdom::Platform> {
      tairitsu_web::WitPlatform::new()
  }
  ```
- **`web` and `wit-bindings` must not be combined in a single binary**: they pull
  in incompatible runtime dependencies and target different ABIs.  Cargo feature
  unification may silently enable both when multiple dependency paths exist;
  verify with `cargo tree --features wit-bindings`.
