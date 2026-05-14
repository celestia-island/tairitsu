# Quick Start

Get Tairitsu running in 5 minutes.

## Prerequisites

- **Rust** (stable) — [rustup.rs](https://rustup.rs)
- **just** — `cargo install just` or [github.com/casey/just](https://github.com/casey/just)
- **Python 3** — for WIT generation scripts
- **Node.js** — for browser-glue TypeScript runtime

## Install the CLI

```bash
# Build and install the tairitsu CLI globally
just install-packager

# Verify
tairitsu --version
```

## Create a Project

```bash
tairitsu new my-app
cd my-app
```

This scaffolds a Tairitsu project with:
- `Cargo.toml` with `[profile.dev-wasm]` for WASM builds
- `src/lib.rs` bootstrap code
- `justfile` with `just dev` / `just build`

## Start Developing

```bash
# Development server with hot reload
tairitsu dev
```

Open `http://localhost:3000`. Edit `src/lib.rs` — changes appear instantly.

## Build for Production

```bash
tairitsu build
```

Outputs optimized WASM components and static assets in `dist/`.

## Verify the Workspace (for contributors)

```bash
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

## Verify Browser Glue

```bash
cd packages/browser-glue
npm run typecheck
```

## Verify E2E Tests

```bash
cargo test -p tairitsu-e2e
```

## Next Steps

- [Getting Started Tutorial](getting-started.md) — build a full app from scratch
- [System Overview](../system/overview.md) — understand the architecture
- [From Dioxus to Tairitsu](migration/dioxus-to-tairitsu.md) — migrate an existing app
