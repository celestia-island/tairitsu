# Tairitsu 实现完备性计划 (PLAN.md)

## 当前状态

本轮计划项已全部完成，当前无待办阻塞项。

## 已落地范围

- WIT 世界扩展：`browser-extended` 已落地并可被 `wit_bindgen` 正常解析。
- 核心接口补全：Streams / File API / IndexedDB / Geolocation / Storage 已完成 poll-handle 化。
- 打包链路补全：packager 已支持 `tairitsu build --target component`。
- 包名统一：自动生成域已统一为 `tairitsu-browser:*` 命名空间。
- 文档同步：`web-backends` 已更新为当前真实实现。
- 编译注入：`packages/web/build.rs` 已注入 `TAIRITSU_DIST_DIR`。

## 验证结论

- `cargo check -p tairitsu-packager` 通过。
- `cargo check -p tairitsu-web --features wit-bindings --target wasm32-wasip2` 通过。
- `cargo test -p tairitsu-e2e` 通过（当前无测试用例，0 tests）。

## 后续原则

当前阶段仅做增量维护：

1. 新增接口优先遵循 poll-handle 模式。
2. 变更 WIT 签名时同步更新文档与 packager 入口。
3. 每次改动保持可编译、可回溯、可验证。
