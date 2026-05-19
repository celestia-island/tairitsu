# 系统概览

Tairitsu 是一个基于 WASM Component Model 的全栈框架。同一个 WASM 组件可以在服务端（Container 运行时）、浏览器（VDOM 运行时）和边缘节点运行——全部通过相同的 WIT 接口定义。

## 四层架构

```mermaid
graph TD
    subgraph L4["4. 工具层"]
        T1["packager、dev server、MCP、视觉回归、脚本"]
    end
    subgraph L3["3. 平台层"]
        P1["WitPlatform（WIT 绑定）"]
        P2["WebPlatform（web-sys）"]
        P3["browser-glue（TypeScript ↔ WIT 桥接）"]
    end
    subgraph L2["2. 运行时层"]
        R1["Container / Registry / Image 生命周期"]
        R2["WIT 绑定、动态调用（RON + binary canonical ABI）"]
    end
    subgraph L1["1. 接口层"]
        I1["WIT world 定义、browser-worlds"]
        I2["W3C WebIDL → WIT 代码生成管线"]
    end
    L1 --> L2 --> L3 --> L4
```

## 请求流程

### 浏览器（客户端路径）

```mermaid
graph TD
    A["用户点击按钮"] --> B["DOM 事件触发"]
    B --> C["browser-glue 捕获事件，转换为 WIT ABI"]
    C --> D["WASM 组件接收类型化事件<br/>(MouseEvent/KeyboardEvent/...)"]
    D --> E["Signal 更新 → VDOM diff → Patch 操作"]
    E --> F["Patch 通过 DomOps 应用 → DOM 更新"]
```

### 服务端（SSR 路径）

```mermaid
graph TD
    A["HTTP 请求到达"] --> B["axum dev server 或<br/>独立 wasmtime 宿主"]
    B --> C["Container 实例化 WASM 组件"]
    C --> D["组件通过 WIT 调用渲染 VNode 树"]
    D --> E["SSR 引擎序列化为 HTML 字符串"]
    E --> F["流式响应发送给客户端"]
```

## 核心设计决策

### 为什么选择 Component Model 而非 wasm-bindgen？

| wasm-bindgen 路径 | WIT 路径（Tairitsu） |
|:--|:--|
| Rust → wasm-bindgen → JS shim → 浏览器 | Rust → WIT → canonical ABI → 浏览器（未来原生） |
| 与 JS 运行时紧密耦合 | 语言无关的 WIT 接口 |
| 无法在服务端复用 | 同一组件可在任何 wasmtime 宿主运行 |
| 成熟稳定的生态（Leptos, Dioxus, Yew） | 新兴、面向未来 |

Tairitsu 押注 Component Model 将成为浏览器-wasm 互操作的标准，从而消除对 wasm-bindgen JS 胶水层的需求。

### 为什么采用 Docker-like 的 Image/Container/Registry？

WASM 组件需要类似容器的生命周期管理：

- **Image** = 编译后的 `.wasm` 二进制 + 元数据（类似 Docker 镜像）
- **Container** = 运行中实例，带宿主提供的 WIT imports（类似 Docker 容器）
- **Registry** = 镜像和活跃容器的集合（类似 Docker daemon）

这一模型支持：
- 开发时热重载（替换 Image，保留 Container）
- 版本化部署（标记镜像，回滚）
- 多租户隔离（分离容器，共享宿主）
- 动态调用（在运行时调用运行中的组件）

## 下一步

- [运行时与容器模型](runtime.md) — 深入了解 Container/Image/Registry
- [VDOM 与渲染](vdom.md) — 浏览器端 VDOM 工作原理
- [WIT 管线](wit-pipeline.md) — W3C WebIDL → WIT 生成
- [Web 后端](web-backends.md) — 双 WitPlatform / WebPlatform 策略
- [浏览器胶水层](browser-glue.md) — TypeScript 桥接层
