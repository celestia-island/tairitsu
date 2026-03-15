# Tairitsu 实现完备性计划 (PLAN.md)

## 当前状态

本计划已全部完成，当前无待办阻塞项。

## 已完成项

- browser-extended 世界已落地，WIT 解析与绑定可用。
- Streams / File API / IndexedDB / Geolocation / Storage 已完成 poll-handle 模式补全。
- packager 已支持 component 构建路径（wasm32-wasip2 + browser-glue）。
- WIT 包命名已统一为 tairitsu-browser:*。
- web 后端文档已与当前实现同步。
- web 构建阶段已注入 TAIRITSU_DIST_DIR 环境变量。

## 自检结果

- cargo check --workspace --all-targets：通过
- cargo clippy --workspace --all-targets -- -D warnings：通过
- cargo check -p tairitsu-web --features wit-bindings --target wasm32-wasip2：通过
- cargo test -p tairitsu-e2e：通过（当前 0 tests）

## 持续约束

1. 新增接口优先遵循 poll-handle 模式。
2. 变更 WIT 签名时同步更新文档与 packager 入口。
3. 每次改动保持可编译、可回溯、可验证。
