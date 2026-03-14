# 版本与兼容性策略

本页是简体中文入口，完整策略见 [docs/versioning.md](../../versioning.md)。

## 版本面向对象

- Rust crates（`packages/*`）
- WIT packages（`packages/browser-worlds/wit/*`）
- TypeScript host（`packages/browser-glue`）

## 核心原则

1. Rust 包与 WIT 包独立演进
2. WIT 破坏性变更必须提升兼容级别并记录
3. `wit-bindings` 与 `wasm-bindgen` 版本节奏解耦
4. `TAIRITSU_WIT_REGISTRY` 支持私有/本地注册表覆盖

## 维护要求

- 任何 public trait/WIT 签名变更都要同步文档
- 生成规则变更需说明缓存与重建影响
