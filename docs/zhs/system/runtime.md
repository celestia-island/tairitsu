# 运行时与容器模型

`packages/runtime` 是 Tairitsu 的核心执行层，提供类似镜像/容器的抽象。

## 核心对象

- `Image`：WASM 组件二进制及其元数据
- `Container`：可执行实例，封装存储、链接器、调用上下文
- `ContainerBuilder`：配置式构建器，支持注入 guest 初始化与 host linker

## 典型流程

1. 读取 WASM 组件并构建 `Image`
2. 通过 `Container::builder(image)` 配置执行上下文
3. 注册 host 导入、初始化 guest 实例
4. 通过 typed 或 dynamic 路径调用导出函数

## 调用模式

- 编译期绑定：`wasmtime::component::bindgen!` 生成强类型 API
- 运行期动态：RON / Binary canonical ABI

## 动态调用能力

- 导出函数发现与描述
- 结构化参数序列化/反序列化
- 基础类型与复杂嵌套类型支持

## 实践建议

- 固定接口、高性能要求场景：优先编译期绑定
- 插件化、热插拔场景：优先动态调用
- 对外开放接口：为参数与返回值定义稳定版本策略
