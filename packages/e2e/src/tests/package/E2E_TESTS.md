# Tairitsu Package E2E Tests

## 测试目标

验证 tairitsu-package 能够从零开始正确创建和构建项目。

## 测试场景

### 1. 项目初始化测试

**测试流程：**
```bash
# 1. 创建新项目
cargo new test-app --lib
cd test-app

# 2. 添加依赖
cargo add tairitsu-vdom tairitsu-hooks tairitsu-macros

# 3. 创建最小配置的 Cargo.toml
cat > Cargo.toml << 'EOF'
[package]
name = "test-app"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
tairitsu-vdom = { path = "../../packages/vdom" }
tairitsu-hooks = { path = "../../packages/hooks" }
tairitsu-macros = { path = "../../packages/macros" }
wasm-bindgen = "0.2"

[package.metadata.tairitsu]
app-name = "Test App"

[package.metadata.tairitsu.build]
target = "wasm"
output-dir = "dist"

[package.metadata.tairitsu.dev]
port = 3000
EOF

# 4. 创建最小源代码
cat > src/lib.rs << 'EOF'
use wasm_bindgen::prelude::*;
use tairitsu_macros::rsx;

#[wasm_bindgen(start)]
pub fn main() {
    let _node = rsx! {
        div {
            "Hello, Tairitsu!"
        }
    };
}
EOF

# 5. 构建
tairitsu build

# 6. 验证输出
test -f dist/index.html
test -f dist/test_app.js
test -f dist/test_app_bg.wasm
```

**断言：**
- ✅ Cargo.toml 解析成功
- ✅ WASM 编译成功
- ✅ HTML 自动生成
- ✅ 输出文件完整

### 2. 资源嵌入测试

**测试配置：**
```toml
[package.metadata.tairitsu.assets]
inline-limit = 100
include = ["assets/**"]
```

**测试资源：**
```
assets/
├── small.txt     (< 100 bytes, should inline)
└── large.txt     (> 100 bytes, should copy)
```

**验证：**
- small.txt 被内联为 base64
- large.txt 被复制并添加哈希

### 3. 多平台打包测试

**测试命令：**
```bash
# Windows
tairitsu package --platform windows
test -f dist/test-app-setup.exe

# macOS
tairitsu package --platform macos
test -f dist/test-app.dmg

# Linux
tairitsu package --platform linux
test -f dist/test-app_0.1.0_amd64.deb
```

### 4. 开发服务器测试

**测试流程：**
```bash
# 启动服务器
tairitsu dev &
PID=$!

# 等待启动
sleep 3

# 测试访问
curl http://localhost:3000

# 测试 HMR
touch src/lib.rs
sleep 2
curl http://localhost:3000

# 停止服务器
kill $PID
```

### 5. 错误处理测试

**测试场景：**
1. 缺少必要配置
2. 资源文件不存在
3. 编译错误
4. 无效的配置值

## 测试数据

### 最小 Cargo.toml

```toml
[package]
name = "minimal-app"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
tairitsu-vdom = "0.1"
wasm-bindgen = "0.2"

[package.metadata.tairitsu.build]
target = "wasm"
```

### 完整 Cargo.toml

```toml
[package]
name = "full-featured-app"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
tairitsu-vdom = "0.1"
tairitsu-hooks = "0.1"
tairitsu-macros = "0.1"
wasm-bindgen = "0.2"

[package.metadata.tairitsu]
app-name = "Full Featured App"
title = "Full App - All Features"
description = "Testing all tairitsu-package features"

[package.metadata.tairitsu.build]
target = "wasm"
output-dir = "dist"
optimize = true
sourcemap = true

[package.metadata.tairitsu.dev]
port = 3000
hot-reload = true
open-browser = false

[package.metadata.tairitsu.assets]
inline-limit = 8192
include = ["assets/**", "public/**"]
exclude = ["**/*.md"]

[package.metadata.tairitsu.html]
lang = "zh-CN"
charset = "UTF-8"
viewport = "width=device-width, initial-scale=1.0"
favicon = "assets/favicon.ico"
head = """
<meta name="theme-color" content="#667eea">
"""
body-class = "app-container"

[package.metadata.tairitsu.css]
files = ["styles/main.css"]
autoprefixer = true
minify = true

[package.metadata.tairitsu.native]
identifier = "com.example.fullapp"
icon = "assets/icon.png"
copyright = "Copyright 2024"
```

## 测试套件

### 1. 单元测试

- Cargo.toml 解析器
- 资源处理器
- HTML 生成器
- 配置验证器

### 2. 集成测试

- 完整构建流程
- 开发服务器
- 热重载
- 资源处理

### 3. E2E 测试

- 项目初始化
- 多平台打包
- 生产部署
