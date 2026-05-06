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

## 6. VTty 调试设施改进（与 entelecheia 联调发现）

### 背景

使用 vtty 对 entelecheia 的 health daemon 进行端到端联调时，发现以下问题。

### 联调实测 2026-05-06

尝试通过 vtty_launch 启动 `just dev --clean` 验证 entelecheia 时间轴修复。
结果：**vtty_launch 全部超时（MCP error -32001: Request timed out）**。

根因分析：
1. `vtty_launch` 内部等待进程启动并输出初始帧，但 `just dev --clean` 需要先编译（~90s）再启动 TUI
2. 编译期间 PTY 无输出，导致 vtty_launch 在等待首个 screen frame 时超时
3. 即使编译完成，entelecheia-tui 需要真实 TTY（`crossterm::terminal::enable_raw_mode` 失败于非 PTY 环境）
4. vtty 提供的是 pseudo-TTY，但 MCP transport 层的 request timeout（~65s）先于 TUI 启动触发

这证实了 V5 的影响范围比预期更大：不仅是 `vtty_wait`，连 `vtty_launch` 对长时间启动命令也会超时。

### 问题清单

| # | 文件 | 问题 | 影响 |
|---|------|------|------|
| V1 | `vtty/pty_unix.rs:72-90` | `read_nonblocking()` 每次调用 `libc::dup()` 复制 fd，读完后 dup 的 fd 随 `File` drop 关闭。高频调用下产生大量 fd 开销，且 `dup` + `fcntl(O_NONBLOCK` 每次都是 syscall | 性能损耗、潜在 fd 泄漏 |
| V2 | `vtty/mod.rs:111-140` | `read_and_update()` 只在被调用时（screenshot/wait/ping）才从 PTY 读取。无后台 reader 线程持续消费。如果进程持续高速输出，内核 PTY buffer（默认 4096 字节）可能溢出，导致输出丢失或进程 write 阻塞 | 长时间运行会话可能丢失输出 |
| V3 | `vtty/mod.rs:142-144` | `screenshot()` 先调 `read_and_update()` 再返回屏幕文本，但没有区分「本次是否读到了新数据」。MCP 工具侧无法判断是否需要重试 | 调用方无法感知增量更新 |
| V4 | `vtty/screen.rs` | `Vt100Screen` 使用固定行列 grid，没有 scrollback buffer。超出行高的内容直接丢失，无法回看 | 无法查看被滚出屏幕的历史输出 |
| V5 | MCP tool handler | `vtty_wait` 用 `tokio::time::sleep` 实现超时，长时间 wait（>60s）会触发 MCP transport timeout，返回 `-32001 Request timed out`。此时 session 仍然 alive，后续 screenshot 可恢复，但用户看不到中间产出 | 联调时 65s wait 必定超时 |
| V6 | MCP tool handler `vtty_launch` | `vtty_launch` 等待进程的首个 screen frame 输出。当命令启动慢（如 `just dev --clean` 需 90s 编译）时，MCP request timeout 先触发，导致 launch 本身返回 `-32001`。更严重的是：TUI 程序（crossterm/ratatui）调用 `enable_raw_mode()` 需要真实 TTY，vtty 的 pseudo-TTY 也会导致 TUI 启动失败 | 无法通过 vtty 启动任何需要编译的 TUI 程序 |
| V7 | MCP transport 重连 | opencode 通过 `systemctl --user restart opencode-serve.service` 重启后，tairitsu-mcp 子进程会被 systemd 杀死并由 opencode 重新 spawn。但实测发现：新 spawn 的 tairitsu-mcp 进程（PID 变化确认已重建）在 opencode 恢复后仍然返回 `Not connected`。所有 `vtty_*` 工具调用均失败。可能原因：(1) 新进程的 stdin/stdout pipe 未正确绑定到 opencode 的 JSON-RPC transport；(2) opencode 对已崩溃 MCP server 的 reconnect 逻辑有 race condition——新进程还没完成初始化就被标记为 connected，后续请求发到了死连接上；(3) systemd 服务重启后 opencode 内部缓存的 MCP server handle 未失效 | 重启 opencode 后 tairitsu MCP 工具不可用，必须完全退出并重新启动 opencode |
| V8 | MCP transport 连接不稳定 | **第二次实测（19:32）**：opencode restart 后 tairitsu-mcp 进程全部重建（PID 确认变化），`vtty_launch` 成功返回 session_id 和 pid，但后续 `vtty_send_text` / `vtty_screenshot` / `vtty_list` 全部返回 `Not connected`。首个 tool call 成功但第二个即断。怀疑：tairitsu-mcp 的 session state 是内存级的（HashMap），进程重建后所有 session 丢失。但问题更深——不是 session 找不到而是整个 MCP transport pipe 断了。可能是 tairitsu-mcp 收到第一个请求后 panic/crash，或者 opencode 的 MCP client 在收到第一个 response 后关闭了 pipe。建议在 tairitsu-mcp 入口加 `stderr` logging 确认进程是否存活。 | MCP 只能工作一次调用 |
| V9 | MCP routing 断开 | **第四次实测（20:02）**：通过 `dup2` 重定向 stderr 到 `/tmp/tairitsu-mcp/{pid}.log`，确认 5 个 tairitsu-mcp 进程全部成功完成 MCP 握手（initialize → tools/list → prompts/list），心跳持续。但**从未收到任何 `tools/call` 请求**——opencode 路由层未将 vtty/browser 工具调用分发到这些进程。根因定位于 **opencode MCP dispatch**，非 tairitsu-mcp 崩溃 | opencode 侧需修复 |
| V10 | `extract_master_fd` 返回 fd=0 | **第五次实测（20:20）**：reader_log 显示 `reader_eof total:0`，PTY 数据从未被后台 reader 捕获。通过 `pty_spawn` 诊断发现 `master_fd: 0`（stdin）而非 PTY fd。根因：`unsafe { (*file_ptr).as_raw_fd() }` 中 `*const dyn MasterPty as *const File` 的 transmute 对 portable-pty 0.8.x 的 concrete type 布局假设错误。**修复**：改用 `try_clone_reader()` → transmute to `File` → `as_raw_fd()` → `libc::dup(fd)` → drop reader。修复后 `vtty_launch` + `vtty_ready` + `vtty_screenshot` 全部正常工作 | ✅ 已修复 |

### 修复建议

#### Phase 1: PTY Reader 后台线程

```
pty_unix.rs:
  - 将 dup+fcntl 改为在 spawn 时一次性 dup 并设 O_NONBLOCK，缓存为 self.read_fd
  - read_nonblocking() 直接从 self.read_fd 读取，不再每次 dup

vtty/mod.rs:
  - launch() 时启动一个 reader thread：
    loop { read(master_fd) → lock(screen).process() }
  - read_and_update() 改为空操作或仅返回最近读取量
  - screenshot() 不再需要先读，直接返回 screen 快照
```

#### Phase 2: Scrollback Buffer

```
screen.rs:
  - 增加 scrollback: Vec<Vec<Cell>>，容量默认 1000 行
  - 当新行推入时，被滚出的行进入 scrollback
  - 新增 get_scrollback() 方法供 MCP 工具调用
  - 新增 MCP 工具 vtty_scrollback 供查看历史
```

#### Phase 3: Wait + Launch 优化

```
lib.rs / vtty_wait handler:
  - 内部改为轮询模式：每 200ms ping + check pattern，而非 sleep 全时长
  - 检测到 pattern 立即返回，减少无效等待
  - 同时设置合理的 MCP response timeout（如 120s），或在 wait 期间发送 progress notification

lib.rs / vtty_launch handler:
  - launch 应立即返回 session_id，不等待首个 frame
  - 新增 vtty_ready 工具：轮询直到 session 有 screen 输出
  - 这样 AI 调用方可以自己控制等待策略
  - 对于 crossterm TUI 兼容性问题：考虑设置 TERM=dumb 或添加 --no-tui flag 给 entelecheia
```

#### Phase 4: 结构化日志穿透

联调中发现 entelecheia daemon 使用 `JournalLayer` 将日志发送到 systemd journal，vtty 终端中只能看到 JournalLayer 的 raw 字段输出，可读性极差。

建议：
- `JournalLayer` 增加 `with_stdout_fallback(bool)` 方法：当检测到 stderr 不是 journal socket 时，自动启用 fmt layer
- 或在 MCP 工具层面新增 `vtty_journal` 工具，直接读取 systemd journal 并格式化输出

#### Phase 5: MCP Transport 重连可靠性

**实测记录：**
- 第一次（19:11 restart）：所有 vtty_* 返回 Not connected
- 第二次（19:32 restart）：vtty_launch 成功返回 session，但紧接的 send_text 断连
- 结论：tairitsu-mcp 进程能被 spawn 且能响应首个请求，但 transport pipe 在第一次请求后即断

排查方向：
```
1. 在 tairitsu-mcp 入口（lib.rs / mod.rs）添加 structured logging：
   - 启动时记录：PID, parent PID, stdin isatty, env vars
   - 每次 tool call 时记录：session lookup 结果
   - stdout write 时记录前 N bytes

2. 检查 opencode MCP client 侧：
   - 它如何管理子进程生命周期？
   - 是否有 graceful shutdown signal？
   - restart 后是否等待子进程 ready？
   - stdin/stdout pipe 是否在旧进程死后被 close，新进程启动后重新建立？

3. 可能的修复：
   - tairitsu-mcp 在启动时通过 stderr 发送 JSON-RPC notification 通知
     client "ready"（类似 LSP initialize），client 等待此通知再发送请求
   - 或者 opencode 侧：检测到 MCP server 退出后，标记为 disconnected，
     下次 tool call 时 transparent respawn
   - 最小修复：确认 tairitsu-mcp 首个请求后是否 panic——
     检查 stderr 输出：journalctl --user -u opencode-serve --since "19:32"
```

### 验收标准

- [ ] vtty session 持续运行 10 分钟不丢失输出
- [ ] 高频输出进程（如 `find /`）不导致 write 阻塞
- [ ] vtty_wait 60s+ 不触发 MCP timeout
- [ ] 可通过 MCP 工具查看 scrollback 历史输出
- [ ] vtty_launch 对长时间启动命令不超时（立即返回 session_id）
- [ ] entelecheia-tui 可通过 vtty 启动并正常渲染（或提供无 TTY 降级模式）
- [ ] opencode 重启后 tairitsu MCP 工具自动恢复可用（无需完全退出重启）

### 第三次联调实测（2026-05-06 深夜）

**背景：** entelecheia 侧时间轴修复已全部完成并合入 `dev` 分支（commit `eea6d3e35`），测试全绿（TUI 268/268，shared 7/7）。需要进行端到端 TUI 验证，确认以下修复在真实渲染中生效：
1. 重试标记去重（`groups.rs::build_groups`）
2. 错误消息国际化（8 种语言）
3. 模型名称格式 `提供商简称 #编号 > 模型名`（`render.rs::enrich_provider_label`）
4. 错误颜色统一为灰色斜体（`timeline_renderer/mod.rs::header_color`）

**测试过程：**

```
步骤 1: tairitsu_vtty_launch(command="bash", name="dev-shell")
结果:  返回 "Not connected"（非超时，立即失败）

步骤 2: 再次尝试 vtty_launch
结果:  同样 "Not connected"

步骤 3: 尝试 vtty_list（确认是否有残留 session）
结果:  同样 "Not connected"
```

**关键观察：**

1. **错误类型变化**：前两次联调（V7/V8）是 MCP error `-32001: Request timed out`，说明 tairitsu-mcp 进程存活但处理超时。这次是 **立即返回 "Not connected"**，无超时等待，说明：
   - tairitsu-mcp 进程可能根本没被 spawn，或者
   - opencode 的 MCP client 判断 transport pipe 已断，直接拒绝请求

2. **未做任何重启操作**：本次测试前没有 `systemctl restart opencode-serve`，opencode 进程是从上次联调一直运行的。V8 中记录的 transport 断连一直没恢复。

3. **不涉及 PTY/TUI 问题**：连基本的 `bash` shell 都无法启动，与 crossterm raw_mode 无关，是 MCP transport 层的问题。

**对 tairitsu 的建议（按优先级排序）：**

| 优先级 | 建议 | 详情 |
|--------|------|------|
| P0 | **加 stderr structured logging** | tairitsu-mcp 入口（`lib.rs` / `mod.rs`）每次 tool call 记录：收到请求、session lookup、响应发送。确认进程是否 panic 或被 kill。`tracing_subscriber::fmt().with_writer(std::io::stderr())` 即可 |
| P0 | **确认 opencode MCP client spawn 逻辑** | 查看 opencode 在什么条件下会重新 spawn MCP server，什么条件下标记为 "Not connected" 并拒绝请求。可能需要在 opencode 侧加日志 |
| P1 | **实现 Phase 3（launch 立即返回）** | `vtty_launch` 应立即返回 session_id，不等待首个 frame。新增 `vtty_ready` 轮询。这对长时间编译场景是必须的 |
| P1 | **实现 Phase 5（transport 重连）** | 当前状态：一旦 transport 断连，整个会话期间都无法恢复。需要 tairitsu-mcp 或 opencode 侧实现 reconnect |
| P2 | **实现 Phase 1（后台 reader 线程）** | 对短时间测试影响不大，但对长时间会话是必须的 |

**entelecheia 侧准备状态：**

- [x] 代码已提交 `dev` 分支，可直接 `just dev --clean` 编译运行
- [x] 所有单元测试通过，无需额外修改
- [ ] 等待 vtty MCP 恢复后进行 TUI 端到端验证
- [ ] 如 vtty 持续不可用，可考虑手动启动 `just dev` 截图验证

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
