# Tairitsu 改进计划

> 基于 celestia/PLAN.md 基础设施综述 + hikari 实际开发反馈
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

---

## 待办（按优先级）

### P1 — 高优先级

（T-1b 和 T-3 已完成）

### P2 — 中优先级
- 更安全，适合在事件 handler / 全局状态中使用

当前 `use_signal()` 绑定到 component_id。`Signal::new()` 可以独立创建但不触发 re-render。

方案：
```rust
// 方案 A: 全局 ReactiveStore
let store = ReactiveStore::new();
store.set("key", value);

// 方案 B: Signal::new_standalone() 自动 flush
let signal = Signal::new_standalone(initial);
signal.set(new_value); // 自动 mark_dirty + flush_render
```

### P2 — 中优先级

#### T-5: 统一 Reactive 系统
- `runtime.rs`（signal-based）和 `scheduler.rs`（rAF-based）职责重叠
- 统一为一个 scheduler，消除维护负担

#### T-6: Platform Trait 拆分
当前 ~70 个 required method 导致 mock/testing 困难。
```rust
trait DomOps { create_element, append_child, set_attribute, ... }
trait TimerOps { set_timeout, set_interval, ... }
trait LayoutOps { get_bounding_client_rect, inner_width, ... }
trait Platform: DomOps + TimerOps + LayoutOps + ...
```

#### T-7: 事件监听器选项
当前 `use_capture` 硬编码 `false`。需要：
- WIT 层添加 `listener-options` record { passive, once, capture }
- Platform trait 签名扩展
- browser-glue 实现支持

### P3 — 低优先级

- browser-glue runtime.js 内嵌（消除路径解析 fallback）
- WIT 层添加 `get_bounding_client_rect` 独立函数（供事件 handler 调用）
- SubmitEvent 专用类型（当前 on_submit 复用 ChangeEvent）
- 事件 target/current_target 支持扩展到更多事件类型

---

## 架构决策记录

### AD-1: 事件便利方法实现策略
- **决策**: 在 VElement impl 块中直接添加方法，内部调用 `on_event()` + downcast
- **原因**: 最小侵入，不修改 trait 或 WIT 层
- **替代方案**: 宏生成（减少重复代码，但降低可读性）
- **后续**: 当方法数量超过 25 个时考虑重构为宏
