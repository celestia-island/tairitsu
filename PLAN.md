# PLAN: WASM Event System — Complete

## Status: ✅ ALL DONE

## Summary

### Phase 1: Event Bridge (3 bugs fixed, verified end-to-end)
1. **Patch system didn't carry handler closures** → Fixed in `patch.rs`, `diff.rs`
2. **`apply_patch()` was no-op for events** → Fixed in `wit_platform.rs:1930-1955`
3. **Runtime initial render silent drop** → Fixed in `runtime.rs:264-271`

### Phase 2: Re-render Mechanism (new)
4. **`init_runtime()` not called by website** → Wired into `lib.rs` with cfg-gated WASM path
5. **No initial VNode for diffing** → Added `store_initial_vnode()` to inject mounted VNode
6. **Sync fallback when no rAF scheduler** → Modified `schedule_render()` to call `flush_render()` directly
7. **Missing jco wrapper imports** → Added `nodeList.ts`, `getChildNodes()`, `insertBefore()` to runtime

### Interactive Components (new)
8. **Wi-Fi switch toggle** → `Cell<bool>` + `on_event("click", ...)` + `rerender()`
9. **Dark mode toggle** → `DARK_MODE Cell` + conditional `hi-layout-dark` class
10. **Language selector dropdown** → `LANG_OPEN Cell` + show/hide dropdown

## Acceptance Criteria (all met)
- [x] Clicking `.hi-switch` toggles visual state
- [x] Dark mode button switches theme
- [x] Language selector shows/hides dropdown
- [x] Event test page shows incrementing counter
- [x] 13 pages render via Playwright E2E (baseline snapshots saved)
- [x] 6/7 event bridge tests pass (1 pre-existing 404 noise)

## Event Flow (verified working)
```
RSX onclick: |e| { CLICK_COUNT.set(n+1); rerender(); }
  → VElement.event_handlers["click"] = EventHandler(Rc<RefCell<Closure>>)
  → render_vnode() → platform.add_event_listener(el, "click", handler)
    → WIT bindings → JS glue → element.addEventListener("click", listener_fn)
  → User clicks button
    → listener_fn(event) fires
      → callbacks.onMouseEvent(listenerId, eventHandle, data)
        → Rust BrowserComponent::on_mouse_event(...)
          → dispatch_event(id, "mouse", event)
            → EVENT_CALLBACKS[id](event)  // user's closure runs
              → CLICK_COUNT.increment + mark_dirty(id)
                → schedule_render() → flush_render()
                  → render_component(id): App.render() → diff(old, new) → apply_patches(DOM)
```

## Commits to `dev`
1. `d192f77` feat(reactive): wire re-render loop — init_runtime, store_initial_vnode, sync fallback, interactive switch
2. `d97556e` feat(website): add interactive dark mode toggle and language selector dropdown
3. `7c5ba91` fix(browser-glue): add missing node-list runtime + getChildNodes/insertBefore for jco wrapper
