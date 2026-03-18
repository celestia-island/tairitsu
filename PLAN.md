# Tairitsu Packager - Icon Support

## Status: ✅ Implemented

Icon support has been fully implemented in tairitsu-packager.

## Module Structure

```
packages/packager/src/icons/
├── mod.rs           # Public API (IconSource, IconStyle, IconConfig)
├── fetcher.rs       # Async MDI downloader with caching
├── generator.rs     # Rust code generation (Icon enum, SVG paths)
└── metadata.rs      # Cargo.toml config and MDI metadata parsing
```

## CLI Commands

```bash
tairitsu icons fetch --source mdi [--force]  # Download icons
tairitsu icons build [--output path]          # Generate Rust code
tairitsu icons list --source mdi [--tag ...]  # List available icons
```

## Cargo.toml Configuration

```toml
[package.metadata.tairitsu.icons]
source = "mdi"
icons = ["moon-waning-crescent", "sun", "account"]
tags = ["Nature", "Account"]
styles = ["filled", "outline"]
output = "src/generated/icons.rs"
```

## Generated API

```rust
pub enum Icon {
    moon_waning_crescent,
    sun,
    account,
}

impl Icon {
    pub fn name(&self) -> &'static str;
    pub fn svg_path(&self) -> &'static str;
    pub fn tags(&self) -> &'static [&'static str];
    pub fn all() -> &'static [Icon];
}

pub const ICON_COUNT: usize;
pub const ICON_NAMES: &[&str];
```

## File Locations

| Artifact     | Location                                     |
| ------------ | -------------------------------------------- |
| Icon cache   | `target/tairitsu/icons/mdi/`                 |
| Generated    | Configured via Cargo.toml (default applies)  |
| Metadata     | `target/tairitsu/icons/mdi/metadata.json`    |

## Future Enhancements

- Auto-discovery (scan codebase for icon usage)
- Lucide icon source support
- Custom icon sources (local SVG files)
- Icon optimization (SVGO integration)
