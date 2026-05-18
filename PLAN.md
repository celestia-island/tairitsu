# Tairitsu 改进计划

> 架构审计 + 竞品横评反馈的综合改造路线图
> 最后更新: 2026-05-15

---

## 全部任务已完成

所有六个阶段的任务均已完成。以下是最终状态：

| 阶段 | 任务 | 状态 |
|------|------|------|
| — | T-1 ~ T-7: 事件便利方法 + Platform 拆分 + Reactive 统一 | ✅ |
| 一 | B-1 ~ B-3: 定位语 + API 分层 + 全栈示例 | ✅ |
| 二 | P0-1 ~ P0-3: unsafe hack 修复 + 脚手架 + Edition 2021 | ✅ |
| 三 | P1-1 ~ P1-4: npm 合并 + keyed diff + 事件补全 + WIT 去重 | ✅ |
| 四 | P2-1 ~ P2-3: Scheduler 移除 + 测试覆盖 + API 文档 | ✅ |
| 五 | D-1 ~ D-2: 英文文档重构 + 开发者教程 | ✅ |
| 六 | 细粒度响应式 + effect 清理 + 组件卸载 | ✅ |

### 组件卸载清理（方案 B）实现详情 (2026-05-15)

通过 DOM handle → ComponentId 映射表实现，3 个文件 ~80 行新增代码：

- `runtime.rs`: 新增 `element_to_component` HashMap + `register_element()` + `on_element_removed()`
- `wit_platform.rs`: `render_vnode` 中注册元素，`remove_child_at` 中触发清理
- `lib.rs`: 导出新 API

关键设计：当移除的 DOM 元素属于**当前正在渲染的组件**时跳过清理（仅移除映射条目），属于**其他组件**时自动调用 `cleanup_component` 停止 effects、清理 signal 订阅、移除 render function。

---

## 架构决策记录

### AD-4: 细粒度响应式采用 Hybrid 策略而非纯 fine-grained (2026-05-14)
- **决策**: 保持 VDOM + Signal 混合模型，只在 leaf level 做 DynamicText/DynamicAttr 细粒度更新
- **原因**: VDOM 对 Portal/条件渲染/SSR/列表 reconciliation 仍然有价值，完全去掉得不偿失
- **参考**: Dioxus 2024+ 也采用相同 hybrid 策略（VDOM 管结构，Signal 管值）

### AD-5: Dynamic VNode 在 vdom crate 中实现 (2026-05-14)
- **决策**: DynamicText/DynamicAttr/DynamicClass 作为 VNode 变体在 `tairitsu-vdom` 中实现
- **原因**: 这是渲染引擎的基础能力，所有基于 Tairitsu 的框架都需要
- **影响**: `tairitsu-macros` 的 `rsx!{}` 宏需要同步改造以自动生成 Dynamic 变体

### AD-7: 组件卸载清理策略 — 分阶段实施 (2026-05-14)
- **决策**: 0.5.x 用方案 B（DOM handle → ComponentId 映射表），0.6.0 用方案 A（`VNode::Component` 变体）
- **原因**: 方案 A 是 breaking change，不适合 patch release
- **实现**: 方案 B 已于 2026-05-15 完成并测试通过
