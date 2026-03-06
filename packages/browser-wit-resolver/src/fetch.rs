//! HTTP fetch client for downloading WIT packages from the registry.
//!
//! The actual HTTP call is gated behind the `fetch` feature flag.
//! When the flag is disabled the client returns a "not available" error,
//! directing users to run the fetch subcommand or enable the feature.

use std::collections::HashMap;

use anyhow::{bail, Result};
#[cfg(feature = "fetch")]
use tracing::debug;

use crate::resolver::PackageSpec;

/// Client for fetching WIT package files from a remote registry.
pub struct FetchClient {
    #[allow(dead_code)]
    registry_url: String,
}

impl FetchClient {
    /// Create a new client pointing at the given registry base URL.
    pub fn new(registry_url: String) -> Self {
        Self { registry_url }
    }

    /// Fetch all WIT files for `spec` from the registry.
    ///
    /// Returns a map of `filename → bytes`.
    ///
    /// # Behaviour without the `fetch` feature
    /// Returns an error instructing the user to enable the feature or use the
    /// CLI fetch command.
    pub fn fetch(&self, spec: &PackageSpec) -> Result<HashMap<String, Vec<u8>>> {
        #[cfg(not(feature = "fetch"))]
        {
            bail!(
                "Live network fetch is not compiled in.\n\
                 Enable the `fetch` feature or run `tairitsu wit fetch {}` \
                 to populate the local cache.",
                spec.id()
            );
        }

        #[cfg(feature = "fetch")]
        {
            self.fetch_impl(spec)
        }
    }

    /// Real HTTP implementation (compiled only with the `fetch` feature).
    #[cfg(feature = "fetch")]
    fn fetch_impl(&self, spec: &PackageSpec) -> Result<HashMap<String, Vec<u8>>> {
        use anyhow::Context;

        let base = format!(
            "{}/{}/{}/{}",
            self.registry_url.trim_end_matches('/'),
            spec.namespace,
            spec.name,
            spec.version,
        );

        // Fetch the manifest first to learn which files are available.
        let manifest_url = format!("{base}/manifest.json");
        debug!("GET {manifest_url}");
        let manifest_resp = reqwest::blocking::get(&manifest_url)
            .with_context(|| format!("Fetching manifest from {manifest_url}"))?
            .error_for_status()
            .with_context(|| format!("HTTP error fetching {manifest_url}"))?;

        let manifest: crate::cache::CacheManifest = manifest_resp
            .json()
            .with_context(|| "Parsing registry manifest JSON")?;

        // Fetch each WIT file listed in the manifest.
        let mut files = HashMap::new();
        for filename in manifest.file_hashes.keys() {
            let url = format!("{base}/{filename}");
            debug!("GET {url}");
            let bytes = reqwest::blocking::get(&url)
                .with_context(|| format!("Fetching {url}"))?
                .error_for_status()
                .with_context(|| format!("HTTP error fetching {url}"))?
                .bytes()
                .with_context(|| format!("Reading body of {url}"))?
                .to_vec();
            files.insert(filename.clone(), bytes);
        }

        Ok(files)
    }
}
