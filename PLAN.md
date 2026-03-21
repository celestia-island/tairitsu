# WIT Bindings Refactoring Plan

## Overview

Refactor the browser-glue and WIT bindings system to ensure Rust code can properly call TypeScript glue implementations at runtime through jco transpiled imports.

## Completed ✅

1. **WIT Generation**
   - `scripts/generate_browser_wit.py` generates `browser-full.wit` automatically
   - Global singleton interfaces (window, document, etc.) no longer have `self` parameters
   - Storage interfaces have proper `self` parameters
   - URL.href correctly typed as `string` (not `u64`)
   - Stringifier attributes handled correctly
   - **Union type handling**: bool > string > numeric > interface priority

2. **TypeScript Glue**
   - `scripts/generate_browser_glue.py` generates glue files to `src/`
   - 28 domains, 454 interfaces, 3974 functions
   - **0 TypeScript compilation errors** ✅

3. **Code Generator Fixes**
   - Fixed ast_parser.py bug: global singleton first param not treated as self_param
   - Added GLOBAL_SINGLETONS: crypto, screen, location, history, performance
   - Added EVENT_HANDLER_PROPERTIES config for event handler getters
   - Added event handler synthetic type and lookup logic
   - Fixed ENUM_SETTER_PROPERTIES for string-type parameters
   - Removed innerHTML/outerHTML from enum properties (plain strings)

4. **Manual Interface Implementations**
   - `consoleGlue.ts`: log, warn, error
   - `styleGlue.ts`: setStyleProperty, getStyleProperty, removeStyleProperty
   - `eventTargetGlue.ts`: addEventListener, removeEventListener, preventDefault, stopPropagation

5. **Interface Wrappers**
   - `scripts/generate_interface_wrappers.py` creates interface-level wrappers
   - Generated: document.js, node.js, window.js, console.js, style.js, event-target.js

6. **Packager**
   - Import Map updated to: `"tairitsu-browser:full/": "./browser-glue/"`

7. **Rust Code Updates**
   - Updated `packages/web/src/wit_platform.rs` for new WIT
   - Fixed `get_element_by_id` to use `non_element_parent_node` interface
   - Fixed `set_style_property` result handling
   - All function names updated (e.g., `body()` -> `get_body()`)

8. **End-to-End Testing**
   - WASM component builds successfully ✅
   - E2E tests pass (7/7) ✅
   - No TODO/FIXME/Mock implementations ✅

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
│  │ - document.js, node.js, window.js                  │    │
│  │ - console.js, style.js, event-target.js            │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
                           ↑ WIT imports
┌─────────────────────────────────────────────────────────────┐
│                   WASM Component (jco)                       │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ Rust wit-bindgen bindings                           │    │
│  │ bindings::tairitsu_browser::full::document::*       │    │
│  │ bindings::tairitsu_browser::full::console::*        │    │
│  │ bindings::tairitsu_browser::full::style::*          │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## Commands

```bash
# Regenerate WIT
python3 scripts/generate_browser_wit.py

# Regenerate TypeScript glue
python3 scripts/generate_browser_glue.py

# Check TypeScript errors (should be 0)
cd packages/browser-glue && npx tsc --noEmit 2>&1 | grep -c "error TS"

# Build browser-glue
cd packages/browser-glue && npm run build

# Generate interface wrappers
python3 scripts/generate_interface_wrappers.py

# Build WASM component
cd examples/website && cargo build --target wasm32-wasip2 --lib --release

# Run E2E tests
cd packages/e2e && cargo test
```
