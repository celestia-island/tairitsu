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
- ✅ 添加了 `platform-helpers` 接口，包含 DOM 辅助函数（get-document, create-element 等）
- ✅ 添加了 `dom-rect` 记录类型到 `types` 接口
- ✅ 添加了回调接口（timer-callbacks, animation-callbacks, resize-observer-callbacks 等）
- ✅ 修复了多个 WIT 接口的类型导入问题

**已知问题** (2026-03-24 更新):
- `resize-observer-entry` 接口的 `get-content-rect` 方法存在类型编组（marshalling）问题
- **最新进展**：错误从 "expected `u64` found `record"` 变为 "expected 4-tuple, found 1-tuple"
- **根本原因**：wasmtime 的 `func_wrap` API 将 `Result<(f64, f64, f64, f64)>` 解释为 1-tuple，而不是展开内部的 4-tuple
- **技术细节**：
  - WIT 记录类型在 canonical ABI 中被展平为元组
  - `dom-rect` (4个 f64 字段) 变成 `(f64, f64, f64, f64)`
  - 但 `func_wrap` 的 Result 包装导致 wasmtime 看到的是 1-tuple
- **尝试的解决方案**：
  - 嵌套元组：`Result<((f64, f64, f64, f64),), ...>` → 错误："expected tuple found record"
  - 直接元组（移除 Result）→ 编译错误（func_wrap 要求 Result）
  - WIT 返回元组而非记录 → 同样的 "expected 4-tuple, found 1-tuple" 错误
- **下一步**：
  - 研究 wasmtime 源码中的 `func_wrap` 实现
  - 尝试其他 API（如 `func_wrap_raw`、`func_wrap_async`）
  - 或暂时 stub `resize-observer-entry` 以解锁其他 SSR 测试
- 详细分析见 `/mnt/sdb1/tairitsu/RESIZE_OBSERVER_ISSUE.md`
- 此问题暂时阻止了 P0 任务的完成

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

### P2 — 完善 HTML 渲染质量 ✅ 已完成

**实现内容**：

- ✅ 添加了 `FullDocumentConfig` 结构体，包含 lang、charset、viewport、title、css_links 字段
- ✅ 实现了 `render_full_document_html()` 方法，生成完整的 HTML 文档
- ✅ 支持 `<!DOCTYPE html>` 声明
- ✅ 支持 `<html lang="...">` 属性
- ✅ 支持 `<head>` 内的 `<title>`、`<meta charset>`、`<meta name="viewport">`、CSS `<link>` 标签
- ✅ 添加了 `serde` 依赖用于配置序列化

**API 示例**：
```rust
use tairitsu_ssr::{SsrDom, FullDocumentConfig};

let config = FullDocumentConfig {
    lang: "zh-CN".to_string(),
    charset: "UTF-8".to_string(),
    viewport_content: "width=device-width, initial-scale=1.0".to_string(),
    title: "我的网站".to_string(),
    css_links: vec!["/styles/main.css".to_string()],
};

let html = dom.render_full_document_html(&config);
```

---

### P3 — Packager 集成（`tairitsu dev --ssr` / `tairitsu build --ssr`） ✅ 已完成

**实现内容**：

- ✅ `tairitsu dev --ssr` - SSR 开发服务器，支持实时渲染
- ✅ `tairitsu build --ssr` - 静态站点生成，预渲染 HTML
- ✅ 错误页面渲染和友好的错误消息
- ✅ SPA 回退支持（非根路径）
- ✅ 无缓存头支持
- ✅ 自动端口选择和浏览器自动打开

**功能特性**：
- 开发服务器模式：监听文件变化，重新编译 WASM，每次请求时调用 SSR
- 预渲染模式：生成静态 HTML 文件，提高首屏加载速度
- 错误处理：友好的错误页面显示包名和错误信息
- 模板回退：当 index.html 不存在时使用默认模板

**使用示例**：
```bash
# 开发服务器模式
tairitsu dev --ssr

# 预渲染模式
tairitsu build --ssr
```

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
