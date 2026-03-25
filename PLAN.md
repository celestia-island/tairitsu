# Tairitsu Migration Plan: Hand-Written Interface Removal

## Overview

This document outlines the migration plan for removing hand-written WIT interfaces (`console`, `style`, `event-target`) and replacing them with auto-generated W3C standard interfaces.

**Date:** 2026-03-25
**Status:** Ready for Implementation
**Priority:** High

---

## Summary of Changes

| Removed Interface | Replacement Interface | Domain |
|-------------------|----------------------|--------|
| `tairitsu-browser:full/console` | *No direct replacement* | N/A |
| `tairitsu-browser:full/style` | `css-style-declaration` + `element-css-inline-style` | `css` |
| `tairitsu-browser:full/event-target` | `event-target` (W3C) + `event` | `dom` + `events` |

---

## Affected Files

### Rust Code
- `packages/web/src/wit_platform.rs` - Uses all three removed interfaces
- `packages/ssr/src/linker.rs` - Implements all three removed interfaces
- `packages/ssr/src/interfaces/console.rs` - Console implementation
- `packages/ssr/src/interfaces/style.rs` - Style implementation
- `packages/ssr/src/interfaces/event_target.rs` - EventTarget implementation

### TypeScript/JavaScript
- `packages/browser-glue/src/glue/index.ts` - Removed exports
- `packages/browser-glue/src/runtime/registry.ts` - Removed registrations

---

## Detailed Migration Guide

### 1. Console Interface (`tairitsu-browser:full/console`)

**Removed Functions:**
```wit
log(message: string)
warn(message: string)
error(message: string)
```

**Migration Strategy:**
- **No W3C replacement exists** - Console is a namespace, not an interface
- **Option 1:** Use browser's `console.log()` directly in glue code
- **Option 2:** Create a simple `logging` interface for WASM → host logging
- **Recommended:** For SSR, continue using `tracing` macros directly

**Files to Update:**
1. `packages/web/src/wit_platform.rs` (lines 686-713)
2. `packages/ssr/src/linker.rs` (lines 41-64)
3. `packages/ssr/src/interfaces/console.rs` (entire file)

**Example Migration (Rust):**
```rust
// OLD (removed)
bindings::tairitsu_browser::full::console::log(&message);

// NEW (direct console)
// In browser glue: console.log(message);
// In SSR: tracing::info!("{}", message);
```

---

### 2. Style Interface (`tairitsu-browser:full/style`)

**Removed Functions:**
```wit
set-style-property(element: u64, property: string, value: string) -> result<_, string>
get-style-property(element: u64, property: string) -> option<string>
remove-style-property(element: u64, property: string) -> result<_, string>
```

**Replacement (W3C Standard):**

```wit
// Step 1: Get the style object from element
element-css-inline-style::get-style(element: u64) -> u64

// Step 2: Use CSSStyleDeclaration methods
css-style-declaration::set-property(style: u64, property: string, value: string, priority: option<string>)
css-style-declaration::get-property-value(style: u64, property: string) -> string
css-style-declaration::remove-property(style: u64, property: string) -> string
```

**Key Differences:**
1. Two-step process: Get style handle, then operate on it
2. `set-property` takes optional `priority` parameter (e.g., "important")
3. `remove-property` returns the old value (not result)
4. Methods use `self` parameter (OOP style)

**Files to Update:**
1. `packages/web/src/wit_platform.rs` (lines 753-759)
2. `packages/ssr/src/linker.rs` (lines 263-304)
3. `packages/ssr/src/interfaces/style.rs` (entire file)

**Example Migration (Rust):**
```rust
// OLD (removed)
bindings::tairitsu_browser::full::style::set_style_property(element, "color", "red")?;

// NEW (W3C standard)
let style = bindings::tairitsu_browser::css::element_css_inline_style::get_style(element);
bindings::tairitsu_browser::css::css_style_declaration::set_property(
    style, "color", "red", None
);

// For SSR: update the linker to use new interface names
// "tairitsu-browser:full/style@0.2.0" → "tairitsu-browser:css/css-style-declaration@0.2.0"
// AND "tairitsu-browser:css/element-css-inline-style@0.2.0"
```

---

### 3. Event Target Interface (`tairitsu-browser:full/event-target`)

**Removed Functions:**
```wit
add-event-listener(target: u64, event-type: string, use-capture: bool) -> result<u64, string>
remove-event-listener(target: u64, listener-id: u64) -> result<_, string>
prevent-default(event: u64)
stop-propagation(event: u64)
```

**Replacement (W3C Standard):**

```wit
// EventTarget methods (in dom domain)
event-target::add-event-listener(
    self: u64,           // event target handle
    type: string,        // event type
    callback: option<u64>, // event listener callback handle
    options: option<bool>  // use capture
)

event-target::remove-event-listener(
    self: u64,
    type: string,
    callback: option<u64>,
    options: option<bool>
)

event-target::dispatch-event(self: u64, event: u64) -> bool

// Event methods (in events domain)
event::prevent-default(self: u64)
event::stop-propagation(self: u64)
event::stop-immediate-propagation(self: u64)
```

**Key Differences:**
1. Uses `self` parameter (OOP style)
2. Requires callback handle (from `event-callbacks` export)
3. No listener-id return - use callback handle directly
4. `prevent-default` and `stop-propagation` are on `Event` interface, not `EventTarget`

**Files to Update:**
1. `packages/web/src/wit_platform.rs` (lines 766-810)
2. `packages/ssr/src/linker.rs` (lines 310-337)
3. `packages/ssr/src/interfaces/event_target.rs` (entire file)

**Example Migration (Rust):**
```rust
// OLD (removed)
let listener_id = bindings::tairitsu_browser::full::event_target::add_event_listener(
    element, "click", false
)?;
bindings::tairitsu_browser::full::event_target::remove_event_listener(element, listener_id)?;
bindings::tairitsu_browser::full::event_target::prevent_default(event)?;

// NEW (W3C standard)
// Note: Requires callback registration through event-callbacks export
bindings::tairitsu_browser::dom::event_target::add_event_listener(
    element, "click", Some(callback_handle), Some(false)
);
bindings::tairitsu_browser::dom::event_target::remove_event_listener(
    element, "click", Some(callback_handle), Some(false)
);
bindings::tairitsu_browser::events::event::prevent_default(event)?;
```

---

## Implementation Tasks

### Phase 1: CSS/Style Migration (Low Risk)
- [ ] Update `packages/web/src/wit_platform.rs` to use W3C style interfaces
- [ ] Update `packages/ssr/src/linker.rs` style implementation
- [ ] Update `packages/ssr/src/interfaces/style.rs` or remove if integrated elsewhere
- [ ] Test style operations in browser

### Phase 2: Event/EventTarget Migration (High Risk)
- [ ] Update `packages/web/src/wit_platform.rs` to use W3C event interfaces
- [ ] Update `packages/ssr/src/linker.rs` event implementation
- [ ] Update `packages/ssr/src/interfaces/event_target.rs` or remove if integrated elsewhere
- [ ] Verify callback registration through `event-callbacks`
- [ ] Test event handling in browser

### Phase 3: Console Migration (Medium Risk)
- [ ] Decide on console strategy (direct vs new interface)
- [ ] Update `packages/web/src/wit_platform.rs` console calls
- [ ] Update `packages/ssr/src/linker.rs` console implementation
- [ ] Update `packages/ssr/src/interfaces/console.rs` or remove
- [ ] Test logging in both browser and SSR

### Phase 4: Cleanup
- [ ] Remove old interface files if not already done
- [ ] Update documentation
- [ ] Run full test suite
- [ ] Update examples

---

## Risk Assessment

| Interface | Risk Level | Reason |
|-----------|-----------|--------|
| `style` | **Low** | Direct mechanical translation, well-defined W3C replacement |
| `console` | **Medium** | No direct W3C replacement, requires architectural decision |
| `event-target` | **High** | Callback model change, affects core event system |

---

## Testing Checklist

After migration, verify:
- [ ] Style properties can be set/get/removed on elements
- [ ] Event listeners can be added and removed
- [ ] `prevent-default()` and `stop-propagation()` work correctly
- [ ] Console logging works in both browser and SSR
- [ ] All existing tests pass
- [ ] No regressions in examples

---

## Notes

1. **Event Callbacks**: The new W3C `event-target` interface requires callback handles to be registered through the `event-callbacks` export interface. This is a significant architectural change from the listener-id model.

2. **Interface Naming**: W3C interfaces use kebab-case (e.g., `css-style-declaration`) while the old hand-written interfaces used simple names (e.g., `style`).

3. **Package Paths**: The new interfaces are in different packages:
   - `tairitsu-browser:dom/*` - DOM interfaces
   - `tairitsu-browser:css/*` - CSS interfaces
   - `tairitsu-browser:events/*` - Event interfaces

4. **SSR Compatibility**: The SSR implementations will need to be updated to use the new W3C interface names while maintaining the same functionality.
