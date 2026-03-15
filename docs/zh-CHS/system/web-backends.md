# Web 平台双后端：`web` 与 `wit-bindings`

`packages/web` 当前支持两条后端路径：

- `web`：基于 `wasm-bindgen` / `web-sys`（遗留路径）
- `wit-bindings`：基于 `wit-bindgen` 与 `tairitsu-browser:*` WIT 世界

## 能力对比

| 项目 | `web` | `wit-bindings` |
|---|---|---|
| 编译目标 | wasm32-unknown-unknown | wasm32-wasip2 |
| 宿主依赖 | 浏览器 JS + wasm-bindgen glue | Component Model host（browser-glue） |
| WIT 世界 | — | `browser-full`（Phase 0）或 `browser-extended`（全量） |
| W3C 接口覆盖率 | 取决于 web-sys 版本 | Phase 0：DOM/Events/Fetch/Canvas；Phase A：22 个自动生成域 |
| 兼容历史生态 | 高 | 中 |
| 面向未来协议演进 | 中 | 高 |

## WIT 世界层级

### Phase 0 — `tairitsu-browser:full@0.1.0`

手写核心接口，位于 `packages/browser-worlds/wit/browser-full.wit`：

- `node`、`document`、`window`、`style`、`event-target`：DOM 节点与样式
- `fetch-api`、`async-fetch`：标准 Fetch API（同步 + 异步 poll-handle）
- `canvas2d`：Canvas 2D 绘图接口
- 导出：`event-callbacks`（宿主在事件触发时回调 guest）

使用方式：

```rust
wit_bindgen::generate!({
    path:  "../browser-worlds/wit",
    world: "browser-full",
});
```

### Phase A — `tairitsu-browser:{domain}@0.x.0`

由 `scripts/generate_browser_wit.py` 从 W3C webref IDL 自动生成，位于
`packages/browser-worlds/wit/generated/`，共 22 个独立包：

| 包名 | WIT 文件 | 说明 |
|---|---|---|
| `tairitsu-browser:canvas@0.2.0` | canvas.wit | Canvas API（扩展） |
| `tairitsu-browser:css@0.2.0` | css.wit | CSS 对象模型 |
| `tairitsu-browser:crypto@0.2.0` | crypto.wit | Web Crypto API |
| `tairitsu-browser:device@0.2.0` | device.wit | Navigator / Device API |
| `tairitsu-browser:dom@0.2.0` | dom.wit | DOM（扩展） |
| `tairitsu-browser:events@0.2.0` | events.wit | 事件（扩展） |
| `tairitsu-browser:fetch@0.2.0` | fetch.wit | Fetch（扩展） |
| `tairitsu-browser:file-api@0.1.0` | file-api.wit | Blob/File/FileReader |
| `tairitsu-browser:geolocation@0.1.0` | geolocation.wit | Geolocation API（含 poll） |
| `tairitsu-browser:html@0.2.0` | html.wit | HTML 元素接口（扩展） |
| `tairitsu-browser:indexed-db@0.1.0` | indexed-db.wit | IndexedDB（完整 CRUD） |
| `tairitsu-browser:media@0.2.0` | media.wit | Media Streams & Devices |
| `tairitsu-browser:notifications@0.2.0` | notifications.wit | Web Notifications |
| `tairitsu-browser:observers@0.2.0` | observers.wit | MutationObserver 等 |
| `tairitsu-browser:performance@0.2.0` | performance.wit | Performance Timeline |
| `tairitsu-browser:permissions@0.2.0` | permissions.wit | Permissions API |
| `tairitsu-browser:resize-observer@0.1.0` | resize-observer.wit | ResizeObserver |
| `tairitsu-browser:storage@0.2.0` | storage.wit | StorageManager（含 poll） |
| `tairitsu-browser:streams@0.1.0` | streams.wit | WHATWG Streams |
| `tairitsu-browser:url@0.2.0` | url.wit | URL / URLSearchParams |
| `tairitsu-browser:web-animations@0.1.0` | web-animations.wit | Web Animations |
| `tairitsu-browser:webrtc@0.2.0` | webrtc.wit | WebRTC |
| `tairitsu-browser:websocket@0.2.0` | websocket.wit | WebSocket |
| `tairitsu-browser:workers@0.2.0` | workers.wit | Web Workers |

> **包命名约定**：所有包统一使用 `tairitsu-browser:{domain}` 前缀（已去除旧的 `-gen` 后缀）。

### 全量世界 — `tairitsu-browser:extended@0.1.0`

`packages/browser-worlds/wit/browser-extended.wit` 通过 WIT `include`
将 Phase 0 与所有 Phase A 域合并为一个超级世界：

```rust
wit_bindgen::generate!({
    path:  "../browser-worlds/wit",
    world: "browser-extended",
});
```

## 异步 API 模式（Poll-Handle Pattern）

所有原本返回 `Promise` 或依赖回调的浏览器 API 均采用 **poll-handle 模式**：

```wit
// 发起异步操作，返回 request-id（u64 句柄）
foo-async: func(...) -> result<u64, string>;

// 轮询结果，None = 尚未完成，Some = 完成（成功或错误）
poll-foo: func(request-id: u64) -> option<result<T, string>>;
```

适用于：fetch、IndexedDB、Blob 读取、Geolocation、StorageManager、FileReader 等。

## 打包工具（packager）

`packages/packager` 提供 `tairitsu build` CLI，支持以下 target：

| target | 说明 |
|---|---|
| `wasm`（默认） | wasm32-unknown-unknown + wasm-bindgen 路径 |
| `component` | wasm32-wasip2 + browser-glue 路径（Component Model） |

**组件构建流程**（`tairitsu build --target component`）：

1. 验证 `wasm32-wasip2` toolchain 已安装
2. `cargo build --target wasm32-wasip2 --lib`
3. 将 `.wasm` 组件拷贝到 `output_dir`
4. 将 `packages/browser-glue/dist/` 拷贝到 `output_dir/browser-glue/`
5. 生成宿主 `index.html`（通过 browser-glue 加载组件）

> **前提**：在 `packages/browser-glue/` 目录先运行 `npm run build` 以编译 TypeScript glue 代码。

## 代码入口

- `WebPlatform`：`packages/web/src/platform.rs`（`web` feature，web-sys 路径）
- `WitPlatform`：`packages/web/src/wit_platform.rs`（`wit-bindings` feature，WIT 路径）

## 使用建议

- 新项目且计划走组件模型：优先 `wit-bindings` + `browser-extended`
- 现有 `wasm-bindgen` 工程：先保持 `web`，再按迁移指南逐步切换
