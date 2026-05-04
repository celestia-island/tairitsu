# PLAN: Cross-Platform VTty MCP Server for TUI Debugging

## Status: 📋 Planning

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
