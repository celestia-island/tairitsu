# Tairitsu <- Hikari 迁移协同计划

## 背景

Hikari 已开始将默认构建入口切换到 `tairitsu-packager`，但其网站代码仍为 Dioxus 架构。
需要在 Tairitsu 侧补齐迁移支撑能力，降低改造成本。

## 来自 Hikari 的已落地对接点

- `just dev/watch/watch-dev/dev-by-agent` 已改为调用 `tairitsu-packager`。
- Hikari `examples/website/Cargo.toml` 已提供 `package.metadata.tairitsu`。
- `wasm-bindgen-cli` 安装脚本已进入弃用状态。

## 已完成

1. 迁移文档与模板

   - 提供了从 Dioxus 到 Tairitsu 组件 API 的迁移手册（最小可运行示例）。
   - 提供了 router / state / event 对照表与常见替代方案。
   - 文档位置: `docs/en-US/guides/migration/dioxus-to-tairitsu.md` 和 `docs/zh-CHS/guides/migration/dioxus-to-tairitsu.md`

1. packager 兼容模式

   - 增加了更清晰的错误提示：检测到 Dioxus / wasm-bindgen 依赖时给出迁移建议。
   - 提供了 `tairitsu doctor` 命令，输出阻塞项列表。
   - 支持文本和 JSON 输出格式

1. browser-glue 与组件宿主可观测性

   - 补充了浏览器端初始化错误诊断信息。
   - 实现了 preventDefault/stopPropagation 事件控制
   - 添加了诊断回调接口和错误报告机制
   - 增强了句柄表统计和诊断功能

1. E2E 测试增强

   - 添加了 doctor 命令测试
   - 添加了组件生命周期测试
   - 添加了事件处理测试
   - 添加了构建流程测试
   - 添加了错误处理测试

## 协同里程碑

- M1: Hikari 网站最小页面（非 Dioxus）在 `tairitsu-packager dev` 成功启动。
- M2: Hikari 移除 `wasm-bindgen-cli` 运行依赖。
- M3: Hikari 移除 Dioxus 依赖并完成 CI 切换。
