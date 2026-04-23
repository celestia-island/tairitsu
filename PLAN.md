# Daemon Windows Spawn Fix — Active Work Plan

## Status: IMPLEMENTED LOCALLY — direct `CreateProcessW` path now compiles and passes local terminal smoke test

## Latest Validation (2026-04-23)

### Real MCP bash result for the old approach: HANGS

- Command: `tairitsu.exe --manifest-path examples/website dev --daemon` (from opencode MCP bash tool)
- Parent process output: `Starting daemon...` then `EXIT: elapsed=415ms` — parent exited in 415ms
- **BUT: MCP bash tool hung after parent exit** — the tool's stdout/stderr pipe never closed
- Root cause: child daemon process inherited the MCP tool's pipe handles despite `SetHandleInformation` clearing `HANDLE_FLAG_INHERIT` on stdout/stderr
- Conclusion: Rust's `Command::spawn()` internally re-duplicates handles (or the MCP bash tool's pipe wiring bypasses `GetStdHandle`), so `SetHandleInformation` on standard handles is insufficient

### Local validation for the new `CreateProcessW` path: PASSED

- `cargo build --bin tairitsu -p tairitsu-packager` now passes after stopping a stale `examples/website` daemon that was locking `target/debug/tairitsu.exe`
- `target/debug/tairitsu.exe --manifest-path examples/website dev --daemon` returned control to the terminal runner after printing `Starting daemon...`
- `target/debug/tairitsu.exe --manifest-path examples/website --status` reported a running daemon (`PID: 39628`)
- `target/debug/tairitsu.exe --manifest-path examples/website dev --shutdown` stopped the daemon successfully
- `target/debug/tairitsu.exe --manifest-path examples/website --status` then reported `No daemon is currently running.`
- This is stronger than the earlier PowerShell redirected-pipe probe because it exercised the real binary end-to-end, but it is still **not** the final opencode MCP bash revalidation

## Current Blocker: real opencode MCP bash revalidation still pending

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
| -------- | ------ |
| `CREATE_NEW_PROCESS_GROUP` + `Stdio::null()` + `mem::forget` | Child alive but MCP tool hangs (pipe inherited) |
| `DETACHED_PROCESS` + `Stdio::null()` + `exit(0)` | Pops up console window (unacceptable) |
| `CREATE_NO_WINDOW` + `Stdio::null()` + `exit(0)` | MCP tool still hangs (pipe still inherited) |
| `SetHandleInformation` to clear `HANDLE_FLAG_INHERIT` on parent's stdout/stderr before spawn, then restore after | **Local validation passed** — redirected stdout/stderr drains cleanly and child survives |
| Direct `CreateProcessW` + `STARTUPINFOEXW` + `PROC_THREAD_ATTRIBUTE_HANDLE_LIST` restricted to explicit `NUL` stdio handles | **Current implementation** — compiles and local terminal smoke test passes |
| `Stdio::from(file)` + `DETACHED_PROCESS` + `exit(0)` | Popup console window |
| PowerShell `Start-Process -WindowStyle Hidden` directly from MCP bash | Works perfectly (no popup, child survives) |
| `$env:TAIRITSU_DAEMON="1"; & tairitsu.exe dev ...` directly in MCP bash | Works (child alive and building) |

### Current Code (active Windows implementation)

```rust
// Strategy: bypass Rust's Command::spawn() on Windows and call
// CreateProcessW directly, inheriting only the handles we explicitly
// provide (NUL for stdin/stdout/stderr).

let stdin_handle = OwnedHandle::nul(GENERIC_READ)?;
let stdout_handle = OwnedHandle::nul(GENERIC_WRITE)?;
let stderr_handle = OwnedHandle::nul(GENERIC_WRITE)?;

let mut inherited_handles = [stdin_handle.raw(), stdout_handle.raw(), stderr_handle.raw()];
let mut attribute_list = OwnedAttributeList::new(1)?;
attribute_list.set_handle_list(&mut inherited_handles)?;

let mut startup_info = STARTUPINFOEXW {
    StartupInfo: unsafe { mem::zeroed() },
    lpAttributeList: attribute_list.ptr,
};
startup_info.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
startup_info.StartupInfo.hStdInput = stdin_handle.raw();
startup_info.StartupInfo.hStdOutput = stdout_handle.raw();
startup_info.StartupInfo.hStdError = stderr_handle.raw();

CreateProcessW(
    application_name.as_ptr(),
    command_line.as_mut_ptr(),
    ptr::null(),
    ptr::null(),
    1,
    CREATE_NO_WINDOW | EXTENDED_STARTUPINFO_PRESENT,
    ptr::null(),
    ptr::null(),
    &mut startup_info as *mut STARTUPINFOEXW as *mut _,
    &mut process_info,
);

std::process::exit(0);
```

**This code now has local validation.** The new path compiled, launched the daemon, reported status, and shut it down cleanly in the local terminal runner.

### Why This Should Work

The handle list passed through `PROC_THREAD_ATTRIBUTE_HANDLE_LIST` contains only the inheritable `NUL` stdio handles. Even though `CreateProcessW` is still called with handle inheritance enabled, Windows should only duplicate the handles present in that explicit list, not the caller's stdout/stderr pipe handles.

### Remaining Risk to Investigate

1. The real opencode MCP bash wrapper still needs to be rerun against this exact implementation.
2. If it still hangs, the next thing to inspect is whether the wrapper shell process itself remains alive independently of `tairitsu`, rather than another stray inherited handle inside the child.

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

**Re-run the real opencode MCP/bash reproduction against the new `CreateProcessW` implementation.** The old `SetHandleInformation` path is no longer the active fix.

To test:

```powershell
# From MCP bash:
Remove-Item -Force examples/website/target/tairitsu-packager.pid,examples/website/target/tairitsu-packager.ready -ErrorAction SilentlyContinue
cargo build --bin tairitsu
# Run with short timeout — if it returns within 5s without the bash tool hanging, the fix works
D:\源代码\工程项目\celestia\tairitsu\target\debug\tairitsu.exe --manifest-path examples/website dev --daemon

# Then verify child survived:
Get-Process -Name tairitsu | Select-Object Id,ProcessName
Get-Content examples/website/target/tairitsu-packager.pid
```

## Key Files Modified

| File | Change |
| ---- | ------ |
| `packages/packager/Cargo.toml` | Added `Win32_Security`, `Win32_Storage_FileSystem`, and `Win32_System_Threading` features for direct `CreateProcessW` spawning |
| `packages/packager/src/main.rs` | Calls `handle_sync_daemon()` outside tokio; `exit(0)` on sync path |
| `packages/packager/src/cli/mod.rs` | Added hidden `--daemon-child-process` arg so the Windows child can identify itself without a custom environment block |
| `packages/packager/src/daemon/mod.rs` | `is_daemon()` accepts env-or-hidden-arg; Windows spawn now uses direct `CreateProcessW` + explicit inherited handle list + `CREATE_NO_WINDOW` |
| `packages/packager/src/config/mod.rs` | `AssetsConfig`: added `extra_public_dirs` field |
| `packages/packager/src/wasm/mod.rs` | Copies extra_public_dirs assets; calls `signal_ready()` after initial build |
| `examples/website/Cargo.toml` | Added `extra-public-dirs = ["../../../hikari/public"]` |

## Reference

- **daemon_forge**: <https://github.com/ninunez14/daemon_forge> — cross-platform Rust daemon library using `DETACHED_PROCESS` + `exit(0)` on Windows
- **Windows `CreateProcess` docs**: <https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw>
- **`SetHandleInformation` docs**: <https://learn.microsoft.com/en-us/windows/win32/api/handleapi/nf-handleapi-sethandleinformation>
- **`HANDLE_FLAG_INHERIT`**: When cleared on a handle, `CreateProcess(bInheritHandles=TRUE)` will NOT duplicate it to the child
- **Opencode bash tool**: Uses `detached: process.platform !== "win32"` — does NOT detach on Windows; spawns per-command shell and waits for stdout/stderr pipes to close
