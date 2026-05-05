//! `tairitsu wit` subcommand implementation.
//!
//! Provides commands for managing the `target/tairitsu-wit` cache:
//! - `fetch`  — download WIT packages from the registry
//! - `verify` — check cache integrity
//! - `list`   — list cached packages

use anyhow::Result;
use std::path::{Path, PathBuf};

use tairitsu_browser_wit_resolver::{
    CACHE_DIR_NAME,
    cache::Cache,
    resolver::{PackageSpec, ResolveOptions, Resolver},
};
use tracing::error;

/// Determine the workspace target directory.
fn resolve_target_dir(workspace_root: &Path) -> PathBuf {
    // Prefer $CARGO_TARGET_DIR, then fall back to <workspace_root>/target.
    if let Ok(dir) = std::env::var("CARGO_TARGET_DIR") {
        return PathBuf::from(dir);
    }
    workspace_root.join("target")
}

/// Fetch one or more WIT packages and store them in the local cache.
///
/// `specs` is a list of package identifiers in the form
/// `namespace:name@version` (e.g. `tairitsu-browser:dom@0.1.0`).
/// Pass an empty slice to fetch all known browser-world packages.
pub fn cmd_fetch(workspace_root: &Path, specs: &[String], offline: bool) -> Result<()> {
    let target_dir = resolve_target_dir(workspace_root);
    let mut opts = ResolveOptions::new(&target_dir);
    if offline {
        opts.offline = true;
    }

    let specs_to_fetch: Vec<PackageSpec> = if specs.is_empty() {
        // Default: fetch all embedded browser-world packages.
        tairitsu_browser_worlds::EMBEDDED_PACKAGES
            .iter()
            .map(|p| PackageSpec {
                namespace: p.namespace.to_owned(),
                name: p.name.to_owned(),
                version: p.version.to_owned(),
            })
            .collect()
    } else {
        specs
            .iter()
            .map(|s| PackageSpec::parse(s))
            .collect::<Result<Vec<_>>>()?
    };

    let resolver = Resolver::new(opts);
    let mut any_error = false;

    for spec in &specs_to_fetch {
        match resolver.resolve(spec) {
            Ok(pkg) => {
                if pkg.from_cache {
                    crate::log_info!("{} (already cached at {})", pkg.id, pkg.wit_dir.display());
                } else {
                    crate::log_info!("{} → {}", pkg.id, pkg.wit_dir.display());
                }
            }
            Err(e) => {
                error!("Failed to resolve {}: {:#}", spec.id(), e);
                any_error = true;
            }
        }
    }

    if any_error {
        anyhow::bail!("One or more WIT packages could not be resolved");
    }
    Ok(())
}

/// Verify cache integrity for all or selected packages.
pub fn cmd_verify(workspace_root: &Path, specs: &[String]) -> Result<()> {
    let target_dir = resolve_target_dir(workspace_root);
    let cache = Cache::new(target_dir.join(CACHE_DIR_NAME));

    let specs_to_check: Vec<PackageSpec> = if specs.is_empty() {
        // Verify everything in the cache.
        let ids = cache.list()?;
        ids.iter()
            .map(|id| PackageSpec::parse(id))
            .collect::<Result<Vec<_>>>()?
    } else {
        specs
            .iter()
            .map(|s| PackageSpec::parse(s))
            .collect::<Result<Vec<_>>>()?
    };

    if specs_to_check.is_empty() {
        crate::log_info!("No packages in cache — nothing to verify.");
        return Ok(());
    }

    let mut ok_count = 0usize;
    let mut fail_count = 0usize;

    for spec in &specs_to_check {
        match cache.lookup(spec) {
            Ok(Some(_)) => {
                crate::log_info!("✓ {}", spec.id());
                ok_count += 1;
            }
            Ok(None) => {
                error!("✗ {} — not in cache or integrity check failed", spec.id());
                fail_count += 1;
            }
            Err(e) => {
                error!("✗ {} — error: {:#}", spec.id(), e);
                fail_count += 1;
            }
        }
    }

    crate::log_info!("{ok_count} ok, {fail_count} failed");
    if fail_count > 0 {
        anyhow::bail!("{fail_count} package(s) failed verification");
    }
    Ok(())
}

/// List all packages currently in the local cache.
pub fn cmd_list(workspace_root: &Path) -> Result<()> {
    let target_dir = resolve_target_dir(workspace_root);
    let cache = Cache::new(target_dir.join(CACHE_DIR_NAME));
    let ids = cache.list()?;

    if ids.is_empty() {
        crate::log_info!(
            "No WIT packages cached in {}",
            target_dir.join(CACHE_DIR_NAME).display()
        );
    } else {
        crate::log_ok!("Cached WIT packages ({}):", ids.len());
        for id in &ids {
            crate::log_info!("  {id}");
        }
    }
    Ok(())
}
