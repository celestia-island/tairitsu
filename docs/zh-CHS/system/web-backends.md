# Web 平台双后端：`web` 与 `wit-bindings`

`packages/web` 当前支持两条后端路径：

- `web`：基于 `wasm-bindgen` / `web-sys`
- `wit-bindings`：基于 `wit-bindgen` 与 `tairitsu-browser:full`

## 能力对比

| 项目 | `web` | `wit-bindings` |
|---|---|---|
| 编译目标 | wasm32-unknown-unknown | wasm32-wasip2 |
| 宿主依赖 | 浏览器 JS + wasm-bindgen glue | Component Model host（browser-glue） |
| 兼容历史生态 | 高 | 中 |
| 面向未来协议演进 | 中 | 高 |

## 代码入口

- `WebPlatform`：`packages/web/src/platform.rs`
- `WitPlatform`：`packages/web/src/wit_platform.rs`

## 实现要点

- `WitPlatform` 类型在 `wit-bindings` 启用时可见
- 完整 `Platform` trait 实现在 `target_family = "wasm"` 下编译
- 非 wasm32 环境调用 `WitPlatform::new()` 返回 `Err`

## 使用建议

- 新项目且计划走组件模型：优先 `wit-bindings`
- 现有 `wasm-bindgen` 工程：先保持 `web`，再按迁移指南逐步切换
