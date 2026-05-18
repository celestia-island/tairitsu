//! In-memory cache for fetch responses

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// Cache entry with expiration time
#[derive(Debug, Clone)]
struct CacheEntry {
    /// The cached data
    data: Vec<u8>,
    /// When this entry expires
    expires_at: Instant,
    /// When this entry was created
    created_at: Instant,
    /// Number of times this entry has been accessed
    access_count: u64,
}

impl CacheEntry {
    /// Create a new cache entry
    fn new(data: Vec<u8>, ttl: Duration) -> Self {
        let now = Instant::now();
        Self {
            data,
            expires_at: now + ttl,
            created_at: now,
            access_count: 0,
        }
    }

    /// Check if this entry has expired
    fn is_expired(&self) -> bool {
        Instant::now() > self.expires_at
    }

    /// Record an access to this entry
    fn record_access(&mut self) {
        self.access_count += 1;
    }

    /// Get the age of this entry
    fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
}

/// Thread-safe in-memory cache for fetch responses
#[derive(Debug, Clone)]
pub struct Cache {
    inner: Arc<RwLock<InnerCache>>,
}

#[derive(Debug)]
struct InnerCache {
    entries: HashMap<String, CacheEntry>,
    default_ttl: Duration,
    max_entries: usize,
}

impl Cache {
    /// Create a new cache with the given default TTL
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            inner: Arc::new(RwLock::new(InnerCache {
                entries: HashMap::new(),
                default_ttl,
                max_entries: 1000,
            })),
        }
    }

    /// Create a new cache with a custom max entry limit
    pub fn with_limits(default_ttl: Duration, max_entries: usize) -> Self {
        Self {
            inner: Arc::new(RwLock::new(InnerCache {
                entries: HashMap::new(),
                default_ttl,
                max_entries,
            })),
        }
    }

    /// Get a cached value if it exists and hasn't expired
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let mut cache = self.inner.write().ok()?;
        let entry = cache.entries.get_mut(key)?;

        if entry.is_expired() {
            cache.entries.remove(key);
            return None;
        }

        entry.record_access();
        Some(entry.data.clone())
    }

    /// Insert a value into the cache with the default TTL
    pub fn insert(&self, key: impl Into<String>, data: Vec<u8>) {
        let key = key.into();
        let mut cache = self.inner.write().unwrap();
        let ttl = cache.default_ttl;

        // Evict expired entries and enforce size limit
        Self::evict_expired(&mut cache.entries);
        if cache.entries.len() >= cache.max_entries {
            Self::evict_lru(&mut cache.entries);
        }

        cache.entries.insert(key, CacheEntry::new(data, ttl));
    }

    /// Insert a value with a custom TTL
    pub fn insert_with_ttl(&self, key: impl Into<String>, data: Vec<u8>, ttl: Duration) {
        let key = key.into();
        let mut cache = self.inner.write().unwrap();

        // Evict expired entries and enforce size limit
        Self::evict_expired(&mut cache.entries);
        if cache.entries.len() >= cache.max_entries {
            Self::evict_lru(&mut cache.entries);
        }

        cache.entries.insert(key, CacheEntry::new(data, ttl));
    }

    /// Remove a specific entry from the cache
    pub fn remove(&self, key: &str) -> bool {
        let mut cache = self.inner.write().unwrap();
        cache.entries.remove(key).is_some()
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        let mut cache = self.inner.write().unwrap();
        cache.entries.clear();
    }

    /// Get the number of entries in the cache
    pub fn len(&self) -> usize {
        let cache = self.inner.read().unwrap();
        cache.entries.len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get statistics about the cache
    pub fn stats(&self) -> CacheStats {
        let cache = self.inner.read().unwrap();
        let mut total_access_count = 0;
        let mut total_age = Duration::ZERO;

        for entry in cache.entries.values() {
            total_access_count += entry.access_count;
            total_age += entry.age();
        }

        let avg_age = if cache.entries.is_empty() {
            Duration::ZERO
        } else {
            total_age / cache.entries.len() as u32
        };

        CacheStats {
            entries: cache.entries.len(),
            max_entries: cache.max_entries,
            total_access_count,
            avg_age,
        }
    }

    /// Evict expired entries
    fn evict_expired(entries: &mut HashMap<String, CacheEntry>) {
        entries.retain(|_, entry| !entry.is_expired());
    }

    /// Evict the least recently used entry
    fn evict_lru(entries: &mut HashMap<String, CacheEntry>) {
        let lru_key = entries
            .iter()
            .min_by_key(|(_, entry)| entry.access_count)
            .map(|(key, _)| key.clone());

        if let Some(key) = lru_key {
            entries.remove(&key);
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of entries in the cache
    pub entries: usize,
    /// Maximum number of entries allowed
    pub max_entries: usize,
    /// Total access count across all entries
    pub total_access_count: u64,
    /// Average age of cache entries
    pub avg_age: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_insert_get() {
        let cache = Cache::new(Duration::from_secs(60));

        assert!(cache.get("key1").is_none());

        cache.insert("key1", b"data1".to_vec());
        assert_eq!(cache.get("key1"), Some(b"data1".to_vec()));

        cache.insert("key2", b"data2".to_vec());
        assert_eq!(cache.get("key2"), Some(b"data2".to_vec()));
    }

    #[test]
    fn test_cache_expiration() {
        let cache = Cache::new(Duration::from_millis(100));

        cache.insert("key1", b"data1".to_vec());
        assert_eq!(cache.get("key1"), Some(b"data1".to_vec()));

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(150));
        assert!(cache.get("key1").is_none());
    }

    #[test]
    fn test_cache_remove() {
        let cache = Cache::new(Duration::from_secs(60));

        cache.insert("key1", b"data1".to_vec());
        assert_eq!(cache.get("key1"), Some(b"data1".to_vec()));

        assert!(cache.remove("key1"));
        assert!(cache.get("key1").is_none());
        assert!(!cache.remove("nonexistent"));
    }

    #[test]
    fn test_cache_clear() {
        let cache = Cache::new(Duration::from_secs(60));

        cache.insert("key1", b"data1".to_vec());
        cache.insert("key2", b"data2".to_vec());

        assert_eq!(cache.len(), 2);
        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_cache_custom_ttl() {
        let cache = Cache::new(Duration::from_secs(60));

        // Custom TTL of 50ms
        cache.insert_with_ttl("key1", b"data1".to_vec(), Duration::from_millis(50));
        assert_eq!(cache.get("key1"), Some(b"data1".to_vec()));

        std::thread::sleep(Duration::from_millis(75));
        assert!(cache.get("key1").is_none());
    }

    #[test]
    fn test_cache_stats() {
        let cache = Cache::with_limits(Duration::from_secs(60), 10);

        cache.insert("key1", b"data1".to_vec());
        cache.insert("key2", b"data2".to_vec());

        // Access key1 twice
        cache.get("key1");
        cache.get("key1");

        let stats = cache.stats();
        assert_eq!(stats.entries, 2);
        assert_eq!(stats.max_entries, 10);
        assert_eq!(stats.total_access_count, 2);
    }

    #[test]
    fn test_cache_lru_eviction() {
        let cache = Cache::with_limits(Duration::from_secs(60), 3);

        cache.insert("key1", b"data1".to_vec());
        cache.insert("key2", b"data2".to_vec());
        cache.insert("key3", b"data3".to_vec());

        // Access key1 and key2, but not key3
        cache.get("key1");
        cache.get("key2");

        // Insert key4, should evict key3 (least accessed)
        cache.insert("key4", b"data4".to_vec());

        assert!(cache.get("key1").is_some());
        assert!(cache.get("key2").is_some());
        assert!(cache.get("key3").is_none()); // Evicted
        assert!(cache.get("key4").is_some());
    }
}
