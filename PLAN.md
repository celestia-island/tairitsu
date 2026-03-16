# Tairitsu <- Hikari 迁移协同计划

更新时间: 2026-03-16

## 背景

Hikari 已开始将默认构建入口切换到 `tairitsu-packager`，但其网站代码仍为 Dioxus 架构。
需要在 Tairitsu 侧补齐迁移支撑能力，降低改造成本。

## 来自 Hikari 的已落地对接点

- `just dev/watch/watch-dev/dev-by-agent` 已改为调用 `tairitsu-packager`。
- Hikari `examples/website/Cargo.toml` 已提供 `package.metadata.tairitsu`。
- `wasm-bindgen-cli` 安装脚本已进入弃用状态。

## 需要 Tairitsu 侧补齐

1. 迁移文档与模板

   - 提供从 Dioxus 到 Tairitsu 组件 API 的迁移手册（最小可运行示例）。
   - 提供 router / state / event 对照表与常见替代方案。

1. packager 兼容模式

   - 增加更清晰的错误提示：检测到 Dioxus / wasm-bindgen 依赖时给出迁移建议。
   - 提供 `tairitsu doctor` 或 `tairitsu build --explain`，输出阻塞项列表。

1. 过渡期支持

   - 可选：支持 legacy target 协助过渡（仅用于诊断，不作为长期方案）。
   - 在 watch / dev 日志中输出"下一步迁移建议"。

1. browser-glue 与组件宿主可观测性

   - 补充浏览器端初始化错误诊断信息。
   - 文档明确 `browser-glue` 构建前置步骤及自动检查策略。

## 协同里程碑建议

- M1: Hikari 网站最小页面（非 Dioxus）在 `tairitsu-packager dev` 成功启动。
- M2: Hikari 移除 `wasm-bindgen-cli` 运行依赖。
- M3: Hikari 移除 Dioxus 依赖并完成 CI 切换。
