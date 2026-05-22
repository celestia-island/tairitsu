use tairitsu_macros::rsx;
use tairitsu_vdom::VNode;

use crate::components::breadcrumb;
use crate::markdown::markdown_content;

const GLOSSARY_MD: &str = r#"
Tairitsu 项目中使用的术语和概念说明。

### 核心概念

#### Image（镜像）

不可变的 WASM 组件二进制及其元数据。类似于 Docker 镜像的概念，包含了运行组件所需的所有信息。

#### Container（容器）

运行中的组件实例。封装了 store、linker 和调用上下文，类似于 Docker 容器。

#### WIT（WebAssembly Interface Types）

WebAssembly 接口类型定义语言。用于描述组件之间的接口契约，确保类型安全的跨语言互操作。

#### VDOM（Virtual DOM）

虚拟 DOM。一种 UI 渲染优化技术，通过在内存中维护 DOM 树的轻量级表示来减少实际 DOM 操作。

### 构建相关

#### Component Model

WebAssembly 组件模型。一种标准化的 WASM 模块组合和交互方式，支持跨语言互操作。

#### wasm32-wasip2

WebAssembly System Interface Preview 2 目标平台。支持组件模型的 WASI 版本。

#### Packager（打包器）

Tairitsu 的构建工具。负责编译 WASM 组件、生成宿主资源和管理开发服务器。

### UI 相关

#### rsx! 宏

声明式 UI 语法宏。类似于 JSX，允许用 Rust 代码编写类似 HTML 的 UI 结构。

#### Signal（信号）

响应式状态管理原语。当信号值变化时，自动触发依赖它的 UI 更新。

#### Hook

React Hooks 风格的状态管理函数。如 `use_state`、`use_effect` 等，用于在组件中管理状态和副作用。
"#;

pub fn render() -> VNode {
    rsx! {
        div { id: "page-guides-glossary", class: "hikari-page",
            ..vec![breadcrumb(&[("Home", "/"), ("Guides", "/guides"), ("Glossary", "")])]
            section { class: "page-section",
                h2 { class: "page-section__title",
                    "术语对照表"
                }
                ..vec![markdown_content(GLOSSARY_MD)]
            }
        }
    }
}
