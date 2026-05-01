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
