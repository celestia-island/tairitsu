# Context: Browser Glue Code Generation

## Status: Complete ✅

- **TypeScript errors:** 0
- **WASM build:** Success
- **E2E tests:** 7/7 pass

## Key Fixes Applied

### 1. WIT Generator Union Type Handling

Union types like `(TrustedType or DOMString)` now prioritize types in order:
1. Boolean types → `bool`
2. String types → `string`
3. Numeric types → `f64`
4. Interface types → `u64` (fallback)

### 2. innerHTML/outerHTML

Removed from enum properties - these are plain strings, not enums.

### 3. HTMLElement.hidden

Fixed from `string` to `bool` for union type `(boolean or unrestricted double or DOMString)?`.

### 4. Element.setAttribute value

Fixed from `u64` to `string` for union type `(TrustedType or DOMString)`.

### 5. get_element_by_id

Uses `non_element_parent_node` interface, not `document` interface.

## Key Config Patterns

1. **ENUM_PROPERTIES**: Use property name (camelCase), not function name
2. **NUMBER_TO_BIGINT_PROPERTIES**: Use property name, not function name
3. **BOOLEAN_TO_BIGINT_PROPERTIES**: For getter-but-method functions, use the WIT function name
4. **PARAMETER_HANDLE_MAPPING**: Format: `(interface, function, param): (target_interface, TypeScript_type)`
5. **PARAMETER_BIGINT_TO_NUMBER**: Use `"any"` for complex types that need casting

## Commands

```bash
# Regenerate WIT
python3 scripts/generate_browser_wit.py

# Regenerate TypeScript glue
python3 scripts/generate_browser_glue.py

# Verify TypeScript
cd packages/browser-glue && npx tsc --noEmit

# Build WASM
cd examples/website && cargo build --target wasm32-wasip2 --lib --release

# Run E2E tests
cd packages/e2e && cargo test
```
