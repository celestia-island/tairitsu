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
