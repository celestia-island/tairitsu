# ビルド・テスト・リリース

必須ゲート:

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cd packages/browser-glue && npm run typecheck
cargo test -p tairitsu-e2e
```
