//! Icon support module for tairitsu-packager.
//!
//! Provides multi-source icon resolution, caching, woff subset generation, and
//! CLI integration. Configured via `[package.metadata.hikari.icons]` in Cargo.toml.
//!
//! # CLI Usage
//!
//! ```bash
//! tairitsu icons sets          # List available icon sets
//! tairitsu icons fetch         # Fetch icon sets to cache
//! tairitsu icons list          # List icons from cache
//! tairitsu icons resolve       # Resolve + generate .dat files
//! ```

pub mod cache;
pub mod font;
pub mod hikari_resolver;
pub mod resolver;
pub mod sources;

pub use cache::{resolve_cache_root, CacheManifest, IconCache};
pub use font::{generate_woff_subset, is_hb_subset_available};
pub use resolver::{
    read_consumer_metadata, resolve, HikariIconsMetadata, ResolveResult, ResolvedSet, SetConfig,
    Subscript,
};
