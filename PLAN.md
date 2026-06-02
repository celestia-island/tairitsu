# Tairitsu Browser MCP — 故障排查与改进计划

## 当前问题：浏览器工具完全不可用

### 症状

所有 tairitsu MCP 浏览器工具均返回相同错误：

```
MCP error -32603: daemon returned 404 Not Found:
The server is configured with a public base URL of /static/arona/
- did you mean to visit /static/arona/navigate instead?
```

受影响的工具：
- `tairitsu_browser_navigate`
- `tairitsu_browser_snapshot`
- `tairitsu_browser_screenshot`
- `tairitsu_browser_click`
- `tairitsu_browser_type`
- `tairitsu_browser_evaluate`
- `tairitsu_browser_console_messages`
- `tairitsu_browser_press_key`
- `tairitsu_browser_resize`

### 根因分析

#### 1. 守护进程已死亡，ready 文件残留

tairitsu packager 守护进程实际未运行（PID 文件指向不存在的进程），但
`target/tairitsu-packager.ready` 文件仍然存在：

```
$ cat target/tairitsu-packager.ready
ready:3000:3001
```

含义：
- `dev_port = 3000` → tairitsu packager 开发服务器（已宕机，连接拒绝）
- `debug_port = 3001` → debug API 服务器

#### 2. 端口被 Vite 开发服务器占用

端口 3001 被 arona 的 Vite 开发服务器（`pnpm run dev`）占用，而非
tairitsu debug API。当 MCP 发送请求时：

```
MCP → POST http://localhost:3001/navigate
→ 命中的是 Vite 开发服务器，不是 tairitsu debug API
→ Vite 不知道 /navigate 路由
→ 返回 404 + 纯文本错误消息
```

Vite 的 404 响应（`Content-Type: text/plain`）：

```
The server is configured with a public base URL of /static/arona/
- did you mean to visit /static/arona/navigate instead?
```

#### 3. MCP 守护进程 URL 解析没有健康检查

`packages/mcp/src/lib.rs:875-997` 中的 `resolve_daemon_url()` 逻辑：

```
1. 检查 TAIRITSU_DAEMON_URL 环境变量
2. 搜索 target/tairitsu-packager.ready 文件
3. 解析 ready 文件获取 debug_port
4. 返回 http://localhost:<debug_port>
```

**关键缺陷**：找到 ready 文件后直接返回 URL，**不做任何健康检查**。
不会验证 debug API 是否真正可达。

#### 4. 守护进程退出时不会清理 ready 文件

`packages/packager/src/daemon/mod.rs` 中的守护进程启动流程写入了
`target/tairitsu-packager.ready`，但没有注册退出时的清理钩子。
守护进程被 kill 或崩溃后，残留的 ready 文件持续误导 MCP。

### 连接路径图

```
正确的路径（设计意图）：
  opencode → tairitsu-mcp (stdio)
    → 解析 daemon URL（从 ready 文件）
    → POST http://localhost:<debug_port>/navigate
    → tairitsu debug API server (axum)
    → chromiumoxide CDP commands
    → Chromium 浏览器

实际发生的路径（当前环境）：
  opencode → tairitsu-mcp (stdio)
    → 读取 ready 文件 → "ready:3000:3001"
    → POST http://localhost:3001/navigate
    → 命中 Vite 开发服务器（不是 debug API）
    → 404 + HTML 错误页
    → MCP 解析失败，返回错误给 opencode
```

#### 5. ready 文件散落在多个项目中

MCP 的 fallback 搜索逻辑会扫描 `/mnt/sdb1/*` 下所有目录的
`target/tairitsu-packager.ready`，发现以下残留文件：

| 路径 | 内容 | 状态 |
|------|------|------|
| `/mnt/sdb1/tairitsu/target/tairitsu-packager.ready` | (已删除) | 过期 |
| `/mnt/sdb1/tairitsu/examples/website/target/tairitsu-packager.ready` | `ready:3000:3001` | **这是导致 MCP 连接错误 3001 的根因** |
| `/mnt/sdb1/shittim-chest/target/tairitsu-packager.ready` | Cargo.toml 解析错误写入 | 过期 |
| `/mnt/sdb1/hikari/examples/website/target/tairitsu-packager.ready` | 编译错误写入 | 过期 |

所有残留文件已清除。

---

## 改进计划：自主检查与修复浏览器依赖

### P0: MCP 守护进程健康检查

**位置**：`packages/mcp/src/lib.rs:resolve_daemon_url()`

在返回 URL 之前验证 debug API 可达：

```rust
async fn resolve_daemon_url() -> Result<String> {
    // ... 现有的 ready 文件查找逻辑 ...

    let url = format!("http://localhost:{debug_port}");

    // 新增：健康检查
    let client = reqwest::Client::new();
    match client.get(format!("{url}/health")).timeout(Duration::from_secs(2)).send().await {
        Ok(resp) if resp.status().is_success() => Ok(url),
        _ => {
            // ready 文件过期，尝试清理
            let _ = std::fs::remove_file(&ready_path);
            Err(anyhow!("Daemon not responding at {url}. Stale ready file removed."))
        }
    }
}
```

### P1: 守护进程退出时清理 ready 文件

**位置**：`packages/packager/src/daemon/mod.rs`

在守护进程启动时注册信号处理：

```rust
fn daemonize_self() {
    // ... 现有的 daemonize 逻辑 ...

    let ready_path = ready_path.clone();
    ctrlc::set_handler(move || {
        let _ = std::fs::remove_file(&ready_path);
        std::process::exit(0);
    });
}
```

同时在启动时检查端口是否已被占用，避免写入指向其他进程的 ready 文件：

```rust
// 写入 ready 文件前验证端口确实是自己的
if !is_port_owned(dev_port) || !is_port_owned(debug_port) {
    return Err("Port already in use by another process");
}
```

### P2: 浏览器依赖自主检查与安装

**位置**：`packages/mcp/src/lib.rs` 或新建 `packages/mcp/src/browser_check.rs`

#### 2a. 浏览器二进制检查

```rust
async fn ensure_browser_available() -> Result<PathBuf> {
    // 优先级：
    // 1. CHROME_PATH 环境变量
    // 2. ~/.cache/tairitsu/browsers/chromium/<version>/chrome
    // 3. 系统 PATH 中的 chromium/chrome

    if let Ok(path) = std::env::var("CHROME_PATH") {
        if PathBuf::from(&path).exists() {
            return Ok(PathBuf::from(path));
        }
    }

    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from("~/.cache"))
        .join("tairitsu/browsers/chromium");

    // 检查缓存版本
    if let Some(cached) = find_cached_browser(&cache_dir) {
        return Ok(cached);
    }

    // 系统PATH
    for name in &["chromium-browser", "chromium", "google-chrome", "chrome"] {
        if let Ok(output) = Command::new("which").arg(name).output() {
            if output.status.success() {
                return Ok(PathBuf::from(String::from_utf8_lossy(&output.stdout).trim()));
            }
        }
    }

    // 自动下载
    eprintln!("No browser found. Auto-installing Chromium...");
    download_chromium(&cache_dir).await
}
```

#### 2b. 自主安装功能

复用 `packages/browser-test/src/browser/downloader.rs` 的逻辑：

```rust
async fn download_chromium(cache_dir: &Path) -> Result<PathBuf> {
    let version = "146.0.7680.153"; // 与 browser-test 保持一致
    let platform = detect_platform()?;
    let url = format!(
        "https://storage.googleapis.com/chrome-for-testing-public/{version}/{platform}/chrome-{platform}.zip"
    );
    // 支持镜像：TAIRITSU_BROWSER_MIRROR 环境变量

    let zip_path = cache_dir.join(format!("{version}/{platform}/chrome.zip"));
    download(&url, &zip_path).await?;
    extract_zip(&zip_path, cache_dir)?;

    let binary = cache_dir.join(format!("{version}/{platform}/chrome-{platform}/chrome"));
    assert!(binary.exists(), "Extraction failed");
    Ok(binary)
}
```

### P3: MCP 启动时自检

**位置**：`packages/mcp/src/main.rs`

MCP 服务器启动时执行完整自检：

```rust
async fn self_check() -> Vec<CheckResult> {
    let mut results = Vec::new();

    // 1. 守护进程可达性
    match resolve_daemon_url().await {
        Ok(url) => results.push(CheckResult::ok("daemon", url)),
        Err(e) => results.push(CheckResult::fail("daemon", e.to_string())),
    }

    // 2. Debug API 健康检查
    // 3. 浏览器二进制可用性
    // 4. 端口冲突检测（ready 文件中的端口是否被其他进程占用）

    results
}
```

暴露为 MCP tool：

```rust
#[tool(description = "Check browser dependency status and auto-repair")]
async fn browser_doctor() -> Result<CallToolResult, McpError> {
    let checks = self_check().await;
    let report = format_check_report(&checks);
    Ok(Self::tool_result(report))
}
```

### P4: ready 文件校验增强

在 `try_read_ready_port_from_candidates()` 中增加校验：

```rust
fn try_read_ready_port_from_candidates(dirs: &[PathBuf]) -> Option<(u16, Option<u16>, PathBuf)> {
    for dir in dirs {
        let ready_path = dir.join("tairitsu-packager.ready");
        if let Ok(content) = std::fs::read_to_string(&ready_path) {
            // 新增：检查文件修改时间，超过 1 天视为过期
            if let Ok(metadata) = std::fs::metadata(&ready_path) {
                if let Ok(modified) = metadata.modified() {
                    if modified.elapsed().unwrap_or_default() > Duration::from_secs(86400) {
                        let _ = std::fs::remove_file(&ready_path);
                        continue;
                    }
                }
            }

            // 新增：检查 PID 文件一致性
            let pid_path = dir.join("tairitsu-packager.pid");
            if let Ok(pid_str) = std::fs::read_to_string(&pid_path) {
                let pid: u32 = pid_str.trim().parse().ok()?;
                if !is_process_running(pid) {
                    let _ = std::fs::remove_file(&ready_path);
                    let _ = std::fs::remove_file(&pid_path);
                    continue;
                }
            }

            // ... 现有的端口解析逻辑 ...
        }
    }
}
```

---

## 实现优先级

| 优先级 | 项目 | 工作量 | 影响 |
|--------|------|--------|------|
| P0 | MCP 健康检查 | 30 min | 消除误连接 Vite 的核心问题 |
| P1 | 退出清理 ready 文件 | 20 min | 防止残留文件污染 |
| P2a | 浏览器二进制检查 | 1 hr | 基础依赖保障 |
| P2b | 自主安装 Chromium | 2 hr | 零配置体验 |
| P3 | MCP 启动自检 + doctor tool | 1 hr | 可观测性 |
| P4 | ready 文件校验增强 | 30 min | 防御性编程 |

## 当前环境状态

```
$ cat target/tairitsu-packager.ready
ready:3000:3001

$ cat target/tairitsu-packager.pid
(PID 已不存在)

$ ss -tlnp | grep -E '3000|3001'
3000: 无进程监听
3001: arona vite dev server (PID 3812559)

$ which chromium
/home/lab/.local/bin/chromium (系统有 chromium)

$ ls ~/.cache/tairitsu/browsers/
(无 tairitsu 缓存)

$ curl http://localhost:3001/navigate
404: The server is configured with a public base URL of /static/arona/
```

**立即修复**：已清除所有残留 ready 文件（6 个位置）。MCP 进程内存中缓存了旧 URL，
需重启 opencode（或重启 tairitsu-mcp 进程）才能生效。清除后的预期错误消息为：

```
No running tairitsu daemon found
```

而非混淆的 `/static/arona/` 消息。
