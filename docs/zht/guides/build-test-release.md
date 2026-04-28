# 建置、測試與發佈

合併前必跑:

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cd packages/browser-glue && npm run typecheck
cargo test -p tairitsu-e2e
```
