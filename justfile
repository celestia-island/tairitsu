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
#   just test            - Run all tests
#   just e2e             - Run all E2E tests
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
    rustup target add wasm32-wasip1
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
    cargo build --target wasm32-wasip1 --release --package tairitsu-example-wit-native-simple --lib

# Build macro example WASM module
build-macro-wasm:
    @echo "Building macro example WASM..."
    cargo build --target wasm32-wasip1 --release --package tairitsu-example-wit-native-macro --lib

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
run-simple-wasm: build-simple-wasm
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
run-macro-wasm: build-macro-wasm
    @echo "Running macro WASM example..."
    cargo run --package tairitsu-example-wit-native-macro --bin macro-wasm-host

# Run all examples
run-all: run-simple-demo run-simple-host run-simple-wasm run-macro-demo run-macro-host run-macro-wasm

# Run all WASM examples
run-all-wasm: run-simple-wasm run-macro-wasm

# ============================================================================
# Test tasks
# ============================================================================

# Run all tests
test:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Running all tests..."
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cargo test --all --all-features

# Run all E2E tests
e2e:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Running all E2E tests..."
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cargo test --package tairitsu --test e2e

# Run all unit tests and E2E tests
test-full: test e2e
    @echo "✅ Full test suite completed"

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

# CI checks (format check + Clippy + tests)
ci: fmt-check clippy test
    @echo "✅ CI checks passed"

# CI full pipeline (checks + E2E tests)
ci-full: fmt-check clippy test-full
    @echo "✅ CI full pipeline passed"

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
