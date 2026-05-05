# PLAN: Cross-Platform VTty MCP Server for TUI Debugging

## Status: 🔧 Needs Fix — Windows ConPTY command routing

### Session Progress (2026-05-05)

- [x] Phase 1-4: VT100 screen, PTY backends, VttySession+Manager, MCP tools
- [x] `debug-api` and `vtty` added to default features
- [x] MCP JSON-RPC compliance fixed (no response to notifications, inputSchema type:object)
- [x] MCP connects successfully via `@modelcontextprotocol/sdk` (27 tools)
- [x] Rebased onto latest `origin/dev`, pushed cleanly
- [ ] **BUG: Windows ConPTY command routing** — `vtty_launch` fails on Windows
- [ ] Entelecheia TUI VTty integration test on Windows
- [ ] Delete Python vtty scripts after Rust replacement verified

---

## BUG: Windows ConPTY `CommandBuilder` passes entire command as executable name

### Symptom

```
ConPTY spawn failed: CreateProcessW `"\"cmd.exe /c echo hello\"\0"` in cwd ... failed: 
系统找不到指定的路径。 (os error 3)
```

### Root Cause

In `packages/packager/src/vtty/pty_win.rs:38`:

```rust
let mut cmd = CommandBuilder::new(command);
```

`CommandBuilder::new(command)` treats the entire `command` string (e.g. `"cmd.exe /c echo hello"`) as the **executable filename**. On Windows, `CreateProcessW` then tries to find a file literally named `"cmd.exe /c echo hello"` — which doesn't exist.

On Unix, the shell interprets the string, but on Windows ConPTY there is no shell wrapper.

### Fix Required

In `pty_win.rs`, the `spawn()` function needs to route through the system shell on Windows:

```rust
// BEFORE (broken):
let mut cmd = CommandBuilder::new(command);

// AFTER (fixed):
let mut cmd = if cfg!(target_os = "windows") {
    let mut c = CommandBuilder::new("cmd.exe");
    c.arg("/C");
    c.arg(command);
    c
} else {
    CommandBuilder::new(command)
};
```

This way:
- Windows: `cmd.exe /C <user command>` — `cmd.exe` is the executable, the rest is args
- Unix: unchanged behavior (shell interprets the command string)

**IMPORTANT**: This is a cfg check at compile time. Since this file is `pty_win.rs` (only compiled on Windows), you could also just always use `cmd.exe /C` without the cfg check. But the cfg guard is safer if the code is ever shared.

### Verification

After the fix, test from opencode (tairitsu MCP must be rebuilt):

```
vtty_launch(command="echo hello", rows=10, cols=80)
```

Should return a session_id. Then:

```
vtty_screenshot(session_id=<id>)
```

Should show "hello" in the terminal output.

Also test:
```
vtty_launch(command="cargo run -p entelecheia-tui -- --source-build", rows=40, cols=120)
```

This is the real use case — launching the TUI inside VTty for automated testing.

### File to Modify

| File | Change |
|------|--------|
| `packages/packager/src/vtty/pty_win.rs:38` | Wrap command through `cmd.exe /C` on Windows |

After fixing, rebuild and commit:

```bash
cargo build --package tairitsu-packager --features "debug-api,vtty"
# Test via MCP
# Then commit and push
```

## Motivation

The entelecheia TUI needs interactive debugging — launch it, observe the screen, send keystrokes, verify container status changes in real-time. The current Python vtty (`scripts/vtty/vtty_server_win.py`) uses pywinpty which is fragile and adds a non-Rust dependency. Meanwhile, entelecheia's own `packages/e2e_tests/src/vtty` already has a complete Rust `VttySession` (forkpty + vte VT100 parser), but it's Unix-only and not exposed as an MCP server.

The right home for this capability is **tairitsu packager** — it already has:
- A daemon mode with PID management, log streaming, readiness signaling
- An MCP server (`packages/packager/src/mcp/`) exposing browser automation tools to AI assistants
- Feature flags for native/dev-server/debug-api
- Cross-platform awareness (Windows + Unix in daemon code)

Adding a VTty MCP tool surface alongside the existing browser MCP creates a **unified debug surface**: WebUI (browser) + TUI (terminal) + native apps (future), all driven by the same MCP protocol.

## Architecture

```
┌─────────────────────────────────────────┐
│  tairitsu packager (CLI / daemon)        │
│                                          │
│  ┌──────────┐  ┌──────────────────────┐  │
│  │ Browser  │  │ VTty (NEW)           │  │
│  │ MCP      │  │                      │  │
│  │ (wry)    │  │  ConPTY (Windows)    │  │
│  │          │  │  forkpty (Unix)      │  │
│  └──────────┘  │  vte VT100 parser    │  │
│                │                      │  │
│                │  Tools:              │  │
│                │  - vtty_launch       │  │
│                │  - vtty_kill         │  │
│                │  - vtty_send_keys    │  │
│                │  - vtty_send_text    │  │
│                │  - vtty_screenshot   │  │
│                │  - vtty_wait         │  │
│                │  - vtty_resize       │  │
│                │  - vtty_list         │  │
│                │  - vtty_ping         │  │
│                └──────────────────────┘  │
└─────────────────────────────────────────┘
```

## Implementation Plan

### Phase 1: Extract & Generalize VT100 Screen (shared)

**Goal:** Move `Vt100Screen` from `entelecheia/packages/e2e_tests/src/vtty/` into a shared location.

**Changes:**
- Copy `vt100_screen.rs` + `vt100_grid.rs` (or whatever the grid/cell types are named) into `tairitsu/packages/packager/src/vtty/`
- Dependencies: add `vte` crate to packager's Cargo.toml (behind feature flag `"vtty"`)
- No platform-specific code here — pure VT100 state machine

### Phase 2: Platform PTY Backends

**Goal:** Implement platform-specific PTY session creation.

#### Unix Backend (`vtty_unix.rs`, cfg(unix))
- Reuse the existing `forkpty` approach from entelecheia e2e
- `VttyPty::new(command, cols, rows, env, cwd) -> Result<Self>`
  - Calls `libc::forkpty()` to get master fd + child PID
  - Spawns reader thread that reads from master fd and feeds `Vt100Screen`
  - `write()` writes to master fd
  - `resize()` calls `libc::ioctl(TIOCSWINSZ)`
  - `kill()` sends SIGTERM to child process group

#### Windows Backend (`vtty_win.rs`, cfg(windows))
- Use Windows ConPTY API via `windows-sys` crate
- `VttyPty::new(command, cols, rows, env, cwd) -> Result<Self>`
  - Calls `CreatePseudoConsole()` with anonymous pipe handles
  - `CreateProcessW()` with `PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE` in `STARTUPINFOEX`
  - Spawns reader thread that reads from output pipe and feeds `Vt100Screen`
  - `write()` writes to input pipe
  - `resize()` calls `ResizePseudoConsole()`
  - `kill()` calls `TerminateProcess()`
- Dependencies: add `windows-sys` with `Win32_System_Console`, `Win32_Foundation` features (behind `cfg(windows)`)

**Critical: Windows ConPTY gotchas (from Python debugging session)**
- `CreatePseudoConsole` takes `COORD` by value (4 bytes: two `c_short` packed as `c_uint`)
- Pipe handles passed to ConPTY must be inheritable (the opposite ends must be non-inheritable)
- `STARTUPINFOEXW` must have `lpAttributeList` properly initialized via `InitializeProcThreadAttributeList` + `UpdateProcThreadAttribute` with attribute `PROC_THREAD_ATTRIBUTE_PSEUDOCONSOLE` (0x00020016)
- `ClosePseudoConsole` is the cleanup function (not `CloseHandle` on the HPC)
- Reading output pipe should use overlapped I/O or a dedicated reader thread with small sleep intervals

### Phase 3: Unified `VttySession` + Session Manager

**Goal:** Wrap platform PTY + VT100 screen behind a unified interface.

```rust
pub struct VttySession {
    id: String,
    name: String,
    command: String,
    cols: u16,
    rows: u16,
    alive: AtomicBool,
    pty: VttyPty,        // platform-specific
    screen: Arc<Mutex<Vt100Screen>>,  // shared VT100 parser
}

pub struct VttyManager {
    sessions: HashMap<String, VttySession>,
    counter: AtomicU32,
}
```

- `VttyManager` lives in packager state, accessible from MCP handler
- Thread safety: `screen` behind `Arc<Mutex<>>`, PTY handles are Send+Sync
- Session lifecycle: launch → use → kill, state persisted to `target/vtty-state.json`

### Phase 4: MCP Tool Surface

**Goal:** Add VTty tools to the existing MCP server in `packages/packager/src/mcp/`.

Add a new module `vtty_tools.rs` alongside existing browser tools:

```rust
// Each tool maps 1:1 to existing vtty Python API
fn vtty_launch(command, cols, rows, env, cwd, name) -> session_id
fn vtty_kill(session_id) -> ()
fn vtty_send_keys(session_id, keys) -> ()
fn vtty_send_text(session_id, text) -> ()
fn vtty_mouse_click(session_id, col, row, button) -> ()
fn vtty_mouse_scroll(session_id, direction, amount) -> ()
fn vtty_screenshot(session_id) -> { text, rows, cols, alive }
fn vtty_wait(session_id, seconds, pattern, timeout) -> { found, alive }
fn vtty_resize(session_id, cols, rows) -> ()
fn vtty_list() -> [session_info]
fn vtty_ping(session_id) -> { alive, pid }
```

Feature gate: `vtty` feature (default off, enabled when packager is used for TUI debugging).

### Phase 5: CLI Integration

**Goal:** Add `tairitsu vtty` subcommand for standalone use.

```bash
# Start MCP server with VTty support
tairitsu mcp --features vtty

# Or via daemon
tairitsu dev --daemon --features vtty
```

No new subcommand needed — `tairitsu mcp` already starts the MCP server. Just add the vtty tools when the feature is enabled.

## Dependency Changes

```toml
# packages/packager/Cargo.toml

[target.'cfg(unix)'.dependencies]
libc = "0.2"

[target.'cfg(windows)'.dependencies]
windows-sys = { version = "0.59", features = [
    "Win32_System_Console",
    "Win32_Foundation",
    "Win32_System_Threading",
    "Win32_Security",
] }

[features]
vtty = ["vte"]
```

## Key Risks & Mitigations

| Risk | Mitigation |
|------|-----------|
| ConPTY API is tricky on Windows | Test with simple `cmd /c echo` first, then progressively more complex commands |
| VT100 parser doesn't handle all crossterm sequences | `vte` crate is the standard; it handles CSI/OSC/DCS. Crossterm uses standard sequences |
| Thread safety of PTY handles | Arc<Mutex> on screen, reader thread owns the pipe read side exclusively |
| MCP server startup adds latency | VTty tools are lazy — no PTY created until `vtty_launch` is called |

## Acceptance Criteria

- [ ] `vtty_launch(command="cmd /c echo hello")` works on Windows
- [ ] `vtty_launch(command="/bin/sh -c echo hello")` works on Unix
- [ ] `vtty_screenshot()` returns correct text after `echo`
- [ ] `vtty_send_keys("Enter")` sends carriage return
- [ ] Can launch entelecheia TUI via vtty and observe splash screen
- [ ] Can navigate TUI menus via vtty_send_keys
- [ ] Session state survives MCP server restart (reattach via pid check)
- [ ] Registered in opencode config as `entelecheia-vtty` pointing to `tairitsu mcp`

## Migration Path for Entelecheia

Once tairitsu packager has VTty MCP:

1. Update `~/.config/opencode/opencode.json`:
   ```json
   "entelecheia-vtty": {
       "type": "local",
       "command": ["tairitsu", "mcp", "--features", "vtty"],
       "enabled": true
   }
   ```
2. Delete `scripts/vtty/vtty_server_win.py` and `scripts/vtty/vtty_server.py`
3. Update `scripts/vtty/README.md` to point to tairitsu
4. Remove `pywinpty` / `mcp` / `pyte` Python dependencies from entelecheia

## References

- Existing Rust VTty: `entelecheia/packages/e2e_tests/src/vtty/` (forkpty + vte, Unix-only)
- Existing Python VTty: `entelecheia/scripts/vtty/vtty_server.py` (Unix), `vtty_server_win.py` (Windows, pywinpty)
- Tairitsu MCP: `tairitsu/packages/packager/src/mcp/`
- Tairitsu Daemon: `tairitsu/packages/packager/src/daemon/`
- ConPTY API: https://learn.microsoft.com/en-us/windows/console/createpseudoconsole

---

# PLAN4: Fix Browser MCP — Daemon Discovery + Debug API Routing

## Status: 🔴 Diagnosed — Ready for Implementation

### Session Date: 2026-05-05

## Problem Statement

The `tairitsu-virtual-browser` MCP server (registered in opencode's `opencode.jsonc`) **cannot connect** to a running `tairitsu dev --daemon`. All `browser_*` tools fail with:

```
Browser tools require a running daemon. Start with: tairitsu dev --daemon
```

Even when the daemon is confirmed running and serving on `localhost:3000`.

## Root Cause Analysis (2 bugs)

### Bug 1: Debug API routes not compiled into dev server

**Evidence:** HTTP GET `http://localhost:3000/__tairitsu_debug/status` returns the SPA `index.html` (fallback), not JSON.

**Cause:** In `packages/packager/src/wasm/mod.rs:2185-2200`, all `/__tairitsu_debug/*` routes are gated behind:

```rust
#[cfg(feature = "debug-api")]
let app = {
    app.nest("/__tairitsu_debug", axum::Router::new()
        .route("/status", ...)
        .route("/snapshot", ...)
        .route("/screenshot", ...)
        // ... 8 routes total
};
```

When `tairitsu dev --daemon` is launched **without** `--features debug-api`, these routes don't exist. Every request falls through to the SPA fallback handler.

**Impact:** Even if the MCP successfully connects to the daemon URL, every tool call returns HTML instead of debug data.

### Bug 2: Ready-file path mismatch between daemon writer and MCP reader

**Evidence:** The ready file is written to:
```
D:\源代码\工程项目\celestia\tairitsu\target\tairitsu-packager.ready  ✅ exists, content = "ready:3000"
```

But the MCP server (`mcp/mod.rs:86-88`) reads from:
```rust
fn try_read_ready_port() -> Option<u16> {
    let ready_path = std::path::PathBuf::from("target")
        .join("tairitsu-packager.ready");
    // → resolves to {CWD}/target/tairitsu-packager.ready
}
```

**Cause:**
- **Daemon side** (`daemon/mod.rs:26-28`): writes to `project_root().join("target/tairitsu-packager.ready")`
  - `PROJECT_ROOT` is set during daemon startup; for `tairitsu dev --manifest-path <hikari>/Cargo.toml`, it resolves to **tairitsu repo root**
  - So file lands at `<tairitsu>/target/tairitsu-packager.ready`
- **MCP side**: reads from `{CWD}/target/tairitsu-packager.ready`
  - When opencode launches MCP via `cargo run --package tairitsu-packager -- mcp`, CWD = **workspace root** (tairitsu/) in ideal case
  - But opencode may set CWD to user home dir, project dir, or elsewhere — **unreliable**

**Impact:** MCP cannot discover the daemon's port → falls through to "no running daemon" error.

## Proposed Fixes

### Fix 1: Enable `debug-api` feature by default for `tairitsu dev` / `tairitsu dev --daemon`

**File:** `packages/packager/src/cli/mod.rs` (or wherever dev-server feature flags are configured)

**Change:** Add `debug-api` to the default feature set or to the `dev` / `daemon` command's implicit features:

```rust
// Option A: Add to default features in Cargo.toml
[features]
default = ["native", "debug-api"]  // was probably just ["native"]

// Option B: In cli/mod.rs, when --daemon is used, force-enable debug-api
if args.daemon {
    // Ensure debug-api routes are compiled in
}
```

**Verification:** After fix, `curl http://localhost:3000/__tairitsu_debug/status` should return JSON like `{"ok":true,...}` not HTML.

### Fix 2: Make ready-file discovery robust

**Option A (Recommended): Use PID file as fallback**

The daemon already writes a PID file at the same location as the ready file. The MCP should try multiple discovery strategies:

```rust
fn resolve_daemon_url() -> crate::Result<String> {
    // Strategy 1: Read ready file (current behavior, works when CWD matches)
    if let Some(port) = try_read_ready_port() {
        return Ok(format!("http://localhost:{}", port));
    }

    // Strategy 2: Search common project roots for ready/PID files
    for candidate_dir in &search_project_roots() {
        let p = candidate_dir.join("target").join("tairitsu-packager.ready");
        if let Some(port) = read_ready_port_from(&p) {
            return Ok(format!("http://localhost:{}", port));
        }
    }

    // Strategy 3: Check if daemon PID is alive (cross-platform)
    if let Some(pid) = try_read_pid_file() {
        if is_process_alive(pid) {
            return Err(...)  // Running but port unknown — suggest --port
        }
    }

    Err(...)
}

fn search_project_roots() -> Vec<PathBuf> {
    // 1. CWD
    // 2. TAIRITSU_PROJECT_ROOT env var (if set by opencode/integrator)
    // 3. Parent dirs up to workspace root (look for Cargo.toml)
    // 4. Common locations: ./target, ../target, ../../target
    // ...
}
```

**Option B (Simpler): Accept `--port` or `TAIRITSU_DAEMON_URL` env var**

Let the user/explicitly configure the URL:

```jsonc
// opencode.jsonc
{
    "mcp": {
        "tairitsu-virtual-browser": {
            "type": "local",
            "command": ["cargo", "run", "--package", "tairitsu-packager",
                        "--features", "debug-api,vtty", "--quiet", "--", "mcp"],
            "environment": {
                "TAIRITSU_DAEMON_URL": "http://localhost:3000"
            },
            "enabled": true
        }
    }
}
```

Then in `mcp/mod.rs`:
```rust
let base_url = if let Ok(url) = std::env::var("TAIRITSU_DAEMON_URL") {
    url
} else {
    resolve_daemon_url().await?
};
```

### Fix 3 (Nice-to-have): MCP should report connection errors clearly

Currently the error `"Browser tools require a running daemon"` is opaque. Improve to include *why* discovery failed:

```
[MCP] Cannot find tairitsu daemon:
  - Tried ready file: {cwd}/target/tairitsu-packager.ready → not found
  - Searched project roots: [<list>] → not found
  - Hint: Set TAIRITSU_DAEMON_URL env var, or ensure `tairitsu dev --daemon` is running
     with --features debug-api
```

## Implementation Order

1. **Fix 1 first** (debug-api feature) — without this, nothing else matters
2. **Fix 2** (ready-file discovery) — makes MCP work reliably across CWD contexts
3. **Fix 3** (error messages) — improves DX when things still go wrong

## Testing Checklist

After fixes, verify end-to-end:

```bash
# Terminal 1: Start daemon with debug-api
tairitsu dev --daemon --manifest-path ../hikari/examples/website/Cargo.toml --port 3000

# Terminal 2: Verify debug API
curl http://localhost:3000/__tairitsu_debug/status       # → JSON
curl http://localhost:3000/__tairitsu_debug/snapshot      # → accessibility tree

# Terminal 3: Restart opencode (picks up new opencode.jsonc config)
# Then use any browser_* tool from chat:
#   browser_navigate → http://localhost:3000/components/layer2/cascader/
#   browser_screenshot  → captures PNG
#   browser_snapshot   → accessibility tree
#   vtty_launch       → launches terminal session
```

## Files to Modify

| File | Change |
|------|--------|
| `packages/packager/Cargo.toml` | Add `debug-api` to default features (Fix 1) |
| `packages/packager/src/mcp/mod.rs` | Robust daemon discovery + `TAIRITSU_DAEMON_URL` env support (Fix 2+3) |
| `packages/packager/src/daemon/mod.rs` | Optionally: write ready file to additional well-known locations |
