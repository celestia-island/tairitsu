# Resize Observer Entry Type Mismatch Issue

## Problem Description

When running the SSR test (`hikari-ssr-test`), the following error occurs:

```
Error: component imports instance `tairitsu-browser:full/resize-observer-entry@0.2.0`, but a matching implementation was not found in the linker
Caused by:
    0: instance export `get-content-rect` has the wrong type
    1: type mismatch with results
    2: expected `u64` found `record`
```

## Root Cause Analysis

1. **WIT Definition (Correct)**: The WIT file at `/mnt/sdb1/tairitsu/packages/browser-worlds/wit/browser-full.wit` correctly defines `get-content-rect` to return `dom-rect` record type:
```wit
interface resize-observer-entry {
    use types.{dom-rect};
    get-content-rect: func(self: resize-observer-entry-handle) -> dom-rect;
}
```

2. **WASM Component Expectations (Old)**: The WASM component (`/mnt/sdb1/hikari/public/website.wasm`) is expecting the old signature where `get-content-rect` returns `u64` instead of `dom-rect`.

3. **Type Mismatch**: This creates a fundamental type mismatch between what the WASM component expects (`u64`) and what the WIT file defines (`dom-rect` record).

## Investigation Steps Taken

1. ✅ Updated WIT file to use `dom-rect` record type
2. ✅ Updated host implementation to return `(f64, f64, f64, f64)` (4-tuple)
3. ✅ Rebuilt tairitsu-ssr multiple times
4. ✅ Rebuilt tairitsu-web with `wit-bindings` feature
5. ✅ Completely rebuilt hikari website (removed target directory)
6. ✅ Verified auto-generated stubs exclude `resize-observer-entry`
7. ✅ Tried different return type formats (flat tuple, nested tuple, u64)

## Current Status

The issue persists despite all rebuild attempts. The error message "expected `u64` found `record`" confirms that:
- The WASM component is using old WIT bindings that expect `u64`
- The WIT file has been updated to define `dom-rect` as a record type
- Wasmtime is enforcing type safety and rejecting the mismatch

## Latest Update (2026-03-24 22:30)

**NEW FINDING**: Error message has changed from "expected `u64` found `record`" to "expected 4-tuple, found 1-tuple". This indicates that:

1. The WASM component NOW correctly expects a 4-tuple (matching the flattened `dom-rect` record)
2. But wasmtime's `func_wrap` is interpreting `Result<(f64, f64, f64, f64)>` as a 1-tuple

**Root Cause Identified**:
- WIT record types are flattened to tuples in the canonical ABI
- So `dom-rect` (record with 4 f64 fields) becomes `(f64, f64, f64, f64)` in the ABI
- However, wasmtime's `func_wrap` API wraps the return value in a Result
- The Result type is being interpreted as a 1-tuple, hiding the 4-tuple inside

**Attempted Workarounds**:
1. Nested tuple: `Result<((f64, f64, f64, f64),), wasmtime::Error>` → Error: "expected tuple found record"
2. Direct tuple: Removed Result wrapper → Compilation error (func_wrap requires Result)
3. Changed WIT to return tuple instead of record → Same "expected 4-tuple, found 1-tuple" error

**Current Understanding**:
The issue is a fundamental limitation in how wasmtime's `func_wrap` API handles multiple return values from WIT record types. The API appears to interpret the Result type as a 1-tuple, preventing the correct marshalling of multi-value returns.

**Next Steps Required**:
1. Investigate wasmtime source code for `func_wrap` implementation
2. Look for alternative APIs (e.g., `func_wrap_raw`, `func_wrap_async`) that might handle tuples correctly
3. Consider using wasmtime's lower-level APIs to manually handle the type marshalling
4. Or temporarily stub out `resize-observer-entry` to unblock other SSR testing

## Workaround

Currently blocked on this issue. The SSR test cannot proceed until this type mismatch is resolved.

## Related Files

- `/mnt/sdb1/tairitsu/packages/browser-worlds/wit/browser-full.wit` - WIT definitions
- `/mnt/sdb1/tairitsu/packages/ssr/src/linker.rs` - Host implementation
- `/mnt/sdb1/tairitsu/packages/web/src/wit_platform.rs` - WIT bindings usage
- `/mnt/sdb1/hikari/public/website.wasm` - WASM component (8.5MB, timestamp: 2026-03-24 19:35)
