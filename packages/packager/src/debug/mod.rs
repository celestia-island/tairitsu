//! Thin adapter onto shirabe's debug server.
//!
//! 2026-10-17 browser-backbone consolidation: the HTTP debug API, the raw
//! CDP engine, devtools polling and zero-config Chromium resolution moved
//! out of this repository — `tairitsu-debug-core` and
//! `tairitsu-browser-fetch` are archived, and shirabe owns that surface.
//! This module only keeps the packager-facing entry point so the CLI
//! (`tairitsu dev` / `tairitsu debug`) keeps working unchanged.
//!
//! Public API is unchanged: `start_debug_server(cfg, port)` returning
//! `crate::Result<()>`.

use crate::utils::error::TairitsuPackagerError;

pub use shirabe::{DebugServerConfig, start_debug_server as shirabe_start_debug_server};

/// Launch the standalone debug API (+ CDP browser engine).
///
/// Browser resolution happens inside shirabe (build-time baked path →
/// system Chrome → runtime fetch); see shirabe's `SHIRABE_BACKEND` /
/// `CHROME_PATH` env contract.
pub async fn start_debug_server(cfg: DebugServerConfig, debug_port: u16) -> crate::Result<()> {
    shirabe_start_debug_server(cfg, debug_port)
        .await
        .map_err(|e| TairitsuPackagerError::DebugError(e.to_string()))
}
