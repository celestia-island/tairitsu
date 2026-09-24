//! Shared best-effort sweeper for stale download temp dirs.
//!
//! Included twice on purpose:
//! - by the lib itself (`lib.rs`, under `runtime-fetch`), and
//! - by `build.rs` via `#[path = "src/sweep.rs"]`,
//!
//! so the build-script downloader and the runtime downloader cannot drift
//! apart. Keep this module dependency-free (std only) — build scripts can
//! only see std plus their own explicit deps.

use std::path::Path;

/// Best-effort sweep of stale temp dirs (from crashed runs) under `parent`,
/// older than `max_age`. Live downloads have fresh mtimes and are left alone.
pub fn sweep_stale_temps(parent: &Path, max_age: std::time::Duration) {
    let Ok(entries) = std::fs::read_dir(parent) else {
        return;
    };
    let cutoff = std::time::SystemTime::now() - max_age;
    for entry in entries.flatten() {
        let fname = entry.file_name();
        let Some(name) = fname.to_str() else {
            continue;
        };
        if !name.starts_with('.') || !name.ends_with(".tmp") {
            continue;
        }
        if let Ok(meta) = entry.metadata() {
            if meta.is_dir() {
                if let Ok(mtime) = meta.modified() {
                    if mtime < cutoff {
                        let _ = std::fs::remove_dir_all(entry.path());
                    }
                }
            }
        }
    }
}
