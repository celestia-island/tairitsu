# Tairitsu 改进计划

> 架构审计 + 竞品横评反馈的综合改造路线图
> 最后更新: 2026-05-15

---

## 剩余工作

### Gap 3: cleanup_component() 未被调用 — 组件卸载时 effect 泄漏（P1）

> **现状**: `cleanup_component(id)` 已定义（`runtime.rs:328`），能批量 `stop()` 所有 `EffectHandle`，但**全局零调用**。组件卸载后 effect 闭包仍会执行（DOM 操作是 no-op，但闭包 + Rc 引用链不会被释放）。
>
> **根本原因**: VNode 树和 Patch 系统是纯结构性的，不携带 ComponentId 信息。组件卸载的 DOM 操作（`Patch::RemoveChild { index }`）到达平台层时已丢失 ComponentId。

#### 架构分析

**当前数据流**:
```
render_component(id)                     ← ComponentId 可用
  → diff(old, new) → Vec<Patch>          ← Patch 不含 ComponentId
  → apply_patches_callback(id, patches)  ← ComponentId 可用（但只传给了 with_render_component）
  → apply_patch(platform, element, patch) ← ComponentId 丢失
  → remove_child_at(platform, parent, i) ← 纯 DOM 操作
```

**关键缺失**: 没有 `VNode::Component { id }` 变体。`render_component` 只重渲染单个组件，无嵌套子组件概念。

#### 实现方案

**方案 A: 添加 `VNode::Component` 变体（推荐）**

在 `packages/vdom/src/vnode.rs` 的 `VNode` 枚举中新增：

```rust
pub enum VNode {
    Element(VElement),
    Text(VText),
    DynamicText(DynamicText),
    Fragment(Vec<VNode>),
    Component {
        id: ComponentId,
        render_fn: RenderFn,
        children: Box<VNode>,  // 缓存的渲染结果
    },
}
```

改造点：
1. **`diff.rs`**: `VNode::Component` 被移除时生成 `Patch::RemoveComponent { id: ComponentId }`
2. **`patch.rs`**: 新增 `Patch::RemoveComponent { id: ComponentId }` 变体
3. **`runtime.rs`** `render_component()`: diff 结果包含 `RemoveComponent` 时调用 `cleanup_component(id)`
4. **`runtime.rs`** `use_component()`: 返回的 VNode 包裹在 `VNode::Component` 中
5. **`rsx!{}`**: 用户调用 `Component { prop: value }` 时生成 `VNode::Component` 节点

**影响范围**: `vnode.rs` + `diff.rs` + `patch.rs` + `runtime.rs` + `macros/rsx.rs` + `wit_platform.rs`

**方案 B: DOM handle → ComponentId 映射表（轻量级）**

在 `runtime.rs` 中维护：
```rust
element_to_component: HashMap<u64, ComponentId>  // raw DOM handle → ComponentId
```

改造点：
1. `render_vnode` 中为每个创建的 DOM 元素注册映射（需要传入 ComponentId）
2. `remove_child_at` 中查询被移除子树的所有映射，批量 `cleanup_component`
3. 不需要改 VNode/Patch 数据结构

**优点**: 改动小，不改 VNode 枚举（避免 breaking change）
**缺点**: 只能在 `wit_platform.rs` 层实现，runtime 层无感知；需要遍历子树查找所有映射

#### 预估工作量

| 方案 | 改动文件 | 行数 | Breaking? |
|------|----------|------|-----------|
| A (VNode::Component) | 6 个文件 | ~400 行 | 是（VNode 枚举新增变体） |
| B (映射表) | 3 个文件 | ~150 行 | 否 |

**建议**: 0.5.x 用方案 B 快速止血，0.6.0 用方案 A 做正式组件生命周期系统。

#### 验收标准

1. 组件卸载时 `cleanup_component(id)` 被调用
2. 卸载后 signal 变化不再触发已卸载组件的 effect
3. `EffectHandle::is_stopped()` 返回 `true`
4. 虚拟列表（高频创建/销毁）场景无内存泄漏

---

## 架构决策记录

### AD-4: 细粒度响应式采用 Hybrid 策略而非纯 fine-grained (2026-05-14)
- **决策**: 保持 VDOM + Signal 混合模型，只在 leaf level 做 DynamicText/DynamicAttr 细粒度更新
- **原因**: VDOM 对 Portal/条件渲染/SSR/列表 reconciliation 仍然有价值，完全去掉得不偿失
- **参考**: Dioxus 2024+ 也采用相同 hybrid 策略（VDOM 管结构，Signal 管值）
- **替代方案**: 改为 Leptos 式纯 fine-grained（需重写整个 diff/patch 系统，代价太大）

### AD-5: Dynamic VNode 在 vdom crate 中实现 (2026-05-14)
- **决策**: DynamicText/DynamicAttr/DynamicClass 作为 VNode 变体在 `tairitsu-vdom` 中实现
- **原因**: 这是渲染引擎的基础能力，所有基于 Tairitsu 的框架（Hikari 及未来的）都需要
- **影响**: `tairitsu-macros` 的 `rsx!{}` 宏需要同步改造以自动生成 Dynamic 变体

### AD-7: 组件卸载清理策略 — 分阶段实施 (2026-05-14)
- **决策**: 0.5.x 用方案 B（DOM handle → ComponentId 映射表）快速止血，0.6.0 用方案 A（`VNode::Component` 变体）做正式组件生命周期
- **原因**: 方案 A 是 breaking change（VNode 枚举新增变体），不适合 patch release；方案 B 改动小、无 breaking，可立即止血 effect 泄漏
- **影响**: 方案 B 仅改 `runtime.rs` + `wit_platform.rs` + `runtime_integration.rs`，约 150 行
