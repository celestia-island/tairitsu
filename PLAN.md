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
| P1 | Library-usable browser client crate | Medium |
| P2 | Structured DOM extraction helpers | Medium |

---

## 6. Library-Usable Browser Client Crate (for Entelecheia opcua_browse)

**Context**: Entelecheia's Skemma agent has an `opcua_browse` tool that is currently a stub. Many industrial OPC-UA deployments expose web-based visualization dashboards (e.g., Ignition Perspective, Siemens WinCC WebUX, Prosys OPC-UA Simulation Server web UI, UA Cloud Viewer). The tool should use tairitsu's embedded browser to navigate these web interfaces and extract node tree data.

**Current**: Browser automation tools are embedded in `tairitsu-mcp` as an MCP server binary. They communicate with a tairitsu daemon via HTTP. The tools cannot be used as a library from other Rust crates.

**Needed**: 
1. Extract browser interaction logic into a reusable library crate `tairitsu-browser-client` that other crates (including entelecheia agents) can depend on directly.
2. Add structured data extraction helpers for common industrial dashboard patterns.

### Proposed Architecture

```
tairitsu-browser-client (new crate)
├── BrowserClient (struct)
│   ├── new(base_url: &str) -> Self
│   ├── navigate(url: &str) -> Result<()>
│   ├── snapshot(selector: Option<&str>) -> Result<AccessibilityTree>
│   ├── evaluate(js: &str) -> Result<serde_json::Value>
│   ├── click(selector: &str) -> Result<()>
│   ├── type_text(selector: &str, text: &str, submit: bool) -> Result<()>
│   ├── press_key(key: &str) -> Result<()>
│   ├── screenshot(element: Option<&str>, full_page: bool) -> Result<PngData>
│   ├── console_messages(level: Option<&str>) -> Result<Vec<ConsoleEntry>>
│   └── resize(width: u32, height: u32) -> Result<()>
├── AccessibilityTree (struct)
│   ├── nodes: Vec<AccessibilityNode>
│   └── to_json() -> serde_json::Value
└── PngData
    └── raw: Vec<u8>
```

### New Interfaces Needed

#### a) `browser_extract_table` helper

Many OPC-UA web viewers present node data in HTML tables. A high-level helper:

```rust
/// Extract structured data from an HTML table on the current page.
/// Returns rows as Vec<HashMap<String, String>> using header cells as keys.
async fn browser_extract_table(client: &BrowserClient, table_selector: &str) -> Result<Vec<HashMap<String, String>>>;
```

This would use `browser_evaluate` internally to run JS that:
1. Finds the table by selector
2. Reads `<thead>` for column names
3. Iterates `<tbody><tr>` for row data
4. Returns JSON

#### b) `browser_wait_for_selector` helper

Industrial dashboards often load asynchronously. Need:

```rust
/// Wait until a CSS selector matches at least one element on the page.
/// Polls accessibility snapshot until found or timeout.
async fn browser_wait_for_selector(
    client: &BrowserClient,
    selector: &str,
    timeout: Duration,
) -> Result<()>;
```

#### c) `browser_extract_json_ld` / `browser_extract_meta` helpers

For dashboards that embed structured data:

```rust
/// Extract JSON-LD or <meta> structured data from the current page.
async fn browser_extract_structured_data(client: &BrowserClient) -> Result<Vec<serde_json::Value>>;
```

### Entelecheia Integration

Once `tairitsu-browser-client` exists, entelecheia will:
1. Add to workspace Cargo.toml:
   ```toml
   tairitsu-browser-client = { path = "../tairitsu/packages/browser-client" }
   ```
2. Skemma's `opcua_browse` will use it:
   ```rust
   // packages/agents/skemma/src/mcp/tools/opcua_browse.rs
   let client = tairitsu_browser_client::BrowserClient::new(&daemon_url);
   client.navigate(&web_url).await?;
   client.wait_for_selector(".opcua-node-tree", Duration::from_secs(10)).await?;
   let tree = client.snapshot(None).await?;
   // Parse tree into OpcuaBrowseResult
   ```

### What Needs To Change In tairitsu

1. **New crate `tairitsu-browser-client`**: Move the HTTP client logic from `tairitsu-mcp/src/lib.rs` (the `http_get`/`http_post`/`http_post_fire_and_forget` methods) into a standalone library crate.
2. **Keep `tairitsu-mcp` as a thin MCP wrapper**: It should depend on `tairitsu-browser-client` and expose the same MCP tools.
3. **Add the 3 helper functions** (`extract_table`, `wait_for_selector`, `extract_structured_data`) to the client crate.
4. **Add `AccessibilityTree` parsing**: The current `browser_snapshot` returns raw JSON. Add typed deserialization.

### Effort Estimate

| Task | Lines | Effort |
|------|-------|--------|
| Extract `BrowserClient` from `tairitsu-mcp` | ~200 | Small |
| Add `wait_for_selector` | ~50 | Small |
| Add `extract_table` | ~100 | Medium |
| Add `extract_structured_data` | ~80 | Medium |
| Typed `AccessibilityTree` | ~150 | Medium |
| Update `tairitsu-mcp` to use client crate | ~100 | Small |

---

## Integration Point

Entelecheia will add this to workspace Cargo.toml:
```toml
tairitsu-runtime = { path = "../tairitsu/packages/runtime" }
```

And create a `packages/shared/plugin_host/` crate that wraps tairitsu-runtime with entelecheia-specific host functions (HTTP client, event forwarding, AI query, KV store).

---

## VTTY: Fix PTY ECHO for Interactive TUI (crossterm) Keyboard Input

### Problem

VTTY creates a real PTY (`/dev/pts/N`) with `TERM=xterm-256color`. Raw mode (`cfmakeraw` / `tcsetattr`) works correctly on the PTY slave (verified via `stty raw -echo` + `dd` test). A minimal crossterm program (`enable_raw_mode()` + `event::poll()` + `event::read()`) correctly detects Enter key (`\r`) sent via `vtty_send_keys`.

**However**: interactive crossterm-based TUIs (like entelecheia-tui's splash screen modal dialogs) do not respond to keyboard input via VTTY. The language selection dialog, checklist confirmations, and other modal interactions are unresponsive to `Enter`, `Escape`, `Tab`, and arrow keys — even though those keys are confirmed to arrive at the PTY.

### Root Cause Analysis

Evidence from debugging entelecheia-tui splash screen (language dialog):

1. **Keys arrive at the PTY**: Sending `Escape` via `vtty_send_keys` causes `^[` to appear in VTTY's screen output below the dialog. This is the raw ESC byte (`\x1b`) being rendered by VTTY's VTE screen emulator.
2. **ECHO is likely ON on the PTY slave**: In a properly raw-mode PTY, `ECHO` should be off — input bytes should never appear in the output stream. The presence of `^[` in VTTY's screen capture means the PTY slave's line discipline is echoing input back through the master, which the VTE parser then renders.
3. **Echoed escape sequences corrupt crossterm's event stream**: crossterm's `InternalEventReader` thread reads raw bytes from stdin (the PTY slave). If the terminal driver echoes input bytes back to the output stream, they become visible in VTTY's screen but do NOT affect crossterm's event reading directly. However, the TUI's complex event loop (30ms poll + frame rendering) may encounter timing issues where the echoed bytes appear between event reads, or the terminal's output buffer gets polluted in ways that affect the rendering pipeline.
4. **Minimal crossterm test works**: The test sends Enter, disables raw mode, and prints result. It's a single-shot read with no rendering loop. The TUI has a continuous event-render loop that's more sensitive to buffer state.

### Fix: Ensure PTY Slave Starts With ECHO Off

The `portable-pty` crate opens the PTY pair via `native_pty_system().openpty(size)`. After `pair.slave.spawn_command(cmd)`, the child process inherits the PTY slave's default termios state, which typically has `ECHO` enabled (the system default for pseudo-terminals is often cooked mode).

Crossterm's `enable_raw_mode()` calls `tcsetattr(STDIN_FILENO, TCSANOW, ...)` to disable `ECHO`, `ICANON`, `ISIG`, etc. This **should** work on a real PTY slave — and our tests confirm it does for simple programs.

But the entelecheia-tui issue suggests that something is interfering. Possible causes:

- **`portable-pty` may set `ECHO` after spawn**: Some PTY libraries re-enable echo on the master side.
- **The VTTY MCP reader thread may be configuring the master**: The dedicated reader thread (`reader_loop` in `pty_unix.rs`) opens a `dup(master_fd)` for reading. Some older PTY implementations can interfere with slave settings.
- **Bash startup may re-enable echo**: The shell spawned by VTTY (`/bin/bash -c "<command>"`) may restore terminal settings during initialization.

### Recommended Actions

1. **Explicitly disable ECHO on the PTY slave after spawn**:

   In `packages/mcp/src/vtty/pty_unix.rs` and `packages/packager/src/vtty/pty_unix.rs`, after `pair.slave.spawn_command(cmd)`:

   ```rust
   // Disable echo on the PTY slave to prevent input bytes from appearing
   // in the output stream. crossterm-based TUIs enable raw mode themselves,
   // but bash and other intermediate processes may re-enable echo.
   unsafe {
       use std::os::unix::io::AsRawFd;
       let slave_fd = pair.slave.as_raw_fd().unwrap_or(-1);
       if slave_fd >= 0 {
           let mut termios: libc::termios = std::mem::zeroed();
           if libc::tcgetattr(slave_fd, &mut termios) == 0 {
               termios.c_lflag &= !(libc::ECHO | libc::ECHONL | libc::ICANON);
               termios.c_iflag &= !(libc::ICRNL | libc::INLCR);
               libc::tcsetattr(slave_fd, libc::TCSANOW, &termios);
           }
       }
   }
   ```

   Note: this keeps `ISIG` (signal generation) enabled so that Ctrl+C still works. Full `cfmakeraw` would disable signals too, preventing Ctrl+C from reaching the child process.

2. **Add a smoke test for interactive TUI input**:

   In `packages/mcp/src/vtty/mod.rs` tests, add a test that:
   - Launches a minimal crossterm program with a modal dialog loop
   - Sends `Enter` via `send_keys`
   - Verifies the dialog is dismissed (via screenshot text change)

3. **Check `native_pty_system()` behavior**:

   Verify that `portable-pty`'s `openpty()` does not configure the master to echo. If it does, we need to work around it or configure the master differently.

### Priority

P1 — blocks interactive TUI testing via VTTY in entelecheia's CI/CD pipeline.
