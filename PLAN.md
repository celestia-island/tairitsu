# Tairitsu <- Hikari 迁移协同计划

## 背景

Hikari 已开始将默认构建入口切换到 `tairitsu-packager`，但其网站代码仍为 Dioxus 架构。
需要在 Tairitsu 侧补齐迁移支撑能力，降低改造成本。

## 已落地对接点

- `just dev/watch/watch-dev/dev-by-agent` 已改为调用 `tairitsu-packager`。
- Hikari `examples/website/Cargo.toml` 已提供 `package.metadata.tairitsu`。
- `wasm-bindgen-cli` 安装脚本已进入弃用状态。

## 协同里程碑

- M1: Hikari 网站最小页面（非 Dioxus）在 `tairitsu-packager dev` 成功启动。
- M2: Hikari 移除 `wasm-bindgen-cli` 运行依赖。
- M3: Hikari 移除 Dioxus 依赖并完成 CI 切换。
