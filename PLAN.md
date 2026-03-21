# WIT Bindings Refactoring Plan

## Overview

Refactor the browser-glue and WIT bindings system to ensure Rust code can properly call TypeScript glue implementations at runtime through jco transpiled imports.

## Current State

### Completed

1. **WIT Generation**
   - `scripts/generate_browser_wit.py` now generates `browser-full.wit` automatically
   - Global singleton interfaces (window, document, etc.) no longer have `self` parameters
   - Added `console` interface for logging
   - Added `style` interface wrapper
   - Duplicate interfaces are filtered (e.g., `types`, `event-target`)

2. **TypeScript Glue**
   - `scripts/generate_browser_glue.py` generates glue files to `src/`
   - 28 domains, 454 interfaces, 3974 functions
   - **186 → 7 errors** (97% reduction)

3. **Code Generator Fixes**
   - Fixed ast_parser.py bug: global singleton first param not treated as self_param
   - Added GLOBAL_SINGLETONS: crypto, screen, location, history, performance
   - Added EVENT_HANDLER_PROPERTIES config for event handler getters
   - Added event handler synthetic type and lookup logic
   - Added 80+ event handler setter configs to PARAMETER_BIGINT_TO_NUMBER

4. **Interface Wrappers**
   - `scripts/generate_interface_wrappers.py` creates interface-level wrappers
   - Generated: `document.js`, `node.js`, `window.js`, `event-target.js`

5. **Packager**
   - Import Map updated to: `"tairitsu-browser:full/": "./browser-glue/"`

### Remaining Issues (7 WIT Definition Errors)

These require WIT generator fixes:

| File | Issue | Cause |
|------|-------|-------|
| htmlGlue.ts:17861-17890 | Storage methods | WIT missing self parameter |
| urlGlue.ts:306,314 | URL.href type | WIT defines as u64 instead of string |

## Action Items

### Phase 1: Fix WIT Definition Issues (7 errors)

1. **Storage Interface**
   - Add `self: storage-handle` parameter to all methods
   - Or make Storage a global singleton using `window.localStorage`

2. **URL.href**
   - Change type from `u64` to `string` in WIT generator

### Phase 2: Complete Rust Code Updates

Update `packages/web/src/wit_platform.rs`:

```rust
// Old:
bindings::tairitsu_browser::full::document::body()
bindings::tairitsu_browser::full::document::create_element(tag)
bindings::tairitsu_browser::full::node::set_attribute(el, name, value)

// New:
bindings::tairitsu_browser::full::document::get_body()  // returns option<u64>
bindings::tairitsu_browser::full::document::create_element(tag, None)
bindings::tairitsu_browser::full::element::set_attribute(el, name, value)
```

### Phase 3: Build and Test

1. `cd packages/browser-glue && npm run build`
2. `python3 scripts/generate_interface_wrappers.py`
3. `cd examples/website && cargo build --target wasm32-wasip2 --lib --release`
4. Run packager dev server
5. Open browser, check console for errors

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Browser Runtime                          │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ Import Map                                           │    │
│  │ "tairitsu-browser:full/" -> "./browser-glue/"       │    │
│  └─────────────────────────────────────────────────────┘    │
│                           ↓                                  │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ browser-glue/*.js (SWC compiled)                    │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                           ↑ WIT imports
┌─────────────────────────────────────────────────────────────┐
│                   WASM Component (jco)                       │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ Rust wit-bindgen bindings                           │    │
│  │ bindings::tairitsu_browser::full::document::*       │    │
│  │ bindings::tairitsu_browser::full::node::*           │    │
│  │ bindings::tairitsu_browser::full::console::*        │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## Commands

```bash
# Regenerate WIT
python3 scripts/generate_browser_wit.py

# Regenerate TypeScript glue
python3 scripts/generate_browser_glue.py

# Check TypeScript errors
cd packages/browser-glue && npx tsc --noEmit 2>&1 | grep -c "error TS"

# Build browser-glue
cd packages/browser-glue && npm run build

# Generate interface wrappers
python3 scripts/generate_interface_wrappers.py

# Build WASM component
cd examples/website && cargo build --target wasm32-wasip2 --lib --release
```
