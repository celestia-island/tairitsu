# W3C WebIDL 到 WIT 的生成流水线

Tairitsu 通过脚本化流程将标准 WebIDL 转换为可消费的 WIT 包。

## 数据源

- 主数据源：`w3c/webref`（curated）
- 缓存目录：`target/tairitsu-wit/webidl-cache/`

## 主要脚本

- `scripts/fetch_w3c_idl.py`：拉取 IDL 源
- `scripts/webidl_to_wit.py`：解析并生成 WIT
- `scripts/gen_wit_from_webidl.py`：一键编排

## 关键规则

- interface 对象使用 `u64` 句柄模式
- `DOMString/USVString` → `string`
- `sequence<T>` → `list<T>`
- nullable `T?` → `option<T>`

## 常用命令

```bash
just gen-wit-fetch
just gen-wit
just gen-wit-all
```

## 输出位置

- 手写基线：`packages/browser-worlds/wit/*.wit`
- 自动生成：`packages/browser-worlds/wit/generated/*.wit`

## 运维建议

- 变更生成规则时，必须同时更新文档与示例
- 对 generated 目录变更启用更严格 code review
