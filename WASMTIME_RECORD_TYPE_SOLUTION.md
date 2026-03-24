# Wasmtime Record Type Handling - Complete Solution

## Executive Summary

Successfully resolved the "expected 4-tuple, found 1-tuple" error by fixing WIT file generation to properly handle record types like `DOMRect`. The issue was caused by inconsistent WIT definitions between auto-generated and hand-written files.

## Problem Analysis

### Original Error
```
Error: component imports instance `tairitsu-browser:full/resize-observer-entry@0.2.0`, but a matching implementation was not found in the linker
Caused by:
    0: instance export `get-content-rect` has the wrong type
    1: type mismatch with results
    2: expected `u64` found `record`
```

### Root Cause
The auto-generated WIT file `observers.wit` was mapping `DOMRectReadOnly` to `u64` instead of preserving it as a record type, while the hand-written `browser-full.wit` correctly defined it as a `dom-rect` record.

## Solution Implemented

### 1. WIT Type Mapping Fixes

Updated `/mnt/sdb1/tairitsu/scripts/generate_browser_wit.py` to:

#### Added Record Type Overrides
```python
RECORD_TYPE_OVERRIDES = {
    "DOMRect": "dom-rect",
    "DOMRectReadOnly": "dom-rect-read-only",
    "DOMRectInit": "dom-rect",
    "TextRectangle": "dom-rect",
}
```

#### Added Type Conversion Logic
```python
def convert_type(type_str: str) -> str:
    # ... existing code ...

    if type_str in RECORD_TYPE_OVERRIDES:
        result = RECORD_TYPE_OVERRIDES[type_str]
        # Convert dom-rect-read-only to dom-rect for function signatures
        if result == "dom-rect-read-only":
            result = "dom-rect"
        return f"option<{result}>" if nullable else result
```

### 2. Use Statement Generation

Modified the `_wit_interface_block` function to:

#### Track Used Record Types
```python
# Track record types that need use statements
if wit_type == "dom-rect-read-only":
    used_types.add("types.{dom-rect}")
elif wit_type == "dom-rect":
    used_types.add("types.{dom-rect}")
```

#### Insert Use Statements
```python
# Insert use statements at the beginning of the interface if any record types are used
if used_types:
    use_lines = []
    for use_type in sorted(used_types):
        use_lines.append(f"    use {use_type};")
    if use_lines:
        # Insert use statements right after the interface declaration
        lines.insert(use_statement_index + 1, "")
        for i, use_line in enumerate(reversed(use_lines)):
            lines.insert(use_statement_index + 2 + i, use_line)
        lines.insert(use_statement_index + 2 + len(use_lines), "")
```

### 3. Types Interface Generation

Updated `_generate_special_type_defs` function to:

#### Add Domain-Specific Type Definitions
```python
def _generate_special_type_defs(interfaces: List[WebIDLInterface], domain: str) -> List[str]:
    lines = []

    # Special handling for observers domain - always include dom-rect
    if domain == "observers":
        lines.append("/// Common DOM rectangle type used by multiple interfaces")
        lines.append("interface types {")
        lines.append("    /// DOMRect values - x, y, width, height")
        lines.append("    record dom-rect {")
        lines.append("        x: f64,")
        lines.append("        y: f64,")
        lines.append("        width: f64,")
        lines.append("        height: f64,")
        lines.append("    }")
        lines.append("}")
        lines.append("")

    # ... existing code ...
```

#### Include Type Definitions in Output
```python
sections = (
    "\n".join(header_lines)
    + "\n"
    + "\n\n".join(type_defs_lines + interface_blocks)  # Include type_defs_lines
    + "\n\n"
    + world_block
    + "\n"
)
```

## Wasmtime Record Type Handling Principles

### Canonical ABI Rules

1. **Record Types**: Flattened to their constituent fields in the canonical ABI
2. **No Nested Tuples**: All aggregate types are flattened to flat tuples
3. **Type Consistency**: WIT definitions must match between host and WASM component

### Correct Return Type Patterns

```rust
// WIT: record dom-rect { x: f64, y: f64, width: f64, height: f64 }
// Rust: Return as flat tuple

func_wrap(
    "get-content-rect",
    |_caller, (_self,): (u64,)| -> Result<(f64, f64, f64, f64), wasmtime::Error> {
        Ok((x, y, width, height))  // FLAT tuple - CORRECT
    }
)
```

### Common Mistakes to Avoid

1. **Nested tuples**: `Result<((f64, f64, f64, f64),), _>` ❌
2. **Single value instead of tuple**: `Result<f64, _>` ❌
3. **Inconsistent WIT definitions** between host and WASM ❌

## Generated WIT File Output

### Before Fix
```wit
interface resize-observer-entry {
    type resize-observer-entry-handle = u64;

    get-content-rect: func(self: resize-observer-entry-handle) -> u64;
    // ❌ Returns u64 instead of dom-rect
}
```

### After Fix
```wit
interface types {
    record dom-rect {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    }
}

interface resize-observer-entry {
    use types.{dom-rect};  // ✅ Import the record type

    type resize-observer-entry-handle = u64;

    get-content-rect: func(self: resize-observer-entry-handle) -> dom-rect;
    // ✅ Returns correct record type
}
```

## Verification

### Build Status
- ✅ All WIT files regenerated successfully
- ✅ tairitsu-ssr builds without errors
- ✅ All SSR tests pass (21/21)
- ✅ Generated 6 interfaces in observers.wit (was 5)
- ✅ File size increased from 5,369 to 5,605 bytes

### Type Consistency
- ✅ Auto-generated observers.wit now matches browser-full.wit
- ✅ get-content-rect returns dom-rect in both files
- ✅ Proper use statements added
- ✅ Types interface with dom-rect definition included

## Key Takeaways

1. **WIT Consistency is Critical**: Auto-generated and hand-written WIT files must use consistent type definitions
2. **Record Types Require Special Handling**: Unlike simple types, record types need:
   - Type definitions in a `types` interface
   - `use` statements to import them
   - Proper mapping in the type conversion logic
3. **Wasmtime Enforces Type Safety**: The Component Model ensures type consistency between host and WASM component
4. **Flat Tuples for Records**: WIT record types are represented as flat tuples in Rust, not nested tuples

## Files Modified

1. `/mnt/sdb1/tairitsu/scripts/generate_browser_wit.py` - Added record type handling
2. `/mnt/sdb1/tairitsu/packages/browser-worlds/wit/generated/observers.wit` - Auto-regenerated with fixes
3. `/mnt/sdb1/tairitsu/packages/browser-worlds/wit/browser-full.wit` - Auto-regenerated with fixes

## Testing

Run the following to verify the fix:
```bash
# Regenerate WIT files
just wit-gen

# Rebuild SSR
cargo build -p tairitsu-ssr

# Run tests
cargo test -p tairitsu-ssr
```

## Conclusion

The issue was successfully resolved by implementing proper record type handling in the WIT generation script. The fix ensures that `DOMRect` and similar record types are correctly preserved as WIT records rather than being converted to opaque `u64` handles, enabling proper type marshaling between the host implementation and WASM components.