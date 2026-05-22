# Tairitsu Enhancement Plan for Entelecheia Plugin System

> **Purpose**: Entelecheia's `domain_agent_runtime` needs a WASI plugin system. Tairitsu already provides the core WASM Component Model runtime (wasmtime v43, WIT, Image/Container/Registry). This plan tracks what tairitsu needs to add or expose.

---

## What Entelecheia Needs From Tairitsu

### 1. Async Container Invocation

**Current**: `Container::call_guest_raw_desc()` is synchronous (`&mut self`).
**Needed**: Async variant for use in tokio runtime (webhook handling is async).

**Approach options**:
- A) Add `call_guest_raw_desc_async()` using `tokio::task::spawn_blocking` internally
- B) Make Container `Send + Sync` and let callers wrap in `spawn_blocking`

**Recommendation**: B is simpler. Document the pattern.

### 2. Fuel / Resource Limits

**Current**: No fuel metering or memory limits per Container.
**Needed**: Prevent runaway plugins from consuming unbounded resources.

**Approach**:
- Expose `wasmtime::Store::set_fuel()` and `Store::set_epoch_deadline()` in `Container::builder()`
- Add `ContainerBuilder::with_fuel_limit(u64)` and `ContainerBuilder::with_memory_limit(usize)`

### 3. Container State Query

**Current**: No way to check if a Container is running/stopped/errored.
**Needed**: Health checking for plugins.

**Approach**:
- Add `Container::state() -> ContainerState` with enum `Created | Running | Stopped | Error(String)`

### 4. Multiple WIT Worlds

**Current**: `Container::builder()` takes a single `with_guest_initializer()` closure.
**Needed**: Support different WIT worlds for different plugin types (webhook-handler vs bot-handler vs layer3).

**Approach**: Current API is flexible enough — the `with_guest_initializer` closure can bind any WIT world. Document the multi-world pattern.

### 5. Re-export wasmtime Types

**Current**: Callers need to depend on wasmtime directly for `Linker`, `Store`, etc.
**Needed**: Re-export key types from `tairitsu-runtime` to avoid version conflicts.

**Approach**: Add `pub use wasmtime::{Engine, Store, Linker, Config, component::Component};` in `runtime/src/lib.rs`.

---

## WIT Interface for Entelecheia Plugins

Entelecheia will define its own WIT package `entelecheia:plugin`. Tairitsu does not need to define this — it only needs to support loading arbitrary WIT packages.

```wit
package entelecheia:plugin;

interface host-api {
  http-request: func(method: string, url: string, headers: string, body: string) -> result<string, string>;
  forward-event: func(event-json: string) -> result<_, string>;
  query-ai: func(message: string, context: option<string>) -> result<string, string>;
  log: func(level: string, message: string);
  config-get: func(key: string) -> option<string>;
  kv-get: func(key: string) -> option<string>;
  kv-set: func(key: string, value: string) -> result<_, string>;
}

interface webhook-handler {
  name: func() -> string;
  handle-request: func(method: string, path: string, headers: string, body: string) -> result<string, string>;
}

interface bot-handler {
  name: func() -> string;
  on-message: func(platform: string, message: string) -> result<option<string>, string>;
}

world layer2-plugin {
  import host-api;
  export webhook-handler;
}

world layer2-bot {
  import host-api;
  export bot-handler;
}
```

---

## Priority Order

| Priority | Item | Effort |
|----------|------|--------|
| P0 | Re-export wasmtime types | Trivial |
| P0 | Document multi-world pattern | Trivial |
| P1 | Fuel/resource limit API | Medium |
| P1 | Container state query | Small |
| P2 | Async invocation guidance | Small |

---

## Integration Point

Entelecheia will add this to workspace Cargo.toml:
```toml
tairitsu-runtime = { path = "../tairitsu/packages/runtime" }
```

And create a `packages/shared/plugin_host/` crate that wraps tairitsu-runtime with entelecheia-specific host functions (HTTP client, event forwarding, AI query, KV store).
