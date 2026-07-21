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

set shell := ["bash", "-c"]
set windows-shell := ["bash.exe", "-c"]
set unstable
set lists

# Shared celestia-devtools recipes — NOT in git. Stage with: just fetch.
# `import?` silently skips when absent, so this justfile parses pre-fetch.
import? "./.just/git-bash-interop.just"
import? "./.just/celestia-devtools.just"

# Stage shared celestia-devtools recipes into .just/ (gitignored).
# Source order: explicit URL arg → local pip bundle (offline) → GitHub raw.
# curl honors HTTP_PROXY/HTTPS_PROXY/ALL_PROXY env vars automatically.
[script('bash')]
fetch URL='':
    #!/usr/bin/env bash
    set -euo pipefail
    out=.just/celestia-devtools.just
    mkdir -p .just
    if [ -n "{{URL}}" ]; then
      echo "[fetch] {{URL}} -> $out"
      curl -fsSL "{{URL}}" -o "$out"
    elif command -v celestia-devtools >/dev/null 2>&1; then
      src=$(celestia-devtools include-path)
      echo "[fetch] local bundle ($src) -> $out"
      cp "$src" "$out"
    else
      echo "[fetch] github raw -> $out"
      curl -fsSL "https://raw.githubusercontent.com/celestia-island/celestia-devtools/dev/src/celestia_devtools/common.just" -o "$out"
    fi
    echo "[fetch] wrote $out"

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
    rustup component add rustfmt
    rustup component add clippy
    {{python}} scripts/download_wasi_adapters.py

# Build browser-glue runtime bundle (IIFE for HTML <script> tag)
build-glue-runtime:
    mkdir -p packages/browser-glue/dist
    npx esbuild packages/browser-glue/src/runtime/index.ts --bundle --outfile=packages/browser-glue/dist/runtime.js --format=iife --platform=browser

# Install tairitsu-packager CLI binary (tairitsu) + tairitsu-mcp
install-packager: (build-glue-runtime)
    cargo build --release --package tairitsu-packager --package tairitsu-mcp
    {{python}} scripts/install_packager.py

# Development environment setup (install tools and build)
setup: install-tools init
    cargo build --release --all

# ============================================================================
# JS / Node dependency initialization
# ============================================================================

# Install Node.js dependencies for packages/npm/celestia-tairitsu-web-glue (auto-detects pnpm/yarn/npm)
init:
    {{python}} scripts/init_browser_glue.py

# ============================================================================
# Cleanup tasks
# ============================================================================

# Clean all build artifacts
clean:
    cargo clean

# Clean the downloaded WebIDL cache (forces re-fetch on next wit-gen)
clean-idl-cache:
    @{{python}} scripts/clean_idl_cache.py

# ============================================================================
# Build tasks
# ============================================================================

# Build everything (Debug mode)
# Build everything. Release by default; `--dev` for debug, `--clean` to clean first.
build *FLAGS='':
    just init
    just _build ":" "cargo build --all" "cargo build --release --all" {{FLAGS}}

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

# Run macro demo (macro-generated WIT interfaces)
run-macro-demo:
    @echo "Running macro demo..."
    cargo run --package tairitsu-example-wit-native-macro --bin macro-demo

# Run dynamic advanced demo (RON + complex types)
run-dynamic-advanced:
    @echo "Running dynamic advanced example..."
    cargo run --package tairitsu-example-wit-dynamic-advanced --bin dynamic-advanced-demo

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
    cargo check --workspace --all-targets --exclude tairitsu-e2e
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
# NOTE: no `--all` — in a virtual workspace `cargo fmt` formats every member,
# but `--all` would additionally walk path dependencies (e.g. the local
# `../kou` dev override) and rewrite / fail on sibling repos outside this tree.
fmt-check:
    @echo "Checking code formatting..."
    cargo fmt -- --check

# Format all code
# (no `--all`: see fmt-check — avoids traversing path deps like ../kou)
fmt:
    just fmt-toml
    cargo fmt
    python3 scripts/enforce_use_groups.py

# CI checks (format check + clippy + test)
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
    @cargo watch -x check 2>/dev/null || echo "[HINT] Install cargo-watch: cargo install cargo-watch"

# ============================================================================
# Web development
# ============================================================================

# Development mode for website
#   just dev                  - Blocking foreground with hot-reload
#   just dev --daemon         - Start/restart daemon (non-blocking)
#   just dev --daemon --debug - Start daemon + debug API server (port 3001)
#   just dev --daemon stop    - Stop daemon
dev *FLAGS="":
    @tairitsu --help > /dev/null 2>&1 || (echo "  Building tairitsu CLI..." && cargo build --release --package tairitsu-packager > /dev/null 2>&1)
    cd examples/website && tairitsu --manifest-path Cargo.toml dev --port 3000 --watch {{FLAGS}}

# Dev server with debug/inspection API for agent automation
dev-debug *FLAGS="":
    cd examples/website && tairitsu --manifest-path Cargo.toml dev --port 3000 --watch --daemon --debug {{FLAGS}}

# Build web demo for production (using tairitsu-packager + CDN demo)
build-web: init
    @echo "Building website demo with tairitsu-packager..."
    @tairitsu --help > /dev/null 2>&1 || (cargo build --release --package tairitsu-packager && {{python}} scripts/install_packager.py)
    tairitsu --manifest-path examples/website build --release
    @echo "Building CDN modular demo..."
    {{python}} scripts/build_cdn_demo.py --dist target/tairitsu-dist

# Serve web demo (production build)
serve-web: build-web
    @echo "Serving production build..."
    @cd examples/website/dist && {{python}} -m http.server 3001 2>/dev/null || echo "[HINT] Python http.server not available; try: python -m http.server 3001"

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
# Writes: packages/npm/celestia-tairitsu-web-glue/src/generated/*-glue.ts
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
    @echo "   Generated Glue: packages/npm/celestia-tairitsu-web-glue/src/generated/"
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
    @echo "Generating unified browser-glue package..."
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
    cd packages/npm/celestia-tairitsu-web-glue && npm run build
    {{python}} scripts/build_wasm_packages.py

# Publish all npm packages to @celestia scope (requires NPM_TOKEN env var)
publish: (publish-pkg "packages/npm/celestia-tairitsu-web-glue")
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    @echo "All npm packages published!"
    @echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Publish a single npm package (dry-run by default)
publish-pkg dir:
    @echo "Publishing {{dir}}..."
    npm publish --access public --dry-run {{dir}}

# Publish for real (not dry-run) — requires NPM_TOKEN
publish-live:
    @{{python}} scripts/publish_live.py

# Build all npm packages locally
npm-build:
    npm run build -w @celestia/tairitsu-browser-glue || (cd packages/npm/celestia-tairitsu-web-glue && npm run build)
    npm run build -w @celestia/tairitsu-runtime || (cd packages/npm/celestia-tairitsu-runtime && npm run build)

# Build CDN demo with esm.sh CDN URLs (for production deployment)
cdn-demo-prod:
    @echo "Building CDN demo (esm.sh mode)..."
    {{python}} scripts/build_cdn_demo.py --dist target/tairitsu-dist --cdn-mode esm-sh

# ============================================================================
# WIT sync (packages/web embedded copy)
# ============================================================================

# Sync composed WIT files from browser-worlds into packages/web
sync-wit:
    @{{python}} scripts/sync_wit.py

# Check that embedded WIT files are in sync with browser-worlds
sync-wit-check:
    @if [ ! -d packages/web/wit/composed ]; then echo "packages/web/wit/composed does not exist, run: just sync-wit" && exit 1; fi
    @diff -r packages/browser-worlds/wit/composed packages/web/wit/composed \
      || (echo "WIT files out of sync! Run: just sync-wit" && exit 1)
    @echo "WIT files are in sync"

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
    @echo "  - packages/npm/celestia-tairitsu-web-glue:          TypeScript/SWC browser API glue"
    @echo ""
    @echo "Visual regression:"
    @echo "  just visual-capture - Capture screenshots via debug API"
    @echo "  just visual-diff    - Compare screenshots against baseline"
    @echo "  just visual-update  - Update baseline from actual screenshots"

# ============================================================================
# Visual Regression Testing (Phase 3: pixel comparison + HTML report)
# ============================================================================

# Capture screenshots via debug API server (requires running dev --debug)
visual-capture:
    @{{python}} scripts/visual_capture.py

# Run visual diff comparison against baseline
visual-diff tolerance="0.01":
    cargo run --package tairitsu-packager --features visual-diff -- \
        visual-diff \
        --tolerance {{tolerance}}

# Update baseline images from actual screenshots
visual-update:
    cargo run --package tairitsu-packager --features visual-diff -- \
        visual-diff \
        --update-baseline

# Full visual regression pipeline: capture + diff + report
visual-regression: visual-capture
    just visual-diff
