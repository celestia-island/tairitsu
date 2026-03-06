//! WIT package version resolver.
//!
//! Combines cache look-up, optional network fetch, and fallback to embedded
//! WIT content (provided by `tairitsu-browser-worlds`).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::{
    cache::Cache,
    fetch::FetchClient,
    DEFAULT_REGISTRY,
};

/// A fully-resolved WIT package ready for use.
#[derive(Debug, Clone)]
pub struct ResolvedPackage {
    /// Human-readable package identifier, e.g. `tairitsu-browser:dom@0.1.0`.
    pub id: String,
    /// Directory on disk containing the WIT files.
    pub wit_dir: PathBuf,
    /// Whether this was served from the local cache.
    pub from_cache: bool,
}

/// A package specifier as written in configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageSpec {
    /// Namespace of the WIT package (e.g. `tairitsu-browser`).
    pub namespace: String,
    /// Name of the WIT package (e.g. `dom`).
    pub name: String,
    /// Semver-compatible version string (e.g. `0.1.0`).
    pub version: String,
}

impl PackageSpec {
    /// Parse a spec string of the form `namespace:name@version`.
    pub fn parse(s: &str) -> Result<Self> {
        let (namespace_name, version) = s
            .split_once('@')
            .with_context(|| format!("Missing '@version' in package spec '{s}'"))?;
        let (namespace, name) = namespace_name
            .split_once(':')
            .with_context(|| format!("Missing ':name' in package spec '{s}'"))?;
        Ok(Self {
            namespace: namespace.to_owned(),
            name: name.to_owned(),
            version: version.to_owned(),
        })
    }

    /// Canonical identifier string.
    pub fn id(&self) -> String {
        format!("{}:{}@{}", self.namespace, self.name, self.version)
    }

    /// Relative path within the cache directory.
    pub fn cache_rel_path(&self) -> PathBuf {
        PathBuf::from(&self.namespace)
            .join(&self.name)
            .join(&self.version)
    }
}

/// Options controlling resolver behaviour.
#[derive(Debug, Clone)]
pub struct ResolveOptions {
    /// Root of the `target/` directory (typically `$CARGO_TARGET_DIR` or
    /// `<workspace_root>/target`).
    pub target_dir: PathBuf,
    /// Registry base URL (defaults to [`DEFAULT_REGISTRY`]).
    pub registry_url: String,
    /// When `true`, skip any network requests and fail if the package is not
    /// already in the local cache.
    pub offline: bool,
}

impl ResolveOptions {
    /// Construct options with sensible defaults derived from the current
    /// working directory.
    pub fn new(target_dir: impl AsRef<Path>) -> Self {
        let registry_url = std::env::var("TAIRITSU_WIT_REGISTRY")
            .unwrap_or_else(|_| DEFAULT_REGISTRY.to_owned());
        let offline = std::env::var("TAIRITSU_WIT_OFFLINE")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        Self {
            target_dir: target_dir.as_ref().to_owned(),
            registry_url,
            offline,
        }
    }
}

/// Main resolver entry point.
pub struct Resolver {
    cache: Cache,
    fetch_client: FetchClient,
    opts: ResolveOptions,
}

impl Resolver {
    /// Create a new resolver.
    pub fn new(opts: ResolveOptions) -> Self {
        let cache_root = opts.target_dir.join(crate::CACHE_DIR_NAME);
        Self {
            cache: Cache::new(cache_root),
            fetch_client: FetchClient::new(opts.registry_url.clone()),
            opts,
        }
    }

    /// Resolve a package, returning the local directory containing WIT files.
    ///
    /// Resolution order:
    /// 1. Check local cache (`target/tairitsu-wit/<ns>/<name>/<ver>/`).
    /// 2. If not cached (and not offline), fetch from the registry.
    /// 3. Store fetched content in the cache.
    pub fn resolve(&self, spec: &PackageSpec) -> Result<ResolvedPackage> {
        let id = spec.id();

        // 1. Cache look-up.
        if let Some(entry) = self.cache.lookup(spec)? {
            debug!("Cache hit for {id}");
            return Ok(ResolvedPackage {
                id,
                wit_dir: entry.wit_dir,
                from_cache: true,
            });
        }

        // 2. Offline mode: fail fast.
        if self.opts.offline {
            anyhow::bail!(
                "WIT package '{id}' not found in cache and offline mode is enabled.\n\
                 Run `tairitsu wit fetch {id}` while online to populate the cache."
            );
        }

        // 3. Fetch from registry.
        info!("Fetching WIT package {id} from {}", self.opts.registry_url);
        let files = self
            .fetch_client
            .fetch(spec)
            .with_context(|| format!("Failed to fetch WIT package '{id}'"))?;

        // 4. Store in cache.
        let entry = self.cache.store(spec, files)?;
        info!("Cached WIT package {id} at {}", entry.wit_dir.display());

        Ok(ResolvedPackage {
            id,
            wit_dir: entry.wit_dir,
            from_cache: false,
        })
    }

    /// Resolve multiple packages and return all results.
    pub fn resolve_all(&self, specs: &[PackageSpec]) -> Result<Vec<ResolvedPackage>> {
        specs.iter().map(|s| self.resolve(s)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_spec_parse_valid() {
        let spec = PackageSpec::parse("tairitsu-browser:dom@0.1.0").unwrap();
        assert_eq!(spec.namespace, "tairitsu-browser");
        assert_eq!(spec.name, "dom");
        assert_eq!(spec.version, "0.1.0");
        assert_eq!(spec.id(), "tairitsu-browser:dom@0.1.0");
    }

    #[test]
    fn package_spec_parse_missing_at() {
        assert!(PackageSpec::parse("tairitsu-browser:dom").is_err());
    }

    #[test]
    fn package_spec_parse_missing_colon() {
        assert!(PackageSpec::parse("tairitsu-browser@0.1.0").is_err());
    }

    #[test]
    fn package_spec_cache_rel_path() {
        let spec = PackageSpec::parse("tairitsu-browser:dom@0.1.0").unwrap();
        assert_eq!(
            spec.cache_rel_path(),
            PathBuf::from("tairitsu-browser/dom/0.1.0")
        );
    }
}
