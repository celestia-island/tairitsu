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

## Latest Update (2026-03-24 23:45)

**ROOT CAUSE IDENTIFIED**:

The issue is a fundamental limitation of wasmtime's `instance().func_wrap()` API. When returning `Result<(f64, f64, f64, f64), wasmtime::Error>`, wasmtime interprets this as a WIT `result` type (1-tuple) instead of a 4-tuple.

**Technical Details**:
1. The `Result<T, E>` type in Rust implements the `Lower` trait as a WIT `result` type
2. When `T = (f64, f64, f64, f64)`, the `Lower` implementation for `Result<T, E>` takes precedence over the `Lower` implementation for tuples
3. This causes wasmtime to see `Result<(f64, f64, f64, f64), wasmtime::Error>` as a 1-tuple (the Result itself) instead of a 4-tuple

**Investigation Results**:
1. ✅ WIT definition is correct - `get-content-rect` returns `dom-rect` record type
2. ✅ Manual implementation is correct - uses `Result<(f64, f64, f64, f64), wasmtime::Error>` with `Ok((0.0, 0.0, 0.0, 0.0))`
3. ✅ Auto-generated stubs have the same issue - confirms the problem is with wasmtime's API
4. ✅ Other similar functions (like `get-bounding-client-rect`) use the same pattern and work correctly (or at least don't cause errors if not called)

**Attempted Workarounds**:
1. Using nested tuples: `Result<((f64, f64, f64, f64),), ...>` → Error: "expected tuple found record"
2. Using different error types (`anyhow::Error` instead of `wasmtime::Error`) → Same error
3. Using `func_wrap_async` → Compilation error
4. Stubbing out the interface → Auto-generated stubs have the same issue

**Recommended Next Steps**:
1. **File a bug with wasmtime** about the `instance().func_wrap()` API not correctly handling multi-value returns
2. **Use wasmtime-bindgen** or similar tool to generate the correct bindings instead of using `func_wrap` manually
3. **Consider using a different approach** for implementing the `resize-observer-entry` interface (e.g., using raw wasmtime APIs)
4. **Temporary workaround**: Accept that this interface cannot be implemented with the current wasmtime API and stub it out

**Current Status**:
This is a **wasmtime API limitation**, not a bug in our code. The issue cannot be fixed without changes to wasmtime or using a different approach for implementing the interface.

## Workaround

Currently blocked on this issue. The SSR test cannot proceed until this type mismatch is resolved.

## Related Files

- `/mnt/sdb1/tairitsu/packages/browser-worlds/wit/browser-full.wit` - WIT definitions
- `/mnt/sdb1/tairitsu/packages/ssr/src/linker.rs` - Host implementation
- `/mnt/sdb1/tairitsu/packages/web/src/wit_platform.rs` - WIT bindings usage
- `/mnt/sdb1/hikari/public/website.wasm` - WASM component (8.5MB, timestamp: 2026-03-24 19:35)
