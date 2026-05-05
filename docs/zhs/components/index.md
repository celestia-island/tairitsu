# 包分层与职责总览

为保持与 `hikari` 文档结构一致，`components` 目录在 Tairitsu 中用于说明“工程组件（包）分层”。

## Layer 1：基础运行层

- `runtime`
- `macros`
- `vdom`

## Layer 2：平台与协议层

- `web`
- `browser-worlds`
- `browser-wit-resolver`

## Layer 3：工具与交付层

- `packager`
- `e2e`
- `hooks` / `style`
- `browser-glue`（TS host）

## 进一步阅读

- [Workspace 包清单](./packages.md)
- [系统总览](../system/overview.md)
