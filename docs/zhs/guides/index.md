# Tairitsu 文档中心（简体中文）

> 泛型 WASM Component Runtime 引擎

本目录是 Tairitsu 的简体中文主文档，覆盖从入门到架构、从开发到发布的完整链路。

## 文档导航

### Guides

- [快速开始](./quick-start.md)
- [工作区地图](./workspace-map.md)
- [构建、测试与发布](./build-test-release.md)
- [从 web-sys 迁移到 WIT 绑定](./migration.md)
- [从 Dioxus 迁移到 Tairitsu](./migration/dioxus-to-tairitsu.md)
- [故障排除指南](./troubleshooting.md)
- [术语对照表](./glossary.md)

### System

- [系统总览](../system/overview.md)
- [运行时与容器模型](../system/runtime.md)
- [W3C WebIDL → WIT 生成流水线](../system/wit-pipeline.md)
- [Web 平台双后端（web / wit-bindings）](../system/web-backends.md)
- [Browser Glue 架构](../system/browser-glue.md)
- [版本与兼容性策略](../system/versioning.md)

### Components

- [包分层与职责总览](../components/index.md)
- [Workspace 包清单](../components/packages.md)

## 目标读者

- 想快速跑通示例与测试的新贡献者
- 需要定制 WIT 接口并托管 WASM 组件的工程师
- 计划将浏览器接口从 `wasm-bindgen` 迁移到 WIT Component Model 的团队

## 推荐阅读路径

1. 新用户：快速开始 → 系统总览 → 运行时与容器模型
2. 浏览器方向：迁移说明 → Web 平台双后端 → WIT 流水线
3. 维护者：工作区地图 → 包清单 → 版本策略
