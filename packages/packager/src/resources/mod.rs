//! Resource indexing module for tairitsu-packager.
//!
//! This module provides resource discovery, indexing, and cache busting capabilities.
//! It scans directories for SCSS and SVG files, computes content hashes, and generates
//! an index file for use during builds.
//!
//! # Example Usage
//!
//! ```rust,ignore
//! use tairitsu_packager::resources::{ResourceIndexer, ResourceIndex};
//!
//! let indexer = ResourceIndexer::new(".");
//! let index = indexer.scan()?;
//! index.save_to_target()?;
//! ```
//!
//! # Generated Index Format
//!
//! The index is saved to `target/tairitsu/resources/index.json`:
//!
//! ```json
//! {
//!   "scss": [
//!     {"source": "src/styles/main.scss", "hash": "abc123", "output": "main.abc123.css"}
//!   ],
//!   "svg": [
//!     {"source": "src/icons/sun.svg", "hash": "def456", "id": "sun"}
//!   ]
//! }
//! ```

mod indexer;
mod scss;
mod svg;

pub use indexer::{ResourceIndex, ResourceIndexer};
pub use scss::{ScssResource, ScssUtils};
pub use svg::{SvgResource, SvgUtils};

/// Directory name for resource index output
pub const RESOURCE_DIR: &str = "tairitsu/resources";

/// Index file name
pub const INDEX_FILE: &str = "index.json";

use serde::{Deserialize, Serialize};

/// A resource entry with content hash for cache busting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEntry {
    /// Source path relative to project root
    pub source: String,
    /// Content hash (first 8 characters of SHA-256)
    pub hash: String,
}

impl ResourceEntry {
    /// Create a new resource entry
    pub fn new(source: impl Into<String>, hash: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            hash: hash.into(),
        }
    }
}
