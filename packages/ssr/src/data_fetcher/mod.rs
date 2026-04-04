//! Server-side data fetching utilities for Tairitsu
//!
//! This crate provides async data fetching utilities with caching support,
//! usable on both server and client platforms.

#![warn(missing_docs)]

use std::{collections::HashMap, time::Duration};

// Re-exports for convenience
pub use cache::Cache;
pub use error::FetchError;
pub use fetcher::Fetcher;
pub use http_fetcher::HttpFetcher;

mod cache;
mod error;
mod fetcher;
mod hooks;
pub mod http_fetcher;

pub use hooks::{use_fetch, use_fetch_json, use_fetch_json_with_fetcher, use_fetch_with_fetcher, use_lazy_fetch};

/// A resource that represents the state of an async data fetch
///
/// This is used by the `use_fetch` hook to track loading states,
/// data, and errors.
#[derive(Clone, Debug)]
pub enum Resource<T> {
    /// The fetch is in progress
    Loading,
    /// The fetch completed successfully with data
    Success(T),
    /// The fetch failed with an error
    Error(String),
}

impl<T> Resource<T> {
    /// Returns true if the resource is currently loading
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    /// Returns true if the resource has successfully loaded
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success(_))
    }

    /// Returns true if the resource has errored
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    /// Get a reference to the success value, if any
    pub fn data(&self) -> Option<&T> {
        match self {
            Self::Success(data) => Some(data),
            _ => None,
        }
    }

    /// Get the error message, if any
    pub fn error(&self) -> Option<&str> {
        match self {
            Self::Error(msg) => Some(msg),
            _ => None,
        }
    }

    /// Map the inner value using a function
    pub fn map<U, F>(self, f: F) -> Resource<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Self::Loading => Resource::Loading,
            Self::Success(data) => Resource::Success(f(data)),
            Self::Error(msg) => Resource::Error(msg),
        }
    }
}

/// Configuration for fetch operations
#[derive(Clone, Debug)]
pub struct FetchConfig {
    /// Request timeout duration
    pub timeout: Duration,
    /// Whether to cache responses
    pub cache: bool,
    /// Cache TTL (time-to-live)
    pub cache_ttl: Duration,
    /// Custom headers to include in requests
    pub headers: HashMap<String, String>,
}

impl Default for FetchConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            cache: true,
            cache_ttl: Duration::from_secs(300), // 5 minutes
            headers: HashMap::new(),
        }
    }
}

impl FetchConfig {
    /// Create a new fetch config with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the request timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Enable or disable caching
    pub fn with_cache(mut self, cache: bool) -> Self {
        self.cache = cache;
        self
    }

    /// Set the cache TTL
    pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    /// Add a custom header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resource_states() {
        let resource: Resource<i32> = Resource::Loading;
        assert!(resource.is_loading());
        assert!(!resource.is_success());
        assert!(!resource.is_error());

        let resource = Resource::Success(42);
        assert!(!resource.is_loading());
        assert!(resource.is_success());
        assert!(!resource.is_error());
        assert_eq!(resource.data(), Some(&42));

        let resource: Resource<i32> = Resource::Error("failed".to_string());
        assert!(!resource.is_loading());
        assert!(!resource.is_success());
        assert!(resource.is_error());
        assert_eq!(resource.error(), Some("failed"));
    }

    #[test]
    fn test_resource_map() {
        let resource = Resource::Success(42);
        let mapped = resource.map(|x| x * 2);
        assert_eq!(mapped.data(), Some(&84));

        let resource: Resource<i32> = Resource::Loading;
        let mapped = resource.map(|x| x * 2);
        assert!(mapped.is_loading());

        let resource: Resource<i32> = Resource::Error("failed".to_string());
        let mapped: Resource<i32> = resource.map(|x: i32| x * 2);
        assert!(mapped.is_error());
    }

    #[test]
    fn test_fetch_config_builder() {
        let config = FetchConfig::new()
            .with_timeout(Duration::from_secs(60))
            .with_cache(false)
            .with_cache_ttl(Duration::from_secs(120))
            .with_header("Authorization", "Bearer token");

        assert_eq!(config.timeout, Duration::from_secs(60));
        assert!(!config.cache);
        assert_eq!(config.cache_ttl, Duration::from_secs(120));
        assert_eq!(
            config.headers.get("Authorization"),
            Some(&"Bearer token".to_string())
        );
    }
}
