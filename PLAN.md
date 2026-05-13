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

---

## 待办（按优先级）

### P1 — 高优先级

（T-1b、T-3、T-6 已完成）

### ✅ T-5: 统一 Reactive 系统 (2026-05-14)

- `Scheduler` 标记为 `#[deprecated]`（生产环境只用 `runtime.rs`）
- `runtime.rs` 为唯一调度器，`Scheduler` 将在 0.3.0 移除

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

## 架构决策记录

### AD-1: 事件便利方法实现策略
- **决策**: 在 VElement impl 块中直接添加方法，内部调用 `on_event()` + downcast
- **原因**: 最小侵入，不修改 trait 或 WIT 层
- **替代方案**: 宏生成（减少重复代码，但降低可读性）
- **后续**: 当方法数量超过 25 个时考虑重构为宏
