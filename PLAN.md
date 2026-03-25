# Tairitsu 升级规划 — 支撑 Hikari Glow 动画迁移

> **目标**：补齐 tairitsu 框架中 hikari Glow 组件动画效果所需的基础设施，使命令式动画系统（AnimationBuilder / ButtonStateMachine）能在 tairitsu 上正常运作。

---

## 背景

Hikari 正从 dioxus 迁移到 tairitsu。其中 Glow Wrapper 组件需要：
- 响应 `mouseenter`/`mouseleave`/`mousedown`/`mouseup` 事件
- 在回调中**命令式**操作 DOM 元素的 inline style（`--glow-x`、`--glow-y`、`--glow-intensity-scale`）
- 长期需要 `requestAnimationFrame` 驱动的帧循环做缓动插值

当前 tairitsu 已有 Platform trait 中的 `request_animation_frame`、事件枚举中的 `MouseEvent`，以及 hooks 中的 `UseAnimation` 状态容器，但关键集成环节缺失。

---

## 现状盘点

### 已完成 ✅

| 模块 | 能力 |
|------|------|
| **事件系统** | `MouseEvent`（`client_x/y`, `screen_x/y`, modifiers）、`KeyboardEvent`、`FocusEvent`、`DragEvent`、`InputEvent`、`ChangeEvent` |
| **RSX 事件映射** | `onclick`/`mousedown`/`mouseup`/`mousemove`/`mouseenter`/`mouseleave` → `MouseEvent` |
| **Platform trait** | `request_animation_frame`/`cancel_animation_frame`、`set_style`、`get_bounding_client_rect`、`set_timeout`/`clear_timeout` |
| **WIT Platform 实现** | `request_animation_frame` 已通过 WIT FFI 实现（`packages/web/src/wit_platform.rs`） |
| **Hooks** | `use_signal`、`use_state`、`use_effect`、`use_memo`、`use_callback`、`use_ref`、`use_animation`（状态容器） |
| **样式** | `CssProperty`（300+ 属性）、`StyleStringBuilder`、`StyleBuilder`、`ClassesBuilder` |
| **CSS 变量 Hook** | `use_css_var`（读写 `:root` CSS 变量，文件存在但未从 lib.rs 导出） |
| **组件系统** | `#[component]` 宏 + `rsx!` 宏 + Props builder |

### 缺失项（按优先级排列）

---

## Phase 0 — Glow 最小可用（P0，无需 AnimationBuilder）

Hikari 方案 A（最小化恢复）只需 event handler 中直接操作 CSS 变量。tairitsu 侧需验证/补齐：

### 0.1 验证 `mouseenter`/`mouseleave` 事件派发 ☐

**现状**：RSX 映射层已将 `onmouseenter`/`onmouseleave` 映射到 `MouseEvent`，但需验证：
1. WIT platform 的 `add_event_listener` 是否正确注册 `mouseenter`/`mouseleave`（而非 `mouseover`/`mouseout`）
2. 事件在冒泡/非冒泡行为上是否符合 W3C 规范（`mouseenter` 不冒泡）
3. `event_handle` 的 `prevent_default()`/`stop_propagation()` 是否正确工作

**验证方式**：编写最小测试组件，在 `onmouseenter` 回调中 `log()` 确认调用。

### 0.2 确认 event handler 中 DOM 操作可行 ☐

Hikari Glow 的 event handler 调用 `crate::platform::set_style_property`（底层为 WIT FFI）。需验证：
1. 在事件回调闭包中调用 `Platform::set_style` 是否正常工作
2. 通过 class name 查询元素（`element_from_point`、`querySelector`）是否在事件回调中可用
3. `get_bounding_client_rect` 返回值是否正确

**无需新增功能**——仅验证现有 API 在事件回调上下文中正常工作。

---

## Phase 1 — 动画基础设施（P1）

### 1.1 `UseAnimation` 连接 rAF 驱动 ☐

**现状**：`UseAnimation` 只维护 `AnimationState` 和 `progress: f32`，`start()` 仅改状态为 `Running`，无实际帧循环。

**需要**：
```rust
impl UseAnimation {
    /// 启动动画，连接 requestAnimationFrame 循环
    pub fn start(&self, platform: &impl Platform) {
        *self.state.borrow_mut() = AnimationState::Running;
        self.start_raf_loop(platform);
    }

    fn start_raf_loop(&self, platform: &impl Platform) {
        // 1. 记录起始时间
        // 2. 在每帧 callback 中：
        //    - 计算 elapsed 时间
        //    - 应用 EasingFunction 得到 progress
        //    - 更新 self.progress
        //    - 通知订阅者（或触发重渲染）
        //    - 如未完成则继续 request_animation_frame
    }
}
```

**关键设计决策**：
- 如何获取 `Platform` 引用——通过 context/全局单例/参数传入？
- 帧更新如何触发 UI 变化——直接操作 DOM（命令式）还是更新 Signal 触发重渲染（声明式）？

**建议**：短期采用命令式（在 rAF callback 中直接 `platform.set_style()`），长期再引入响应式重渲染。

### 1.2 `EasingFunction::evaluate(t)` 实现 ☐

**现状**：`EasingFunction` 枚举有 `Linear`/`Ease`/`EaseIn`/`EaseOut`/`EaseInOut`/`CubicBezier(f32,f32,f32,f32)` 变体，但无计算方法。

**需要**：
```rust
impl EasingFunction {
    pub fn evaluate(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::Ease => cubic_bezier(0.25, 0.1, 0.25, 1.0, t),
            Self::EaseIn => cubic_bezier(0.42, 0.0, 1.0, 1.0, t),
            Self::EaseOut => cubic_bezier(0.0, 0.0, 0.58, 1.0, t),
            Self::EaseInOut => cubic_bezier(0.42, 0.0, 0.58, 1.0, t),
            Self::CubicBezier(x1, y1, x2, y2) => cubic_bezier(*x1, *y1, *x2, *y2, t),
        }
    }
}
```

Cubic Bezier 求解可用 Newton-Raphson 迭代或二分法。hikari 的 `packages/animation/src/easing.rs` 中可能已有参考实现。

### 1.3 `use_element_ref` — DOM 元素引用 Hook ☐

**现状**：`use_ref<T>` 只是 `Rc<RefCell<T>>`，无法绑定到实际 DOM 元素。

**需要**：
```rust
pub struct ElementRef {
    handle: Rc<RefCell<Option<Platform::Element>>>,
}

pub fn use_element_ref() -> ElementRef { ... }

// 在 rsx! 中：
rsx! {
    div { ref_: my_ref, class: "hi-glow-wrapper", ... }
}
```

**实现要点**：
1. `ElementRef` 在组件渲染时创建，初始为 `None`
2. `rsx!` 宏识别 `ref_:` 属性，在 VNode 中标记
3. patch/mount 阶段，当 DOM 元素实际创建后，回填 `ElementRef.handle`
4. 此后 hook 层可通过 `element_ref.get()` 获取句柄，调用 `Platform::set_style` 等

**这是 AnimationBuilder 集成的核心前置条件**——没有它就无法从 hook 层直接操作 DOM 元素。

---

## Phase 2 — MouseEvent 扩展（P2）

### 2.1 补充 `offset_x/y`、`page_x/y`、`movement_x/y` ☐

**现状**：`MouseEvent` 仅有 `client_x: i32`、`client_y: i32`、`screen_x: i32`、`screen_y: i32`。

**需要**：
```rust
pub struct MouseEvent {
    // 已有
    pub client_x: i32,
    pub client_y: i32,
    pub screen_x: i32,
    pub screen_y: i32,
    // 新增
    pub offset_x: i32,      // 相对于 target 元素的坐标
    pub offset_y: i32,
    pub page_x: i32,        // 相对于文档的坐标（含滚动偏移）
    pub page_y: i32,
    pub movement_x: i32,    // 自上次事件的鼠标位移
    pub movement_y: i32,
    // ...existing fields
}
```

**WIT 侧**：需要在 `browser.wit` 的鼠标事件接口中新增对应字段，并在 JS glue 层从 `MouseEvent` 对象提取。

**影响**：
- hikari Glow 当前用 `client_x/y` + `getBoundingClientRect` 计算相对坐标（可继续工作）
- 有 `offset_x/y` 后可简化计算并提升精度
- `movement_x/y` 对拖拽/惯性滚动等场景有价值

---

## Phase 3 — 响应式渲染循环（P0 长期）

> 这是 tairitsu 作为完整 UI 框架的基础能力，不仅限于 Glow 组件。

### 3.1 Signal → 重渲染调度 ✅

**现状**：`Signal::set()` 通知订阅者，但无机制连接到 VDOM 重渲染。

**需要**：
1. 组件渲染时自动追踪所读取的 Signal ✅ 已实现 (`runtime::track_signal`)
2. Signal 变化时标记对应组件为 dirty ✅ 已实现 (`runtime::mark_dirty`)
3. 调度 microtask 或 rAF 批量重渲染 dirty 组件 ✅ 已实现 (`Scheduler`)
4. 重新调用组件 render → diff → patch ✅ 已实现

**实现细节**：
- 新增 `runtime` 模块：提供组件注册、信号追踪、脏标记功能
- 新增 `scheduler` 模块：提供 rAF 调度的批量渲染
- `Signal::get()` 自动追踪依赖到当前组件
- `Signal::set()` 自动通知依赖组件并标记为 dirty

### 3.2 `apply_patch` 实现 ☐

**现状**：diff 算法存在（`diff.rs`），patch 数据结构存在（`patch.rs`），但无 `apply_patch` 到实际 DOM 的实现。

**需要**：在 `WitPlatform` 中实现 patch apply 逻辑——遍历 patches，调用 `Platform` trait 方法更新 DOM。

---

## Phase 4 — 高级事件类型（P3）

| 事件类型 | 用途 |
|----------|------|
| `WheelEvent` | 滚轮缩放、自定义滚动 |
| `TouchEvent` | 移动端触摸交互 |
| `PointerEvent` | 统一的指针抽象（鼠标+触摸+笔） |
| `TransitionEvent` | CSS transition 完成回调 |
| `AnimationEvent` | CSS animation 生命周期回调 |

---

## 联调计划

### 第一轮联调（Phase 0 验证 → hikari 方案 A）

```
tairitsu: 验证 mouseenter/mouseleave 派发 + 事件回调中 DOM 操作
    ↓
hikari: 在 glow.rs handler 中 set_style_property("--glow-intensity-scale", "1"/"0")
    ↓
联合测试: 浏览器中鼠标移入/移出 Button，glow 效果可见
```

### 第二轮联调（Phase 1 → hikari 方案 B）

```
tairitsu: 实现 use_element_ref + UseAnimation rAF 循环 + EasingFunction
    ↓
hikari: Glow 集成 ButtonStateMachine + AnimationBuilder
    ↓
联合测试: 鼠标交互有缓动过渡效果，按下/释放有强度变化
```

### 第三轮联调（Phase 2-3）

```
tairitsu: MouseEvent 扩展字段 + Signal→重渲染
    ↓
hikari: 简化 Glow 坐标计算 + 响应式重构
```

---

## 任务优先级总览

| 优先级 | 任务 | 阶段 | 状态 |
|--------|-----|------|------|
| **P0** | 验证 mouseenter/mouseleave 事件派发 | Phase 0 | ✅ 已完成 |
| **P0** | 验证事件回调中 DOM 操作可行性 | Phase 0 | ✅ 已完成 (详见 PHASE_0.2_VERIFICATION.md) |
| **P1** | `UseAnimation` 连接 rAF 帧循环 | Phase 1 | 部分完成 (已有 RAF 循环实现) |
| **P1** | `EasingFunction::evaluate(t)` 数学实现 | Phase 1 | ✅ 已完成 |
| **P1** | `use_element_ref` DOM 引用 Hook | Phase 1 | 待实现 |
| **P2** | `MouseEvent` 补充 offset/page/movement 字段 | Phase 2 | 待实现 |
| **P0（长期）** | Signal → 重渲染调度循环 | Phase 3 | ✅ 已完成 |
| **P0（长期）** | `apply_patch` 实现 | Phase 3 | 待实现 |
| **P2** | 导出 `css_var`、`style` hooks（已有文件未 re-export） | Phase 1 | 待修复 |
| **P3** | WheelEvent/TouchEvent/PointerEvent/TransitionEvent/AnimationEvent | Phase 4 | 待实现 |
