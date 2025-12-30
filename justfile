# Tairitsu Build Automation with Just
# https://github.com/casey/just

# Default recipe to display help
default:
    just --list

# Install required tools
install-tools:
    rustup target add wasm32-wasip1
    rustup component add rustfmt --toolchain nightly
    rustup component add clippy

# Development setup (install tools and build)
setup: install-tools
    cargo build --release --all

# Run all tests
test:
    cargo test --all --all-features

# Run formatting check
fmt-check:
    cargo +nightly fmt --all -- --check --unstable-features

# Format code
fmt:
    cargo +nightly fmt --all -- --unstable-features

# Run clippy lints
clippy:
    cargo clippy --all --all-features -- -D warnings

# Run all CI checks (format, clippy, test)
ci: fmt-check clippy test

# Clean build artifacts
clean:
    cargo clean

# Build documentation
doc:
    cargo doc --no-deps --all-features

# Open documentation in browser
doc-open: doc
    cargo doc --no-deps --all-features --open

# Check project size and statistics
stats:
    python3 scripts/project_stats.py

# Build simple example WASM module
build-simple-wasm:
    cargo build --target wasm32-wasip1 --release --package tairitsu-example-wit-native-simple --lib

# Build macro example WASM module
build-macro-wasm:
    cargo build --target wasm32-wasip1 --release --package tairitsu-example-wit-native-macro --lib

# Run simple demo (trait-based composable WIT interfaces)
run-simple-demo:
    cargo run --package tairitsu-example-wit-native-simple --bin simple-demo

# Run simple host (full integration example)
run-simple-host:
    cargo run --package tairitsu-example-wit-native-simple --bin simple-host

# Run simple example with WASM (full bidirectional communication)
run-simple-wasm: build-simple-wasm
    cargo run --package tairitsu-example-wit-native-simple --bin simple-wasm-host

# Run macro demo (macro-generated WIT interfaces)
run-macro-demo:
    cargo run --package tairitsu-example-wit-native-macro --bin macro-demo

# Run macro host (full integration example)
run-macro-host:
    cargo run --package tairitsu-example-wit-native-macro --bin macro-host

# Run macro example with WASM (full bidirectional communication)
run-macro-wasm: build-macro-wasm
    cargo run --package tairitsu-example-wit-native-macro --bin macro-wasm-host

# Run all examples
run-all: run-simple-demo run-simple-host run-simple-wasm run-macro-demo run-macro-host run-macro-wasm

# Run all WASM examples
run-all-wasm: run-simple-wasm run-macro-wasm

# Watch and rebuild on file changes (requires cargo-watch)
watch:
    cargo watch -x check
