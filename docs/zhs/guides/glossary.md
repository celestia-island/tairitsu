# 术语对照表

本表用于多语言文档的一致性翻译。

| 中文 | 英文 | 说明 |
|---|---|---|
| WIT | WIT | WebAssembly Interface Types |
| 组件模型 | Component Model | 可组合 wasm 组件的 ABI 与链接模型 |
| 宿主导入 | Host import | 由宿主提供给组件的函数 |
| 客体导出 | Guest export | 由组件暴露给宿主的函数 |
| web 后端 | web backend | wasm-bindgen/web-sys 路径 |
| wit-bindings 后端 | wit-bindings backend | wit-bindgen + wasm32-wasip2 路径 |
| 容器 | Container | 组件运行时实例 |
| 镜像 | Image | wasm 组件二进制抽象 |
| 解析器 | Resolver | WIT 包解析与缓存模块 |
| 注册表覆盖 | Registry override | TAIRITSU_WIT_REGISTRY 环境变量机制 |

## 一致性检查建议

1. 新增术语时，优先在本表登记后再翻译。
2. 每个语种至少保留一个入口页并指向基线文档。
3. 发布前抽样检查 en-US、ja-JP、zh-CHT 的术语一致性。
