# Tairitsu 升级规划

## 背景与目标

本文档追踪 tairitsu 框架为支持 hikari 设计系统 **方案 B（AnimationBuilder 集成）** 而需要完成的工程工作。

hikari 方案 B 的目标是在 `Glow` 组件中集成 `ButtonStateMachine`，通过 `EasingFunction` 在 rAF 中插值 `--glow-intensity-scale`，实现平滑的鼠标进入/离开/按下动画。这要求 tairitsu 侧有一个**端到端可运行**的 rAF 动画系统、可靠的 `use_element_ref` DOM 绑定，以及 Signal → 重渲染 → DOM patch 的完整调度链。

---

## 实施任务完成状态

所有 PLAN.md 任务（Task 1-7）已完成。

### Task 1 — ElementRef DOM 绑定（P0）✅
- 在 `render_vnode()` 和 `create_vnode_element()` 中填充 element_ref
- 修复 patch 创建元素时的 ref 填充

### Task 2 — rAF 循环自动续帧（P0）✅
- 实现 `schedule_frame()` 自续帧逻辑
- 使用 `Weak<RefCell>` 避免循环引用

### Task 3 — Signal → 调度 → DOM patch 完整链路（P0）✅
- 回调式 runtime 架构
- `ReactiveSignal` 自动脏跟踪
- `runtime_integration` 模块

### Task 4 — `ButtonStateMachine`（P1）✅
- 新建 `state_machine.rs`
- 完整状态转换表
- `use_interaction_state()` hook

### Task 5 — `AnimationBuilder` 流畅 API（P1）✅
- Builder 模式 API
- `lerp_f32()` 辅助函数
- 15 个单元测试

### Task 6 — 兼容性修复（P2）✅
- KeyboardEvent 修饰键字段
- css_var.rs wasm32-only 编译

### Task 7 — 端到端集成测试（P2）✅
- 20 个集成测试
- MockPlatform 测试基础设施
- ElementRef、rAF、Signal、状态机测试覆盖

---

## 验收标准（整体）

1. ✅ `use_element_ref()` 返回的 ref 在组件挂载后 `.get()` 非空
2. ✅ `UseAnimation::start_with_platform()` 启动的 300ms 动画在 ≥ 5 帧内完成（60fps），最终状态为 `Completed`
3. ✅ `use_signal(0)` 调用 `set(1)` 后，无需手动调用任何 flush，DOM 在下一个 rAF tick 中自动更新
4. ✅ `ButtonStateMachine::transition()` 通过状态机完整测试
5. ⏳ hikari 的 `Glow` 组件（方案 B）编译通过，鼠标移入时 `--glow-intensity-scale` 在约 200ms 内从 0 插值到 0.5（待 hikari 集成）
