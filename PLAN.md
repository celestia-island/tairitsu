# Tairitsu <- Hikari 迁移协同计划

## 背景

Hikari 已开始将默认构建入口切换到 `tairitsu-packager`，但其网站代码仍为 Dioxus 架构。
需要在 Tairitsu 侧补齐迁移支撑能力，降低改造成本。

## 已落地对接点

- `just dev/watch/watch-dev/dev-by-agent` 已改为调用 `tairitsu-packager`。
- Hikari `examples/website/Cargo.toml` 已提供 `package.metadata.tairitsu`。
- `wasm-bindgen-cli` 安装脚本已进入弃用状态。

## 协同里程碑

- M1: Hikari 网站最小页面（非 Dioxus）在 `tairitsu-packager dev` 成功启动。
- M2: Hikari 移除 `wasm-bindgen-cli` 运行依赖。
- M3: Hikari 移除 Dioxus 依赖并完成 CI 切换。

---

## ✅ Dioxus 兼容层状态

从 Hikari 迁移需要的兼容性别名和类型：

| Dioxus 类型 | Tairitsu 对应 | 状态 |
| ----------- | ------------- | ---- |
| `Element` | `VNode` | ✅ 别名已创建 |
| `#[derive(Props)]` | `#[component]` | ✅ 可用（语法不同但功能等效） |
| `use_signal` | `use_signal` | ✅ 已有 |
| `use_context_provider` | `provide_context` | ✅ 已有 |
| `Callback<T>` | `Callback<T, ()>` | ✅ 已添加 (`tairitsu-vdom`) |
| `EventHandler<T>` | `EventHandler<T>` | ✅ 已添加（类型别名） |
| `use_memo` | `use_memo` | ✅ 已添加 (`tairitsu-hooks`) |
| `use_callback` | `use_callback` | ✅ 已添加 (`tairitsu-hooks`) |

## 实现详情

### packages/vdom/src/callback.rs

- `Callback<T, R>` - 通用回调类型，包装 `Fn(T) -> R`
- `EventHandler<T>` - Dioxus 兼容别名 (`Callback<T, ()>`)
- 支持 `Clone`, `Deref`, `From` trait
- 提供 `call()`, `no_arg()`, `simple()` 方法

### packages/hooks/src/memo.rs

- `use_memo(compute, deps)` - 依赖驱动的响应式计算
- `Memo<T, D, F>` 结构体，支持 `Clone` 和依赖更新

### packages/hooks/src/callback.rs

- `use_callback(factory, deps)` - 依赖驱动的回调缓存
- `use_void_callback(callback, deps)` - 无返回值回调
- `use_return_callback(callback, deps)` - 有返回值回调

---

## 剩余工作

- [ ] 完善组件宏的 Props 默认值语法糖（可选）
- [ ] E2E 测试覆盖新增的 hooks 和类型
