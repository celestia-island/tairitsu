# PLAN — Remaining Work

---

## 1–3. ✅ DONE (VTty, Browser MCP, Daemon Discovery)

See git history for details.

---

## v0.4.0 Release Status ✅

All 10 packages prepared for crates.io publishing:

- tairitsu-macros, tairitsu-browser-worlds, tairitsu-vdom, tairitsu (runtime)
- tairitsu-style, tairitsu-hooks, tairitsu-browser-wit-resolver
- tairitsu-ssr, tairitsu-web, tairitsu-packager

### Completed for v0.4.0

- [x] Synchronized versions to 0.4.0 across all packages
- [x] `wry`/`tao` made optional in packager (behind `debug-browser` feature)
- [x] `debug_api` module gated behind `debug-browser` feature
- [x] hikari deps use git URL (not local path) for CI
- [x] `examples/website` removed from workspace members (hikari pulls unpublished deps)
- [x] CI verify-versions simplified to grep-based (no cargo needed)
- [x] All clippy warnings resolved (`cargo clippy --all-targets --all-features -- -D warnings` passes)
- [x] All fmt issues resolved (`cargo +nightly fmt --all -- --check` passes)
- [x] `release.toml` configured for publish order

---

## 5. MCP 独立工具模式：脱离项目清单和 daemon 运行 🔄

### 问题陈述

tairitsu MCP 当前架构要求 `tairitsu dev --daemon` 已运行，且必须在含 `[package]` 的 `Cargo.toml` 项目目录下启动。但实际使用场景：

1. **VTty 纯终端工具** — 只需 forkpty，不依赖任何 web 浏览器或 daemon
2. **MCP 被 AI 编码助手调用** — 在任意 CWD 下通过 stdio JSON-RPC 工作，无权选择工作目录
3. **CLI 一次性命令** — `tairitsu mcp` 应可在任何路径直接启动，开箱即用

### 根因分析

| # | 文件 | 行号 | 问题 |
|---|------|------|------|
| R1 | `mcp/mod.rs` | 36-51 | `run()` 调用 `resolve_daemon_url()`，失败时 vtty 分支只是 warning 并设 `base_url=""`，所有 `browser_*` 工具被 `require_daemon()` 拒绝 |
| R2 | `mcp/mod.rs` | 66-98 | `resolve_daemon_url()` 在找不到 ready file 时直接返回 Err |
| R3 | `daemon/mod.rs` | 19-24 | `project_root()` 依赖 `PROJECT_ROOT` OnceLock，fallback 到 CWD，但 CWD 可能不是 tairitsu 项目 |
| R4 | `daemon/mod.rs` | 782-845 | `fork_daemon()` 需要项目 config 的 `[package]` section |
| R5 | `mcp/mod.rs` | 154-161 | `require_daemon()` 对所有 `browser_*` 工具统一拦截 |

### 设计目标

1. **MCP 无项目启动** — `tairitsu mcp` 在任何目录下启动，vtty 工具立即可用
2. **Daemon 按需启动** — 首次调用 `browser_*` 工具时自动启动 daemon
3. **CLI vtty 子命令** — `tairitsu vtty <command>` 直接启动 VTty 会话
4. **连接恢复** — MCP 重启后能 reattach 已存在的 daemon

### 修复方案

#### Phase 1: MCP 启动解耦

`mcp/mod.rs::run()` — 三层 fallback: env → ready file → auto_launch → ""

#### Phase 2: Daemon 无项目模式

`daemon/mod.rs` — 新增 `daemon_home_dir()`，`config::load_config()` 无项目时使用默认配置

#### Phase 3: CLI vtty 子命令

`cli/mod.rs` — 新增 `Vtty` 命令

#### Phase 4: MCP 工具降级策略

`require_daemon()` → `ensure_daemon()`，首次调用时按需启动

### 验收标准

- [ ] 在 `/tmp` 目录下运行 `tairitsu mcp`，vtty 工具立即可用
- [ ] 通过 MCP 调用 `browser_navigate` 时自动启动 daemon
- [ ] `tairitsu vtty "echo hello"` 直接输出终端内容并退出
- [ ] MCP 重启后能 reattach 已有 daemon

---

## 4. Website: Pending Enhancements

### P1 — Enhancement

| # | Item | Status |
|---|------|--------|
| 15 | **Dynamic markdown rendering** — pulldown-cmark → VNode | ✅ Done |
| 17 | **Sidebar item icons** — SVG icons per menu item | ✅ Done |

### P2 — Polish

| # | Item | Details | Status |
|---|------|---------|--------|
| 19 | **state_test.rs stub handlers** | oninput TODO, dead remove buttons | 🔄 |
| 20 | **Logo is unicode char** | `\u{273F}` instead of actual image | 🔄 |
| 21 | **No favicon.ico verified** | Referenced in Cargo.toml | 🔄 |
| 22 | **Keyboard navigation** | Escape to close drawer | ✅ Done |

### P3 — Infrastructure

| # | Gap |
|---|-----|
| 23 | Dynamic doc loading missing (all content compiled into WASM) |
| 24 | i18n.rs not wired to all content pages |
| 25 | No keyboard navigation (arrow keys for menu) |
| 26 | No search functionality |
