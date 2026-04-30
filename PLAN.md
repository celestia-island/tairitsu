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

### Phase 3: Visual Diffing (Future)
- [ ] Pixel comparison against baseline (`imageMagick compare` or `rust-image`)
- [ ] Tolerance threshold (< 1% pixel diff = pass)
- [ ] HTML report with side-by-side slider view
- [ ] CI gate: fail PR if any component exceeds threshold

### Phase 4: CI Integration (Future)
- [ ] GitHub Actions workflow running `just e2e-verify`
- [ ] Automatic baseline updates on main branch
- [ ] PR comment with diff report links
