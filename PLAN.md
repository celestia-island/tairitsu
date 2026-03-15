# Tairitsu 实现完备性计划 (PLAN.md)

> 扫描日期：2026-03-15  
> 基准版本：当前 main 分支

---

## 现状摘要

项目 WIT 接口层分为两轨：

- **Phase-0 手写层**（`packages/browser-worlds/wit/*.wit` `@0.1.0`）：DOM / Events / Fetch / Canvas，质量高，已与 `WitPlatform` + `browser-glue` 完全打通。
- **Phase-A 自动生成层**（`wit/generated/*.wit` `@0.2.0`）：由 `scripts/generate_browser_wit.py` 从 W3C webref IDL 自动生成，覆盖 22+ 规范领域，但**尚未整合进 `browser-full` world**，且多个接口体方法为空。

---

## 差距清单与优先级

### P0 — 阻塞性问题

#### G-1：`browser-full` world 未整合 Phase-A 接口

- **问题**：`browser-full.wit` 只聚合了 DOM/Events/Fetch/Canvas 四个 Phase-0 接口。`WitPlatform` 的 `wit_bindgen::generate!` 指向该 world，导致 WebRTC / WebSocket / Storage / Streams 等接口运行时不可用。
- **解法**：新建 `packages/browser-worlds/wit/browser-extended.wit` super-world，将所有 Phase-A 接口合入；同时在 `browser-full.wit` 注释处标记迁移路径。
- **状态**：✅ 完成 — 见 `browser-extended.wit`

#### G-2：`Streams` / `Blob` / `File` 接口体为空

- **问题**：`streams.wit` 中 `ReadableStream`、`WritableStream`、`TransformStream` 三个接口仅有句柄类型，无任何方法；`file-api.wit` 中 `Blob`、`File` 同样为空壳，`FileReader` 的 `read-as-*` 均被 skipped。
- **根因**：IDL 的 `ReadableStreamDefaultReader` / `ReadableStreamBYOBReader` 依赖回调与 Promise，自动生成脚本无法映射。
- **解法**：采用**轮询句柄 (poll-handle) 模式**，参照 `async-fetch` 的设计：
  - `ReadableStream`：`read-chunk(handle) -> option<list<u8>>` + `is-closed(handle) -> bool`
  - `WritableStream`：`write-chunk(handle, data: list<u8>) -> result<_, string>` + `close(handle)`
  - `Blob`：`slice(handle, start, end) -> blob-handle` + `read-all(handle) -> list<u8>`
  - `FileReader`：`read-as-array-buffer(handle, blob) -> u64` + `poll-result(request-id) -> option<list<u8>>`
- **状态**：✅ 完成 — `streams.wit` / `file-api.wit` 已全量实现 poll-handle 方法

#### G-3：`IndexedDB` 大量操作 skipped

- **问题**：`IDBFactory.open`、`IDBObjectStore.put/add/get/delete` 等核心操作均被 `// skipped (unmappable return type)` 标注，导致 IDB 实际上无法使用。
- **根因**：`IDBRequest<T>` 是泛型 Promise-like 对象，IDL-to-WIT 脚本遇到复杂返回类型时放弃。
- **解法**：在 `indexed-db.wit` 中引入统一的 `idb-request-handle = u64` + `poll-idb-request(handle) -> option<result<string, string>>` 模式（value 序列化为 JSON string），并手写 `idb-factory` 的 `open` 操作补丁。
- **状态**：✅ 完成 — `indexed-db.wit` 已实现完整 CRUD + poll-handle 模式

#### G-4：packager 不支持 `wasm32-wasip2` + browser-glue 打包

- **问题**：`packager/src/wasm/mod.rs` 的整个构建流程（`build_wasm` → `run_wasm_bindgen` → `generate_html`）硬编码走 `wasm32-unknown-unknown` + `wasm-bindgen`。切到 `wit-bindings` 后无对应的打包路径，browser-glue TypeScript 不会被自动注入到 dist 目录。
- **解法**：
  1. 在 `BuildConfig` 增加 `target: ComponentTarget { Wasm | Component }` 变体
  2. `Component` 路径：`cargo build --target wasm32-wasip2 --lib` → `wasm-tools component` 封装 → 将 `packages/browser-glue/` bundle（esbuild/rollup）→ 生成 HTML loader
  3. `output_dir` 同样由 `Tairitsu.toml` `[build] output_dir` 驱动，无需 build.rs
- **状态**：✅ 完成 — `packager/src/wasm/mod.rs` 新增 `build_component()`；`cli/mod.rs` 路由 `"component"` target

---

### P1 — 重要但非阻塞

#### G-5：两套包名 / 版本不统一

- **问题**：Phase-0 为 `tairitsu-browser:*@0.1.0`，Phase-A 为 `tairitsu-browser-gen:*@0.2.0`。`-gen` 后缀和版本不一致对使用者不友好，也无法在同一 world 中无缝 `use`。
- **解法**：将所有自动生成文件迁移至 `tairitsu-browser:*@0.2.0` 命名空间，删除 `-gen` 后缀；在生成脚本 `generate_browser_wit.py` 中更新 `PACKAGE_NAMESPACE`。
- **状态**：✅ 完成 — 所有 27 个生成 WIT 文件及 `generate_browser_wit.py` 统一为 `tairitsu-browser:*`

#### G-6：Geolocation / Promise-returning API 缺少 poll 接口

- **问题**：`geolocation.wit` 的 `get-current-position` 只接受一个 `geo-handle` 参数，丢失了 `successCallback` / `errorCallback`；`StorageManager.persisted()` 等返回裸 `u64`（Promise 句柄），没有配套的 poll 函数。
- **解法**：统一设计 `promise-handle + poll-promise(handle) -> option<result<string, string>>` 模式（参照 async-fetch），在 `geolocation.wit` 和 `storage.wit` 中补充。
- **状态**：✅ 完成 — `geolocation.wit` / `storage.wit` 均已实现完整 poll 模式

#### G-7：`html.wit` 未整合

- **问题**：`generated/html.wit` 覆盖了 HTMLElement、HTMLInputElement 等常用 HTML 接口，但未出现在任何 world 中。
- **解法**：合入 `browser-extended.wit`。
- **状态**：✅ 完成 — `html-world` 已纳入 `browser-extended.wit`

---

### P2 — 体验优化

#### G-8：`build.rs` 无 `OUT_DIR` 驱动

- **问题**：对直接使用 `wit-bindings` feature 的下游 crate，没有 build.rs 机制把 dist 目录路径注入为环境变量，无法在代码里用 `env!("TAIRITSU_DIST_DIR")` 引用输出路径。
- **解法**：在 `packages/web/` 增加 build.rs，读取 `[package.metadata.tairitsu] output_dir`，通过 `cargo:rustc-env=TAIRITSU_DIST_DIR=…` 暴露。
- **状态**：⏳ 暂缓 — packager 侧 Component 流程完成；`web/build.rs` 注入可作为后续迭代

#### G-9：`browser-full.wit` 注释 Phase 标记过时

- **问题**：文件头注释写的是 "Phase 0 — covers dom, events, fetch, canvas. Phase 2 will add storage, workers, websocket, streams." 与实际 Phase-A 进度不符。
- **解法**：更新注释，反映 `browser-extended.wit` 路径。
- **状态**：✅ 完成 — `browser-full.wit` 注释中已增加 `browser-extended.wit` 路径说明

---

## 任务列表

| ID | 标题 | 优先级 | 状态 |
|----|------|--------|------|
| T-1 | 更新 `PLAN.md`（本文件） | — | ✅ 完成 |
| T-2 | 补全 Streams / Blob / File 接口体（poll 模式） | P0 | ✅ 完成 |
| T-3 | 补全 IndexedDB poll 模式操作 | P0 | ✅ 完成 |
| T-4 | 新建 `browser-extended.wit` super-world，整合所有 Phase-A 接口 | P0 | ✅ 完成 |
| T-5 | packager 支持 `wasm32-wasip2` Component 打包路径 + browser-glue bundle | P0 | ✅ 完成 |
| T-6 | 统一包名 `tairitsu-browser:*@0.2.0`，去除 `-gen` 后缀 | P1 | ✅ 完成 |
| T-7 | 补充 Geolocation / Promise-returning API 的 poll 接口 | P1 | ✅ 完成 |
| T-8 | 更新 `docs/zh-CHS/system/web-backends.md` 文档 | P2 | ✅ 完成 |

---

## 实施顺序

```
T-6（统一包名）→ T-2（Streams/Blob/File）→ T-3（IndexedDB）→ T-7（Geolocation/Promise）
                                                              ↓
                                                T-4（browser-extended.wit）
                                                              ↓
                                                T-5（packager Component 路径）
                                                              ↓
                                                T-8（文档更新）
```

T-6 先行是因为后续所有 `.wit` 文件都要用统一包名，避免二次迁移。

---

## 设计参考：通用 promise-handle poll 模式

所有 Promise-returning 或 callback-based 的浏览器 API 统一用以下 WIT 惯用模式表达：

```wit
/// Enqueue an async operation; returns an opaque request handle.
foo-async: func(...) -> result<u64, string>;

/// Poll a pending request.
/// Returns `none` while in-flight, `some(ok(...))` on success,
/// `some(err(...))` on failure.
poll-foo: func(handle: u64) -> option<result<string, string>>;

/// Cancel a pending request.
cancel-foo: func(handle: u64);
```

value 统一序列化为 JSON string，由 browser-glue 侧负责 stringify/parse，避免在 WIT 层引入复杂的多态类型。
