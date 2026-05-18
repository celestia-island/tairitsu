//! Handle caching for performance optimization.
//!
//! Caches opaque handles (u64) to avoid repeated WIT calls for frequently
//! accessed DOM objects like style declarations.

use std::cell::RefCell;
use std::collections::HashMap;

/// Cache for element-related handles to avoid repeated WIT calls.
///
/// Stores handles for:
/// - Style declarations (CSSStyleDeclaration)
/// - Computed styles (future)
/// - Other element-specific resources (future)
#[derive(Debug)]
pub struct HandleCache {
    /// Element handle -> Style handle mapping
    style_handles: RefCell<HashMap<u64, u64>>,
    /// Cache statistics
    hits: RefCell<u64>,
    misses: RefCell<u64>,
}

impl HandleCache {
    /// Create a new empty handle cache.
    pub fn new() -> Self {
        Self {
            style_handles: RefCell::new(HashMap::new()),
            hits: RefCell::new(0),
            misses: RefCell::new(0),
        }
    }

    /// Get the cached style handle for an element.
    ///
    /// Returns `Some(handle)` if cached, `None` otherwise.
    pub fn get_style_handle(&self, element: u64) -> Option<u64> {
        if let Some(handle) = self.style_handles.borrow().get(&element) {
            *self.hits.borrow_mut() += 1;
            Some(*handle)
        } else {
            *self.misses.borrow_mut() += 1;
            None
        }
    }

    /// Cache a style handle for an element.
    pub fn set_style_handle(&self, element: u64, style_handle: u64) {
        self.style_handles
            .borrow_mut()
            .insert(element, style_handle);
    }

    /// Invalidate the cached style handle for an element.
    ///
    /// Call this when the element's style declaration is replaced or
    /// the element is removed from the DOM.
    pub fn invalidate_style_handle(&self, element: u64) {
        self.style_handles.borrow_mut().remove(&element);
    }

    /// Clear all cached handles.
    ///
    /// Call this when doing a large-scale DOM update or navigation.
    pub fn clear(&self) {
        self.style_handles.borrow_mut().clear();
    }

    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        let hits = *self.hits.borrow();
        let misses = *self.misses.borrow();
        let total = hits + misses;
        let hit_rate = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        CacheStats {
            hits,
            misses,
            total,
            hit_rate,
            size: self.style_handles.borrow().len(),
        }
    }
}

impl Default for HandleCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for HandleCache {
    fn clone(&self) -> Self {
        Self {
            style_handles: RefCell::new(self.style_handles.borrow().clone()),
            hits: RefCell::new(*self.hits.borrow()),
            misses: RefCell::new(*self.misses.borrow()),
        }
    }
}

/// Cache statistics.
#[derive(Debug, Clone, Copy)]
pub struct CacheStats {
    /// Number of cache hits
    pub hits: u64,
    /// Number of cache misses
    pub misses: u64,
    /// Total cache lookups
    pub total: u64,
    /// Hit rate as percentage (0.0 - 100.0)
    pub hit_rate: f64,
    /// Number of entries in the cache
    pub size: usize,
}

#[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
thread_local! {
    /// Thread-local handle cache instance.
    ///
    /// Uses thread_local storage for WASM compatibility and to avoid
    /// synchronization overhead.
    pub static HANDLE_CACHE: HandleCache = HandleCache::new();
}

#[cfg(all(feature = "wit-bindings", target_family = "wasm"))]
impl HandleCache {
    /// Get the thread-local handle cache.
    pub fn get() -> Self {
        HANDLE_CACHE.with(|cache| cache.clone())
    }

    /// Run a function with the thread-local cache.
    pub fn with<R>(f: impl FnOnce(&HandleCache) -> R) -> R {
        HANDLE_CACHE.with(f)
    }
}

#[cfg(not(all(feature = "wit-bindings", target_family = "wasm")))]
impl HandleCache {
    /// On non-wasm32 targets, returns a no-op cache.
    pub fn get() -> Self {
        Self::new()
    }

    /// On non-wasm32 targets, runs the function with a no-op cache.
    pub fn with<R>(f: impl FnOnce(&HandleCache) -> R) -> R {
        f(&Self::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_miss_returns_none() {
        let cache = HandleCache::new();
        assert_eq!(cache.get_style_handle(42), None);
    }

    #[test]
    fn test_cache_hit_after_set() {
        let cache = HandleCache::new();
        cache.set_style_handle(42, 100);
        assert_eq!(cache.get_style_handle(42), Some(100));
    }

    #[test]
    fn test_cache_invalidate() {
        let cache = HandleCache::new();
        cache.set_style_handle(42, 100);
        assert_eq!(cache.get_style_handle(42), Some(100));

        cache.invalidate_style_handle(42);
        assert_eq!(cache.get_style_handle(42), None);
    }

    #[test]
    fn test_cache_clear() {
        let cache = HandleCache::new();
        cache.set_style_handle(42, 100);
        cache.set_style_handle(43, 101);

        cache.clear();

        assert_eq!(cache.get_style_handle(42), None);
        assert_eq!(cache.get_style_handle(43), None);
    }

    #[test]
    fn test_cache_stats() {
        let cache = HandleCache::new();

        // Initial stats
        let stats = cache.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.total, 0);
        assert_eq!(stats.size, 0);

        // Add entry
        cache.set_style_handle(42, 100);

        // Hit
        cache.get_style_handle(42);

        // Miss
        cache.get_style_handle(99);

        let stats = cache.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.total, 2);
        assert_eq!(stats.size, 1);
        assert!((stats.hit_rate - 50.0).abs() < 0.01); // 50% hit rate
    }

    #[test]
    fn test_cache_multiple_elements() {
        let cache = HandleCache::new();
        cache.set_style_handle(1, 10);
        cache.set_style_handle(2, 20);
        cache.set_style_handle(3, 30);

        assert_eq!(cache.get_style_handle(1), Some(10));
        assert_eq!(cache.get_style_handle(2), Some(20));
        assert_eq!(cache.get_style_handle(3), Some(30));
        assert_eq!(cache.get_style_handle(4), None);

        let stats = cache.stats();
        assert_eq!(stats.size, 3);
        assert_eq!(stats.hits, 3);
        assert_eq!(stats.misses, 1);
    }

    #[test]
    fn test_cache_overwrite() {
        let cache = HandleCache::new();
        cache.set_style_handle(42, 100);
        assert_eq!(cache.get_style_handle(42), Some(100));

        // Overwrite with new handle
        cache.set_style_handle(42, 200);
        assert_eq!(cache.get_style_handle(42), Some(200));

        let stats = cache.stats();
        assert_eq!(stats.size, 1); // Still only one entry
    }
}
