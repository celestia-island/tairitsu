# Workspace 包清单

## Rust 包（packages）

| 包名 | 主要职责 |
|---|---|
| `runtime` | WASM 组件执行、容器生命周期、动态调用 |
| `macros` | `rsx!`、`wit_interface!`、`wit_world!` 等宏 |
| `vdom` | 平台无关 VDOM 抽象、事件与 diff |
| `web` | 浏览器平台适配（`web` 与 `wit-bindings`） |
| `hooks` | 状态/副作用等响应式 hooks |
| `style` | 样式相关辅助能力 |
| `packager` | CLI、WIT fetch/verify/list 能力 |
| `browser-worlds` | 手写与生成 WIT 资源集合 |
| `browser-wit-resolver` | WIT registry 解析、缓存、网络拉取 |
| `e2e` | 端到端测试入口与支持代码 |
| `testing` | 测试支持（当前未纳入 workspace members） |

## TypeScript 包

| 包名 | 主要职责 |
|---|---|
| `browser-glue` | 在浏览器/Node.js 中实现 `tairitsu-browser:full` 宿主导入 |

## 示例包（examples）

| 示例 | 定位 |
|---|---|
| `wit-native-macro` | 宏驱动接口定义路径 |
| `wit-native-simple` | trait 驱动接口定义路径 |
| `wit-runtime` | 运行时加载 WIT |
| `wit-compile-time` | 编译期绑定路径 |
| `wit-dynamic-advanced` | 高级动态调用（RON + Binary） |
| `website` | 展示与验证用网站示例 |
