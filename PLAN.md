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

---

## 计划中 📋

### WASM 组件浏览器接口补全

为了完全替代 `web-sys`/`wasm-bindgen`，以下 WIT 接口需要补充或完善：

#### 1. 动画帧接口 (Animation Frame)

**当前状态**: vdom Platform trait 有 `request_animation_frame`，但 browser-worlds WIT 缺失

**需求**:
```wit
interface animation-frame {
    /// Request an animation frame callback
    /// Returns a request ID that can be passed to cancel-animation-frame
    request-animation-frame: func(callback: func(timestamp: f64)) -> u32;

    /// Cancel a pending animation frame request
    cancel-animation-frame: func(id: u32);
}
```

**使用场景**:
- `packages/animation/src/builder/animation.rs` - 持续动画循环
- `packages/animation/src/timer.rs` - RAF 定时器实现

#### 2. 媒体查询接口 (Media Query)

**当前状态**: css.wit 定义了 `media-query-list` 但未集成到主要 world

**需求**:
```wit
interface media-query {
    type media-query-list-handle = u64;

    /// Create a MediaQueryList for a media query string
    match-media: func(query: string) -> option<media-query-list-handle>;

    /// Check if the media query currently matches
    matches: func(list: media-query-list-handle) -> bool;

    /// Add a listener for media query changes
    /// Returns listener-id for removal
    add-listener: func(list: media-query-list-handle) -> u64;

    /// Remove a media query listener
    remove-listener: func(list: media-query-list-handle, listener-id: u64);
}
```

**使用场景**:
- `packages/animation/src/prefers_reduced_motion.rs` - `prefers-reduced-motion` 检测

#### 3. DOM 几何接口 (DOM Geometry)

**当前状态**: observers.wit 有 `dom-rect` record，但缺少 `getBoundingClientRect`

**需求**:
```wit
interface dom-geometry {
    use types.{dom-rect};

    /// Get the bounding client rect of an element
    get-bounding-client-rect: func(element: u64) -> dom-rect;

    /// Get element dimensions (offsetWidth, offsetHeight)
    get-offset-size: func(element: u64) -> (width: f64, height: f64);

    /// Get element client dimensions (clientWidth, clientHeight)
    get-client-size: func(element: u64) -> (width: f64, height: f64);

    /// Get scroll position
    get-scroll-position: func(element: u64) -> (x: f64, y: f64);

    /// Get scroll dimensions
    get-scroll-size: func(element: u64) -> (width: f64, height: f64);
}
```

**使用场景**:
- `packages/animation/src/context.rs` - `bounding_rect()`
- `packages/animation/src/scrollbar.rs` - 滚动条尺寸计算

#### 4. ClassList 操作

**当前状态**: vdom Platform trait 有 `set_class`，但缺少细粒度操作

**需求**:
```wit
interface class-list {
    /// Add a class to an element
    add-class: func(element: u64, class-name: string) -> result<_, string>;

    /// Remove a class from an element
    remove-class: func(element: u64, class-name: string) -> result<_, string>;

    /// Toggle a class on an element
    toggle-class: func(element: u64, class-name: string, force: option<bool>) -> bool;

    /// Check if an element has a class
    has-class: func(element: u64, class-name: string) -> bool;
}
```

**使用场景**:
- `packages/animation/src/lifecycle.rs` - `apply_classes()`

#### 5. JavaScript 互操作类型

**问题**: `wasm-bindgen` 的 `JsCast`、`JsValue`、`Closure` 生命周期管理

**解决方案选项**:

A. **WIT 资源管理器模式**:
```wit
interface resource-manager {
    /// Create a host-owned reference that keeps a resource alive
    /// The guest can drop this when done
    ref: resource<function-refs> {
        /// Store a function reference for RAF loops
        store-function: func(f: func()) -> u64;
        drop-function: func(id: u64);
    }
}
```

B. **使用 Component Model 的 `resource`**:
```wit
/// Host-managed closure storage
interface closure-storage {
    resource closure-handle {
        /// Call the stored closure
        call: func();
    }

    /// Create a new host-owned closure
    create-closure: func() -> closure-handle;
}
```

**使用场景**:
- `packages/animation/src/events.rs` - 事件监听器生命周期
- `packages/animation/src/builder/animation.rs` - RAF 循环的 `js_sys::Function` 引用

#### 6. Document/Window 全局访问

**当前状态**: 部分接口存在，但缺少完整的 Document/Window 单例访问

**需求补充**:
```wit
interface window-global {
    /// Access global window object (returns a handle)
    get-window: func() -> u64;

    /// Get scroll position
    get-scroll: func() -> (x: f64, y: f64);

    /// Set scroll position
    set-scroll: func(x: f64, y: f64);

    /// Get document
    get-document: func() -> option<u64>;
}

interface document-global {
    /// Create element
    create-element: func(tag: string) -> option<u64>;

    /// Query selector
    query-selector: func(selector: string) -> option<u64>;
}
```

### 实现优先级

1. **高优先级** - 阻塞 hikari-animation 迁移:
   - Animation Frame 接口
   - DOM Geometry 接口 (getBoundingClientRect)
   - Media Query 接口

2. **中优先级** - 提升开发体验:
   - ClassList 细粒度操作
   - Document/Window 全局访问补充

3. **低优先级** - 可通过现有方案绕过:
   - Closure 生命周期管理 (可使用 vdom Platform trait 桥接)

### 迁移模式示例

#### 当前 wasm-bindgen 模式
```rust
// packages/animation/src/events.rs:131
let closure = Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
    // Animation logic here
}));
element
    .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
    .unwrap();
closure.forget(); // Keep alive indefinitely
```

#### 目标 Tairitsu WIT 模式
```wit
// events.wit (已存在)
interface event-target {
    add-event-listener: func(
        target: node-handle,
        event-type: string,
        use-capture: bool,
    ) -> result<u64, string>;  // Returns listener-id
}
```

```rust
// 迁移后代码
let listener_id = platform
    .add_event_listener(element_handle, "click", false)?;

// Guest 端导出的回调函数
#[no_mangle]
pub extern "C" fn on_mouse_event(listener_id: u64, data: MouseEventData) {
    // 根据 listener_id 分发到相应的闭包
}
```

#### RAF 循环迁移
```rust
// 当前代码 (packages/animation/src/builder/animation.rs:244-345)
let f = Rc::new(RefCell::new(None::<js_sys::Function>));
let animation_closure = Closure::wrap(Box::new(move || {
    // Animation loop logic
    if let Some(callback) = &*f.borrow() {
        let _ = web_sys::window()
            .and_then(|w| w.request_animation_frame(callback).ok());
    }
}) as Box<dyn FnMut()>);
let callback: &js_sys::Function = animation_closure.as_ref().unchecked_ref();
*f.borrow_mut() = Some(callback.clone());
```

```rust
// 迁移后代码
struct AnimationLoop {
    platform: Rc<RefCell<dyn Platform>>,
    active: Rc<RefCell<bool>>,
}

impl AnimationLoop {
    fn start(&self) {
        let active = self.active.clone();
        let platform = self.platform.clone();

        let callback = Box::new(move |timestamp: f64| {
            if *active.borrow() {
                // Animation logic here
                let _ = platform.borrow_mut().request_animation_frame(
                    Box::new(callback),  // 递归 - 需要处理
                );
            }
        });

        let _ = platform.borrow_mut().request_animation_frame(callback);
    }
}
```

### hikari-animation 当前 web_sys/wasm_bindgen 使用统计

| 文件 | 主要用途 | 使用的 web_sys 类型 |
|------|---------|-------------------|
| `events.rs` | 事件监听器生命周期 | `Closure`, `JsCast`, `MouseEvent`, `HtmlElement` |
| `builder/animation.rs` | RAF 动画循环 | `Closure`, `JsValue`, `js_sys::Function`, `Window`, `Performance` |
| `context.rs` | 动画上下文 | `HtmlElement`, `Window`, `DomRect`, `Document` |
| `prefers_reduced_motion.rs` | 媒体查询 | `Window`, `MediaQueryList`, `Closure` |
| `lifecycle.rs` | 生命周期管理 | `HtmlElement`, `JsValue`, `console` |
| `timer.rs` | 时间戳 | `js_sys::Date` |

---

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
