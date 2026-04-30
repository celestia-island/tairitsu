# PLAN2: Playwright-Based Visual Regression & Batch Style Testing

## ✅ Phase 1 Complete (archived)

Phase 1 delivered a self-contained `packages/web-test/` Playwright suite (13 pages,
event bridge verification, batch screenshot scripts, justfile recipes).
See git history for details.

---

## Remaining Work

### Phase 2: Embedded Debug API Server (`--debug` flag) — In Progress

**Goal:** `just dev --daemon --debug` starts the dev server *plus* an inspection API
on `/__tairitsu_debug/*`. Agents connect via HTTP to take screenshots, query DOM,
simulate clicks/inputs — no separate browser process needed; the server drives a
headless browser internally.

- [x] Add `--debug` / `--debug-port` CLI flags to `Dev` command
- [x] Create `packages/packager/src/debug/mod.rs` — debug route handlers
- [x] Implement debug endpoints (stub/skeleton — browser integration pending):
  - [x] `GET  /health`          — liveness + version
  - [x] `GET  /info`            — server state (port, pid, dist dir, uptime)
  - [x] `POST /navigate`         — URL resolution (browser nav pending CDP)
  - [x] `POST /screenshot`       — stub (returns 503 until browser connected)
  - [x] `POST /click`            — stub (returns 503 until browser connected)
  - [x] `POST /type`             — stub (returns 503 until browser connected)
  - [x] `POST /evaluate`         — stub (returns 503 until browser connected)
  - [x] `GET  /console`          — in-memory log buffer
  - [x] `GET  /dom`              — stub (returns 503 until browser connected)
- [x] Wire debug router into `dev_server()` Axum app (spawned alongside main server)
- [x] Create `docs/en/skills/debug-agent.md` — skill prompt for agent integration (protocol spec)
- [x] Add justfile recipe: `just dev-debug` → `just dev --daemon --debug`

**Next step:** Wire headless Chromium via CDP so screenshot/click/type/evaluate/dom
endpoints perform real browser automation instead of returning 503 stubs.
Dependencies to evaluate: `chromiumoxide` or raw CDP over HTTP via `reqwest`.

### Phase 3: Visual Diffing (Future)
- [ ] Pixel comparison against baseline (`imageMagick compare` or `rust-image`)
- [ ] Tolerance threshold (< 1% pixel diff = pass)
- [ ] HTML report with side-by-side slider view
- [ ] CI gate: fail PR if any component exceeds threshold

### Phase 4: CI Integration (Future)
- [ ] GitHub Actions workflow running `just e2e-verify`
- [ ] Automatic baseline updates on main branch
- [ ] PR comment with diff report links
