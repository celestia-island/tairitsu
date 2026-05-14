# Tairitsu 改进计划

> 架构审计 + 竞品横评反馈的综合改造路线图
> 最后更新: 2026-05-14

---

## 已完成

### ✅ T-1: VElement 类型化事件便利方法 (2026-05-14)

在 `packages/vdom/src/vnode.rs` 新增 15 个方法，委托到 `on_event()` 自动 downcast：

- `on_click` / `on_dblclick` → `MouseEvent`
- `on_mousedown` / `on_mouseup` / `on_mousemove` / `on_mouseenter` / `on_mouseleave` → `MouseEvent`
- `on_keydown` / `on_keyup` / `on_keypress` → `KeyboardEvent`
- `on_input` → `InputEvent`
- `on_change` → `ChangeEvent`
- `on_focus` / `on_blur` → `FocusEvent`
- `on_submit` → `ChangeEvent`

编译 + 64 个测试通过。

### ✅ T-1b: 补全剩余事件类型化方法 (2026-05-14)

新增 22 个方法：
- `on_wheel` → `WheelEvent`
- `on_dragstart` / `on_dragend` / `on_dragover` / `on_dragleave` / `on_drop` → `DragEvent`
- `on_touchstart` / `on_touchmove` / `on_touchend` / `on_touchcancel` → `TouchEvent`
- `on_pointerdown` / `on_pointerup` / `on_pointermove` / `on_pointerenter` / `on_pointerleave` / `on_pointerover` / `on_pointerout` → `PointerEvent`
- `on_transitionend` → `TransitionEvent`
- `on_animationstart` / `on_animationend` / `on_animationiteration` → `AnimationEvent`
- `on_scroll` → `GenericEvent`

### ✅ T-2: VElement CSS 自定义属性便利方法 (2026-05-14)

添加 `with_css_var(name, value)` 方法，自动补 `--` 前缀，免去手动构建 Style。

### ✅ T-6: Platform Trait 拆分 (2026-05-14)

将 65+ 方法的 `Platform` trait 拆分为 14 个子 trait：
- **DomOps** (base): `Element`/`Event` 关联类型 + DOM 操作
- **TimerOps**: `set_timeout`/`set_interval`/`request_animation_frame`
- **LayoutOps**: `get_bounding_client_rect`/`inner_width`/`scroll_*`
- **ObserverOps**: ResizeObserver/MutationObserver
- **MediaQueryOps**: `match_media`/`media_query_list_*`
- **ClipboardOps**: `copy_to_clipboard`/`clipboard_*_async`
- **ContentEditableOps**: `exec_command`/`get_selection_*`
- **ScrollOps**: `scroll_to`/`on_scroll`/`on_resize`/`prefers_dark_mode`
- **QueryOps**: `get_element_by_id`/`query_selector`/`element_from_point`
- **CanvasOps**: Canvas 2D context 操作
- **MediaOps**: Video/Audio 播放与分析
- **GeoOps**: 地理位置
- **FileOps**: FileReader
- **IdbOps**: IndexedDB

`Platform` 保持为 super-trait + blanket impl，完全向后兼容。需要 `Element` 的子 trait 继承 `DomOps`。

### ✅ T-5: 统一 Reactive 系统 (2026-05-14)

- `Scheduler` 标记为 `#[deprecated]`（生产环境只用 `runtime.rs`）
- `runtime.rs` 为唯一调度器，`Scheduler` 将在 0.5.0 移除

### ✅ T-7: 事件监听器选项 (2026-05-14)

- 新增 `ListenerOptions { passive, once, capture }` 结构
- `DomOps` 新增 `add_event_listener_with_options` 方法
- WIT 层预留 TODO 待上游支持

### ✅ T-P3b: SubmitEvent 专用类型 (2026-05-14)

- `SubmitEvent` 独立于 `FormEvent`，包含 `form_data` 字段
- `on_submit` 使用 `SubmitEvent::from_event_data` downcast

### ✅ T-P3c: 事件 target/current_target 扩展 (2026-05-14)

- 12 个事件类型补全 `target`/`current_target` 字段
- WIT 层回调中提取 `get_target()`/`get_current_target()` 传递给事件对象

---

## 第一阶段：重新定位（Branding & Identity）

> **核心问题**：Tairitsu 当前对外叙事是"Generic WASM Component Runtime Engine"，但实际上代码库的 80% 是 Web 框架（VDOM/hooks/SSR/macros）。Runtime 层只有 ~3000 行。这种叙事与实际的不匹配导致两套用户群体都无法正确定位项目。
>
> **洞察**：Tairitsu 的矛盾不是设计缺陷，而是 WASM Component Model 天然携带的 dual-target 特性 —— 同一个 .wasm 组件可以在服务端（Container/Registry）运行，也可以在客户端（浏览器 VDOM）运行。这正是它唯一的技术护城河，应该从 bug 叙事翻转为 feature 叙事。

### B-1: 重新定义项目定位语

| 当前 | 目标 |
|------|------|
| **Generic WASM Component Runtime Engine** | **Full-stack framework powered by the WASM Component Model** |
| 暗示：仅服务端运行时 | 暗示：服务端 + 客户端一体化 |
| 对标：wasmtime/wasmCloud | 对标：组合了 Leptos/Dioxus + wasmCloud 的能力 |

副标题：*Write components once, run them anywhere — server, browser, edge. All communication typed via WIT.*

### B-2: 对外三层 API 分层命名

当前 `tairitsu` crate 同时是 runtime 引擎又 re-export 所有宏，导致不同场景的用户被无关符号污染。

目标结构（0.5.0）：

```
┌──────────────────────────────────────────────────────────┐
│  tairitsu (umbrella, 全栈用户)                            │
│                                                          │
│  📦 tairitsu-core    → 服务端运行时                       │
│     Container / Registry / WIT binding / RON / Binary     │
│     对标：wasmCloud, Spin, wasmtime 裸用                  │
│                                                          │
│  🎨 tairitsu-web     → 客户端框架 & SSR 渲染              │
│     VDOM / hooks / rsx! / scss! / SSR / Suspense         │
│     对标：Leptos, Dioxus, Yew                             │
│                                                          │
│  🔧 tairitsu-cli     → 构建 / 调试 / 部署工具              │
│     packager, dev server, MCP, visual diff, VTty          │
│     对标：Vite, Trunk, dioxus-cli                         │
└──────────────────────────────────────────────────────────┘
```

具体操作：
- `tairitsu` crate 保留为 umbrella（依赖 core + web + ssr），不做 breaking change
- 新增 `tairitsu-core` crate 别名（`packages/runtime` 的对外名称）
- `tairitsu-web` 保持现有结构，补充文档
- `tairitsu-packager` 提供 `tairitsu` CLI binary

### B-3: 统一 Hello World 全栈示例

创建 `examples/todo-fullstack/`，展示同一个 WIT 接口在三种环境运行：

```
examples/todo-fullstack/
├── wit/
│   └── todo.wit          # 统一的 WIT 接口定义
├── src/
│   ├── lib.rs            # 核心组件（可在 server/client 两端编译）
│   ├── server.rs         # 服务端 Container 入口
│   └── client.rs         # 客户端 VDOM 入口
├── Cargo.toml
└── README.md
```

核心示例展示：
```rust
// 同一个组件，同一套代码
#[component]
fn TodoApp() -> VNode {
    rsx! {
        div { class: "todo-app",
            Suspense {
                fallback: rsx! { "Loading..." },
                TodoList {}  // 在 client 环境走 fetch, server 环境直接调 WIT
            }
        }
    }
}
```

命令行展示三个执行路径：
- `tairitsu dev` → dev server + 浏览器 VDOM 渲染
- `tairitsu serve` → 生产 SSR 渲染（Container 运行时）
- `tairitsu build` → 编译为独立 WASM 组件，放入任意 wasmtime 环境

---

## 第二阶段：稳定性与开发者体验（P0 – 必须）

### P0-1: 修复事件系统的 unsafe extern "C" hack

**当前问题**（`packages/vdom/src/events.rs:35-42`）：
```rust
unsafe {
    unsafe extern "C" {
        fn tairitsu_prevent_default(event_handle: u64);
    }
    tairitsu_prevent_default(handle);
}
```
因为 WIT 层面尚未支持事件监听器选项，`preventDefault`/`stopPropagation` 通过 weak symbol 与 TypeScript 胶水层通信，脆弱且不可移植。

**方案**：
- **短期**（0.5.0）：在 WIT 中显式定义事件控制接口，browser-glue 的 TS 侧通过 canonical ABI 调用实现，移除所有 unsafe extern "C"
- **长期**（0.6+）：等 W3C WebIDL → WIT pipeline 自动包含 PreventDefault()/StopPropagation() 方法后，迁移到标准 WIT

### P0-2: 实现 `tairitsu new` / `tairitsu init` 脚手架

当前 README 的 "Quick Start" 只有一段 `Container::builder()` 代码片段，对前端用户完全无入口。

需求：
```bash
tairitsu new my-app          # 全栈模板（含 server + client 编译目标）
tairitsu new my-app --web    # 纯前端模板（仅 client wasm target）
tairitsu new my-app --server # 纯服务端模板（仅 server native target）
tairitsu new my-app --example todo  # 从内置模板创建
```

模板应预置：
- 标准目录结构
- `Cargo.toml` 含 `[profile.dev-wasm]`
- `src/lib.rs` 引导代码（`WitPlatform::new()` + `mount_vnode_to_app()`）
- `.gitignore`
- `justfile`（含 `just dev` / `just build` / `just test`）

### P0-3: 降级到 stable Rust + Edition 2021

当前状态：
- `edition = "2024"` — 在 Rust 生态中极其小众
- `rustfmt` 需要 nightly toolchain
- 目标 `wasm32-wasip2` 本身就非 stable

**降级收益**：
- `cargo fmt` 不再需要特殊 toolchain 配置
- CI 可以去掉 nightly 相关步骤
- 降低首次使用者的心智负担
- Edition 2021 完全满足当前代码需求

**迁移步骤**：
1. 全局修改 `edition = "2024"` → `edition = "2021"`
2. 检查 `gen` keyword、`use<..>` lifetime capture syntax 等 2024 特有语法，如有则移除
3. 移除 CI 中的 nightly toolchain 安装步骤（保留 `wasm32-wasip2` target 安装）
4. 更新 `just fmt` recipe，使用 stable rustfmt

---

## 第三阶段：架构精耕（P1 – 重要）

### P1-1: 39 个 npm 子包合并为单一 `@tairitsu/browser-glue`

**当前状态**：`packages/npm/` 下有 39 个 per-domain 子包（`glue-dom`, `glue-fetch`, `glue-canvas`, `glue-animation`...）。

**问题**：
- 每个子包需要独立维护、版本管理、发布
- 版本不同步的风险极高
- 用户需要安装几十个包或用具名 import

**方案**：
```
@tairitsu/browser-glue         # 唯一对外 npm 包
├── src/
│   ├── dom/                   # 内部模块，对应原 glue-dom
│   ├── fetch/                 # 内部模块，对应原 glue-fetch
│   ├── canvas/                # 内部模块，对应原 glue-canvas
│   └── ...
├── package.json
└── tsconfig.json
```

- 通过 TypeScript path mapping 组织内部模块
- 利用 esbuild/rollup tree-shaking 保证 bundle 体积
- 向后兼容：保留 39 个子包作为 `@tairitsu/browser-glue` 的 re-export
- 0.6.0 时废弃旧子包

### P1-2: VDOM diff 算法升级

**当前状态**：`packages/vdom/src/diff.rs` (756 行) 实现基础 O(n) children diff，仅通过元素索引匹配。

**问题**：
- 动态列表（如 TodoMVC 的增删改）会导致过多 DOM 操作
- 缺少 keyed reconciliation（通过 key 做最小移动）
- SSR fast-refresh 路径也有相关性能瓶颈

**方案**：
1. 在 diff 层增加 `diff_keyed_children` 函数，实现标准的 keyed diff（参考 React reconcileChildrenArray / Vue2 双端对比）
2. 添加 feature flag `keyed-diff` 让用户选择
3. 同时将 diff 拆分为独立模块：`diff_nodes.rs`, `diff_children.rs`, `diff_attributes.rs`

### P1-3: 事件系统剩余补全

WIT 层中的 TODO：
- `SubmitEvent.form_data` 在 browser-glue 中需要实现
- `ListenerOptions` (passive/once/capture) 在 WIT 层标记待上游支持 —— 影响滚动性能
- `WheelEvent` 缺少 `deltaX/Y/Z` 的标准化 WIT 类型映射
- `TransitionEvent` 缺少 `elapsedTime` / `propertyName` 字段

### P1-4: WIT generation pipeline 去重

**当前状态**：两条并行的 WIT 生成管线：
1. `scripts/gen_wit_from_webidl.py` — 主管线，fetch 50+ WebIDL specs
2. `scripts/gen-wit-all` — 备用管线（simpler, fewer specs, idl-cache/）

**方案**：合并为单管线，通过 CLI 参数控制 spec 范围。

---

## 第四阶段：工程质量（P2 – 应该）

### P2-1: 移除 Scheduler 双轨制

T-5 已标记 `packages/vdom/src/scheduler.rs` (760 行) 为 deprecated，0.5.0 中彻底移除。所有调度逻辑统一到 `packages/vdom/src/runtime.rs`。

### P2-2: 测试覆盖率提升

运行 `cargo llvm-cov` 或 `cargo tarpaulin` 获取基准，然后逐步补充：

| 模块 | 当前问题 | 目标 |
|------|---------|------|
| `vdom/diff.rs` (756行) | 仅有集成测试，缺少 keyed/fragment/attribute 单测 | 80%+ 行覆盖 |
| `runtime/dynamic/deserialize.rs` (385行) | 缺少复杂嵌套类型 RON 反序列化测试 | 90%+ 行覆盖 |
| `ssr/streaming.rs` (280行) | 缺少 streaming SSR 集成测试 | 核心路径覆盖 |
| `ssr/error_overlay/` (525+472行) | 缺少错误模板渲染测试 | 关键模板覆盖 |

### P2-3: 补充对外 API 文档（rustdoc）

所有 `pub` 类型的核心方法应当在对应模块的 `//!` 注释中有 rustdoc 示例：
- `Container::builder()` / `Container::call_guest_raw_desc()`
- `VNode` / `VElement` 构造方法
- `Signal::new()` / `Signal::get()` / `Signal::set()`
- `rsx!` 宏的各种语法（children、属性、事件、条件渲染、列表渲染）
- `scss!` / `include_scss!` / `svg!` 宏的用法

---

## 第五阶段：文档体系（P2 – 应该）

### D-1: 重构英文文档入口层次

当前的 `docs/en/` 仅有骨架，多篇核心文档为 stub（5-18 行）。

目标结构：
```
docs/en/
├── index.md                    # 主入口（重写）
├── guides/
│   ├── index.md                # 导航枢纽（重写）
│   ├── getting-started.md      # 从零开始的完整教程（新增）
│   ├── quick-start.md          # 快速体验（重写）
│   ├── workspace-map.md        # 仓库导览
│   ├── build-test-release.md   # 构建与发布
│   ├── migration/
│   │   └── dioxus-to-tairitsu.md
│   ├── troubleshooting.md
│   └── glossary.md
├── system/
│   ├── overview.md             # 系统架构全景（重写）
│   ├── runtime.md              # Container/Registry 模型
│   ├── vdom.md                 # VDOM 渲染引擎（新增）
│   ├── wit-pipeline.md         # WIT 代码生成管线
│   ├── web-backends.md         # 双后端策略
│   ├── browser-glue.md         # 浏览器胶水层
│   └── versioning.md           # 版本策略
├── components/
│   ├── index.md                # 分层包清单（重写）
│   └── packages.md             # 逐包详解
├── skills/
│   └── debug-agent.md
└── enterprise/
    └── support.md
```

### D-2: 新增开发者教程文档

`docs/en/guides/getting-started.md`：
- 第一章：为什么选择 Tairitsu（WASM Component Model 的 dual-target 优势）
- 第二章：安装与脚手架（`tairitsu new`）
- 第三章：你的第一个全栈组件（rsx! + Signal + WIT interface）
- 第四章：在浏览器中运行（`tairitsu dev`）
- 第五章：在服务端渲染（`tairitsu serve`）
- 第六章：部署到 Registry（`tairitsu deploy`）

整个教程围绕一个逐步构建的 "Guestbook" 示例展开，展示同一个组件在 client/server 两端的行为。

---

## 第六阶段：渲染性能 — 细粒度响应式更新（P1）✅ 核心已完成，2 个 gap 待补

> **审计结论（2026-05-14 Hikari 锐评后全面复验）**: DynamicText/DynamicAttr/DynamicClass 的完整链路已经实现并可用。Hikari 已新增 Signal 驱动的 Reactive Counter 演示组件验证该链路。
>
> **完整链路**: `Signal<T>` → `IntoVNodeChild` → `VNode::DynamicText` → `render_vnode` 挂载 `create_effect` → signal 变化时直接 `set_text_content`（不经过 VDOM diff）

### 已完成的实现清单

| 层级 | 实现 | 代码位置 |
|------|------|----------|
| `Signal<T>` + 自动依赖追踪 | `Signal::get()` 推入 `DEPENDENCIES`，`create_effect` 自动订阅 | `vdom/src/reactive/mod.rs` |
| `VNode::DynamicText` 变体 | `initial: String` + `compute: Rc<RefCell<dyn FnMut() -> String>>` | `vdom/src/vnode.rs` |
| `VElement.dynamic_attributes` | `Vec<(String, DynamicCompute)>` + `.dynamic_attr()` builder | `vdom/src/vnode.rs` |
| `VElement.dynamic_styles` | `Vec<(String, DynamicCompute)>` + `.dynamic_style()` builder | `vdom/src/vnode.rs` |
| `VElement.dynamic_classes` | `Vec<DynamicCompute>` + `.dynamic_class()` builder | `vdom/src/vnode.rs` |
| `IntoVNodeChild for Signal<T>` | `rsx!{ {signal} }` 自动生成 DynamicText | `vdom/src/vnode.rs` |
| `Dyn<F>` wrapper | 手动标记属性为 dynamic：`.attr("name", Dyn(compute))` | `vdom/src/vnode.rs` |
| `use_dynamic_text()` hook | `Signal<T>` → `DynamicText` VNode | `hooks/src/dynamic.rs` |
| diff 跳过 DynamicText | `DynamicText vs DynamicText → continue`（零 patch） | `vdom/src/diff.rs` |
| 挂载时创建 effect | `render_vnode` 中为 DynamicText/Attr/Style/Class 调用 `create_effect` | `web/src/wit_platform.rs:2010-2061` |
| Hikari Platform 层复活 | 30+ stub 函数改为委托到 `Box<dyn Fn>` 函数指针表 | `hikari/packages/components/src/platform/` |

### Gap 1: rsx!{} 属性位置不自动生成 Dynamic — ✅ 已修复

**实现**: 为 `IntoDynamicAttr`、`IntoClassValue`、`IntoStyleValue` 三个 trait 补充了 `Signal<T>` 实现。Rust 的 trait dispatch 自动处理——当属性值类型是 `Signal<T>` 时，编译器选择 Signal 版本的 impl，生成 `dynamic_attributes`/`dynamic_classes`/`dynamic_styles`，挂载时通过 `create_effect` 实现细粒度更新。

**代码位置**: `packages/vdom/src/vnode.rs` — `impl IntoDynamicAttr for Signal<T>`、`impl IntoClassValue for Signal<T>`、`impl IntoStyleValue for Signal<T>`

**额外修复**: 补充 `IntoDynamicAttr` for `bool` 和 `&String` impl，消除 Hikari website 中的 3 个编译错误。

### Gap 2: create_effect EffectHandle 泄漏 — ✅ 已修复

**实现**:
1. `runtime.rs` 新增 `effect_handles: HashMap<ComponentId, Vec<EffectHandle>>` + `register_effect_handle()` 公开函数
2. `EffectHandle` 实现 `Clone`（内部仅 `Rc<Cell<bool>>`）
3. `wit_platform.rs` 新增 `create_tracked_effect()` 辅助函数，内部通过 `CURRENT_RENDER_COMPONENT` thread-local 将 handle 注册到当前组件
4. `runtime_integration.rs` 的 `apply_patches` 回调通过 `with_render_component(id, ...)` 设置渲染上下文
5. `cleanup_component(id)` 批量 `stop()` 所有 handle
6. 全部 9 处 `tairitsu_vdom::create_effect` 调用替换为 `create_tracked_effect`

**代码位置**:
- `packages/vdom/src/runtime.rs` — `effect_handles` 字段 + `register_effect_handle()`
- `packages/vdom/src/reactive/mod.rs` — `impl Clone for EffectHandle`
- `packages/web/src/wit_platform.rs` — `CURRENT_RENDER_COMPONENT` thread-local + `create_tracked_effect()`
- `packages/web/src/runtime_integration.rs` — `with_render_component()` 包裹 `apply_patches`

---

## 架构决策记录

### AD-1: 事件便利方法实现策略
- **决策**: 在 VElement impl 块中直接添加方法，内部调用 `on_event()` + downcast
- **原因**: 最小侵入，不修改 trait 或 WIT 层
- **替代方案**: 宏生成（减少重复代码，但降低可读性）
- **后续**: 当方法数量超过 25 个时考虑重构为宏

### AD-2: 项目身份叙事策略 (2026-05-14)
- **决策**: 将定位从 "Generic WASM Runtime Engine" 改为 "Full-stack WASM Component Model Framework"
- **原因**: 代码库中 Web 框架部分占 80%，容器运行时仅 3000 行。Runtime 是基础设施而非用户面
- **核心叙事**: Tairitsu = WASM Component Model 的 dual-target 特性（同一组件在 server/client 两端运行）作为唯一护城河
- **替代方案**: 拆分为两个独立项目（放弃全栈叙事优势）

### AD-3: crate 分层命名 (2026-05-14)
- **决策**: 保留 `tairitsu` 为 umbrella，新增 `tairitsu-core` 别名，强化 `tairitsu-web` / `tairitsu-cli` 定位
- **原因**: 不 breaking change 的前提下，让不同场景用户看到清晰的命名
- **替代方案**: 激进重命名所有 crate（breaking every user）

### AD-4: 细粒度响应式采用 Hybrid 策略而非纯 fine-grained (2026-05-14)
- **决策**: 保持 VDOM + Signal 混合模型，只在 leaf level 做 DynamicText/DynamicAttr 细粒度更新
- **原因**: VDOM 对 Portal/条件渲染/SSR/列表 reconciliation 仍然有价值，完全去掉得不偿失
- **参考**: Dioxus 2024+ 也采用相同 hybrid 策略（VDOM 管结构，Signal 管值）
- **替代方案**: 改为 Leptos 式纯 fine-grained（需重写整个 diff/patch 系统，代价太大）

### AD-5: Dynamic VNode 在 vdom crate 中实现 (2026-05-14)
- **决策**: DynamicText/DynamicAttr/DynamicClass 作为 VNode 变体在 `tairitsu-vdom` 中实现
- **原因**: 这是渲染引擎的基础能力，所有基于 Tairitsu 的框架（Hikari 及未来的）都需要
- **影响**: `tairitsu-macros` 的 `rsx!{}` 宏需要同步改造以自动生成 Dynamic 变体

### AD-6: Hikari Platform 层采用 Box<dyn Fn> 函数指针表 (2026-05-14)
- **决策**: Hikari 的 `platform::wit.rs` 不直接依赖 `WitPlatform`，而是通过 `Box<dyn Fn(Args) -> Ret>` 闭包表间接调用
- **原因**: Platform trait 有关联类型 `type Element: ElementHandle`，不满足 dyn-safe；`WitPlatform` 的 trait impl 仅在 `#[cfg(target_family = "wasm")]` 下编译，native build 会失败
- **影响**: 每个 Hikari 应用入口需调用 `platform_init::register(&platform)` 一次注册

---

## 进度总览

| 阶段 | 任务 | 状态 |
|------|------|------|
| — | T-1 ~ T-7 | ✅ 全部完成 |
| 一 | B-1: 定位语改为 Full-stack Framework | ✅ 完成 (README + docs repositioned) |
| 一 | B-2: 三层 API 分层命名 | ✅ 完成 (tairitsu → tairitsu-core, 0.5.0 breaking) |
| 一 | B-3: todo-fullstack 全栈示例 | ✅ 完成 (examples/todo-app with rsx! + Signal + 7 tests) |
| 二 | P0-1: 修复 unsafe extern "C" 事件 hack | ✅ 完成 (injected callback pattern) |
| 二 | P0-2: tairitsu new/init 脚手架 | ✅ 完成 (rsx! + Signal + component template) |
| 二 | P0-3: 降级 Edition 2021 + stable toolchain | ✅ 完成 (50+ let-chain refactors, CI updated) |
| 三 | P1-1: 39→1 npm 包合并 | ✅ 完成 (34 glue-* → browser-glue, glue-full deprecated shim) |
| 三 | P1-2: VDOM diff 算法升级 | ✅ 已有 (LIS-based keyed reconciliation with 15 tests) |
| 三 | P1-3: 事件系统剩余补全 | ✅ 完成 (ListenerOptions.capture wired, SubmitEvent added, ChangeEvent enhanced) |
| 三 | P1-4: WIT generation pipeline 去重 | ✅ 完成 (removed 4 deprecated justfile aliases, single pipeline remains) |
| **六** | **P1-5: DynamicText VNode 变体** | ✅ 完成 (effect-based fine-grained text update) |
| **六** | **P1-6: DynamicAttr/DynamicClass 绑定** | ✅ 完成 (VElement dynamic_* fields + platform effects) |
| **六** | **P1-7: rsx!{} 宏自动识别 signal 表达式** | ✅ 完成 (IntoVNodeChild for Signal<T> + IntoDynamicAttr/Class/Style for Signal<T>) |
| **六** | **P1-8: 运行时 effect 清理机制** | ✅ 完成 (effect_handles HashMap + register_effect_handle + create_tracked_effect + cleanup_component batch stop) |
| 四 | P2-1: 移除 Scheduler 双轨制 | ✅ 完成 (scheduler.rs deleted, 760 lines removed) |
| 四 | P2-2: 测试覆盖率提升 | ✅ 完成 (37 diff + 16 reactive + 10 event + 5 event = 68 new tests) |
| 四 | P2-3: 补充对外 API 文档 | ✅ 完成 (Signal, create_effect, batch, IntoVNodeChild rustdoc) |
| 五 | D-1: 重构英文文档入口层次 | ✅ 完成 (7 docs rewritten, 2 new docs created) |
| 五 | D-2: 新增开发者教程 | ✅ 完成 (getting-started.md + vdom.md) |
