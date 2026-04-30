# PLAN2: Playwright-Based Visual Regression & Batch Style Testing

## ✅ Phase 1 Complete (archived)

Phase 1 delivered a self-contained `packages/web-test/` Playwright suite (13 pages,
event bridge verification, batch screenshot scripts, justfile recipes).
See git history for details.

---

## Remaining Work

### ✅ Phase 2 Complete (archived)

Phase 2 delivered a full embedded debug API server (`--debug` flag) with
CDP-based Chromium automation engine. All 9 endpoints implemented and tested.
See commits `723a8b5`, `446261a`, `cc319d3`.

### ✅ Phase 3 Complete (archived)

Phase 3 delivered a pixel-level visual diffing engine using `image` crate v0.25:
- `packages/packager/src/visual_diff/mod.rs` — core engine (compare, diff image, HTML report)
- `VisualDiff` CLI subcommand (`tairitsu visual-diff`) with tolerance, baseline management
- HTML report with side-by-side diff images, JSON report output
- justfile recipes: `visual-capture`, `visual-diff`, `visual-update`, `visual-regression`

### ✅ Phase 4 Complete (archived)

Phase 4 delivered CI integration for visual regression:
- `.github/workflows/visual-regression.yml` — full pipeline:
  - Build with debug-browser + visual-diff features
  - Start dev server + debug API
  - Capture screenshots via debug endpoint
  - Run visual diff comparison
  - Upload artifacts (HTML report + diff images)
  - Auto baseline update on main branch push
  - PR comment with results table
  - Fail CI if any screenshot exceeds tolerance threshold

### Remaining Work

None — all phases complete. Future enhancements may include:
- [ ] SSIM/structural similarity metric alongside pixel diff ratio
- [ ] Per-component threshold configuration
- [ ] Interactive web UI for baseline review/approval
