//! Thin adapter around the shared debug implementation
//! (`tairitsu-debug-core`). The ~3000-line HTTP debug API + raw-CDP engine
//! used to be duplicated here (in `tairitsu-browser`) and had drifted; it now
//! lives in one place. This module only binds the crate-specific hooks:
//!
//! - logging goes through the packager `logfmt` console output
//!   (`log_ok!`/`log_info!`/`log_warn!`/`log_fail!`)
//! - Chromium resolution goes to `tairitsu-browser-fetch`
//!   (`$CHROME_PATH` → build-time baked path → system Chrome → runtime
//!   fetch; zero-config) when `debug-browser` is enabled
//!
//! Public API is unchanged: `start_debug_server(cfg, port)` returning
//! `crate::Result<()>`.

pub use tairitsu_debug_core::{DebugHost, DebugServerConfig};

/// Launch the standalone debug API (+ CDP browser engine when the
/// `debug-browser` feature is enabled). See [`DebugServerConfig`] for the
/// inputs and [`tairitsu_debug_core::run_debug_server`] for the mechanics.
pub async fn start_debug_server(cfg: DebugServerConfig, debug_port: u16) -> crate::Result<()> {
    tairitsu_debug_core::run_debug_server(cfg, debug_port, debug_host()).await?;
    Ok(())
}

fn debug_host() -> DebugHost {
    DebugHost {
        log_ok,
        log_info,
        log_warn,
        log_fail,
        resolve_executable,
    }
}

fn log_ok(args: std::fmt::Arguments<'_>) {
    crate::logfmt::ok(args);
}

fn log_info(args: std::fmt::Arguments<'_>) {
    crate::logfmt::info(args);
}

fn log_warn(args: std::fmt::Arguments<'_>) {
    crate::logfmt::warn(args);
}

fn log_fail(args: std::fmt::Arguments<'_>) {
    crate::logfmt::fail(args);
}

#[cfg(feature = "debug-browser")]
fn resolve_executable() -> Result<String, String> {
    tairitsu_browser_fetch::resolve()
        .map(|p| p.to_string_lossy().into_owned())
        .map_err(|e| e.to_string())
}

#[cfg(not(feature = "debug-browser"))]
fn resolve_executable() -> Result<String, String> {
    Err("debug-browser feature is not enabled".to_string())
}
