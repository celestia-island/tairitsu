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

### P2 — 中优先级

#### T-5: 统一 Reactive 系统
- `runtime.rs`（signal-based）和 `scheduler.rs`（rAF-based）职责重叠
- 统一为一个 scheduler，消除维护负担

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
