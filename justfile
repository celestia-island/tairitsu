# Tairitsu Build Automation with Just
# https://github.com/casey/just

# Default recipe to display help
default:
    @just --list

# Build the WASM guest module
build-wasm:
    @echo "Building WASM guest module..."
    cargo build --target wasm32-wasip1 --release --package tairitsu-example-hybrid

# Build the native host binary
build-host:
    @echo "Building native host binary..."
    cargo build --release --package tairitsu-example-hybrid --bin host --features host-binary

# Build both WASM and host
build: build-wasm build-host
    @echo "✓ Build complete"

# Run the hybrid example (builds WASM first, then runs host)
run: build-wasm
    @echo ""
    @echo "Running native host..."
    cargo run --package tairitsu-example-hybrid --bin host --features host-binary

# Run all tests
test:
    @echo "Running tests..."
    cargo test --all --all-features

# Run formatting check
fmt-check:
    @echo "Checking code formatting..."
    cargo +nightly fmt --all -- --check --unstable-features

# Format code
fmt:
    @echo "Formatting code..."
    cargo +nightly fmt --all -- --unstable-features

# Run clippy lints
clippy:
    @echo "Running clippy..."
    cargo clippy --all --all-features -- -D warnings

# Run all CI checks (format, clippy, test)
ci: fmt-check clippy test
    @echo "✓ All CI checks passed"

# Clean build artifacts
clean:
    @echo "Cleaning build artifacts..."
    cargo clean

# Build documentation
doc:
    @echo "Building documentation..."
    cargo doc --no-deps --all-features

# Open documentation in browser
doc-open: doc
    cargo doc --no-deps --all-features --open

# Install required tools
install-tools:
    @echo "Installing required tools..."
    rustup target add wasm32-wasip1
    rustup component add rustfmt --toolchain nightly
    rustup component add clippy

# Development setup (install tools and build)
setup: install-tools build
    @echo "✓ Development environment ready"

# Watch and rebuild on file changes (requires cargo-watch)
watch:
    @echo "Watching for changes..."
    cargo watch -x "build --package tairitsu-example-hybrid --bin host --features host-binary"

# Check project size and statistics
stats:
    @python3 scripts/project_stats.py
