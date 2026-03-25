# Tairitsu Migration Plan: Hand-Written Interface Removal

## Overview

This document outlines the completed migration from hand-written WIT interfaces (`console`, `style`, `event-target`) to W3C standard interfaces.

**Date:** 2026-03-25
**Status:** Completed

---

## Summary of Changes

| Removed Interface | Replacement Interface | Domain |
|-------------------|----------------------|--------|
| `tairitsu-browser:full/console` | *No direct replacement* | N/A |
| `tairitsu-browser:full/style` | `css-style-declaration` + `element-css-inline-style` | `css` |
| `tairitsu-browser:full/event-target` | `event-target` (W3C) + `event` | `dom` + `events` |

---

## Implementation Tasks

### Phase 1: CSS/Style Migration (Low Risk)
- [x] Update `packages/web/src/wit_platform.rs` to use W3C style interfaces
- [x] Update `packages/ssr/src/linker.rs` style implementation
- [x] Update `packages/ssr/src/interfaces/style.rs` or remove if integrated elsewhere
- [x] Test style operations in browser

### Phase 2: Event/EventTarget Migration (High Risk)
- [x] Update `packages/web/src/wit_platform.rs` to use W3C event interfaces
- [x] Update `packages/ssr/src/linker.rs` event implementation
- [x] Update `packages/ssr/src/interfaces/event_target.rs` or remove if integrated elsewhere
- [x] Verify callback registration through `event-callbacks`
- [x] Test event handling in browser

### Phase 3: Console Migration (Medium Risk)
- [x] Decide on console strategy (direct vs new interface)
- [x] Update `packages/web/src/wit_platform.rs` console calls
- [x] Update `packages/ssr/src/linker.rs` console implementation
- [x] Update `packages/ssr/src/interfaces/console.rs` or remove
- [x] Test logging in both browser and SSR

### Phase 4: Cleanup
- [x] Remove old interface files if not already done
- [x] Update documentation
- [x] Run full test suite
- [x] Update examples

---

## Testing Checklist

After migration, verify:
- [x] Style properties can be set/get/removed on elements
- [x] Event listeners can be added and removed
- [x] `prevent-default()` and `stop-propagation()` work correctly
- [x] Console logging works in both browser and SSR
- [x] All existing tests pass
- [x] No regressions in examples

---

## Migration Details

### Console Interface
- **Removed:** `tairitsu-browser:full/console` interface
- **For wasm32-unknown-unknown:** Uses wasm-bindgen direct console access
- **For wasm32-wasip2:** Console operations are no-ops (interface removed)
- **For SSR:** Uses `tracing` macros directly

### Style Interface
- **Migrated to:** W3C CSSOM interfaces
- `element-css-inline-style::get-style(element) -> style-handle`
- `css-style-declaration::set-property(style, property, value, priority)`
- `css-style-declaration::get-property-value(style, property) -> value`
- `css-style-declaration::remove-property(style, property) -> old-value`

### Event-Target Interface
- **Migrated to:** W3C standard event-target + event interfaces
- `event-target::add-event-listener(self, type, callback, options)`
- `event-target::remove-event-listener(self, type, callback, options)`
- `event::prevent-default(self)` and `event::stop-propagation(self)`

### Additional Fixes
- Fixed WIT reserved keyword `stream` in blob interface (renamed to `get-stream`)
- Fixed `stream` keyword in fetch.wit
- Fixed WIT reserved keyword `type` in mutation-entry (renamed to `mutation-type`)
- Added missing callback interfaces: timer-callbacks, animation-callbacks, resize-observer-callbacks, mutation-observer-callbacks
- Added platform-helpers interface implementation for SSR
