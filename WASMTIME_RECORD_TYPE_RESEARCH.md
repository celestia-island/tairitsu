# Wasmtime Record Type Handling in func_wrap - Deep Dive Research

## Executive Summary

Based on extensive research into wasmtime's type system and Component Model implementation, I've identified the root cause of the "expected 4-tuple, found 1-tuple" error and the solution for properly handling WIT record types in `func_wrap`.

## Problem Analysis

### Current Error
```
Error: component imports instance `tairitsu-browser:full/resize-observer-entry@0.2.0`, but a matching implementation was not found in the linker
Caused by:
    0: instance export `get-content-rect` has the wrong type
    1: type mismatch with results
    2: expected `u64` found `record`
```

### Key Discrepancies Found

1. **WIT File Definition** (browser-full.wit):
```wit
interface resize-observer-entry {
    use types.{dom-rect};
    get-content-rect: func(self: resize-observer-entry-handle) -> dom-rect;
}

interface types {
    record dom-rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    }
}
```

2. **Auto-generated WIT** (generated/observers.wit):
```wit
interface resize-observer-entry {
    get-content-rect: func(self: resize-observer-entry-handle) -> u64;
}
```

3. **Current Host Implementation** (linker.rs):
```rust
resize_observer_entry.func_wrap(
    "get-content-rect",
    |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
     (_self,): (u64,)|
     -> Result<(f64, f64, f64, f64), wasmtime::Error> {
        Ok((0.0, 0.0, 0.0, 0.0))
    },
)?;
```

## Wasmtime Component Model Type System

### Canonical ABI Rules

The WebAssembly Component Model uses a **Canonical ABI** that defines how complex types are flattened:

1. **Record Types**: Flattened to their constituent fields
2. **Tuple Types**: Treated as anonymous records, also flattened
3. **Variant Types**: Flattened with discriminant
4. **List Types**: Passed as pointers/length pairs

### Key Principle: **No Nested Tuples**

In the Component Model's canonical ABI, there are **no nested tuples**. All aggregate types are flattened:

```rust
// WIT record: record dom-rect { x: f64, y: f64, width: f64, height: f64 }
// Canonical ABI: (f64, f64, f64, f64) - FLAT tuple

// ❌ WRONG: Nested tuple
-> Result<((f64, f64, f64, f64),), wasmtime::Error>

// ✅ CORRECT: Flat tuple
-> Result<(f64, f64, f64, f64), wasmtime::Error>
```

## Root Cause

The error "expected 4-tuple, found 1-tuple" indicates that:

1. **WIT Definition**: The `dom-rect` record type is properly defined in `browser-full.wit`
2. **WASM Component**: Compiled against **old** WIT bindings expecting `u64` (from generated/observers.wit)
3. **Host Implementation**: Correctly returns `(f64, f64, f64, f64)` for the record type
4. **Type Mismatch**: Wasmtime enforces type safety between WIT definition and WASM component expectations

## Solution

### 1. Understanding Record Type Flattening

WIT record types are **automatically flattened** by wasmtime's func_wrap:

```rust
// WIT: record dom-rect { x: f64, y: f64, width: f64, height: f64 }
// Rust: Return as flat tuple, not nested

func_wrap(
    "get-content-rect",
    |_caller, (_self,): (u64,)| -> Result<(f64, f64, f64, f64), wasmtime::Error> {
        Ok((x, y, width, height))  // FLAT tuple - CORRECT
    }
)

// NOT: Result<((f64, f64, f64, f64),), wasmtime::Error>  // NESTED tuple - WRONG
```

### 2. Type Mapping Rules

#### Single Value Returns
```rust
// WIT: func() -> u64
-> Result<(u64,), wasmtime::Error> { Ok((value,)) }
```

#### Multiple Value Returns
```rust
// WIT: func() -> (u64, f64)
-> Result<(u64, f64), wasmtime::Error> { Ok((val1, val2)) }
```

#### Record Type Returns
```rust
// WIT: record point { x: f64, y: f64 }
// WIT: func() -> point
-> Result<(f64, f64), wasmtime::Error> { Ok((x, y)) }
```

#### Empty Returns
```rust
// WIT: func()
-> Result<(), wasmtime::Error> { Ok(()) }
```

### 3. Build.rs Type Mapping (Already Correct)

The current implementation in `build.rs` correctly maps `dom-rect` to `(f64, f64, f64, f64)`:

```rust
// Handle common record types
"dom-rect" => "(f64, f64, f64, f64)".to_string(),
```

## The Real Issue: WIT File Inconsistency

### Problem
There are **two different WIT definitions** for the same interface:

1. **browser-full.wit** (lines 11750-11760): Returns `dom-rect` record
2. **generated/observers.wit** (line 107): Returns `u64`

### Solution Options

#### Option 1: Fix the Auto-generation (Recommended)
Update the WIT generation script to use `dom-rect` instead of `u64` for `get-content-rect`.

#### Option 2: Use Consistent WIT Files
Ensure only one authoritative WIT definition exists and both WASM compilation and host implementation use it.

#### Option 3: Transitional Compatibility
Provide both implementations and let the linker choose based on WASM component expectations.

## Testing and Validation

### Correct Implementation Pattern

```rust
// For record types that return multiple values
resize_observer_entry.func_wrap(
    "get-content-rect",
    |_caller: wasmtime::StoreContextMut<'_, SsrHostState>,
     (_self,): (u64,)|  // Single parameter: tuple with one element
     -> Result<(f64, f64, f64, f64), wasmtime::Error> {  // Flat tuple for record
        Ok((0.0, 0.0, 0.0, 0.0))  // Return flat tuple, not nested
    },
)?;
```

### Common Mistakes to Avoid

1. **Nested tuples**: `Result<((f64, f64, f64, f64),), _>` ❌
2. **Single value instead of tuple**: `Result<f64, _>` ❌
3. **Missing tuple wrapper for single values**: `Result<u64, _>` ❌ (should be `Result<(u64,), _>`)

## Wasmtime Version Specifics

**Current Version**: wasmtime 40.0.4 (latest: 43.0.0)

The Component Model and record type flattening behavior is **stable** across these versions, so upgrading won't change the fundamental approach.

## Recommendations

1. **Immediate Fix**: Resolve WIT file inconsistency between `browser-full.wit` and `generated/observers.wit`
2. **Long-term**: Establish single source of truth for WIT definitions
3. **Validation**: Add type checking tests to catch mismatches during build
4. **Documentation**: Document the record type flattening rules for future developers

## Conclusion

The issue is **not** about how to return record types from `func_wrap` (the current implementation is correct), but rather about **ensuring consistent WIT definitions** between the host implementation and WASM component compilation.

The correct way to return record types from `func_wrap` is:
- **Flattened tuples** matching the record's field order
- **No nested tuples**
- **Type consistency** between WIT definitions and implementation