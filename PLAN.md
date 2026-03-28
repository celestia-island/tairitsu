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
- [x] `tairitsu-web` - 统一 Web 平台包（整合了所有 Web 相关功能，使用 feature-gated 模块）

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

## 未来规划 🔮
