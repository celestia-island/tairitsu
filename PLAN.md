# PLAN — Remaining Work

---

## 1. VTty: Windows ConPTY Command Routing Bug ✅ DONE

Already fixed in `packages/packager/src/vtty/pty_win.rs:38` — command is routed through `cmd.exe /C` on Windows.

---

## 2. VTty: Remaining Tasks ✅ DONE

- [x] Python vtty scripts already deleted (not in tree)
- [x] Rust VTty replacement in place (`packages/packager/src/vtty/`)

### VTty Acceptance Criteria

- [x] `vtty_launch(command="cmd /c echo hello")` works on Windows (cmd.exe /C routing)
- [x] `vtty_launch(command="/bin/sh -c echo hello")` works on Unix
- [x] `vtty_screenshot()` returns correct text after `echo`
- [x] `vtty_send_keys("Enter")` sends carriage return
- [ ] Can launch entelecheia TUI via vtty and observe splash screen
- [ ] Can navigate TUI menus via vtty_send_keys
- [ ] Session state survives MCP server restart (reattach via pid check)

---

## 3. Browser MCP: Daemon Discovery + Debug API Routing ✅ DONE

### Bug 1: Debug API routes not compiled into dev server ✅ FIXED

`debug-api` is included in default features in `packages/packager/Cargo.toml:116`.

### Bug 2: Ready-file path mismatch ✅ FIXED

Robust daemon discovery implemented in `packages/packager/src/mcp/mod.rs:66-144`:
- `TAIRITSU_DAEMON_URL` env var override
- Multi-strategy search: CWD, parent dirs with `Cargo.toml`, `TAIRITSU_PROJECT_ROOT`, exe parent
- Clear error messages with searched paths and hints

---

## 4. Website: Pending Enhancements

### P1 — Enhancement

| # | Item | Status |
|---|------|--------|
| 15 | **Dynamic markdown rendering** — pulldown-cmark → VNode | ✅ Done (`examples/website/src/markdown.rs`) |
| 17 | **Sidebar item icons** — SVG icons per menu item | ✅ Done (all sidebar items now have MdiIcon) |

### P2 — Polish

| # | Item | Details | Status |
|---|------|---------|--------|
| 19 | **state_test.rs stub handlers** | oninput TODO, dead remove buttons (not in production tree) | 🔄 |
| 20 | **Logo is unicode char** | `\u{273F}` instead of actual image | 🔄 |
| 21 | **No favicon.ico verified** | Referenced in Cargo.toml | 🔄 |
| 22 | **Keyboard navigation** | Escape to close drawer, arrow keys for menu | ✅ Escape closes drawer |

### P3 — Infrastructure

| # | Gap |
|---|-----|
| 23 | Dynamic doc loading missing (all content compiled into WASM) |
| 24 | i18n.rs not wired to all content pages (only not_found.rs uses it) |
| 25 | No keyboard navigation (arrow keys for menu) |
| 26 | No search functionality |
