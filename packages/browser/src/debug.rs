//! Thin adapter around the shared debug implementation
//! (`tairitsu-debug-core`). The ~3000-line HTTP debug API + raw-CDP engine
//! used to be duplicated here and in `tairitsu-packager`; it now lives in one
//! place. This module only binds the crate-specific hooks:
//!
//! - logging goes to `tracing` (`info`/`warn`/`error`)
//! - Chromium resolution goes to the crate-local simplified
//!   [`crate::browser_fetch`] resolver (no runtime auto-fetch)
//!
//! Public API is unchanged: `start_debug_server(cfg, port)` returning
//! `anyhow::Result<()>`.

pub use tairitsu_debug_core::{DebugHost, DebugServerConfig};

/// Launch the standalone debug API (+ CDP browser engine when the
/// `debug-browser` feature is enabled). See [`DebugServerConfig`] for the
/// inputs and [`tairitsu_debug_core::run_debug_server`] for the mechanics.
pub async fn start_debug_server(cfg: DebugServerConfig, debug_port: u16) -> anyhow::Result<()> {
    tairitsu_debug_core::run_debug_server(cfg, debug_port, debug_host()).await
}

fn debug_host() -> DebugHost {
    DebugHost {
        log_ok,
        log_info,
        log_warn,
        log_fail,
        resolve_executable: crate::browser_fetch::resolve_executable,
    }
}

fn log_ok(args: std::fmt::Arguments<'_>) {
    tracing::info!("{args}");
}

fn log_info(args: std::fmt::Arguments<'_>) {
    tracing::info!("{args}");
}

fn log_warn(args: std::fmt::Arguments<'_>) {
    tracing::warn!("{args}");
}

fn log_fail(args: std::fmt::Arguments<'_>) {
    tracing::error!("{args}");
}
