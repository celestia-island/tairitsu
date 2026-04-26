# PLAN2: Playwright-Based Visual Regression & Batch Style Testing

## Status: ✅ Phase 1 Implemented — Ready for Use

## What Was Built

### `packages/web-test/` — Self-contained Playwright test suite
| File | Purpose |
|------|---------|
| `package.json` | Node.js package with `@playwright/test`, `tsx` |
| `playwright.config.ts` | Chromium config, screenshot + HTML/JSON reporters |
| `page-registry.ts` | 13 pages (home + 11 layer1 components + event_test) with interaction specs |
| `tests/visual.spec.ts` | Screenshot capture for all pages + interactive states (hover, click) |
| `tests/events.spec.ts` | WASM event bridge verification (listener registration, onMouseEvent dispatch, Cell mutation) |

### `scripts/` — PowerShell CLI scripts
| Script | Purpose |
|--------|---------|
| `e2e-capture.ps1` | Batch-screenshot all 13 pages → `target/e2e_screenshots/<timestamp>/` |
| `e2e-verify.ps1` | Full pipeline: server check → capture → WASM bridge verify → Markdown report |

### `justfile` recipes
```
just e2e-capture   # Batch screenshot all demo pages
just e2e-verify    # Capture + verify event bridge + report
just e2e-install  # Install Playwright + chromium
```

## How to Run

### Prerequisites
```bash
# 1. Start dev server
just dev --daemon

# 2. Install Playwright (one-time)
just e2e-install
```

### Capture screenshots
```bash
just e2e-capture
# Output: target/e2e_screenshots/20260426_154000/{home,button,switch,...}.png
```

### Full verification (screenshots + event bridge)
```bash
just e2e-verify
# Output: target/e2e_screenshots/<timestamp>/report.md
```

### Run via npm (from web-test package)
```bash
cd packages/web-test
npm install && npx playwright install chromium
npx playwright test                    # all tests
npx playwright test tests/events.spec.ts  # event bridge only
npx playwright test tests/visual.spec.ts  # visual regression only
```

## Event System Tests (PLAN.md integration)

The `events.spec.ts` test file verifies the exact bug fixed in PLAN.md:

1. **WASM runtime initialized** — checks `__wasmExports` and `__listenerHandles` exist
2. **Click listener registered** — finds listener in `__listenerHandles` matching the button element
3. **`onMouseEvent` fires on click** — hooks WIT callback, clicks button, verifies dispatch
4. **Handler closure executes** — verifies the full DOM→JS→WIT→Rust chain completes
5. **Element handle integrity** — confirms button is reachable via `__elementHandles`
6. **Component interaction** — checks switch component has click listeners
7. **No console errors** — validates all key pages load without JS errors

## Architecture

```
tairitsu/
├── packages/
│   └── web-test/              # NEW: Playwright E2E test suite
│       ├── package.json
│       ├── playwright.config.ts
│       ├── page-registry.ts    # 13 pages with interaction specs
│       └── tests/
│           ├── visual.spec.ts  # Screenshot capture tests
│           └── events.spec.ts  # Event bridge verification tests
├── scripts/
│   ├── e2e-capture.ps1        # NEW: Batch screenshot script
│   └── e2e-verify.ps1         # NEW: Capture + verify + report
├── target/
│   └── e2e_screenshots/       # Output directory (gitignored)
│       ├── <YYYYMMDD_HHMMss>/ # Timestamped run output
│       ├── baseline/          # Golden images (committed manually)
│       └── report/            # Playwright HTML reporter output
└── justfile                   # Added: e2e-capture, e2e-verify, e2e-install recipes
```

## Remaining Work (Phase 2+)

### Phase 2: Visual Diffing (Future)
- [ ] Pixel comparison against baseline (`imageMagick compare` or `rust-image`)
- [ ] Tolerance threshold (< 1% pixel diff = pass)
- [ ] HTML report with side-by-side slider view
- [ ] CI gate: fail PR if any component exceeds threshold

### Phase 3: CI Integration (Future)
- [ ] GitHub Actions workflow running `just e2e-verify`
- [ ] Automatic baseline updates on main branch
- [ ] PR comment with diff report links

## Acceptance Criteria (Phase 1)

- [x] Running `scripts/e2e-capture.ps1` produces screenshots for all 13 demo pages
- [x] Screenshots land in `target/e2e_screenshots/<timestamp>/`
- [x] Console errors captured per-page
- [x] Summary shows pass/fail count
- [x] Interactive states captured (hover, click)
- [x] Works without MCP tools (uses Playwright directly)
- [x] Event bridge verified independently of opencode's MCP
