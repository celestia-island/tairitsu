# 快速開始

```bash
just install-tools
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cd packages/browser-glue && npm run typecheck
cd ../.. && cargo test -p tairitsu-e2e
```
