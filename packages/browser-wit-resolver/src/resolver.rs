//! WIT package version resolver.
//!
//! Combines cache look-up, optional network fetch, and fallback to embedded
//! WIT content (provided by `tairitsu-browser-worlds`).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::cache::Cache;
use crate::fetch::FetchClient;
use crate::DEFAULT_REGISTRY;

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
        let registry_url =
            std::env::var("TAIRITSU_WIT_REGISTRY").unwrap_or_else(|_| DEFAULT_REGISTRY.to_owned());
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
    /// 2. If not cached, check embedded packages (if `embedded` feature is enabled).
    /// 3. If not embedded (and not offline), fetch from the registry.
    /// 4. Store fetched/embedded content in the cache.
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

        // 2. Embedded fallback (if available).
        #[cfg(feature = "embedded")]
        {
            if let Some(embedded) = tairitsu_browser_worlds::find_embedded(&id) {
                debug!("Using embedded WIT package {id}");
                let files: std::collections::HashMap<String, Vec<u8>> = embedded
                    .files
                    .iter()
                    .map(|(name, bytes)| ((*name).to_owned(), bytes.to_vec()))
                    .collect();
                let entry = self.cache.store(spec, files)?;
                info!(
                    "Embedded WIT package {id} cached at {}",
                    entry.wit_dir.display()
                );
                return Ok(ResolvedPackage {
                    id,
                    wit_dir: entry.wit_dir,
                    from_cache: false,
                });
            }
        }

        // 3. Offline mode: fail fast (embedded fallback already attempted).
        if self.opts.offline {
            anyhow::bail!(
                "WIT package '{id}' not found in cache{} and offline mode is enabled.\n\
                 Run `tairitsu wit fetch {id}` while online to populate the cache.",
                if cfg!(feature = "embedded") {
                    " or embedded packages"
                } else {
                    ""
                }
            );
        }

        // 4. Fetch from registry.
        info!("Fetching WIT package {id} from {}", self.opts.registry_url);
        let files = self
            .fetch_client
            .fetch(spec)
            .with_context(|| format!("Failed to fetch WIT package '{id}'"))?;

        // 5. Store in cache.
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

    // --- ResolvedPackage tests ---

    #[test]
    fn resolved_package_fields() {
        let pkg = ResolvedPackage {
            id: "test:pkg@1.0.0".to_owned(),
            wit_dir: PathBuf::from("/test/path"),
            from_cache: true,
        };
        assert_eq!(pkg.id, "test:pkg@1.0.0");
        assert_eq!(pkg.wit_dir, PathBuf::from("/test/path"));
        assert!(pkg.from_cache);
    }

    #[test]
    fn resolved_package_clone() {
        let pkg = ResolvedPackage {
            id: "test:pkg@1.0.0".to_owned(),
            wit_dir: PathBuf::from("/test/path"),
            from_cache: false,
        };
        let cloned = pkg.clone();
        assert_eq!(cloned.id, pkg.id);
        assert_eq!(cloned.wit_dir, pkg.wit_dir);
        assert_eq!(cloned.from_cache, pkg.from_cache);
    }

    // --- ResolveOptions tests ---

    #[test]
    fn resolve_options_default_values() {
        let target_dir = PathBuf::from("/tmp/target");
        let opts = ResolveOptions::new(&target_dir);
        assert_eq!(opts.target_dir, target_dir);
        assert_eq!(opts.registry_url, DEFAULT_REGISTRY);
        assert!(!opts.offline);
    }

    #[test]
    fn resolve_options_custom_target_dir() {
        let custom_dir = PathBuf::from("/custom/target/dir");
        let opts = ResolveOptions::new(&custom_dir);
        assert_eq!(opts.target_dir, custom_dir);
    }

    #[test]
    fn resolve_options_clone() {
        let opts = ResolveOptions {
            target_dir: PathBuf::from("/test"),
            registry_url: "https://example.com".to_owned(),
            offline: true,
        };
        let cloned = opts.clone();
        assert_eq!(cloned.target_dir, opts.target_dir);
        assert_eq!(cloned.registry_url, opts.registry_url);
        assert_eq!(cloned.offline, opts.offline);
    }

    // --- Resolver tests ---

    #[test]
    fn resolver_new() {
        let opts = ResolveOptions::new("/tmp/target");
        let resolver = Resolver::new(opts);
        assert_eq!(resolver.opts.target_dir, PathBuf::from("/tmp/target"));
        assert_eq!(resolver.opts.registry_url, DEFAULT_REGISTRY);
        assert!(!resolver.opts.offline);
    }

    #[test]
    fn resolver_with_custom_options() {
        let opts = ResolveOptions {
            target_dir: PathBuf::from("/custom/target"),
            registry_url: "https://custom.registry.com".to_owned(),
            offline: true,
        };
        let resolver = Resolver::new(opts);
        assert_eq!(resolver.opts.target_dir, PathBuf::from("/custom/target"));
        assert_eq!(resolver.opts.registry_url, "https://custom.registry.com");
        assert!(resolver.opts.offline);
    }

    // --- Environment variable tests ---

    #[test]
    fn resolve_options_registry_from_env() {
        // Save original value
        let original = std::env::var("TAIRITSU_WIT_REGISTRY").ok();
        unsafe {
            std::env::set_var(
                "TAIRITSU_WIT_REGISTRY",
                "https://custom-registry.example.com",
            );
        }
        let opts = ResolveOptions::new("/tmp/target");
        assert_eq!(opts.registry_url, "https://custom-registry.example.com");
        assert!(!opts.offline);
        // Restore original value
        match original {
            Some(v) => unsafe { std::env::set_var("TAIRITSU_WIT_REGISTRY", v) },
            None => unsafe { std::env::remove_var("TAIRITSU_WIT_REGISTRY") },
        }
    }

    #[test]
    fn resolve_options_offline_from_env_true() {
        let original = std::env::var("TAIRITSU_WIT_OFFLINE").ok();
        unsafe {
            std::env::set_var("TAIRITSU_WIT_OFFLINE", "1");
        }
        let opts = ResolveOptions::new("/tmp/target");
        assert_eq!(opts.registry_url, DEFAULT_REGISTRY);
        assert!(opts.offline);
        match original {
            Some(v) => unsafe { std::env::set_var("TAIRITSU_WIT_OFFLINE", v) },
            None => unsafe { std::env::remove_var("TAIRITSU_WIT_OFFLINE") },
        }
    }

    #[test]
    fn resolve_options_offline_from_env_true_case_insensitive() {
        let original = std::env::var("TAIRITSU_WIT_OFFLINE").ok();

        unsafe {
            std::env::set_var("TAIRITSU_WIT_OFFLINE", "TRUE");
        }
        let opts1 = ResolveOptions::new("/tmp/target");
        assert!(opts1.offline);

        unsafe {
            std::env::set_var("TAIRITSU_WIT_OFFLINE", "true");
        }
        let opts2 = ResolveOptions::new("/tmp/target");
        assert!(opts2.offline);

        match original {
            Some(v) => unsafe { std::env::set_var("TAIRITSU_WIT_OFFLINE", v) },
            None => unsafe { std::env::remove_var("TAIRITSU_WIT_OFFLINE") },
        }
    }

    #[test]
    fn resolve_options_offline_from_env_false() {
        let original = std::env::var("TAIRITSU_WIT_OFFLINE").ok();

        unsafe {
            std::env::set_var("TAIRITSU_WIT_OFFLINE", "0");
        }
        let opts1 = ResolveOptions::new("/tmp/target");
        assert!(!opts1.offline);

        unsafe {
            std::env::set_var("TAIRITSU_WIT_OFFLINE", "false");
        }
        let opts2 = ResolveOptions::new("/tmp/target");
        assert!(!opts2.offline);

        match original {
            Some(v) => unsafe { std::env::set_var("TAIRITSU_WIT_OFFLINE", v) },
            None => unsafe { std::env::remove_var("TAIRITSU_WIT_OFFLINE") },
        }
    }

    #[test]
    fn resolve_options_both_env_vars() {
        let original_reg = std::env::var("TAIRITSU_WIT_REGISTRY").ok();
        let original_off = std::env::var("TAIRITSU_WIT_OFFLINE").ok();

        unsafe {
            std::env::set_var("TAIRITSU_WIT_REGISTRY", "https://my-registry.com");
            std::env::set_var("TAIRITSU_WIT_OFFLINE", "1");
        }
        let opts = ResolveOptions::new("/tmp/target");
        assert_eq!(opts.registry_url, "https://my-registry.com");
        assert!(opts.offline);

        match original_reg {
            Some(v) => unsafe { std::env::set_var("TAIRITSU_WIT_REGISTRY", v) },
            None => unsafe { std::env::remove_var("TAIRITSU_WIT_REGISTRY") },
        }
        match original_off {
            Some(v) => unsafe { std::env::set_var("TAIRITSU_WIT_OFFLINE", v) },
            None => unsafe { std::env::remove_var("TAIRITSU_WIT_OFFLINE") },
        }
    }
}
