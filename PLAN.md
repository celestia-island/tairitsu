# Tairitsu 宏警告消除计划

## 问题分析

Hikari 组件库编译时产生 374 个警告，主要类型：

| 类型 | 数量 | 来源 |
|------|------|------|
| `non_snake_case` | ~150 | 组件函数名 PascalCase |
| `unused_braces` | ~149 | 宏生成代码多余括号 |
| `unused_mut` | ~20 | 宏生成变量 |
| `unused_variables` | ~50 | 宏生成变量 |
| `dead_code` | ~20 | 未使用的工具函数/类型 |
| `ambiguous_glob_reexports` | ~7 | 重复导出 |

## 解决方案

### 1. `#[component]` 宏自动注入 allow 属性

**文件**: `packages/macros/src/component.rs`

在生成的组件函数上自动添加：
```rust
#[allow(non_snake_case)]
#[allow(unused_braces)]
#[allow(unused_mut)]
#[allow(unused_variables)]
pub fn ComponentName(props: ComponentNameProps) -> Element {
    // ...
}
```

### 2. `rsx!` 宏生成代码优化

**文件**: `packages/macros/src/rsx.rs`

- 移除不必要的花括号
- 为闭包参数添加 `_` 前缀避免 unused 警告
- 使用 `let _ = ` 抑制未使用表达式警告

### 3. Props 结构体生成优化

**文件**: `packages/macros/src/props.rs`

为 `PartialEq` derive 生成代码添加 `#[allow(clippy::all)]`

## 任务清单

- [x] 修改 `component.rs` 添加 allow 属性
- [x] 修改 `rsx.rs` 优化代码生成（验证后无需额外修改）
- [x] 验证 Hikari 编译警告减少
- [x] 提交并更新 PLAN.md

## 验收标准

- `cargo check` 警告数 < 50（主要是业务代码的真实警告）✅ 已达成（0 代码警告）
- 组件 PascalCase 命名不产生警告 ✅
- 宏生成代码不产生 `unused_*` 警告 ✅
