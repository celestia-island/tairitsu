# Tairitsu 文档中心（简体中文）

> 基于 WASM Component Model 的全栈框架

## 入门

| 文档 | 说明 |
|:--|:--|
| [入门教程](getting-started.md) | 从零开始构建全栈应用。涵盖 `tairitsu new`、第一个组件、服务端+浏览器运行、部署。 |
| [快速开始](quick-start.md) | 5 分钟安装与验证。 |
| [工作区导览](workspace-map.md) | Monorepo 结构一览。 |
| [构建、测试与发布](build-test-release.md) | 如何使用 `just` 命令进行开发工作流。 |

## 迁移

| 文档 | 说明 |
|:--|:--|
| [从 web-sys 迁移到 WIT 绑定](migration.md) | 从 `wasm-bindgen`/`web-sys` 迁移到 Component Model WIT 绑定。 |
| [从 Dioxus 迁移到 Tairitsu](migration/dioxus-to-tairitsu.md) | API 对比：组件、Hooks、事件、路由、状态管理、条件渲染。 |

## 参考

| 文档 | 说明 |
|:--|:--|
| [术语对照表](glossary.md) | 核心术语：WIT、Component Model、VNode、Signal、Platform、Container 等 |
| [故障排除](troubleshooting.md) | 常见问题与解决方案。 |

## 架构

| 文档 | 说明 |
|:--|:--|
| [系统总览](../system/overview.md) | 四层架构：Interface → Runtime → Platform → Tooling |
| [运行时与容器模型](../system/runtime.md) | Image/Container/Registry 生命周期、WIT 绑定、动态调用 |
| [VDOM 与渲染](../system/vdom.md) | 虚拟 DOM 差分、修补、事件系统、响应式调度器 |
| [W3C WebIDL → WIT 流水线](../system/wit-pipeline.md) | 50+ WebIDL 规格如何转换为 WIT 接口 |
| [Web 平台双后端](../system/web-backends.md) | WitPlatform 与 WebPlatform 策略 |
| [Browser Glue 架构](../system/browser-glue.md) | 桥接 WIT ABI 与 DOM 的 TypeScript 层 |
| [版本与兼容性策略](../system/versioning.md) | 多 Crate 工作区的语义化版本管理 |

## 包参考

| 文档 | 说明 |
|:--|:--|
| [包分层与职责总览](../components/index.md) | 四层 Crate 层级与依赖图 |
| [Workspace 包清单](../components/packages.md) | 各 Crate 的详细说明 |

## 高级

| 文档 | 说明 |
|:--|:--|
| [调试代理](../skills/debug-agent.md) | 使用 MCP 服务器进行 AI 辅助调试 |
| [企业支持](../enterprise/support.md) | 商业支持选项 |
