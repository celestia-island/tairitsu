# Tairitsu Packager - Icon Support Plan

## Overview

Add icon fetching and building capabilities to tairitsu-packager, allowing projects to declaratively configure which icons they need via `Cargo.toml` metadata.

## Motivation

Currently, Hikari uses a Python script (`scripts/icons/fetch_mdi_icons.py`) to:
1. Download MDI icons from GitHub
2. Generate icon enum (Rust code)
3. Save SVG files and metadata

This should be unified into tairitsu-packager for:
- Consistent build pipeline
- No Python dependency for icon handling
- Declarative icon selection via Cargo.toml

## Proposed Design

### 1. Cargo.toml Metadata

```toml
[package.metadata.tairitsu.icons]
# Icon source
source = "mdi"  # mdi, lucide, custom

# Selection strategies (choose one or combine)
icons = ["moon-waning-crescent", "sun", "account"]
tags = ["Nature", "Account"]  # Include all icons with these tags
styles = ["filled", "outline"]  # Which style variants to include

# Output configuration
output = "src/generated/icons.rs"
```

### 2. New Module: `packages/packager/src/icons/`

```
packages/packager/src/icons/
├── mod.rs           # Public API
├── fetcher.rs       # Download icons from sources
├── generator.rs     # Generate Rust code
└── metadata.rs      # Parse icon metadata
```

### 3. Public API

```rust
pub mod icons;

/// Icon source library
pub enum IconSource {
    MaterialDesign,  // MDI - 7,447 icons
    Lucide,          // Lucide - ~500 icons
}

/// Icon style variant
pub enum IconStyle {
    Filled,
    Outline,
}

/// Icon selection configuration
pub struct IconConfig {
    pub source: IconSource,
    pub names: Vec<String>,
    pub tags: Vec<String>,
    pub styles: Vec<IconStyle>,
    pub output: PathBuf,
}

/// Fetch and build icons
pub fn build_icons(config: &IconConfig) -> Result<IconBuildResult>;
```

### 4. CLI Support

```bash
# Fetch icons based on Cargo.toml metadata
tairitsu icons fetch

# Build icon module (usually called during build)
tairitsu icons build

# List available icons
tairitsu icons list --source mdi --tag Nature
```

### 5. Integration with `tairitsu dev`

The `tairitsu dev` command should:
1. Check for `[package.metadata.tairitsu.icons]` in Cargo.toml
2. If present, fetch/update icons if not cached
3. Build icon module before compiling WASM
4. Watch icon source directory for changes

## Implementation Tasks

### Phase 1: Core Infrastructure
- [ ] Create `packages/packager/src/icons/mod.rs`
- [ ] Implement `IconFetcher` for MDI (download from GitHub)
- [ ] Parse MDI metadata (meta.json from zip)
- [ ] Cache downloaded icons in target directory

### Phase 2: Code Generation
- [ ] Generate Rust icon enum
- [ ] Generate icon data module with SVG paths
- [ ] Support selection by name, tag, style

### Phase 3: Cargo.toml Integration
- [ ] Parse `[package.metadata.tairitsu.icons]` from Cargo.toml
- [ ] Integrate with `tairitsu dev` build pipeline
- [ ] Add `--icons` flag to force icon rebuild

### Phase 4: CLI Commands
- [ ] `tairitsu icons fetch` - Download icon library
- [ ] `tairitsu icons build` - Build icon module
- [ ] `tairitsu icons list` - List available icons

### Phase 5: Advanced Features
- [ ] Auto-discovery (scan codebase for icon usage)
- [ ] Custom icon sources (local SVG files)
- [ ] Icon optimization (SVGO integration)
- [ ] Multiple icon sources in one project

## File Locations

| Artifact | Location |
|----------|----------|
| Icon cache | `target/tairitsu/icons/mdi/` |
| Generated code | Configured in Cargo.toml (default: `src/generated/icons.rs`) |
| Metadata | `target/tairitsu/icons/mdi_metadata.json` |

## Migration from Hikari

1. Move `hikari/scripts/icons/fetch_mdi_icons.py` logic to Rust
2. Update `hikari/packages/builder/src/icons.rs` to use tairitsu-packager
3. Add `[package.metadata.tairitsu.icons]` to hikari Cargo.toml
4. Remove Python icon script

## Benefits

- **No Python dependency** for icon handling
- **Declarative configuration** via Cargo.toml
- **Faster builds** with caching
- **Better DX** with `tairitsu icons list` command
- **Unified tooling** - one CLI for everything
