# 构建、测试与发布

## 日常开发命令

```bash
just build-dev
just test
just clippy
just fmt
```

## 关键验证门

建议在 PR 合并前至少运行：

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

浏览器 glue 层附加检查：

```bash
cd packages/browser-glue
npm run typecheck
```

## WIT 生成流水线

```bash
just gen-wit-fetch
just gen-wit
# 或一键
just gen-wit-all
```

## 发布建议流程（预 1.0）

1. 更新文档与版本策略说明
2. 运行全量检查并保存摘要
3. 检查 WIT 改动是否涉及破坏性变更
4. 同步 `browser-glue` 对应实现
5. 生成 release note（标注兼容性风险）

## CI 建议

- 拆分 Rust 与 TypeScript 作业
- 对 `wit/generated` 变更启用额外审查
- 对 `docs/zh-CHS` 变更执行链接与路径检查
