# PLAN: WASM Event System — Click Events Not Firing

## Status: 🔴 BUG — Root Cause Investigation Needed

## Symptom

**ALL `on_event("click", ...)` handlers registered via tairitsu-vdom are silently ignored.**
No JS errors, no Rust panics, no console warnings. The DOM elements render correctly,
event handlers are attached (in Rust), but clicking produces zero effect.

## Reproduction

```bash
cd hikari/examples/website && just dev --daemon
# Open http://localhost:3000/components/layer1/switch
# Click any Switch, the dark-mode toggle, or the language selector → nothing happens
```

## Debugging Tooling: Browser MCP (Playwright)

**We have Playwright MCP available for live debugging.** It connects via Edge (`--browser msedge`)
and can navigate, click, screenshot, snapshot DOM, evaluate JS, and inspect console errors.

Key commands for debugging this issue:
```
playwright_browser_navigate("http://localhost:3000/components/layer1/switch")
playwright_browser_snapshot()          # full accessibility tree with refs
playwright_browser_take_screenshot()   # visual verification
playwright_browser_click(ref="e910")  # click by ref from snapshot
playwright_browser_evaluate("() => { ... }")  # run JS to inspect DOM state
playwright_browser_console_messages(level="error")  # check for JS errors
```

Use `evaluate` to check if event listeners are actually attached:
```js
// Check if dropdown responds to clicks
const trigger = document.querySelector('.hi-select-trigger');
trigger.click();
getComputedStyle(document.querySelector('.hi-select-dropdown')).display; // should be "block"

// Check if switch toggles
document.querySelector('.hi-switch').click();
document.querySelector('.hi-switch').getAttribute('aria-checked'); // should change
```

## Evidence Collected (via Playwright MCP browser inspection)

### 0. Visual rendering — all correct ✅
Screenshot-verified: Home, Button, Feedback, Avatar, Tag, Empty, Switch pages all render perfectly.
MdiIcon icons display correctly, CSS variables apply, ARIA attributes present, layout intact.
**The problem is exclusively in the event bridge — not rendering.**

### 1. DOM is correct
- `.hi-select-dropdown` exists with correct HTML (all language options present)
- Initial state: `display:none`, `aria-expanded:"false"` ✅
- After click: **unchanged** — still `display:none`, `aria-expanded:"false"` ❌

### 2. All click handlers affected equally
| Element | Handler Type | Expected Behavior | Actual |
|---------|-------------|-------------------|--------|
| `.hi-switch` (hikari-components) | `on_event("click", ...)` | toggle checked state | no response |
| Dark mode button (`aside_footer.rs`) | `on_event("click", theme_on_click)` | toggle theme class | no response |
| Language trigger (`.hi-select-trigger`) | `on_event("click", trigger_on_click)` | show dropdown | no response |
| Dropdown backdrop | `on_event("click", backdrop_on_click)` | close dropdown | N/A (never opens) |
| Dropdown options | `on_event("click", ...)` | select language + reload | N/A |

### 3. JS `.click()` also fails
```js
document.querySelector('.hi-select-trigger').click();
// getComputedStyle(dropdown).display === "none"  ← unchanged
```
This rules out Playwright-specific issues — even native JS dispatch doesn't reach the Rust handler.

### 4. No console errors
Only error is `favicon.ico` 404 — completely unrelated.

### 5. Visual rendering is perfect
All components render correctly: icons (MdiIcon SVG), CSS variables, ARIA attributes,
layout, variants, sizes. The problem is **exclusively in the event bridge**.

## Hypotheses (ordered by likelihood)

### H1 (Most Likely): Event listener not actually attached to DOM
The vdom diff/patch process may create the element and set attributes but **skip
registering the Wasm-side event callback** with the browser's `addEventListener`.

**Where to look:**
- `packages/vdom/src/vnode.rs` — the `VElement::on_event()` method: does it store the handler?
- The **patching/diffing algorithm** that reconciles virtual DOM → real DOM:
  - When a new VElement with `on_event` is mounted, is `addEventListener` called?
  - Is there a code path where event registration is conditional on some flag that isn't set?
- Check if there's an **event registry / callback map** that maps DOM event type + element ID → Rust closure
- Look for `addEventListener` or equivalent in the WASI browser bindings

**Quick check:** Search for `add_event_listener`, `onclick`, `event_callback`, `on_event` in the vdom patch code.

### H2: Event dispatcher receives the call but drops it
The JS→Wasm bridge works (listener IS attached), but the callback lookup fails or
the closure doesn't execute.

**Where to look:**
- The **event dispatch function** that receives callbacks from JS
- Is there a `match` on event type that misses `"click"`? (e.g., only handles `"input"`, `"change"`)
- Is the element handle/u64 ID correctly preserved between mount time and dispatch time?
- Could there be a **lifetime issue** where closures are dropped after the initial render?

### H3: `ref_()` handles are invalid at click time
The `trigger_on_click` closure calls `select_ref_t.borrow()` → `downcast_ref::<u64>()`.
If the ref was never populated (element not yet mounted when ref runs), the handler
silently does nothing.

**Evidence against:** Even simple handlers like theme toggle (which just flips a Cell bool)
don't fire, so this can't explain ALL failures. But it could compound the issue for
handlers that do DOM manipulation via refs.

### H4: WASI browser bindings event API gap
The `tairitsu_browser::full::event` or equivalent bindings may not properly expose
click events to the WASM side, or may require explicit opt-in per event type.

**Where to look:**
- `packages/web/src/wit_platform/` or wherever browser bindings are generated
- The WIT/world definition for event handling
- Compare with working events (if any) — e.g., does `oninput` work but `onclick` not?

## Suggested Debugging Steps

1. **Add a `console.log` / `println!` in the event bridge entry point**
   - If it fires → problem is in dispatch/closure lookup (H2/H3)
   - If it doesn't → problem is in listener attachment (H1/H4)

2. **Check if ANY event type works**
   - Try `on_event("input", ...)` on a text input
   - Try `on_event("change", ...)` on a checkbox
   - This would confirm/narrow to click-specific vs all-events broken

3. **Inspect the actual DOM listeners**
   ```js
   const el = document.querySelector('.hi-select-trigger');
   // Check if addEventListener was called:
   getEventListeners(el);  // DevTools-only, or use custom wrapper
   // Or check __tairitsu_events / similar internal property
   ```

4. **Verify the patch loop calls event binding**
   - Set a breakpoint or log in the vdom mount/patch function
   - Confirm `on_event` entries in VElement are iterated and bound

5. **Check for a global event delegation model**
   - Some vdom libs use a single `window.addEventListener('click', ...)`
   - with event delegation via data attributes
   - If this exists, check if the delegate handler is registered

## Files Most Likely Relevant

```
packages/vdom/
├── src/vnode.rs          # VElement::on_event(), VNode construction
├── src/patch.rs          # (or equiv) DOM diffing & patching logic
├── src/dom.rs            # (or equiv) DOM operations: createElement, addEventListener
└── src/lib.rs            # Public API surface

packages/web/
├── src/wit_platform/     # WASI browser bindings (generated or hand-written)
│   └── wasm_impl/bindings/tairitsu_browser/full/
│       └── event.*       # Event handling bindings
└── src/                  # Runtime glue between vdom + browser
```

## Acceptance Criteria

- [ ] Clicking a `.hi-switch` toggles its visual state
- [ ] Clicking dark mode button switches `data-theme` between `hikari` ↔ `tairitsu`
- [ ] Clicking language selector shows dropdown menu (changes `display` to `block`)
- [ ] Clicking a language option triggers `set_locale()` + page reload
- [ ] Clicking dropdown backdrop closes the dropdown
