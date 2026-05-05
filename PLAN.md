# PLAN — Remaining Work

---

## 1. VTty: Windows ConPTY Command Routing Bug

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

### Fix

In `pty_win.rs`, route through the system shell on Windows:

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

| File | Change |
|------|--------|
| `packages/packager/src/vtty/pty_win.rs:38` | Wrap command through `cmd.exe /C` on Windows |

### Verification

```bash
cargo build --package tairitsu-packager --features "debug-api,vtty"
vtty_launch(command="echo hello", rows=10, cols=80)
vtty_screenshot(session_id=<id>)  # Should show "hello"
```

---

## 2. VTty: Remaining Tasks

- [ ] Entelecheia TUI VTty integration test on Windows
- [ ] Delete Python vtty scripts after Rust replacement verified (`scripts/vtty/vtty_server_win.py`, `scripts/vtty/vtty_server.py`)

### VTty Acceptance Criteria (Unchecked)

- [ ] `vtty_launch(command="cmd /c echo hello")` works on Windows
- [ ] `vtty_launch(command="/bin/sh -c echo hello")` works on Unix
- [ ] `vtty_screenshot()` returns correct text after `echo`
- [ ] `vtty_send_keys("Enter")` sends carriage return
- [ ] Can launch entelecheia TUI via vtty and observe splash screen
- [ ] Can navigate TUI menus via vtty_send_keys
- [ ] Session state survives MCP server restart (reattach via pid check)

---

## 3. Browser MCP: Daemon Discovery + Debug API Routing

### Bug 1: Debug API routes not compiled into dev server

HTTP GET `http://localhost:3000/__tairitsu_debug/status` returns SPA `index.html` (fallback), not JSON.

**Cause:** In `packages/packager/src/wasm/mod.rs:2185-2200`, all `/__tairitsu_debug/*` routes are gated behind `#[cfg(feature = "debug-api")]`. When `tairitsu dev --daemon` is launched without `--features debug-api`, these routes don't exist.

**Fix:** Add `debug-api` to default features in `packages/packager/Cargo.toml`, or force-enable it when `--daemon` is used.

| File | Change |
|------|--------|
| `packages/packager/Cargo.toml` | Add `debug-api` to default features |
| `packages/packager/src/cli/mod.rs` | Optionally: force-enable for `--daemon` |

### Bug 2: Ready-file path mismatch between daemon writer and MCP reader

Daemon writes to `project_root().join("target/tairitsu-packager.ready")`, but MCP reads from `{CWD}/target/tairitsu-packager.ready` — unreliable if CWD differs.

**Fix:** Make discovery robust — try multiple strategies:

1. Read ready file from CWD (current)
2. Search common project roots (env `TAIRITSU_PROJECT_ROOT`, parent dirs with `Cargo.toml`)
3. Accept `TAIRITSU_DAEMON_URL` env var as override
4. Report clear error messages on failure

| File | Change |
|------|--------|
| `packages/packager/src/mcp/mod.rs` | Robust daemon discovery + `TAIRITSU_DAEMON_URL` env support |
| `packages/packager/src/daemon/mod.rs` | Optionally: write ready file to additional well-known locations |

### Implementation Order

1. Fix 1 (debug-api feature) — without this, nothing else matters
2. Fix 2 (ready-file discovery)
3. Error messages improvement

---

## 4. Website: Pending Enhancements

### P1 — Enhancement

| # | Item | Status |
|---|------|--------|
| 15 | **Dynamic markdown rendering** — pulldown-cmark → VNode (dep exists, unused) | 🔄 |
| 17 | **Sidebar item icons** — SVG icons per menu item | 🔄 |

### P2 — Polish

| # | Item | Details |
|---|------|---------|
| 19 | **state_test.rs stub handlers** | oninput TODO, dead remove buttons (not in production tree) |
| 20 | **Logo is unicode char** | `\u{273F}` instead of actual image |
| 21 | **No favicon.ico verified** | Referenced in Cargo.toml |
| 22 | **Keyboard navigation** | Escape to close drawer, arrow keys for menu |

### P3 — Infrastructure

| # | Gap |
|---|-----|
| 23 | Dynamic doc loading missing (all content compiled into WASM) |
| 24 | i18n.rs not wired to all content pages (only not_found.rs uses it) |
| 25 | No keyboard navigation |
| 26 | No search functionality |
