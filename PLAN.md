# Tairitsu SSR 集成规划

> **目标**：使 tairitsu-ssr 能加载 hikari 网站 `.wasm`，调用组件入口，将 in-memory DOM 序列化为完整 HTML，并通过 E2E 验证对接正确。

---

## 当前状态

- `packages/ssr` crate 已完成核心实现，`cargo check` 通过
- 公开 API 已就绪：`render_to_html(wasm_bytes, config)` / `render_full_page(...)`
- 核心 WIT 接口已手动实现：`document`、`node`、`element`、`style`、`console`、`window`、`platform_helpers`、`event_target`
- 438 个非核心 WIT 接口由 `build.rs` 自动生成 stub（返回默认值或无操作）
- in-memory DOM（`SsrDom`）+ HTML 序列化（`html_render.rs`）已实现
- `call_lifecycle_start()` 通过 `[export-lifecycle]start` 导出名调用组件

**已知问题** (2026-03-24):
- `resize-observer-entry` 接口的 `get-content-rect` 方法存在类型不匹配错误
- 错误信息："expected `u64` found `record`"
- 详细分析见 `/mnt/sdb1/tairitsu/RESIZE_OBSERVER_ISSUE.md`
- 此问题阻止了 P0 任务的完成，需要深入研究 WIT 记录类型的处理方式

---

## 待完成任务

### P0 — 联调 hikari 网站 wasm（最高优先级）

**目标**：将 hikari 文档站编译产物 `website.wasm` 载入 `render_to_html()`，验证输出 HTML 正确。

**步骤**：

1. 在 tairitsu-ssr 的 `tests/` 目录添加集成测试 `test_hikari_website.rs`
2. 读取 `website.wasm`（由 `just build` 产出）
3. 调用 `render_to_html(&wasm_bytes, SsrConfig::default())`
4. 断言输出 HTML 包含：
   - `#hikari-app`
   - `.hi-layout`
   - `.hikari-page`
   - 至少一个 `<h1>` 或 `<h2>` 标题

**可能遇到的问题**：

- `lifecycle::start()` 导出名与 wasmtime Component Model 约定不一致 → 需排查实际导出名
- 组件依赖的 WIT 接口未在 stub 中注册 → 运行时 `linker error: unknown import`
- WASI 预览版本不匹配 → 检查 `wasmtime_wasi::p2` 与组件编译目标 `wasm32-wasip2`

**成功标准**：`cargo test -p tairitsu-ssr -- test_hikari_website` 通过。

---

### P1 — 修复已知联调问题

根据最近几次提交（`fix: 修复 tairitsu-ssr 中的多个类型映射问题`、`fix: 修复 SSR 中的 WIT 路径和重复 map entry 错误`），以下问题已部分修复，联调时需验证：

1. **WIT 类型映射**：`u64` handle ↔ WIT `u64` 的映射是否覆盖所有接口
2. **重复 map entry**：`register_all_auto_stubs` 中是否存在已手动实现的接口被再次注册
3. **接口路径**：`tairitsu-browser:full/xxx@0.2.0` 命名空间是否与组件导入声明完全一致

---

### P2 — 完善 HTML 渲染质量

**场景**：`render_body_html()` 当前只输出 `<body>` 子树。完整 SSR 页面需要：

- `<!DOCTYPE html>` 声明
- `<head>` 内的 `<title>`、`<meta charset>`、`<meta name="viewport">`、CSS `<link>` 标签
- 正确的 `<html lang="...">` 属性

**实现思路**：

1. 在 `html_render.rs` 中新增 `render_full_document_html(&self, config: &FullPageConfig) -> String`
2. `render_full_page()` 改为填充真实 `<head>` 内容，而非只做字符串替换 `<div id="app">`
3. CSS 链接列表由调用方（packager）传入 `SsrConfig`

---

### P3 — Packager 集成（`tairitsu dev --ssr` / `tairitsu build --ssr`）

现状：`packages/packager` 中已有 `--ssr` 参数骨架（来自 `feat(ssr): add SSR support to packager CLI`）。

**待完成**：

1. `tairitsu build --ssr` 流程：
   - 编译 wasm
   - 调用 `render_to_html()` 生成 HTML
   - 将生成 HTML 写入输出目录

2. `tairitsu dev --ssr` 流程（开发服务器）：
   - 监听文件变化，重新编译 wasm
   - 每次请求时调用 `render_to_html()` 或缓存上次结果
   - HTTP 响应直接返回 SSR HTML

---

### P4 — Hydration 支持（长期）

当客户端 wasm 加载后，需要接管 SSR 服务端已输出的 DOM 节点，而非重新创建。

**需要的机制**：

- `tairitsu-web` 中新增 `mount_to_existing_dom()` 入口
- 服务端渲染时在 DOM 节点上打 `data-hk-*` marker，客户端按 marker 复用节点
- hikari 组件代码无需感知 SSR/CSR 区别

---

## 公开 API 约定（供 hikari 侧参考）

```rust
use tairitsu_ssr::{render_to_html, render_full_page, SsrConfig};

// 基本用法：获取 <body> 内 HTML
let html = render_to_html(&wasm_bytes, SsrConfig::default())?;

// 完整页面：注入到 index.html 模板
let page = render_full_page(&wasm_bytes, SsrConfig::default(), template_html)?;

// 自定义 viewport（影响 window.innerWidth 等接口返回值）
let cfg = SsrConfig::new(1280, 720);
let html = render_to_html(&wasm_bytes, cfg)?;
```

---

## 任务优先级

| 优先级 | 任务 | 状态 |
|--------|-----|------|
| P0 | 联调 hikari website.wasm | 进行中 |
| P1 | 修复 WIT 类型映射 / 重复注册问题 | 已部分修复，需验证 |
| P2 | 完善 HTML 渲染（完整 document 模式） | 待实现 |
| P3 | Packager `--ssr` 集成 | 骨架已有，待完善 |
| P4 | Hydration | 长期 |

---

## 验收标准

1. `cargo test -p tairitsu-ssr` 全部通过（含 hikari 联调测试）
2. `render_to_html(&hikari_wasm, default)` 返回包含 `#hikari-app`、`.hi-layout`、`.hikari-page` 的 HTML
3. 输出 HTML 在禁用 JS 的浏览器中能正确显示页面内容
4. hikari 侧 `test_e2e_no_js_visibility` 测试通过（直接 HTTP fetch 验证）
