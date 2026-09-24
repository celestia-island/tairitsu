//! tairitsu-debug-core — single shared implementation of the Tairitsu HTTP
//! debug API and the raw-CDP headless-Chromium engine.
//!
//! Previously this ~3000-line implementation was duplicated verbatim in
//! `tairitsu-browser` (`src/debug.rs`) and `tairitsu-packager`
//! (`src/debug/mod.rs`) and had started to drift. Both crates are now thin
//! adapters that bind their crate-specific integration points through
//! [`DebugHost`]:
//!
//! - logging (browser: `tracing` / packager: `logfmt` console output)
//! - Chromium executable resolution (browser: local simplified resolver /
//!   packager: `tairitsu-browser-fetch` incl. runtime fetch)
//! - error surface (browser: `anyhow::Result` / packager: `crate::Result`)
//!
//! The `debug-browser` feature gates the CDP engine; consumers forward their
//! own feature of the same name to this one.

mod debug;

pub use debug::{run_debug_server, DebugHost, DebugServerConfig};

/// Version of tairitsu-debug-core (tracks the workspace release). Surfaced by
/// the `/health` and `/info` endpoints.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
