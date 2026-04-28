# Build, Test, and Release

## Daily commands

```bash
just build-dev
just test
just clippy
just fmt
```

## Required gates before merge

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
cd packages/browser-glue && npm run typecheck
cargo test -p tairitsu-e2e
```

## WIT generation pipeline

```bash
just gen-wit-fetch
just gen-wit
just gen-wit-all
```
