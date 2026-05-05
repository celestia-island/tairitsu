//! HTTP fetcher implementation

#[cfg(feature = "data-fetcher")]
use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use super::{FetchConfig, cache::Cache, error::FetchError, fetcher::Fetcher};

/// HTTP fetcher for making HTTP requests with caching support
#[derive(Clone)]
pub struct HttpFetcher {
    /// HTTP client (only available on server side)
    #[cfg(feature = "data-fetcher")]
    client: Option<reqwest::Client>,
    /// Cache for responses
    cache: Arc<Cache>,
    /// Fetch configuration (only available on server side)
    #[cfg(feature = "data-fetcher")]
    config: FetchConfig,
}

impl HttpFetcher {
    /// Create a new HTTP fetcher with default configuration
    pub fn new() -> Self {
        Self::with_config(FetchConfig::default())
    }

    /// Create a new HTTP fetcher with the given configuration
    pub fn with_config(config: FetchConfig) -> Self {
        #[cfg(feature = "data-fetcher")]
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .ok();

        #[cfg(not(feature = "data-fetcher"))]
        let _ = config; // Suppress unused warning

        Self {
            #[cfg(feature = "data-fetcher")]
            client,
            #[cfg(feature = "data-fetcher")]
            cache: Arc::new(Cache::new(config.cache_ttl)),
            #[cfg(not(feature = "data-fetcher"))]
            cache: Arc::new(Cache::new(FetchConfig::default().cache_ttl)),
            #[cfg(feature = "data-fetcher")]
            config,
        }
    }

    /// Create a new HTTP fetcher with a custom cache
    #[allow(unused_variables)]
    pub fn with_cache(config: FetchConfig, cache: Arc<Cache>) -> Self {
        #[cfg(feature = "data-fetcher")]
        let client = reqwest::Client::builder()
            .timeout(config.timeout)
            .build()
            .ok();

        Self {
            #[cfg(feature = "data-fetcher")]
            client,
            cache,
            #[cfg(feature = "data-fetcher")]
            config,
        }
    }

    /// Get a reference to the cache
    pub fn cache(&self) -> &Cache {
        &self.cache
    }

    /// Get mutable reference to the cache
    pub fn cache_mut(&mut self) -> &mut Cache {
        Arc::make_mut(&mut self.cache)
    }

    /// Create a cache key for a request
    #[allow(dead_code)]
    fn cache_key(method: &str, url: &str, body: &[u8]) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        method.hash(&mut hasher);
        url.hash(&mut hasher);
        if !body.is_empty() {
            body.hash(&mut hasher);
        }
        format!("{}:{:x}", method, hasher.finish())
    }

    /// Build headers for the request
    #[cfg(feature = "data-fetcher")]
    fn build_headers(&self, additional: &HashMap<String, String>) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();

        // Add default headers
        headers.insert(
            reqwest::header::ACCEPT,
            reqwest::header::HeaderValue::from_static("application/json"),
        );

        // Add custom headers from config
        #[cfg(feature = "data-fetcher")]
        for (key, value) in &self.config.headers {
            if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(key.as_bytes())
                && let Ok(header_value) = reqwest::header::HeaderValue::from_str(value)
            {
                headers.insert(header_name, header_value);
            }
        }

        // Add additional headers
        for (key, value) in additional {
            if let Ok(header_name) = reqwest::header::HeaderName::from_bytes(key.as_bytes())
                && let Ok(header_value) = reqwest::header::HeaderValue::from_str(value)
            {
                headers.insert(header_name, header_value);
            }
        }

        headers
    }
}

impl Default for HttpFetcher {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Fetcher for HttpFetcher {
    #[allow(unused_variables)]
    async fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        // Check cache first
        #[cfg(feature = "data-fetcher")]
        #[cfg(feature = "data-fetcher")]
        if self.config.cache {
            let cache_key = Self::cache_key("GET", url, &[]);
            if let Some(data) = self.cache.get(&cache_key) {
                return Ok(data);
            }
        }

        #[cfg(feature = "data-fetcher")]
        {
            let client = self
                .client
                .as_ref()
                .ok_or_else(|| FetchError::network("HTTP client not available"))?;

            let headers = self.build_headers(&HashMap::new());

            let response = client
                .get(url)
                .headers(headers)
                .send()
                .await
                .map_err(|e| FetchError::network(e.to_string()))?;

            let status = response.status();
            let bytes = response
                .bytes()
                .await
                .map_err(|e| FetchError::network(e.to_string()))?;

            if !status.is_success() {
                return Err(FetchError::http(
                    status.as_u16(),
                    String::from_utf8_lossy(&bytes).to_string(),
                ));
            }

            let data = bytes.to_vec();

            // Store in cache
            #[cfg(feature = "data-fetcher")]
            if self.config.cache {
                let cache_key = Self::cache_key("GET", url, &[]);
                self.cache.insert(cache_key, data.clone());
            }

            Ok(data)
        }

        #[cfg(not(feature = "data-fetcher"))]
        {
            Err(FetchError::network(
                "HTTP client not available (server feature required)",
            ))
        }
    }

    #[allow(unused_variables)]
    async fn post(&self, url: &str, body: Vec<u8>) -> Result<Vec<u8>, FetchError> {
        // Check cache first (for POST with same body)
        #[cfg(feature = "data-fetcher")]
        if self.config.cache {
            let cache_key = Self::cache_key("POST", url, &body);
            if let Some(data) = self.cache.get(&cache_key) {
                return Ok(data);
            }
        }

        #[cfg(feature = "data-fetcher")]
        {
            let client = self
                .client
                .as_ref()
                .ok_or_else(|| FetchError::network("HTTP client not available"))?;

            let headers = self.build_headers(&HashMap::new());

            let response = client
                .post(url)
                .headers(headers)
                .body(body)
                .send()
                .await
                .map_err(|e| FetchError::network(e.to_string()))?;

            let status = response.status();
            let bytes = response
                .bytes()
                .await
                .map_err(|e| FetchError::network(e.to_string()))?;

            if !status.is_success() {
                return Err(FetchError::http(
                    status.as_u16(),
                    String::from_utf8_lossy(&bytes).to_string(),
                ));
            }

            let data = bytes.to_vec();

            // Store in cache
            #[cfg(feature = "data-fetcher")]
            if self.config.cache {
                let cache_key = Self::cache_key("POST", url, &[]);
                self.cache.insert(cache_key, data.clone());
            }

            Ok(data)
        }

        #[cfg(not(feature = "data-fetcher"))]
        {
            Err(FetchError::network(
                "HTTP client not available (server feature required)",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_http_fetcher_new() {
        let fetcher = HttpFetcher::new();
        assert_eq!(fetcher.cache.len(), 0);
    }

    #[test]
    fn test_http_fetcher_with_config() {
        let config = FetchConfig::new()
            .with_timeout(Duration::from_secs(60))
            .with_cache(false);

        let _fetcher = HttpFetcher::with_config(config);
        #[cfg(feature = "data-fetcher")]
        {
            assert_eq!(_fetcher.config.timeout, Duration::from_secs(60));
            assert!(!_fetcher.config.cache);
        }
    }

    #[test]
    fn test_cache_key_generation() {
        let key1 = HttpFetcher::cache_key("GET", "http://example.com", &[]);
        let key2 = HttpFetcher::cache_key("GET", "http://example.com", &[]);
        let key3 = HttpFetcher::cache_key("POST", "http://example.com", &[]);
        let key4 = HttpFetcher::cache_key("GET", "http://example.com", b"data");

        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
        assert_ne!(key1, key4);
    }

    #[tokio::test]
    #[cfg(feature = "data-fetcher")]
    async fn test_http_fetcher_cache_integration() {
        // This test requires a mock server or would be integration test
        // For now, we test the cache integration logic

        let cache = Arc::new(Cache::new(Duration::from_secs(60)));
        cache.insert("test:key", b"cached data".to_vec());

        let fetcher = HttpFetcher::with_cache(FetchConfig::default(), cache);

        // Verify cache is accessible
        assert_eq!(fetcher.cache().len(), 1);
        assert_eq!(
            fetcher.cache().get("test:key"),
            Some(b"cached data".to_vec())
        );
    }

    #[tokio::test]
    #[cfg(feature = "data-fetcher")]
    async fn test_http_fetcher_no_client_error() {
        // Create a fetcher without a valid client
        let fetcher = HttpFetcher {
            client: None,
            cache: Arc::new(Cache::new(Duration::from_secs(60))),
            config: FetchConfig::default(),
        };

        let result = fetcher.get("http://example.com").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), FetchError::Network(_)));
    }
}
