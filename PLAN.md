# Tairitsu 升级规划

## 背景与目标

本文档追踪 tairitsu 框架为支持 hikari 设计系统 **方案 B（AnimationBuilder 集成）** 而需要完成的工程工作。

hikari 方案 B 的目标是在 `Glow` 组件中集成 `ButtonStateMachine`，通过 `EasingFunction` 在 rAF 中插值 `--glow-intensity-scale`，实现平滑的鼠标进入/离开/按下动画。这要求 tairitsu 侧有一个**端到端可运行**的 rAF 动画系统、可靠的 `use_element_ref` DOM 绑定，以及 Signal → 重渲染 → DOM patch 的完整调度链。

---

## 当前缺口分析

通过代码审查，以下四个关键链路目前是**存根或断路**：

### Gap 1 — `ElementRef` 未在 DOM 挂载时填充

- **位置**：`packages/vdom/src/vnode.rs`（`VElement.element_ref` 字段已存在）、`packages/web/src/wit_platform.rs`（`mount_vnode_to_app` / `apply_patches`）
- **现状**：`VElement` 上已有 `element_ref: Option<AnyElementRef>` 字段，但 `mount_node()` / `apply_patches()` 内从未调用 `element_ref.set()`
- **影响**：`use_element_ref()` 返回的 ref 永远是 `None`，所有依赖它的动画、焦点管理逻辑均无效

### Gap 2 — rAF 回调只触发一帧，不自动续帧

- **位置**：`packages/hooks/src/animation.rs`，`UseAnimation::start_raf_loop()`
- **现状**：代码注释明确写道 *"This is a simplified implementation that only processes one frame"*；回调执行后不重新调用 `request_animation_frame`
- **影响**：`UseAnimation::start_with_platform()` 启动后只执行一帧插值，动画在第一帧后立即停止

### Gap 3 — Signal 变更不触发 DOM 更新

- **位置**：`packages/vdom/src/runtime.rs`（`schedule_render()`）、`packages/vdom/src/scheduler.rs`
- **现状**：`Signal::set()` → `notify_signal()` → `mark_dirty(id)` → `schedule_render()` 的调用链走通，但 `schedule_render()` 内部只写了一条 `trace!` 日志，没有调用 `platform.request_animation_frame()` 或任何实际调度；`Scheduler<P>` 结构体是另一条独立未接入的路径
- **影响**：组件状态通过 `use_signal` 变更后，框架知道哪些组件脏了，但 DOM 永远不会更新

### Gap 4 — `apply_patches()` 在 re-render 路径中没有被调用

- **位置**：`packages/vdom/src/runtime.rs`（`render_component()` / `flush_render()`）
- **现状**：`flush_render()` 计算出 diff patches 后，注释写道 *"In the full implementation, we'd call platform.apply_patches()"*，实际只有 `trace!`
- **影响**：即使 re-render 被触发，新 VNode 也只是计算了 patches，DOM 从未实际修改

---

## 实施任务

### 优先级说明

| 级别 | 含义 |
|------|------|
| **P0** | 方案 B 的硬依赖，缺少则动画完全无法工作 |
| **P1** | 方案 B 的软依赖，缺少则体验降级但可局部绕过 |
| **P2** | 框架完整性、安全性和长期维护 |
| **P3** | 未来扩展、开发者体验 |

---

### Task 1 — ElementRef DOM 绑定（P0）

**目标**：`use_element_ref()` 返回的 ref 在组件挂载后可通过 `.get()` 拿到真实 DOM 元素句柄。

**涉及文件**：
- `packages/web/src/wit_platform.rs`：`mount_node()` 递归函数
- `packages/vdom/src/vnode.rs`：`AnyElementRef` API 确认

**实现要点**：

1. `mount_node()` 处理 `VNode::Element(el)` 时，在创建 DOM 节点（`document::create_element(tag)`）之后，检查 `el.element_ref.is_some()`  
2. 若有 ref，将创建出的句柄包装为 `WitElement(handle)` 并调用 `el.element_ref.as_ref().unwrap().set(Box::new(element.clone()))`  
3. `apply_patches()` 中处理 `ReplaceNode` / `RemoveNode` 时相应调用 `.clear()` 以防悬垂引用

**验收标准**：

```rust
let wrapper_ref = use_element_ref::<WitElement>();
// mount 后：
assert!(wrapper_ref.get().is_some(), "ref must be populated after mount");
```

---

### Task 2 — rAF 循环自动续帧（P0）

**目标**：`UseAnimation::start_with_platform()` 启动后，rAF 回调在每帧后自动重新注册，直到动画完成或被取消。

**涉及文件**：
- `packages/hooks/src/animation.rs`：`start_raf_loop()` / `AnimationHandle`

**实现要点**：

1. 将续帧逻辑提取为独立 `fn schedule_next_frame(state: Rc<RefCell<AnimationState>>, platform: &dyn Platform, callback_id: u64)`  
2. rAF 回调内（`on_animation_frame` 处理器）：
   - 计算 `elapsed = timestamp - start_time`
   - 若 `elapsed < duration`：调用帧回调、再次调用 `platform.request_animation_frame(Box::new(next_frame_closure))`
   - 若 `elapsed >= duration`：调用最后一帧（t=1.0）、设置状态为 `Completed`、调用 `on_complete` 回调
3. `AnimationHandle::cancel()` 设置状态为 `Cancelled`，下次帧回调检查到后停止注册

**续帧闭包所有权模式**（避免 Rc cycle）：

```rust
// 使用 Weak<RefCell<AnimationState>> 在续帧闭包中持有，
// 避免 AnimationState 与闭包之间的循环引用
let weak_state = Rc::downgrade(&state);
```

**验收标准**：

```rust
let anim = use_simple_animation(300); // 300ms
let handle = anim.start_with_platform(&platform);
// 300ms 后（若干帧）：
assert_eq!(handle.state(), AnimationState::Completed);
// 期间 on_frame 回调被调用 ≥ 5 次（60fps 情况）
```

---

### Task 3 — Signal → 调度 → DOM patch 完整链路（P0）

这是框架最核心的缺口，分为三个子任务：

#### Task 3a — `schedule_render()` 接入 rAF 调度器

**位置**：`packages/vdom/src/runtime.rs`

**实现要点**：

1. `Runtime` 结构体中增加 `platform: Option<Box<dyn Platform>>` 字段（或通过全局 TLS 注入）  
2. `schedule_render()` 调用 `platform.request_animation_frame(Box::new(|| flush_render()))`，而非仅日志  
3. 加入 `scheduled` 保护标志避免重复注册 rAF  
4. `mount_vnode_to_app()` 调用后将 platform 实例注入 runtime（`runtime::set_platform(platform.clone())`）

#### Task 3b — `flush_render()` 调用 `apply_patches()`

**位置**：`packages/vdom/src/runtime.rs`, `packages/web/src/wit_platform.rs`

**实现要点**：

1. `flush_render()` 内遍历脏组件，对每个组件：
   - 调用组件渲染函数得到新 VNode
   - 用 `diff(old, new)` 计算 patches
   - 调用 `platform.apply_patches(root_element, &patches)`
   - 更新组件存储的 VNode 快照
2. Platform trait 暴露 `apply_patches(&self, root: &Self::Element, patches: &[Patch]) -> Result<()>`（已存在，确认签名一致）

#### Task 3c — `use_state` 接入 Signal/re-render

**位置**：`packages/hooks/src/state.rs`

**现状**：`use_state` 是纯 `Rc<RefCell<T>>` + setter，没有任何响应式订阅。

**实现要点**：

选项 A（推荐，简单）：`use_state` 内部改为基于 `Signal<T>` 实现，`setter` 调用 `signal.set()`，自动触发调度链。

选项 B（兼容性优先）：`setter` 内部主动调用 `runtime::mark_dirty(current_component_id())` + `runtime::schedule_render()`。

**验收标准**（Task 3 整体）：

```rust
let (count, set_count) = use_state(|| 0u32);
// 点击按钮调用 set_count(count + 1)
// → 期望 DOM 中 span 文字变为新值（不需要手动刷新页面）
```

---

### Task 4 — `ButtonStateMachine`（P1）

**目标**：提供一个轻量级状态机，管理交互元素的 `Idle / Hover / Active / Focused / Disabled` 状态转换，供 hikari `Glow`、`Button` 等组件使用。

**建议位置**：`packages/hooks/src/state_machine.rs`（新建），在 `packages/hooks/src/lib.rs` 中 pub use

**状态定义**：

```rust
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InteractionState {
    Idle,
    Hover,
    Active,    // mousedown / touchstart
    Focused,   // :focus-visible
    Disabled,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum InteractionEvent {
    MouseEnter,
    MouseLeave,
    MouseDown,
    MouseUp,
    Focus,
    Blur,
    Disable,
    Enable,
}
```

**转换表**：

| 当前状态 | 事件 | 下一状态 |
|----------|------|----------|
| Idle | MouseEnter | Hover |
| Hover | MouseLeave | Idle |
| Hover | MouseDown | Active |
| Hover | Focus | Focused |
| Active | MouseUp | Hover |
| Active | MouseLeave | Idle |
| Focused | MouseEnter | Hover |
| Focused | Blur | Idle |
| Any | Disable | Disabled |
| Disabled | Enable | Idle |

**API**：

```rust
pub struct ButtonStateMachine {
    state: InteractionState,
}

impl ButtonStateMachine {
    pub fn new() -> Self
    pub fn transition(&mut self, event: InteractionEvent) -> Option<InteractionState>
    // 返回 Some(new_state) 若状态发生变化，None 表示无效转换（忽略）
    pub fn state(&self) -> InteractionState
    pub fn is_interactive(&self) -> bool  // !Disabled
}

// Hook 封装（依赖 Task 3 完成后才能触发自动重渲染）
pub fn use_interaction_state() -> (InteractionState, Callback<InteractionEvent>)
```

**验收标准**：

```rust
let mut sm = ButtonStateMachine::new();
assert_eq!(sm.transition(InteractionEvent::MouseEnter), Some(InteractionState::Hover));
assert_eq!(sm.transition(InteractionEvent::MouseDown), Some(InteractionState::Active));
assert_eq!(sm.transition(InteractionEvent::MouseUp), Some(InteractionState::Hover));
assert_eq!(sm.transition(InteractionEvent::MouseLeave), Some(InteractionState::Idle));
```

---

### Task 5 — `AnimationBuilder` 流畅 API（P1）

**目标**：提供比 `AnimationConfig` struct 更直观的建造者，并封装"从当前值插值到目标值"的常用模式。

**建议位置**：`packages/hooks/src/animation.rs`（扩展现有文件）

**核心 API**：

```rust
pub struct AnimationBuilder {
    duration_ms: u64,
    easing: EasingFunction,
    delay_ms: u64,
    on_frame: Option<Box<dyn Fn(f32)>>,     // 参数为 eased progress [0.0, 1.0]
    on_complete: Option<Box<dyn Fn()>>,
}

impl AnimationBuilder {
    pub fn new() -> Self
    pub fn duration(mut self, ms: u64) -> Self
    pub fn easing(mut self, f: EasingFunction) -> Self
    pub fn delay(mut self, ms: u64) -> Self
    pub fn on_frame(mut self, f: impl Fn(f32) + 'static) -> Self
    pub fn on_complete(mut self, f: impl Fn() + 'static) -> Self
    pub fn build(self) -> AnimationConfig
}

// 插值辅助（CSS 变量常用场景）
pub fn lerp_f32(from: f32, to: f32, t: f32) -> f32 {
    from + (to - from) * t
}
```

**hikari 使用示例**（方案 B 集成后的 glow.rs）：

```rust
let animation = AnimationBuilder::new()
    .duration(200)
    .easing(EasingFunction::EaseOut)
    .on_frame(move |t| {
        let scale = lerp_f32(0.0, 0.5, t);
        set_style_property(&wrapper_el, "--glow-intensity-scale", &scale.to_string());
    })
    .build();
```

---

### Task 6 — 补充兼容性工作（P2）

#### 6a — `KeyboardEvent` 缺失修饰键字段

**位置**：`packages/web/src/wit_platform.rs` → `on_keyboard_event` 回调

**问题**：WIT 定义的 `keyboard-event-data` 包含 `ctrlKey/shiftKey/altKey/metaKey`，但 dispatch 代码中未传入这些字段（只有 `key` 和 `code`）。

**修复**：在 `on_keyboard_event` 中补充读取并传递修饰键字段。

#### 6b — 确认 `WheelEvent` 缺少鼠标坐标字段

**位置**：`packages/vdom/src/events.rs` → `WheelEvent` 结构体  

**问题**：Rust 侧 `WheelEvent` 有 `client_x/y`，WIT 定义的 `wheel-event-data` 也应有 `screen-x/y`、`page-x/y`、`offset-x/y`（与 `MouseEvent` 对齐）——需要确认 WIT 声明和 Rust 结构体是否一致。

#### 6c — 移除/替换 `packages/hooks/src/css_var.rs` 中的 `web-sys` 依赖

**位置**：`packages/hooks/src/css_var.rs`  

**问题**：`use_css_var` 使用 `web-sys` 直接操作，与 `wasm32-wasip2` + WIT 架构不兼容。应改为通过 Platform trait 的 `set_style_property` 实现，或标记为 `#[cfg(feature = "web-sys-compat")]` 并提供 WIT 版本。

---

### Task 7 — 端到端集成测试（P2）

在 `packages/web/` 或单独的 `packages/integration-tests/` 中增加以下测试：

1. **ElementRef 挂载测试**：创建带 `element_ref` 的 VNode，挂载后断言 ref 非空
2. **rAF 动画完整性测试**：启动 300ms 动画，mock `request_animation_frame`，驱动若干帧，验证 `on_frame` 调用次数 ≥ 2 且最终 t ≈ 1.0
3. **Signal → DOM patch 测试**：`use_signal(0)` → `set(1)` → 经过一个 rAF tick → 断言 DOM 文本节点已更新
4. **ButtonStateMachine 状态转换测试**：完整轮历所有合法/非法转换

---

## 任务依赖图

```
Task 1 (ElementRef)    ────────────────────────────────► hikari 方案 B (Glow ref 绑定)
                                                         │
Task 2 (rAF 续帧)  ──────────────────────────────────►  │
                                                         │
Task 3a (schedule_render)                                ▼
    ↓                                          AnimationBuilder 集成
Task 3b (flush_render + apply_patches)         ButtonStateMachine 集成
    ↓
Task 3c (use_state re-render)

Task 4 (ButtonStateMachine) ─────────────────────────────────── (独立实现)
Task 5 (AnimationBuilder)   ─────────────────────────────────── (独立实现，可先行)
Task 6 (兼容性修复)         ─────────────────────────────────── (独立)
Task 7 (集成测试)           ──── 依赖 Task 1~3 全部完成 ────────────────
```

---

## 任务清单

| 优先级 | 任务 | 文件 | 状态 |
|--------|------|------|------|
| **P0** | Task 1：ElementRef 挂载时填充 DOM 句柄 | `packages/web/src/wit_platform.rs` | ⬜ 待实现 |
| **P0** | Task 2：rAF 回调自动续帧 | `packages/hooks/src/animation.rs` | ⬜ 待实现 |
| **P0** | Task 3a：`schedule_render()` 接入 rAF | `packages/vdom/src/runtime.rs` | ⬜ 待实现 |
| **P0** | Task 3b：`flush_render()` 调用 `apply_patches()` | `packages/vdom/src/runtime.rs` | ⬜ 待实现 |
| **P0** | Task 3c：`use_state` / `use_signal` 触发重渲染 | `packages/hooks/src/state.rs` | ⬜ 待实现 |
| **P1** | Task 4：`ButtonStateMachine` | `packages/hooks/src/state_machine.rs`（新） | ⬜ 待实现 |
| **P1** | Task 5：`AnimationBuilder` 流畅 API | `packages/hooks/src/animation.rs` | ⬜ 待实现 |
| **P2** | Task 6a：`KeyboardEvent` 补充修饰键 | `packages/web/src/wit_platform.rs` | ⬜ 待确认 |
| **P2** | Task 6b：`WheelEvent` 坐标字段对齐 | `packages/vdom/src/events.rs` | ⬜ 待确认 |
| **P2** | Task 6c：移除 `use_css_var` 的 `web-sys` 依赖 | `packages/hooks/src/css_var.rs` | ⬜ 待实现 |
| **P2** | Task 7：端到端集成测试 | `packages/web/` 或新 crate | ⬜ 待实现 |

---

## 验收标准（整体）

1. `use_element_ref()` 返回的 ref 在组件挂载后 `.get()` 非空
2. `UseAnimation::start_with_platform()` 启动的 300ms 动画在 ≥ 5 帧内完成（60fps），最终状态为 `Completed`
3. `use_signal(0)` 调用 `set(1)` 后，无需手动调用任何 flush，DOM 在下一个 rAF tick 中自动更新
4. `ButtonStateMachine::transition()` 通过状态机完整测试（见 Task 4 验收标准）
5. hikari 的 `Glow` 组件（方案 B）编译通过，鼠标移入时 `--glow-intensity-scale` 在约 200ms 内从 0 插值到 0.5
