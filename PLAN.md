# Fix Plan: `just dev` crashes with signal 15

## Problem

Running `just dev` (which calls `tairitsu dev --watch`) sometimes terminates
with `signal 15` (SIGTERM). Two distinct root causes have been identified.

---

## Root Cause 1: No SIGTERM handler — silent death

**Files:**
- `packages/packager/src/wasm/mod.rs:2444-2511` (watch loop)
- `packages/packager/src/wasm/mod.rs:2234-2251` (non-watch serve)

The watch loop (`run_watch_loop`) only handles `ctrl_c()` (SIGINT) in its
`tokio::select!`. When the process receives SIGTERM (e.g. from `just` forwarding
a signal, or `kill_daemon()`, or OS process manager), it dies immediately
without any cleanup or user-visible message. `just` then reports
"terminated by signal 15".

The non-watch path (`axum::serve(listener, app).await`) also has no
graceful-shutdown mechanism — no `with_graceful_shutdown()`.

**Fix:**

### 1a. Add SIGTERM handler to watch loop (`wasm/mod.rs`)

In the `tokio::select!` block inside `run_watch_loop` (around line 2504), add a
SIGTERM handler alongside the existing `ctrl_c()` handler:

```rust
// Add at the top of the file or in run_watch_loop:
#[cfg(unix)]
let mut sigterm = {
    use tokio::signal::unix::{SignalKind, signal};
    signal(SignalKind::terminate()).unwrap()
};

// In the tokio::select! block, add:
#[cfg(unix)]
_ = sigterm.recv() => {
    crate::log_ok!("{}", locale().dev.stopping);
    crate::logfmt::clear_active_pb();
    use std::io::Write;
    let _ = std::io::stdout().write_all(b"\x1b[?25h");
    let _ = std::io::stdout().flush();
    break 'watch;
}
```

Alternatively, merge both SIGINT and SIGTERM into a single handler arm to avoid
code duplication.

### 1b. Add `with_graceful_shutdown` to non-watch serve (`wasm/mod.rs:2250`)

```rust
// Replace:
//   axum::serve(listener, app).await?;

// With a graceful shutdown triggered by SIGINT or SIGTERM:
let shutdown = async {
    tokio::signal::ctrl_c().await.ok();
};
#[cfg(unix)]
let shutdown = async {
    use tokio::signal::unix::{SignalKind, signal};
    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {},
        _ = sigterm.recv() => {},
    }
};
axum::serve(listener, app)
    .with_graceful_shutdown(shutdown)
    .await?;
```

Note: Make sure `tokio` has the `signal` feature enabled (it already does per
`Cargo.toml` lines 30-38). You may also need to add the `net` feature for
`axum::serve`.

---

## Root Cause 2: TTY detection uses `atty::Stream::Stdout` — wrong stream

**Files:**
- `packages/packager/src/daemon/mod.rs:219-221`
- `packages/packager/src/cli/mod.rs:446-452`
- `packages/packager/src/cli/mod.rs:529-531`

`is_tty()` checks `atty::Stream::Stdout`, but when `just` (or any process
manager) runs `tairitsu`, stdout is piped while stderr may still be a TTY.
When `is_tty()` returns `false`, the code at `cli/mod.rs:529-531` prints a
non-interactive hint via `log_warn!`/`log_info!` (which write to **stderr**),
then exits with `Ok(())`. But if `logfmt` hasn't been initialized yet (it
hasn't — `init_tracing` is called at line 522 but `init()` is at line 426), the
output may be swallowed, and the process appears to silently exit.

Additionally, `atty` crate is deprecated (unmaintained since 2020). Modern
replacement is `std::io::IsTerminal` (stable since Rust 1.70).

**Fix:**

### 2a. Check Stdin instead of Stdout (`daemon/mod.rs:219-221`)

The interactive dev server needs stdin for the keyboard command listener
(`wasm/mod.rs:2414-2435`). Stdin is the correct stream to check:

```rust
pub fn is_tty() -> bool {
    std::io::stdin().is_terminal()
}
```

This avoids false negatives when stdout is piped (e.g. by `just`) but the user
is still at an interactive terminal.

Alternatively, check both stdin and stderr as a fallback:

```rust
pub fn is_tty() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal() || std::io::stderr().is_terminal()
}
```

### 2b. Replace `atty` with `std::io::IsTerminal` everywhere

- `daemon/mod.rs:219-221` — `is_tty()`
- `daemon/mod.rs:808` — `fork_daemon_with_args` color detection
- `daemon/mod.rs:837` — same, Windows branch
- `logfmt.rs:23-29` — `stdout_is_tty()`, `stderr_is_tty()`

Replace:
```rust
atty::is(atty::Stream::Stdout)
```
With:
```rust
std::io::stdout().is_terminal()
```

Then remove `atty` from `Cargo.toml` dependencies.

### 2c. Ensure log output is visible even in non-TTY mode (`cli/mod.rs:529-531`)

Move the non-interactive check to AFTER `logfmt::init_tracing()` (line 522),
or ensure `print_non_tty_hint()` forces output to stderr regardless of
initialization state. Currently the check is at line 529 which IS after
`init_tracing`, so this should be fine — but verify that `log_warn!`/`log_info!`
actually produce output when stdout is piped.

---

## Implementation Order

1. **Replace `atty` with `std::io::IsTerminal`** (Root Cause 2b) — removes
   deprecated dependency, fixes TTY detection accuracy.
2. **Fix `is_tty()` to check stdin** (Root Cause 2a) — fixes `just dev`
   false-negative on TTY detection.
3. **Add SIGTERM handler** (Root Cause 1a) — prevents silent signal-15 death
   in watch mode.
4. **Add `with_graceful_shutdown`** (Root Cause 1b) — prevents silent death
   in non-watch mode.

## Testing

After fixes, verify:
1. `just dev` starts and remains running (TTY detection passes)
2. `kill -TERM <pid>` on a running `tairitsu dev` prints "Stopping..." and
   exits cleanly (SIGTERM handler works)
3. `just dev` without a TTY (e.g. from a CI script) shows the non-interactive
   hint message clearly
4. `just dev --daemon` still works correctly
