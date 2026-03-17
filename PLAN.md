# Tairitsu <- Hikari 迁移协同计划

## 背景

Hikari 已开始将默认构建入口切换到 `tairitsu-packager`，但其组件库代码仍使用 Dioxus 风格的 rsx! 语法。
需要在 Tairitsu 侧补齐迁移支撑能力，降低改造成本。

## 协同里程碑

- M1: Hikari 网站最小页面（非 Dioxus）在 `tairitsu-packager dev` 成功启动。
- M2: Hikari 移除 `wasm-bindgen-cli` 运行依赖。
- M3: Hikari 移除 Dioxus 依赖并完成 CI 切换。

---

## ✅ 已完成：rsx! 宏控制流支持

### rsx! 控制流功能

| 功能 | 状态 | 备注 |
| ---- | ---- | ---- |
| `if`/`else` 表达式 | ✅ | 完整支持 RsxIf 结构体 |
| `match` 表达式 | ✅ | 完整支持 RsxMatch 结构体 |
| 递归解析 rsx 子元素 | ✅ | RsxChild::If, RsxChild::Match |
| 动态子元素 `{expr}` | ✅ | 支持 |
| 展开子元素 `..expr` | ✅ | RsxChild::Spread 语法 |
| 字符串属性 `"data-attr": val` | ✅ | 支持 data-* 属性 |
| `dangerous_inner_html` | ✅ | 支持原始 HTML |

### 其他已完成功能

| 功能 | 状态 | 备注 |
| ---- | ---- | ---- |
| `#[component]` 宏支持现有 Props | ✅ | 检测 `props: XxxProps` 参数，跳过自动生成 |
| `#[derive(Props)]` 宏 | ✅ | 兼容 Dioxus 语法 |
| `Props` 属性 `#[props(default)]` | ✅ | 解析支持 |
| `VNode::empty()` 方法 | ✅ | 创建空文本节点 |
| `Signal.read()`/`.write()` 方法 | ✅ | Dioxus 兼容别名 |
| `Key` 枚举 | ✅ | 键盘事件键码处理 |
| `PartialEq` for VNode/VElement | ✅ | 支持节点比较 |
| `PartialEq` for Callback | ✅ | 支持回调比较（Rc ptr_eq） |

---

## 🟡 当前状态：Hikari 组件迁移

### 剩余 Hikari 端问题

构建 hikari-components 时的错误类型分布：

| 错误类型 | 数量 | 描述 |
| -------- | ---- | ---- |
| E0308 mismatched types | 274 | 类型不匹配（需要 Hikari 端修改） |
| E0317 if missing else | 22 | rsx! 中 if 缺少 else |
| E0061 wrong argument count | 32 | 组件参数数量错误 |
| E0433 web_sys unresolved | 13 | WASM 特定代码 |
| E0277 PhantomData issues | 31 | Props 默认值问题 |
| E0432 icons import | 0 | ✅ 已修复（hikari_icons） |

### 需要在 Hikari 端处理的问题

1. **Signal 调用语法** ✅ - 已将 `signal()` 替换为 `signal.get()`
2. **图标导入** ✅ - 已将 `use icons::` 替换为 `use hikari_icons::`
3. **类型不匹配** - 需要 Hikari 组件适配 Tairitsu API
4. **web_sys 依赖** - 需要 cfg 条件编译或 WASM 抽象层

---

## ✅ Dioxus 兼容层状态

| Dioxus 类型 | Tairitsu 对应 | 状态 |
| ----------- | ------------- | ---- |
| `Element` | `VNode` | ✅ 别名已创建 |
| `#[derive(Props)]` | `#[derive(Props)]` | ✅ derive 宏已添加 |
| `#[component]` | `#[component]` | ✅ 支持现有 Props |
| `use_signal` | `use_signal` | ✅ 接受闭包参数 |
| `use_context_provider` | `provide_context` | ✅ 别名已添加 |
| `Callback<T>` | `Callback<T, ()>` | ✅ 已添加，支持 PartialEq |
| `EventHandler<T>` | `EventHandler<T>` | ✅ 已添加 |
| `use_memo` | `use_memo` | ✅ 已添加，支持 .read() |
| `use_callback` | `use_callback` | ✅ 已添加 |
| `Key` 枚举 | `Key` | ✅ 键盘事件支持 |

## 实现详情

### packages/vdom/src/vnode.rs

- `VNode::empty()` - 创建空文本节点
- `VElement::inner_html()` - 设置原始 HTML（dangerouslySetInnerHTML）
- `PartialEq` for VNode, VElement, Style, Classes

### packages/vdom/src/callback.rs

- `Callback<T, R>` - 通用回调类型，包装 `Fn(T) -> R`
- `EventHandler<T>` - Dioxus 兼容别名
- 支持 `Clone`, `Deref`, `From`, `PartialEq` trait
- 提供 `call()`, `no_arg()`, `simple()` 方法

### packages/vdom/src/events.rs

- `Key` 枚举 - 键盘键码处理
- `KeyboardEvent::key_code()` - 返回 Key 枚举

### packages/hooks/src/memo.rs

- `use_memo(compute, deps)` - 依赖驱动的响应式计算
- `Memo<T, D, F>` 结构体，支持 `Clone` 和依赖更新
- `Memo::read()` 方法 - 直接获取值

### packages/hooks/src/signal.rs

- `use_signal(|| initial)` - 接受闭包参数（Dioxus 兼容）

### packages/macros/src/rsx.rs

- 完整的 if/match 控制流解析
- `RsxChild::Spread` 语法支持 `..expr`
- `dangerous_inner_html` 属性支持
