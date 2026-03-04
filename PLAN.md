# Tairitsu - 全栈 SaaS 服务框架

## 定位

**Tairitsu** 是一个基于 Rust WASM 的全栈 SaaS 服务框架，目标是让 Rust 的开发体验追上 Next.js 等现代框架，同时保持 Rust 的性能和安全性优势。

### 核心理念

```
传统框架                    Tairitsu
─────────────────────────────────────────────────────
JavaScript/TypeScript  →   Rust (类型安全 + 高性能)
复杂的构建配置          →   零配置开箱即用
运行时错误频繁          →   编译时错误检查
性能优化需要经验        →   WASM 原生高性能
安全漏洞常见            →   内存安全保证
```

### 与 Next.js 对比

| 特性 | Next.js | Tairitsu |
|------|---------|----------|
| 语言 | TypeScript | Rust |
| 类型安全 | 运行时 + 编译时（有限） | 编译时（完整） |
| 性能 | V8 优化 | WASM 原生 |
| 内存安全 | GC | 所有权系统 |
| 热重载 | ✅ | ✅ (计划中) |
| SSR | ✅ | ✅ (计划中) |
| API 路由 | ✅ | ✅ |
| 边缘部署 | ✅ | ✅ (计划中) |
| 组件生态 | 丰富 | Hikari Components |

## 架构愿景

```
┌─────────────────────────────────────────────────────────────────┐
│                    Tairitsu 全栈框架                             │
├─────────────────────────────────────────────────────────────────┤
│  开发体验 (DX)                                                   │
│  ├── 热重载 (Hot Reload)                                        │
│  ├── 类型安全 (Type Safety)                                     │
│  ├── 自动补全 (IDE Support)                                     │
│  └── 零配置 (Zero Config)                                       │
├─────────────────────────────────────────────────────────────────┤
│  前端 (Frontend)                                                 │
│  ├── 虚拟 DOM (Virtual DOM)                                     │
│  ├── 响应式系统 (Reactive System)                               │
│  ├── UI 组件库 (Hikari Components)                              │
│  └── 样式系统 (CSS Variables + Builders)                        │
├─────────────────────────────────────────────────────────────────┤
│  后端 (Backend)                                                  │
│  ├── WASM 容器运行时 (WASM Container Runtime) ← 现有功能        │
│  ├── 服务端渲染 (SSR)                                           │
│  ├── API 路由 (API Routes)                                      │
│  └── 数据库集成 (Database Integration)                          │
├─────────────────────────────────────────────────────────────────┤
│  部署 (Deployment)                                               │
│  ├── Edge Computing (边缘计算)                                  │
│  ├── Serverless (无服务器)                                      │
│  ├── Container (容器化)                                         │
│  └── Hybrid (混合部署)                                          │
├─────────────────────────────────────────────────────────────────┤
│  安全性 (Security)                                               │
│  ├── 内存安全 (Memory Safety)                                   │
│  ├── 类型安全 (Type Safety)                                     │
│  ├── 沙箱隔离 (Sandbox Isolation)                               │
│  └── 最小权限原则 (Least Privilege)                             │
└─────────────────────────────────────────────────────────────────┘
```

## 与 Hikari 的配合

### 项目关系

```
┌─────────────────────────────────────────────────────────────────┐
│                         应用层                                   │
│  hikari-website / 用户应用                                       │
├─────────────────────────────────────────────────────────────────┤
│                       组件层                                     │
│  hikari-components (Button, Card, Glow, ...)                    │
│  ├── 使用 tairitsu::vdom 渲染                                   │
│  ├── 使用 tairitsu::reactive 状态管理                           │
│  └── 使用 hikari-animation/hikari-palette 样式                  │
├─────────────────────────────────────────────────────────────────┤
│                       框架层                                     │
│  Tairitsu (本项目)                                               │
│  ├── vdom (虚拟 DOM + 平台抽象 + 响应式)                        │
│  ├── web (Web 渲染器)                                           │
│  ├── hooks (Hooks 系统)                                         │
│  └── macros (过程宏)                                            │
├─────────────────────────────────────────────────────────────────┤
│                       基础设施层                                 │
│  hikari-animation (动画系统)                                     │
│  hikari-palette (样式系统)                                       │
│  hikari-theme (主题系统)                                         │
└─────────────────────────────────────────────────────────────────┘
```

## 包结构

### 现有包 (保持不变)

```
packages/
├── runtime/          # WASM 容器运行时 (现有)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── container.rs
│       ├── image.rs
│       └── ...
│
└── macros/           # 过程宏 (现有，需扩展)
    ├── Cargo.toml
    └── src/
        └── lib.rs
```

### 新增包

```
packages/
├── vdom/             # 虚拟 DOM (核心包，包含平台抽象和响应式)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       │
│       │  # === 平台抽象层 ===
│       ├── platform/
│       │   ├── mod.rs
│       │   ├── trait.rs       # Platform trait
│       │   ├── element.rs     # ElementHandle trait
│       │   └── event.rs       # EventHandle trait
│       │
│       │  # === 响应式系统 ===
│       ├── reactive/
│       │   ├── mod.rs
│       │   ├── signal.rs      # Signal 实现
│       │   ├── effect.rs      # Effect 系统
│       │   ├── computed.rs    # 计算属性
│       │   └── scheduler.rs   # 更新调度
│       │
│       │  # === 虚拟 DOM ===
│       ├── vnode.rs           # VNode 定义
│       ├── velement.rs        # VElement 定义
│       ├── diff.rs            # Diff 算法
│       ├── patch.rs           # Patch 操作
│       └── attribute.rs       # 属性系统
│
├── web/              # Web 后端
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── platform.rs        # WebPlatform 实现
│       ├── dom.rs             # DOM 操作封装
│       ├── events.rs          # 事件系统
│       └── wrapper.rs         # web-sys wrapper
│
├── hooks/            # Hooks 系统
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── state.rs           # use_state
│       ├── signal.rs          # use_signal
│       ├── effect.rs          # use_effect
│       ├── style.rs           # use_style
│       └── animation.rs       # use_animation
│
└── macros/           # 过程宏 (扩展)
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        ├── rsx.rs             # rsx! 宏 (新增)
        ├── component.rs       # component 宏 (新增)
        └── wit.rs             # 现有 WIT 宏
```

## 核心设计

### 1. vdom 包 - 统一核心

```rust
// packages/vdom/src/lib.rs

pub mod platform;
pub mod reactive;
pub mod vnode;
pub mod diff;
pub mod patch;

// 重导出常用类型
pub use platform::{Platform, ElementHandle, EventHandle};
pub use reactive::{Signal, Effect, create_effect, batch};
pub use vnode::{VNode, VElement, VText, Style, Classes};
pub use patch::Patch;
```

### 2. 平台抽象 (vdom::platform)

```rust
// packages/vdom/src/platform/trait.rs

/// 平台抽象 trait
pub trait Platform: Sized + 'static {
    type Element: ElementHandle;
    type Event: EventHandle;
    
    fn create_element(&self, tag: &str) -> Self::Element;
    fn create_text_node(&self, text: &str) -> Self::Element;
    fn append_child(&self, parent: &Self::Element, child: &Self::Element);
    fn remove_child(&self, parent: &Self::Element, child: &Self::Element);
    fn set_attribute(&self, element: &Self::Element, name: &str, value: &str);
    fn remove_attribute(&self, element: &Self::Element, name: &str);
    fn set_style(&self, element: &Self::Element, name: &str, value: &str);
    fn set_class(&self, element: &Self::Element, class: &str);
    fn add_event_listener(&self, element: &Self::Element, event: &str, handler: Box<dyn FnMut()>);
    fn remove_event_listener(&self, element: &Self::Element, event: &str);
}

/// 元素句柄 trait
pub trait ElementHandle: Clone + 'static {
    fn as_any(&self) -> &dyn std::any::Any;
}

/// 事件句柄 trait  
pub trait EventHandle: 'static {
    fn as_any(&self) -> &dyn std::any::Any;
}
```

### 3. 响应式系统 (vdom::reactive)

```rust
// packages/vdom/src/reactive/signal.rs

/// 响应式信号
pub struct Signal<T> {
    inner: Rc<RefCell<SignalInner<T>>>,
}

impl<T: Clone> Signal<T> {
    pub fn new(value: T) -> Self { ... }
    pub fn get(&self) -> T { ... }
    pub fn set(&self, value: T) { ... }
}

// packages/vdom/src/reactive/effect.rs

/// Effect - 自动追踪依赖
pub fn create_effect<F>(f: F) -> EffectHandle
where
    F: FnMut() + 'static,
{
    // 自动追踪 Signal 依赖
}

/// 批量更新
pub fn batch<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    // 合并多个 Signal 更新
}
```

### 4. 虚拟 DOM (vdom)

```rust
// packages/vdom/src/vnode.rs

/// 虚拟节点
pub enum VNode {
    Element(VElement),
    Text(VText),
    Fragment(Vec<VNode>),
    Component(VComponent),
}

/// 虚拟元素
pub struct VElement {
    pub tag: &'static str,
    pub key: Option<Key>,
    pub attributes: Attributes,
    pub children: Vec<VNode>,
    pub style: Style,
    pub class: Classes,
    pub event_handlers: EventHandlers,
}

/// 样式系统 - 支持 StyleBuilder 集成
pub struct Style {
    pub static_styles: String,
    pub css_variables: Vec<(&'static str, String)>,
}

/// 类名系统 - 支持 ClassesBuilder 集成
pub struct Classes {
    pub static_classes: String,
}

// packages/vdom/src/patch.rs

/// Diff 结果
pub enum Patch {
    CreateElement { id: NodeId, tag: &'static str, attrs: Attributes, style: Style, class: Classes },
    RemoveElement { id: NodeId },
    UpdateAttribute { id: NodeId, name: &'static str, value: String },
    UpdateStyle { id: NodeId, name: &'static str, value: String },
    UpdateClass { id: NodeId, class: String },
    InsertChild { parent_id: NodeId, child: VNode, index: usize },
    RemoveChild { parent_id: NodeId, index: usize },
}
```

### 5. Web 后端 (web 包)

```rust
// packages/web/src/platform.rs

#[cfg(feature = "web")]
pub struct WebPlatform {
    document: web_sys::Document,
}

#[cfg(feature = "web")]
impl Platform for WebPlatform {
    type Element = web_sys::Element;
    type Event = web_sys::Event;
    
    fn create_element(&self, tag: &str) -> Self::Element {
        self.document.create_element(tag).unwrap()
    }
    
    fn set_style(&self, element: &Self::Element, name: &str, value: &str) {
        element.dyn_ref::<web_sys::HtmlElement>()
            .unwrap()
            .style()
            .set_property(name, value)
            .unwrap();
    }
    
    fn set_class(&self, element: &Self::Element, class: &str) {
        element.set_attribute("class", class).unwrap();
    }
}
```

### 6. rsx! 宏 (macros 包)

```rust
/// rsx! 宏示例
rsx! {
    div {
        class: ClassesBuilder::new()
            .add("my-class")
            .add_if("active", || is_active)
            .build(),
        style: StyleBuilder::build_string(|s| {
            s.add_custom("--glow-intensity", "0.5")
             .add(CssProperty::Width, "100px")
        }),
        onclick: move |_| count += 1,
        "Hello, {name}!"
    }
}
```

## Feature Flags

```toml
[features]
default = ["native"]

# 平台支持
native = ["wasmtime", "wasmtime-wasi", "wit-component", "wit-parser"]
web = ["web-sys", "js-sys", "wasm-bindgen"]

# 功能模块
hooks = []
dev-tools = []
```

## 依赖关系图

```
tairitsu (主包)
├── tairitsu-vdom (核心: 平台抽象 + 响应式 + 虚拟DOM)
│   └── (无平台依赖)
│
├── tairitsu-web (Web 平台) [feature = "web"]
│   ├── tairitsu-vdom
│   ├── web-sys
│   └── js-sys
│
├── tairitsu-native (Native 平台) [feature = "native"]
│   ├── tairitsu-vdom
│   ├── wasmtime
│   └── wasmtime-wasi
│
├── tairitsu-hooks (Hooks)
│   └── tairitsu-vdom
│
└── tairitsu-macros (过程宏)
    └── proc-macro2, quote, syn
```

## 与 Hikari 的接口约定

### StyleBuilder 集成

```rust
// hikari-animation 中的 StyleBuilder
impl StyleBuilder<'_> {
    pub fn build_for_vdom(self) -> tairitsu_vdom::Style {
        tairitsu_vdom::Style {
            static_styles: self.build(),
            css_variables: self.css_variables,
        }
    }
}
```

### ClassesBuilder 集成

```rust
// hikari-palette 中的 ClassesBuilder
impl ClassesBuilder {
    pub fn build_for_vdom(self) -> tairitsu_vdom::Classes {
        tairitsu_vdom::Classes {
            static_classes: self.build(),
        }
    }
}
```

### 组件示例

```rust
// hikari-components 中的 Glow 组件
use tairitsu::{rsx, VNode, Signal, create_effect};
use hikari_animation::StyleBuilder;
use hikari_palette::ClassesBuilder;

pub fn Glow(props: GlowProps) -> VNode {
    let intensity = Signal::new(0.0);
    
    rsx! {
        div {
            class: ClassesBuilder::new()
                .add("hi-glow-wrapper")
                .add("hi-glow-soft")
                .build_for_vdom(),
            style: StyleBuilder::new()
                .add_custom("--glow-intensity-scale", &intensity.get().to_string())
                .build_for_vdom(),
            onmousedown: move |_| intensity.set(1.0),
            onmouseup: move |_| intensity.set(0.5),
            {props.children}
        }
    }
}
```

## 开发计划

### Phase 1: 核心基础 (2-3 周)

1. **tairitsu-vdom**
   - [ ] 平台抽象 trait (platform/)
   - [ ] 响应式系统 (reactive/)
   - [ ] VNode/VElement 定义
   - [ ] 基础 Diff 算法
   - [ ] Patch 系统

### Phase 2: Web 后端 (1-2 周)

1. **tairitsu-web**
   - [ ] WebPlatform 实现
   - [ ] DOM 操作封装
   - [ ] 事件系统
   - [ ] web-sys wrapper

### Phase 3: 宏系统 (1-2 周)

1. **tairitsu-macros**
   - [ ] rsx! 宏解析器
   - [ ] 代码生成

### Phase 4: Hooks (1 周)

1. **tairitsu-hooks**
   - [ ] use_state
   - [ ] use_signal
   - [ ] use_effect
   - [ ] use_style

### Phase 5: 集成测试 (2 周)

1. **与 Hikari 集成**
   - [ ] 迁移 Glow 组件
   - [ ] 迁移 Button 组件
   - [ ] 性能测试

## 下一步行动

1. **创建包结构**: 在 `packages/` 下创建新包目录
2. **实现 tairitsu-vdom**: 平台抽象 + 响应式 + 虚拟DOM
3. **实现 tairitsu-web**: WebPlatform
4. **实现 tairitsu-macros**: rsx! 宏
5. **编写测试**: 确保核心功能正确

---

*此计划与 Hikari 的 PLAN.md 配合使用，两边同步开发。*
