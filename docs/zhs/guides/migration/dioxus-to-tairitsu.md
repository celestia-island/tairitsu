# 从 Dioxus 迁移到 Tairitsu

本指南帮助您将应用从 Dioxus 迁移到 Tairitsu。Tairitsu 是一个 WebAssembly Component Model 框架，提供了与 Dioxus 相似的功能，但具有不同的架构和 API。

## 目录

- [架构对比](#架构对比)
- [环境设置](#环境设置)
- [API 对照](#api-对照)
  - [路由](#路由)
  - [状态管理](#状态管理)
  - [事件](#事件)
- [组件迁移](#组件迁移)
- [Hooks 对照](#hooks-对照)
- [事件系统差异](#事件系统差异)
- [最小可运行示例](#最小可运行示例)
- [常见迁移模式](#常见迁移模式)

## 架构对比

### Dioxus 架构

```
Dioxus 应用
    |
    v
虚拟 DOM (RSX)
    |
    v
渲染器 (Web/桌面/移动)
```

- **RSX 宏**: 编译时类似 JSX 的语法
- **Props**: 基于结构体的组件属性
- **Hooks**: `use_hook`, `use_state`, `use_effect`
- **调度器**: 内部的类似 fiber 的协调算法
- **渲染器**: 平台特定的渲染器

### Tairitsu 架构

```
Tairitsu 应用 (WASM 组件)
    |
    v
VDOM + 平台抽象
    |
    v
平台层 (WebPlatform/WitPlatform)
    |
    v
浏览器主机 (browser-glue) 或原生主机
```

- **rsx! 宏**: 类似语法的声明式 UI 构建
- **Props**: 使用 `#[component]` 属性的构建器模式
- **Reactive**: 基于 Signal/Effect 的响应式系统
- **Platform Trait**: 抽象 DOM 操作
- **双后端**: `WebPlatform` (web-sys) 和 `WitPlatform` (WIT 绑定)

## 环境设置

### Dioxus 设置

```toml
[dependencies]
dioxus = "0.5"
dioxus-web = "0.5"
```

### Tairitsu 设置

```toml
[dependencies]
tairitsu-vdom = { path = "../../../packages/vdom" }
tairitsu-macros = { path = "../../../packages/macros" }
tairitsu-web = { path = "../../../packages/web", features = ["wit-bindings"] }

[lib]
crate-type = ["cdylib", "rlib"]
```

### 目标差异

| 方面 | Dioxus | Tairitsu |
|------|--------|----------|
| 目标平台 | `wasm32-unknown-unknown` | `wasm32-wasip2` |
| 绑定方式 | `wasm-bindgen` | `wit-bindgen` |
| 模型 | 普通 WASM | 组件模型 |

### 构建命令

```bash
# Dioxus
dx build

# Tairitsu
just build-web
# 或
cargo build --target wasm32-wasip2 --features wit-bindings
```

## API 对照

### 路由

#### Dioxus 路由

```rust
use dioxus::prelude::*;
use dioxus_router::prelude::*;

#[derive(Routable, Clone, Copy)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/about")]
    About {},
}

fn App() -> Element {
    use_init_router();
    render! { Router::<Route> {} }
}
```

#### Tairitsu 路由

Tairitsu 不包含内置路由器。您需要自己实现路由：

```rust
use tairitsu_vdom::{VNode, VElement};

#[derive(Clone, Copy, PartialEq)]
enum Route {
    Home,
    About,
}

struct Router {
    current_route: Route,
}

impl Router {
    fn new() -> Self {
        Self {
            current_route: Route::Home,
        }
    }

    fn navigate(&mut self, route: Route) {
        self.current_route = route;
    }

    fn render(&self) -> VNode {
        match self.current_route {
            Route::Home => render_home(),
            Route::About => render_about(),
        }
    }
}
```

### 状态管理

#### Dioxus 状态

```rust
fn App() -> Element {
    let mut count = use_signal(|| 0);
    let mut text = use_signal(|| String::new());

    render! {
        div {
            button { onclick: move |_| count += 1, "{count}" }
            input {
                value: "{text}",
                oninput: move |e| text = e.value()
            }
        }
    }
}
```

#### Tairitsu 状态

```rust
use tairitsu_vdom::{Signal, VNode};

fn app() -> VNode {
    let count = Signal::new(0);
    let text = Signal::new(String::new());

    rsx! {
        div {
            button {
                onclick: move |_| {
                    let c = count.get();
                    count.set(c + 1);
                },
                ..txt(&format!("{}", count.get()))
            }
            input {
                value: text.get().clone(),
                oninput: move |e| {
                    if let Some(input) = e.downcast_ref::<tairitsu_vdom::InputEvent>() {
                        text.set(input.data.clone());
                    }
                }
            }
        }
    }
}
```

### 事件

#### Dioxus 事件

```rust
#[derive(Props, Clone, PartialEq)]
struct ButtonProps {
    onclick: EventHandler<MouseEvent>,
    children: Element,
}

fn Button(props: ButtonProps) -> Element {
    render! {
        button { onclick: props.onclick, {props.children} }
    }
}
```

#### Tairitsu 事件

```rust
use tairitsu_macros::component;
use tairitsu_vdom::{VNode, EventData, MouseEvent};

#[component]
fn Button(
    #[default] onclick: Option<Box<dyn FnMut(Box<dyn EventData>)>>,
    #[children] children: Vec<VNode>,
) -> VNode {
    rsx! {
        button {
            onclick: onclick,
            ..children
        }
    }
}
```

## 组件迁移

### Dioxus 组件

```rust
#[component]
fn Counter(
    #[props(default)] initial_value: i32,
) -> Element {
    let mut count = use_signal(|| initial_value);

    render! {
        div { class: "counter",
            h2 { "Counter: {count}" }
            button { onclick: move |_| count += 1, "Increment" }
            button { onclick: move |_| count -= 1, "Decrement" }
        }
    }
}
```

### Tairitsu 组件

```rust
use tairitsu_macros::component;
use tairitsu_vdom::{VNode, Signal};

#[component]
fn Counter(
    initial_value: i32,
    #[default] class: Option<String>,
) -> VNode {
    let count = Signal::new(initial_value);

    rsx! {
        div {
            class: class.unwrap_or_else(|| String::from("counter")),
            h2 { ..txt(&format!("Counter: {}", count.get())) }
            button {
                onclick: move |_| {
                    let c = count.get();
                    count.set(c + 1);
                },
                "Increment"
            }
            button {
                onclick: move |_| {
                    let c = count.get();
                    count.set(c - 1);
                },
                "Decrement"
            }
        }
    }
}

fn txt(value: &str) -> Vec<VNode> {
    vec![VNode::Text(tairitsu_vdom::VText::new(value))]
}
```

### Props 对照

| Dioxus | Tairitsu |
|--------|----------|
| `#[props(default)]` | `#[default]` |
| `#[props(!optional)]` | 无标记（必需） |
| `#[props(children)]` | `#[children]` |
| props 上的 `clone` | 自动生成的构建器 |

## Hooks 对照

### use_state -> Signal

```rust
// Dioxus
let mut count = use_signal(|| 0);
count.set(count + 1);

// Tairitsu
let count = Signal::new(0);
let c = count.get();
count.set(c + 1);
```

### use_effect -> create_effect

```rust
// Dioxus
use_effect(move || {
    println!("Count changed: {}", count());
});

// Tairitsu
create_effect(move || {
    let c = count.get();
    println!("Count changed: {}", c);
});
```

### use_memo -> Signal with derive

```rust
// Dioxus
let doubled = use_memo(move || count() * 2);

// Tairitsu - 手动计算
let doubled = {
    let count = count.clone();
    Signal::new({
        let c = count.get();
        c * 2
    })
};
```

### use_resource -> 手动异步

Tairitsu 没有内置的资源管理。您需要手动实现异步模式。

### use_coroutine -> 手动通道

Tairitsu 没有内置的协程支持。使用通道进行异步通信。

## 事件系统差异

### 事件类型映射

| Dioxus 事件 | Tairitsu 事件 |
|-------------|---------------|
| `MouseEvent` | `MouseEvent` |
| `KeyboardEvent` | `KeyboardEvent` |
| `InputEvent` | `InputEvent` |
| `FocusEvent` | `FocusEvent` |
| `FormData` | `ChangeEvent` |

### 事件处理器差异

```rust
// Dioxus
onclick: move |e: MouseEvent| {
    println!("Clicked at: ({}, {})", e.clientX(), e.clientY());
    e.stop_propagation();
}

// Tairitsu
onclick: move |e: Box<dyn EventData>| {
    if let Some(mouse) = e.downcast_ref::<MouseEvent>() {
        println!("Clicked at: ({}, {})", mouse.client_x, mouse.client_y);
    }
}
```

### 事件修饰符

Dioxus 支持事件修饰符如 `prevent_default`、`stop_propagation`。在 Tairitsu 中，这些在事件对象上调用：

```rust
// Dioxus
onclick: move |e: MouseEvent| {
    e.prevent_default();
}

// Tairitsu
onclick: move |e: Box<dyn EventData>| {
    if let Some(mouse) = e.downcast_ref::<MouseEvent>() {
        mouse.prevent_default();
    }
}
```

## 最小可运行示例

### Dioxus 最小示例

```rust
use dioxus::prelude::*;

fn main() {
    dioxus_web::launch(App);
}

fn App() -> Element {
    let mut count = use_signal(|| 0);

    render! {
        div {
            h1 { "Hello Dioxus!" }
            p { "Count: {count}" }
            button { onclick: move |_| count += 1, "Increment" }
        }
    }
}
```

### Tairitsu 最小示例

```rust
use tairitsu_vdom::{Signal, VNode, VText};
use tairitsu_macros::rsx;

fn main() {
    // 由 browser-glue 主机引导
}

#[no_mangle]
pub extern "C" fn tairitsu_component_bootstrap() {
    let platform = tairitsu_web::WitPlatform::new().expect("Failed to create platform");
    let app = App::new();
    let vnode = app.render();
    platform.mount_vnode_to_app(&vnode).expect("Failed to mount");
}

struct App {
    count: Signal<i32>,
}

impl App {
    fn new() -> Self {
        Self {
            count: Signal::new(0),
        }
    }

    fn render(&self) -> VNode {
        let count = self.count.clone();

        rsx! {
            div {
                class: "app-container",
                h1 { "Hello Tairitsu!" }
                p { ..txt(&format!("Count: {}", count.get())) }
                button {
                    onclick: move |_| {
                        let c = count.get();
                        count.set(c + 1);
                    },
                    "Increment"
                }
            }
        }
    }
}

fn txt(value: &str) -> Vec<VNode> {
    vec![VNode::Text(VText::new(value))]
}
```

### Tairitsu 的 Cargo.toml

```toml
[package]
name = "my-tairitsu-app"
version = "0.1.0"
edition = "2024"

[dependencies]
tairitsu-vdom = { path = "../../../packages/vdom" }
tairitsu-macros = { path = "../../../packages/macros" }
tairitsu-web = { path = "../../../packages/web", features = ["wit-bindings"] }
anyhow = "1.0"

[lib]
crate-type = ["cdylib", "rlib"]
```

## 常见迁移模式

### 条件渲染

```rust
// Dioxus
render! {
    div {
        {if show { render! { "Content" } } else { render! {} }}
    }
}

// Tairitsu
rsx! {
    div {
        ..if show {
            vec![VNode::Text(VText::new("Content"))]
        } else {
            vec![]
        }
    }
}
```

### 列表渲染

```rust
// Dioxus
let items = vec
![1, 2, 3];

render! {
    ul {
        {items.iter().map(|&item| render! {
            li { key: "{item}", "{item}" }
        })}
    }
}

// Tairitsu
let items = vec
![1, 2, 3];

let list_items: Vec<VNode> = items
    .iter()
    .map(|&item| {
        rsx! {
            li { key: item, ..txt(&item.to_string()) }
        }
    })
    .collect();

rsx! {
    ul { ..list_items }
}
```

### 类名条件

```rust
// Dioxus
render! {
    div { class: if active { "active" } else { "" }, "Content" }
}

// Tairitsu - 使用 Classes 构建器
use tairitsu_vdom::Classes;

let class = Classes::new()
    .add("base-class")
    .add_if("active", active);

rsx! {
    div { class: class, "Content" }
}
```

### 样式条件

```rust
// Dioxus
render! {
    div { style: "background-color: {color}", "Content" }
}

// Tairitsu - 使用 Style 构建器
use tairitsu_vdom::Style;

let style = Style::new()
    .add("background-color", &color)
    .add("padding", "16px");

rsx! {
    div { style: style, "Content" }
}
```

### 属性展开

```rust
// Dioxus
render! {
    div { ..attrs, "Content" }
}

// Tairitsu
rsx! {
    div {
        id: &attrs.id,
        class: &attrs.class,
        "Content"
    }
}
```

## 延伸阅读

- [快速开始指南](../quick-start.md)
- [系统概述](../../system/overview.md)
- [运行时与容器模型](../../system/runtime.md)
- [VDOM API 参考](../../components/packages.md#tairitsu-vdom)
