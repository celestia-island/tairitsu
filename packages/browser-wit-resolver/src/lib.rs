//! `tairitsu-browser-wit-resolver`
//!
//! Resolves versioned WIT world packages for the browser interface layer.
//! Packages are fetched from a configurable cloud registry and cached locally
//! under `target/tairitsu-wit/<namespace>/<name>/<version>/`.
//!
//! # Feature flags
//! - `fetch` — enables live HTTP download via `reqwest`. Without this flag the
//!   resolver operates in cache-only (offline) mode.

pub mod cache;
pub mod fetch;
pub mod resolver;

pub use resolver::{ResolveOptions, ResolvedPackage, Resolver};

/// Default registry base URL.
pub const DEFAULT_REGISTRY: &str = "https://wit.tairitsu.dev";

/// Default sub-directory inside `target/` used for the local WIT cache.
pub const CACHE_DIR_NAME: &str = "tairitsu-wit";
