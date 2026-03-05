# Tairitsu Web Demo

Simple web demo showcasing Tairitsu's Virtual DOM, Hooks, and rsx! macro.

**配置方式**: 使用 Cargo.toml metadata，无需 HTML 模板

## Quick Start

```bash
# 安装 tairitsu-package（构建工具）
cargo install --path ../../packages/package

# 启动开发服务器
tairitsu dev

# 或使用 just
just dev
```

The demo will be available at http://localhost:3000

## 配置说明

### Cargo.toml 配置

所有配置都在 `Cargo.toml` 的 `[package.metadata.tairitsu]` 中：

```toml
[package.metadata.tairitsu]
app-name = "Tairitsu Web Demo"
title = "Tairitsu Demo"

[package.metadata.tairitsu.build]
target = "wasm"
output-dir = "dist"

[package.metadata.tairitsu.dev]
port = 3000
hot-reload = true

[package.metadata.tairitsu.html]
lang = "zh-CN"
charset = "UTF-8"
viewport = "width=device-width, initial-scale=1.0"
head = """
<style>
  body { /* 自定义样式 */ }
</style>
"""
```

### 自动生成的 HTML

`tairitsu build` 会根据配置自动生成 `dist/index.html`：

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta name="theme-color" content="#667eea">
    <title>Tairitsu Demo</title>
    <style>/* 配置中的样式 */</style>
</head>
<body class="app-container">
    <div id="app"></div>
    <script type="module" src="/tairitsu_web_demo.js"></script>
</body>
</html>
```

## Building for Production

```bash
# 构建优化版本
tairitsu build

# 或使用 just
just build-web
```

Output will be in `dist/`

## Project Structure

```
examples/web-demo/
├── Cargo.toml          # 包含所有配置（无需 HTML）
├── src/
│   └── lib.rs          # WASM 入口
├── assets/             # 静态资源（可选）
└── README.md           # 本文件
```

## Features Demonstrated

### Main Demo
- ✅ Virtual DOM rendering
- ✅ rsx! macro for declarative UI
- ✅ WASM compilation
- ✅ Cargo.toml 配置（无 HTML 模板）
- ✅ 自动生成 HTML

## Development

### Prerequisites

- Rust (with wasm32-unknown-unknown target)
- tairitsu-package (构建工具)
- just (command runner)

```bash
# Install prerequisites
rustup target add wasm32-unknown-unknown
cargo install --path ../../packages/package
cargo install just
```

### Hot Reload

The development server automatically reloads when you make changes to:
- Rust code (`.rs` files)
- Assets in configured directories

## 与 Trunk 的区别

| 特性 | Trunk | Tairitsu Package |
|------|-------|------------------|
| 配置方式 | HTML 模板 | Cargo.toml metadata |
| HTML 文件 | 需要手动编写 | 自动生成 |
| 资源引用 | `<link data-trunk>` | 配置文件 |
| 类型安全 | ❌ | ✅ 编译时检查 |
| IDE 支持 | ❌ | ✅ TOML schema |

## Browser Support

- Chrome/Edge 88+
- Firefox 78+
- Safari 14+

## Related Documentation

- [Main README](../../README.md)
- [Examples README](../README.md)
- [PLAN.md - tairitsu-package](../../PLAN.md#phase-7-tairitsu-package-)
