# 从 `web` 迁移到 `wit-bindings`

> 本页是简体中文入口，完整步骤与细节参考根文档 [docs/migration.md](../../migration.md)。

## 迁移目标

将浏览器平台实现从 `wasm-bindgen + web-sys` 切换到 WIT Component Model 路线。

## 最小步骤

1. Cargo 特性切换：`web` → `wit-bindings`
2. 编译目标切换：`wasm32-unknown-unknown` → `wasm32-wasip2`
3. 平台实例化切换：`WebPlatform::new()` → `WitPlatform::new()?`
4. 确认宿主具备 `tairitsu-browser:full` 所需导入（通常由 `browser-glue` 提供）

## 检查命令

```bash
cargo check -p tairitsu-web --features wit-bindings
```

## 注意事项

- `web` 与 `wit-bindings` 不应在同一最终二进制中同时启用
- `WitPlatform` 在非 wasm32 目标会返回 `Err`，这是预期行为
