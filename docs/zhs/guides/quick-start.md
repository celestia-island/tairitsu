# 快速开始

本指南帮助你在本地完成最小可用链路：安装工具、构建、测试、运行示例。

## 1. 环境准备

- Rust（建议 stable）
- `just` 命令工具
- Python 3（用于 WIT 生成脚本）
- Node.js（用于 `packages/browser-glue`）

执行：

```bash
just install-tools
```

## 2. 一次性全量校验

```bash
just test
```

该命令会运行核心编译与检查流程，用于快速确认环境与依赖可用。

## 3. 运行示例

```bash
# 宏驱动示例
just run-macro-demo

# trait 驱动示例
just run-simple-demo

# 动态调用示例
just run-dynamic-advanced
```

更多示例见 [examples/README.md](../../../examples/README.md)。

## 4. 浏览器 WIT 路线（可选）

若你计划使用 Component Model 浏览器接口：

```bash
cargo check -p tairitsu-web --features wit-bindings
```

并参考 [迁移说明](./migration.md)。

## 5. 常见问题

### 找不到 `wasm32-wasip2` target

执行：

```bash
rustup target add wasm32-wasip2
```

### Python 脚本报错

优先确认：

- `python3` 可执行
- 网络可访问 `w3c/webref`
- 项目目录可写

### TypeScript 检查失败

在 `packages/browser-glue` 中执行：

```bash
npm install
npm run typecheck
```
