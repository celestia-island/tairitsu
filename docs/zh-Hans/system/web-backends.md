# Web 后端：唯一路径 = WIT 契约 + JS 宿主实现

`packages/web` 只有**一条** Web 后端路径。早期文档描述过的「双后端」——`web`
（`wasm-bindgen` / `web-sys`）与 `wit-bindings`——已经不存在：`web` 侧没有消费者，
其依赖与 `cfg` 分支均已退役。

## 这条路径是什么

| 项目 | 取值 |
| --- | --- |
| 编译目标 | `wasm32-wasip2`（WASI 0.2 / 0.3 组件模型） |
| 契约 | WIT 世界 `tairitsu-browser:*`（`packages/browser-worlds/wit/`） |
| Guest 侧绑定 | `wit-bindgen`（`wit-bindings` feature） |
| 浏览器侧实现 | JS 宿主（`packages/browser-glue/`） |
| Guest 浏览器能力 | Component Model host，经宿主注入的 imports |

承载入口是 `WitPlatform`（`packages/web/src/wit_platform.rs`，`wit-bindings` feature）。
`BrowserPlatform`（`packages/web/src/browser.rs`）是一个占位实现——立即回调、默认 rect——
不是 web-sys 后端。

## 为什么只剩这一条

1. **组件模型改变了「能力」的交付方式**。`wasm32-unknown-unknown` + `web-sys` 把浏览器能力
   直接编译进 guest 的 import 段，宿主是谁、授予了什么能力都无从声明。WIT 契约把能力面
   写成可审阅、可版本化的 import 接口，宿主按实例授权。
2. **构建期转译，不是运行时自举**。jco 在构建期把 component 转译成 core wasm + JS
   canonical ABI 运行时；浏览器**没有**原生 `WebAssembly.Component`。所以工程上要求的
   「契约在 WIT、实现在 JS」是当前唯一可行形态，不是取舍。
3. **宿主职责被显式化**。实例化、能力授权、异步调度、生命周期（含硬终止）归 tairitsu
   宿主；guest 只描述它需要什么。

## 世界层级

WIT 世界按 `tairitsu-browser:{domain}` 分包，位于 `packages/browser-worlds/wit/`
（`browser-full.wit` 与 `generated/` 下的按域包）。具体包清单以该目录为准。

## 构建与打包

`packages/packager` 提供 `tairitsu build` CLI，**只支持 `component` 一个 target**
（`wasm32-wasip2` + browser-glue 路径）。传入其它 target 会以
`Unknown build target … Only 'component' is supported.` 失败。

组件构建流程：

1. 验证 `wasm32-wasip2` toolchain 已安装
2. `cargo build --target wasm32-wasip2 --lib`
3. 将 `.wasm` 组件拷贝到 `output_dir`
4. 将 browser-glue 运行时拷贝到 `output_dir/browser-glue/`
5. 生成宿主 `index.html`（通过 browser-glue 加载组件）

> **前提**：在 `packages/browser-glue/` 目录先运行 `npm run build` 以编译 TypeScript glue 代码。

## 使用建议

- 新工程直接按 `wit-bindings` + WIT 世界编写；不存在需要「先保持 `web` 再迁移」的存量路径。
- 需要新的浏览器能力时，先在 WIT 侧声明接口，再在 JS 宿主演实现——不要在 guest 里
  直接引 DOM 绑定库。
- 异步 API 的契约形状以 WIT 侧当前声明为准（见 `packages/browser-worlds/wit/`），
  本文不复述具体函数签名。