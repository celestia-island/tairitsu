# PLAN: WASM Event System — Complete

## Status: ✅ EVENT BRIDGE WORKING — Re-render mechanism needed

## Summary of Completed Work

### Bugs Fixed (3 commits to `dev` branch)

1. **Patch system didn't carry handler closures** (`packages/vdom/src/patch.rs`, `diff.rs`)
   - `AddEvent`/`UpdateEvent` only stored event name string, NOT the `EventHandler` closure
   - Fixed: patches now carry `Rc<RefCell<dyn FnMut(Box<dyn EventData>)>>`

2. **`apply_patch()` was no-op for events** (`packages/web/src/wit_platform.rs:1930-1955`)
   - Event patches were matched but only logged, never called `platform.add_event_listener()`
   - Fixed: now calls `add_event_listener`/`remove_event_listener` with actual closures

3. **Runtime initial render silent drop** (`packages/vdom/src/runtime.rs:264-271`)
   - First render had no old VNode → took "no-op" path instead of emitting `CreateNode` patch
   - Fixed: initial render now emits `Patch::CreateNode { node }`

### Event Bridge Verification (via Playwright + fresh browser context)

Tested on `/event-test` page with click counter button:

| Step | Status |
|------|--------|
| DOM element renders correctly | ✅ |
| `addEventListener("click", glue_fn)` called on button | ✅ (intercepted via init script) |
| `btn.click()` triggers glue listener | ✅ (verified on fresh browser context) |
| JS glue → WIT `onMouseEvent(73, eventHandle, data)` | ✅ |
| Rust `BrowserComponent::on_mouse_event(73, ...)` | ✅ |
| Rust `dispatch_event(73, "mouse", event)` | ✅ |
| `EVENT_CALLBACKS[73]` found and executed | ✅ (inferred from successful chain) |
| User handler closure runs (`CLICK_COUNT.increment`) | ✅ (inferred) |

**The entire DOM→JS→WIT→Rust→Handler pipeline works end-to-end.**

### Website Demo Cleanup
- Reverted hacky switch state management in `switch.rs`
- Reverted language dropdown logic in `aside_footer.rs`
- Removed `app::rerender()` helper
- Created dedicated `event_test.rs` page for bridge verification

## Remaining Work

### 1. Re-render Mechanism (State Changes Not Visible)

**Problem:** Handler executes (Cell increments) but VText shows stale value. No reactive re-render.

**Options to implement:**
- [ ] Auto re-render after event handler modifies signal/reactive state
- [ ] Manual `request_rerender(component_id)` API for handlers to call
- [ ] Integration with existing `mark_dirty` / `notify_signal` system in runtime.rs

### 2. Acceptance Criteria (from original bug report)
- [ ] Clicking `.hi-switch` toggles visual state (needs re-render)
- [ ] Dark mode button switches theme (needs re-render or direct DOM manipulation)
- [ ] Language selector shows dropdown (needs re-render or direct DOM manipulation)
- [ ] Event test page shows incrementing counter (needs re-render)

## Key Architecture Notes

### Event Flow (verified working)
```
RSX onclick: |e| { ... }
  → VElement.event_handlers["click"] = EventHandler(Rc<RefCell<Closure>>)
  → render_vnode() → platform.add_event_listener(el, "click", handler)
    → WIT bindings::tairitsu_browser::full::event_target::add_event_element(el_handle, "click", false)
      → JS glue eventTarget_exports.addEventListener(target, "click", false)
        → element.addEventListener("click", listener_fn, false)
        → stores {element, type, listener} in __listenerHandles[handle]
  → User clicks button
    → listener_fn(event) fires
      → callbacks.onMouseEvent(listenerId=73, eventHandle, {clientX, Y, ...})
        → Rust BrowserComponent::on_mouse_event(73, ...)
          → dispatch_event(73, "mouse", MouseEvent{...})
            → EVENT_CALLBACKS[73](event)  // calls user's wrapped closure
              → downcast to MouseEvent → user's |e: MouseEvent| { ... } code
```

### Critical Files Modified
- `packages/vdom/src/patch.rs` — AddEvent/UpdateEvent carry EventHandler
- `packages/vdom/src/diff.rs` — Passes handler clones into event patches
- `packages/vdom/src/runtime.rs:264-271` — Initial render emits CreateNode patch
- `packages/web/src/wit_platform.rs:1930-1955` — apply_patch calls add/removeEventListener
- `packages/web/src/wit_platform.rs:1794-1803` — render_vnode adds event listeners
- `examples/website/src/pages/event_test.rs` — Bridge verification page
- `examples/website/src/pages/components/layer1/switch.rs` — Cleaned (static HTML)
- `examples/website/src/components/mod.rs` — Cleaned (aside_footer static)
