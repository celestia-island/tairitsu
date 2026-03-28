# Tairitsu Framework 计划

> Tairitsu 是一个现代化的 Rust Web 框架，支持服务端渲染、客户端渲染和静态站点生成。

---

## 当前包结构分析

```
packages/
├── browser-glue/          # TypeScript WIT 实现 ✅ 保留
├── browser-worlds/        # WIT 定义 ✅ 保留
├── browser-wit-resolver/  # WIT 解析器 ✅ 保留
├── vdom/                  # VDOM 核心 ✅ 保留（核心类型）
├── runtime/               # WASM 运行时 ✅ 保留（核心）
├── macros/                # 过程宏 ✅ 保留（核心）
├── hooks/                 # Hooks 系统 ✅ 保留（核心）
├── style/                 # CSS 生成系统 ⚠️ 需整合 css-values
├── router/                # 文件系统路由 ⚠️ 可整合到 web
├── ssr/                   # 服务端渲染 ⚠️ 需整合 hmr/fast-refresh/error-overlay
├── data-fetcher/          # 数据获取 ⚠️ 可整合到 ssr
├── hmr/                   # 热模块替换 ❌ 应合并到 ssr
├── fast-refresh/          # 快速刷新 ❌ 应合并到 ssr
├── error-overlay/         # 错误覆盖层 ❌ 应合并到 ssr
├── css-values/            # CSS 值类型 ❌ 应合并到 style
├── i18n/                  # 国际化 ⚠️ 可整合到 web
├── packager/              # CLI 工具 ✅ 保留
├── web/                   # Web 平台实现 ❌ 需真正整合代码
├── e2e/                   # E2E 测试 ✅ 保留
└── testing/               # 测试工具 ✅ 保留
```

### 依赖关系分析

| 包 | 依赖 | 被依赖 | 建议操作 |
|---|---|---|---|
| `css-values` | 无外部依赖 | style | **合并到 style** |
| `hmr` | vdom | 无 | **合并到 ssr** |
| `fast-refresh` | vdom, hooks | 无 | **合并到 ssr** |
| `error-overlay` | vdom | 无 | **合并到 ssr** |
| `data-fetcher` | vdom | 无 | **合并到 ssr** |
| `router` | vdom | 无 | **合并到 web** |
| `i18n` | hooks | 无 | **合并到 web** |
| `style` | vdom, css-values | 无 | **接收 css-values** |

---

## 包架构重整计划 🔧

### 第一阶段：类型系统整合

#### 1.1 合并 `css-values` → `style`

**原因**：
- `css-values` 是纯类型定义，无外部依赖
- `style` 是其唯一使用者
- 分离增加复杂度，无独立存在的必要

**操作**：
```
packages/style/
├── src/
│   ├── values/          # 从 css-values 迁移
│   │   ├── mod.rs
│   │   ├── parser.rs
│   │   └── ...
│   ├── properties.rs
│   ├── builder.rs
│   └── lib.rs
└── Cargo.toml           # 更新依赖，移除 css-values
```

#### 1.2 合并 `router` → `web`

**原因**：
- Router 是 Web 应用核心功能
- 所有 Web 应用都需要路由
- 应作为 web 包的基础模块

**操作**：
```
packages/web/
├── src/
│   ├── router/          # 从 router 迁移
│   │   ├── mod.rs
│   │   ├── segment.rs
│   │   └── ...
│   ├── ...
```

#### 1.3 合并 `i18n` → `web`

**原因**：
- 国际化是 Web 应用通用需求
- 依赖 hooks，与 web 生态紧密

**操作**：
```
packages/web/
├── src/
│   ├── i18n/            # 从 i18n 迁移
│   │   ├── mod.rs
│   │   └── ...
```

### 第二阶段：开发体验功能整合到 SSR

#### 2.1 合并 `hmr` → `ssr`

**原因**：
- HMR 是开发服务器功能
- SSR 服务端需要支持 HMR
- 协议定义和注册应在 ssr 内部

**操作**：
```
packages/ssr/
├── src/
│   ├── hmr/             # 从 hmr 迁移
│   │   ├── mod.rs
│   │   ├── protocol.rs
│   │   └── registry.rs
│   ├── fast_refresh/    # 从 fast-refresh 迁移
│   │   ├── mod.rs
│   │   └── diff.rs
│   ├── error_overlay/   # 从 error-overlay 迁移
│   │   ├── mod.rs
│   │   └── templates.rs
│   ├── data_fetcher/    # 从 data-fetcher 迁移
│   │   ├── mod.rs
│   │   └── hooks.rs
│   └── ...
```

**新增 features**：
```toml
[features]
default = []
hmr = []
fast-refresh = ["hmr"]
error-overlay = ["hmr"]
data-fetcher = []
dev = ["hmr", "fast-refresh", "error-overlay", "data-fetcher"]
```

### 第三阶段：Web 包真正整合

#### 3.1 当前 `web` 包问题

当前 `web` 包只是重新导出（re-export），并未真正整合代码：
```rust
// 当前实现 - 只是转发
#[cfg(feature = "vdom")]
pub use tairitsu_vdom::*;
```

#### 3.2 真正整合目标

**整合后的结构**：
```
packages/web/
├── src/
│   ├── lib.rs           # 主入口
│   ├── prelude.rs       # Prelude 模块
│   ├── router/          # 整合自 router
│   ├── i18n/            # 整合自 i18n
│   ├── platform/        # 平台抽象
│   │   ├── browser.rs
│   │   ├── ssr.rs
│   │   └── mod.rs
│   └── wit/             # WIT 绑定
│       ├── platform.rs
│       └── bindings.rs
```

**Cargo.toml 简化**：
```toml
[dependencies]
tairitsu = { path = "../runtime" }
tairitsu-vdom = { path = "../vdom" }
tairitsu-hooks = { path = "../hooks" }
tairitsu-macros = { path = "../macros" }
tairitsu-style = { path = "../style" }      # 已包含 css-values
tairitsu-ssr = { path = "../ssr" }          # 已包含 hmr/fast-refresh/error-overlay/data-fetcher

# 整合后移除这些依赖：
# tairitsu-router (已合并到 web)
# tairitsu-i18n (已合并到 web)
# tairitsu-css-values (已合并到 style)
# tairitsu-hmr (已合并到 ssr)
# tairitsu-fast-refresh (已合并到 ssr)
# tairitsu-error-overlay (已合并到 ssr)
# tairitsu-data-fetcher (已合并到 ssr)
```

### 第四阶段：更新 examples

更新所有 examples 的依赖：
```toml
# 之前
tairitsu-web = { path = "../../packages/web", features = ["full"] }
tairitsu-router = { path = "../../packages/router" }
tairitsu-i18n = { path = "../../packages/i18n" }

# 之后
tairitsu-web = { path = "../../packages/web", features = ["full"] }
# router/i18n 直接从 web 包使用
```

---

## 重整后的包结构

```
packages/
├── browser-glue/          # TypeScript WIT 实现
├── browser-worlds/        # WIT 定义
├── browser-wit-resolver/  # WIT 解析器
│
├── vdom/                  # VDOM 核心类型
├── runtime/               # WASM 运行时
├── macros/                # 过程宏
├── hooks/                 # Hooks 系统
├── style/                 # CSS 系统（含 css-values）
│
├── ssr/                   # 服务端渲染（含 hmr/fast-refresh/error-overlay/data-fetcher）
│
├── web/                   # Web 平台（含 router/i18n）
│
├── packager/              # CLI 工具
├── e2e/                   # E2E 测试
└── testing/               # 测试工具
```

**包数量**：从 21 个减少到 12 个

---

## 实施步骤

### Step 1: 备份当前状态
- [ ] 提交当前代码到 dev

### Step 2: css-values → style
- [ ] 将 `css-values/src/*` 复制到 `style/src/values/`
- [ ] 更新 `style/src/lib.rs` 导出 values 模块
- [ ] 更新 `style/Cargo.toml`，移除 css-values 依赖
- [ ] 更新 `style/build.rs`（如有）
- [ ] 删除 `packages/css-values/`

### Step 3: router → web
- [ ] 将 `router/src/*` 复制到 `web/src/router/`
- [ ] 更新 `web/src/lib.rs` 导出 router 模块
- [ ] 更新 `web/Cargo.toml`，移除 router 依赖
- [ ] 删除 `packages/router/`

### Step 4: i18n → web
- [ ] 将 `i18n/src/*` 复制到 `web/src/i18n/`
- [ ] 更新 `web/src/lib.rs` 导出 i18n 模块
- [ ] 更新 `web/Cargo.toml`，移除 i18n 依赖
- [ ] 删除 `packages/i18n/`

### Step 5: hmr → ssr
- [ ] 将 `hmr/src/*` 复制到 `ssr/src/hmr/`
- [ ] 更新 `ssr/src/lib.rs` 导出 hmr 模块
- [ ] 更新 `ssr/Cargo.toml`，添加 hmr features
- [ ] 删除 `packages/hmr/`

### Step 6: fast-refresh → ssr
- [ ] 将 `fast-refresh/src/*` 复制到 `ssr/src/fast_refresh/`
- [ ] 更新 `ssr/src/lib.rs` 导出 fast_refresh 模块
- [ ] 更新 `ssr/Cargo.toml`，添加 fast-refresh feature
- [ ] 删除 `packages/fast-refresh/`

### Step 7: error-overlay → ssr
- [ ] 将 `error-overlay/src/*` 复制到 `ssr/src/error_overlay/`
- [ ] 更新 `ssr/src/lib.rs` 导出 error_overlay 模块
- [ ] 更新 `ssr/Cargo.toml`，添加 error-overlay feature
- [ ] 删除 `packages/error-overlay/`

### Step 8: data-fetcher → ssr
- [ ] 将 `data-fetcher/src/*` 复制到 `ssr/src/data_fetcher/`
- [ ] 更新 `ssr/src/lib.rs` 导出 data_fetcher 模块
- [ ] 更新 `ssr/Cargo.toml`，添加 data-fetcher feature
- [ ] 删除 `packages/data-fetcher/`

### Step 9: 更新 workspace
- [ ] 更新根 `Cargo.toml` members
- [ ] 删除废弃包的 entries

### Step 10: 更新 examples
- [ ] 更新所有 examples 的 Cargo.toml
- [ ] 更新导入路径
- [ ] 验证编译通过

### Step 11: 测试
- [ ] 运行 `cargo test --workspace`
- [ ] 运行 `cargo clippy --workspace`
- [ ] 验证 examples 正常工作

---

## 已完成功能 ✅

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

### 子包（功能完成，待整合）
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
- [x] 动画帧接口 (Animation Frame)
- [x] 媒体查询接口 (Media Query)
- [x] DOM 几何接口 (DOM Geometry)
- [x] ClassList 操作
- [x] Document/Window 全局访问

---

## 未来规划 🔮

### E2E 测试框架增强
- [ ] 内嵌浏览器运行时（而非依赖外部 Selenium）
- [ ] 原生 WASM 组件测试支持
- [ ] 视觉回归测试
