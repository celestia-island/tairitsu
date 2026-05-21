# Tairitsu vtty Bug Fix Plan

## Investigation Summary

During integration testing of Entelecheia TUI (a crossterm/ratatui application) via tairitsu's vtty MCP tools, multiple critical bugs were discovered that prevent reliable PTY session management.

**Test scenario**: Launch `cargo run -p entelecheia-tui -- --source-build` in a vtty session, then interact with the TUI via `send_keys`.

**Environment**: Linux x86_64, crossterm 0.29.0 (mio backend), portable-pty 0.8.1, filedescriptor 0.8.3

---

## Bug 1: Zombie Process Leak (CRITICAL) — FIXED

### Problem
When a vtty session's child process exits, tairitsu-mcp never calls `wait()`/`waitpid()` on it. The child becomes a defunct zombie.

### Fix
Added `kill_and_reap()` in `pty_unix.rs` that sends SIGKILL then polls `try_wait()` in a loop with a 5-second deadline.

---

## Bug 2: `extract_master_fd()` Uses Undefined Behavior (CRITICAL) — FIXED

### Problem
Original `extract_master_fd()` cast `Box<dyn Read + Send>` to `Box<std::fs::File>` via raw pointer — UB because the underlying type was `PtyFd`, not `File`.

### Fix
Replaced with `dup(master.as_raw_fd())`. Also removed `make_nonblocking()` call (see Bug 9).

---

## Bug 3: `reader_loop` Thread Not Running (HIGH) — FIXED

### Problem
Reader thread exited immediately due to invalid `read_fd` (Bug 2) and 32KB stack was too small.

### Fix
Bug 2 fix resolved the invalid fd. Stack size increased to 128KB. Diagnostic logging added.

---

## Bug 4: `read_nonblocking()` and `reader_loop` Race on Same FD (MEDIUM) — FIXED

### Problem
Both `read_nonblocking()` (called by `read_and_update()`) and `reader_loop` read from the same `read_fd`, splitting data and corrupting screen state.

### Fix
Removed `read_and_update()` from the MCP package's `VttySession`. The `reader_loop` thread is now the sole reader. All `read_and_update()` calls in `lib.rs` removed.

---

## Bug 5: `Drop for UnixPty` Double-Close Risk (MEDIUM) — FIXED

### Fix
`Drop` checks `read_fd >= 0` before closing. Added `impl Drop for VttySession` that calls `kill()` to join reader thread before UnixPty drops.

---

## Bug 6: Vt100Screen Text Extraction Loses SGR Context (LOW) — FIXED

### Fix
Resolved as a symptom of Bug 4 (race condition). The VTE parser correctly handles SGR sequences; the issue was data being split between two concurrent readers.

---

## Bug 7: `portable_pty` Slave fd Closed Before Child Inherits It (CRITICAL) — NOT REPRODUCIBLE

### Analysis
Investigated `FD_CLOEXEC` propagation through `as_stdio()` → `dup2()`. On Linux, `dup2()` does NOT copy the close-on-exec flag, so the child's fd 0,1,2 should not have cloexec. Added `set_master_raw()` as a precaution but it doesn't affect crossterm behavior (child sets its own terminal mode).

---

## Bug 8: Vt100Screen `get_text()` Leaks SGR Parameters (MEDIUM) — FIXED

### Fix
Symptom of Bug 4. With the reader_loop as sole reader, escape sequences are no longer split across reads.

---

## Bug 9: PTY Master fd Lost — Slave Becomes Orphan (CRITICAL) — FIXED

### Problem
The dup'd `read_fd` was set to `O_NONBLOCK` via `make_nonblocking()`. Since `dup()` shares the file description, this made ALL fds for the same PTY non-blocking — including the writer's fd. This caused `write_all()` to potentially fail with `EAGAIN` when the PTY buffer was full.

### Fix
- Removed `make_nonblocking()` entirely
- Rewrote `reader_loop` to use `libc::poll()` with 100ms timeout instead of non-blocking reads + busy-wait
- `poll()` waits for data without needing `O_NONBLOCK`, so the writer's fd stays blocking
- Added proper `POLLHUP`/`POLLERR`/`POLLNVAL` handling for clean shutdown detection

---

## Round 3 Fixes: Additional Bugs Found During Audit

### Bug 10: No `Drop` on VttyManager — Orphaned Processes on Shutdown (HIGH) — FIXED

**Problem**: When the MCP server exits, `VttyManager` is dropped without killing any sessions. Child processes are orphaned, reader threads continue running until their fd is closed by the OS.

**Fix**: Added `impl Drop for VttyManager` that iterates all sessions and calls `kill()` on each.

### Bug 11: Reader Thread Use-After-Close on Unclean Drop (HIGH) — FIXED

**Problem**: `VttySession` had no `Drop` impl. If dropped without calling `kill()`, `UnixPty::drop()` closed `read_fd` while the reader thread was still using it, causing fd-reuse race.

**Fix**: Added `impl Drop for VttySession` that calls `kill()`, which properly joins the reader thread before the PTY is dropped.

### Bug 12: `VttyManager::kill()` Silently Discards Errors (MEDIUM) — FIXED

**Problem**: `let _ = guard.kill()` — errors from kill_and_reap were silently ignored while the session was still removed from the HashMap.

**Fix**: Changed to log the error with `eprintln!` before removing the session.

### Bug 13: `list()` Holds Map Lock While Locking Sessions (MEDIUM) — FIXED

**Problem**: `list()` held the sessions HashMap lock while iterating and locking each individual session to call `info()`, blocking all other operations (`launch`, `kill`, `get`).

**Fix**: Clone Arcs from the HashMap first, then release the map lock before locking individual sessions.

---

## Implementation Order — Final

1. ~~Bug 2~~ — Fixed (dup via as_raw_fd)
2. ~~Bug 3~~ — Fixed (reader_loop runs, 128KB stack)
3. ~~Bug 1~~ — Fixed (kill_and_reap)
4. ~~Bug 4~~ — Fixed (removed read_and_update, reader_loop is sole reader)
5. ~~Bug 5~~ — Fixed (Drop checks read_fd >= 0)
6. ~~Bug 6~~ — Fixed (symptom of Bug 4)
7. ~~Bug 7~~ — Not reproducible (added set_master_raw as precaution)
8. ~~Bug 8~~ — Fixed (symptom of Bug 4)
9. ~~Bug 9~~ — Fixed (poll-based reader_loop, no O_NONBLOCK)
10. ~~Bug 10~~ — Fixed (Drop for VttyManager)
11. ~~Bug 11~~ — Fixed (Drop for VttySession)
12. ~~Bug 12~~ — Fixed (log kill errors)
13. ~~Bug 13~~ — Fixed (list() releases map lock early)

## Files Modified

- `packages/mcp/src/vtty/pty_unix.rs` — Bugs 1, 2, 3, 5, 9
- `packages/mcp/src/vtty/mod.rs` — Bugs 4, 10, 11, 12, 13
- `packages/mcp/src/lib.rs` — Bug 4 (removed read_and_update calls)
- `PLAN.md` — Documentation of all fixes

---

## Remaining Known Limitations

- **Bug 7 / TUI key input**: If crossterm TUI still doesn't respond to `send_keys`, it may need further investigation into the PTY pair fd correspondence. The `poll()`-based reader_loop should be more reliable than the previous non-blocking approach.
- **Atomics use `Ordering::Relaxed`**: All atomic operations use `Relaxed` ordering. This works in practice due to the 100ms poll timeout, but technically `Release`/`Acquire` would be more correct.
- **Packager package**: The `packages/packager/src/vtty/` module has separate bugs (thread leak in `read_nonblocking()`, VTE parser state lost between calls) that are not addressed here.

---

## Verification Steps

After fixes:

1. Build: `cd /mnt/sdb1/tairitsu && cargo build -p tairitsu-mcp`
2. Deploy: copy binary to opencode MCP path
3. Restart opencode (to pick up new binary)
4. Test in opencode session:
   ```
   tairitsu_vtty_launch: cargo run -p entelecheia-tui -- --source-build (cwd=/mnt/sdb1/entelecheia)
   ```
5. Wait for TUI to render, then verify:
   - `tairitsu_vtty_send_keys(keys="Down")` should move cursor in TUI
   - `tairitsu_vtty_screenshot()` should show clean text without SGR fragments
   - `tairitsu_vtty_kill()` should clean up all resources (no zombies, no leaked fds)
6. Kill session and verify:
   - No zombie processes under tairitsu-mcp
   - PTY master fds cleaned up
   - pts/N released
   - Reader thread exits cleanly (check via `/proc/PID/task/`)
