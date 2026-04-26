# PLAN2: Playwright-Based Visual Regression & Batch Style Testing

## Status: 🟡 Planning — Infrastructure to Build

## Goal

Establish a **semi-permanent Playwright testing infrastructure** that can:
1. **Batch-screenshot** every component demo page after each build
2. **Write screenshots** into `target/e2e_screenshots/` (already gitignored)
3. **Diff against baselines** to catch visual regressions
4. **Verify interactive states** (hover, focus, active, open/closed) via Playwright's snapshot + evaluate APIs

## Why This Belongs in tairitsu (not hikari)

- hikari is a **consumer** of tairitsu-vdom / tairitsu-web — it exercises the platform
- The **event system bug** (PLAN.md) lives in tairitsu's vdom patch layer
- A reusable test harness benefits **all** tairitsu-based projects, not just hikari
- Screenshot output goes into `target/` (build artifact), keeping source trees clean

## Architecture

```
tairitsu/
├── packages/
│   └── vdom/
│       └── tests/                    # NEW: E2E visual test suite
│           ├── e2e_mod.rs            #   test entry point (#[cfg(test)])
│           ├── screenshot_runner.rs  #   Playwright orchestration logic
│           ├── page_registry.rs      #   URL list + metadata per component page
│           └── expectations/         #   baseline reference images (committed)
│               ├── button_default.png
│               ├── feedback_alerts.png
│               ├── avatar_sizes.png
│               └── ...
├── scripts/
│   └── e2e-playwright.sh             # NEW: launch dev server → run tests → collect
└── target/
    └── e2e_screenshots/              # NEW: actual output (gitignored, per .gitignore line 6)
        ├── 2026-04-26_T123456/
        │   ├── button.png
        │   ├── feedback.png
        │   └── ...
        └── latest/                   # symlink or copy of most recent run
```

## Phase 1: Screenshot Capture (MVP)

### 1.1 Page Registry

Define a static list of all pages to screenshot:

```rust
// packages/vdom/tests/page_registry.rs
pub struct PageSpec {
    pub url: String,
    pub name: String,          // e.g. "button", "feedback"
    pub category: &'static str, // e.g. "layer1", "layer2"
    pub selector: Option<String>, // specific element to crop (None = full viewport)
    pub interactions: Vec<InteractionSpec>,
}

pub struct InteractionSpec {
    pub action: InteractionAction,
    pub suffix: String,         // appended to filename, e.g. "_open", "_hover"
}

pub enum InteractionAction {
    Click { ref_selector: String },
    Hover { ref_selector: String },
    Focus { ref_selector: String },
}
```

Initial registry (hikari demo pages):

| URL | Name | Interactions |
|-----|------|-------------|
| `/` | home | — |
| `/components/layer1/button` | button | hover on primary |
| `/components/layer1/form` | form | — |
| `/components/layer1/search` | search | — |
| `/components/layer1/switch` | switch | click first switch |
| `/components/layer1/feedback` | feedback | — |
| `/components/layer1/display` | display | — |
| `/components/layer1/avatar` | avatar | — |
| `/components/layer1/image` | image | — |
| `/components/layer1/tag` | tag | — |
| `/components/layer1/empty` | empty | — |
| `/components/layer1/comment` | comment | — |
| `/components/layer1/description-list` | description_list | — |

### 1.2 Screenshot Runner

Two modes of operation:

**Mode A: External Playwright MCP (current setup)**
- Uses `@playwright/mcp@latest --browser msedge` from opencode.jsonc
- Called interactively during development sessions
- Screenshots written to `target/e2e_screenshots/adhoc/`
- No Rust code needed — just a script that calls the MCP tools sequentially

**Mode B: Internal Rust test with playwright-rust (future)**
- `cargo test --package tairitsu-vdom --test e2e_visual`
- Uses `playwright` crate directly, no MCP dependency
- Runs in CI, produces diff reports
- More complex setup but fully automated

**Recommendation:** Start with Mode A (we have it working now), migrate to B when stable.

### 1.3 Ad-Hoc Script (Mode A)

A shell/PowerShell script that:
1. Ensures dev server is running (`just dev --daemon`)
2. Reads page registry
3. For each page: navigate → wait for DOM → screenshot → save to `target/e2e_screenshots/<timestamp>/<name>.png`
4. For pages with interactions: perform interaction → screenshot → save as `<name>_<suffix>.png`
5. Print summary table: PASS/FAIL per page (FAIL = console errors detected)

```powershell
# scripts/e2e-capture.ps1 (pseudo)
$base = "http://localhost:3000"
$out = "target/e2e_screenshots/$(Get-Date -Format 'yyyyMMdd_HHmmss')"
New-Item -ItemType Directory -Path $out -Force

$pages = @(
    @{url="/"; name="home"},
    @{url="/components/layer1/button"; name="button"},
    # ... full registry
)

foreach ($p in $pages) {
    # Navigate via Playwright MCP
    # Take screenshot → "$out/$($p.name).png"
    # Check console errors → log to "$out/$($p.name).errors.log"
}
```

### 1.4 Output Location Convention

```
target/e2e_screenshots/
├── <YYYYMMDD_HHMMss>/          # timestamped run
│   ├── home.png                # full-page or viewport
│   ├── button.png
│   ├── button_hover_primary.png
│   ├── switch.png
│   ├── switch_after_click.png
│   ├── feedback.png
│   ├── avatar.png
│   ├── tag.png
│   ├── tag_closable.png
│   ├── empty.png
│   └── ...                     # one file per (page × state)
├── baseline/                   # manually curated golden images
│   ├── home.png
│   ├── button.png
│   └── ...
├── diff/                       # auto-generated pixel diffs (Phase 2)
│   ├── button_diff.png
│   └── ...
└── report.html                 # HTML comparison report (Phase 2)
```

Key rules:
- All output under `target/` → already gitignored globally
- `baseline/` is the only folder whose contents may be committed (golden images)
- Timestamped runs are ephemeral, kept for N days by a cleanup policy

## Phase 2: Visual Diffing (Future)

Once we have stable baselines:

1. **Pixel comparison**: Compare new screenshot vs baseline using `imageMagick compare` or `rust-image`
2. **Tolerance threshold**: Ignore anti-aliasing differences (< 1% pixel diff = pass)
3. **HTML report**: Side-by-side slider view (before/after) + highlighted diff regions
4. **CI gate**: Fail PR if any component exceeds threshold

## Phase 3: Event System Validation (ties into PLAN.md)

Use Playwright to **automatically verify** the event system fix:

```javascript
// Test spec for language selector dropdown
await page.goto('http://localhost:3000/components/layer1/switch');
const trigger = await page.locator('.hi-select-trigger');
await trigger.click();
await page.screenshot({ path: 'target/e2e_screenshots/latest/lang_dropdown_open.png' });

const dropdown = await page.locator('.hi-select-dropdown');
const display = await dropdown.evaluate(el => getComputedStyle(el).display);
assert(display === 'block', 'Dropdown should be visible after click');
```

```javascript
// Test spec for dark mode toggle
await page.goto('http://localhost:3000');
await page.getByRole('switch', { name: 'Toggle dark mode' }).click();
const layout = await page.locator('.hi-layout');
const theme = await layout.getAttribute('data-theme');
assert(theme === 'tairitsu', 'Theme should switch to dark');
await page.screenshot({ path: 'target/e2e_screenshots/latest/dark_mode.png' });
```

These specs become permanent regression guards — once PLAN.md's event bug is fixed,
these tests confirm it stays fixed.

## Files to Create

| File | Purpose | Priority |
|------|---------|----------|
| `packages/vdom/tests/page_registry.rs` | Page URL + interaction definitions | P1 |
| `scripts/e2e-capture.ps1` | Batch screenshot capture via Playwright MCP | P1 |
| `scripts/e2e-verify.ps1` | Run capture + check console errors + summary | P1 |
| `target/e2e_screenshots/.gitkeep` | Ensure directory exists (gitignored) | P1 |
| `packages/vdom/tests/e2e_event.rs` | Event system regression tests (after PLAN.md fix) | P2 |
| `scripts/e2e-diff.ps1` | Pixel diff against baseline | P3 |

## Acceptance Criteria (Phase 1)

- [ ] Running `scripts/e2e-capture.ps1` produces screenshots for all 12+ demo pages
- [ ] Screenshots land in `target/e2e_screenshots/<timestamp>/`
- [ ] Console errors (if any) are captured per-page into `.log` files
- [ ] Summary shows pass/fail count and links to each screenshot
- [ ] Interactive states (dropdown open, dark mode, switch toggle) are captured when applicable
- [ ] Works with existing Playwright MCP setup (`--browser msedge`, no Chrome install needed)

## Dependencies

- ✅ Playwright MCP configured in opencode.jsonc (`--browser msedge`)
- ✅ Dev server builds successfully (`just dev --daemon`)
- ✅ `.playwright-mcp/` gitignored
- ⏳ `target/e2e_screenshots/` already gitignored (line 6 of hikari/.gitignore)
- ❌ PLAN.md event bug must be fixed before interaction screenshots work correctly
