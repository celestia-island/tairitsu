# Tairitsu <- Hikari 迁移协同计划

## 背景

Hikari 已开始将默认构建入口切换到 `tairitsu-packager`，但其组件库代码仍使用 Dioxus 风格的 rsx! 语法。
需要在 Tairitsu 侧补齐迁移支撑能力，降低改造成本。

## 协同里程碑

- M1: Hikari 网站最小页面（非 Dioxus）在 `tairitsu-packager dev` 成功启动。
- M2: Hikari 移除 `wasm-bindgen-cli` 运行依赖。
- M3: Hikari 移除 Dioxus 依赖并完成 CI 切换。

---

## ✅ 已完成：Tairitsu Dioxus 兼容层

### rsx! 宏功能

| 功能 | 状态 | 备注 |
| ---- | ---- | ---- |
| `if`/`else` 表达式 | ✅ | 完整支持 RsxIf 结构体，无 else 时返回 VNode::empty() |
| `match` 表达式 | ✅ | 完整支持 RsxMatch 结构体 |
| 递归解析 rsx 子元素 | ✅ | RsxChild::If, RsxChild::Match |
| 动态子元素 `{expr}` | ✅ | 支持 |
| 展开子元素 `..expr` | ✅ | RsxChild::Spread 语法 |
| 字符串属性 `"data-attr": val` | ✅ | 支持 data-* 属性 |
| `dangerous_inner_html` | ✅ | 支持原始 HTML |
| 属性泛型支持 | ✅ | `.attr()` 接受 `impl ToString` |

### component 宏功能

| 功能 | 状态 | 备注 |
| ---- | ---- | ---- |
| `#[component]` 宏支持现有 Props | ✅ | 检测 `props: XxxProps` 参数，跳过自动生成 |
| 无默认值参数 | ✅ | 使用 `Option<T>` 包装 |
| `#[derive(Props)]` 宏 | ✅ | 兼容 Dioxus 语法 |
| `Props` 属性 `#[props(default)]` | ✅ | 解析支持 |

### VNode/VElement 功能

| 功能 | 状态 | 备注 |
| ---- | ---- | ---- |
| `VNode::empty()` 方法 | ✅ | 创建空文本节点 |
| `VElement::inner_html()` 方法 | ✅ | 支持原始 HTML |
| `VElement::attr()` 泛型 | ✅ | 接受 `impl ToString` |
| `PartialEq` for VNode/VElement | ✅ | 支持节点比较 |
| `PartialEq` for Callback | ✅ | 支持回调比较 |

### Signal/Hooks 功能

| 功能 | 状态 | 备注 |
| ---- | ---- | ---- |
| `Signal::read()` 方法 | ✅ | 返回 T（Dioxus 兼容） |
| `Signal::write()` 方法 | ✅ | 设置值（Dioxus 兼容） |
| `use_signal` 闭包参数 | ✅ | 接受 `|| T` 闭包 |
| `Memo::read()` 方法 | ✅ | 直接获取值 |
| `use_context_provider` 别名 | ✅ | provide_context 的别名 |

### 事件系统

| 功能 | 状态 | 备注 |
| ---- | ---- | ---- |
| `Key` 枚举 | ✅ | 键盘键码处理 |
| `KeyboardEvent::key_code()` | ✅ | 返回 Key 枚举 |
| `Callback`/`EventHandler` | ✅ | Clone, PartialEq 支持 |

---

## 🟡 Hikari 端剩余工作

### 当前编译错误（~200 个）

| 错误类型 | 数量 | 描述 | 解决方案 |
| -------- | ---- | ---- | -------- |
| E0308 mismatched types | 56 | 属性类型不匹配 | 需为更多类型实现 ToString |
| E0061 wrong arg count | 32 | 函数/方法参数数量 | 需检查 API 签名 |
| unexpected token | 17 | rsx! 宏语法 | 可能是属性解析问题 |
| expected pattern | 16 | rsx! 宏解析 | for 循环等未实现 |
| Option<String>: ToString | 16 | Option 未实现 ToString | Tairitsu 需特殊处理 |
| web_sys unresolved | 13 | WASM 特定代码 | 需 cfg 条件编译 |
| VNode: ToString | 8 | children 作为属性传递 | 需特殊处理 |
| Event type missing | 6 | 缺少 Event 类型 | 需添加 Event 类型 |

### 需要在 Hikari 端处理

1. **Option<T> 属性** - 需要为 `Option<T>` 实现特殊处理，当 `Some` 时渲染属性
2. **web_sys 依赖** - 需要 cfg 条件编译或创建抽象层
3. **Event 类型** - hikari 使用的事件类型需要映射到 Tairitsu
4. **for 循环** - rsx! 宏不支持 `for x in iter { ... }` 语法
5. **剩余类型 ToString** - 为 ArrowDirection, MdiIcon 等实现 ToString

---

## 🔴 Tairitsu 待实现功能

### 高优先级

1. **Option<T> 属性处理**
   - 当值为 `Some(v)` 时渲染属性
   - 当值为 `None` 时不渲染属性

2. **for 循环支持**
   ```rust
   rsx! {
       for item in items {
           Item { item }
       }
   }
   ```

3. **Event 类型导出**
   - 导出 `Event` 类型别名
   - 或映射到 Tairitsu 的 `EventData`

### 中优先级

4. **children 作为属性**
   - 某些组件使用 `children: Vec<VNode>` 作为属性
   - 需要支持将 VNode 转换为字符串属性值

5. **展开运算符增强**
   - `..props` 语法完整支持
   - 自动展开所有 Props 字段

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
