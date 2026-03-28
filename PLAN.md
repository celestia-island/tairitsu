# Tairitsu Framework 计划

> Tairitsu 是一个现代化的 Rust Web 框架，支持服务端渲染、客户端渲染和静态站点生成。

---

## 已完成 ✅

### 核心

- [x] VDOM 系统
- [x] 组件系统 (`rsx!` 宏)
- [x] Hooks 系统 (`use_signal`, `use_effect` 等)
- [x] 上下文系统 (`use_context_provider`)
- [x] 事件处理
- [x] 样式系统集成
- [x] CSS 值类型系统
- [x] 全局状态管理

### 平台支持

- [x] WASI (wasm32-wasip2) 统一目标
- [x] 浏览器平台
- [x] 服务端渲染

### 子包

- [x] `tairitsu-runtime` - 核心运行时
- [x] `tairitsu-hooks` - Hooks 原语
- [x] `tairitsu-style` - CSS 生成系统
- [x] `tairitsu-ssr` - 服务端渲染
- [x] `tairitsu-vdom` - 虚拟 DOM 类型定义
- [x] `tairitsu-packager` - 资源打包
- [x] `tairitsu-cli` - 命令行工具
- [x] `tairitsu-css-values` - 类型安全的 CSS 值系统
- [x] `tairitsu-router` - 文件系统路由
- [x] `tairitsu-data-fetcher` - 服务端数据获取
- [x] `tairitsu-hmr` - 热模块替换
- [x] `tairitsu-fast-refresh` - 快速刷新
- [x] `tairitsu-error-overlay` - 错误覆盖层

### 异步支持

- [x] 异步组件 (`<Suspense>`)
- [x] 服务端数据获取
- [x] 流式 SSR

### WASM 组件浏览器接口

- [x] 动画帧接口 (Animation Frame) - `request-animation-frame`, `cancel-animation-frame`
- [x] 媒体查询接口 (Media Query) - `match-media`, `media-query-list`
- [x] DOM 几何接口 (DOM Geometry) - `get-bounding-client-rect`, scroll/size 方法
- [x] ClassList 操作 - `get-class-list`, `DOMTokenList`
- [x] Document/Window 全局访问 - `get-window`, `get-document`, `get-scroll-x/y`

---

## 计划中 📋

## 架构重整 🏗️

### 当前问题

当前 packages/ 下有 15+ 个功能包，存在以下问题：

1. **过度分散** - 用户需要分别引入多个包
2. **职责模糊** - web/hooks 和 web/style 分离但紧密耦合
3. **依赖复杂** - 跨包依赖频繁
4. **命名冗余** - 所有包都带 `tairitsu-` 前缀

### 目标架构

```
packages/
├── browser-glue/          # TypeScript WIT 实现
├── browser-worlds/        # WIT 定义
├── browser-wit-resolver/  # WIT 解析器
│
├── vdom/                  # VDOM 核心（hikari 基础设施，保持独立）
├── runtime/               # WASM 运行时（保持独立）
├── macros/                # 过程宏（保持独立）
│
├── web/                   # 🔴 整合目标：Web 框架核心
│   ├── src/
│   │   ├── core/          # 核心：Platform, Renderer, Component
│   │   ├── hooks/         # Hooks 系统
│   │   ├── dom/           # DOM 操作封装（新增）
│   │   ├── events/        # 事件处理（新增）
│   │   ├── style/         # 样式系统
│   │   ├── router/        # 路由
│   │   ├── ssr/           # SSR
│   │   └── data/          # 数据获取
│   └── Cargo.toml         # feature-gated 子模块
│
├── css-values/            # CSS 值类型（通用，保持独立）
├── i18n/                  # 国际化（通用，保持独立）
└── packager/              # CLI 工具（保持独立）

tests/                     # 测试工具（不发布）
├── browser/               # 原 browser-test
├── e2e/                   # 原 e2e
└── testing/               # 原 testing
```

### 包迁移映射

| 原包 | 新位置 | Feature |
|------|--------|---------|
| `tairitsu-hooks` | `tairitsu-web::hooks` | `hooks` |
| `tairitsu-style` | `tairitsu-web::style` | `style` |
| `tairitsu-router` | `tairitsu-web::router` | `router` |
| `tairitsu-ssr` | `tairitsu-web::ssr` | `ssr` |
| `tairitsu-data-fetcher` | `tairitsu-web::data` | `data` |
| `tairitsu-hmr` | `tairitsu-web::dev::hmr` | `hmr` |
| `tairitsu-fast-refresh` | `tairitsu-web::dev::fast_refresh` | `fast-refresh` |
| `tairitsu-error-overlay` | `tairitsu-web::dev::error_overlay` | `error-overlay` |
| `tairitsu-web` (旧) | `tairitsu-web::core` + `browser` | `browser` |
| `tairitsu-testing` | `tests/testing` | - |
| `tairitsu-e2e` | `tests/e2e` | - |
| `tairitsu-browser-test` | `tests/browser` | - |
| `tairitsu-i18n` | `tairitsu-i18n` | 保持独立 |

### 新 web 包结构

```
packages/web/
├── Cargo.toml               # feature-gated
├── src/
│   ├── lib.rs               # 统一入口
│   ├── core/                # 核心：Platform, Renderer
│   ├── hooks/               # Hooks 系统
│   ├── dom/                 # DOM 操作（新增）
│   ├── events/              # 事件处理（新增）
│   ├── style/               # 样式系统
│   ├── router/              # 路由
│   ├── ssr/                 # SSR
│   ├── data/                # 数据获取
│   └── dev/                 # 开发工具
│       ├── mod.rs
│       ├── hmr/
│       ├── fast_refresh/
│       └── error_overlay/
└── tests/                   # 集成测试
```

### Cargo.toml Features

```toml
[features]
default = ["browser", "hooks", "style"]

# 核心功能
core = []                    # 总是启用

# 浏览器平台
browser = ["core"]           # WIT 平台实现

# Hooks 系统
hooks = ["core"]             # use_signal, use_effect 等

# DOM 操作（新增）
dom = ["browser"]            # query_selector, class_list 等

# 事件处理（新增）
events = ["browser"]         # 事件监听器封装

# 样式系统
style = ["dom", "css-values"]# 样式操作

# 路由
router = ["core"]            # 文件系统路由

# SSR
ssr = ["core", "browser"]    # 服务端渲染

# 数据获取
data = ["core"]              # use_resource, fetch 等

# 开发工具
dev = []                     # 开发工具集合
hmr = ["dev"]                # 热模块替换
fast-refresh = ["dev", "hooks"] # 快速刷新
error-overlay = ["dev"]      # 错误覆盖层

# 全功能
full = ["browser", "hooks", "dom", "events", "style",
        "router", "ssr", "data", "dev", "hmr", "fast-refresh", "error-overlay"]
```

### 用户代码迁移

**Before:**
```toml
[dependencies]
tairitsu-vdom = "0.1"
tairitsu-hooks = "0.1"
tairitsu-style = "0.1"
tairitsu-router = "0.1"
```

**After:**
```toml
[dependencies]
tairitsu-vdom = "0.1"
tairitsu-web = { version = "0.2", features = ["hooks", "style", "router"] }
```

**Before:**
```rust
use tairitsu_hooks::use_signal;
use tairitsu_style::Style;
```

**After:**
```rust
use tairitsu_web::{use_signal, Style};
```

### 实施步骤

1. **Phase 1**: 创建 `packages/web-next/` 骨架
2. **Phase 2**: 按优先级迁移模块（core → hooks → dom → events → style → router → ssr → data → dev）
3. **Phase 3**: 更新 `packager/` 和 examples 依赖
4. **Phase 4**: 发布 `tairitsu-web v0.2.0`，标记旧包 deprecated
5. **Phase 5**: 移动测试包到 `tests/`，删除已整合的旧包
6. **Phase 6**: 重命名 `web-next → web`

---

## 未来规划 🔮
