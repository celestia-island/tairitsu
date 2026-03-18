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

### 2. Module Structure: `packages/packager/src/icons/`

```
packages/packager/src/icons/
├── mod.rs           # Public API and core types
├── fetcher.rs       # Download icons from sources (MDI, async)
├── generator.rs     # Generate Rust code (enum, SVG paths)
└── metadata.rs      # Parse icon metadata and Cargo.toml config
```

### 3. CLI Support

```bash
# Fetch icons from configured source
tairitsu icons fetch --source mdi [--force]

# Build icon module
tairitsu icons build [--output path] [--icons icon1,icon2] [--tags tag1,tag2]

# List available icons
tairitsu icons list --source mdi [--tag Nature] [--search query]
```

### 4. Integration with `tairitsu dev`

The `tairitsu dev` command can check for `[package.metadata.tairitsu.icons]` and build icons as needed.

## Implementation Tasks

### Phase 1: Core Infrastructure ✅
- [x] Create `packages/packager/src/icons/mod.rs`
- [x] Implement `IconFetcher` for MDI (download from GitHub/npm)
- [x] Parse MDI metadata (meta.json)
- [x] Cache downloaded icons in target directory

### Phase 2: Code Generation ✅
- [x] Generate Rust icon enum
- [x] Generate icon data module with SVG paths
- [x] Support selection by name, tag

### Phase 3: Cargo.toml Integration ✅
- [x] Parse `[package.metadata.tairitsu.icons]` from Cargo.toml
- [x] `IconsConfig` struct with serde support

### Phase 4: CLI Commands ✅
- [x] `tairitsu icons fetch` - Download icon library
- [x] `tairitsu icons build` - Build icon module
- [x] `tairitsu icons list` - List available icons

### Phase 5: Advanced Features (Future)
- [ ] Auto-discovery (scan codebase for icon usage)
- [ ] Custom icon sources (local SVG files)
- [ ] Icon optimization (SVGO integration)
- [ ] Multiple icon sources in one project

## File Locations

| Artifact | Location |
|----------|----------|
| Icon cache | `target/tairitsu/icons/mdi/` |
| Generated code | Configured in Cargo.toml (default: `src/generated/icons.rs`) |
| Metadata | `target/tairitsu/icons/mdi/metadata.json` |

## API

```rust
pub mod icons;

pub enum IconSource {
    Mdi,      // Material Design Icons
    Lucide,   // Lucide icons
    Custom,   // Local SVG files
}

pub enum IconStyle {
    Filled,
    Outline,
}

pub struct IconConfig {
    pub source: IconSource,
    pub names: Vec<String>,
    pub tags: Vec<String>,
    pub styles: Vec<IconStyle>,
    pub output: PathBuf,
}

pub fn build_icons(config: &IconConfig, target_dir: &Path) -> Result<IconBuildResult>;
pub fn fetch_icons(source: &IconSource, cache_dir: &Path) -> Result<IconMetadata>;
```
