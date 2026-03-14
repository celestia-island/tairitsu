# 工作区地图

## 顶层结构

```text
packages/     Rust 核心包与工具包
examples/     示例工程（不同 WIT 集成路径）
scripts/      WebIDL/WIT 生成与辅助脚本
docs/         项目文档（多语言）
tests/        端到端相关资产
```

## 核心包说明

- `packages/runtime`：容器/镜像模型与 Wasmtime 组件执行
- `packages/macros`：`rsx!`、`wit_world!` 等宏
- `packages/vdom`：平台无关 VDOM 抽象与事件模型
- `packages/web`：Web 平台实现（`web` + `wit-bindings`）
- `packages/browser-worlds`：WIT 世界与嵌入资源
- `packages/browser-wit-resolver`：WIT 解析、缓存、拉取
- `packages/packager`：CLI 打包与 `wit` 子命令
- `packages/hooks`、`packages/style`：UI 运行时辅助层
- `packages/e2e`：端到端校验入口

## 示例工程建议阅读顺序

1. `examples/wit-native-macro`
2. `examples/wit-native-simple`
3. `examples/wit-runtime`
4. `examples/wit-compile-time`
5. `examples/wit-dynamic-advanced`
6. `examples/website`

## 关键生成产物与缓存

- `target/tairitsu-wit/`：WIT registry 缓存与 WebIDL 缓存
- `packages/browser-worlds/wit/generated/`：由脚本生成的 WIT 文件

## 新贡献者最小心智模型

1. `browser-worlds` 定义接口协议
2. `runtime` 承载组件执行
3. `web` 提供具体平台实现
4. `packager` 和脚本负责分发与生成
