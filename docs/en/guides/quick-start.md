# Quick Start

## Prerequisites
- Rust (stable recommended)
- just
- Python 3
- Node.js (for browser-glue)

## Setup

```bash
just install-tools
```

## Verify the workspace

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Create a new project

```bash
cargo install tairitsu-packager
tairitsu init my-app
cd my-app
```

The generated `Cargo.toml` includes a `[profile.dev-wasm]` section (required
for debug WASM builds). The `src/lib.rs` bootstraps the app via
`WitPlatform::new()` + `mount_vnode_to_app()`.

```bash
tairitsu dev
```

Open `http://localhost:3000` to see the running app.

## Verify browser glue

```bash
cd packages/browser-glue
npm run typecheck
```

## Verify E2E package

```bash
cd ../..
cargo test -p tairitsu-e2e
```
