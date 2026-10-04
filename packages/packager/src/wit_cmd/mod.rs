//! `tairitsu wit` subcommand implementation.
//!
//! Provides commands for managing the `target/tairitsu-wit` cache:
//! - `fetch`  — download WIT packages from the registry
//! - `verify` — check cache integrity
//! - `list`   — list cached packages

use anyhow::Result;
use std::path::{Path, PathBuf};

use tairitsu_browser_wit_resolver::{
    cache::Cache,
    resolver::{PackageSpec, ResolveOptions, Resolver},
    CACHE_DIR_NAME,
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
/// `namespace:name@version` (e.g. `tairitsu-browser:dom@0.2.0`).
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

/// Verify that every package's registry entry (namespace/name/version) matches
/// the `package namespace:name@version;` declaration actually written inside
/// its embedded WIT files, and that all package identities are unique.
///
/// Generic over the package slice so tests can feed a synthetic drifted corpus.
pub fn verify_registry_consistency(
    packages: &[tairitsu_browser_worlds::EmbeddedPackage],
) -> Result<()> {
    let mut problems = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for pkg in packages {
        if !seen.insert(pkg.id) {
            problems.push(format!("duplicate registry entry: {}", pkg.id));
            continue;
        }
        let expected = format!("package {}:{}@{};", pkg.namespace, pkg.name, pkg.version);
        for (filename, bytes) in pkg.files {
            let content = String::from_utf8_lossy(bytes);
            let decls: Vec<String> = content
                .lines()
                .filter(|l| l.trim_start().starts_with("package "))
                .map(|l| l.trim().to_owned())
                .collect();
            match decls.as_slice() {
                [decl] if decl == &expected => {}
                [decl] => problems.push(format!(
                    "{}: {filename} declares `{decl}` but the registry entry is `{expected}`",
                    pkg.id
                )),
                found => problems.push(format!(
                    "{}: {filename} must declare exactly one `package ...;` line, found {}: {found:?}",
                    pkg.id,
                    found.len()
                )),
            }
        }
    }

    if problems.is_empty() {
        Ok(())
    } else {
        for p in &problems {
            error!("WIT registry inconsistency: {p}");
        }
        anyhow::bail!(
            "{} WIT registry inconsistency(ies):\n{}",
            problems.len(),
            problems.join("\n")
        );
    }
}

/// Parse every embedded WIT file with the real WIT parser (syntax check).
/// Files are pushed into a single `Resolve` so duplicate package identities
/// (two files declaring the same `namespace:name@version`) are caught too.
fn parse_embedded_corpus(packages: &[tairitsu_browser_worlds::EmbeddedPackage]) -> Result<()> {
    use wit_parser::Resolve;

    let mut resolve = Resolve::default();
    let mut failed = Vec::new();
    for pkg in packages {
        for (filename, bytes) in pkg.files {
            let content = String::from_utf8_lossy(bytes);
            if let Err(e) = resolve.push_str(filename, &content) {
                failed.push(format!("{} ({filename}): {e}", pkg.id));
            }
        }
    }
    if failed.is_empty() {
        Ok(())
    } else {
        for f in &failed {
            error!("WIT syntax error: {f}");
        }
        anyhow::bail!("{} embedded WIT file(s) failed to parse", failed.len())
    }
}

/// Run the `check wit` doctor on the embedded WIT corpus:
///
/// 1. registry ↔ file version-token consistency (no more doctor-passes-on-drift);
/// 2. every embedded file parses with the real WIT parser;
/// 3. every embedded package resolves through cache/embedded fallback.
///
/// With `offline` set, the resolution probe may not touch the network.
/// Fails with a non-zero exit on any inconsistency.
pub fn cmd_check(workspace_root: &Path, offline: bool) -> Result<()> {
    let packages = tairitsu_browser_worlds::EMBEDDED_PACKAGES;
    let file_count: usize = packages.iter().map(|p| p.files.len()).sum();

    verify_registry_consistency(packages)?;
    crate::log_ok!(
        "Registry ↔ package declarations consistent ({} packages, {file_count} files)",
        packages.len()
    );

    parse_embedded_corpus(packages)?;
    crate::log_ok!("All {file_count} embedded WIT files parse cleanly");

    let target_dir = resolve_target_dir(workspace_root);
    let mut opts = ResolveOptions::new(&target_dir);
    opts.offline = offline;
    let resolver = Resolver::new(opts);
    let mut failures = 0usize;
    for pkg in packages {
        let spec = PackageSpec {
            namespace: pkg.namespace.to_owned(),
            name: pkg.name.to_owned(),
            version: pkg.version.to_owned(),
        };
        if let Err(e) = resolver.resolve(&spec) {
            error!("✗ {} — resolution failed: {:#}", pkg.id, e);
            failures += 1;
        }
    }
    if failures > 0 {
        anyhow::bail!("{failures} WIT package(s) failed to resolve");
    }
    crate::log_ok!(
        "All {} WIT packages resolve ({}).",
        packages.len(),
        if offline { "offline" } else { "online" }
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tairitsu_browser_worlds::EmbeddedPackage;

    fn synthetic(
        id: &'static str,
        name: &'static str,
        version: &'static str,
        files: &'static [(&'static str, &'static [u8])],
    ) -> EmbeddedPackage {
        EmbeddedPackage {
            id,
            namespace: "tairitsu-browser",
            name,
            version,
            files,
        }
    }

    #[test]
    fn consistency_detects_version_drift() {
        // The exact drift this check exists for: registry claims @0.1.0 while
        // the embedded file declares @0.2.0.
        let pkgs = [synthetic(
            "tairitsu-browser:websocket@0.1.0",
            "websocket",
            "0.1.0",
            &[("test.wit", b"package tairitsu-browser:websocket@0.2.0;\n")],
        )];
        let err = verify_registry_consistency(&pkgs).expect_err("drift must be an error");
        let msg = format!("{err:#}");
        assert!(
            msg.contains("websocket@0.1.0"),
            "names registry side: {msg}"
        );
        assert!(msg.contains("websocket@0.2.0"), "names file side: {msg}");
    }

    #[test]
    fn consistency_detects_missing_package_declaration() {
        let pkgs = [synthetic(
            "tairitsu-browser:broken@0.2.0",
            "broken",
            "0.2.0",
            &[("test.wit", b"interface not-a-package-decl {}")],
        )];
        assert!(verify_registry_consistency(&pkgs).is_err());
    }

    #[test]
    fn consistency_accepts_aligned_synthetic_corpus() {
        let pkgs = [synthetic(
            "tairitsu-browser:dom@0.2.0",
            "dom",
            "0.2.0",
            &[("test.wit", b"package tairitsu-browser:dom@0.2.0;\n")],
        )];
        assert!(verify_registry_consistency(&pkgs).is_ok());
    }

    #[test]
    fn real_embedded_registry_is_consistent() {
        verify_registry_consistency(tairitsu_browser_worlds::EMBEDDED_PACKAGES)
            .expect("embedded registry must match embedded files");
    }
}
