# Daemon Windows Spawn Fix — Active Work Plan

## Status: IN PROGRESS (blocked on pipe handle inheritance)

## What's Done

### CSS 404 Fix (COMPLETED)
- Root cause: packager only copied project's `public/`, not hikari's
- Added `extra_public_dirs` field to `AssetsConfig` in `config/mod.rs`
- Website `Cargo.toml` now references `../../../hikari/public`
- Verified: `target/tairitsu-dist/styles/` contains `bundle.css`, `spa.css`, etc.

### Daemon Path Mismatch Fix (COMPLETED)
- Parent in `handle_sync_daemon()` never set `project_root` → looked in wrong `target/` dir
- Child set `project_root` from `--manifest-path` → wrote ready file to different path
- Fix: `set_project_root()` called in `handle_sync_daemon()` before daemon ops

### Daemon Refactoring (COMPLETED)
- `main.rs`: daemon sync ops happen OUTSIDE tokio runtime via `handle_sync_daemon()`
- `cli/mod.rs`: `handle_sync_daemon()` for sync ops (status/shutdown/parent-fork)
- `command` field is `Option<Commands>` so `--status`/`--shutdown`/`--daemon` work without subcommand
- Windows parent path: fork + `exit(0)` (no `wait_for_child_signal`)
- Unix parent path: fork + `wait_for_child_signal(120)` (unchanged)

## Current Blocker: MCP Tool Pipe Hang on Windows

### The Problem

When `tairitsu --daemon` is called from an MCP bash tool (e.g. opencode's bash tool):

1. The MCP tool spawns the parent process with **pipe handles** for stdout/stderr
2. The parent forks a child process via `Command::spawn()`
3. Rust's `Command::spawn()` calls Windows `CreateProcess` with `bInheritHandles = TRUE`
4. **ALL inheritable handles** are copied to the child — including the MCP tool's pipe handles
5. Parent calls `std::process::exit(0)` — parent terminates
6. But child still holds references to the MCP tool's stdout/stderr pipes
7. MCP tool waits for its pipe to close → pipe doesn't close because child has copies → **HANGS FOREVER**

### What We've Tried

| Approach | Result |
|----------|--------|
| `CREATE_NEW_PROCESS_GROUP` + `Stdio::null()` + `mem::forget` | Child alive but MCP tool hangs (pipe inherited) |
| `DETACHED_PROCESS` + `Stdio::null()` + `exit(0)` | Pops up console window (unacceptable) |
| `CREATE_NO_WINDOW` + `Stdio::null()` + `exit(0)` | MCP tool still hangs (pipe still inherited) |
| `SetHandleInformation` to clear `HANDLE_FLAG_INHERIT` on parent's stdout/stderr before spawn, then restore after | **COMPILES but NOT YET TESTED** — this is the current code |
| `Stdio::from(file)` + `DETACHED_PROCESS` + `exit(0)` | Popup console window |
| PowerShell `Start-Process -WindowStyle Hidden` directly from MCP bash | Works perfectly (no popup, child survives) |
| `$env:TAIRITSU_DAEMON="1"; & tairitsu.exe dev ...` directly in MCP bash | Works (child alive and building) |

### Current Code (in `daemon/mod.rs` fork_daemon_with_args, Windows branch)

```rust
// Strategy: mark parent's stdout/stderr pipe handles as non-inheritable
// before spawn, so child doesn't inherit MCP tool's pipe handles.
// Then restore inheritability after spawn (parent still needs them
// for the brief moment before exit(0)).

const CREATE_NO_WINDOW: u32 = 0x08000000;

unsafe fn make_non_inheritable(std_handle: u32) -> Option<*mut c_void> {
    let handle = GetStdHandle(std_handle);
    if handle.is_null() || handle == INVALID_HANDLE_VALUE {
        return None;
    }
    if SetHandleInformation(handle as _, HANDLE_FLAG_INHERIT, 0) == 0 {
        return None;
    }
    Some(handle)
}

let saved_out = unsafe { make_non_inheritable(STD_OUTPUT_HANDLE) };
let saved_err = unsafe { make_non_inheritable(STD_ERROR_HANDLE) };

let result = Command::new(&exe)
    .env("TAIRITSU_DAEMON", "1")
    .args(&args)
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .creation_flags(CREATE_NO_WINDOW)
    .spawn();

// Restore inheritability (though we exit immediately after)
if let Some(h) = saved_out { unsafe { restore_inheritable(h) }; }
if let Some(h) = saved_err { unsafe { restore_inheritable(h) }; }

let _child = result?;
std::process::exit(0);
```

**This code compiles and builds successfully but has NOT been tested yet.** The last test was interrupted before we could verify.

### Why This Should Work

`SetHandleInformation(handle, HANDLE_FLAG_INHERIT, 0)` clears the inherit flag on the pipe handle. When `CreateProcess` is called with `bInheritHandles = TRUE`, it only duplicates handles that have the `HANDLE_FLAG_INHERIT` flag set. So the MCP tool's pipe handles should NOT be copied to the child.

### Potential Issues to Investigate

1. **Rust's `Command::spawn()` may internally duplicate handles before calling `CreateProcess`**: The Rust std library might create its own inheritable duplicates of handles during the spawn process. If so, our `SetHandleInformation` call might not be sufficient. Need to check Rust's `sys_common/process.rs` Windows implementation.

2. **`CREATE_NO_WINDOW` + non-inheritable stdout/stderr might cause the child to crash**: The child has no console and no inherited handles. When it tries to write to stdout/stderr (during initialization before `daemonize_self()` redirects them), it might panic. But since we use `Stdio::null()`, the child's stdio handles should be NUL — this should be fine.

3. **The MCP tool's pipe might not be the standard handle**: The MCP tool might not use Windows standard handles for communication. It might use a different mechanism (e.g., anonymous pipes created with `CreatePipe`). If the pipe is not the standard handle, `GetStdHandle` won't return it, and our fix won't apply.

### Alternative Approaches to Try Next

#### Option A: Use `CreateProcess` directly via `windows-sys`
Instead of Rust's `Command::spawn()`, call `CreateProcessW` directly with `bInheritHandles = FALSE`. This completely prevents any handle inheritance. Pass only the handles we explicitly want (NUL for stdio) via `STARTUPINFOEX` with `PROC_THREAD_ATTRIBUTE_HANDLE_LIST`.

```rust
// Pseudocode:
let startup_info_ex = STARTUPINFOEXW { 
    lpAttributeList: proc_thread_attribute_list,
    // Only inherit NUL handles, NOT the MCP tool's pipes
};
CreateProcessW(
    exe_path,
    cmd_line,
    lpProcessAttributes: null,
    lpThreadAttributes: null,
    bInheritHandles: FALSE,  // <-- KEY: no inheritance at all
    dwCreationFlags: CREATE_NO_WINDOW | EXTENDED_STARTUPINFO_PRESENT,
    lpEnvironment: null,
    lpCurrentDirectory: null,
    &startup_info_ex.StartupInfo,
    lpProcessInformation: &mut proc_info,
);
```

#### Option B: Spawn via PowerShell/cmd.exe as intermediary
Since `Start-Process -WindowStyle Hidden` works from MCP bash, we could have the parent spawn `cmd.exe /c start /B tairitsu.exe ...` as the intermediary. The `start /B` command creates a detached process that doesn't inherit handles from the caller.

```rust
Command::new("cmd.exe")
    .args(&["/C", "start", "/B", "/MIN", &exe.to_string_lossy(), &args.join(" ")])
    .creation_flags(CREATE_NO_WINDOW)
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null())
    .spawn()?;
std::process::exit(0);
```

Caveat: passing complex arguments with spaces/quotes through cmd.exe is fragile.

#### Option C: Self-exec via `CreateProcess` with no handle inheritance
Write a thin wrapper that calls `CreateProcessW` directly with `bInheritHandles = FALSE` and `STARTF_USESTDHANDLES` pointing to NUL. This avoids Rust's `Command` entirely.

#### Option D: Use Job Objects
Create a Windows Job Object, mark it as "breakaway OK", spawn the child into the job. The child detaches from the parent's handle table. Complex but correct.

### Recommended Next Step

**Test the current `SetHandleInformation` approach first.** If it doesn't work (likely because Rust's `Command::spawn` internally creates new inheritable duplicates), then proceed to **Option A** (direct `CreateProcessW` with `bInheritHandles = FALSE`).

To test:
```powershell
# From MCP bash:
Remove-Item -Force examples/website/target/tairitsu-packager.pid,examples/website/target/tairitsu-packager.ready -ErrorAction SilentlyContinue
cargo build --bin tairitsu
# Run with short timeout — if it returns within 5s, the fix works
D:\源代码\工程项目\celestia\tairitsu\target\debug\tairitsu.exe --manifest-path examples/website dev --daemon

# Then verify child survived:
Get-Process -Name tairitsu | Select-Object Id,ProcessName
Get-Content examples/website/target/tairitsu-packager.pid
```

## Key Files Modified

| File | Change |
|------|--------|
| `packages/packager/Cargo.toml` | Added `Win32_Foundation` feature to windows-sys |
| `packages/packager/src/main.rs` | Calls `handle_sync_daemon()` outside tokio; `exit(0)` on sync path |
| `packages/packager/src/cli/mod.rs` | `handle_sync_daemon()`: sets project_root, handles status/shutdown/fork |
| `packages/packager/src/daemon/mod.rs` | `fork_daemon_with_args()`: Windows uses SetHandleInformation + CREATE_NO_WINDOW + exit(0) |
| `packages/packager/src/config/mod.rs` | `AssetsConfig`: added `extra_public_dirs` field |
| `packages/packager/src/wasm/mod.rs` | Copies extra_public_dirs assets; calls `signal_ready()` after initial build |
| `examples/website/Cargo.toml` | Added `extra-public-dirs = ["../../../hikari/public"]` |

## Reference

- **daemon_forge**: https://github.com/ninunez14/daemon_forge — cross-platform Rust daemon library using `DETACHED_PROCESS` + `exit(0)` on Windows
- **Windows `CreateProcess` docs**: https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw
- **`SetHandleInformation` docs**: https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-sethandleinformation
- **`HANDLE_FLAG_INHERIT`**: When cleared on a handle, `CreateProcess(bInheritHandles=TRUE)` will NOT duplicate it to the child
- **Opencode bash tool**: Uses `detached: process.platform !== "win32"` — does NOT detach on Windows; spawns per-command shell and waits for stdout/stderr pipes to close
