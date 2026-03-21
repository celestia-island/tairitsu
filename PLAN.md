# WIT Bindings Refactoring Plan

## Overview

Refactor the browser-glue and WIT bindings system to ensure Rust code can properly call TypeScript glue implementations at runtime through jco transpiled imports.

## Current State

### Completed

1. **WIT Generation**
   - `scripts/generate_browser_wit.py` generates `browser-full.wit` automatically
   - Global singleton interfaces (window, document, etc.) no longer have `self` parameters
   - Storage interfaces have proper `self` parameters
   - URL.href correctly typed as `string` (not `u64`)
   - Stringifier attributes handled correctly

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

4. **Interface Wrappers**
   - `scripts/generate_interface_wrappers.py` creates interface-level wrappers
   - Generated: `document.js`, `node.js`, `window.js`, `event-target.js`

5. **Packager**
   - Import Map updated to: `"tairitsu-browser:full/": "./browser-glue/"`

### Remaining Tasks

1. **Rust Code Updates**
   - Update `packages/web/src/wit_platform.rs` for new WIT
   - Function names changed (e.g., `body()` -> `get_body()`)
   - Module locations changed (e.g., `node::set_attribute` -> `element::set_attribute`)

2. **Interface Wrapper Completion**
   - Add `console.js` and `style.js` wrappers
   - Regenerate after browser-glue build

3. **End-to-End Testing**
   - Build browser-glue
   - Build WASM component
   - Run dev server
   - Verify browser calls work

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
│  │ - document.js → cssGlue.js, domGlue.js              │    │
│  │ - node.js → domGlue.js                              │    │
│  │ - window.js → htmlGlue.js                           │    │
│  │ - console.js → deviceGlue.js                        │    │
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

# Check TypeScript errors (should be 0)
cd packages/browser-glue && npx tsc --noEmit 2>&1 | grep -c "error TS"

# Build browser-glue
cd packages/browser-glue && npm run build

# Generate interface wrappers
python3 scripts/generate_interface_wrappers.py

# Build WASM component
cd examples/website && cargo build --target wasm32-wasip2 --lib --release
```
