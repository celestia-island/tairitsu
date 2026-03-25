# Tairitsu 升级规划 — 支撑 Hikari Glow 动画迁移

> **目标**：补齐 tairitsu 框架中 hikari Glow 组件动画效果所需的基础设施，使命令式动画系统（AnimationBuilder / ButtonStateMachine）能在 tairitsu 上正常运作。

---

## 背景

Hikari 正从 dioxus 迁移到 tairitsu。其中 Glow Wrapper 组件需要：

- 响应 `mouseenter`/`mouseleave`/`mousedown`/`mouseup` 事件
- 在回调中**命令式**操作 DOM 元素的 inline style（`--glow-x`、`--glow-y`、`--glow-intensity-scale`）
- 长期需要 `requestAnimationFrame` 驱动的帧循环做缓动插值

---

## 现状盘点

### 已完成 ✅

| 模块 | 能力 |
|------|------|
| **事件系统** | `MouseEvent`（完整坐标）、`KeyboardEvent`、`FocusEvent`、`DragEvent`、`InputEvent`、`ChangeEvent` |
| **高级事件** | `WheelEvent`、`TouchEvent`、`PointerEvent`、`TransitionEvent`、`AnimationEvent` |
| **RSX 事件映射** | `onclick`/`mousedown`/`mouseup`/`mousemove`/`mouseenter`/`mouseleave` 等 |
| **Platform trait** | `request_animation_frame`、`set_style`、`get_bounding_client_rect`、DOM 操作 |
| **Hooks** | `use_signal`、`use_state`、`use_effect`、`use_memo`、`use_callback`、`use_ref`、`use_element_ref`、`use_animation`、`use_css_var`、`use_style` |
| **动画系统** | `UseAnimation` rAF 帧循环、`EasingFunction::evaluate()` |
| **响应式** | Signal → 重渲染调度、`apply_patch` 实现 |
| **样式** | `CssProperty`（300+ 属性）、`StyleStringBuilder`、`StyleBuilder`、`ClassesBuilder` |
| **组件系统** | `#[component]` 宏 + `rsx!` 宏 + Props builder |

---

## 实现总结

### Phase 0 — Glow 最小可用 ✅

- 验证 `mouseenter`/`mouseleave` 事件派发
- 验证事件回调中 DOM 操作可行性
- 导出 `use_css_var` 和 `use_style` hooks

### Phase 1 — 动画基础设施 ✅

- `UseAnimation` 连接 rAF 驱动
- `EasingFunction::evaluate(t)` 数学实现
- `use_element_ref` DOM 引用 Hook

### Phase 2 — MouseEvent 扩展 ✅

- 补充 `offset_x/y`、`page_x/y`、`movement_x/y` 字段

### Phase 3 — 响应式渲染循环 ✅

- Signal → 重渲染调度
- `apply_patch` 实现

### Phase 4 — 高级事件类型 ✅

- `WheelEvent` - 滚轮缩放、自定义滚动
- `TouchEvent` - 移动端触摸交互
- `PointerEvent` - 统一的指针抽象
- `TransitionEvent` - CSS transition 完成回调
- `AnimationEvent` - CSS animation 生命周期回调

---

## 所有任务已完成 ✅

PLAN.md 中的所有任务已全部实现并通过测试。tairitsu 框架现在具备完整的基础设施来支持 Hikari Glow 组件的动画效果迁移。
