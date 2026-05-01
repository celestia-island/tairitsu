# PLAN2: Debug Browser Automation & Visual Regression Testing

## Phase 1 — ✅ Complete

Embedded debug API server (`--debug` flag) with wry-based WebView engine.
- `packages/packager/src/debug/mod.rs` — Axum HTTP server + wry WebView engine
- 9 endpoints: health, info, navigate, screenshot, click, type, evaluate, console, dom
- IPC-based JS execution via `window.ipc.postMessage`
- Cross-platform via wry+tao (WebKitGTK on Linux, WebKit on macOS, WebView2 on Windows)
- Headless support via Xvfb on Linux

## Phase 2 — ✅ Complete

Pixel-level visual diffing engine.
- `packages/packager/src/visual_diff/mod.rs` — compare, diff image, HTML report
- `VisualDiff` CLI subcommand with tolerance and baseline management
- HTML report with side-by-side diff images, JSON report output

## Phase 3 — ✅ Complete

CI integration pipeline.
- `.github/workflows/visual-regression.yml` — build, capture, diff, upload artifacts
- Auto baseline update on main push, PR comment with results

## Phase 4 — ✅ Complete

Unified `tairitsu test` CLI subcommand.
- `packages/packager/src/test_runner/mod.rs` — visual regression + event bridge tests
- `tairitsu test --url --baseline-dir --events --update-baselines`
- Calls debug API directly via reqwest (no external browser binary needed)
- `packages/web-test/` kept as standalone `cargo test` entry point (same logic)

## Future Enhancements

- [ ] SSIM/structural similarity metric alongside pixel diff ratio
- [ ] Per-component threshold configuration in Cargo.toml metadata
- [ ] Interactive web UI for baseline review/approval
- [ ] WASM event bridge click simulation fix (el.click() does not trigger WASM listeners)

---

## Phase 5 — Hikari Integration Requirements (from consumer side)

> **Consumer:** `hikari-e2e` package → **Provider:** `tairitsu-debug` HTTP API (wry)
>
> Hikari's visual debugging pipeline (`hikari-visual-debug-wry`) now routes all browser
> automation through tairitsu-debug instead of launching Chromium directly via chromiumoxide.
> Below are requirements and improvement requests discovered during integration.

### 5.1 — Screenshot Quality (Critical)

**Problem:** Current Canvas-based screenshot (`canvas.toDataURL`) does NOT capture:
- CSS `box-shadow` / `filter: drop-shadow()` (FUI glow effects invisible)
- CSS gradients (`linear-gradient`, `radial-gradient`)
- `<img>` elements and background images
- `backdrop-filter: blur()` and other compositing effects
- Sub-pixel anti-aliased text rendering

**Impact:** Hikari's FUI glow system (button glows, switch checked-state glows, table container shadows)
is invisible in wry screenshots, making AI vision analysis unreliable for visual quality assessment.

**Request:**
- [ ] **Add pixel-level screenshot mode** using platform-native capture APIs:
  - Linux: `XComposite` / `XFixes` or `pipewire` portal screencopy
  - macOS: `CGWindowListCreateImage`
  - Windows: `PrintWindow` / `Windows.Graphics.Capture`
- [ ] Fallback: Allow injecting a JS snippet that re-renders shadows/gradients as solid colors before canvas capture
- [ ] New endpoint or flag: `POST /screenshot { "mode": "pixel" | "canvas" }`

### 5.2 — WASM Hydration Readiness

**Problem:** After `POST /navigate`, the page HTML loads but WASM module may still be compiling/hydrating.
Current workaround is `sleep(800ms)` which is fragile — too short on slow machines, wasteful on fast ones.

**Request:**
- [ ] **`GET /ready`** endpoint that polls for WASM hydration completion:
  - Checks `globalThis.__wasmExports !== undefined` AND
  - Checks custom hydration marker (e.g., `document.documentElement.dataset.hydration === "complete"`)
  - Returns `{ ready: bool, wasm_loaded: bool, hydrated: bool, wait_ms: u64 }`
- [ ] Auto-inject hydration marker into served HTML when `--debug` is active
- [ ] `POST /navigate` accepts `"wait_for": "hydration"` option to block until ready

### 5.3 — Batch / Concurrent Operations

**Problem:** All debug API operations are sequential over a single HTTP connection.
Capturing 13+ routes sequentially takes ~20-30 seconds.

**Request (Nice-to-have):**
- [ ] `POST /batch` endpoint accepting array of operations to execute sequentially server-side:
  ```json
  [{ "op": "navigate", "url": "/button" },
   { "op": "screenshot", "name": "button" },
   { "op": "navigate", "url": "/table" },
   { "op": "screenshot", "name": "table" }]
  ```
  Returns zip archive of named screenshots — eliminates network round-trip per operation
- [ ] Response compression for base64 screenshot payloads (`Content-Encoding: gzip`)

### 5.4 — Enhanced DOM Inspection

**Current:** `GET /dom?selector=...` returns tag, text, html, attributes, visible, count.

**Requests:**
- [ ] Add computed style query: `GET /dom/computed?selector=...&property=...`
  - Returns resolved CSS values (critical for verifying FUI glow `box-shadow` values)
- [ ] Add bounding client rect with sub-element info:
  ```json
  { "rect": { "x": 100, "y": 200, "width": 300, "height": 40 },
    "children_visible": 5, "children_total": 8,
    "overflowing": false }
  ```
- [ ] Add accessibility tree snapshot: `GET /a11y?selector=...` (name, role, description)

### 5.5 — Console Log Enhancements

**Requests:**
- [ ] Filter by level: `GET /console?level=error,warn`
- [ ] Filter by source (WASM vs JS vs console plugin):
  - WASM logs from `console_error_hook` / `console_log_hook`
  - Trunk/Polyfill injected scripts
  - Application code
- [ ] Clear log buffer: `DELETE /console` (useful between test steps)
- [ ] Structured error stack trace parsing (not just raw text)

### 5.6 — Network & Performance Observability

**Requests:**
- [ ] `GET /network` returns list of resources loaded (URL, status, size, duration, content-type)
- [ ] Ability to intercept/block/throttle requests (for testing loading/error states)
- [ ] `GET /performance` returns timing metrics:
  - WASM compile time, hydration time, first paint, first contentful paint
  - Memory usage if available

### 5.7 — Keyboard & Input Extensions

**Requests:**
- [ ] `POST /press` for single key presses: `{ "key": "Enter" }` / `{ "key": "Tab" }` / `{ "key": "Escape" }`
- [ ] Modifier key support: `{ "key": "a", "modifiers": ["Control"] }` for Ctrl+A etc.
- [ ] `POST /drag` for drag-and-drop: `{ "from_selector": "...", "to_selector": "..." }`
- [ ] `POST /scroll` for precise scroll: `{ "selector": "...", "x": 0, "y": 200 }` or `"end"`

### 5.8 — Viewport & Responsive Testing

**Requests:**
- [ ] `POST /resize { width, height }` to change WebView viewport size
- [ ] `GET /viewport` returns current dimensions + device pixel ratio
- [ ] Preset profiles: `POST /resize { preset: "mobile" | "tablet" | "desktop" | "wide" }`

### 5.9 — CI / Headless Improvements

**Requests:**
- [ ] **Linux headless without Xvfb**: Use `wry`'s offscreen rendering mode
  (WebkitGTK has offscreen mode; avoids Xvfb dependency in CI)
- [ ] `--debug-headless` flag that auto-detects display availability
- [ ] Graceful degradation: if no display available, fall back to offscreen mode automatically
- [ ] Exit code / health check reflects browser initialization success/failure

### 5.10 — Error Diagnostics

**Requests:**
- [ ] `GET /errors` returns structured JS errors caught via `window.onerror` + unhandled promise rejections
- [ ] Source map resolution for minified WASM/JS stack traces
- [ ] WebSocket connection status (for live-reload / HMR state)

### Priority Matrix

| # | Requirement | Priority | Effort | Hikari Blocker? |
|---|------------|----------|--------|----------------|
| 5.1 Pixel screenshots | **P0** | High | Yes — FUI glow invisible |
| 5.2 WASM hydration readiness | **P1** | Low | No — sleep works but fragile |
| 5.3 Batch operations | P2 | Medium | No — sequential is fine |
| 5.4 Enhanced DOM | P1 | Low | No — evaluate() workaround |
| 5.5 Console filtering | P2 | Low | No |
| 5.6 Network/perf | P3 | Medium | No |
| 5.7 Keyboard extensions | P2 | Low | No |
| 5.8 Viewport/responsive | P2 | Low | No |
| 5.9 Headless without Xvfb | **P1** | Medium | Yes — CI needs this |
| 5.10 Error diagnostics | P2 | Low | No |
