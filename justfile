# Tairitsu Build System
#
# Usage:
#   just <recipe>        - Run specified recipe
#   just --list          - List all available recipes
#   just --summary       - Briefly list all recipe names
#
# Main tasks:
#   just build           - Build everything (Release)
#   just init            - Install JS dependencies (auto-detects pnpm/yarn/npm)
#   just build           - Build everything (Release, runs init first)
#   just build-dev       - Build everything (Debug, runs init first)
#   just test            - Run all checks (check + clippy + examples verification)
#   just fmt             - Format code
#   just clippy          - Run Clippy checks
#   just clean           - Clean build artifacts
#   just install-packager- Install tairitsu CLI to ~/.cargo/bin
#
# WIT generation (W3C WebIDL → WIT):
#   just wit-gen         - Full pipeline: fetch 50 specs + generate 18 domain WIT files
#   just wit-stats       - Show per-domain interface coverage statistics
#   just gen-wit-all     - Alternative pipeline (simpler, fewer specs, idl-cache/)

# Configure Windows to use PowerShell (UTF-8 encoding)
set windows-shell := ["pwsh.exe", "-NoLogo", "-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", "[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; $PSDefaultParameterValues['*:Encoding'] = 'utf8';"]

# Python interpreter — Windows ships as 'python', Unix as 'python3'
python := if os_family() == "windows" { "python" } else { "python3" }

# Default: show help information
default:
    @just --list

# ============================================================================
# Tool installation and initialization
# ============================================================================

# Install required Rust toolchain components
install-tools:
    rustup target add wasm32-wasip2
    rustup component add rustfmt --toolchain nightly
    rustup component add clippy
    {{python}} scripts/download_wasi_adapters.py

# Build browser-glue runtime bundle (IIFE for HTML <script> tag)
build-glue-runtime:
    mkdir -p packages/browser-glue/dist
    npx esbuild packages/browser-glue/src/runtime/index.ts --bundle --outfile=packages/browser-glue/dist/runtime.js --format=iife --platform=browser

# Install tairitsu-packager CLI binary (tairitsu) to ~/.cargo/bin
install-packager: (build-glue-runtime)
    cargo build --release --package tairitsu-packager
    {{python}} scripts/install_packager.py

# Development environment setup (install tools and build)
setup: install-tools init
    cargo build --release --all

# ============================================================================
# JS / Node dependency initialization
# ============================================================================

# Install Node.js dependencies for packages/browser-glue (auto-detects pnpm/yarn/npm)
init:
    {{python}} scripts/init_browser_glue.py

# ============================================================================
# Cleanup tasks
# ============================================================================

# Clean all build artifacts
clean:
    cargo clean

# Clean the downloaded WebIDL cache (forces re-fetch on next gen-wit-fetch)
clean-idl-cache:
    @echo "Removing IDL cache..."
    rm -rf scripts/idl-cache

# ============================================================================
# W3C WebIDL → WIT generation
# ============================================================================

# Fetch WebIDL specs from W3C WebRef (https://github.com/w3c/webref, curated branch)
# Downloads IDL files for: dom, fetch, html, websockets, streams, service-workers,
# file-api, indexed-db, geolocation, observers, web-animations, and more.
# Output: scripts/idl-cache/*.idl
gen-wit-fetch:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Fetching W3C WebIDL specs from webref..."
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    {{python}} scripts/fetch_w3c_idl.py

# Re-fetch all specs (ignore cache)
gen-wit-fetch-force:
    {{python}} scripts/fetch_w3c_idl.py --force

# Generate WIT interface files from cached WebIDL specs.
# Reads:  scripts/idl-cache/*.idl
# Writes: packages/browser-worlds/wit/generated/*.wit
# Run gen-wit-fetch first if the cache is empty.
gen-wit:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Generating WIT interfaces from W3C WebIDL..."
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    {{python}} scripts/webidl_to_wit.py

# Full pipeline: fetch WebIDL specs then generate WIT files.
gen-wit-all: gen-wit-fetch gen-wit
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "✅ WebIDL → WIT pipeline complete!"
    @echo "   Generated WIT: packages/browser-worlds/wit/generated/"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# ============================================================================
# Build tasks
# ============================================================================

# Build everything (Debug mode)
build-dev: init
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Building all (Debug mode)..."
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cargo build --all

# Build everything (Release mode)
build: init
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

# Run all checks (cargo check + run examples)
test:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Running comprehensive checks..."
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Step 1/9: Checking code compilation..."
    cargo check --workspace --all-targets
    @echo "✅ Check passed"
    @echo ""
    @echo "Step 2/9: Running compile-time demo..."
    cargo run --package tairitsu-example-wit-compile-time --bin compile-time-demo
    @echo "✅ Compile-time demo passed"
    @echo ""
    @echo "Step 3/9: Running runtime demo..."
    cargo run --package tairitsu-example-wit-runtime --bin runtime-demo
    @echo "✅ Runtime demo passed"
    @echo ""
    @echo "Step 4/9: Running dynamic advanced demo..."
    cargo run --package tairitsu-example-wit-dynamic-advanced --bin dynamic-advanced-demo
    @echo "✅ Dynamic advanced demo passed"
    @echo ""
    @echo "Step 5/9: Building simple WASM module..."
    cargo build --target wasm32-wasip2 --release --package tairitsu-example-wit-native-simple --lib
    @echo "✅ Simple WASM built"
    @echo ""
    @echo "Step 6/9: Running simple WASM host..."
    cargo run --package tairitsu-example-wit-native-simple --bin simple-wasm-host
    @echo "✅ Simple WASM host passed"
    @echo ""
    @echo "Step 7/9: Building macro WASM module..."
    cargo build --target wasm32-wasip2 --release --package tairitsu-example-wit-native-macro --lib
    @echo "✅ Macro WASM built"
    @echo ""
    @echo "Step 8/9: Running macro WASM host..."
    cargo run --package tairitsu-example-wit-native-macro --bin macro-wasm-host
    @echo "✅ Macro WASM host passed"
    @echo ""
    @echo "Step 9/9: Running unit tests..."
    cargo test --workspace --lib --features dynamic
    @echo "✅ Unit tests passed"
    @echo ""
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "✅ All checks passed successfully!"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# ============================================================================
# Code quality checks
# ============================================================================

# Run Clippy linter (requires clippy component)
clippy:
    @echo "Running Clippy..."
    cargo clippy --workspace --all-targets -- -D warnings

# Run formatting check
fmt-check:
    @echo "Checking code formatting..."
    cargo +nightly fmt --all -- --check --unstable-features

# Format all code
fmt:
    @echo "Formatting all code..."
    cargo +nightly fmt --all -- --unstable-features

# CI checks (format check + test)
ci: fmt-check test
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
# Web development
# ============================================================================

# Development mode for website
#   just dev                  - Blocking foreground with hot-reload
#   just dev --daemon         - Start/restart daemon (non-blocking)
#   just dev --daemon --debug - Start daemon + debug API server (port 3001)
#   just dev --daemon stop    - Stop daemon
dev *FLAGS="":
    cd examples/website && tairitsu --manifest-path Cargo.toml dev --port 3000 --watch {{FLAGS}}

# Dev server with debug/inspection API for agent automation
dev-debug *FLAGS="":
    cd examples/website && tairitsu --manifest-path Cargo.toml dev --port 3000 --watch --daemon --debug {{FLAGS}}

# Build web demo for production (using tairitsu-packager + CDN demo)
build-web: init
    @echo "Building website demo with tairitsu-packager..."
    @{{python}} scripts/install_packager.py --quick || (cargo build --release --package tairitsu-packager && {{python}} scripts/install_packager.py)
    tairitsu --manifest-path examples/website build --release
    @echo "Building CDN modular demo..."
    {{python}} scripts/build_cdn_demo.py --dist target/tairitsu-dist

# Serve web demo (production build)
serve-web: build-web
    @echo "Serving production build..."
    cd examples/website/dist && {{python}} -m http.server 3001

# ============================================================================
# WIT generation — W3C WebIDL → WIT interface pipeline
# ============================================================================

# Fetch WebIDL specs from w3c/webref + generate WIT (full pipeline)
# Requires internet access on the first run; subsequent runs use the cached files.
# Cached WebIDL: target/tairitsu-wit/webidl-cache/  (git-ignored)
# Generated WIT: packages/browser-worlds/wit/generated/  (committed to git)
wit-gen:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "WIT generation pipeline (WebIDL → WIT)"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    {{python}} scripts/gen_wit_from_webidl.py

# Step 1: Fetch W3C/WHATWG WebIDL spec files into target/tairitsu-wit/webidl-cache/
wit-fetch-idl:
    @echo "Fetching WebIDL specs from w3c/webref..."
    {{python}} scripts/fetch_webidl.py

# Step 2: Parse cached WebIDL and generate WIT files under packages/browser-worlds/wit/generated/
wit-gen-wit:
    @echo "Generating WIT from cached WebIDL..."
    {{python}} scripts/generate_browser_wit.py

# Re-download all WebIDL specs (force even if cached)
wit-fetch-force:
    @echo "Force re-fetching all WebIDL specs..."
    {{python}} scripts/fetch_webidl.py --force

# Show WIT generation coverage statistics
wit-stats:
    {{python}} scripts/generate_browser_wit.py --stats

# ============================================================================
# TypeScript Glue generation (WIT → TypeScript)
# ============================================================================

# Generate TypeScript glue code from WIT files
# Reads:  packages/browser-worlds/wit/generated/*.wit
# Writes: packages/browser-glue/src/generated/*-glue.ts
glue-gen:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "TypeScript Glue generation (WIT → TypeScript)"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    {{python}} scripts/generate_browser_glue.py

# Show TypeScript glue generation coverage statistics
glue-stats:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "TypeScript Glue Statistics"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    {{python}} scripts/generate_browser_glue.py --stats

# Dry-run: show what glue generation would do without writing
glue-dry-run:
    {{python}} scripts/generate_browser_glue.py --dry-run

# Full pipeline: WIT + TypeScript Glue generation
wit-full: wit-gen glue-gen
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "✅ Full WIT → TypeScript Glue pipeline complete!"
    @echo "   Generated WIT: packages/browser-worlds/wit/generated/"
    @echo "   Generated Glue: packages/browser-glue/src/generated/"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Show all W3C data sources used by the pipeline
wit-sources:
    {{python}} scripts/gen_wit_from_webidl.py --list-sources

# List all target WebIDL specs and their cache status
wit-list-specs:
    {{python}} scripts/fetch_webidl.py --list-specs

# Dry-run: show what the pipeline would do without downloading/writing
wit-dry-run:
    {{python}} scripts/gen_wit_from_webidl.py --dry-run

# ============================================================================
# Browser testing tasks
# ============================================================================

# Download and cache Chromium browser
browser-install:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Installing Chromium for browser testing..."
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cargo run --package tairitsu-browser-test -- browser install

# List cached browser versions
browser-list:
    @echo "Listing cached browser versions..."
    cargo run --package tairitsu-browser-test -- browser list

# Clear browser cache
browser-clear:
    @echo "Clearing browser cache..."
    cargo run --package tairitsu-browser-test -- browser clear

# Run browser-glue tests
test-browser:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Running browser-glue tests..."
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    cargo run --package tairitsu-browser-test -- test run --headless

# CI: Install browser + run tests
test-browser-ci: browser-install test-browser
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "✅ Browser tests completed!"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

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
# NPM publishing
# ============================================================================

# Generate and build all per-domain npm glue packages
npm-build-glue:
    @echo "Generating per-domain glue packages..."
    {{python}} scripts/build_npm_glue_packages.py

# Build Rust crates into optimized wasm component npm packages
npm-build-wasm crate="":
    @echo "Building WASM component packages..."
    {{python}} scripts/build_wasm_packages.py {{crate}}

# List compilable WASM crates
npm-list-wasm:
    {{python}} scripts/build_wasm_packages.py --list

# Build all npm packages (glue + runtime + wasm)
npm-build-all: npm-build-glue
    cd packages/npm/runtime && npm run build
    cd packages/npm/glue-core && npm run build
    {{python}} scripts/build_wasm_packages.py

# Publish all npm packages to @celestia scope (requires NPM_TOKEN env var)
publish: (publish-pkg "packages/browser-glue") (publish-pkg "packages/npm/runtime")
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "All npm packages published!"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Publish a single npm package (dry-run by default)
publish-pkg dir:
    @echo "Publishing {{dir}}..."
    npm publish --access public --dry-run {{dir}}

# Publish for real (not dry-run) — requires NPM_TOKEN
publish-live:
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "Publishing all npm packages (LIVE)..."
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @if [ -z "$NPM_TOKEN" ]; then echo "Error: NPM_TOKEN environment variable is not set."; exit 1; fi
    npm config set //registry.npmjs.org/:_authToken $NPM_TOKEN
    cd packages/npm/runtime && npm run build && npm publish --access public
    cd packages/npm/glue-core && npm run build && npm publish --access public
    cd packages/browser-glue && npm run build:production && npm publish --access public
    @for dir in packages/npm/glue-*/; do cd "$$dir" && npm publish --access public && cd -; done
    @for dir in packages/npm/*-wasm/; do cd "$$dir" && npm publish --access public && cd -; done
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "All npm packages published (LIVE)!"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Build all npm packages locally
npm-build:
    npm run build -w @celestia/tairitsu-browser-glue || (cd packages/browser-glue && npm run build)
    npm run build -w @celestia/tairitsu-runtime || (cd packages/npm/runtime && npm run build)

# Build CDN demo with esm.sh CDN URLs (for production deployment)
cdn-demo-prod:
    @echo "Building CDN demo (esm.sh mode)..."
    {{python}} scripts/build_cdn_demo.py --dist target/tairitsu-dist --cdn-mode esm-sh

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
    @echo "  - website:           New website demo (run with 'just dev')"
    @echo "  - wit-native-simple: trait-based composable WIT interfaces"
    @echo "  - wit-native-macro: macro-generated WIT interfaces"
    @echo ""
    @echo "Quick start:"
    @echo "  just dev            - Start web demo with hot reload"
    @echo "  just build-web      - Build web demo for production"
    @echo ""
    @echo "WIT generation (W3C WebIDL → WIT):"
    @echo "  just wit-gen        - Full pipeline: fetch + generate"
    @echo "  just wit-fetch-idl  - Only download WebIDL spec files"
    @echo "  just wit-gen-wit    - Only generate WIT from cache"
    @echo "  just wit-stats      - Show interface coverage statistics"
    @echo "  just wit-sources    - Show data source information"
    @echo ""
    @echo "Package structure:"
    @echo "  - packages/runtime:               Tairitsu core runtime"
    @echo "  - packages/macros:                Procedural macros"
    @echo "  - packages/browser-wit-resolver:  WIT package resolution + cache"
    @echo "  - packages/browser-worlds:        WIT world definitions (0.1.x hand-written,"
    @echo "                                    0.2.x generated from W3C WebIDL)"
    @echo "  - packages/browser-glue:          TypeScript/SWC browser API glue"
    @echo ""
    @echo "E2E testing:"
    @echo "  just e2e-capture   - Batch screenshot all demo pages"
    @echo "  just e2e-verify    - Capture + verify event bridge + report"

# ============================================================================
# E2E Testing (PLAN2: Playwright-based visual regression)
# ============================================================================

# Batch-screenshot all demo pages via Playwright
e2e-capture:
    pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/e2e-capture.ps1

# Full verification: screenshots + WASM bridge check + report
e2e-verify:
    pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/e2e-verify.ps1

# Install Playwright dependencies for web-test package
e2e-install:
    cd packages/web-test && npm install && npx playwright install chromium
