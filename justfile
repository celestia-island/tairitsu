# Tairitsu Build System
#
# Usage:
#   just <recipe>        - Run specified recipe
#   just --list          - List all available recipes
#   just --summary       - Briefly list all recipe names
#
# Main tasks:
#   just build           - Build everything (Release)
#   just build-dev       - Build everything (Debug)
#   just test            - Run all checks (check + clippy + examples verification)
#   just fmt             - Format code
#   just clippy          - Run Clippy checks
#   just clean           - Clean build artifacts

# Configure Windows to use PowerShell (UTF-8 encoding)
set windows-shell := ["pwsh.exe", "-NoLogo", "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; $PSDefaultParameterValues['*:Encoding'] = 'utf8';"]

# Default: show help information
default:
    @just --list

# ============================================================================
# Tool installation and initialization
# ============================================================================

# Install required tools
install-tools:
    rustup target add wasm32-wasip2
    rustup component add rustfmt --toolchain nightly
    rustup component add clippy
    python scripts/download_wasi_adapters.py

# Development environment setup (install tools and build)
setup: install-tools
    cargo build --release --all

# ============================================================================
# Cleanup tasks
# ============================================================================

# Clean all build artifacts
clean:
    cargo clean

# ============================================================================
# Build tasks
# ============================================================================

# Build everything (Debug mode)
build-dev:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Building all (Debug mode)..."
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cargo build --all

# Build everything (Release mode)
build:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Building all (Release mode)..."
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cargo build --release --all

# Build simple example WASM module
build-simple-wasm:
    @echo "Building simple example WASM..."
    cargo build --target wasm32-wasip2 --release --package tairitsu-example-wit-native-simple --lib

# Build macro example WASM module
build-macro-wasm:
    @echo "Building macro example WASM..."
    cargo build --target wasm32-wasip2 --release --package tairitsu-example-wit-native-macro --lib

# ============================================================================
# Run examples
# ============================================================================

# Run simple demo (trait-based composable WIT interfaces)
run-simple-demo:
    @echo "Running simple demo..."
    cargo run --package tairitsu-example-wit-native-simple --bin simple-demo

# Run simple host (complete integration example)
run-simple-host:
    @echo "Running simple host..."
    cargo run --package tairitsu-example-wit-native-simple --bin simple-host

# Run simple WASM example (complete bidirectional communication)
run-simple-wasm:
    @echo "Building simple WASM..."
    cargo build --target wasm32-wasip2 --release --package tairitsu-example-wit-native-simple --lib
    @echo "Running simple WASM example..."
    cargo run --package tairitsu-example-wit-native-simple --bin simple-wasm-host

# Run macro demo (macro-generated WIT interfaces)
run-macro-demo:
    @echo "Running macro demo..."
    cargo run --package tairitsu-example-wit-native-macro --bin macro-demo

# Run macro host (complete integration example)
run-macro-host:
    @echo "Running macro host..."
    cargo run --package tairitsu-example-wit-native-macro --bin macro-host

# Run macro WASM example (complete bidirectional communication)
run-macro-wasm:
    @echo "Building macro WASM..."
    cargo build --target wasm32-wasip2 --release --package tairitsu-example-wit-native-macro --lib
    @echo "Running macro WASM example..."
    cargo run --package tairitsu-example-wit-native-macro --bin macro-wasm-host

# Run dynamic advanced demo (RON + complex types)
run-dynamic-advanced:
    @echo "Running dynamic advanced example..."
    cargo run --package tairitsu-example-wit-dynamic-advanced --bin dynamic-advanced-demo

# Run all examples
run-all: run-simple-demo run-simple-host run-simple-wasm run-macro-demo run-macro-host run-macro-wasm run-dynamic-advanced

# Run all WASM examples
run-all-wasm:
    @echo "Building all WASM modules..."
    cargo build --target wasm32-wasip2 --release --package tairitsu-example-wit-native-simple --lib
    cargo build --target wasm32-wasip2 --release --package tairitsu-example-wit-native-macro --lib
    @echo "Running simple WASM example..."
    cargo run --package tairitsu-example-wit-native-simple --bin simple-wasm-host
    @echo "Running macro WASM example..."
    cargo run --package tairitsu-example-wit-native-macro --bin macro-wasm-host

# ============================================================================
# Test tasks
# ============================================================================

# Run all unit tests
test-unit:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Running unit tests..."
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cargo test --workspace --lib

# Run all unit tests with dynamic feature
test-unit-dynamic:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Running unit tests with dynamic feature..."
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cargo test --workspace --lib --features dynamic

# Run integration tests
test-integration:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Running integration tests..."
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cargo test --package tairitsu --test integration_test --features dynamic

# Build all WASM components for testing
build-test-wasm:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Building test WASM components..."
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Building wit-native-simple WASM..."
    cargo build --target wasm32-wasip2 --release --package tairitsu-example-wit-native-simple --lib
    @echo "Building wit-native-macro WASM..."
    cargo build --target wasm32-wasip2 --release --package tairitsu-example-wit-native-macro --lib
    @echo "✅ All test WASM components built"

# Run full test suite (unit + integration + WASM)
test-full: build-test-wasm test-unit test-unit-dynamic test-integration
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "✅ Full test suite completed!"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Run all checks (cargo check + clippy + run examples)
test:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Running comprehensive checks..."
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Step 1/10: Checking code compilation..."
    cargo check --workspace --all-targets
    @echo "✅ Check passed"
    @echo ""
    @echo "Step 2/10: Running Clippy..."
    cargo clippy --workspace --all-targets -- -D warnings
    @echo "✅ Clippy passed"
    @echo ""
    @echo "Step 3/10: Running compile-time demo..."
    cargo run --package tairitsu-example-wit-compile-time --bin compile-time-demo
    @echo "✅ Compile-time demo passed"
    @echo ""
    @echo "Step 4/10: Running runtime demo..."
    cargo run --package tairitsu-example-wit-runtime --bin runtime-demo
    @echo "✅ Runtime demo passed"
    @echo ""
    @echo "Step 5/10: Running dynamic advanced demo..."
    cargo run --package tairitsu-example-wit-dynamic-advanced --bin dynamic-advanced-demo
    @echo "✅ Dynamic advanced demo passed"
    @echo ""
    @echo "Step 6/10: Building simple WASM module..."
    cargo build --target wasm32-wasip2 --release --package tairitsu-example-wit-native-simple --lib
    @echo "✅ Simple WASM built"
    @echo ""
    @echo "Step 7/10: Running simple WASM host..."
    cargo run --package tairitsu-example-wit-native-simple --bin simple-wasm-host
    @echo "✅ Simple WASM host passed"
    @echo ""
    @echo "Step 8/10: Building macro WASM module..."
    cargo build --target wasm32-wasip2 --release --package tairitsu-example-wit-native-macro --lib
    @echo "✅ Macro WASM built"
    @echo ""
    @echo "Step 9/10: Running macro WASM host..."
    cargo run --package tairitsu-example-wit-native-macro --bin macro-wasm-host
    @echo "✅ Macro WASM host passed"
    @echo ""
    @echo "Step 10/10: Running unit tests..."
    cargo test --workspace --lib --features dynamic
    @echo "✅ Unit tests passed"
    @echo ""
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "✅ All checks passed successfully!"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# ============================================================================
# Code quality checks
# ============================================================================

# Run formatting check
fmt-check:
    @echo "Checking code formatting..."
    cargo +nightly fmt --all -- --check --unstable-features

# Format all code
fmt:
    @echo "Formatting all code..."
    cargo +nightly fmt --all -- --unstable-features

# Run Clippy checks
clippy:
    @echo "Running Clippy..."
    cargo clippy --all --all-features -- -D warnings

# CI checks (format check + Clippy + test)
ci: fmt-check clippy test
    @echo "✅ CI checks passed"

# ============================================================================
# Watch tasks (using cargo-watch)
# ============================================================================

# Watch code changes and auto-check (using cargo-watch)
watch:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Watching for changes..."
    @echo "Press Ctrl+C to stop"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cargo watch -x check

# ============================================================================
# Documentation tasks
# ============================================================================

# Build documentation
doc:
    @echo "Building documentation..."
    cargo doc --no-deps --all-features

# Open documentation in browser
doc-open: doc
    @echo "Opening documentation in browser..."
    cargo doc --no-deps --all-features --open

# ============================================================================
# Utilities
# ============================================================================

# Update all dependencies
update:
    @echo "Updating dependencies..."
    cargo update

# Show project information
info:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Tairitsu Build System"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @rustc --version
    @cargo --version
    @just --version
    @echo ""
    @echo "Available examples:"
    @echo "  - wit-native-simple: trait-based composable WIT interfaces"
    @echo "  - wit-native-macro: macro-generated WIT interfaces"
    @echo ""
    @echo "Package structure:"
    @echo "  - packages/runtime: Tairitsu core runtime (includes macro re-exports)"
    @echo "  - packages/macros:  Procedural macros (internal package, re-exported via runtime)"
    @echo "  - examples/wit-native-simple:  Simple example"
    @echo "  - examples/wit-native-macro:   Macro example"
