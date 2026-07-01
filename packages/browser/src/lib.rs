//! tairitsu-browser — Lightweight headless browser automation.
//!
//! Provides a CDP (Chrome DevTools Protocol) engine and HTTP debug API
//! for driving headless Chrome/Chromium. No WASM runtime, no SSR —
//! just browser automation.
//!
//! ## Quick Start
//!
//! ```no_run
//! use tairitsu_browser::{start_debug_server, DebugServerConfig};
//!
//! # async fn run() -> anyhow::Result<()> {
//! let cfg = DebugServerConfig {
//!     base_url: "about:blank".to_string(),
//!     proxy: None,
//! };
//! start_debug_server(cfg, 3001).await?;
//! # Ok(())
//! # }
//! ```

pub mod browser_fetch;
pub mod debug;

pub use debug::{DebugServerConfig, start_debug_server};
