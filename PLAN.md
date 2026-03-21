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
   - `scripts/generate_browser_glue.py` generates glue files to `src/` (not `src/generated/`)
   - `src/generated/` directory deleted
   - 28 domains, 454 interfaces, 3974 functions

3. **Interface Wrappers**
   - `scripts/generate_interface_wrappers.py` creates interface-level wrappers
   - Generated: `document.js`, `node.js`, `window.js`, `event-target.js`

4. **Packager**
   - Import Map updated to: `"tairitsu-browser:full/": "./browser-glue/"`

### Remaining Issues

1. **TypeScript Compilation Errors (29 errors)**
   - Event handler type mismatches (EventHandler vs bigint)
   - Some properties returning wrong types
   - Need to update `scripts/generator/config.py`

2. **Rust Code Updates (incomplete)**
   - Function names changed (e.g., `body()` -> `get_body()`)
   - Return types changed (e.g., `Result<T>` -> `T` directly)
   - Module locations changed (e.g., `node::set_attribute` -> `element::set_attribute`)
   - New interfaces: `console::log/warn/error`, `style::set_style_property`

3. **Interface Wrapper Completion**
   - Need to add `console.js` and `style.js` wrappers
   - Need to regenerate after TS compilation is fixed

4. **End-to-End Testing**
   - Build browser-glue
   - Build WASM component
   - Run dev server
   - Verify browser calls work

## Action Items

### Phase 1: Fix TypeScript Compilation

1. Continue fixing errors in `scripts/generator/config.py`
2. Key error patterns:
   - `EventHandler` types should be `u64` (already fixed in WIT generator)
   - Some properties return `null` but WIT expects `bigint`
   - Some getters/setters have type mismatches

### Phase 2: Complete Rust Code Updates

Update `packages/web/src/wit_platform.rs`:

```rust
// Old:
bindings::tairitsu_browser::full::document::body()
bindings::tairitsu_browser::full::document::create_element(tag)
bindings::tairitsu_browser::full::node::set_attribute(el, name, value)
bindings::tairitsu_browser::full::window::console_log(msg)

// New:
bindings::tairitsu_browser::full::document::get_body()  // returns option<u64>
bindings::tairitsu_browser::full::document::create_element(tag, None)  // takes option
bindings::tairitsu_browser::full::element::set_attribute(el, name, value)  // different module
bindings::tairitsu_browser::full::console::log(msg)  // different module
```

Key changes:
- `document::body()` -> `document::get_body()` returns `option<u64>`
- `document::create_element(tag)` -> `create_element(tag, None)` takes optional options
- `document::get_element_by_id()` -> `non_element_parent_node::get_element_by_id()`
- `node::set_attribute()` -> `element::set_attribute()` (also returns void, not Result)
- `node::append_child()` -> returns `u64` (the node), not `Result`
- `window::console_log()` -> `console::log()`
- `window::inner_width()` -> `window::get_inner_width()` returns `s32`

### Phase 3: Complete Interface Wrappers

1. Add to `scripts/generate_interface_wrappers.py`:
   ```python
   USED_INTERFACES = {
       # ... existing ...
       "console": ["log", "warn", "error"],
       "style": ["set-style-property", "get-style-property", "remove-style-property"],
   }
   ```

2. Regenerate wrappers after TS build succeeds

### Phase 4: Build and Test

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
│  │ tairitsu_website.js (transpiled)                    │    │
│  │ import { createElement } from 'tairitsu-browser:full/document'│
│  │ import { appendChild } from 'tairitsu-browser:full/node'     │
│  │ import { log } from 'tairitsu-browser:full/console'          │
│  └─────────────────────────────────────────────────────┘    │
│                           ↑                                  │
│  ┌─────────────────────────────────────────────────────┐    │
│  │ Rust wit-bindgen bindings                           │    │
│  │ bindings::tairitsu_browser::full::document::*       │    │
│  │ bindings::tairitsu_browser::full::node::*           │    │
│  │ bindings::tairitsu_browser::full::console::*        │    │
│  └─────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
```

## Files Changed

### Modified
- `scripts/generate_browser_wit.py` - Auto-generate browser-full.wit with proper structure
- `scripts/generate_browser_glue.py` - Output to `src/` instead of `src/generated/`
- `scripts/generate_interface_wrappers.py` - NEW: Generate interface wrappers
- `packages/packager/src/wasm/mod.rs` - Updated Import Map
- `packages/web/src/wit_platform.rs` - Partial update for new WIT (incomplete)

### Generated
- `packages/browser-worlds/wit/browser-full.wit` - Now auto-generated
- `packages/browser-glue/src/*.ts` - Auto-generated glue files
- `packages/browser-glue/dist/browser-glue/*.js` - Interface wrappers

### Deleted
- `packages/browser-glue/src/generated/` - Moved contents up
- `packages/browser-glue/src/dom.ts`, `events.ts`, `http.ts`, `canvas.ts` - Hand-written duplicates
- `packages/browser-glue/dist/*-glue.js` - Legacy files

## Commands

```bash
# Regenerate WIT
python3 scripts/generate_browser_wit.py

# Regenerate TypeScript glue
python3 scripts/generate_browser_glue.py

# Build browser-glue
cd packages/browser-glue && npm run build

# Generate interface wrappers
python3 scripts/generate_interface_wrappers.py

# Build WASM component
cd examples/website && cargo build --target wasm32-wasip2 --lib --release

# Check TypeScript errors
cd packages/browser-glue && npx tsc --noEmit 2>&1 | wc -l
```
