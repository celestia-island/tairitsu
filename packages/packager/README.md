# tairitsu-packager

Build tooling and packaging for Tairitsu WASM applications.

## Overview

`tairitsu-packager` provides the CLI and build system for compiling, bundling, and deploying Tairitsu applications.

## CLI Commands

### Build

Compile your Tairitsu application:

```bash
tairitsu build
```

Options:
- `--release`: Build with optimizations
- `--target`: Specify target directory
- `--features`: Enable cargo features

### Doctor

Check your development environment:

```bash
tairitsu doctor
```

Checks for:
- Rust toolchain
- WASM target
- WIT dependencies
- Browser glue setup

### Init

Create a new Tairitsu project:

```bash
tairitsu init my-app
cd my-app
```

### Dev

Run development server:

```bash
tairitsu dev
```

Features:
- Hot module reloading
- Fast rebuilds
- Local development server

## Configuration

Configuration is loaded from `tairitsu.toml`:

```toml
[package]
name = "my-app"
version = "0.1.0"

[build]
target = "wasm32-wasip2"
features = ["wit-bindings"]

[dev]
port = 8080
hot_reload = true

[assets]
# Static files to include
styles = ["styles/**/*.css"]
scripts = ["scripts/**/*.js"]
```

## Build Process

1. **Compilation**: Rust code compiled to WASM component
2. **WIT Generation**: WIT bindings generated from WIT files
3. **Bundling**: Assets and styles bundled together
4. **Optimization**: WASM optimized for size and performance
5. **Output**: Ready-to-deploy application

## Output Structure

```
target/
├── tairitsu-wit/           # WIT package cache
├── release/                # Compiled WASM
│   ├── app.wasm
│   ├── app.js
│   └── index.html
└── dist/                   # Production bundle
    ├── assets/
    │   ├── styles.css
    │   └── scripts.js
    └── app.wasm
```

## Features

### Style Injection

Automatic CSS injection from Rust:

```rust
// In your component
let styles = tairitsu_style::css!(
    ".container { background: blue; }"
);
```

### Resource Indexing

Automatic asset discovery and bundling:
- Images
- Fonts
- Static files
- Data files

### SSR/SSG

Support for server-side rendering and static site generation.

## Advanced Usage

### Custom Build Steps

Define custom build commands in `tairitsu.toml`:

```toml
[build.custom]
pre-build = ["npm run generate-assets"]
post-build = ["npm run optimize"]
```

### Environment Variables

```bash
export TAIRITSU_WIT_REGISTRY="https://my-registry.com"
export TAIRITSU_WIT_OFFLINE="1"
tairitsu build
```

## Troubleshooting

### Build Errors

If build fails, try:

```bash
tairitsu doctor
tairitsu clean
tairitsu build
```

### WIT Resolution Issues

Clear the WIT cache:

```bash
rm -rf target/tairitsu-wit
tairitsu build
```

## See Also

- [Build Guide](../../docs/en-US/guides/build-test-release.md): Detailed build instructions
- [WIT Pipeline](../../docs/en-US/system/wit-pipeline.md): WIT generation process
- [Deployment](../../docs/en-US/guides/deployment.md): Deploying your app
