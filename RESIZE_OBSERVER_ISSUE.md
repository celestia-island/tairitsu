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

## Latest Update (2026-03-24 20:01)

Successfully rebuilt the hikari website WASM component with a clean build:
- Previous WASM file: 8.5MB (old build with stale WIT bindings)
- New WASM file: 425KB (fresh build with updated dependencies)
- Error persists: "expected `u64` found `record`"

This confirms that the issue is **NOT** a build caching problem, but a fundamental type mismatch between:
1. The WIT file definition (expects `dom-rect` record)
2. The WASM component's WIT bindings (expect `u64`)

## Root Cause

The issue appears to be that the `wit-bindgen` macro is generating bindings based on an **old version of the WIT file** or there's a mismatch in how the record types are being interpreted.

The fact that the error says "expected `u64` found `record`" suggests that the WASM component's bindings were generated when `get-content-rect` returned `u64`, but the WIT file has been updated to return `dom-rect` record.

## Current Status

**BLOCKED**: The SSR test cannot proceed until this fundamental type mismatch is resolved. The issue requires:
1. Deep investigation of how `wit-bindgen` generates bindings from WIT files
2. Verification that the WIT file changes are being picked up by `wit-bindgen`
3. Potential refactoring of how record types are handled in the host implementation

## Workaround Attempted

Tried changing the host implementation to return `u64` to match the WASM component's expectations, but this resulted in error "expected `u64` found `record`", confirming that wasmtime is enforcing the WIT file's type definition over the host implementation.

## Workaround

Currently blocked on this issue. The SSR test cannot proceed until this type mismatch is resolved.

## Related Files

- `/mnt/sdb1/tairitsu/packages/browser-worlds/wit/browser-full.wit` - WIT definitions
- `/mnt/sdb1/tairitsu/packages/ssr/src/linker.rs` - Host implementation
- `/mnt/sdb1/tairitsu/packages/web/src/wit_platform.rs` - WIT bindings usage
- `/mnt/sdb1/hikari/public/website.wasm` - WASM component (8.5MB, timestamp: 2026-03-24 19:35)
